//! Módulos de negocio por capacidad. Ver docs/ARCHITECTURE.md §4 y §8.4.
//!
//! `dead_code` desactivado a nivel de crate: quedan tipos de contrato
//! (ARCHITECTURE.md) que recién se instancian con el orquestador de M1-PR8.
//! Retirar este allow cuando el wiring del ciclo esté completo.
#![allow(dead_code)]

mod audio;
mod config;
mod core;
mod delivery;
mod hotkeys;
mod ipc;
mod persistence;
mod providers;
mod speech;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            let settings = persistence::load_settings(&config_dir);
            let hotkey = settings.general.hotkey.clone();

            let event_tx = core::orchestrator::spawn(app.handle().clone());
            app.manage(ipc::commands::AppState::new(
                config_dir,
                settings,
                event_tx.clone(),
            ));

            // Si el hotkey no se puede registrar la app arranca igual: el
            // usuario lo corrige desde settings (set_settings re-registra).
            if let Err(e) = hotkeys::register_ptt(app.handle(), &hotkey, move |ev| {
                let _ = event_tx.send(ev);
            }) {
                tracing::error!(error = %e, hotkey, "no se pudo registrar el hotkey inicial");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::get_settings,
            ipc::commands::set_settings,
            ipc::commands::set_api_key,
            ipc::commands::get_api_key_status,
            ipc::commands::test_provider,
            ipc::commands::get_app_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
