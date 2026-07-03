# ADR-0010 — Auto-update preparado desde v1.0, activo post-MVP

- **Estado:** Aceptada · **Fecha:** 2026-07-02

## Contexto
El auto-updater funcional está fuera del MVP, pero el PRD exige que exista como decisión de arquitectura desde el inicio: activarlo después no debe requerir cambios estructurales. El mecanismo elegido es el **updater oficial de Tauri** (plugin `tauri-plugin-updater`), que exige artefactos firmados con un par de claves propio y un manifiesto de versiones.

## Decisión
Desde la **primera release**:
1. Generar y custodiar el par de claves de firma del updater (privada como secreto de CI; pública embebida en `tauri.conf.json`).
2. Firmar todos los artefactos en el pipeline de release y publicar el manifiesto (`latest.json`) como asset del Release de GitHub.
3. Reservar el componente `Updater` en la arquitectura (ARCHITECTURE §4.11) sin implementarlo.

Activación (v1.x): habilitar el plugin + UI de notificación (detectar, avisar, descargar, instalar). Endpoint: el propio Release de GitHub.

## Consecuencias
- (+) Los usuarios de v1.0 podrán actualizarse automáticamente a v1.x sin reinstalar a mano.
- (+) Cero re-arquitectura al activarlo.
- (−) Gestión de la clave privada desde el día 1 (perderla = romper la cadena de actualización; guardarla también fuera de CI).
- (−) Cobertura Linux parcial: el updater cubre AppImage; `.deb` se actualiza por gestor de paquetes — documentado en README.
