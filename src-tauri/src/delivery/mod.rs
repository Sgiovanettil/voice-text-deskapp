//! Inserción de texto en la app activa y modo clipboard. Mecanismo según
//! ADR-0005. Ver docs/ARCHITECTURE.md §4.6. Implementación: M2.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryMode {
    Insert,
    Clipboard,
}

// TODO(M2): clipboard + pegado sintético con restauración (primario), perfiles
// de pegado por app, fallback de tecleo simulado (enigo). Ningún #[cfg(target_os)]
// fuera de este módulo (ADR-002, regla estructural de ARCHITECTURE.md §1).
