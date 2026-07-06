# Architecture Decision Records

Formato: contexto → decisión → consecuencias. Estados: Propuesta · Aceptada · Rechazada · Reemplazada.

| ADR | Título | Estado |
|---|---|---|
| [0001](0001-stack-tauri-react-rust.md) | Stack: Tauri v2 + React/TS + Rust | Aceptada |
| [0002](0002-logica-de-negocio-en-rust.md) | Toda la lógica de negocio en Rust | Aceptada |
| [0003](0003-abstraccion-speech-provider.md) | Abstracción SpeechProvider y organización por capacidades | Aceptada |
| [0004](0004-estrategia-linux-x11-wayland.md) | Linux: X11 y Wayland como targets de primera clase | Aceptada (revisada) |
| [0005](0005-mecanismo-insercion-texto.md) | Mecanismo de inserción de texto | Aceptada (spike R3 validado; implementada en M2) |
| [0006](0006-almacenamiento-secretos-y-config.md) | Secretos en keyring del SO; config versionada en archivo | Aceptada |
| [0007](0007-trunk-based-development.md) | Trunk-Based Development | Rechazada (se adoptó Git Flow, PRD §11) |
| [0008](0008-push-to-talk.md) | Push-to-Talk como modo de activación del MVP | Aceptada |
| [0009](0009-arquitectura-orientada-a-eventos.md) | Arquitectura orientada a eventos de dominio | Aceptada |
| [0010](0010-auto-update-preparado.md) | Auto-update Tauri preparado desde v1.0, activo post-MVP | Aceptada |
| [0011](0011-activacion-toggle-vad.md) | Activación toggle + VAD con Silero (v1.x) | Aceptada (diseño; implementación v1.x) |
| [0012](0012-segundo-proveedor-stt-groq.md) | Segundo proveedor STT: Groq (v1.x) | Implementada (v1.x, #58; validada en Windows) |
| [0013](0013-diccionario-personal.md) | Diccionario personal / reemplazos (v1.x) | Aceptada (diseño; implementación v1.x) |
| [0014](0014-modos-dictado-postprocesado-llm.md) | Modos de dictado y post-procesado LLM (v2.x) | Aceptada (diseño; implementación v2.x) |
