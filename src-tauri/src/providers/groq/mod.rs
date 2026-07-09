//! Proveedor Groq (`POST https://api.groq.com/openai/v1/audio/transcriptions`),
//! API compatible con OpenAI (ADR-0012). Reusa toda la lógica de
//! `providers::openai_compatible`; aquí solo identidad, base URL y modelo por
//! defecto (`whisper-large-v3-turbo`).

use crate::providers::openai_compatible::OpenAiCompatibleProvider;
use crate::providers::ModelKind;

pub const ID: &str = "groq";
pub const DEFAULT_MODEL: &str = "whisper-large-v3-turbo";
const BASE_URL: &str = "https://api.groq.com/openai/v1";

/// Construye el proveedor Groq con la key dada (misma disciplina que OpenAI:
/// key fresca del keyring por ciclo).
pub fn provider(api_key: String) -> OpenAiCompatibleProvider {
    OpenAiCompatibleProvider::new("groq", BASE_URL, api_key)
}

/// Patrones no conversacionales del catálogo de Groq: TTS (playai-tts*) y
/// modelos de seguridad (llama-guard, prompt-guard) que responden etiquetas,
/// no chat útil.
const CHAT_EXCLUDE: &[&str] = &["tts", "guard", "embedding"];

/// Clasifica un id de modelo de Groq. STT: todo lo que contenga "whisper"
/// (whisper-large-v3*, distil-whisper*). Chat: el resto salvo denylist —
/// Groq solo publica modelos servibles por chat/completions, así que un
/// modelo nuevo desconocido entra a chat por defecto (mejor de más que de
/// menos: el catálogo chat aún no maneja tráfico, ADR-0014).
pub fn classify_model(id: &str) -> Option<ModelKind> {
    let id = id.to_ascii_lowercase();
    if id.contains("whisper") {
        return Some(ModelKind::Stt);
    }
    if CHAT_EXCLUDE.iter().any(|p| id.contains(p)) {
        return None;
    }
    Some(ModelKind::Chat)
}
