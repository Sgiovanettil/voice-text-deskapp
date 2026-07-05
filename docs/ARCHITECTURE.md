# Arquitectura — VoiceText (nombre provisional)

- **Versión:** 0.1.0 · **Fecha:** 2026-07-02 · **Estado:** En revisión
- **Documentos relacionados:** [PRD](PRD.md) · [ADRs](adr/)

Este documento detalla la arquitectura técnica del sistema descrito en el PRD. No contiene código de implementación; los fragmentos en Rust/TypeScript son **contratos conceptuales**.

---

## 1. Vista general

```
┌────────────────────────── Tauri App ──────────────────────────┐
│                                                               │
│  WebView Overlay        WebView Settings                      │
│  (React, sin foco)      (React + i18n)                        │
│        ▲                      ▲ ▼                             │
│   eventos (IPC)         comandos (IPC)                        │
│        │                      │                               │
│  ┌─────┴──────────────────────┴───────────────────────────┐   │
│  │                    ipc/  (borde Tauri)                 │   │
│  ├────────────────────────────────────────────────────────┤   │
│  │ core/ — máquina de estados + despachador de eventos    │   │
│  │   ▲emite/consume eventos de dominio (ADR-009)          │   │
│  │   │                                                    │   │
│  │ hotkeys/   audio/   speech/ ──► providers/openai/      │   │
│  │ delivery/  config/  persistence/  i18n(claves error)   │   │
│  │ logging (tracing, transversal)                         │   │
│  └────────────────────────────────────────────────────────┘   │
│  Plugins Tauri: global-shortcut, tray, autostart,             │
│  clipboard-manager, (updater: preparado, inactivo)            │
└───────────────────────────────────────────────────────────────┘
```

Reglas estructurales (de los principios del PRD §2):

1. **Todo el negocio en Rust** (ADR-002). El frontend solo presenta estado y formularios.
2. **Comunicación por eventos de dominio** (ADR-009): los módulos de negocio no se invocan entre sí; emiten eventos que el core orquesta.
3. **Capacidades tras traits** (ADR-003): el core conoce `SpeechProvider`, nunca `OpenAiProvider`.
4. **Plataforma aislada** en `delivery/` y detrás de plugins Tauri: ningún `#[cfg(target_os)]` fuera de los módulos de plataforma.

---

## 2. Eventos de dominio (catálogo v1)

Los eventos son el contrato público interno. Tipados, serializables (serde), con nombre y payload estables. En MVP los despacha el core (sin Event Bus formal); el puente `ipc/` reenvía al frontend los que afectan a la UI.

| Evento | Emisor | Payload (esencial) | UI |
|---|---|---|---|
| `HotkeyPressed` | hotkeys | `timestamp` | — |
| `HotkeyReleased` | hotkeys | `timestamp` | — |
| `OverlayOpened` | core | `mode` | ✔ |
| `RecordingStarted` | audio | `device_id, sample_rate` | ✔ |
| `RecordingStopped` | audio | `duration_ms, samples` | ✔ |
| `RecordingFailed` | audio | `error_key, detail` | ✔ |
| `TranscriptionStarted` | core | `provider_id, model` | ✔ |
| `TranscriptionCompleted` | speech | `text, latency_ms, provider_id` | ✔ |
| `TranscriptionFailed` | speech | `error_key, retryable, detail` | ✔ |
| `TextDeliveryStarted` | core | `mode (insert\|clipboard)` | ✔ |
| `TextDeliveryCompleted` | delivery | `mode, chars` | ✔ |
| `TextDeliveryFailed` | delivery | `error_key, fallback_used` | ✔ |
| `OverlayClosed` | core | `outcome (ok\|error\|cancelled)` | ✔ |
| `ConfigChanged` | config | `changed_keys` | ✔ |
| `UpdateAvailable` | updater | `version, notes, pub_date` | ✔ |
| `UpdateDownloadProgress` | updater | `downloaded, content_length` | ✔ |
| `UpdateFailed` | updater | `error_key, detail` | ✔ |

