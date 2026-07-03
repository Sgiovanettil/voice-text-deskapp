# ADR-0003 — Abstracción `SpeechProvider` y organización por capacidades

- **Estado:** Aceptada (confirmada por el autor) · **Fecha:** 2026-07-02

## Contexto
Principio 5 del PRD: ningún proveedor es indispensable. El MVP usa OpenAI por simplicidad y velocidad de implementación, pero el resto de la aplicación no debe conocer detalles del proveedor concreto.

## Decisión
- Trait `SpeechProvider` (id, `capabilities()`, `transcribe()`) con tipos agnósticos (`AudioData`, `Transcript`, `SpeechError` con taxonomía estable).
- `ProviderRegistry` resuelve la implementación según configuración.
- Código organizado por **capacidades** (speech, futuro llm/tts/vision/embeddings/realtime), no por proveedores; cada proveedor en su propio módulo/crate.
- `capabilities()` declara batch/streaming/idiomas/formatos para incorporar streaming después **sin romper el contrato**.

## Consecuencias
- (+) Agregar proveedor = implementar trait + registrar (RNF-08); cero cambios en core/UI.
- (+) El mismo patrón se replica para LlmProvider, TtsProvider, etc.
- (−) El trait debe diseñarse mirando 2–3 proveedores aunque solo exista uno: pequeña sobre-ingeniería consciente (principio 8).
- El segundo proveedor real actuará como test de validación del ADR.
