# ADR-0016 — Listado dinámico de modelos por proveedor (v1.x)

- **Estado:** Implementada (v1.x, #73) · **Fecha:** 2026-07-09

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
