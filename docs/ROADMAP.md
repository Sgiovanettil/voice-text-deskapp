# Roadmap — VoiceText

Fuente de verdad del alcance por hito: [PRD §12](PRD.md#12-roadmap). Este documento hace seguimiento del avance real; el PRD no se actualiza por cada commit.

## MVP (v0.x → v1.0)

| Hito | Contenido | Estado |
| --- | --- | --- |
| **M0 — Fundaciones** | Repo Git (Git Flow), scaffold Tauri, CI, calidad, i18n base, docs, ADRs 001–010. Spikes de riesgo (R1/R2/R3) preparados como binarios ejecutables. | ✅ **Completo** (10 PRs mergeados a `develop`) |
| **M1 — Ciclo de dictado core** | Hotkey PTT + eventos de dominio + captura de audio + provider OpenAI + texto al clipboard. Primera versión usable. | ✅ **Completo y validado** end-to-end en Windows 11 (2026-07-03, instalador de desarrollo) |
| **M2 — Inserción y overlay** | Inserción de texto en la app activa (según ADR-005 ya decidido); overlay con estados. | ✅ **Completo y validado** end-to-end en Windows 11 (2026-07-03) — overlay flotante se muestra sin robar foco e inserción automática (modo insert) llega donde está el cursor |
| **M3 — Settings y residencia** | Tray, settings UI completa (con i18n), keyring, autostart, persistencia. | ✅ **Completo y validado** end-to-end en Windows 11 (2026-07-03) — bandeja con cerrar-a-bandeja, autostart sincronizado con el setting y UI de settings completa; keyring y persistencia venían de M1 |
| **M4 — Endurecimiento y release** | Manejo de errores pulido, logging, empaquetado firmado compatible con updater y release automatizado → **v1.0**. | ✅ **Completo** (2026-07-03) — panic hook + logging de fallos, firma de updater compatible, y release automatizado por tag: **v1.0.0** construido con bundles firmados (Windows NSIS/MSI, Linux AppImage/deb/rpm) + `latest.json`, publicado como Release en borrador |

### M0 — detalle de lo entregado

1. `docs/` versionado (PRD, ARCHITECTURE, 10 ADRs, brief original).
2. Bootstrap del repo: `.gitignore`, `README.md`, `AGENTS.md`/`CLAUDE.md`, `.editorconfig`; graphify instalado con hook post-commit.
3. Scaffold Tauri v2 + React/TS reorganizado a la estructura de ARCHITECTURE §8.4 (`src/settings/`, `src/overlay/`, `src/shared/`, `src/i18n/`).
4. Esqueleto compilable de los módulos Rust por capacidad (`core/`, `audio/`, `speech/`, `providers/openai/`, `delivery/`, `config/`, `persistence/`, `hotkeys/`, `ipc/`) — sin lógica de negocio.
5. Catálogo de eventos de dominio v1 (`DomainEvent`) con tests de snapshot, espejado en `src/shared/events.ts`.
6. i18n base con `react-i18next`, catálogos `es`/`en`.
7. Tooling de calidad: ESLint (+ regla anti-literales JSX), Prettier, commitlint, husky, rustfmt.
8. CI (`ci.yml`: lint/test/build en matriz Windows+Linux) y CD (`release.yml`: changelog + bundles firmados, inerte hasta el primer tag).
9. Los 3 spikes de riesgo (`spike-overlay`, `spike-delivery`, `spike-hotkey-portal`) como binarios compilables, con plantillas de reporte en `docs/spikes/`.
10. Esta guía de roadmap + `docs/DEVELOPMENT.md`.

**Spikes** (bloqueante del PRD §14, parcialmente resuelto): R2 y R3 validados en Windows 11 (2026-07-03) con resultado positivo — ver `docs/spikes/`. Pendientes: columna Linux de R2/R3 y R1 completo (solo Wayland), cuando se retomen pruebas en Linux. Con la evidencia Windows se decidió avanzar M1 enfocado solo en Windows.

### M1 — detalle de lo entregado (2026-07-03)

1. Máquina de estados del ciclo (transiciones §3 + reglas: reentrada ignorada, descarte <300 ms, corte a 120 s, Error→Idle ~3 s), pura y con reloj simulado.
2. Captura de audio: cpal (dispositivo por defecto, F32/I16/U16) → mono 16 kHz (rubato FFT) → WAV en memoria (hound). Nunca toca disco.
3. Provider OpenAI real (`/v1/audio/transcriptions`, multipart, 1 reintento ante Network/RateLimited) + `check_auth` vía `GET /models` — testeado contra wiremock.
4. Settings persistidos (defaults §4.7, carga con degradación segura, save atómico) y API key en keyring del SO (UI solo ve últimos 4).
5. Hotkey PTT global (plugin global-shortcut, Windows/X11) con re-registro en caliente al cambiarlo.
6. Entrega al clipboard (arboard).
7. Settings UI mínima (API key + test de conexión + hotkey, i18n es/en) y borde IPC completo (`{ code, errorKey }`).
8. Orquestador: hilo dueño de máquina + recorder, efectos async como eventos, espejo de estado, eventos `domain-event` al frontend 1:1. Job CI `installer` (workflow_dispatch): instalador NSIS de desarrollo sin firma.
9. Observabilidad (derivada de la primera prueba real): estado del ciclo en vivo y última transcripción en la ventana de settings, logs a archivo con rotación diaria (`app_log_dir`, nivel por `RUST_LOG`), y fix del espejo TS de eventos (variantes camelCase, la forma real del wire).

**Validación (2026-07-03)**: ciclo completo verificado en Windows 11 con el instalador de desarrollo — API key en keyring, test de conexión, dictado por PTT (`Ctrl+Shift+Z`) y pegado desde clipboard. Aprendizaje de UX: la primera prueba falló porque el usuario esperaba modo toggle (apretar una vez) en vez de mantener presionado — refuerza la prioridad del modo toggle+VAD ya planificado para v1.x y motivó los textos de estado explícitos ("mantén presionado… suéltalo").

### M2 — detalle de lo entregado (2026-07-03)

1. **Overlay flotante** (spike R2 → feature): ventana Tauri aparte (`overlay.html`), siempre encima, sin foco, sin decoraciones, transparente y fuera de la barra de tareas. Muestra en vivo escuchando → transcribiendo → insertando → listo/error, ligada a las transiciones del core (`OverlayOpened`/`OverlayClosed`), posicionada abajo-centro, reutilizada entre ciclos con dwell de 900 ms.
2. **Inserción automática** (modo `insert`, ADR-0005): clipboard + pegado sintético (Ctrl+V vía `enigo`, patrón del spike R3) + restauración del clipboard anterior; degradación a pegado manual si falla. Selector "qué hacer con el texto" (insertar / solo copiar) en settings; default insert → el texto aparece solo donde está el cursor.

**Validación (2026-07-03)**: verificado en Windows 11 — el overlay flotante aparece con los estados del ciclo sin robar el foco de la app activa, y la inserción automática (modo insert) hace que el texto transcrito aparezca solo donde está el cursor. La selección de combo por app activa (terminales con Ctrl+Shift+V) queda para los perfiles de pegado (post-MVP).

### M3 — detalle de lo entregado (2026-07-03)

Keyring (API key en el almacén del SO) y persistencia versionada de settings ya se
entregaron en M1; M3 aporta la residencia y completa la configuración:

1. **Bandeja del sistema** (`tray/`): icono con menú (Configuración / Salir) localizado
   según `general.ui_language`, click izquierdo abre la configuración. **Cerrar-a-bandeja**:
   cerrar la ventana de settings la oculta en vez de terminar la app (se sale solo desde el
   menú del tray). La ventana arranca oculta (`visible: false`) y se muestra al iniciar salvo
   `general.start_minimized`.
2. **Autostart** (`autostart/`): registra `tauri-plugin-autostart` y reconcilia el estado real
   del SO (registro/LaunchAgent/`.desktop`) contra `general.autostart` en cada arranque y cada
   guardado — el setting es la fuente de verdad; best-effort, no bloquea el arranque.
3. **UI de settings completa**: selector de idioma de la app (aplica i18n en vivo y arranca en
   el idioma persistido), modelo e idioma de transcripción, y toggles de arranque automático y
   de iniciar minimizado. Todo pasa por i18n (es/en).

**Validación (2026-07-03)**: verificado en Windows 11 conforme a lo definido — residencia en
bandeja (cerrar-a-bandeja, click en el icono, menú), arranque automático que se registra/elimina
al togglear, iniciar minimizado y cambio de idioma en vivo. Limitación conocida (aceptada): el
menú nativo del tray cambia de idioma recién al reiniciar la app.

### M4 — detalle de lo entregado (2026-07-03)

1. **Observabilidad / endurecimiento**: panic hook global que enruta cualquier panic (incluidos
   los de hilos de trabajo: orquestador, efectos async, dwell del overlay) al archivo de log con
   ubicación y nombre de hilo; `DomainEvent::is_failure()` sube los eventos `*Failed` a `warn`
   para que un ciclo fallido resalte en el log. El flujo ya degradaba bien (cada efecto convierte
   errores en eventos de dominio), así que el foco fue la red de seguridad, no re-plumbing.
2. **Firma de updater compatible** (RNF-09, ADR-010): `plugins.updater.pubkey` (minisign) en la
   conf para que bundles y `latest.json` sean verificables por el updater oficial desde la primera
   release. En ese hito el updater seguía inactivo en runtime (sin endpoints, sin plugin); su
   activación quedó para v1.x — **ya implementada**: `tauri-plugin-updater` habilitado, endpoints
   al `latest.json` del Release y UI de aviso/instalación (ver `docs/RELEASING.md` y ADR-010).
3. **Release automatizado → v1.0.0**: bump de versión por `release/1.0.0` → `main` (tag `v1.0.0`)
   que dispara `release.yml`: changelog (git-cliff) + bundles firmados (Windows NSIS/MSI, Linux
   AppImage/deb/rpm) + `latest.json`, publicados como **Release en borrador** para revisión y
   publicación manual. Fix de CI en el camino: el job de release necesitaba `permissions:
   contents: write` (el `GITHUB_TOKEN` es de solo-lectura por defecto).

**Firma Authenticode de Windows**: diferida a post-v1.0 por decisión del autor — v1.0 usa solo la
firma de updater (gratis); el instalador NSIS/MSI queda sin Authenticode, así que Windows muestra
"editor desconocido" en SmartScreen. Agregarla después no requiere cambios estructurales.

**MVP completo (v1.0)**: M0–M4 entregados; M1–M3 validados end-to-end en Windows 11. La v1.0.0
quedó construida y firmada, a la espera de publicar el borrador.

## Post-MVP (orden tentativo)

### v1.x — en curso

| Item | Estado |
| --- | --- |
| Activación del auto-updater (Windows + AppImage) | ✅ **Entregado** en v1.1.0 (aviso + confirmación del usuario) |
| Toggle + VAD ([ADR-0011](adr/0011-activacion-toggle-vad.md)) | ✅ **Entregado y validado** end-to-end en Windows 11 (2026-07-05) — modo toggle con corte por silencio (Silero VAD) mergeado a `develop` (#56) |
| Selección de micrófono (spec en ARCHITECTURE §4.3) | ⏳ Pendiente |
| Segundo proveedor STT: Groq ([ADR-0012](adr/0012-segundo-proveedor-stt-groq.md), valida ADR-003) | ✅ **Entregado y validado** end-to-end en Windows 11 (2026-07-05) — Groq (`whisper-large-v3-turbo`) mergeado a `develop` (#58); cliente OpenAI-compatible factorizado, key por proveedor, selector + test por proveedor y refresco del widget |
| Diccionario personal/reemplazos ([ADR-0013](adr/0013-diccionario-personal.md)) | ⏳ Pendiente |
| Inglés en la UI | ⏳ Pendiente |
| Ampliación de la matriz Wayland (compositores wlroots) | ⏳ Pendiente |

#### Toggle + VAD — detalle de lo entregado (2026-07-05)

1. Nuevo setting `general.activation_mode` (`ptt` default / `toggle`), mismo hotkey para ambos modos. En toggle: una pulsación inicia; corta la segunda pulsación, el silencio sostenido del VAD o el tope de 120 s.
2. Motor VAD: crate `voice_activity_detector` (Silero sobre ONNX Runtime), chunks de 512 samples @ 16 kHz, con `VadGate` en el hilo de captura (resampler lineal propio para el VAD; el audio transcrito sigue por rubato). El corte solo se arma tras detectar habla — una pausa inicial no cierra el dictado. Sección `vad` en settings: `threshold` (0.5) y `silence_hangover_ms` (1200).
3. Sin cambios al contrato de eventos salvo el evento aditivo `SilenceDetected` (ADR-0009: solo agregar); telemetría `vad-speaking` para que el overlay muestre "En silencio… cerrando". UI: selector de modo en Atajos, textos según el modo, i18n es/en. Degradación elegante: si ONNX no inicializa, se graba sin corte automático.

#### Segundo proveedor STT: Groq — detalle de lo entregado (2026-07-05)

1. Cliente HTTP factorizado en `OpenAiCompatibleProvider` (multipart, timeout, reintento y mapeo de errores compartidos); OpenAI y Groq solo aportan id, base URL y modelo por defecto. `providers::resolve(id, key)` elige el proveedor por `stt.provider` — el orquestador ya no construye OpenAI a mano (antes puenteaba el registry).
2. API key por proveedor en el keyring (`openai_api_key`/`groq_api_key`); OpenAI conserva su usuario para no perder la key ya guardada. `check_auth` pasó a ser método del trait `SpeechProvider` (aditivo) para que `test_provider` resuelva por id.
3. UI: selector de proveedor con key, estado, test y modelos por proveedor; pistas de onboarding; el overlay refresca proveedor/modelo ante `ConfigChanged`. i18n es/en. Validado con key real de Groq en Windows 11.
4. Diferido (anotado en ADR-0012): migración de `providers/*` a crates propios, para acotar el PR; ortogonal a validar el trait.

### v2.x

Streaming STT, Event Bus formal con suscriptores dinámicos, capacidad **LLM** (post-procesado del dictado: limpieza, formato, comandos de voz "en modo prompt" — spec anticipada en [ADR-0014](adr/0014-modos-dictado-postprocesado-llm.md)).

### v3.x+

TTS, Vision (capturas), Embeddings/RAG, Realtime, plugins; macOS.
