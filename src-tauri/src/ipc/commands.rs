//! Comandos IPC (frontend → core). Validación en el borde; errores mapeados a
//! `{ code, error_key }` (ARCHITECTURE §4.8).

use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Mutex;

use tauri::{Emitter, State};

use crate::config::Settings;
use crate::core::events::DomainEvent;
use crate::core::state_machine::CoreState;
use crate::persistence;

/// Estado gestionado por Tauri. `core_state` es el espejo consultable del
/// estado del ciclo (lo mantiene al día el orquestador); `event_tx` es la
/// entrada de eventos al orquestador (la usa el re-registro de hotkey).
pub struct AppState {
    pub config_dir: PathBuf,
    pub settings: Mutex<Settings>,
    pub core_state: Mutex<CoreState>,
    pub event_tx: Sender<DomainEvent>,
}

impl AppState {
    pub fn new(config_dir: PathBuf, settings: Settings, event_tx: Sender<DomainEvent>) -> Self {
        Self {
            config_dir,
            settings: Mutex::new(settings),
            core_state: Mutex::new(CoreState::Idle),
            event_tx,
        }
    }
}

/// Error del borde IPC: `code` para logs, `error_key` para que la UI
/// traduzca (catálogo err.* de src/i18n).
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IpcError {
    pub code: String,
    pub error_key: String,
}

impl IpcError {
    pub(crate) fn new(code: impl Into<String>, error_key: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            error_key: error_key.into(),
        }
    }
}

impl From<persistence::PersistenceError> for IpcError {
    fn from(e: persistence::PersistenceError) -> Self {
        let key = match &e {
            persistence::PersistenceError::Keyring(_) => "err.keyring.unavailable",
            _ => "err.config.invalid",
        };
        IpcError::new(e.to_string(), key)
    }
}

impl From<crate::hotkeys::HotkeyError> for IpcError {
    fn from(e: crate::hotkeys::HotkeyError) -> Self {
        let key = e.error_key();
        IpcError::new(e.to_string(), key)
    }
}

/// Estado de la API key para la UI: nunca la key completa (§4.7).
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyStatus {
    pub is_set: bool,
    pub masked: Option<String>,
}

fn api_key_status(provider_id: &str) -> Result<ApiKeyStatus, IpcError> {
    let key = persistence::get_api_key(provider_id)?;
    Ok(ApiKeyStatus {
        is_set: key.is_some(),
        masked: key.as_deref().map(persistence::mask_api_key),
    })
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.lock().expect("settings lock").clone()
}

#[tauri::command]
pub fn set_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<(), IpcError> {
    // Validación en el borde: un hotkey imparseable no llega a disco.
    crate::hotkeys::parse_accelerator(&settings.general.hotkey)?;

    let previous = state
        .settings
        .lock()
        .expect("settings lock")
        .general
        .clone();
    let previous_hotkey = previous.hotkey.clone();

    // El arranque automático se aplica al SO cuando cambia el toggle; el
    // setting es la fuente de verdad (mismo criterio que en el arranque).
    if previous.autostart != settings.general.autostart {
        crate::autostart::reconcile(&app, settings.general.autostart);
    }

    if previous_hotkey != settings.general.hotkey {
        // Re-registro en caliente: primero el nuevo (si falla, se conserva el
        // anterior y el error llega a la UI), después se suelta el viejo.
        let tx = state.event_tx.clone();
        crate::hotkeys::register_ptt(&app, &settings.general.hotkey, move |ev| {
            let _ = tx.send(ev);
        })?;
        let _ = crate::hotkeys::unregister(&app, &previous_hotkey);
    }

    persistence::save_settings(&state.config_dir, &settings)?;
    *state.settings.lock().expect("settings lock") = settings;

    // Aviso a las webviews (overlay y settings) de que la config cambió, para
    // que refresquen lo que muestran (p. ej. proveedor/modelo del overlay).
    let _ = app.emit(
        "domain-event",
        &DomainEvent::ConfigChanged {
            changed_keys: vec!["settings".into()],
        },
    );
    Ok(())
}

#[tauri::command]
pub fn set_api_key(provider: String, key: String) -> Result<ApiKeyStatus, IpcError> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        persistence::delete_api_key(&provider)?;
    } else {
        persistence::set_api_key(&provider, trimmed)?;
    }
    api_key_status(&provider)
}

#[tauri::command]
pub fn get_api_key_status(provider: String) -> Result<ApiKeyStatus, IpcError> {
    api_key_status(&provider)
}

#[tauri::command]
pub async fn test_provider(provider: String, model: String) -> Result<bool, IpcError> {
    let key = persistence::get_api_key(&provider)?
        .ok_or_else(|| IpcError::new("api key no configurada", "err.stt.auth"))?;
    crate::providers::resolve(&provider, key)
        .check_auth(&model)
        .await
        .map_err(|e| {
            let key = match e {
                crate::speech::SpeechError::Auth => "err.stt.auth",
                crate::speech::SpeechError::Network => "err.stt.network",
                crate::speech::SpeechError::RateLimited => "err.stt.rate",
                _ => "err.stt.provider",
            };
            IpcError::new(e.to_string(), key)
        })?;
    Ok(true)
}

/// Micrófonos de entrada disponibles, por nombre (ARCHITECTURE §4.3). La UI
/// los ofrece en un dropdown; el elegido se guarda en `audio.input_device`.
#[tauri::command]
pub fn list_input_devices() -> Vec<String> {
    crate::audio::list_input_devices()
}

#[tauri::command]
pub fn get_app_state(state: State<'_, AppState>) -> String {
    let s = match *state.core_state.lock().expect("core state lock") {
        CoreState::Idle => "idle",
        CoreState::Recording => "recording",
        CoreState::Transcribing => "transcribing",
        CoreState::Delivering => "delivering",
        CoreState::Error => "error",
    };
    s.to_string()
}
