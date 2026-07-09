//! Post-procesado LLM del dictado (ADR-0014). Trait `TextProcessor` análogo
//! a `SpeechProvider` (ADR-0003): el core habla con la abstracción y cada
//! proveedor concreto aporta base URL y credencial. OpenAI y Groq comparten
//! el contrato `POST /chat/completions`, así que hay una sola implementación
//! HTTP. Reusa `SpeechError` — la casuística de red/auth/rate es idéntica —
//! pero el borde la mapea a claves `err.llm.*` propias.

pub mod prompts;

use std::time::Duration;

use crate::speech::SpeechError;

/// Petición de una pasada LLM ya resuelta (prompt construido por el core).
#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    /// Prompt de sistema (fijo y versionado, `llm::prompts`).
    pub system: String,
    /// Contenido del usuario: la transcripción (mejorado) o la instrucción
    /// dictada (prompt).
    pub user: String,
    pub timeout: Duration,
}

/// Procesador de texto por LLM. Análogo a `SpeechProvider`: agregar un
/// proveedor = implementar el trait + una rama en `resolve()`.
#[async_trait::async_trait]
pub trait TextProcessor: Send + Sync {
    async fn process(&self, request: ChatRequest) -> Result<String, SpeechError>;
}

/// Cliente `POST /chat/completions` compatible con OpenAI. Mismo contrato en
/// OpenAI y Groq (ADR-0012 aplica también al endpoint de chat).
pub struct OpenAiCompatibleChat {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl OpenAiCompatibleChat {
    pub fn new(base_url: impl Into<String>, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            api_key,
        }
    }
}

#[async_trait::async_trait]
impl TextProcessor for OpenAiCompatibleChat {
    async fn process(&self, request: ChatRequest) -> Result<String, SpeechError> {
        #[derive(serde::Deserialize)]
        struct ChatResponse {
            choices: Vec<Choice>,
        }
        #[derive(serde::Deserialize)]
        struct Choice {
            message: Message,
        }
        #[derive(serde::Deserialize)]
        struct Message {
            content: String,
        }

        let body = serde_json::json!({
            "model": request.model,
            "messages": [
                { "role": "system", "content": request.system },
                { "role": "user", "content": request.user },
            ],
        });
        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .timeout(request.timeout)
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
                let body: ChatResponse =
                    response.json().await.map_err(|e| SpeechError::Provider {
                        code: format!("respuesta inválida: {e}"),
                    })?;
                let text = body
                    .choices
                    .into_iter()
                    .next()
                    .map(|c| c.message.content)
                    .ok_or_else(|| SpeechError::Provider {
                        code: "respuesta sin choices".into(),
                    })?;
                Ok(text.trim().to_string())
            }
            401 | 403 => Err(SpeechError::Auth),
            429 => Err(SpeechError::RateLimited),
            status => Err(SpeechError::Provider {
                code: status.to_string(),
            }),
        }
    }
}

/// Construye el procesador para el proveedor de `settings.llm.provider`,
/// reusando las base URLs de los proveedores STT. Id desconocido cae a
/// OpenAI (mismo criterio que `providers::resolve`).
pub fn resolve(provider_id: &str, api_key: String) -> Box<dyn TextProcessor> {
    let base_url = if provider_id == crate::providers::groq::ID {
        "https://api.groq.com/openai/v1"
    } else {
        "https://api.openai.com/v1"
    };
    Box::new(OpenAiCompatibleChat::new(base_url, api_key))
}

/// Clave i18n `err.llm.*` para un fallo de la etapa LLM (≠ `err.stt.*`: la
/// UI distingue qué etapa falló).
pub fn error_key(e: &SpeechError) -> &'static str {
    match e {
        SpeechError::Auth => "err.llm.auth",
        SpeechError::Network => "err.llm.network",
        SpeechError::RateLimited => "err.llm.rate",
        _ => "err.llm.provider",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header_exists, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn request() -> ChatRequest {
        ChatRequest {
            model: "gpt-4o-mini".into(),
            system: "sistema".into(),
            user: "eh… hola mundo".into(),
            timeout: Duration::from_secs(5),
        }
    }

    fn chat_for(server: &MockServer) -> OpenAiCompatibleChat {
        OpenAiCompatibleChat::new(server.uri(), "sk-test".into())
    }

    #[tokio::test]
    async fn respuesta_200_devuelve_el_contenido_recortado() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header_exists("authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [
                    { "message": { "role": "assistant", "content": "  Hola, mundo.\n" } }
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let text = chat_for(&server).process(request()).await.unwrap();
        assert_eq!(text, "Hola, mundo.");
    }

    #[tokio::test]
    async fn respuesta_401_mapea_a_auth() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&server)
            .await;

        let err = chat_for(&server).process(request()).await.unwrap_err();
        assert!(matches!(err, SpeechError::Auth));
    }

    #[tokio::test]
    async fn respuesta_429_mapea_a_rate_limited() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(429))
            .expect(1)
            .mount(&server)
            .await;

        let err = chat_for(&server).process(request()).await.unwrap_err();
        assert!(matches!(err, SpeechError::RateLimited));
    }

    #[tokio::test]
    async fn respuesta_sin_choices_mapea_a_provider() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": []
            })))
            .expect(1)
            .mount(&server)
            .await;

        let err = chat_for(&server).process(request()).await.unwrap_err();
        assert!(matches!(err, SpeechError::Provider { code } if code == "respuesta sin choices"));
    }

    #[tokio::test]
    async fn sin_conexion_mapea_a_network() {
        let chat = OpenAiCompatibleChat::new("http://127.0.0.1:9", "sk-test".into());
        let err = chat.process(request()).await.unwrap_err();
        assert!(matches!(err, SpeechError::Network));
    }

    #[test]
    fn error_key_distingue_la_etapa_llm() {
        assert_eq!(error_key(&SpeechError::Auth), "err.llm.auth");
        assert_eq!(error_key(&SpeechError::Network), "err.llm.network");
        assert_eq!(error_key(&SpeechError::RateLimited), "err.llm.rate");
        assert_eq!(
            error_key(&SpeechError::Provider { code: "500".into() }),
            "err.llm.provider"
        );
    }

    #[test]
    fn prompts_parametrizan_el_idioma() {
        assert!(prompts::improve_system_prompt(Some("es")).contains("responde en español"));
        assert!(prompts::improve_system_prompt(Some("en")).contains("respond in English"));
        assert!(prompts::improve_system_prompt(None).contains("mismo idioma"));
        assert!(prompts::instruction_system_prompt(Some("es")).contains("escribe en español"));
    }
}
