# ADR-0013 — Diccionario personal / reemplazos (v1.x)

- **Estado:** Aceptada (diseño; implementación en v1.x) · **Fecha:** 2026-07-05

## Contexto

Los STT transcriben mal nombres propios, siglas y jerga del usuario ("graphify", "Tauri",
apellidos). El roadmap v1.x incluye "diccionario personal/reemplazos" sin spec. Esta ADR fija
formato, punto de aplicación, UI y persistencia.

## Decisión

### Formato (v1.x)

Lista ordenada de reglas de **reemplazo literal** (sin regex ni fuzzy en v1.x — extensión
futura explícita):

```jsonc
"dictionary": {
  "enabled": true,
  "rules": [
    { "from": "grafifai", "to": "graphify", "case_insensitive": true },
    { "from": "taurí", "to": "Tauri", "case_insensitive": true }
  ]
}
```

- Coincidencia por **palabra completa** (límites Unicode), para no corromper substrings.
- `case_insensitive` por regla (default `true`); el reemplazo usa `to` tal cual está escrito.
- Aplicación **en el orden de la tabla**, una sola pasada, determinista; el resultado de una
  regla no se re-procesa por las siguientes.

### Punto de aplicación

En **Rust, dentro del core**, como paso único entre `TranscriptionCompleted` y la entrega
(`delivery`) — nunca en el frontend (ADR-0002: todo el negocio en Rust). Pre-transcripción se
descartó: los endpoints STT no aceptan vocabulario custom de forma uniforme entre proveedores,
y el post-procesamiento es agnóstico del proveedor.

Sin eventos de dominio nuevos: el texto ya corregido viaja en los eventos existentes. (Si en el
futuro se quiere trazabilidad, se puede agregar un campo opcional — nunca renombrar.)

### UI y persistencia

- Tabla editable (agregar/editar/eliminar/reordenar) en una sección nueva de settings, con
  toggle general `enabled`. Toda cadena por i18n.
- Persistencia dentro del `settings.json` versionado existente (sección `dictionary`); viaja
  con el mecanismo de migraciones actual. Import/export a archivo queda anotado como futuro.

## Justificación

- Reemplazos literales por palabra completa cubren el 90% del caso de uso (nombres propios y
  siglas) con implementación trivial y comportamiento predecible para el usuario.
- Post-STT en el core: un solo punto, agnóstico del proveedor, testeable puro.
- Reutilizar `settings.json` evita otro archivo/formato y hereda migraciones y save atómico.

## Consecuencias

- (+) Corrige errores recurrentes del STT sin tocar proveedores; testeable con unit tests puros.
- (+) Compatible con ADR-0014: el diccionario corrige el reconocimiento **antes** de cualquier
  post-procesado LLM.
- (−) Sin regex, casos avanzados (patrones, plurales) quedan fuera de v1.x.
- (−) Tablas grandes se aplican en cada dictado — irrelevante en la práctica (< 1 ms para
  cientos de reglas), pero fija el límite de no hacer nada más pesado en esta etapa.
