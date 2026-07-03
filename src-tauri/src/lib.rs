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
mod tray;

use tauri::Manager;

/// Mantiene vivo el writer no bloqueante de tracing-appender durante toda la
/// vida de la app (si se dropea, los logs dejan de escribirse).
struct LogGuard(tracing_appender::non_blocking::WorkerGuard);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .on_window_event(|window, event| {
            // Cerrar la ventana de configuración la oculta a la bandeja en vez
            // de terminar la app; se sale solo desde el menú del tray.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "settings" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .setup(|app| {
            // Logs a archivo (rotación diaria) en el dir de logs del SO —
            // en Windows: %LOCALAPPDATA%\dev.sgiovanettil.voicetext\logs.
            // El guard debe vivir tanto como la app o el writer se cierra.
            let log_dir = app.path().app_log_dir()?;
            let (writer, guard) = tracing_appender::non_blocking(tracing_appender::rolling::daily(
                &log_dir,
                "voicetext.log",
            ));
            tracing_subscriber::fmt()
                .with_writer(writer)
                .with_ansi(false)
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
                )
                .init();
            app.manage(LogGuard(guard));
            tracing::info!(version = env!("CARGO_PKG_VERSION"), "VoiceText iniciando");

            let config_dir = app.path().app_config_dir()?;
            let settings = persistence::load_settings(&config_dir);
            let hotkey = settings.general.hotkey.clone();
            let ui_language = settings.general.ui_language.clone();
            let start_minimized = settings.general.start_minimized;

            // Bandeja del sistema: la app queda residente. La ventana de
            // configuración arranca oculta (`visible: false` en la conf) y se
            // muestra ahora salvo que el usuario pida arrancar minimizado.
            tray::build(app.handle(), &ui_language)?;
            if !start_minimized {
                tray::show_settings(app.handle());
            }

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
