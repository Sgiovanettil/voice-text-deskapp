//! Cliente HTTP compartido para APIs de transcripción compatibles con OpenAI
//! (`POST /audio/transcriptions`, multipart WAV con campos `file`/`model`/
//! `language`). OpenAI y Groq usan exactamente este contrato (ADR-0012), así
//! que la lógica de multipart, timeout (30 s), 1 reintento ante
//! `Network`/`RateLimited` y el mapeo a `SpeechError` viven una sola vez acá.
//! Cada proveedor concreto (`providers/openai`, `providers/groq`) solo aporta
//! su `id`, su base URL y su modelo por defecto.

use std::time::Instant;

use crate::audio::{convert, AudioData};
use crate::speech::{
    ProviderCapabilities, SpeechError, SpeechProvider, TranscribeOptions, Transcript,
};

/// Proveedor STT contra un endpoint compatible con OpenAI. El `id` y la
/// `base_url` los fija el módulo del proveedor concreto.
pub struct OpenAiCompatibleProvider {
    id: &'static str,
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl OpenAiCompatibleProvider {
    pub fn new(id: &'static str, base_url: impl Into<String>, api_key: String) -> Self {
        Self {
            id,
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            api_key,
        }
    }

    /// Apunta a otro endpoint (tests con wiremock).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Valida credenciales sin transcribir: `GET /models/{model}` es gratis
    /// y responde 401 con key inválida. Lo usa el comando `test_provider`.
    async fn check_auth_impl(&self, model: &str) -> Result<(), SpeechError> {
        let response = self
            .client
            .get(format!("{}/models/{model}", self.base_url))
            .bearer_auth(&self.api_key)
            .timeout(std::time::Duration::from_secs(10))
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
            200 => Ok(()),
            401 | 403 => Err(SpeechError::Auth),
            429 => Err(SpeechError::RateLimited),
            // 404: key válida pero el modelo no existe — lo tratamos como
            // error de proveedor para que la UI muestre el modelo como causa.
            status => Err(SpeechError::Provider {
                code: status.to_string(),
            }),
        }
    }

    /// Lista los ids de modelos disponibles (`GET /models`, contrato OpenAI:
    /// `{ "object": "list", "data": [ { "id": ... }, ... ] }`). Groq añade
    /// campos extra (`active`, `context_window`); serde los ignora salvo
    /// `active`, que usamos para descartar modelos desactivados.
    pub async fn fetch_model_ids(&self) -> Result<Vec<String>, SpeechError> {
        #[derive(serde::Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelEntry>,
        }
        #[derive(serde::Deserialize)]
        struct ModelEntry {
            id: String,
            /// Solo Groq lo envía; ausente (OpenAI) = activo.
            #[serde(default)]
            active: Option<bool>,
        }

        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .bearer_auth(&self.api_key)
            .timeout(std::time::Duration::from_secs(10))
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
                let body: ModelsResponse =
                    response.json().await.map_err(|e| SpeechError::Provider {
                        code: format!("respuesta inválida: {e}"),
                    })?;
                Ok(body
                    .data
                    .into_iter()
                    .filter(|m| m.active != Some(false))
                    .map(|m| m.id)
                    .collect())
            }
            401 | 403 => Err(SpeechError::Auth),
            429 => Err(SpeechError::RateLimited),
            status => Err(SpeechError::Provider {
                code: status.to_string(),
            }),
        }
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
impl SpeechProvider for OpenAiCompatibleProvider {
    fn id(&self) -> &'static str {
        self.id
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

    async fn check_auth(&self, model: &str) -> Result<(), SpeechError> {
        self.check_auth_impl(model).await
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
            model: "gpt-4o-mini-transcribe".into(),
            timeout: Duration::from_secs(5),
        }
    }

    async fn provider_for(server: &MockServer) -> OpenAiCompatibleProvider {
        OpenAiCompatibleProvider::new("openai", "http://unused", "sk-test".into())
            .with_base_url(server.uri())
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

    #[tokio::test]
    async fn check_auth_200_ok() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/models/gpt-4o-mini-transcribe"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        provider_for(&server)
            .await
            .check_auth("gpt-4o-mini-transcribe")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn check_auth_401_mapea_a_auth() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/models/gpt-4o-mini-transcribe"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&server)
            .await;

        let err = provider_for(&server)
            .await
            .check_auth("gpt-4o-mini-transcribe")
            .await
            .unwrap_err();
        assert!(matches!(err, SpeechError::Auth));
    }

    #[tokio::test]
    async fn fetch_model_ids_200_devuelve_ids() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/models"))
            .and(header_exists("authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "object": "list",
                "data": [
                    { "id": "whisper-1", "object": "model", "created": 1_677_532_384, "owned_by": "openai-internal" },
                    { "id": "gpt-4o", "object": "model", "created": 1_715_367_049, "owned_by": "system" }
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let ids = provider_for(&server).await.fetch_model_ids().await.unwrap();
        assert_eq!(ids, vec!["whisper-1".to_string(), "gpt-4o".to_string()]);
    }

    #[tokio::test]
    async fn fetch_model_ids_filtra_inactivos_de_groq() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "object": "list",
                "data": [
                    { "id": "whisper-large-v3", "object": "model", "created": 1_693_721_698,
                      "owned_by": "OpenAI", "active": true, "context_window": 448 },
                    { "id": "modelo-viejo", "object": "model", "created": 1_693_721_698,
                      "owned_by": "Meta", "active": false, "context_window": 8192 }
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let ids = provider_for(&server).await.fetch_model_ids().await.unwrap();
        assert_eq!(ids, vec!["whisper-large-v3".to_string()]);
    }

    #[tokio::test]
    async fn fetch_model_ids_401_mapea_a_auth() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&server)
            .await;

        let err = provider_for(&server)
            .await
            .fetch_model_ids()
            .await
            .unwrap_err();
        assert!(matches!(err, SpeechError::Auth));
    }

    #[tokio::test]
    async fn fetch_model_ids_429_mapea_a_rate_limited() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(429))
            .expect(1)
            .mount(&server)
            .await;

        let err = provider_for(&server)
            .await
            .fetch_model_ids()
            .await
            .unwrap_err();
        assert!(matches!(err, SpeechError::RateLimited));
    }

    #[tokio::test]
    async fn fetch_model_ids_500_mapea_a_provider() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&server)
            .await;

        let err = provider_for(&server)
            .await
            .fetch_model_ids()
            .await
            .unwrap_err();
        assert!(matches!(err, SpeechError::Provider { code } if code == "500"));
    }

    #[tokio::test]
    async fn fetch_model_ids_json_malformado_mapea_a_provider() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "unexpected": true
            })))
            .expect(1)
            .mount(&server)
            .await;

        let err = provider_for(&server)
            .await
            .fetch_model_ids()
            .await
            .unwrap_err();
        assert!(
            matches!(err, SpeechError::Provider { ref code } if code.starts_with("respuesta inválida"))
        );
    }

    #[tokio::test]
    async fn fetch_model_ids_sin_conexion_mapea_a_network() {
        // Puerto sin listener: reqwest falla con is_connect() → Network. No se
        // testea el timeout de 10 s (alargaría la suite); comparte rama de mapeo.
        let provider =
            OpenAiCompatibleProvider::new("openai", "http://127.0.0.1:9", "sk-test".into());
        let err = provider.fetch_model_ids().await.unwrap_err();
        assert!(matches!(err, SpeechError::Network));
    }
}
