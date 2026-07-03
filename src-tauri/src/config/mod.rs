//! Modelo de settings, versionado (`schema_version`) con migraciones. Ver
//! docs/ARCHITECTURE.md §4.7. Loader, validación y migraciones: M1/M3.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub schema_version: u32,
    pub general: GeneralSettings,
    pub stt: SttSettings,
    pub delivery: DeliverySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub ui_language: String,
    pub hotkey: String,
    pub autostart: bool,
    pub start_minimized: bool,
    pub output_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttSettings {
    pub provider: String,
    pub model: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeliverySettings {
    pub paste_combo_overrides: std::collections::HashMap<String, String>,
    pub fallback_typing: bool,
}

// TODO(M1): defaults centralizados, validación, migraciones desde schema_version=1.
