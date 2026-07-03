# ADR-0008 — Push-to-Talk como modo de activación del MVP

- **Estado:** Aceptada · **Fecha:** 2026-07-02

## Contexto
Alternativas de activación del dictado: push-to-talk (mantener el hotkey), toggle (presionar para iniciar/terminar) y toggle+VAD (corte por detección de silencio).

## Decisión
El MVP implementa **exclusivamente Push-to-Talk**: la grabación dura exactamente lo que el hotkey permanece presionado. Flujo oficial: Hotkey → Overlay → Grabación → Transcripción → Entrega del texto → Cerrar Overlay.

## Justificación
- Control total del usuario sobre inicio/fin: sin cortes prematuros ni grabaciones fantasma (principios 1 y 3).
- Elimina la necesidad de VAD (complejidad de DSP y calibración) en MVP.
- El estado `Recording` de la máquina de estados queda delimitado por eventos de hotkey — el diseño admite agregar toggle/VAD después como nuevos disparadores de los mismos eventos `RecordingStarted/Stopped`, sin cambiar el core.

## Consecuencias
- (+) UX predecible; implementación mínima; privacidad clara (se graba solo mientras se presiona).
- (−) Dictados largos exigen mantener la tecla; incómodo para manos ocupadas → toggle y VAD en v1.x.
- (−) Algunos teclados/SO tienen límites con auto-repeat de teclas modificadoras: el spike de hotkeys debe validar press/release confiables.
