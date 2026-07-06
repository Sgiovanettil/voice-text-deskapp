# ADR-0015 — Pantalla de gastos por API key con estimación local (v1.x)

- **Estado:** Aceptada (diseño; implementación en v1.x) · **Fecha:** 2026-07-05

## Contexto

Con un segundo proveedor STT (Groq, ADR-0012) y cobros por uso, el usuario necesita visibilidad
del gasto que genera cada API key registrada. Los proveedores cobran la transcripción por
duración de audio, pero **no exponen el gasto real de forma utilizable**: la API de uso de OpenAI
ya no responde con API keys normales (exige clave de admin/organización) y agregar llamadas de
facturación externas contradice los principios 1 (privacidad primero) y 10 (observabilidad sin
exposición). Se necesita una fuente que funcione offline, sin cuentas extra y sin filtrar datos.

## Decisión

### Fuente: estimación local

El gasto se **estima localmente**, nunca se consulta una API de facturación. Tras cada
transcripción exitosa se acumula la duración del audio (ya disponible en el ciclo,
`AudioData::duration_ms`) y se aplica:

```
gasto_estimado_usd = (segundos_audio / 60) × tarifa_por_minuto(proveedor, modelo)
```

Es una **estimación**, no la factura del proveedor — la UI lo dice explícitamente. Justificación:
es lo único viable con una API key normal, respeta privacidad (cero red) y sirve para el objetivo
real (saber el orden de magnitud del gasto y compararlo entre proveedores/modelos).

### Desglose: por proveedor + modelo, histórico mensual

El ledger acumula por la clave **`(proveedor, modelo, mes)`** (mes en `YYYY-MM`, hora local):

- `transcriptions` — cantidad de transcripciones.
- `audio_seconds` — segundos de audio acumulados.
- `estimated_cost_usd` — gasto estimado acumulado.

La pantalla agrupa por proveedor (≡ "por API key registrada"), muestra el mes en curso y el total
histórico, con desglose por modelo, y permite **reset manual** (por proveedor o global).

### Tabla de tarifas por defecto, editable

Se embarca una tabla de tarifas por defecto (USD por minuto de audio) por `(proveedor, modelo)`,
**editable** por el usuario en la UI, porque los precios cambian y los modelos `gpt-4o-*-transcribe`
se cobran por tokens de audio (el por-minuto es una aproximación). Los overrides del usuario se
guardan en settings (nueva sección `pricing`); los defaults viven en código.

Valores por defecto iniciales (aprox., revisar al implementar):

| Proveedor | Modelo | USD/min |
|---|---|---|
| openai | gpt-4o-mini-transcribe | 0.003 |
| openai | gpt-4o-transcribe | 0.006 |
| openai | whisper-1 | 0.006 |
| groq | whisper-large-v3-turbo | 0.00067 |
| groq | whisper-large-v3 | 0.00185 |

### Persistencia e integración

- **Archivo propio** `usage.json` en el dir de config del SO, versionado (`schema_version`),
  separado de `settings.json`: es telemetría de escritura frecuente con ciclo de vida distinto y
  no debe ensuciar ni arriesgar la config. Escritura atómica (tmp + rename), como settings.
- El orquestador registra el uso al recibir `TranscriptionCompleted` (conoce proveedor, modelo y
  duración). Un fallo al escribir el ledger **nunca** afecta el dictado (best-effort, warn al log).
- Borde IPC: comandos `get_usage` (devuelve el ledger agregado para la UI) y `reset_usage`
  (por proveedor o global). No requiere eventos de dominio nuevos; la pantalla consulta bajo
  demanda y refresca tras un ciclo.
- Nueva sección de navegación "Gastos" en Settings, i18n es/en. Toda cadena por i18n.

## Justificación

- Estimación local es la única fuente factible con API keys de usuario y la única coherente con
  los principios 1 y 10 (sin red, sin exposición).
- El desglose por proveedor+modelo mensual coincide con cómo facturan los proveedores (ciclos
  mensuales) y permite comparar el costo real de Groq vs OpenAI por modelo.
- Tarifas editables evitan que la estimación quede obsoleta al cambiar los precios.
- Archivo separado mantiene la config a salvo y simplifica el reset.

## Consecuencias

- (+) Visibilidad del gasto offline, privada, por API key, sin cuentas ni permisos extra.
- (+) Ayuda a decidir proveedor/modelo por costo, reforzando ADR-0012.
- (−) Es una estimación: puede divergir de la factura real (redondeos del proveedor, precios por
  token en los `gpt-4o-*-transcribe`, cambios de tarifa). Se comunica como estimación en la UI.
- (−) Nuevo archivo persistido (`usage.json`) y una tabla de tarifas que hay que mantener al día.
- (−) La duración contabilizada es la del audio enviado; si el proveedor factura distinto
  (mínimos por request, etc.) la estimación no lo refleja.
