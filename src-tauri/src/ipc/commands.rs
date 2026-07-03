//! Comandos IPC (frontend → core). Validación en el borde; errores mapeados a
//! `{ code, error_key }` (M1+). Cuerpos `todo!()` hasta que exista wiring real
//! — el objetivo de M0 es que el borde IPC completo compile (RNF-08).

use crate::config::Settings;

#[tauri::command]
pub fn get_settings() -> Settings {
    todo!("M1: leer settings vía persistence::load_settings")
}

#[tauri::command]
pub fn set_settings(_settings: Settings) {
    todo!("M1: validar y persistir vía persistence::save_settings")
}

#[tauri::command]
pub fn set_api_key(_key: String) {
    todo!("M3: guardar en keyring vía persistence")
}

#[tauri::command]
pub fn test_provider() -> bool {
    todo!("M1: probar credenciales contra el proveedor configurado")
}

#[tauri::command]
pub fn get_app_state() -> String {
    todo!("M1: estado actual de core::state_machine::CoreState")
}
