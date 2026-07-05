# ADR-0014 — Modos de dictado y post-procesado LLM (v2.x, spec anticipada)

- **Estado:** Aceptada (diseño; implementación diferida a v2.x) · **Fecha:** 2026-07-05

## Contexto

Necesidad del autor: que la app no solo pegue lo transcrito literal, sino que pueda interpretar
y mejorar la redacción. Es un problema de arquitectura, no de modelo: los STT
(`gpt-4o-mini-transcribe`, whisper) solo transcriben y no aceptan instrucciones de estilo. Se
requiere una **segunda pasada por un LLM**. El roadmap ya lo prevé como capacidad LLM de v2.x;
esta ADR deja el diseño listo sin adelantar su implementación.

## Decisión

### Tres modos de dictado

Nuevo setting `general.dictation_mode` (a futuro, hotkeys distintos por modo):

- **`literal`** (default, comportamiento actual): STT → inserción tal cual.
- **`mejorado`:** STT → LLM con **prompt fijo y versionado** de limpieza (muletillas,
  puntuación, redacción) → inserción. Regla dura: el LLM no puede cambiar el significado ni
  agregar contenido.
- **`prompt`:** la voz es una instrucción ("redacta un correo diciendo que…") → el LLM genera
  el texto → inserción.

### Pipeline

```
STT → diccionario personal (ADR-0013) → [LLM si el modo lo pide] → delivery
```

El diccionario corrige el reconocimiento **antes** del LLM (nombres propios bien escritos
mejoran el resultado). La etapa LLM es opcional y vive en el core como un efecto async más,
igual que la transcripción.

### Eventos de dominio

Los eventos existentes **no se renombran ni cambian de semántica** (contrato IPC). Se agregan
eventos nuevos para la etapa: `PostProcessingStarted`, `PostProcessingCompleted`,
`PostProcessingFailed(retryable)` — el overlay muestra un estado "puliendo…" entre transcribir
e insertar. Ante fallo del LLM, **degradación segura: se entrega el texto literal** (nunca se
pierde el dictado) con aviso en el overlay.

### Proveedor LLM

- Default: **la credencial OpenAI existente** (misma entrada de keyring), con un modelo
  económico de texto (clase `gpt-4o-mini`), configurable en settings.
- Abstracción por trait (`TextProcessor`, análogo a `SpeechProvider` de ADR-0003) para permitir
  otros proveedores después sin tocar el core.
- Los prompts del modo `mejorado` son fijos, versionados en el repo y parametrizados solo por
  idioma del dictado; en `prompt` la instrucción es la voz del usuario.

## Justificación

- Separar STT de post-procesado mantiene cada etapa simple, testeable y con proveedores
  intercambiables; un "modelo que haga todo" no existe en los endpoints STT actuales.
- La degradación a literal preserva el principio de nunca perder un dictado.
- Reusar credencial y patrones de ADR-0003 minimiza el costo de entrada cuando llegue v2.x.

## Consecuencias

- (+) El diseño queda cerrado: implementar v2.x no requiere decisiones nuevas de arquitectura.
- (+) Los modos son aditivos: `literal` intacto como default, sin riesgo de regresión.
- (−) Latencia extra perceptible en `mejorado`/`prompt` (una llamada LLM adicional) — el
  overlay debe comunicarla; streaming del LLM es optimización futura.
- (−) Costo por dictado adicional en modos LLM (documentar en settings).
- (−) En `prompt`, la frontera "instrucción vs. dictado" depende del usuario: sin heurísticas
  mágicas en v2.x, el modo se elige explícitamente.
