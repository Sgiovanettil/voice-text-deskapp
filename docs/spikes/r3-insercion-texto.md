# Spike R3 — Mecanismo de inserción de texto (ADR-0005)

- **Binario:** `cargo run --bin spike-delivery -- "texto a insertar"`
- **ADR que resuelve:** [ADR-0005](../adr/0005-mecanismo-insercion-texto.md) — pasa de "Propuesta" a "Aceptada" con esta evidencia
- **Fecha de ejecución:** <completar>
- **Ejecutado por:** <completar>

## Protocolo

Para cada app de la matriz: hacer click en el campo de texto destino, correr el comando (da 3 s para posicionar el cursor), y verificar visualmente si el texto quedó insertado. Repetir la fila "terminal" cambiando el combo de pegado a mano (`Ctrl+Shift+V`) si `Ctrl+V` no funciona ahí.

Campo con contraseña: verificar explícitamente que el pegado **no** exponga el texto en claro de forma insegura, o que se comporte de forma segura y documentada.

## Matriz de compatibilidad

| App                              | SO      | ¿Se insertó? | Combo usado | Notas |
| --------------------------------- | ------- | ------------- | ------------ | ----- |
| VS Code                           | Windows |               | Ctrl+V       |       |
| VS Code                           | X11     |               | Ctrl+V       |       |
| Windows Terminal                  | Windows |               |              |       |
| gnome-terminal / kitty            | X11     |               |              |       |
| Navegador (campo web)             | Windows |               | Ctrl+V       |       |
| Navegador (campo web)             | X11     |               | Ctrl+V       |       |
| Notepad                           | Windows |               | Ctrl+V       |       |
| gedit                             | X11     |               | Ctrl+V       |       |
| Campo de contraseña               | Windows |               |              |       |
| Campo de contraseña               | X11     |               |              |       |

## Criterio de aceptación (del ADR-0005)

- [ ] El mecanismo primario (clipboard + pegado sintético) funciona en ≥ 90% de la matriz
- [ ] Los casos restantes quedan cubiertos por perfiles de pegado por app o por el fallback de tecleo simulado (`enigo`, off por defecto)
- [ ] El clipboard del usuario se restauró correctamente en todos los casos probados

## Conclusión

- [ ] ADR-0005 pasa a **Aceptada** tal como está redactado
- [ ] ADR-0005 requiere ajustes: <detallar>
- [ ] Bloqueante — replantear el mecanismo de inserción antes de M2

## Evidencia

(capturas de pantalla opcionales)
