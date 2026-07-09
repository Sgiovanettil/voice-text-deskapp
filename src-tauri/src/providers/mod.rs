//! Implementaciones concretas de `SpeechProvider`. Un módulo por proveedor,
//! aislado — el core nunca importa un proveedor concreto: resuelve por id con
//! `resolve()` (RNF-08: agregar un proveedor = implementar el trait + una rama
//! acá). La lógica HTTP compartida vive en `openai_compatible`.

pub mod groq;
pub mod openai;
pub mod openai_compatible;

use crate::speech::{SpeechError, SpeechProvider};

/// Ids de proveedores conocidos, en orden de presentación en la UI.
pub const KNOWN: [&str; 2] = [openai::ID, groq::ID];

/// Construye el proveedor STT correspondiente al id de config, con la key ya
/// resuelta desde el keyring. Un id desconocido cae a OpenAI (mismo criterio
/// que los defaults de config: nunca dejar el core sin proveedor).
pub fn resolve(provider_id: &str, api_key: String) -> Box<dyn SpeechProvider> {
    if provider_id == groq::ID {
        Box::new(groq::provider(api_key))
    } else {
        Box::new(openai::provider(api_key))
    }
}

/// Modelo por defecto de un proveedor (para reset al cambiar de proveedor).
pub fn default_model(provider_id: &str) -> &'static str {
    if provider_id == groq::ID {
        groq::DEFAULT_MODEL
    } else {
        openai::DEFAULT_MODEL
    }
}

/// Categoría de un modelo listado por la API del proveedor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelKind {
    Stt,
    Chat,
}

/// Catálogo de modelos por categoría, listo para el borde IPC. `stt` alimenta
/// el selector de settings; `chat` queda disponible para el post-procesado
/// LLM (ADR-0014, sin UI todavía).
#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalog {
    pub stt: Vec<String>,
    pub chat: Vec<String>,
}

/// Lista los modelos del proveedor vía `GET {base_url}/models` y los
/// clasifica con la heurística del proveedor. Un id desconocido cae a
/// OpenAI (mismo criterio que `resolve`).
pub async fn list_models(provider_id: &str, api_key: String) -> Result<ModelCatalog, SpeechError> {
    let (provider, classify) = if provider_id == groq::ID {
        (
            groq::provider(api_key),
            groq::classify_model as fn(&str) -> Option<ModelKind>,
        )
    } else {
        (
            openai::provider(api_key),
            openai::classify_model as fn(&str) -> Option<ModelKind>,
        )
    };
    let ids = provider.fetch_model_ids().await?;
    Ok(build_catalog(ids, classify, default_model(provider_id)))
}

/// Clasifica, deduplica y ordena. STT: el default del proveedor primero (la
/// UI usa "primero = default"), resto alfabético case-insensitive. Chat:
/// alfabético case-insensitive.
fn build_catalog(
    ids: Vec<String>,
    classify: fn(&str) -> Option<ModelKind>,
    default: &str,
) -> ModelCatalog {
    let mut catalog = ModelCatalog::default();
    for id in ids {
        match classify(&id) {
            Some(ModelKind::Stt) if !catalog.stt.contains(&id) => catalog.stt.push(id),
            Some(ModelKind::Chat) if !catalog.chat.contains(&id) => catalog.chat.push(id),
            _ => {}
        }
    }
    catalog.stt.sort_by_key(|m| m.to_ascii_lowercase());
    catalog.chat.sort_by_key(|m| m.to_ascii_lowercase());
    if let Some(pos) = catalog.stt.iter().position(|m| m == default) {
        let d = catalog.stt.remove(pos);
        catalog.stt.insert(0, d);
    }
    catalog
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_openai_stt_chat_y_excluidos() {
        for id in [
            "whisper-1",
            "gpt-4o-transcribe",
            "gpt-4o-mini-transcribe",
            "gpt-4o-transcribe-diarize",
        ] {
            assert_eq!(openai::classify_model(id), Some(ModelKind::Stt), "{id}");
        }
        for id in [
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4.1",
            "gpt-4.1-mini",
            "gpt-5",
            "chatgpt-4o-latest",
            "o3-mini",
            "o4-mini",
        ] {
            assert_eq!(openai::classify_model(id), Some(ModelKind::Chat), "{id}");
        }
        for id in [
            "text-embedding-3-small",
            "tts-1",
            "gpt-4o-mini-tts",
            "dall-e-3",
            "gpt-image-1",
            "omni-moderation-latest",
            "gpt-4o-realtime-preview",
            "gpt-4o-audio-preview",
            "gpt-4o-search-preview",
            "computer-use-preview",
            "gpt-3.5-turbo-instruct",
            "davinci-002",
            "babbage-002",
            "codex-mini-latest",
            "sora-2",
        ] {
            assert_eq!(openai::classify_model(id), None, "{id}");
        }
    }

    #[test]
    fn classify_groq_stt_chat_y_excluidos() {
        for id in [
            "whisper-large-v3",
            "whisper-large-v3-turbo",
            "distil-whisper-large-v3-en",
        ] {
            assert_eq!(groq::classify_model(id), Some(ModelKind::Stt), "{id}");
        }
        for id in [
            "llama-3.3-70b-versatile",
            "llama-3.1-8b-instant",
            "meta-llama/llama-4-scout-17b-16e-instruct",
            "qwen/qwen3-32b",
            "deepseek-r1-distill-llama-70b",
            "gemma2-9b-it",
            "moonshotai/kimi-k2-instruct",
            "openai/gpt-oss-120b",
            "groq/compound",
            "allam-2-7b",
        ] {
            assert_eq!(groq::classify_model(id), Some(ModelKind::Chat), "{id}");
        }
        for id in [
            "playai-tts",
            "playai-tts-arabic",
            "meta-llama/llama-guard-4-12b",
            "meta-llama/llama-prompt-guard-2-86m",
        ] {
            assert_eq!(groq::classify_model(id), None, "{id}");
        }
    }

    #[test]
    fn build_catalog_ordena_con_default_primero() {
        let ids = vec![
            "whisper-1".to_string(),
            "gpt-4o-transcribe".to_string(),
            "gpt-4o-mini-transcribe".to_string(),
            "gpt-4o".to_string(),
            "tts-1".to_string(),
        ];
        // default "whisper-1" NO es el alfabéticamente primero: prueba que la
        // reordenación lo sube al tope.
        let catalog = build_catalog(ids, openai::classify_model, "whisper-1");
        assert_eq!(
            catalog.stt,
            vec!["whisper-1", "gpt-4o-mini-transcribe", "gpt-4o-transcribe"]
        );
        assert_eq!(catalog.chat, vec!["gpt-4o"]);
    }

    #[test]
    fn build_catalog_deduplica() {
        let ids = vec!["whisper-1".to_string(), "whisper-1".to_string()];
        let catalog = build_catalog(ids, openai::classify_model, "whisper-1");
        assert_eq!(catalog.stt, vec!["whisper-1"]);
    }

    #[test]
    fn build_catalog_vacio_si_nada_clasifica() {
        let ids = vec!["dall-e-3".to_string(), "tts-1".to_string()];
        let catalog = build_catalog(ids, openai::classify_model, "gpt-4o-mini-transcribe");
        assert!(catalog.stt.is_empty());
        assert!(catalog.chat.is_empty());
    }
}
