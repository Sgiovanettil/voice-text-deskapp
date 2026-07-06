//! Proveedor Groq (`POST https://api.groq.com/openai/v1/audio/transcriptions`),
//! API compatible con OpenAI (ADR-0012). Reusa toda la lógica de
//! `providers::openai_compatible`; aquí solo identidad, base URL y modelo por
//! defecto (`whisper-large-v3-turbo`).

use crate::providers::openai_compatible::OpenAiCompatibleProvider;

pub const ID: &str = "groq";
pub const DEFAULT_MODEL: &str = "whisper-large-v3-turbo";
const BASE_URL: &str = "https://api.groq.com/openai/v1";

/// Construye el proveedor Groq con la key dada (misma disciplina que OpenAI:
/// key fresca del keyring por ciclo).
pub fn provider(api_key: String) -> OpenAiCompatibleProvider {
    OpenAiCompatibleProvider::new("groq", BASE_URL, api_key)
}
