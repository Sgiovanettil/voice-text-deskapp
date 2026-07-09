# Plan de implementación — Listado dinámico de modelos por proveedor

- **Estado:** implementado (PR #73, 2026-07-09; ADR-0016) · **Fecha del plan:** 2026-07-09
- **Ejecutor previsto:** agente autónomo (clase Sonnet) sin acceso a la conversación de diseño.
  Este documento es autocontenido: todo lo necesario está aquí. Ante una discrepancia entre este
  plan y el código real (líneas movidas por commits posteriores), **manda el patrón descrito, no
  el número de línea**.
- **Rama:** `feature/dynamic-model-listing` desde `develop` (Git Flow).
- **PR:** a `develop`, squash merge. Título (será el commit de squash):
  `feat(providers): modelos dinámicos por proveedor (STT + chat)`.

## 0. Contexto y objetivo

Hoy la lista de modelos por proveedor está **hardcodeada** en el frontend
(`src/settings/App.tsx`, constante `MODELS_BY_PROVIDER`) y solo cubre modelos STT. Los catálogos
de OpenAI y Groq cambian con frecuencia (modelos nuevos, retiros), y el post-procesado LLM
diseñado en ADR-0014 (v2.x) necesitará además modelos conversacionales.

**Objetivo:** consultar en vivo `GET {base_url}/models` de cada proveedor y clasificar los
modelos en dos categorías:

1. **STT** — alimenta el selector de modelo existente en la ventana de settings.
2. **Chat** — queda disponible en el backend y el DTO IPC para el ADR-0014 (sin UI todavía).

Requisitos acordados con el usuario:

- **Sin fallback estático:** si no hay API key guardada, no hay red o el proveedor falla, el
  selector queda vacío y deshabilitado con un aviso i18n. `MODELS_BY_PROVIDER` se elimina.
- **Refresco:** al abrir settings, al cambiar de proveedor, al guardar una API key, y con un
  botón manual de refresco.
- **Nunca pisar config silenciosamente:** si el modelo guardado ya no aparece en el catálogo,
  se conserva como opción extra marcada "(no disponible)".

## 1. Reglas del repo que el ejecutor DEBE respetar

- Leer `AGENTS.md` (raíz) antes de empezar. Git Flow, PR obligatorio, Conventional Commits
  (commitlint + husky los valida en cada commit).
- **Prohibido** agregar trailers `Co-Authored-By`, firmas "Generated with..." o cualquier
  atribución de IA en commits y PR (regla fija del dueño del repo).
- Todo el negocio en Rust (ADR-0002): la clasificación de modelos vive en el backend; el
  frontend solo presenta.
- Toda cadena visible en la UI pasa por i18n (`react-i18next`); ESLint bloquea literales JSX.
  Los ids de modelo/proveedor son identificadores y NO se traducen.
- Catálogo de eventos IPC (`src-tauri/src/core/events.rs` + `src/shared/events.ts`): **no se
  toca**. Este plan no introduce eventos nuevos (la ventana de settings usa `invoke` directo).
- Tipos espejo TS (`src/shared/settings.ts`): cambian en el mismo PR que el lado Rust.
- Commitear siempre sobre la rama feature, nunca sobre `develop` (tras el squash-merge, un
  `develop` local con commits propios diverge del remoto).
- El working tree puede tener archivos modificados ajenos a esta tarea (p. ej. `AGENTS.md`):
  **no incluirlos** en los commits; agregar archivos de forma explícita con `git add <ruta>`.

Verificación local antes de cada push (todas deben pasar):

```bash
cd src-tauri && cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
cd .. && npm run lint && npm run format:check && npm test && npm run build
```

Si `format:check` falla por archivos nuevos, correr `npm run format` y re-verificar. Para Rust,
`cargo fmt --all` antes del commit.

## 2. Decisiones de diseño (ya tomadas — no re-decidir)

| Punto | Decisión |
|---|---|
| Dónde vive el fetch HTTP | Método inherente `fetch_model_ids()` en `OpenAiCompatibleProvider` (`src-tauri/src/providers/openai_compatible.rs`). **NO** se toca el trait `SpeechProvider`: listar modelos chat no es una capacidad de speech (ADR-0003 intacto). Ambos proveedores comparten el contrato `GET /models` (ADR-0012) |
| Orquestación | `providers::list_models(provider_id, api_key)` en `src-tauri/src/providers/mod.rs`; id desconocido cae a OpenAI (mismo criterio que `resolve()`) |
| Clasificación STT/chat | Función pura `classify_model(id) -> Option<ModelKind>` por proveedor, en `providers/openai/mod.rs` y `providers/groq/mod.rs`. STT por **allowlist** (un falso positivo rompe la transcripción); chat por **denylist** (un modelo nuevo desconocido entra a chat: aún no maneja tráfico real) |
| Orden del catálogo | STT: default del proveedor primero (la UI asume "primero = default"), resto alfabético case-insensitive. Chat: alfabético case-insensitive. Deduplicado. Groq: se descartan modelos con `"active": false` |
| Sin key guardada | Error IPC dedicado `err.models.noKey` (≠ de los `err.stt.*`): la UI distingue "configura tu key" de "reintenta" |
| Modelo guardado ausente | Opción extra al tope del select, etiquetada con i18n `settings.stt.modelUnavailable` (`"{{model}} (no disponible)"`), seleccionada. No se modifica `settings.stt.model` |
| Reset al cambiar proveedor | La UI conserva un espejo mínimo `DEFAULT_MODEL_BY_PROVIDER` (2 entradas, espejo de `providers::default_model`); el catálogo dinámico no puede ser la fuente del default porque sin key no hay catálogo |
| 429 (rate limit) | Sin reintento automático en el listado (a diferencia de `transcribe`); el usuario tiene botón de refresco |
| Eventos de dominio | Ninguno nuevo |
| ADR | Nuevo **ADR-0016** (el número 0015 está RESERVADO para la pantalla de gastos, issue #60 — no usarlo aunque el archivo no exista aún) |

---

## FASE 1 — Backend Rust: fetch y clasificación

Commit sugerido al cierre de la fase:
`feat(providers): fetch y clasificación de modelos por proveedor`.

### Paso 1.1 — `src-tauri/src/providers/mod.rs`: tipos y orquestación

El archivo hoy tiene ~34 líneas: doc-comment del módulo, `pub mod groq/openai/openai_compatible`,
`use crate::speech::SpeechProvider;`, `KNOWN`, `resolve()` y `default_model()`.

1. Ampliar el import: `use crate::speech::{SpeechError, SpeechProvider};`
2. Añadir al final del archivo:

```rust
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
pub async fn list_models(
    provider_id: &str,
    api_key: String,
) -> Result<ModelCatalog, SpeechError> {
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
```

Nota: ambos brazos del `if` devuelven el tipo concreto `OpenAiCompatibleProvider` (así lo
construyen `openai::provider()` y `groq::provider()`), por eso el método inherente
`fetch_model_ids` es alcanzable sin pasar por el trait.

### Paso 1.2 — `src-tauri/src/providers/openai_compatible.rs`: fetch HTTP

Añadir dentro del bloque `impl OpenAiCompatibleProvider` existente, inmediatamente después de
`check_auth_impl` (que termina cerca de la línea 70) y antes de `request_once`. El mapeo de
errores es **idéntico** al de `check_auth_impl` (misma casuística, mismo timeout de 10 s):

```rust
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
```

### Paso 1.3 — Heurística OpenAI: `src-tauri/src/providers/openai/mod.rs`

La API de OpenAI no expone tipo/capacidad del modelo, así que se clasifica por patrones del id.
Añadir al final del archivo (y el `use` junto al existente):

```rust
use crate::providers::ModelKind;

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
```

Comportamiento esperado con ids reales (estos ejemplos van a los tests del paso 1.5):

- **STT:** `whisper-1`, `gpt-4o-transcribe`, `gpt-4o-mini-transcribe`, `gpt-4o-transcribe-diarize`.
- **Chat:** `gpt-4o`, `gpt-4o-mini`, `gpt-4.1`, `gpt-4.1-mini`, `gpt-5`, `chatgpt-4o-latest`,
  `o3-mini`, `o4-mini`.
- **Excluidos (`None`):** `text-embedding-3-small`, `tts-1`, `gpt-4o-mini-tts`, `dall-e-3`,
  `gpt-image-1`, `omni-moderation-latest`, `gpt-4o-realtime-preview`, `gpt-4o-audio-preview`,
  `gpt-4o-search-preview`, `computer-use-preview`, `gpt-3.5-turbo-instruct`, `davinci-002`,
  `babbage-002`, `codex-mini-latest`, `sora-2`.

El orden de los checks importa: el allowlist STT va PRIMERO (así `gpt-4o-mini-transcribe` no cae
en el prefijo `gpt-` de chat, y `whisper-*` no se evalúa contra la denylist).

### Paso 1.4 — Heurística Groq: `src-tauri/src/providers/groq/mod.rs`

Añadir al final del archivo:

```rust
use crate::providers::ModelKind;

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
```

Ojo: la denylist de Groq es DISTINTA a la de OpenAI a propósito — en Groq `instruct` es un
sufijo normal de modelos de chat (`moonshotai/kimi-k2-instruct`, `gemma2-9b-it`), no un
descarte.

Comportamiento esperado con ids reales:

- **STT:** `whisper-large-v3`, `whisper-large-v3-turbo`, `distil-whisper-large-v3-en`.
- **Chat:** `llama-3.3-70b-versatile`, `llama-3.1-8b-instant`,
  `meta-llama/llama-4-scout-17b-16e-instruct`, `qwen/qwen3-32b`,
  `deepseek-r1-distill-llama-70b`, `gemma2-9b-it`, `moonshotai/kimi-k2-instruct`,
  `openai/gpt-oss-120b`, `groq/compound`, `allam-2-7b`.
- **Excluidos (`None`):** `playai-tts`, `playai-tts-arabic`, `meta-llama/llama-guard-4-12b`,
  `meta-llama/llama-prompt-guard-2-86m`.

### Paso 1.5 — Tests Rust

**A. Tests HTTP con wiremock**, en el módulo `#[cfg(test)] mod tests` ya existente al final de
`openai_compatible.rs`. Reutilizar el helper `provider_for(&server)` que ya está definido ahí
(construye un `OpenAiCompatibleProvider` apuntando al mock con `with_base_url`). Añadir estos
7 tests (mismo estilo que los existentes):

```rust
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

        let err = provider_for(&server).await.fetch_model_ids().await.unwrap_err();
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

        let err = provider_for(&server).await.fetch_model_ids().await.unwrap_err();
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

        let err = provider_for(&server).await.fetch_model_ids().await.unwrap_err();
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

        let err = provider_for(&server).await.fetch_model_ids().await.unwrap_err();
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
```

**B. Tests puros de clasificación y catálogo**, en un módulo `#[cfg(test)] mod tests` NUEVO al
final de `src-tauri/src/providers/mod.rs`:

```rust
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
```

**Criterio de aceptación Fase 1:** desde `src-tauri/`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings` y `cargo test --workspace` en verde
(los 12 tests nuevos incluidos).

---

## FASE 2 — Comando IPC y espejo TS

Commit sugerido: `feat(ipc): comando list_models con catálogo stt/chat`.

### Paso 2.1 — `src-tauri/src/ipc/commands.rs`: comando `list_models`

Insertar inmediatamente después de `test_provider` (que termina con `Ok(true)` cerca de la
línea 220), calcando su estructura. `persistence` ya está en scope en ese archivo:

```rust
/// Modelos disponibles del proveedor, consultados en vivo a su API y
/// clasificados en STT (selector de settings) y chat (post-procesado LLM
/// futuro, ADR-0014). Sin key guardada devuelve `err.models.noKey` para que
/// la UI pida configurar la clave en vez de sugerir reintentar.
#[tauri::command]
pub async fn list_models(provider: String) -> Result<crate::providers::ModelCatalog, IpcError> {
    let key = persistence::get_api_key(&provider)?
        .ok_or_else(|| IpcError::new("api key no configurada", "err.models.noKey"))?;
    crate::providers::list_models(&provider, key)
        .await
        .map_err(|e| {
            let key = match e {
                crate::speech::SpeechError::Auth => "err.stt.auth",
                crate::speech::SpeechError::Network => "err.stt.network",
                crate::speech::SpeechError::RateLimited => "err.stt.rate",
                _ => "err.stt.provider",
            };
            IpcError::new(e.to_string(), key)
        })
}
```

Notas:

- El `?` sobre `get_api_key` usa el `From<PersistenceError> for IpcError` existente: keyring
  roto → `err.keyring.unavailable`. Eso es correcto y distinto de "key ausente" (`Ok(None)` →
  `err.models.noKey`).
- Timeout total efectivo: los 10 s internos de `fetch_model_ids`. No añadir otro.

### Paso 2.2 — `src-tauri/src/lib.rs`: registrar el comando

En el bloque `tauri::generate_handler![...]` (líneas ~195-207), insertar
`ipc::commands::list_models,` en la línea siguiente a `ipc::commands::test_provider,`.

### Paso 2.3 — `src/shared/settings.ts`: tipo espejo

Añadir al final del archivo (después de `IpcError`):

```ts
// Espejo de providers::ModelCatalog (serde camelCase): modelos del proveedor
// clasificados en vivo. `chat` queda para el post-procesado LLM (ADR-0014).
export interface ModelCatalog {
  stt: string[];
  chat: string[];
}
```

**Criterio de aceptación Fase 2:** `cargo check` (en `src-tauri/`) y `npm run build` (raíz) en
verde.

---

## FASE 3 — Frontend: selector dinámico en settings

Todo en `src/settings/App.tsx` salvo i18n (3.5) y tests (3.6).
Commit sugerido: `feat(settings): selector de modelos dinámico con refresco manual`.

### Paso 3.1 — Constantes e imports

1. En el import de React (línea 1) añadir `useCallback`:
   `import { useCallback, useEffect, useState } from "react";`
2. En el import de tipos de `../shared/settings` (líneas 7-15) añadir `ModelCatalog`.
3. **Eliminar** la constante `MODELS_BY_PROVIDER` completa (líneas 28-33, incluido su
   comentario de las líneas 28-29) y reemplazarla por:

```ts
// Espejo de providers::default_model (el backend manda): solo para resetear
// el modelo al cambiar de proveedor; la lista real llega por `list_models`.
const DEFAULT_MODEL_BY_PROVIDER: Record<string, string> = {
  openai: "gpt-4o-mini-transcribe",
  groq: "whisper-large-v3-turbo",
};
```

### Paso 3.2 — Estado y `fetchModels`

Junto a las declaraciones de estado existentes (después de `const [micSpeaking, ...]`, ~línea
71), añadir:

```ts
  // Catálogo de modelos del proveedor, consultado en vivo. Sin fallback
  // estático: null = sin datos (sin key, sin red o proveedor caído).
  const [models, setModels] = useState<ModelCatalog | null>(null);
  const [modelsLoading, setModelsLoading] = useState(false);
  // errorKey i18n del último fetch fallido (err.models.noKey, err.stt.*).
  const [modelsError, setModelsError] = useState<string | null>(null);

  const fetchModels = useCallback((forProvider: string) => {
    setModelsLoading(true);
    setModelsError(null);
    invoke<ModelCatalog>("list_models", { provider: forProvider })
      .then(setModels)
      .catch((e: unknown) => {
        setModels(null);
        const err = e as IpcError;
        setModelsError(err?.errorKey ?? "err.stt.provider");
      })
      .finally(() => setModelsLoading(false));
  }, []);
```

`useCallback` con deps vacías: solo usa `invoke` (import de módulo) y setters (estables). Así
puede entrar en las deps del efecto de montaje sin re-disparos ni warnings de
`react-hooks/exhaustive-deps`.

### Paso 3.3 — Disparadores del fetch (los 4 acordados)

1. **Montaje** — en el `useEffect` inicial (~líneas 73-93), dentro del `.then((s) => { ... })`
   de `get_settings`, después del bloque que pide `get_api_key_status`, añadir:
   `fetchModels(s.stt.provider);` — y cambiar el array de deps del efecto de `[i18n]` a
   `[i18n, fetchModels]`.
2. **Cambio de proveedor** — reemplazar `changeProvider` completo (~líneas 240-252) por:

```ts
  // Cambia de proveedor: resetea el modelo al default del nuevo proveedor,
  // refresca el estado de la key (cada proveedor tiene la suya) y repuebla el
  // catálogo de modelos.
  const changeProvider = (next: string) => {
    if (!settings) return;
    const model = DEFAULT_MODEL_BY_PROVIDER[next] ?? settings.stt.model;
    const updated: Settings = { ...settings, stt: { ...settings.stt, provider: next, model } };
    invoke("set_settings", { settings: updated })
      .then(() => {
        setSettings(updated);
        setApiKeyInput("");
        refreshKeyStatus(next);
        setModels(null); // el catálogo del proveedor anterior ya no vale
        fetchModels(next);
        setFeedback({ kind: "ok", text: t("settings.stt.saved") });
      })
      .catch(showError);
  };
```

3. **Guardar API key** — en `saveApiKey` (~líneas 228-236), dentro del `.then((status) => {...})`,
   añadir `fetchModels(provider);` después de `setKeyStatus(status);`. (Cubre también el caso
   key vacía → `set_api_key` borra la key → el fetch siguiente devuelve `err.models.noKey`.)
4. **Botón manual** — parte del JSX del paso 3.4.

### Paso 3.4 — Derivadas y JSX del selector

Junto a las derivadas existentes (`const provider = ...`, ~línea 219), añadir:

```ts
  // Modelos STT del catálogo vivo; el modelo guardado se conserva como opción
  // extra si el proveedor ya no lo lista (no se pisa config silenciosamente).
  const sttModels = models?.stt ?? [];
  const savedModelMissing =
    !!settings && settings.stt.model !== "" && !sttModels.includes(settings.stt.model);
```

En la sección `recognition`, reemplazar SOLO el `<label className="field">` del selector de
modelo (hoy ~líneas 525-538, el que itera `MODELS_BY_PROVIDER[provider]`) por:

```tsx
              <label className="field">
                {t("settings.stt.model")}
                <select
                  value={settings?.stt.model ?? ""}
                  disabled={
                    !settings || modelsLoading || (sttModels.length === 0 && !savedModelMissing)
                  }
                  onChange={(e) => patchStt({ model: e.target.value }, "settings.stt.saved")}
                >
                  {savedModelMissing && (
                    <option value={settings?.stt.model ?? ""}>
                      {t("settings.stt.modelUnavailable", { model: settings?.stt.model })}
                    </option>
                  )}
                  {sttModels.map((m) => (
                    <option key={m} value={m}>
                      {m}
                    </option>
                  ))}
                </select>
              </label>
              <div className="row">
                <button
                  type="button"
                  onClick={() => fetchModels(provider)}
                  disabled={modelsLoading}
                  aria-label={t("settings.stt.modelsRefresh")}
                >
                  {modelsLoading ? t("settings.stt.modelsLoading") : t("settings.stt.modelsRefresh")}
                </button>
              </div>
              {modelsError && <p className="feedback-error">{t(modelsError)}</p>}
              {!modelsLoading && !modelsError && models !== null && sttModels.length === 0 && (
                <p className="hint">{t("settings.stt.modelsEmpty")}</p>
              )}
```

El selector de idioma que sigue (`settings.stt.language`) queda intacto. Las clases `.row`,
`.hint` y `.feedback-error` ya existen en `src/settings/App.css` — no tocar CSS.

Comportamiento resultante (verificarlo a mano si hay entorno gráfico):

- Sin key: select vacío y deshabilitado + mensaje de `err.models.noKey` + botón habilitado.
- Fallo de red: mensaje `err.stt.network`; si hay modelo guardado, se muestra "(no disponible)".
- Cargando: select y botón deshabilitados; el botón dice "Cargando modelos…".
- Éxito: opciones = catálogo `stt` (default del proveedor primero, orden del backend).

### Paso 3.5 — i18n (es + en, mismo commit)

`src/i18n/es/translation.json` — el bloque `settings.stt` queda así (se añaden 4 claves entre
`"saved"` y `"languages"`):

```json
  "stt": {
   "label": "Transcripción",
   "model": "Modelo",
   "language": "Idioma del dictado",
   "saved": "Preferencia de transcripción guardada.",
   "modelsRefresh": "Actualizar modelos",
   "modelsLoading": "Cargando modelos…",
   "modelsEmpty": "El proveedor no devolvió modelos compatibles.",
   "modelUnavailable": "{{model}} (no disponible)",
   "languages": {
    "auto": "Detección automática",
    "es": "Español",
    "en": "Inglés"
   }
  }
```

Y en el bloque `err` del mismo archivo (que hoy contiene `stt`, `audio`, `hotkey`, `delivery`,
`keyring`, `config`, `update`), insertar entre `"config"` y `"update"`:

```json
  "models": {
   "noKey": "Guarda tu clave de API para ver los modelos disponibles."
  }
```

`src/i18n/en/translation.json` — mismas posiciones:

```json
   "modelsRefresh": "Refresh models",
   "modelsLoading": "Loading models…",
   "modelsEmpty": "The provider returned no compatible models.",
   "modelUnavailable": "{{model}} (unavailable)"
```

```json
  "models": {
   "noKey": "Save your API key to see the available models."
  }
```

Respetar la indentación real de cada archivo (verificar con prettier: `npm run format`).

### Paso 3.6 — Tests frontend: `src/settings/App.test.tsx`

**A. Refactor del mock (obligatorio):** el mock actual de `@tauri-apps/api/core` rechaza los
comandos no contemplados, así que sin el caso `list_models` TODOS los tests que rendericen la
app dispararían un rechazo al montar. Extraer el switch a una función con nombre (las function
declarations sí son alcanzables desde la factory hoisted de `vi.mock`) y resetear en
`beforeEach` para que las sobre-escrituras por test no contaminen a los demás:

```ts
// jsdom no tiene el runtime de Tauri: se simula el borde IPC por comando.
function defaultInvoke(cmd: string): Promise<unknown> {
  switch (cmd) {
    case "get_settings":
      return Promise.resolve({
        /* ... payload existente sin cambios ... */
      });
    case "get_api_key_status":
      return Promise.resolve({ isSet: true, masked: "…1234" });
    case "list_input_devices":
      return Promise.resolve(["Micrófono interno", "USB Mic"]);
    case "list_models":
      return Promise.resolve({
        stt: ["gpt-4o-mini-transcribe", "gpt-4o-transcribe", "whisper-1"],
        chat: ["gpt-4o", "gpt-4o-mini"],
      });
    case "set_settings":
    case "start_mic_test":
    case "stop_mic_test":
      return Promise.resolve();
    default:
      return Promise.reject(new Error(`comando no mockeado: ${cmd}`));
  }
}
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => defaultInvoke(cmd)),
}));
```

y dentro del `describe`, antes de los tests:

```ts
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockImplementation((cmd: string) => defaultInvoke(cmd));
  });