Convenciones:

- Nombres en pasado (hechos consumados), payloads planos, sin referencias a tipos de proveedor concretos.
- `error_key` es una clave i18n; el detalle técnico va solo al log.
- Evolución: se pueden **añadir** campos y eventos; nunca renombrar ni cambiar semántica de los existentes (los futuros suscriptores — LLM, plugins, telemetría opt-in — dependen de esto).
- Post-MVP el despachador del core se sustituye por un Event Bus con suscripción dinámica **sin cambiar el catálogo**.

## 3. Máquina de estados del core

```
            HotkeyPressed                RecordingStopped
  ┌──────┐ ───────────────► ┌───────────┐ ─────────────► ┌──────────────┐
  │ Idle │                  │ Recording │                │ Transcribing │
  └──────┘ ◄─── (reset) ─── └───────────┘                └──────────────┘
     ▲            ▲                │ RecordingFailed            │ TranscriptionCompleted
     │            │                ▼                            ▼
     │        ┌───────┐   TranscriptionFailed /        ┌────────────┐
     └─────── │ Error │ ◄───TextDeliveryFailed──────── │ Delivering │
    (timeout  └───────┘                                └────────────┘
     ~3 s)                                                   │ TextDeliveryCompleted
                                                             ▼
                                                       Completed → Idle (~1 s)
```

Reglas:

- Estado global único; un `HotkeyPressed` durante un ciclo activo se ignora (MVP) — evita reentradas.
- Dictado máximo: 120 s (corte con transición a `Transcribing` y aviso en overlay).
- Grabación < 300 ms se descarta como pulsación accidental (a `Idle`, sin llamar al proveedor).
- Cancelación (v1.x, reservado): `Esc` durante `Recording` → `OverlayClosed(cancelled)`.
- Todo estado tiene timeout de seguridad (p. ej. `Transcribing` = timeout del proveedor + margen) para no quedar colgado.

## 4. Módulos

### 4.1 `core/`
Máquina de estados (§3) + despachador de eventos. Sin I/O propio: recibe eventos, decide, ordena (vía comandos internos a audio/speech/delivery) y emite eventos. Es el único módulo que conoce la secuencia del ciclo.

Post-MVP: entre transcripción y entrega se insertan etapas de post-procesamiento del texto — diccionario personal (v1.x, [ADR-0013](adr/0013-diccionario-personal.md)) y pasada LLM opcional según modo de dictado (v2.x, [ADR-0014](adr/0014-modos-dictado-postprocesado-llm.md)).

### 4.2 `hotkeys/`
Traduce press/release del acelerador configurado a `HotkeyPressed/Released`; valida y re-registra ante `ConfigChanged`. Diseñado como **abstracción con backends por plataforma** (ADR-004, Wayland de primera clase):

- **Windows / X11:** `tauri-plugin-global-shortcut` (registro directo).
- **Wayland:** XDG Desktop Portal `GlobalShortcuts` (aprobación única del usuario vía diálogo del portal).
- **Wayland sin portal:** activación externa — la app expone `app --dictate` (CLI/D-Bus) para asociarlo a un atajo nativo del compositor; la UI guía la configuración. Nota: en esta vía el press/release no está disponible, por lo que la activación externa opera en modo toggle implícito (inicio/fin por invocación) — única excepción documentada al PTT puro.

El backend activo y su capacidad se publican como estado (`ConfigChanged`/estado de la app) para que la UI comunique degradaciones explícitas.

Post-MVP (v1.x): modo de activación `toggle` con corte por VAD, mismo hotkey y mismos eventos `HotkeyPressed/Released` como disparadores — spec en [ADR-0011](adr/0011-activacion-toggle-vad.md).

