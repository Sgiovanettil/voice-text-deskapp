//! Registro del hotkey global. Abstracción con backends por plataforma
//! (ADR-0004): Windows/X11 vía tauri-plugin-global-shortcut, Wayland vía XDG
//! Desktop Portal GlobalShortcuts. Ver docs/ARCHITECTURE.md §4.2. Implementación: M1.

// TODO(M1): traducir press/release del acelerador configurado a
// HotkeyPressed/HotkeyReleased (core/events.rs, PR5); re-registro ante
// ConfigChanged; degradación explícita si no hay portal (activación externa
// `app --dictate`, ver ADR-0004).
