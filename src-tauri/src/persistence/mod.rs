//! Persistencia de `Settings` en archivo de config del SO y secretos en
//! keyring del SO (ADR-0006). Ver docs/ARCHITECTURE.md §4.7. Implementación: M3.

use crate::config::Settings;

pub fn load_settings() -> Settings {
    todo!("M3: leer settings.json del dir de config del SO, aplicar migraciones")
}

pub fn save_settings(_settings: &Settings) {
    todo!("M3: escribir settings.json")
}

pub fn get_api_key() -> Option<String> {
    todo!("M3: keyring-rs → Credential Manager (Windows) / Secret Service (Linux)")
}