### 4.3 `audio/`
- Captura con `cpal`, dispositivo por defecto del sistema. **Selección de micrófono (v1.x):** enumeración de dispositivos de entrada vía `cpal`, persistencia por **nombre de dispositivo** en settings con fallback al default del SO si el dispositivo desaparece, dropdown en la UI de settings.
- Conversión a **16 kHz mono f32→i16** (resampling con `rubato`), buffer en memoria (`Vec<i16>`, límite 120 s ≈ 3,8 MB).
- Codificación a WAV en memoria justo antes del envío (simple, universal; FLAC como optimización futura).
- El audio **nunca** toca disco (PRD §15.5), salvo flag `--debug-audio` explícito.
- **VAD (v1.x):** Silero vía crate `voice_activity_detector` sobre los mismos frames mono 16 kHz (chunks de 512 samples); solo corta el final de la grabación en modo toggle — spec en [ADR-0011](adr/0011-activacion-toggle-vad.md).

### 4.4 `speech/`

```rust
trait SpeechProvider: Send + Sync {
    fn id(&self) -> &'static str;                      // "openai"
    fn capabilities(&self) -> ProviderCapabilities;    // { batch: true, streaming: false, languages, formats }
    async fn transcribe(&self, audio: AudioData, opts: TranscribeOptions)
        -> Result<Transcript, SpeechError>;
}

struct TranscribeOptions { language: Option<LangTag>, model: String, timeout: Duration }
struct Transcript { text: String, language: Option<LangTag>, latency: Duration }

enum SpeechError {           // taxonomía estable, agnóstica de proveedor
    Auth,                    // key inválida/ausente  → error_key: err.stt.auth
    Network,                 // sin conexión/timeout → err.stt.network (retryable)
    RateLimited,             //                       → err.stt.rate  (retryable)
    InvalidAudio,            //                       → err.stt.audio
    Provider { code: String } // catch-all con código original en log
}
```

`ProviderRegistry`: mapa `id → factory`; resuelve según `config.stt.provider`. Registrar un proveedor nuevo = implementar el trait + una línea en el catálogo del registry (RNF-08). Cuando exista el segundo proveedor, `providers/*` migra a crates propios (PRD §17, ítem 2).

El segundo proveedor (v1.x) es **Groq** (`whisper-large-v3-turbo`, endpoint compatible OpenAI): valida el trait con costo de integración mínimo — spec en [ADR-0012](adr/0012-segundo-proveedor-stt-groq.md).

### 4.5 `providers/openai/`
- Endpoint `POST /v1/audio/transcriptions` (multipart WAV).
- **Modelo por defecto: `gpt-4o-mini-transcribe`** — mejor relación latencia/costo/calidad que `whisper-1`; `gpt-4o-transcribe` y `whisper-1` seleccionables en settings. *(Ratificado por el autor — PRD §17, ítem 3.)*
- Timeout: 30 s. Reintentos: 1 reintento automático solo ante `Network`/`RateLimited` con backoff 1 s (idempotente: mismo audio). Si falla, `TranscriptionFailed(retryable=true)` — en MVP el dictado se pierde tras el error (PRD §17.4).
- HTTP con `reqwest` (rustls, HTTPS only). La key se lee del keyring al momento de uso; nunca se mantiene en config ni cruza el IPC.

### 4.6 `delivery/`
Dos modos (RF-06): `insert` (texto en la app activa) y `clipboard` (solo copiar). El mecanismo de `insert` es la decisión **ADR-005 (Aceptada; validada por el spike R3 e implementada en M2)** — ver evaluación en [`adr/0005`](adr/0005-mecanismo-insercion-texto.md). Resumen de lo implementado:

- **Primario:** clipboard + orden de pegado sintética (Ctrl+V) con **guardado/restauración del clipboard** (solo texto en MVP). El delay antes de restaurar es de **300 ms** tras emitir el pegado (constante `RESTORE_DELAY` en `delivery/mod.rs`) — ventana en la que un gestor de clipboard externo puede capturar el texto (limitación documentada en ADR-005).
- **Perfiles de pegado por app (post-MVP, solo concepto):** combinación configurable (los terminales usan Ctrl+Shift+V); detección por clase de ventana en X11/Windows con tabla editable (en Wayland la identificación de la ventana activa es limitada: perfil por defecto + override manual). En MVP no existe la tabla: siempre Ctrl+V.
- **Secundario (fallback configurable):** inyección de tecleo (`enigo`) para apps que bloquean pegado sintético.
- **Descartado en MVP:** APIs de accesibilidad (UIA/AT-SPI) — máxima fidelidad pero costo alto y cobertura irregular; candidata a v2.x.
- **Garantizado siempre:** modo `clipboard` puro (funciona en todas las plataformas, incluido cualquier Wayland).

