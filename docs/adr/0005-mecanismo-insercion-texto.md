# ADR-0005 — Mecanismo de inserción de texto en la aplicación activa

- **Estado:** Propuesta (pasa a Aceptada tras el spike de compatibilidad) · **Fecha:** 2026-07-02

## Contexto
RF-06: el texto transcrito debe poder insertarse donde está el cursor del usuario. El PRD (v0.2.0) exige evaluar alternativas en la fase de Arquitectura sin asumir implementación. Restricciones: no robar el foco (principio 3), sensación de inmediatez (principio 4), Windows + Linux/X11 garantizados (ADR-0004).

## Alternativas evaluadas

| Criterio | A. Clipboard + pegado sintético | B. Inyección de tecleo (enigo) | C. APIs de accesibilidad (UIA / AT-SPI2) | D. IME virtual |
|---|---|---|---|---|
| Velocidad (texto largo) | Instantánea | Lenta (~char/ms, notoria en >500 chars) | Instantánea | Instantánea |
| Fidelidad (acentos, emoji, layouts) | Excelente | Frágil (depende de layout/keymap) | Excelente | Excelente |
| Cobertura de apps | Muy alta (casi todo acepta pegar) | Alta, pero algunas apps bloquean input sintético | Irregular (terminales y apps no-accesibles fallan) | Alta |
| Efecto colateral | Toca el clipboard (mitigable: guardar/restaurar) | Ninguno | Ninguno | Requiere instalación/permiso a nivel de SO |
| Complejidad Win+X11 | Baja | Baja-media | Alta (dos APIs muy distintas) | Muy alta |
| Wayland futuro | Parcial (pegar sintético también restringido) | Vía ydotool/uinput | AT-SPI2 funciona parcialmente | Compleja |
| Caso borde conocido | Terminales pegan con Ctrl+Shift+V | Apps con anti-input-sintético | — | — |

## Decisión (propuesta)
Estrategia en capas para el modo `insert`:

1. **Primario — A:** copiar al clipboard, emitir la orden de pegado sintética, **restaurar el clipboard anterior** (solo texto en MVP; delay de restauración ~300 ms tras confirmar el pegado).
2. **Perfiles de pegado por aplicación:** combinación configurable según clase de ventana (default `Ctrl+V`; terminales `Ctrl+Shift+V`), con tabla editable en settings avanzados.
3. **Fallback configurable — B:** tecleo simulado para apps que bloqueen el pegado (off por defecto).
4. **C (accesibilidad)** se descarta en MVP por costo/cobertura; queda como candidata v2.x (también útil para leer contexto en capacidades futuras). **D** descartada por intrusividad.
5. El modo `clipboard` puro permanece siempre como salida garantizada (incluso Wayland).

## Validación requerida (spike, previo a M1)
Matriz de compatibilidad en Windows 10/11 y Linux X11: VS Code, terminal (Windows Terminal / gnome-terminal / kitty), navegador (campo web), app nativa (Notepad / gedit), campo con contraseña (debe NO pegarse o pegarse de forma segura — documentar comportamiento). Criterio de aceptación: primario A funciona en ≥ 90% de la matriz; casos restantes cubiertos por perfiles o fallback B.

## Consecuencias
- (+) Rápido, fiel al texto, cobertura amplia, complejidad contenida.
- (−) Manipula el clipboard (ventana de ~300 ms donde un gestor de clipboard externo puede capturar el texto — documentar).
- (−) La restauración solo-texto pierde formatos ricos previos del clipboard (limitación documentada, R3).
