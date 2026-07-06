//! Proveedor OpenAI (`POST https://api.openai.com/v1/audio/transcriptions`).
//! Toda la lógica HTTP vive en `providers::openai_compatible`; aquí solo la
//! identidad, la base URL y el modelo por defecto (ARCHITECTURE §4.5).

use crate::providers::openai_compatible::OpenAiCompatibleProvider;

pub const ID: &str = "openai";
pub const DEFAULT_MODEL: &str = "gpt-4o-mini-transcribe";
const BASE_URL: &str = "https://api.openai.com/v1";

/// Construye el proveedor OpenAI con la key dada (fetch fresco del keyring en
/// cada ciclo; nunca se retiene la key en un registry de larga vida).
pub fn provider(api_key: String) -> OpenAiCompatibleProvider {
    OpenAiCompatibleProvider::new("openai", BASE_URL, api_key)
}
