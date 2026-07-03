//! Proveedor OpenAI (`POST /v1/audio/transcriptions`). Ver docs/ARCHITECTURE.md §4.5.
//! Modelo por defecto: `gpt-4o-mini-transcribe`; timeout 30s; 1 reintento
//! automático solo ante Network/RateLimited (PRD §17.3-17.4).

use std::time::Instant;

use crate::audio::{convert, AudioData};
use crate::speech::{
    ProviderCapabilities, SpeechError, SpeechProvider, TranscribeOptions, Transcript,
};

pub const DEFAULT_MODEL: &str = "gpt-4o-mini-transcribe";
const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

pub struct OpenAiProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key,
        }
    }

    /// Apunta a otro endpoint (tests con wiremock).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    async fn request_once(
        &self,
        wav: Vec<u8>,
        opts: &TranscribeOptions,
    ) -> Result<String, SpeechError> {
        let part = reqwest::multipart::Part::bytes(wav)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|_| SpeechError::InvalidAudio)?;
        let mut form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model", opts.model.clone());
        if let Some(lang) = &opts.language {
            form = form.text("language", lang.clone());
        }

        let response = self
            .client
            .post(format!("{}/audio/transcriptions", self.base_url))
            .bearer_auth(&self.api_key)
            .multipart(form)
            .timeout(opts.timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() || e.is_connect() {
                    SpeechError::Network
                } else {
                    SpeechError::Provider {
                        code: e.to_string(),
                    }
                }
            })?;

        match response.status().as_u16() {
            200 => {
                #[derive(serde::Deserialize)]
                struct Body {
                    text: String,
                }
                let body: Body = response.json().await.map_err(|e| SpeechError::Provider {
                    code: format!("respuesta inválida: {e}"),
                })?;
                Ok(body.text)
            }
            401 | 403 => Err(SpeechError::Auth),
            429 => Err(SpeechError::RateLimited),
            400 | 422 => Err(SpeechError::InvalidAudio),
            status => Err(SpeechError::Provider {
                code: status.to_string(),
            }),
        }
    }
}

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
        audio: AudioData,
        opts: TranscribeOptions,
    ) -> Result<Transcript, SpeechError> {
        let wav = convert::encode_wav(&audio.samples, audio.sample_rate);
        let started = Instant::now();

        let text = match self.request_once(wav.clone(), &opts).await {
            Ok(text) => text,
            // 1 reintento solo ante errores transitorios (PRD §17.3-17.4).
            Err(SpeechError::Network) | Err(SpeechError::RateLimited) => {
                tracing::info!("reintentando transcripción tras error transitorio");
                self.request_once(wav, &opts).await?
            }
            Err(e) => return Err(e),
        };

        Ok(Transcript {
            text,
            language: opts.language,
            latency: started.elapsed(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use wiremock::matchers::{header_exists, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn audio_fixture() -> AudioData {
        AudioData {
            samples: vec![0i16; 1_600],
            sample_rate: 16_000,
        }
    }

    fn opts() -> TranscribeOptions {
        TranscribeOptions {
            language: Some("es".into()),
            model: DEFAULT_MODEL.into(),
            timeout: Duration::from_secs(5),
        }
    }

    async fn provider_for(server: &MockServer) -> OpenAiProvider {
        OpenAiProvider::new("sk-test".into()).with_base_url(server.uri())
    }

    #[tokio::test]
    async fn transcripcion_exitosa_devuelve_texto() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/audio/transcriptions"))
            .and(header_exists("authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "text": "hola mundo"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let transcript = provider_for(&server)
            .await
            .transcribe(audio_fixture(), opts())
            .await
            .unwrap();
        assert_eq!(transcript.text, "hola mundo");
        assert_eq!(transcript.language.as_deref(), Some("es"));
    }

    #[tokio::test]
    async fn respuesta_401_mapea_a_auth_sin_reintento() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/audio/transcriptions"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&server)
            .await;

        let err = provider_for(&server)
            .await
            .transcribe(audio_fixture(), opts())
            .await
            .unwrap_err();
        assert!(matches!(err, SpeechError::Auth));
    }

    #[tokio::test]
    async fn rate_limit_reintenta_una_vez_y_recupera() {
        let server = MockServer::start().await;
        // Primer intento: 429. El mock se agota tras 1 uso.
        Mock::given(method("POST"))
            .and(path("/audio/transcriptions"))
            .respond_with(ResponseTemplate::new(429))
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;
        // Segundo intento: 200.
        Mock::given(method("POST"))
            .and(path("/audio/transcriptions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "text": "recuperado"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let transcript = provider_for(&server)
            .await
            .transcribe(audio_fixture(), opts())
            .await
            .unwrap();
        assert_eq!(transcript.text, "recuperado");
    }

    #[tokio::test]
    async fn rate_limit_persistente_falla_tras_el_reintento() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/audio/transcriptions"))
            .respond_with(ResponseTemplate::new(429))
            .expect(2) // intento original + 1 reintento, no más
            .mount(&server)
            .await;

        let err = provider_for(&server)
            .await
            .transcribe(audio_fixture(), opts())
            .await
            .unwrap_err();
        assert!(matches!(err, SpeechError::RateLimited));
    }

    #[tokio::test]
    async fn error_500_mapea_a_provider() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/audio/transcriptions"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&server)
            .await;

        let err = provider_for(&server)
            .await
            .transcribe(audio_fixture(), opts())
            .await
            .unwrap_err();
        assert!(matches!(err, SpeechError::Provider { code } if code == "500"));
    }

    #[tokio::test]
    async fn timeout_mapea_a_network_y_reintenta() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/audio/transcriptions"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
            .expect(2) // original + reintento, ambos con timeout
            .mount(&server)
            .await;

        let mut o = opts();
        o.timeout = Duration::from_millis(200);
        let err = provider_for(&server)
            .await
            .transcribe(audio_fixture(), o)
            .await
            .unwrap_err();
        assert!(matches!(err, SpeechError::Network));
    }
}
