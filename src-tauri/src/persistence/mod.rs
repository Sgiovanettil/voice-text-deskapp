//! Persistencia de `Settings` en archivo de config del SO y secretos en
//! keyring del SO (ADR-0006). Ver docs/ARCHITECTURE.md §4.7.
//!
//! El directorio de config entra como parámetro (lo resuelve el borde Tauri
//! con `app.path().app_config_dir()`) para mantener esta capa testeable.

use std::path::{Path, PathBuf};

use crate::config::{Settings, CURRENT_SCHEMA_VERSION};

const SETTINGS_FILE: &str = "settings.json";
/// Identificador estable del servicio en el almacén de credenciales del SO.
/// Coincide con el identifier de tauri.conf.json — no cambiar tras release.
const KEYRING_SERVICE: &str = "dev.sgiovanettil.voicetext";
const KEYRING_USER_API_KEY: &str = "openai_api_key";

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialización: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("keyring: {0}")]
    Keyring(String),
    #[error("schema_version {found} no soportada (máx {max})")]
    UnsupportedSchema { found: u32, max: u32 },
}

fn settings_path(config_dir: &Path) -> PathBuf {
    config_dir.join(SETTINGS_FILE)
}

/// Carga settings desde disco. Si el archivo no existe devuelve defaults;
/// si está corrupto, resetea a defaults con warning (la app siempre arranca).
pub fn load_settings(config_dir: &Path) -> Settings {
    let path = settings_path(config_dir);
    let raw = match std::fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Settings::default(),
        Err(e) => {
            tracing::warn!(error = %e, ?path, "no se pudo leer settings.json — usando defaults");
            return Settings::default();
        }
    };
    match parse_settings(&raw) {
        Ok(settings) => settings,
        Err(e) => {
            tracing::warn!(error = %e, ?path, "settings.json inválido — usando defaults");
            Settings::default()
        }
    }
}

fn parse_settings(raw: &str) -> Result<Settings, PersistenceError> {
    let settings: Settings = serde_json::from_str(raw)?;
    if settings.schema_version > CURRENT_SCHEMA_VERSION {
        // Archivo escrito por una versión más nueva de la app: no adivinar.
        return Err(PersistenceError::UnsupportedSchema {
            found: settings.schema_version,
            max: CURRENT_SCHEMA_VERSION,
        });
    }
    // Migraciones v(n)→v(n+1) se encadenan aquí cuando existan.
    Ok(settings)
}

/// Escritura atómica: tmp + rename, para no dejar un JSON a medias si la app
/// muere durante el write.
pub fn save_settings(config_dir: &Path, settings: &Settings) -> Result<(), PersistenceError> {
    std::fs::create_dir_all(config_dir)?;
    let path = settings_path(config_dir);
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(settings)?)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

fn keyring_entry() -> Result<keyring::Entry, PersistenceError> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER_API_KEY)
        .map_err(|e| PersistenceError::Keyring(e.to_string()))
}

/// Lee la API key del almacén del SO. `None` si no está seteada.
pub fn get_api_key() -> Result<Option<String>, PersistenceError> {
    match keyring_entry()?.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(PersistenceError::Keyring(e.to_string())),
    }
}

pub fn set_api_key(key: &str) -> Result<(), PersistenceError> {
    keyring_entry()?
        .set_password(key)
        .map_err(|e| PersistenceError::Keyring(e.to_string()))
}

pub fn delete_api_key() -> Result<(), PersistenceError> {
    match keyring_entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(PersistenceError::Keyring(e.to_string())),
    }
}

/// Representación segura de la key para la UI (§4.7: el frontend nunca ve la
/// key completa — solo si está seteada y sus últimos 4 caracteres).
pub fn mask_api_key(key: &str) -> String {
    let tail: String = key
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("…{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_config_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("voicetext-test-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn load_sin_archivo_devuelve_defaults() {
        let dir = temp_config_dir("load-missing");
        assert_eq!(load_settings(&dir), Settings::default());
    }

    #[test]
    fn save_y_load_roundtrip() {
        let dir = temp_config_dir("roundtrip");
        let mut settings = Settings::default();
        settings.stt.language = "es".into();
        settings.general.autostart = true;

        save_settings(&dir, &settings).unwrap();
        assert_eq!(load_settings(&dir), settings);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn archivo_corrupto_resetea_a_defaults() {
        let dir = temp_config_dir("corrupt");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(SETTINGS_FILE), "{esto no es json").unwrap();
        assert_eq!(load_settings(&dir), Settings::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn schema_del_futuro_no_se_carga() {
        let raw = r#"{ "schema_version": 99 }"#;
        assert!(matches!(
            parse_settings(raw),
            Err(PersistenceError::UnsupportedSchema { found: 99, .. })
        ));
        // Y vía load_settings cae a defaults (la app arranca igual).
        let dir = temp_config_dir("future");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(SETTINGS_FILE), raw).unwrap();
        assert_eq!(load_settings(&dir), Settings::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mask_api_key_muestra_solo_ultimos_4() {
        assert_eq!(mask_api_key("sk-abcdefgh1234"), "…1234");
        assert_eq!(mask_api_key("abc"), "…abc");
    }
}
