# Limitaciones y pendientes conocidos

Registro de limitaciones, pendientes y decisiones diferidas del proyecto. Cada entrada indica
síntoma, causa, estado (`Abierto` / `En progreso` / `Diferido` / `Resuelto`) y workaround.
Fuente de verdad del alcance: [PRD.md](PRD.md); seguimiento de avance: [ROADMAP.md](ROADMAP.md).

## Validación en Linux de los spikes de riesgo

- **Síntoma:** las columnas Linux (X11 y Wayland GNOME/KDE) de los spikes R1/R2/R3 están sin
  completar; la funcionalidad solo está validada end-to-end en Windows 11.
- **Causa:** M1–M4 se avanzaron enfocados en Windows con la evidencia positiva de R2/R3 en
  Windows 11 (2026-07-03); las pruebas en Linux quedaron pausadas (ver
  [ROADMAP.md](ROADMAP.md), nota de spikes).
- **Estado:** Abierto. R2 y R3 validados en Windows; R1
  ([hotkey portal](../2-arquitectura/spikes/r1-hotkey-portal.md)) sin ejecutar; columnas Linux de
  [R2](../2-arquitectura/spikes/r2-overlay-sin-foco.md) y
  [R3](../2-arquitectura/spikes/r3-insercion-texto.md) pendientes.
- **Workaround:** la app funciona en Windows; en Linux debe validarse antes de declarar soporte
  de primera clase según la matriz del [README](../../README.md).

## Degradación funcional en Wayland sin portals

- **Síntoma:** en compositores Wayland sin `xdg-desktop-portal` (p. ej. wlroots antiguos) el
  hotkey global y la inserción sintética de texto no están garantizados.
- **Causa:** Wayland restringe por diseño los hotkeys globales y la síntesis de input desde
  aplicaciones normales ([ADR-0004](../2-arquitectura/DECISIONS/0004-estrategia-linux-x11-wayland.md)).
- **Estado:** Abierto (degradación explícita por diseño, no es un fallo silencioso).
- **Workaround:** activación externa vía comando `app --dictate` asociado a un atajo nativo del
  compositor; el modo `insert` cae a modo `clipboard` (siempre garantizado) cuando no hay vía de
  síntesis disponible ([ADR-0004](../2-arquitectura/DECISIONS/0004-estrategia-linux-x11-wayland.md) §3).

## Versiones mínimas de `xdg-desktop-portal` sin fijar

- **Síntoma:** no está declarada la versión mínima soportada de `xdg-desktop-portal` para los
  portals `GlobalShortcuts` y `RemoteDesktop`.
- **Causa:** depende de ejecutar el spike
  [R1](../2-arquitectura/spikes/r1-hotkey-portal.md) en GNOME, KDE y wlroots reales.
- **Estado:** Abierto (bloqueado por la validación Linux de R1).
- **Workaround:** ninguno; se fija al completar R1.

## Firma Authenticode (Windows) diferida

- **Síntoma:** los instaladores de Windows no llevan firma Authenticode (posibles avisos de
  SmartScreen al instalar).
- **Causa:** el certificado Authenticode se difirió a después de v1.0
  (ver [DEPLOY_PREPROD.md](../3-desarrollo/DEPLOY_PREPROD.md)).
- **Estado:** Diferido. La firma del updater de Tauri (distinta de Authenticode) sí está activa.
- **Workaround:** los bundles del updater van firmados y verificados; la firma Authenticode se
  incorpora cuando se adquiera el certificado.

## Política de retención de datos del proveedor STT sin documentar

- **Síntoma:** falta el enlace a la política de retención vigente del proveedor de transcripción.
- **Causa:** pendiente de documentar antes de la primera release pública ([PRD.md](PRD.md) §15.8).
- **Estado:** Abierto.
- **Workaround:** el audio solo se envía al proveedor por acción explícita del usuario y nunca se
  persiste en disco salvo flag de debug ([README](../../README.md) §Privacidad).

## Nombre del producto provisional

- **Síntoma:** el producto usa el nombre provisional «VoiceText».
- **Causa:** el nombre definitivo aún no se decide ([PRD.md](PRD.md)).
- **Estado:** Abierto.
- **Workaround:** ninguno; no bloquea el desarrollo.

## Auto-update limitado en Linux al AppImage

- **Síntoma:** el auto-update de la app no actualiza el paquete `.deb`.
- **Causa:** en Linux el updater de Tauri cubre el AppImage; el `.deb` se actualiza por el gestor
  de paquetes de la distribución ([README](../../README.md) §Actualizaciones automáticas).
- **Estado:** Conocido (comportamiento esperado por diseño).
- **Workaround:** actualizar el `.deb` con el gestor de paquetes del sistema.
