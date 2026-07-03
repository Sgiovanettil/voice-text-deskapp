# ADR-0009 — Arquitectura orientada a eventos de dominio

- **Estado:** Aceptada · **Fecha:** 2026-07-02

## Contexto
Principio 7 del PRD: cada nueva capacidad (LLM, TTS, Vision, Embeddings, plugins, automatizaciones, agentes, telemetría opt-in) debe incorporarse sin modificar el núcleo. Las llamadas directas entre módulos generan acoplamiento que lo impide.

## Decisión
- La comunicación entre componentes de negocio se realiza mediante **eventos de dominio tipados y serializables** (catálogo en ARCHITECTURE §2: `HotkeyPressed`, `OverlayOpened`, `RecordingStarted/Stopped`, `TranscriptionStarted/Completed/Failed`, `TextDeliveryStarted/Completed/Failed`, `OverlayClosed`, …).
- Los eventos son **contrato estable**: nombres en pasado, payloads planos y agnósticos de proveedor; se permite añadir campos/eventos, nunca renombrar ni cambiar semántica.
- En MVP **no** se implementa un Event Bus formal: un despachador simple dentro del core es suficiente; el puente IPC reenvía al frontend los eventos de UI. El diseño se hace *como si* el bus existiera.
- Post-MVP: el despachador se sustituye por un Event Bus con suscripción dinámica sin cambiar el catálogo; las nuevas capacidades se integran como suscriptores (p. ej. post-procesador LLM escuchando `TranscriptionCompleted`).

## Consecuencias
- (+) Extensibilidad sin tocar el core; el catálogo de eventos sirve simultáneamente de contrato IPC (una sola fuente de verdad).
- (+) Testeabilidad: la máquina de estados se prueba como función eventos→eventos.
- (−) Indirección: seguir un flujo requiere conocer el catálogo (mitigado: documentación §2 y logging estructurado por evento).
- (−) Disciplina de evolución del contrato (tests de snapshot de serialización lo protegen).
