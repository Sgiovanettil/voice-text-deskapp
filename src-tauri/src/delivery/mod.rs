//! Inserción de texto en la app activa y modo clipboard. Mecanismo según
//! ADR-0005. Ver docs/ARCHITECTURE.md §4.6.
//!
//! M1 implementa el modo clipboard (el texto queda listo para pegar). La
//! inserción con pegado sintético y restauración de clipboard llega en M2
//! (mecanismo ya validado por el spike R3 en Windows). Ningún
//! `#[cfg(target_os)]` fuera de este módulo (ADR-002).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryMode {
    Insert,
    Clipboard,
}

#[derive(Debug, thiserror::Error)]
pub enum DeliveryError {
    #[error("clipboard: {0}")]
    Clipboard(String),
}

impl DeliveryError {
    /// Clave i18n para la UI (catálogo err.* en src/i18n).
    pub fn error_key(&self) -> &'static str {
        match self {
            DeliveryError::Clipboard(_) => "err.delivery.clipboard",
        }
    }
}

/// Deja el texto en el clipboard del sistema. Modo de entrega de M1; en M2
/// pasa a ser la primera mitad del modo insert (clipboard + pegado sintético).
pub fn copy_to_clipboard(text: &str) -> Result<(), DeliveryError> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| DeliveryError::Clipboard(e.to_string()))?;
    clipboard
        .set_text(text)
        .map_err(|e| DeliveryError::Clipboard(e.to_string()))
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
    fn error_key_de_clipboard() {
        let err = DeliveryError::Clipboard("x".into());
        assert_eq!(err.error_key(), "err.delivery.clipboard");
    }
}
