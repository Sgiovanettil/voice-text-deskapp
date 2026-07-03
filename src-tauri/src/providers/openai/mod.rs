//! Proveedor OpenAI (`POST /v1/audio/transcriptions`). Ver docs/ARCHITECTURE.md §4.5.
//! Modelo por defecto: `gpt-4o-mini-transcribe`; timeout 30s; 1 reintento
//! automático solo ante Network/RateLimited (PRD §17.3-17.4). Implementación: M1.

use crate::audio::AudioData;
use crate::speech::{
    ProviderCapabilities, SpeechError, SpeechProvider, TranscribeOptions, Transcript,
};

pub struct OpenAiProvider;

#[async_trait::async_trait]
impl SpeechProvider for OpenAiProvider {
    fn id(&self) -> &'static str {
        "openai"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            batch: true,
            streaming: false,
        }
    }

    async fn transcribe(
        &self,
        _audio: AudioData,
        _opts: TranscribeOptions,
    ) -> Result<Transcript, SpeechError> {
        todo!("M1: POST /v1/audio/transcriptions vía reqwest (rustls), key desde keyring")
    }
}
