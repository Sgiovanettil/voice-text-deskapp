//! Catálogo de eventos de dominio v1 (ARCHITECTURE.md §2). Es el contrato
//! público interno y, a la vez, el contrato IPC (espejo manual en
//! `src/shared/events.ts`). Regla de ADR-0009: se pueden añadir campos y
//! eventos nuevos; nunca renombrar ni cambiar la semántica de los existentes
//! — los tests de snapshot de este módulo son la guardia de esa regla.

use serde::{Deserialize, Serialize};

use crate::delivery::DeliveryMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OverlayMode {
    Listening,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OverlayOutcome {
    Ok,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload", rename_all = "camelCase")]
pub enum DomainEvent {
    HotkeyPressed {
        timestamp: i64,
    },
    HotkeyReleased {
        timestamp: i64,
    },
    OverlayOpened {
        mode: OverlayMode,
    },
    #[serde(rename_all = "camelCase")]
    RecordingStarted {
        device_id: String,
        sample_rate: u32,
    },
    #[serde(rename_all = "camelCase")]
    RecordingStopped {
        duration_ms: u64,
        samples: u32,
    },
    #[serde(rename_all = "camelCase")]
    RecordingFailed {
        error_key: String,
        detail: String,
    },
    #[serde(rename_all = "camelCase")]
    TranscriptionStarted {
        provider_id: String,
        model: String,
    },
    #[serde(rename_all = "camelCase")]
    TranscriptionCompleted {
        text: String,
        latency_ms: u64,
        provider_id: String,
    },
    #[serde(rename_all = "camelCase")]
    TranscriptionFailed {
        error_key: String,
        retryable: bool,
        detail: String,
    },
    TextDeliveryStarted {
        mode: DeliveryMode,
    },
    TextDeliveryCompleted {
        mode: DeliveryMode,
        chars: u32,
    },
    #[serde(rename_all = "camelCase")]
    TextDeliveryFailed {
        error_key: String,
        fallback_used: bool,
    },
    OverlayClosed {
        outcome: OverlayOutcome,
    },
    #[serde(rename_all = "camelCase")]
    ConfigChanged {
        changed_keys: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // Un snapshot por variante: cualquier rename o cambio de forma del JSON
    // rompe el test — esa es la guardia de "nunca renombrar" de ADR-0009.

    #[test]
    fn snapshot_hotkey_pressed() {
        insta::assert_json_snapshot!(DomainEvent::HotkeyPressed {
            timestamp: 1_700_000_000
        });
    }

    #[test]
    fn snapshot_hotkey_released() {
        insta::assert_json_snapshot!(DomainEvent::HotkeyReleased {
            timestamp: 1_700_000_001
        });
    }

    #[test]
    fn snapshot_overlay_opened() {
        insta::assert_json_snapshot!(DomainEvent::OverlayOpened {
            mode: OverlayMode::Listening
        });
    }

    #[test]
    fn snapshot_recording_started() {
        insta::assert_json_snapshot!(DomainEvent::RecordingStarted {
            device_id: "default".into(),
            sample_rate: 16_000,
        });
    }

    #[test]
    fn snapshot_recording_stopped() {
        insta::assert_json_snapshot!(DomainEvent::RecordingStopped {
            duration_ms: 4_200,
            samples: 67_200,
        });
    }

    #[test]
    fn snapshot_recording_failed() {
        insta::assert_json_snapshot!(DomainEvent::RecordingFailed {
            error_key: "err.audio.device".into(),
            detail: "no default input device".into(),
        });
    }

    #[test]
    fn snapshot_transcription_started() {
        insta::assert_json_snapshot!(DomainEvent::TranscriptionStarted {
            provider_id: "openai".into(),
            model: "gpt-4o-mini-transcribe".into(),
        });
    }

    #[test]
    fn snapshot_transcription_completed() {
        insta::assert_json_snapshot!(DomainEvent::TranscriptionCompleted {
            text: "hola mundo".into(),
            latency_ms: 850,
            provider_id: "openai".into(),
        });
    }

    #[test]
    fn snapshot_transcription_failed() {
        insta::assert_json_snapshot!(DomainEvent::TranscriptionFailed {
            error_key: "err.stt.network".into(),
            retryable: true,
            detail: "timeout after 30s".into(),
        });
    }

    #[test]
    fn snapshot_text_delivery_started() {
        insta::assert_json_snapshot!(DomainEvent::TextDeliveryStarted {
            mode: DeliveryMode::Insert
        });
    }

    #[test]
    fn snapshot_text_delivery_completed() {
        insta::assert_json_snapshot!(DomainEvent::TextDeliveryCompleted {
            mode: DeliveryMode::Clipboard,
            chars: 42,
        });
    }

    #[test]
    fn snapshot_text_delivery_failed() {
        insta::assert_json_snapshot!(DomainEvent::TextDeliveryFailed {
            error_key: "err.delivery.blocked".into(),
            fallback_used: false,
        });
    }

    #[test]
    fn snapshot_overlay_closed() {
        insta::assert_json_snapshot!(DomainEvent::OverlayClosed {
            outcome: OverlayOutcome::Ok
        });
    }

    #[test]
    fn snapshot_config_changed() {
        insta::assert_json_snapshot!(DomainEvent::ConfigChanged {
            changed_keys: vec!["general.hotkey".into()],
        });
    }
}
