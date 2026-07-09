//! Proveedor OpenAI (`POST https://api.openai.com/v1/audio/transcriptions`).
//! Toda la lógica HTTP vive en `providers::openai_compatible`; aquí solo la
//! identidad, la base URL y el modelo por defecto (ARCHITECTURE §4.5).

use crate::providers::openai_compatible::OpenAiCompatibleProvider;
use crate::providers::ModelKind;

pub const ID: &str = "openai";
pub const DEFAULT_MODEL: &str = "gpt-4o-mini-transcribe";
const BASE_URL: &str = "https://api.openai.com/v1";

/// Construye el proveedor OpenAI con la key dada (fetch fresco del keyring en
/// cada ciclo; nunca se retiene la key en un registry de larga vida).
pub fn provider(api_key: String) -> OpenAiCompatibleProvider {
    OpenAiCompatibleProvider::new("openai", BASE_URL, api_key)
}

/// Patrones que descartan un modelo como conversacional: no sirven en
/// `POST /chat/completions` o son de otra modalidad.
const CHAT_EXCLUDE: &[&str] = &[
    "embedding",
    "tts",
    "dall-e",
    "image",
    "moderation",
    "realtime",
    "audio",
    "search",
    "computer-use",
    "sora",
    "instruct",
    "davinci",
    "babbage",
    "codex",
];

/// Clasifica un id de modelo de OpenAI. STT por allowlist (contiene
/// "transcribe" o empieza con "whisper"); chat por familias conocidas
/// (gpt-*/chatgpt-*/o1*/o3*/o4*) menos la denylist. Lo no reconocido se
/// excluye: en STT un falso positivo rompe la transcripción.
pub fn classify_model(id: &str) -> Option<ModelKind> {
    let id = id.to_ascii_lowercase();
    if id.contains("transcribe") || id.starts_with("whisper") {
        return Some(ModelKind::Stt);
    }
    if CHAT_EXCLUDE.iter().any(|p| id.contains(p)) {
        return None;
    }
    if id.starts_with("gpt-")
        || id.starts_with("chatgpt-")
        || id.starts_with("o1")
        || id.starts_with("o3")
        || id.starts_with("o4")
    {
        return Some(ModelKind::Chat);
    }
    None
}
