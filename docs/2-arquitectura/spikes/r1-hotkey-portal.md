# Spike R1 — Hotkey global vía XDG Desktop Portal (Wayland)

- **Binario:** `cargo run --bin spike-hotkey-portal` (solo Linux — en otros SO imprime un mensaje y termina)
- **ADR que alimenta:** [ADR-0004](../DECISIONS/0004-estrategia-linux-x11-wayland.md)
- **Fecha de ejecución:** <completar>
- **Ejecutado por:** <completar>

## Protocolo

1. Correr `cargo run --bin spike-hotkey-portal` en una sesión Wayland real (GNOME o KDE Plasma).
2. Verificar que aparezca el diálogo de aprobación del portal `GlobalShortcuts` (solicita asignar un atajo).
3. Aprobar y asignar un atajo.
4. Presionar/soltar el atajo asignado y confirmar que el binario imprime el evento de activación.

## Resultados por entorno

| Entorno                     | Versión compositor/portal | ¿Apareció el diálogo? | ¿Llegaron los eventos? | Notas |
| ---------------------------- | -------------------------- | ----------------------- | ------------------------ | ----- |
| Wayland GNOME                 |                             |                          |                          |       |
| Wayland KDE Plasma             |                             |                          |                          |       |
| Wayland wlroots (sway, etc.)   |                             |                          |                          |       |

## Conclusión

- [ ] El portal `GlobalShortcuts` funciona de forma confiable en GNOME y KDE modernos → ADR-0004 confirmado, se fija la versión mínima de `xdg-desktop-portal` soportada: <versión>
- [ ] Falla o no existe el portal en: <compositor> → confirma la necesidad de la degradación explícita (activación externa `app --dictate`) ya prevista en ADR-0004
- [ ] Bloqueante — replantear estrategia de hotkeys en Wayland antes de M1

## Evidencia

(capturas de pantalla opcionales)
