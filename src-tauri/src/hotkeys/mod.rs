//! Registro del hotkey global. Abstracción con backends por plataforma
//! (ADR-0004): Windows/X11 vía tauri-plugin-global-shortcut, Wayland vía XDG
//! Desktop Portal GlobalShortcuts. Ver docs/2-arquitectura/ARCHITECTURE.md §4.2.
//!
//! M1 implementa el backend Windows/X11 (plugin). El backend Wayland/portal y
//! la degradación por activación externa (`app --dictate`) llegan al retomar
//! Linux. Este módulo solo traduce press/release del acelerador a eventos de
//! dominio — la decisión de qué hacer con ellos es de la máquina de estados.

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::core::events::DomainEvent;

#[derive(Debug, thiserror::Error)]
pub enum HotkeyError {
    #[error("acelerador inválido: {0}")]
    InvalidAccelerator(String),
    #[error("registro falló: {0}")]
    Register(String),
}

impl HotkeyError {
    /// Clave i18n para la UI (catálogo err.* en src/i18n).
    pub fn error_key(&self) -> &'static str {
        match self {
            HotkeyError::InvalidAccelerator(_) => "err.hotkey.invalid",
            HotkeyError::Register(_) => "err.hotkey.register",
        }
    }
}

/// Valida un acelerador sin registrarlo (para `set_settings` desde la UI).
pub fn parse_accelerator(accelerator: &str) -> Result<Shortcut, HotkeyError> {
    accelerator
        .parse::<Shortcut>()
        .map_err(|e| HotkeyError::InvalidAccelerator(format!("{accelerator}: {e}")))
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Registra el acelerador PTT: press → `HotkeyPressed`, release →
/// `HotkeyReleased` (ADR-0008, PTT puro). El callback corre en el hilo del
/// plugin — debe limitarse a encolar el evento hacia el core.
pub fn register_ptt<R: tauri::Runtime>(
    app: &AppHandle<R>,
    accelerator: &str,
    on_event: impl Fn(DomainEvent) + Send + Sync + 'static,
) -> Result<(), HotkeyError> {
    let shortcut = parse_accelerator(accelerator)?;
    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            let timestamp = now_ms();
            let domain_event = match event.state() {
                ShortcutState::Pressed => DomainEvent::HotkeyPressed { timestamp },
                ShortcutState::Released => DomainEvent::HotkeyReleased { timestamp },
            };
            on_event(domain_event);
        })
        .map_err(|e| HotkeyError::Register(e.to_string()))
}

/// Quita el acelerador actual (paso previo al re-registro ante `ConfigChanged`).
pub fn unregister<R: tauri::Runtime>(
    app: &AppHandle<R>,
    accelerator: &str,
) -> Result<(), HotkeyError> {
    let shortcut = parse_accelerator(accelerator)?;
    app.global_shortcut()
        .unregister(shortcut)
        .map_err(|e| HotkeyError::Register(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acelerador_por_defecto_de_config_es_valido() {
        // El default de config/ ("Ctrl+Super+Space") tiene que parsear —
        // si esto rompe, la app arrancaría sin hotkey utilizable.
        let default = crate::config::GeneralSettings::default().hotkey;
        assert!(parse_accelerator(&default).is_ok());
    }

    #[test]
    fn aceleradores_tipicos_parsean() {
        for accel in ["Ctrl+Shift+D", "Alt+Space", "F9", "Super+V"] {
            assert!(parse_accelerator(accel).is_ok(), "falló: {accel}");
        }
    }

    #[test]
    fn acelerador_invalido_da_error_con_clave_i18n() {
        let err = parse_accelerator("NoEsUnAtajo++").unwrap_err();
        assert_eq!(err.error_key(), "err.hotkey.invalid");
    }
}
