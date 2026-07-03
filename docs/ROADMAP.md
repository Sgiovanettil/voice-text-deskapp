# Roadmap — VoiceText

Fuente de verdad del alcance por hito: [PRD §12](PRD.md#12-roadmap). Este documento hace seguimiento del avance real; el PRD no se actualiza por cada commit.

## MVP (v0.x → v1.0)

| Hito | Contenido | Estado |
| --- | --- | --- |
| **M0 — Fundaciones** | Repo Git (Git Flow), scaffold Tauri, CI, calidad, i18n base, docs, ADRs 001–010. Spikes de riesgo (R1/R2/R3) preparados como binarios ejecutables. | ✅ **Completo** (10 PRs mergeados a `develop`) |
| **M1 — Ciclo de dictado core** | Hotkey PTT + eventos de dominio + captura de audio + provider OpenAI + texto al clipboard. Primera versión usable. | ✅ **Completo y validado** end-to-end en Windows 11 (2026-07-03, instalador de desarrollo) |
| M2 — Inserción y overlay | Inserción de texto en la app activa (según ADR-005 ya decidido); overlay con estados. | 🔄 **Código completo** (2 PRs) — overlay flotante con estados + inserción automática (modo insert); pendiente validación en Windows |
| M3 — Settings y residencia | Tray, settings UI completa (con i18n), keyring, autostart, persistencia. | ⏳ Pendiente |
| M4 — Endurecimiento y release | Manejo de errores pulido, logging, empaquetado firmado compatible con updater y release automatizado → **v1.0**. | ⏳ Pendiente |

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

**Pendiente de M2**: validar en Windows el overlay (no roba foco, sigue estados) y la inserción en VS Code/navegador/Notepad, más el selector de modo. La selección de combo por app activa (terminales con Ctrl+Shift+V) queda para los perfiles de pegado.

## Post-MVP (orden tentativo)

1. **v1.x:** activación del auto-updater (Windows + AppImage), toggle + VAD, selección de micrófono, segundo proveedor STT (valida ADR-003), diccionario personal/reemplazos, inglés en la UI, ampliación de la matriz Wayland (compositores wlroots).
2. **v2.x:** streaming STT, Event Bus formal con suscriptores dinámicos, capacidad **LLM** (post-procesado del dictado: limpieza, formato, comandos de voz "en modo prompt").
3. **v3.x+:** TTS, Vision (capturas), Embeddings/RAG, Realtime, plugins; macOS.
