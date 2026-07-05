# ADR-0012 — Segundo proveedor STT: Groq (v1.x)

- **Estado:** Aceptada (diseño; implementación en v1.x) · **Fecha:** 2026-07-05

## Contexto

ADR-0003 definió el trait `SpeechProvider` con la promesa de que agregar un proveedor fuese
"implementar el trait + una línea en el registry" (RNF-08). El roadmap v1.x exige un segundo
proveedor real que valide esa abstracción. Candidatos evaluados: Groq, Whisper local
(whisper.cpp), Gemini y Deepgram.

## Decisión

El segundo proveedor es **Groq** con el modelo **`whisper-large-v3-turbo`**.

- **Endpoint:** `POST https://api.groq.com/openai/v1/audio/transcriptions` — API **compatible
  con OpenAI** (mismo multipart WAV, mismos campos `file`/`model`/`language`). El cliente de
  `providers/openai/` se factoriza para parametrizar base URL y modelo; la lógica de multipart,
  timeout (30 s), reintento (1 ante `Network`/`RateLimited`) y mapeo a `SpeechError` se
  comparte.
- **Credencial:** API key propia de Groq en el keyring del SO como entrada separada
  (`voicetext/groq`), mismas reglas que OpenAI: la UI solo ve `is_set` + últimos 4; nunca cruza
  el IPC.
- **Settings:** `stt.provider` acepta `"openai" | "groq"`; selector en la sección de
  transcripción con test de conexión por proveedor (Groq también expone `GET /openai/v1/models`
  para `check_auth`).
- **Registry:** alta en `ProviderRegistry`; al existir el segundo proveedor se ejecuta la
  migración de `providers/*` a crates propios ya prevista (PRD §17, ítem 2).

## Justificación (comparativa)

| Criterio | Groq | Whisper local | Gemini | Deepgram |
|---|---|---|---|---|
| Esfuerzo de integración | Mínimo (API = OpenAI) | Alto (binarios, modelos, gestión de descarga) | Medio (API distinta, no STT dedicado) | Medio (API distinta) |
| Latencia | Excelente (hardware LPU) | Depende del equipo del usuario | Media | Buena |
| Costo | Muy bajo | Cero marginal | Bajo | Medio |
| Privacidad | Nube (igual que OpenAI) | Total/offline | Nube | Nube |
| Valida el trait | Sí, con costo mínimo | Sí, pero mezcla el objetivo con empaquetado pesado | Sí | Sí |

Groq maximiza la relación validación/esfuerzo. Whisper local queda como candidato natural para
un **tercer** proveedor (privacidad/offline) una vez que el registry esté probado con dos
proveedores de nube.

## Consecuencias

- (+) Valida ADR-0003 y RNF-08 con riesgo mínimo; los usuarios ganan un proveedor más rápido y
  barato.
- (+) La factorización del cliente OpenAI-compatible deja lista la puerta para cualquier otro
  endpoint compatible.
- (−) Requiere cuenta y API key de Groq (fricción de onboarding: documentar en settings).
- (−) Sigue siendo nube: no avanza el eje privacidad/offline (explícitamente diferido).