```

(Importar `beforeEach` desde `vitest`. Revisar que los tests existentes sigan en verde: el
`beforeEach` es equivalente al estado inicial de la factory.)

**B. Tests nuevos** (nombres en español, estilo del archivo). Los selects se localizan por
nombre accesible: el de proveedor es `combobox` "Proveedor", el de modelo es `combobox`
"Modelo" (la app testea con i18n en español por defecto):

1. `pide los modelos al montar y los muestra en el selector`:
   render, click en botón "Reconocimiento", luego
   `await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("list_models", { provider: "openai" }))`;
   el `combobox` "Modelo" contiene la opción `whisper-1` y su primera opción es
   `gpt-4o-mini-transcribe` (contrato "primero = default").
2. `al cambiar de proveedor repuebla modelos y resetea el default`:
   `fireEvent.change` del combobox "Proveedor" a `"groq"`; `waitFor` →
   `invokeMock` recibió `("set_settings", ...)` cuyo `settings.stt` tiene
   `provider === "groq"` y `model === "whisper-large-v3-turbo"`, y recibió
   `("list_models", { provider: "groq" })`.
3. `sin api key muestra el aviso y deshabilita el selector`:
   en el test, sobreescribir el mock:

   ```ts
   invokeMock.mockImplementation((cmd: string) => {
     if (cmd === "get_api_key_status") return Promise.resolve({ isSet: false, masked: null });
     if (cmd === "list_models")
       return Promise.reject({ code: "api key no configurada", errorKey: "err.models.noKey" });
     return defaultInvoke(cmd);
   });
   ```

   render + click "Reconocimiento";
   `await screen.findByText("Guarda tu clave de API para ver los modelos disponibles.")`;
   el combobox "Modelo" está `toBeDisabled()`.
4. `el botón de refresco vuelve a pedir los modelos`:
   render + click "Reconocimiento", esperar el primer fetch, `invokeMock.mockClear()`, click en
   botón "Actualizar modelos", `waitFor` → `invokeMock` llamado con
   `("list_models", { provider: "openai" })`.
5. `el modelo guardado ausente del catálogo se conserva marcado`:
   sobreescribir solo `get_settings` para devolver el payload base con
   `stt: { provider: "openai", model: "modelo-retirado", language: "auto" }` (el resto vía
   `defaultInvoke`); render + click "Reconocimiento"; `await screen.findByRole("option",
   { name: "modelo-retirado (no disponible)" })`; el combobox "Modelo" tiene
   `toHaveValue("modelo-retirado")` y NO está deshabilitado.

**Criterio de aceptación Fase 3:** `npm test`, `npm run lint`, `npm run format:check` y
`npm run build` en verde (tests preexistentes incluidos).

---

## FASE 4 — Documentación

Commit sugerido: `docs(adr): ADR-0016 listado dinámico de modelos`.

### Paso 4.1 — Nuevo ADR: `docs/2-arquitectura/DECISIONS/0016-listado-dinamico-de-modelos.md`

**El número es 0016 aunque no exista un 0015:** el 0015 está reservado para la pantalla de
gastos (issue #60). Contenido completo del archivo (ajustar `#NN` al número real del PR):

