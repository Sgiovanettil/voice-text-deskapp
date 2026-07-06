//! Corte por detección de voz (ADR-0011): Silero VAD vía
//! `voice_activity_detector` sobre chunks de 512 samples (32 ms) a 16 kHz.
//!
//! `VadGate` vive dentro del hilo de captura: recibe los mismos buffers
//! interleaved nativos que el buffer de grabación, los baja a mono 16 kHz con
//! un resampler lineal barato (suficiente para VAD; el audio que se
//! transcribe sigue pasando por rubato en `stop()`) y acumula el silencio
//! continuo. Solo corta el final: el corte se arma recién cuando hubo habla,
//! así una pausa inicial para pensar no cierra el dictado.

use voice_activity_detector::VoiceActivityDetector;

use super::TARGET_SAMPLE_RATE;

/// Tamaño de chunk nativo del modelo Silero para 16 kHz.
const CHUNK_SAMPLES: usize = 512;
/// Duración de cada chunk en ms (512 / 16 kHz).
const CHUNK_MS: f32 = CHUNK_SAMPLES as f32 * 1_000.0 / TARGET_SAMPLE_RATE as f32;

/// Resultado de alimentar un buffer al gate.
#[derive(Debug, Default, PartialEq)]
pub struct VadUpdate {
    /// `Some(estado)` si el estado hablando/en-silencio cambió en este buffer.
    pub speaking_changed: Option<bool>,
    /// `Some(ms)` si el silencio sostenido alcanzó el hangover (se emite una
    /// sola vez por grabación).
    pub silence_cut_ms: Option<u64>,
}

type Predictor = Box<dyn FnMut(&[i16]) -> f32 + Send>;

pub struct VadGate {
    predictor: Predictor,
    threshold: f32,
    hangover_ms: u64,
    channels: usize,
    /// Avance en samples nativos por cada sample de 16 kHz.
    step: f32,
    /// Posición fraccional dentro de `pending`.
    cursor: f32,
    /// Mono a frecuencia nativa aún no consumido por el resampler.
    pending: Vec<f32>,
    /// Chunk de 16 kHz i16 en construcción (hasta 512).
    chunk: Vec<i16>,
    silence_ms: f32,
    speaking: bool,
    heard_speech: bool,
    fired: bool,
}

impl VadGate {
    /// Crea el gate con el modelo Silero embebido. Falla solo si ONNX Runtime
    /// no puede inicializarse; el llamador degrada a grabar sin VAD.
    pub fn new(
        from_rate: u32,
        channels: usize,
        threshold: f32,
        hangover_ms: u64,
    ) -> Result<Self, String> {
        let mut detector = VoiceActivityDetector::builder()
            .sample_rate(TARGET_SAMPLE_RATE as i64)
            .chunk_size(CHUNK_SAMPLES)
            .build()
            .map_err(|e| e.to_string())?;
        let predictor: Predictor =
            Box::new(move |chunk: &[i16]| detector.predict(chunk.iter().copied()));
        Ok(Self::with_predictor(
            predictor,
            from_rate,
            channels,
            threshold,
            hangover_ms,
        ))
    }

    fn with_predictor(
        predictor: Predictor,
        from_rate: u32,
        channels: usize,
        threshold: f32,
        hangover_ms: u64,
    ) -> Self {
        Self {
            predictor,
            threshold,
            hangover_ms,
            channels: channels.max(1),
            step: from_rate as f32 / TARGET_SAMPLE_RATE as f32,
            cursor: 0.0,
            pending: Vec::new(),
            chunk: Vec::with_capacity(CHUNK_SAMPLES),
            silence_ms: 0.0,
            speaking: false,
            heard_speech: false,
            fired: false,
        }
    }

