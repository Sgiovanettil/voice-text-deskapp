# Definiciones — glosario de términos de dominio

Términos recurrentes del dominio y su significado en este proyecto. Fuente de la arquitectura:
[ARCHITECTURE.md](../2-arquitectura/ARCHITECTURE.md); decisiones:
[DECISIONS/](../2-arquitectura/DECISIONS/).

| Término | Definición | Fuente |
| --- | --- | --- |
| **STT** (Speech-to-Text) | Transcripción de voz a texto; el propósito central de la app. | [PRD.md](PRD.md) |
| **DomainEvent** | Catálogo tipado de eventos de dominio que modela el ciclo de dictado; **es el contrato IPC** entre backend Rust y frontend, append-only (se agregan eventos/campos, nunca se renombran ni cambian de semántica). | [ARCHITECTURE.md](../2-arquitectura/ARCHITECTURE.md) §2 · [ADR-0009](../2-arquitectura/DECISIONS/0009-arquitectura-orientada-a-eventos.md) |
| **Core** | Máquina de estados pura del ciclo de dictado (Idle → escuchando → transcribiendo → insertando → Idle/Error); no conoce proveedores ni sistema operativo. | [ARCHITECTURE.md](../2-arquitectura/ARCHITECTURE.md) §3 |
| **Orquestador** | Hilo dueño de la máquina de estados y del grabador; ejecuta los efectos async y espeja el estado al frontend como eventos de dominio. | [ARCHITECTURE.md](../2-arquitectura/ARCHITECTURE.md) §4 |
| **SpeechProvider** | Trait de abstracción de proveedor de transcripción; el core solo conoce el trait, nunca una implementación concreta. | [ADR-0003](../2-arquitectura/DECISIONS/0003-abstraccion-speech-provider.md) |
| **ProviderRegistry** | Registro que resuelve el `SpeechProvider` activo según la configuración. | [ADR-0003](../2-arquitectura/DECISIONS/0003-abstraccion-speech-provider.md) |
| **Provider (proveedor STT)** | Implementación concreta de `SpeechProvider`. Por defecto OpenAI (`gpt-4o-mini-transcribe`); Groq disponible como segundo proveedor (API compatible con OpenAI). | [ADR-0012](../2-arquitectura/DECISIONS/0012-segundo-proveedor-stt-groq.md) |
| **PTT** (Push-to-Talk) | Modo de activación: se graba mientras el hotkey está presionado y se transcribe al soltarlo. | [ADR-0008](../2-arquitectura/DECISIONS/0008-push-to-talk.md) |
| **Toggle** | Modo de activación alternativo: una pulsación inicia el dictado y otra (o el corte por silencio) lo termina. | [ADR-0011](../2-arquitectura/DECISIONS/0011-activacion-toggle-vad.md) |
| **VAD** (Voice Activity Detection) | Detección de actividad de voz (motor Silero) que corta el dictado por silencio en modo toggle. | [ADR-0011](../2-arquitectura/DECISIONS/0011-activacion-toggle-vad.md) |
| **Overlay** | Ventana flotante siempre-encima, sin foco y sin decoraciones, que muestra en vivo el estado del ciclo (escuchando → transcribiendo → insertando → listo/error). | [ARCHITECTURE.md](../2-arquitectura/ARCHITECTURE.md) §4 · [spike R2](../2-arquitectura/spikes/r2-overlay-sin-foco.md) |
| **Delivery / inserción** | Entrega del texto transcrito: modo `insert` (pegado sintético en la app activa) o `clipboard` (solo copiar). | [ADR-0005](../2-arquitectura/DECISIONS/0005-mecanismo-insercion-texto.md) |
| **Hotkey global** | Atajo de teclado que activa el dictado desde cualquier parte del sistema; backend por plataforma (X11 directo, Wayland vía portal `GlobalShortcuts`). | [ADR-0004](../2-arquitectura/DECISIONS/0004-estrategia-linux-x11-wayland.md) |
| **Keyring** | Almacén de secretos nativo del sistema operativo donde se guardan las API keys de los proveedores; la config no secreta se versiona aparte en archivo. | [ADR-0006](../2-arquitectura/DECISIONS/0006-almacenamiento-secretos-y-config.md) |
| **`xdg-desktop-portal`** | Servicio D-Bus de portals de Linux; provee `GlobalShortcuts` (hotkey) y `RemoteDesktop` (síntesis de input) en sesiones Wayland. | [ADR-0004](../2-arquitectura/DECISIONS/0004-estrategia-linux-x11-wayland.md) |
| **Spike** | Binario ejecutable de validación de riesgo técnico (R1 hotkey portal, R2 overlay, R3 inserción) previo a comprometer el diseño en un ADR. | [spikes/](../2-arquitectura/spikes/) |
