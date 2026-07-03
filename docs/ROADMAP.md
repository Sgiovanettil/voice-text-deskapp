# Roadmap — VoiceText

Fuente de verdad del alcance por hito: [PRD §12](PRD.md#12-roadmap). Este documento hace seguimiento del avance real; el PRD no se actualiza por cada commit.

## MVP (v0.x → v1.0)

| Hito | Contenido | Estado |
| --- | --- | --- |
| **M0 — Fundaciones** | Repo Git (Git Flow), scaffold Tauri, CI, calidad, i18n base, docs, ADRs 001–010. Spikes de riesgo (R1/R2/R3) preparados como binarios ejecutables. | ✅ **Completo** (10 PRs mergeados a `develop`) |
| M1 — Ciclo de dictado core | Hotkey PTT + eventos de dominio + captura de audio + provider OpenAI + texto al clipboard. Primera versión usable. | ⏳ Pendiente |
| M2 — Inserción y overlay | Inserción de texto en la app activa (según ADR-005 ya decidido); overlay con estados. | ⏳ Pendiente |
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

**Pendiente antes de M1** (bloqueante según PRD §14): ejecutar los 3 spikes en hardware real (Windows, Linux X11, Linux Wayland GNOME/KDE) y completar sus reportes en `docs/spikes/` — esto resuelve ADR-0005 (queda "Aceptada") y confirma/ajusta ADR-0004. No se pudo hacer en la sesión de M0 por falta de entorno gráfico.

## Post-MVP (orden tentativo)

1. **v1.x:** activación del auto-updater (Windows + AppImage), toggle + VAD, selección de micrófono, segundo proveedor STT (valida ADR-003), diccionario personal/reemplazos, inglés en la UI, ampliación de la matriz Wayland (compositores wlroots).
2. **v2.x:** streaming STT, Event Bus formal con suscriptores dinámicos, capacidad **LLM** (post-procesado del dictado: limpieza, formato, comandos de voz "en modo prompt").
3. **v3.x+:** TTS, Vision (capturas), Embeddings/RAG, Realtime, plugins; macOS.