    /// Alimenta un buffer interleaved nativo y devuelve los cambios de estado.
    pub fn feed(&mut self, interleaved: &[f32]) -> VadUpdate {
        if self.fired {
            return VadUpdate::default();
        }
        // Downmix a mono y acumular a frecuencia nativa.
        if self.channels <= 1 {
            self.pending.extend_from_slice(interleaved);
        } else {
            self.pending.extend(
                interleaved
                    .chunks_exact(self.channels)
                    .map(|f| f.iter().sum::<f32>() / self.channels as f32),
            );
        }

        let mut update = VadUpdate::default();
        // Resample lineal a 16 kHz consumiendo `pending`, en chunks de 512.
        while (self.cursor as usize) + 1 < self.pending.len() {
            let i = self.cursor as usize;
            let frac = self.cursor - i as f32;
            let sample = self.pending[i] * (1.0 - frac) + self.pending[i + 1] * frac;
            self.chunk
                .push((sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16);
            self.cursor += self.step;

            if self.chunk.len() == CHUNK_SAMPLES {
                self.process_chunk(&mut update);
                if update.silence_cut_ms.is_some() {
                    break;
                }
            }
        }
        // Descartar lo ya consumido, conservando la fase fraccional.
        let consumed = (self.cursor as usize).min(self.pending.len());
        self.pending.drain(..consumed);
        self.cursor -= consumed as f32;
        update
    }

    fn process_chunk(&mut self, update: &mut VadUpdate) {
        let probability = (self.predictor)(&self.chunk);
        self.chunk.clear();

        let is_speech = probability >= self.threshold;
        if is_speech {
            self.heard_speech = true;
            self.silence_ms = 0.0;
        } else if self.heard_speech {
            self.silence_ms += CHUNK_MS;
        }
        if is_speech != self.speaking {
            self.speaking = is_speech;
            update.speaking_changed = Some(is_speech);
        }
        if self.heard_speech && !self.fired && self.silence_ms >= self.hangover_ms as f32 {
            self.fired = true;
            update.silence_cut_ms = Some(self.silence_ms as u64);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Gate de prueba: el predictor lee de una secuencia fija de
    /// probabilidades (una por chunk de 512).
    fn gate_with_probs(probs: Vec<f32>, hangover_ms: u64) -> VadGate {
        let mut iter = probs.into_iter();
        VadGate::with_predictor(
            Box::new(move |_| iter.next().unwrap_or(0.0)),
            TARGET_SAMPLE_RATE,
            1,
            0.5,
            hangover_ms,
        )
    }

    fn feed_chunks(gate: &mut VadGate, n: usize) -> Vec<VadUpdate> {
        // A 16 kHz mono el resampler es ~identidad: 512 samples ≈ 1 chunk.
        // Se alimenta de a 512 + margen para el sample de interpolación.
        (0..n).map(|_| gate.feed(&vec![0.1f32; 513])).collect()
    }

    #[test]
    fn no_corta_sin_haber_oido_habla() {
        // 60 chunks de silencio ≈ 1.9 s > hangover de 1.2 s: sin habla previa
        // no debe cortar (pausa inicial para pensar).
        let mut gate = gate_with_probs(vec![0.0; 60], 1_200);
        for u in feed_chunks(&mut gate, 60) {
            assert_eq!(u.silence_cut_ms, None);
        }
    }

    #[test]
    fn corta_tras_silencio_sostenido_despues_de_hablar() {
        // 5 chunks de habla y luego silencio: corta al acumular ≥ 1.200 ms
        // (38 chunks de 32 ms).
        let mut probs = vec![0.9; 5];
        probs.extend(vec![0.0; 60]);
        let mut gate = gate_with_probs(probs, 1_200);
        let updates = feed_chunks(&mut gate, 65);
        let cut_at = updates.iter().position(|u| u.silence_cut_ms.is_some());
        let cut = cut_at.expect("debió cortar");
        assert!(cut >= 5 + 37, "cortó antes del hangover: chunk {cut}");
        // Tras el corte no vuelve a disparar.
        assert!(updates[cut + 1..]
            .iter()
            .all(|u| u.silence_cut_ms.is_none()));
    }

    #[test]
    fn habla_intermedia_reinicia_el_silencio() {
        // Habla, media pausa (20 chunks = 640 ms), habla de nuevo, y recién
        // después silencio largo: la pausa corta no debe disparar el corte.
        let mut probs = vec![0.9; 3];
        probs.extend(vec![0.0; 20]);
        probs.extend(vec![0.9; 3]);
        probs.extend(vec![0.0; 45]);
        let mut gate = gate_with_probs(probs, 1_200);
        let updates = feed_chunks(&mut gate, 71);
        let cut_at = updates
            .iter()
            .position(|u| u.silence_cut_ms.is_some())
            .expect("debió cortar al final");
        assert!(cut_at > 3 + 20 + 3, "la pausa intermedia no debía cortar");
    }

    #[test]
    fn reporta_cambios_de_estado_hablando() {
        let mut probs = vec![0.9; 2];
        probs.extend(vec![0.0; 2]);
        let mut gate = gate_with_probs(probs, 10_000);
        let updates = feed_chunks(&mut gate, 4);
        assert_eq!(updates[0].speaking_changed, Some(true));
        assert_eq!(updates[2].speaking_changed, Some(false));
    }
}
