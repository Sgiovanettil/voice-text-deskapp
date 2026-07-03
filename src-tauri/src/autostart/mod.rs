//! Sincroniza el arranque automático del SO con el setting
//! `general.autostart`. El setting es la fuente de verdad: en cada arranque y
//! en cada guardado se reconcilia el estado real del SO (registro en Windows,
//! LaunchAgent en macOS, `.desktop` en Linux) contra él. Detrás del plugin
//! multiplataforma `tauri-plugin-autostart` (ARCHITECTURE.md §1): sin
//! `#[cfg(target_os)]`.

use tauri::{AppHandle, Runtime};
use tauri_plugin_autostart::ManagerExt;

/// Deja el arranque automático del SO en `want`. Best-effort: si el gestor
/// falla (permisos, entorno sin soporte) se registra y la app sigue — el
/// arranque automático es una comodidad, no un requisito para operar.
pub fn reconcile<R: Runtime>(app: &AppHandle<R>, want: bool) {
    let mgr = app.autolaunch();
    match mgr.is_enabled() {
        // Ya coincide: nada que hacer.
        Ok(enabled) if enabled == want => {}
        Ok(_) => {
            let res = if want { mgr.enable() } else { mgr.disable() };
            if let Err(e) = res {
                tracing::warn!(error = %e, want, "no se pudo ajustar el arranque automático");
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "no se pudo consultar el arranque automático")
        }
    }
}
