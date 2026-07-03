//! Captura de micrófono (cpal) y resampling a 16 kHz mono. Ver docs/ARCHITECTURE.md §4.3.
//! Implementación (captura real, resampling, codificación WAV): M1.

pub struct AudioData {
    pub samples: Vec<i16>,
    pub sample_rate: u32,
}

// TODO(M1): captura con cpal (dispositivo por defecto), resampling con rubato,
// codificación WAV en memoria justo antes del envío. El audio nunca toca disco
// salvo flag --debug-audio explícito (PRD §15.5).
