# ADR-0006 — Secretos en keyring del SO; configuración versionada en archivo

- **Estado:** Aceptada · **Fecha:** 2026-07-02

## Contexto
Principios de seguridad del PRD (§15): secretos nunca en texto plano, mecanismo nativo por SO. Las preferencias deben sobrevivir upgrades sin romperse.

## Decisión
- **Secretos (API keys):** `keyring-rs` → Windows Credential Manager / Secret Service (Linux). La key se lee del keyring solo al momento de usarla; nunca se incluye en el archivo de settings, ni cruza el IPC (el frontend ve `is_set` + últimos 4 caracteres).
- **Fallback sin Secret Service (R4):** advertencia explícita y almacenamiento alternativo solo con consentimiento del usuario; nunca silencioso.
- **Preferencias:** `settings.json` en el directorio de configuración estándar del SO, con campo `schema_version` y migraciones automáticas hacia adelante desde la v1.

## Consecuencias
- (+) Cumple PRD §15.1–15.2 por construcción.
- (−) Dependencia del estado del keyring del sistema en Linux (mitigada con detección + fallback consentido).
- (−) Las migraciones de esquema exigen disciplina: todo cambio de settings incrementa versión y añade migración (checklist de PR).