```markdown
# ADR-0016 — Listado dinámico de modelos por proveedor (v1.x)

- **Estado:** Implementada (v1.x, #NN) · **Fecha:** <fecha del merge>

## Contexto

La UI de settings hardcodeaba los modelos por proveedor (`MODELS_BY_PROVIDER` en el frontend),
solo STT. Los catálogos de OpenAI y Groq cambian con frecuencia (altas y retiros), y el
post-procesado LLM (ADR-0014, v2.x) necesitará modelos conversacionales. Ambos proveedores
exponen el mismo contrato `GET {base_url}/models` (ADR-0012), aunque sin campo de
tipo/capacidad: la categoría de cada modelo debe inferirse del id.

## Decisión

- **Consulta en vivo** a `GET {base_url}/models` con la key del keyring, vía un método
  inherente `fetch_model_ids()` en `OpenAiCompatibleProvider` (el trait `SpeechProvider` no se
  toca: listar modelos chat no es una capacidad de speech).
- **Clasificación heurística por proveedor** en Rust (`classify_model(id) -> Option<ModelKind>`
  en `providers/openai` y `providers/groq`): STT por **allowlist** (un falso positivo rompería
  la transcripción), chat por **denylist** (un modelo nuevo desconocido entra a chat: esa
  categoría aún no maneja tráfico). Groq descarta además modelos con `active: false`.
- **Comando IPC `list_models(provider)`** que devuelve `{ stt: string[], chat: string[] }`
  (`ModelCatalog`), con `stt` ordenado default-primero y el error dedicado `err.models.noKey`
  cuando no hay key guardada (la UI distingue "configura tu key" de "reintenta").
- **Sin fallback estático:** sin key/red el selector queda vacío y deshabilitado con aviso; el
  modelo guardado ausente del catálogo se preserva como opción "(no disponible)". El frontend
  solo conserva el espejo mínimo de los modelos default por proveedor (reset al cambiar de
  proveedor).
- **Refresco:** al abrir settings, al cambiar de proveedor, al guardar una key y botón manual.
  Sin caché ni reintento automático ante 429.
- **Cero eventos de dominio nuevos** (ADR-0009 intacto): la ventana de settings usa invoke.

## Consecuencias

- (+) El selector refleja el catálogo real del proveedor; los modelos nuevos aparecen solos.
- (+) `chat` queda listo para el post-procesado LLM (ADR-0014) sin más cambios de contrato.
- (−) Sin key o sin red no hay lista (decisión explícita del usuario: aviso + reintento
  manual, sin lista estática que pueda quedar obsoleta).
- (−) La heurística por id puede clasificar mal un modelo futuro; mitigación: STT allowlist
  conservadora, chat denylist inclusiva, y la opción "(no disponible)" preserva la config del
  usuario ante retiros.
```

