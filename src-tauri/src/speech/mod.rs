//! Abstracción `SpeechProvider` + `ProviderRegistry` (ADR-0003). El core solo
//! conoce este trait, nunca una implementación concreta. Ver docs/ARCHITECTURE.md §4.4.

use std::collections::HashMap;
use std::time::Duration;

use crate::audio::AudioData;

#[derive(Debug, Clone, Copy, Default)]
pub struct ProviderCapabilities {
    pub batch: bool,
    pub streaming: bool,
}

pub struct TranscribeOptions {
    pub language: Option<String>,
    pub model: String,
    pub timeout: Duration,
}

#[derive(Debug)]
pub struct Transcript {
    pub text: String,
    pub language: Option<String>,
    pub latency: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum SpeechError {
    #[error("autenticación inválida o ausente")]
    Auth,
    #[error("error de red o timeout")]
    Network,
    #[error("rate limit del proveedor")]
    RateLimited,
    #[error("audio inválido")]
    InvalidAudio,
    #[error("error del proveedor: {code}")]
    Provider { code: String },
}

#[async_trait::async_trait]
pub trait SpeechProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn capabilities(&self) -> ProviderCapabilities;
    async fn transcribe(
        &self,
        audio: AudioData,
        opts: TranscribeOptions,
    ) -> Result<Transcript, SpeechError>;
    /// Valida credenciales sin transcribir (lo usa el comando `test_provider`).
    /// Un proveedor sin auth remota (p. ej. STT local futuro) devuelve `Ok`.
    async fn check_auth(&self, model: &str) -> Result<(), SpeechError>;
}

/// Mapa `id → proveedor`; resuelve según `config.stt.provider` (RNF-08).
#[derive(Default)]
pub struct ProviderRegistry {
    providers: HashMap<&'static str, Box<dyn SpeechProvider>>,
}

impl ProviderRegistry {
    pub fn register(&mut self, provider: Box<dyn SpeechProvider>) {
        self.providers.insert(provider.id(), provider);
    }

    pub fn get(&self, id: &str) -> Option<&dyn SpeechProvider> {
        self.providers.get(id).map(|p| p.as_ref())
    }
}