**Backends de síntesis de input por plataforma** (ADR-004): Windows (SendInput vía enigo) · X11 (XTest) · Wayland (portal `RemoteDesktop` con consentimiento, o `ydotool`/uinput si está disponible; si no hay ninguna vía, el modo `insert` se deshabilita con mensaje explícito y queda `clipboard`).

El **spike de validación** (R3) se ejecutó con resultado positivo en Windows 11 (2026-07-03) y el ADR pasó a Aceptada; la columna Linux de la matriz queda pendiente para cuando se retomen pruebas en Linux (ver `docs/spikes/`).

### 4.7 `config/` y `persistence/`
- Esquema versionado (`schema_version`) con migraciones; validación y defaults centralizados.
- Archivo `settings.json` en el dir de config del SO (`directories`/`tauri::path`).
- Secretos: `keyring-rs` → Credential Manager (Windows) / Secret Service (Linux). Si no hay Secret Service: la UI lo comunica y ofrece guardar cifrado-débil en archivo **solo con consentimiento explícito** y advertencia (R4).
- El frontend nunca ve la key completa (solo `is_set: bool` + últimos 4 caracteres).

Esquema conceptual:

```jsonc
{
  "schema_version": 1,
  "general": { "ui_language": "es", "hotkey": "Ctrl+Super+Space", "autostart": false,
               "start_minimized": true, "output_mode": "insert" },
  "stt":     { "provider": "openai", "model": "gpt-4o-mini-transcribe", "language": "auto" },
  "delivery": { "paste_combo_overrides": { "terminal": "Ctrl+Shift+V" }, "fallback_typing": false }
}
```

### 4.8 `ipc/`
Borde Tauri. Dos superficies:

- **Comandos** (frontend → core): `get_settings`, `set_settings`, `set_api_key`, `test_provider`, `get_app_state`. Validación en el borde; errores mapeados a `{ code, error_key }`.
- **Eventos** (core → frontend): reenvío 1:1 de los eventos de dominio marcados "UI ✔" en §2, con el mismo nombre y payload (el catálogo de eventos ES el contrato IPC — una sola fuente de verdad, tipos TS generados o espejados en `src/shared/`).

### 4.9 Frontend (`src/`)
- **Overlay:** ventana Tauri `alwaysOnTop, focusable:false, decorations:false, transparent, skipTaskbar`, posicionada junto al cursor o borde inferior. Render puro del último evento recibido. El flag `focusable:false` es objeto del **spike R2**.
- **Settings:** React + react-hook-form; solo comandos IPC.
- **i18n:** `react-i18next` con catálogos JSON en `src/i18n/{es,en}/`. Las claves de error del backend (`err.*`) viven en el mismo catálogo: el backend envía claves, el frontend traduce (RF-10). Lint: regla que prohíbe literales JSX fuera de i18n (R8).

### 4.10 Logging y errores
- `tracing` + `tracing-appender`: rotación diaria, retención 7 días, nivel por env/config. Campos estructurados (`event`, `state`, `latency_ms`).
- Redacción obligatoria: middleware de logging que filtra patrones de secretos; las keys jamás se interpolan (PRD §15.1).
- `thiserror` por módulo; conversión única `Error → { code, error_key, retryable }` en `ipc/`.

