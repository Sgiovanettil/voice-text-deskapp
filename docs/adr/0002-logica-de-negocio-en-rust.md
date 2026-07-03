# ADR-0002 — Toda la lógica de negocio en Rust

- **Estado:** Aceptada · **Fecha:** 2026-07-02

## Contexto
Con dos WebViews (overlay, settings) existe la tentación de poner lógica en el frontend. Futuras superficies (CLI, API local, plugins) no tendrán WebView.

## Decisión
El frontend es exclusivamente presentación y formularios. Máquina de estados, orquestación, providers, audio, delivery, config y secretos viven en Rust. El frontend interactúa solo vía comandos y eventos IPC.

## Consecuencias
- (+) Nuevas superficies reutilizan el core sin duplicar lógica.
- (+) Los secretos jamás cruzan al WebView (la key nunca sale del backend).
- (−) Algunas interacciones triviales requieren round-trip IPC; aceptable dada la baja frecuencia.
- Regla de revisión de PR: rechazar lógica de negocio en `src/`.
