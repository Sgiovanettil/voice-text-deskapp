# ADR-0011 — Activación toggle + VAD con Silero (v1.x)

- **Estado:** Implementada (v1.x; mergeada a `develop` en #56, validada en Windows 11 el 2026-07-05) · **Fecha:** 2026-07-05

## Contexto

ADR-0008 fijó Push-to-Talk como único modo de activación del MVP y dejó toggle y VAD para
v1.x. La validación de M1 lo confirmó como necesidad real: la primera prueba de usuario falló
porque esperaba modo toggle (presionar una vez) en vez de mantener presionado (ROADMAP, M1).
Esta ADR especifica el modo toggle con corte por detección de voz (VAD) para que su
implementación no requiera decisiones nuevas.

## Decisión

### Modos de activación

Nuevo setting `general.activation_mode`: `"ptt"` (default, comportamiento actual) o
`"toggle"`. Ambos usan **el mismo hotkey** configurado:

- **`ptt`:** sin cambios — graba mientras el hotkey está presionado.
- **`toggle`:** una pulsación inicia la grabación; termina por (a) nueva pulsación del hotkey,
  (b) corte automático del VAD tras silencio sostenido, o (c) el tope global de 120 s — lo que
  ocurra primero.

### Motor VAD: Silero

- Crate **`voice_activity_detector`** (Silero VAD sobre ONNX Runtime, modelo ~2 MB embebido en
  el binario). Se eligió sobre umbral de energía RMS (frágil ante ruido de fondo: ventiladores,
  tecleo) y sobre `webrtc-vad` (binding C poco mantenido, calidad inferior).
- Opera sobre el pipeline existente sin conversiones extra: frames mono 16 kHz i16 en chunks de
  **512 samples (32 ms)**, el tamaño nativo del modelo para 16 kHz.
- Parámetros (en `settings.json`, sección nueva `vad`):
  - `threshold` — probabilidad de voz mínima para considerar un chunk como habla. Default
    **0.5**.
  - `silence_hangover_ms` — silencio continuo que dispara el corte. Default **1200 ms**
    (suficiente para pausas de dictado sin cortar frases).
- El VAD **solo corta el final** en modo toggle; nunca recorta el inicio ni descarta audio (el
  buffer completo desde la pulsación se transcribe, principio de no perder dictados).

### Integración con el core

Sin cambios a la máquina de estados ni al catálogo de eventos: toggle y VAD son **nuevos
disparadores de los mismos eventos** `RecordingStarted`/`RecordingStopped`, tal como anticipó
ADR-0008. El descarte de grabaciones < 300 ms y el tope de 120 s se mantienen idénticos.

### UX (overlay)

En modo toggle el overlay indica que la escucha está activa y se cerrará sola ("hablando… / en
silencio, cerrando…"), con textos i18n nuevos. El corte por VAD emite el mismo flujo visual que
soltar el hotkey en PTT.

## Justificación

- Resuelve el aprendizaje de UX de M1 (usuarios esperan toggle) sin sacrificar PTT.
- Silero es el estándar de facto en VAD local: robusto ante ruido, liviano, sin red.
- Reusar hotkey + eventos existentes mantiene el core intacto (ADR-0008) y la activación
  externa de Wayland sin portal (ARCHITECTURE §4.2) ya opera como toggle implícito — este modo
  la formaliza.

## Consecuencias

- (+) Dictados largos sin mantener tecla; manos libres tras iniciar.
- (+) Cero cambios al contrato de eventos (IPC estable).
- (−) Nueva dependencia nativa (ONNX Runtime vía `voice_activity_detector`): crece el binario y
  la superficie de build multiplataforma — validar en CI Windows+Linux antes de mergear.
- (−) El corte automático puede sorprender: el hangover debe ser configurable y el overlay debe
  comunicar la cuenta regresiva de silencio.