### 4.11 Updater (activo — ADR-010)
`tauri-plugin-updater` habilitado con `endpoints` apuntando al `latest.json` del último Release publicado en GitHub y `pubkey` de verificación. El módulo `src-tauri/src/updater/` concentra la lógica (ADR-002): comando `check_for_update` (chequeo manual, devuelve `Option<UpdateInfo>`) e `install_update` (descarga con progreso + `restart`). Al arrancar, un auto-chequeo emite el evento de dominio `UpdateAvailable`; la descarga emite `UpdateDownloadProgress` y los fallos `UpdateFailed`, todos por el canal `domain-event`. UX: avisar y que el usuario decida (nunca instala en silencio). En Linux el updater cubre AppImage; `.deb` se actualiza por gestor de paquetes (documentado en README).

---

## 5. Concurrencia

- Runtime `tokio` (ya requerido por Tauri). El core corre como task única propietaria del estado (mensajería `mpsc` hacia el core; los módulos emiten eventos por canal — evita locks sobre la máquina de estados).
- Captura de audio en hilo dedicado de cpal; el buffer cruza al core al cerrar la grabación.
- Llamada al proveedor en task cancelable (timeout de §4.5).

## 6. Seguridad (aplicación de PRD §15)

| Principio PRD | Mecanismo |
|---|---|
| Keys nunca en logs | Middleware de redacción + keys nunca en structs logueables |
| Secretos nunca en texto plano | keyring-rs; fallback solo con opt-in explícito |
| HTTPS siempre | reqwest + rustls; sin flag para desactivar TLS |
| Nada sale sin acción del usuario | Única llamada de red: `transcribe()` dentro del ciclo PTT |
| Control del usuario | Settings/logs/keys visibles y borrables desde la UI (v1.x: botón "borrar todo") |
| Indicador de grabación | `RecordingStarted` solo se emite con `OverlayOpened` previo (invariante del core) |
| Superficie mínima | CSP estricta Tauri, `assetProtocol` local, sin `remote` URLs, capabilities/permissions mínimos por ventana |

## 7. Testing

- **Unit (Rust):** máquina de estados (transiciones + timeouts, con reloj simulado), config/migraciones, mapeo de errores, provider OpenAI contra `wiremock`.
- **Contrato de eventos:** test de snapshot de serialización de cada evento (protege el contrato IPC/futuros suscriptores).
- **Frontend:** vitest + testing-library para settings; overlay por snapshot de estados.
- **Smoke manual por release:** checklist en `docs/DEVELOPMENT.md` (dictado en editor, terminal y navegador; ambos SO; error de key; sin red).
- **Spikes previos a M1:** (a) overlay sin foco en Windows, X11 y Wayland GNOME/KDE (R2); (b) matriz de inserción de texto en las mismas plataformas (R3/ADR-005); (c) hotkey vía portal GlobalShortcuts en GNOME/KDE (R1/ADR-004).

## 8. Empaquetado y CI

Según PRD §11.3. Detalle de artefactos por tag en `main`: NSIS `.exe` (Windows), AppImage + `.deb` (Linux), todos firmados con la clave del updater + `latest.json`. Matriz CI: `windows-latest`, `ubuntu-22.04` (GLIBC compatible hacia adelante).

## 9. Decisiones abiertas de esta fase

1. **ADR-005**: propuesta redactada (clipboard+paste con restauración + perfiles + fallback typing); pasa a *Aceptada* tras el spike de compatibilidad (Windows, X11, Wayland GNOME/KDE).
2. **ADR-004 (revisado)**: X11 y Wayland de primera clase; el spike debe fijar versiones mínimas de xdg-desktop-portal y la matriz de compositores soportados.
3. ~~Modelo OpenAI y reintentos~~ **Ratificado por el autor** (2026-07-02): `gpt-4o-mini-transcribe` por defecto; 1 reintento automático solo ante Network/RateLimited; el dictado se pierde tras error en MVP.
4. ~~Librería i18n~~ **Ratificado por el autor** (2026-07-02): `react-i18next`.
5. Nombre del producto: se mantiene el placeholder "VoiceText"; se definirá más adelante (no bloqueante).
