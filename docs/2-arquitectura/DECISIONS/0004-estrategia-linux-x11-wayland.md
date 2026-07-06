# ADR-0004 — Linux: X11 y Wayland como targets de primera clase

- **Estado:** Aceptada (revisada 2026-07-02; reemplaza la versión "X11 garantizado, Wayland best-effort") · **Fecha:** 2026-07-02

## Contexto
Wayland restringe por diseño los hotkeys globales y la síntesis de input desde aplicaciones normales. La versión anterior de este ADR relegaba Wayland a best-effort; el autor decidió que **X11 y Wayland son ambos targets de primera clase del MVP**, aplicando estrategias de degradación funcional explícitas solo donde el protocolo lo imponga.

## Decisión
1. **Detección en runtime** del tipo de sesión (`XDG_SESSION_TYPE`, presencia de portals) y selección de backend por función.
2. **Hotkey global:**
   - X11: registro directo (plugin global-shortcut).
   - Wayland: **XDG Desktop Portal `GlobalShortcuts`** (soportado por KDE Plasma y GNOME modernos). El usuario aprueba el atajo una vez vía el diálogo del portal.
   - Degradación explícita si el portal no existe (compositores wlroots antiguos): la app expone un **comando de activación externo** (CLI/D-Bus `app --dictate`) para que el usuario lo asocie a un atajo nativo de su compositor; la UI lo explica con instrucciones.
3. **Inserción de texto (complementa ADR-0005):**
   - X11: pegado sintético directo.
   - Wayland: emisión del atajo de pegado vía **portal `RemoteDesktop`** (con consentimiento del usuario) o **`ydotool`/uinput** si está disponible; la escritura al clipboard funciona de forma nativa en ambos.
   - Degradación explícita: si ninguna vía de síntesis está disponible, el modo `insert` se deshabilita con mensaje claro y la app opera en modo `clipboard` (siempre garantizado).
4. **Overlay:** validar en el spike R2 el comportamiento de `focusable:false`/always-on-top en los compositores objetivo (GNOME, KDE, wlroots) además de X11 y Windows.
5. **Matriz de soporte declarada** (documentada en README): Windows, Linux X11, Linux Wayland GNOME/KDE = experiencia completa; Wayland sin portals = activación externa + clipboard.

## Consecuencias
- (+) Linux moderno (la mayoría de las distros ya usan Wayland por defecto) es ciudadano de primera clase desde v1.0.
- (+) Las degradaciones son explícitas, visibles y documentadas — nunca fallos silenciosos (principio 9).
- (−) Aumenta el alcance del MVP: dos backends para hotkeys y para síntesis de input en Linux, y una matriz de pruebas más grande (impacto en M0–M2 del roadmap; los spikes de riesgo ahora incluyen Wayland obligatoriamente).
- (−) Dependencia del ecosistema de portals (versiones de xdg-desktop-portal por distro); el spike debe fijar las versiones mínimas soportadas.
- La abstracción interna por función (hotkey-backend, input-synthesis-backend, clipboard-backend) queda como requisito de diseño en `hotkeys/` y `delivery/` (ARCHITECTURE §4.2, §4.6).
