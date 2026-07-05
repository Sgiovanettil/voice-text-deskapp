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
    /// Silencio sostenido detectado por el VAD durante la grabación
    /// (ADR-0011): en modo toggle corta el dictado igual que soltar el
    /// hotkey en PTT; en PTT se ignora.
    #[serde(rename_all = "camelCase")]
    SilenceDetected {
        silence_ms: u64,
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
    // Auto-update (ADR-0010). El updater no es una etapa del ciclo de dictado:
    // estos eventos los emite el módulo `updater` directamente al canal
    // `domain-event`, no pasan por el orquestador ni por `is_failure`.
    #[serde(rename_all = "camelCase")]
    UpdateAvailable {
        version: String,
        notes: String,
        pub_date: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    UpdateDownloadProgress {
        downloaded: u64,
        content_length: Option<u64>,
    },
    #[serde(rename_all = "camelCase")]
    UpdateFailed {
        error_key: String,
        detail: String,
    },
}

impl DomainEvent {
    /// `true` si el evento reporta un fallo de una etapa del ciclo. Lo usa el
    /// orquestador para loguear los fallos a `warn` en vez de `info`, de modo
    /// que un ciclo fallido resalte en el archivo de log (observabilidad, M4).
    /// No cambia el contrato IPC: los `*Failed` ya llevan `error_key`/`detail`.
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            DomainEvent::RecordingFailed { .. }
                | DomainEvent::TranscriptionFailed { .. }
                | DomainEvent::TextDeliveryFailed { .. }
        )
    }
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
    fn snapshot_silence_detected() {
        insta::assert_json_snapshot!(DomainEvent::SilenceDetected { silence_ms: 1_200 });
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

    #[test]
    fn snapshot_update_available() {
        insta::assert_json_snapshot!(DomainEvent::UpdateAvailable {
            version: "1.1.0".into(),
            notes: "Correcciones y mejoras.".into(),
            pub_date: Some("2026-07-05T12:00:00Z".into()),
        });
    }

    #[test]
    fn snapshot_update_download_progress() {
        insta::assert_json_snapshot!(DomainEvent::UpdateDownloadProgress {
            downloaded: 1_048_576,
            content_length: Some(8_388_608),
        });
    }

    #[test]
    fn snapshot_update_failed() {
        insta::assert_json_snapshot!(DomainEvent::UpdateFailed {
            error_key: "err.update.network".into(),
            detail: "connection reset".into(),
        });
    }

    #[test]
    fn is_failure_solo_para_los_eventos_de_fallo() {
        assert!(DomainEvent::RecordingFailed {
            error_key: "err.audio.device".into(),
            detail: "x".into(),
        }
        .is_failure());
        assert!(DomainEvent::TranscriptionFailed {
            error_key: "err.stt.network".into(),
            retryable: true,
            detail: "x".into(),
        }
        .is_failure());
        assert!(DomainEvent::TextDeliveryFailed {
            error_key: "err.delivery.blocked".into(),
            fallback_used: false,
        }
        .is_failure());
        // Un evento de éxito no es fallo.
        assert!(!DomainEvent::TextDeliveryCompleted {
            mode: DeliveryMode::Insert,
            chars: 3,
        }
        .is_failure());
    }
}