### Paso 4.2 — Índice de ADRs: `docs/2-arquitectura/DECISIONS/README.md`

Añadir al final de la tabla (después de la fila 0014; si al momento de implementar ya existe la
fila 0015 de gastos, va después de ella):

```markdown
| [0016](0016-listado-dinamico-de-modelos.md) | Listado dinámico de modelos por proveedor (v1.x) | Implementada (v1.x, #NN) |
```

### Paso 4.3 — `docs/2-arquitectura/ARCHITECTURE.md`: catálogo de comandos IPC

En la línea que enumera los comandos (hoy línea 191):

- Antes: `- **Comandos** (frontend → core): \`get_settings\`, \`set_settings\`, \`set_api_key\`, \`test_provider\`, \`get_app_state\`. Validación en el borde; errores mapeados a \`{ code, error_key }\`.`
- Después: la misma línea con `\`list_models\`` añadido entre `\`test_provider\`` y
  `\`get_app_state\``.

### Paso 4.4 — `docs/README.md`: conteo de ADRs

La fila de `2-arquitectura/DECISIONS/` menciona el número de ADRs ("14 decisiones de
arquitectura"). Actualizar el número al total real tras añadir el 0016.

---

## 5. Riesgos y bordes (decisiones ya tomadas — no improvisar)

1. **Key inválida a mitad de sesión:** `list_models` devuelve `err.stt.auth` y se muestra bajo
   el selector. El flujo de transcripción no cambia (ya maneja `Auth`).
2. **Proveedor desconocido en config:** `providers::list_models` cae a OpenAI, mismo criterio
   que `resolve()`.
3. **Respuesta 200 vacía o sin modelos clasificables:** NO es error — catálogo con vectores
   vacíos → hint `settings.stt.modelsEmpty`, select deshabilitado (salvo el modelo guardado,
   que se preserva).
4. **429:** sin reintento automático; el usuario reintenta con el botón. `err.stt.rate` ya
   existe en i18n.
5. **No tocar:** el trait `SpeechProvider`, `core/events.rs`, `src/shared/events.ts`, los
   snapshots `insta`, el flujo de transcripción, `check_auth`/`test_provider`.
6. **`ModelCatalog` es solo aditivo hacia el futuro:** cuando el ADR-0014 se implemente podrá
   agregar campos, nunca renombrar `stt`/`chat` (misma disciplina que el catálogo de eventos).

## 6. Cierre: verificación global y PR

1. Correr la verificación completa (sección 1). Todo en verde.
2. Revisar el diff completo (`git diff develop...HEAD`) contra este plan: sin archivos ajenos a
   la tarea, sin `MODELS_BY_PROVIDER` residual (`grep -rn "MODELS_BY_PROVIDER" src/` debe
   devolver vacío).
3. PR a `develop` con squash. Título: `feat(providers): modelos dinámicos por proveedor (STT + chat)`.
   Cuerpo: resumen de las 4 fases + referencia a este plan
   (`docs/3-desarrollo/PLAN_MODELOS_DINAMICOS.md`) y al ADR-0016. Sin atribuciones de IA.
4. CI de PR (lint/test) en verde antes del merge.
5. **Validación manual del usuario (Windows):** tras el merge, generar instalador con
   `gh workflow run ci.yml --ref develop` para que el usuario pruebe: abrir settings →
   Reconocimiento con key guardada (lista real), sin key (aviso), cambiar OpenAI↔Groq, botón
   "Actualizar modelos", y una transcripción end-to-end con un modelo elegido de la lista.
