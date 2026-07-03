//! Orquestador del ciclo de dictado (ARCHITECTURE §4.1): corre en su propio
//! hilo, es dueño de la máquina de estados y del `Recorder`, y ejecuta los
//! `Command` que la máquina decide. Los módulos le hablan por canal de
//! eventos de dominio; él reenvía cada evento al frontend (`domain-event`) y
//! mantiene el espejo de estado consultable de `AppState`.

use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::audio::{AudioData, Recorder};
use crate::core::events::DomainEvent;
use crate::core::state_machine::{Command, StateMachine};
use crate::delivery::DeliveryMode;
use crate::ipc::commands::AppState;
use crate::speech::{SpeechProvider, TranscribeOptions};

/// Intervalo del tick de timeouts (§3: corte de 120 s, reset de Error ~3 s).
const TICK: Duration = Duration::from_millis(250);
/// Timeout del proveedor STT (PRD §17.3).
const PROVIDER_TIMEOUT: Duration = Duration::from_secs(30);

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Lanza el hilo del orquestador y devuelve el canal de entrada de eventos
/// (lo usan hotkeys y los propios efectos asíncronos del orquestador).
pub fn spawn(app: AppHandle) -> Sender<DomainEvent> {
    let (tx, rx) = channel::<DomainEvent>();
    let self_tx = tx.clone();
    std::thread::spawn(move || {
        let mut sm = StateMachine::new();
        let mut recorder: Option<Recorder> = None;
        let mut captured: Option<AudioData> = None;

        loop {
            let event = match rx.recv_timeout(TICK) {
                Ok(ev) => Some(ev),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => break,
            };
            let now = now_ms();

            if let Some(ev) = &event {
                // Contrato IPC: todo evento de dominio llega al frontend 1:1.
                let _ = app.emit("domain-event", ev);
            }

            let command = match &event {
                Some(ev) => sm.handle(ev, now),
                None => sm.tick(now),
            };

            // Espejo consultable para get_app_state.
            if let Some(state) = app.try_state::<AppState>() {
                *state.core_state.lock().expect("core state lock") = sm.state();
            }

            match command {
                Command::StartRecording => match Recorder::start() {
                    Ok(r) => {
                        recorder = Some(r);
                        let _ = self_tx.send(DomainEvent::RecordingStarted {
                            device_id: "default".into(),
                            sample_rate: crate::audio::TARGET_SAMPLE_RATE,
                        });
                    }
                    Err(e) => {
                        let _ = self_tx.send(DomainEvent::RecordingFailed {
                            error_key: e.error_key().into(),
                            detail: e.to_string(),
                        });
                    }
                },
                Command::StopRecording => {
                    if let Some(r) = recorder.take() {
                        match r.stop() {
                            Ok(audio) => {
                                let duration_ms = audio.duration_ms();
                                let samples = audio.samples.len() as u32;
                                captured = Some(audio);
                                let _ = self_tx.send(DomainEvent::RecordingStopped {
                                    duration_ms,
                                    samples,
                                });
                            }
                            Err(e) => {
                                let _ = self_tx.send(DomainEvent::RecordingFailed {
                                    error_key: e.error_key().into(),
                                    detail: e.to_string(),
                                });
                            }
                        }
                    }
                }
                Command::StartTranscription => {
                    if let Some(audio) = captured.take() {
                        start_transcription(&app, audio, self_tx.clone());
                    }
                }
                Command::DeliverText { text } => {
                    let _ = self_tx.send(DomainEvent::TextDeliveryStarted {
                        mode: DeliveryMode::Clipboard,
                    });
                    let chars = text.chars().count() as u32;
                    match crate::delivery::copy_to_clipboard(&text) {
                        Ok(()) => {
                            let _ = self_tx.send(DomainEvent::TextDeliveryCompleted {
                                mode: DeliveryMode::Clipboard,
                                chars,
                            });
                        }
                        Err(e) => {
                            let _ = self_tx.send(DomainEvent::TextDeliveryFailed {
                                error_key: e.error_key().into(),
                                fallback_used: false,
                            });
                        }
                    }
                }
                Command::None => {}
            }
        }
    });
    tx
}

/// Efecto asíncrono: transcripción vía provider. El resultado vuelve al
/// orquestador como evento — nunca toca la máquina de estados directamente.
fn start_transcription(app: &AppHandle, audio: AudioData, result_tx: Sender<DomainEvent>) {
    let state = app.state::<AppState>();
    let (model, language) = {
        let settings = state.settings.lock().expect("settings lock");
        let lang = match settings.stt.language.as_str() {
            "auto" => None,
            other => Some(other.to_string()),
        };
        (settings.stt.model.clone(), lang)
    };

    let api_key = match crate::persistence::get_api_key() {
        Ok(Some(key)) => key,
        Ok(None) => {
            let _ = result_tx.send(DomainEvent::TranscriptionFailed {
                error_key: "err.stt.auth".into(),
                retryable: false,
                detail: "api key no configurada".into(),
            });
            return;
        }
        Err(e) => {
            let _ = result_tx.send(DomainEvent::TranscriptionFailed {
                error_key: "err.keyring.unavailable".into(),
                retryable: false,
                detail: e.to_string(),
            });
            return;
        }
    };

    let _ = result_tx.send(DomainEvent::TranscriptionStarted {
        provider_id: "openai".into(),
        model: model.clone(),
    });

    tauri::async_runtime::spawn(async move {
        let provider = crate::providers::openai::OpenAiProvider::new(api_key);
        let opts = TranscribeOptions {
            language,
            model,
            timeout: PROVIDER_TIMEOUT,
        };
        let event = match provider.transcribe(audio, opts).await {
            Ok(transcript) => DomainEvent::TranscriptionCompleted {
                text: transcript.text,
                latency_ms: transcript.latency.as_millis() as u64,
                provider_id: "openai".into(),
            },
            Err(e) => DomainEvent::TranscriptionFailed {
                error_key: match &e {
                    crate::speech::SpeechError::Auth => "err.stt.auth",
                    crate::speech::SpeechError::Network => "err.stt.network",
                    crate::speech::SpeechError::RateLimited => "err.stt.rate",
                    crate::speech::SpeechError::InvalidAudio => "err.stt.audio",
                    crate::speech::SpeechError::Provider { .. } => "err.stt.provider",
                }
                .into(),
                retryable: matches!(
                    e,
                    crate::speech::SpeechError::Network | crate::speech::SpeechError::RateLimited
                ),
                detail: e.to_string(),
            },
        };
        let _ = result_tx.send(event);
    });
}
