# Spike R2 — Overlay sin foco

- **Binario:** `cargo run --bin spike-overlay`
- **ADR que alimenta:** ADR-0004 (Linux X11/Wayland), diseño del overlay (ARCHITECTURE §4.9)
- **Fecha de ejecución:** <completar>
- **Ejecutado por:** <completar>

## Protocolo

1. Abrir un editor de texto (o cualquier app) y dejar el cursor en un campo de texto.
2. Ejecutar `cargo run --bin spike-overlay`. Debería aparecer una ventana verde "ESCUCHANDO" siempre-encima.
3. Sin tocar el mouse, escribir un carácter en el teclado.
4. Verificar: ¿el carácter apareció en el editor (foco conservado) o en la ventana del spike (foco robado)?

## Resultados por entorno

| Entorno              | Versión                      | ¿Robó el foco? | ¿Se ve always-on-top? | Notas |
| --------------------- | ----------------------------- | --------------- | ----------------------- | ----- |
| Windows 10/11          |                                |                  |                          |       |
| Linux X11               | (DE, ej. GNOME/KDE sobre X11) |                  |                          |       |
| Linux Wayland GNOME     | (versión GNOME)               |                  |                          |       |
| Linux Wayland KDE       | (versión Plasma)              |                  |                          |       |

## Conclusión

- [ ] `focusable:false` es suficiente en todos los entornos → ADR-0004 queda confirmado sin cambios
- [ ] Se necesitó un workaround adicional en: <entorno> → documentar y decidir si actualiza el ADR
- [ ] Bloqueante — replantear diseño del overlay antes de M2

## Evidencia

(capturas de pantalla opcionales)
