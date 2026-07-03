//! Módulos de negocio por capacidad — esqueleto (M0). Ver docs/ARCHITECTURE.md
//! §4 y §8.4. Implementación real: M1+.
//!
//! `dead_code` desactivado a nivel de crate: en el esqueleto de M0 la mayoría
//! de los tipos (contratos ya fijados por ARCHITECTURE.md) todavía no se
//! construyen en ningún camino de ejecución real — eso llega con el wiring de
//! M1+. Retirar este allow cuando el core empiece a instanciar estos tipos.
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            ipc::commands::get_settings,
            ipc::commands::set_settings,
            ipc::commands::set_api_key,
            ipc::commands::test_provider,
            ipc::commands::get_app_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
