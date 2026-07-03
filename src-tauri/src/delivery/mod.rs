//! Inserción de texto en la app activa y modo clipboard. Mecanismo según
//! ADR-0005. Ver docs/ARCHITECTURE.md §4.6.
//!
//! Dos modos (RF-06): `clipboard` (solo copiar, garantizado en todas las
//! plataformas) e `insert` (clipboard + pegado sintético con restauración del
//! clipboard anterior — el mecanismo validado por el spike R3 en Windows).
//! Todo el input sintético vive aquí; ningún `#[cfg(target_os)]` fuera de este
//! módulo (ADR-002).

use std::thread;
use std::time::Duration;

use enigo::{
    Direction::{Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use serde::{Deserialize, Serialize};

/// Espera tras el pegado antes de restaurar el clipboard anterior, para dar
/// tiempo a que la app destino lea el contenido (ADR-0005: ~300 ms).
const RESTORE_DELAY: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryMode {
    Insert,
    Clipboard,
}

/// Combinación de pegado por clase de app (ADR-0005): la mayoría usa `Ctrl+V`;
/// varios terminales requieren `Ctrl+Shift+V`. En Windows `Ctrl+V` cubre la
/// matriz validada (incluido Windows Terminal); la selección por app activa
/// se abordará junto con el soporte de perfiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteCombo {
    CtrlV,
    CtrlShiftV,
}

#[derive(Debug, thiserror::Error)]
pub enum DeliveryError {
    #[error("clipboard: {0}")]
    Clipboard(String),
    #[error("input sintético: {0}")]
    Input(String),
}

impl DeliveryError {
    /// Clave i18n para la UI (catálogo err.* en src/i18n).
    pub fn error_key(&self) -> &'static str {
        match self {
            DeliveryError::Clipboard(_) => "err.delivery.clipboard",
            DeliveryError::Input(_) => "err.delivery.insert",
        }
    }
}

/// Deja el texto en el clipboard del sistema (modo `clipboard`).
pub fn copy_to_clipboard(text: &str) -> Result<(), DeliveryError> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| DeliveryError::Clipboard(e.to_string()))?;
    clipboard
        .set_text(text)
        .map_err(|e| DeliveryError::Clipboard(e.to_string()))
}

/// Inserta el texto en la app activa (modo `insert`): guarda el clipboard,
/// escribe el texto, envía el pegado sintético y restaura el clipboard
/// anterior. Si el pegado falla, el texto queda igualmente en el clipboard
/// (degradación a clipboard manual) y se reporta el error.
pub fn insert_text(text: &str, combo: PasteCombo) -> Result<(), DeliveryError> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| DeliveryError::Clipboard(e.to_string()))?;
    let previous = clipboard.get_text().ok();
    clipboard
        .set_text(text)
        .map_err(|e| DeliveryError::Clipboard(e.to_string()))?;

    // Si el pegado falla, no restauramos: dejamos nuestro texto en el
    // clipboard para que el usuario pueda pegar a mano.
    synthetic_paste(combo)?;

    if let Some(prev) = previous {
        thread::sleep(RESTORE_DELAY);
        let _ = clipboard.set_text(prev);
    }
    Ok(())
}

/// Emite la combinación de pegado con `enigo`. Patrón (Ctrl abajo → V
/// abajo/arriba → Ctrl arriba) validado por el spike R3 en Windows.
fn synthetic_paste(combo: PasteCombo) -> Result<(), DeliveryError> {
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| DeliveryError::Input(e.to_string()))?;
    let map = |e: enigo::InputError| DeliveryError::Input(e.to_string());
    let shift = matches!(combo, PasteCombo::CtrlShiftV);

    enigo.key(Key::Control, Press).map_err(map)?;
    if shift {
        enigo.key(Key::Shift, Press).map_err(map)?;
    }
    enigo.key(Key::Unicode('v'), Press).map_err(map)?;
    enigo.key(Key::Unicode('v'), Release).map_err(map)?;
    if shift {
        enigo.key(Key::Shift, Release).map_err(map)?;
    }
    enigo.key(Key::Control, Release).map_err(map)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_mode_serializa_en_minuscula() {
        // Forma parte del contrato IPC (events.rs la reusa) — no cambiar.
        assert_eq!(
            serde_json::to_string(&DeliveryMode::Insert).unwrap(),
            "\"insert\""
        );
        assert_eq!(
            serde_json::to_string(&DeliveryMode::Clipboard).unwrap(),
            "\"clipboard\""
        );
    }

    #[test]
    fn error_keys() {
        assert_eq!(
            DeliveryError::Clipboard("x".into()).error_key(),
            "err.delivery.clipboard"
        );
        assert_eq!(
            DeliveryError::Input("x".into()).error_key(),
            "err.delivery.insert"
        );
    }
}
