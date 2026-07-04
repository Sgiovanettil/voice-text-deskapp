//! Modelo de settings, versionado (`schema_version`) con migraciones. Ver
//! docs/ARCHITECTURE.md §4.7.

use serde::{Deserialize, Serialize};

/// Versión actual del esquema. Las migraciones (cuando existan v2+) viven en
/// `persistence::load_settings`.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub general: GeneralSettings,
    #[serde(default)]
    pub stt: SttSettings,
    #[serde(default)]
    pub delivery: DeliverySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralSettings {
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default = "default_true")]
    pub start_minimized: bool,
    #[serde(default = "default_output_mode")]
    pub output_mode: String,
    /// Posición del overlay en píxeles físicos; `None` = abajo-centro.
    #[serde(default)]
    pub overlay_position: Option<OverlayPos>,
}

/// Posición persistida del overlay (píxeles físicos del monitor).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct OverlayPos {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SttSettings {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_stt_language")]
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DeliverySettings {
    #[serde(default)]
    pub paste_combo_overrides: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub fallback_typing: bool,
}

// Defaults centralizados (§4.7, esquema conceptual). En M1 el output_mode
// efectivo es "clipboard" aunque el default documentado sea "insert" — la
// inserción llega en M2; el orquestador degrada explícitamente mientras tanto.
fn default_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}
fn default_ui_language() -> String {
    "es".into()
}
fn default_hotkey() -> String {
    "Ctrl+Super+Space".into()
}
fn default_true() -> bool {
    true
}
fn default_output_mode() -> String {
    "insert".into()
}
fn default_provider() -> String {
    "openai".into()
}
fn default_model() -> String {
    "gpt-4o-mini-transcribe".into()
}
fn default_stt_language() -> String {
    "auto".into()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            general: GeneralSettings::default(),
            stt: SttSettings::default(),
            delivery: DeliverySettings::default(),
        }
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            ui_language: default_ui_language(),
            hotkey: default_hotkey(),
            autostart: false,
            start_minimized: true,
            output_mode: default_output_mode(),
            overlay_position: None,
        }
    }
}

impl Default for SttSettings {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: default_model(),
            language: default_stt_language(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_coinciden_con_el_esquema_documentado() {
        let s = Settings::default();
        assert_eq!(s.schema_version, 1);
        assert_eq!(s.general.ui_language, "es");
        assert_eq!(s.general.hotkey, "Ctrl+Super+Space");
        assert!(!s.general.autostart);
        assert!(s.general.start_minimized);
        assert_eq!(s.general.output_mode, "insert");
        assert_eq!(s.stt.provider, "openai");
        assert_eq!(s.stt.model, "gpt-4o-mini-transcribe");
        assert_eq!(s.stt.language, "auto");
        assert!(!s.delivery.fallback_typing);
    }

    #[test]
    fn json_parcial_completa_con_defaults() {
        // Un settings.json antiguo o editado a mano no debe romper la carga.
        let s: Settings = serde_json::from_str(r#"{ "stt": { "language": "es" } }"#).unwrap();
        assert_eq!(s.schema_version, 1);
        assert_eq!(s.stt.language, "es");
        assert_eq!(s.stt.provider, "openai");
        assert_eq!(s.general.hotkey, "Ctrl+Super+Space");
    }
}
