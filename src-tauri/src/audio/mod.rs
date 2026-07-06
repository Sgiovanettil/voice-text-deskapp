//! Captura de micrófono (cpal) y resampling a 16 kHz mono. Ver docs/2-arquitectura/ARCHITECTURE.md §4.3.
//!
//! Dos capas: `convert` (funciones puras, testeables sin hardware) y
//! `Recorder` (cpal sobre el dispositivo por defecto, en su propio hilo — los
//! streams de cpal no son `Send`). El audio nunca toca disco (PRD §15.5).

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub mod vad;

/// Frecuencia objetivo del pipeline STT (§4.3).
pub const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Nombres de los dispositivos de entrada disponibles (ARCHITECTURE §4.3), en
/// el orden que reporta el host. Vacío si no hay micrófonos o el host falla.
pub fn list_input_devices() -> Vec<String> {
    cpal::default_host()
        .input_devices()
        .map(|devs| {
            devs.filter_map(|d| d.description().ok().map(|desc| desc.name().to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Resuelve el dispositivo de entrada por nombre; si el nombre no existe (o es
/// `None`), cae al default del SO — un micrófono desconectado nunca rompe la
/// captura (principio de degradación elegante).
fn select_input_device(host: &cpal::Host, name: Option<&str>) -> Option<cpal::Device> {
    if let Some(name) = name {
        if let Ok(mut devices) = host.input_devices() {
            if let Some(dev) =
                devices.find(|d| d.description().is_ok_and(|desc| desc.name() == name))
            {
                return Some(dev);
            }
        }
        tracing::warn!(
            device = name,
            "micrófono elegido no encontrado; usando el default"
        );
    }
    host.default_input_device()
}
/// Tope del buffer de captura: 120 s de dictado máximo (ARCHITECTURE §3).
pub const MAX_CAPTURE_SECONDS: u32 = 120;

pub struct AudioData {
    /// 16 kHz mono i16, listo para codificar a WAV.
    pub samples: Vec<i16>,
    pub sample_rate: u32,
}

impl AudioData {
    pub fn duration_ms(&self) -> u64 {
        (self.samples.len() as u64 * 1_000) / u64::from(self.sample_rate)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("no default input device")]
    NoInputDevice,
    #[error("unsupported input config: {0}")]
    UnsupportedConfig(String),
    #[error("stream error: {0}")]
    Stream(String),
    #[error("resample error: {0}")]
    Resample(String),
}

impl AudioError {
    /// Clave i18n para la UI (catálogo err.* en src/i18n).
    pub fn error_key(&self) -> &'static str {
        match self {
            AudioError::NoInputDevice => "err.audio.device",
            AudioError::UnsupportedConfig(_) => "err.audio.config",
            AudioError::Stream(_) => "err.audio.stream",
            AudioError::Resample(_) => "err.audio.resample",
        }
    }
}

pub mod convert {
    use super::{AudioError, TARGET_SAMPLE_RATE};
    use rubato::audioadapter_buffers::direct::SequentialSlice;
    use rubato::{Fft, FixedSync, Resampler};

    /// Mezcla interleaved multicanal a mono promediando canales.
    pub fn mix_to_mono(interleaved: &[f32], channels: usize) -> Vec<f32> {
        if channels <= 1 {
            return interleaved.to_vec();
        }
        interleaved
            .chunks_exact(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    }

    /// Resamplea mono f32 a 16 kHz. Si ya está a 16 kHz, pasa directo.
    pub fn resample_to_target(mono: &[f32], from_rate: u32) -> Result<Vec<f32>, AudioError> {
        if from_rate == TARGET_SAMPLE_RATE {
            return Ok(mono.to_vec());
        }
        if mono.is_empty() {
            return Ok(Vec::new());
        }
        let mut resampler = Fft::<f32>::new(
            from_rate as usize,
            TARGET_SAMPLE_RATE as usize,
            1_024,
            2,
            1,
            FixedSync::Input,
        )
        .map_err(|e| AudioError::Resample(e.to_string()))?;

        let out_len = resampler.process_all_needed_output_len(mono.len());
        let mut out = vec![0.0f32; out_len];
        let input = SequentialSlice::new(mono, 1, mono.len())
            .map_err(|e| AudioError::Resample(e.to_string()))?;
        let mut output = SequentialSlice::new_mut(&mut out, 1, out_len)
            .map_err(|e| AudioError::Resample(e.to_string()))?;

        let (_, written) = resampler
            .process_all_into_buffer(&input, &mut output, mono.len(), None)
            .map_err(|e| AudioError::Resample(e.to_string()))?;
        out.truncate(written);
        Ok(out)
    }

    /// f32 [-1.0, 1.0] → i16 con saturación.
    pub fn to_i16(samples: &[f32]) -> Vec<i16> {
        samples
            .iter()
            .map(|s| (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16)
            .collect()
    }

    /// Codifica WAV PCM 16-bit mono en memoria (§4.3: justo antes del envío).
    pub fn encode_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut cursor = std::io::Cursor::new(Vec::new());
        {
            // Escritura a memoria: no puede fallar salvo bug de hound.
            let mut writer = hound::WavWriter::new(&mut cursor, spec).expect("wav header");
            for &s in samples {
                writer.write_sample(s).expect("wav sample");
            }
            writer.finalize().expect("wav finalize");
        }
        cursor.into_inner()
    }
}

/// Grabación en curso. `stop()` corta el stream y devuelve el audio ya
/// convertido a 16 kHz mono.
pub struct Recorder {
    stop_tx: mpsc::Sender<()>,
    thread: JoinHandle<Result<AudioData, AudioError>>,
    /// Nombre del micrófono que el host abrió de verdad (el resuelto, no el
    /// pedido): alimenta `RecordingStarted.device_id` para que sea observable
    /// qué dispositivo quedó grabando.
    device_name: String,
}

/// Callback de nivel de entrada (0..1, RMS normalizado), invocado desde el
/// hilo de captura con throttle. Telemetría para la onda del overlay.
pub type LevelCallback = Box<dyn Fn(f32) + Send + 'static>;

/// Cada cuánto se reporta el nivel (≈30 Hz: fluido sin inundar el IPC).
const LEVEL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(33);
/// Ganancia para llevar el RMS de voz típico a un rango visual útil.
const LEVEL_GAIN: f32 = 6.0;

/// Configuración del corte por VAD en modo toggle (ADR-0011). Los callbacks
/// corren en el hilo de captura: deben limitarse a encolar eventos.
pub struct VadConfig {
    pub threshold: f32,
    pub silence_hangover_ms: u64,
    /// Silencio sostenido alcanzó el hangover (una vez por grabación).
    pub on_silence: Box<dyn Fn(u64) + Send + 'static>,
    /// Cambio de estado hablando/en-silencio (telemetría para el overlay).
    pub on_speaking: Box<dyn Fn(bool) + Send + 'static>,
}

impl Recorder {
    /// Abre el dispositivo de entrada por defecto y empieza a capturar.
    /// Devuelve error si no hay dispositivo o el stream no arranca.
    pub fn start() -> Result<Self, AudioError> {
        Self::start_with_level(None)
    }

    /// Como [`Recorder::start`], reportando además el nivel de entrada.
    pub fn start_with_level(on_level: Option<LevelCallback>) -> Result<Self, AudioError> {
        Self::start_with_options(on_level, None, None)
    }

    /// Como [`Recorder::start_with_level`], opcionalmente con corte por VAD
    /// (modo toggle) y con el micrófono elegido por nombre (`None` = default
    /// del SO). Si el VAD no puede inicializarse se degrada a grabar sin corte
    /// automático (queda el tope de 120 s); un nombre de micrófono inexistente
    /// cae al default — nunca falla el inicio por configuración de captura.
    pub fn start_with_options(
        on_level: Option<LevelCallback>,
        vad: Option<VadConfig>,
        device_name: Option<String>,
    ) -> Result<Self, AudioError> {
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        // El Ok trae el nombre del micrófono que el host abrió de verdad (§4.3).
        let (ready_tx, ready_rx) = mpsc::channel::<Result<String, AudioError>>();

        let thread = std::thread::spawn(move || {
            capture_thread(&stop_rx, &ready_tx, on_level, vad, device_name.as_deref())
        });

        match ready_rx.recv() {
            Ok(Ok(device_name)) => Ok(Self {
                stop_tx,
                thread,
                device_name,
            }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => Err(AudioError::Stream("capture thread died".into())),
        }
    }

    /// Nombre del micrófono realmente abierto por el host (§4.3).
    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    /// Detiene la captura y entrega el audio procesado.
    pub fn stop(self) -> Result<AudioData, AudioError> {
        let _ = self.stop_tx.send(());
        self.thread
            .join()
            .map_err(|_| AudioError::Stream("capture thread panicked".into()))?
    }
}

fn capture_thread(
    stop_rx: &mpsc::Receiver<()>,
    ready_tx: &mpsc::Sender<Result<String, AudioError>>,
    on_level: Option<LevelCallback>,
    vad: Option<VadConfig>,
    device_name: Option<&str>,
) -> Result<AudioData, AudioError> {
    let host = cpal::default_host();
    let Some(device) = select_input_device(&host, device_name) else {
        let _ = ready_tx.send(Err(AudioError::NoInputDevice));
        return Err(AudioError::NoInputDevice);
    };
    // Nombre del device efectivamente abierto (el resuelto): a `RecordingStarted`
    // y al log, para poder confirmar qué micrófono quedó grabando vs. el pedido.
    let opened_name = device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "desconocido".to_string());
    tracing::info!(
        requested = device_name.unwrap_or("(default)"),
        opened = %opened_name,
        "micrófono de captura resuelto"
    );
    let config = match device.default_input_config() {
        Ok(c) => c,
        Err(e) => {
            let err = AudioError::UnsupportedConfig(e.to_string());
            let _ = ready_tx.send(Err(AudioError::UnsupportedConfig(e.to_string())));
            return Err(err);
        }
    };
    let sample_rate = config.sample_rate();
    let channels = usize::from(config.channels());
    let max_samples = sample_rate as usize * channels * MAX_CAPTURE_SECONDS as usize;

    // Gate del VAD (solo modo toggle). Si Silero/ONNX no inicializa, se
    // degrada explícitamente: warn al log y grabación sin corte automático.
    let mut vad_state = vad.and_then(|cfg| {
        match vad::VadGate::new(
            sample_rate,
            channels,
            cfg.threshold,
            cfg.silence_hangover_ms,
        ) {
            Ok(gate) => Some((gate, cfg)),
            Err(e) => {
                tracing::warn!(error = %e, "VAD no disponible; se graba sin corte por silencio");
                None
            }
        }
    });

    let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
    let writer = Arc::clone(&buffer);
    let mut last_level = std::time::Instant::now() - LEVEL_INTERVAL;
    let push = move |data: &[f32]| {
        if let Some((gate, cfg)) = &mut vad_state {
            let update = gate.feed(data);
            if let Some(speaking) = update.speaking_changed {
                (cfg.on_speaking)(speaking);
            }
            if let Some(silence_ms) = update.silence_cut_ms {
                (cfg.on_silence)(silence_ms);
            }
        }
        // Nivel RMS del chunk, con throttle: alimenta la onda del overlay.
        if let Some(cb) = &on_level {
            if last_level.elapsed() >= LEVEL_INTERVAL && !data.is_empty() {
                last_level = std::time::Instant::now();
                let sum_sq: f32 = data.iter().map(|s| s * s).sum();
                let rms = (sum_sq / data.len() as f32).sqrt();
                cb((rms * LEVEL_GAIN).clamp(0.0, 1.0));
            }
        }
        let mut buf = writer.lock().expect("audio buffer lock");
        let room = max_samples.saturating_sub(buf.len());
        buf.extend_from_slice(&data[..data.len().min(room)]);
    };

    let stream = match build_input_stream(&device, &config, push) {
        Ok(s) => s,
        Err(e) => {
            let msg = e.to_string();
            let _ = ready_tx.send(Err(AudioError::Stream(msg.clone())));
            return Err(AudioError::Stream(msg));
        }
    };
    if let Err(e) = stream.play() {
        let msg = e.to_string();
        let _ = ready_tx.send(Err(AudioError::Stream(msg.clone())));
        return Err(AudioError::Stream(msg));
    }
    let _ = ready_tx.send(Ok(opened_name));

    // Bloquea hasta que Recorder::stop() envíe la señal (o se caiga el otro lado).
    let _ = stop_rx.recv();
    drop(stream);

    let raw = std::mem::take(&mut *buffer.lock().expect("audio buffer lock"));
    let mono = convert::mix_to_mono(&raw, channels);
    let resampled = convert::resample_to_target(&mono, sample_rate)?;
    Ok(AudioData {
        samples: convert::to_i16(&resampled),
        sample_rate: TARGET_SAMPLE_RATE,
    })
}

fn build_input_stream(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    mut push: impl FnMut(&[f32]) + Send + 'static,
) -> Result<cpal::Stream, AudioError> {
    let stream_config: cpal::StreamConfig = config.config();
    let on_err = |e: cpal::Error| tracing::warn!(error = %e, "audio stream error");
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            stream_config,
            move |data: &[f32], _| push(data),
            on_err,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            stream_config,
            move |data: &[i16], _| {
                let f: Vec<f32> = data
                    .iter()
                    .map(|&s| f32::from(s) / -f32::from(i16::MIN))
                    .collect();
                push(&f);
            },
            on_err,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            stream_config,
            move |data: &[u16], _| {
                let f: Vec<f32> = data
                    .iter()
                    .map(|&s| (f32::from(s) - 32_768.0) / 32_768.0)
                    .collect();
                push(&f);
            },
            on_err,
            None,
        ),
        other => {
            return Err(AudioError::UnsupportedConfig(format!(
                "sample format {other:?}"
            )))
        }
    };
    stream.map_err(|e| AudioError::Stream(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::convert::*;
    use super::*;

    #[test]
    fn mix_to_mono_promedia_canales() {
        let stereo = [1.0, 0.0, 0.5, 0.5, -1.0, 1.0];
        assert_eq!(mix_to_mono(&stereo, 2), vec![0.5, 0.5, 0.0]);
    }

    #[test]
    fn mix_to_mono_con_un_canal_es_identidad() {
        let mono = [0.1, 0.2, 0.3];
        assert_eq!(mix_to_mono(&mono, 1), mono.to_vec());
    }

    #[test]
    fn to_i16_satura_fuera_de_rango() {
        let out = to_i16(&[0.0, 1.0, -1.0, 2.0, -2.0]);
        assert_eq!(out[0], 0);
        assert_eq!(out[1], i16::MAX);
        assert_eq!(out[3], i16::MAX);
        // Los valores <= -1.0 saturan al mismo extremo negativo.
        assert_eq!(out[2], out[4]);
    }

    #[test]
    fn resample_misma_frecuencia_es_bypass() {
        let mono = vec![0.5f32; 1_600];
        let out = resample_to_target(&mono, TARGET_SAMPLE_RATE).unwrap();
        assert_eq!(out, mono);
    }

    #[test]
    fn resample_48k_a_16k_reduce_a_un_tercio() {
        // 1 s de señal constante a 48 kHz → ~16.000 muestras a 16 kHz.
        let mono = vec![0.25f32; 48_000];
        let out = resample_to_target(&mono, 48_000).unwrap();
        let expected = 16_000usize;
        let tolerance = expected / 100; // ±1%
        assert!(
            out.len().abs_diff(expected) <= tolerance,
            "len {} vs esperado {expected}",
            out.len()
        );
        // El centro de la señal debe conservar el nivel (evita bordes del filtro).
        let mid = out[out.len() / 2];
        assert!((mid - 0.25).abs() < 0.01, "nivel {mid}");
    }

    #[test]
    fn encode_wav_produce_header_riff_valido() {
        let samples = vec![0i16; 160];
        let wav = encode_wav(&samples, TARGET_SAMPLE_RATE);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        // 44 bytes de header + 2 bytes por muestra.
        assert_eq!(wav.len(), 44 + samples.len() * 2);
        // Reparseable por hound con el spec esperado.
        let reader = hound::WavReader::new(std::io::Cursor::new(&wav)).unwrap();
        assert_eq!(reader.spec().sample_rate, TARGET_SAMPLE_RATE);
        assert_eq!(reader.spec().channels, 1);
    }

    #[test]
    fn audio_data_calcula_duracion() {
        let data = AudioData {
            samples: vec![0; 32_000],
            sample_rate: 16_000,
        };
        assert_eq!(data.duration_ms(), 2_000);
    }
}
