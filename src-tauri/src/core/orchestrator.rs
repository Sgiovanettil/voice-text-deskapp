//! Orquestador del ciclo de dictado (ARCHITECTURE §4.1): corre en su propio
//! hilo, es dueño de la máquina de estados y del `Recorder`, y ejecuta los
//! `Command` que la máquina decide. Los módulos le hablan por canal de
//! eventos de dominio; él reenvía cada evento al frontend (`domain-event`) y
//! mantiene el espejo de estado consultable de `AppState`.

use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use crate::audio::{AudioData, Recorder, VadConfig};
use crate::core::events::{DomainEvent, OverlayMode, OverlayOutcome};
use crate::core::state_machine::{ActivationMode, Command, CoreState, DictationMode, StateMachine};
use crate::delivery::DeliveryMode;
use crate::ipc::commands::AppState;
use crate::speech::TranscribeOptions;

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
                // Los fallos suben a warn para que resalten en el log; el resto
                // queda en info (traza del ciclo).
                if ev.is_failure() {
                    tracing::warn!(event = ?ev, state = ?sm.state(), "evento de fallo del ciclo");
                } else {
                    tracing::info!(event = ?ev, state = ?sm.state(), "evento de dominio");
                }
                // Contrato IPC: todo evento de dominio llega al frontend 1:1.
                let _ = app.emit("domain-event", ev);
            }

            // Los modos vigentes (activación y dictado) se refrescan desde
            // settings antes de decidir: cambiarlos en la UI aplica al ciclo
            // siguiente (o a la etapa siguiente del actual) sin reiniciar.
            if let Some(state) = app.try_state::<AppState>() {
                let (mode, dictation) = {
                    let settings = state.settings.lock().expect("settings lock");
                    (
                        settings.general.activation_mode.clone(),
                        settings.general.dictation_mode.clone(),
                    )
                };
                sm.set_mode(ActivationMode::from_setting(&mode));
                sm.set_dictation_mode(DictationMode::from_setting(&dictation));
            }

            let prev_state = sm.state();
            let command = match &event {
                Some(ev) => sm.handle(ev, now),
                None => sm.tick(now),
            };
            let new_state = sm.state();

            // Espejo consultable para get_app_state.
            if let Some(state) = app.try_state::<AppState>() {
                *state.core_state.lock().expect("core state lock") = sm.state();
            }

            // Eventos de ciclo del overlay ligados a las transiciones del core
            // (ARCHITECTURE §3). Desde v1.x el overlay es residente (lo crea
            // lib.rs al arranque y nunca se oculta): OverlayOpened/Closed ya
            // no muestran/ocultan la ventana, solo marcan el ciclo — el
            // frontend los usa para pasar de "en espera" a activo y volver.
            if prev_state == CoreState::Idle && new_state == CoreState::Recording {
                let _ = app.emit(
                    "domain-event",
                    &DomainEvent::OverlayOpened {
                        mode: OverlayMode::Listening,
                    },
                );
            }
            if prev_state != CoreState::Idle && new_state == CoreState::Idle {
                let outcome = if prev_state == CoreState::Error {
                    OverlayOutcome::Error
                } else {
                    OverlayOutcome::Ok
                };
                let _ = app.emit("domain-event", &DomainEvent::OverlayClosed { outcome });
            }

            match command {
                Command::StartRecording => {
                    // Telemetría de UI (no evento de dominio): el nivel del
                    // micrófono alimenta la onda del overlay a ~30 Hz. Va por
                    // canal propio para no pasar por la máquina ni el log.
                    let level_app = app.clone();
                    let on_level: crate::audio::LevelCallback = Box::new(move |level| {
                        let _ = level_app.emit("audio-level", level);
                    });
                    // En modo toggle se arma el corte por VAD (ADR-0011): el
                    // silencio sostenido entra al core como SilenceDetected y
                    // el estado hablando/en-silencio va al overlay como
                    // telemetría (canal "vad-speaking").
                    let vad = app.try_state::<AppState>().and_then(|state| {
                        let settings = state.settings.lock().expect("settings lock");
                        if settings.general.activation_mode != "toggle" {
                            return None;
                        }
                        let silence_tx = self_tx.clone();
                        let speaking_app = app.clone();
                        Some(VadConfig {
                            threshold: settings.vad.threshold,
                            silence_hangover_ms: settings.vad.silence_hangover_ms,
                            on_silence: Box::new(move |silence_ms| {
                                let _ =
                                    silence_tx.send(DomainEvent::SilenceDetected { silence_ms });
                            }),
                            on_speaking: Box::new(move |speaking| {
                                let _ = speaking_app.emit("vad-speaking", speaking);
                            }),
                        })
                    });
                    // Micrófono elegido (por nombre) desde settings; None =
                    // default del SO.
                    let device_name = app.try_state::<AppState>().and_then(|s| {
                        s.settings
                            .lock()
                            .expect("settings lock")
                            .audio
                            .input_device
                            .clone()
                    });
                    match Recorder::start_with_options(Some(on_level), vad, device_name) {
                        Ok(r) => {
                            // Nombre del micrófono realmente abierto (el resuelto):
                            // observable qué device quedó grabando vs. el pedido.
                            let device_id = r.device_name().to_string();
                            recorder = Some(r);
                            let _ = self_tx.send(DomainEvent::RecordingStarted {
                                device_id,
                                sample_rate: crate::audio::TARGET_SAMPLE_RATE,
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
                Command::StartPostProcessing { text } => {
                    start_post_processing(&app, text, self_tx.clone());
                }
                Command::DeliverText { text } => {
                    deliver_text(&app, &text, &self_tx);
                }
                Command::None => {}
            }
        }
    });
    tx
}

/// Entrega el texto según el `output_mode` de settings: `insert` inserta en la
/// app activa (clipboard + pegado sintético); cualquier otro valor cae al modo
/// `clipboard`. Emite Started/Completed/Failed con el modo efectivo.
fn deliver_text(app: &AppHandle, text: &str, result_tx: &Sender<DomainEvent>) {
    let mode = app
        .try_state::<AppState>()
        .map(|s| {
            s.settings
                .lock()
                .expect("settings lock")
                .general
                .output_mode
                .clone()
        })
        .unwrap_or_else(|| "clipboard".into());
    let mode = if mode == "insert" {
        DeliveryMode::Insert
    } else {
        DeliveryMode::Clipboard
    };

    let _ = result_tx.send(DomainEvent::TextDeliveryStarted { mode });
    let chars = text.chars().count() as u32;

    let result = match mode {
        DeliveryMode::Insert => {
            crate::delivery::insert_text(text, crate::delivery::PasteCombo::CtrlV)
        }
        DeliveryMode::Clipboard => crate::delivery::copy_to_clipboard(text),
    };

    match result {
        Ok(()) => {
            let _ = result_tx.send(DomainEvent::TextDeliveryCompleted { mode, chars });
        }
        Err(e) => {
            let _ = result_tx.send(DomainEvent::TextDeliveryFailed {
                error_key: e.error_key().into(),
                fallback_used: false,
            });
        }
    }
}

/// Efecto asíncrono: pasada LLM del dictado (ADR-0014, modos mejorado y
/// prompt). Degradación segura en TODOS los caminos de fallo (sin key,
/// keyring roto, error del LLM): se emite `PostProcessingFailed` como aviso
/// y luego `PostProcessingCompleted` degradado con el texto literal — un
/// dictado jamás se pierde por el post-procesado.
fn start_post_processing(app: &AppHandle, text: String, result_tx: Sender<DomainEvent>) {
    let state = app.state::<AppState>();
    let (mode, provider_id, model, language) = {
        let settings = state.settings.lock().expect("settings lock");
        let lang = match settings.stt.language.as_str() {
            "auto" => None,
            other => Some(other.to_string()),
        };
        (
            settings.general.dictation_mode.clone(),
            settings.llm.provider.clone(),
            settings.llm.model.clone(),
            lang,
        )
    };

    // Falla antes de llamar al LLM (key ausente o keyring roto): aviso +
    // entrega degradada inmediata del literal.
    let degrade = |error_key: &str, detail: String, tx: &Sender<DomainEvent>, literal: &str| {
        let _ = tx.send(DomainEvent::PostProcessingFailed {
            error_key: error_key.into(),
            retryable: false,
            detail,
        });
        let _ = tx.send(DomainEvent::PostProcessingCompleted {
            text: literal.into(),
            latency_ms: 0,
            degraded: true,
        });
    };

    let api_key = match crate::persistence::get_api_key(&provider_id) {
        Ok(Some(key)) => key,
        Ok(None) => {
            degrade(
                "err.llm.auth",
                "api key no configurada".into(),
                &result_tx,
                &text,
            );
            return;
        }
        Err(e) => {
            degrade("err.keyring.unavailable", e.to_string(), &result_tx, &text);
            return;
        }
    };

    let _ = result_tx.send(DomainEvent::PostProcessingStarted {
        provider_id: provider_id.clone(),
        model: model.clone(),
        mode: mode.clone(),
    });

    tauri::async_runtime::spawn(async move {
        let system = if mode == "prompt" {
            crate::llm::prompts::instruction_system_prompt(language.as_deref())
        } else {
            crate::llm::prompts::improve_system_prompt(language.as_deref())
        };
        let processor = crate::llm::resolve(&provider_id, api_key);
        let request = crate::llm::ChatRequest {
            model,
            system,
            user: text.clone(),
            timeout: PROVIDER_TIMEOUT,
        };
        let started = Instant::now();
        match processor.process(request).await {
            Ok(processed) if !processed.is_empty() => {
                let _ = result_tx.send(DomainEvent::PostProcessingCompleted {
                    text: processed,
                    latency_ms: started.elapsed().as_millis() as u64,
                    degraded: false,
                });
            }
            // Respuesta vacía del LLM: mejor el literal que borrar el dictado.
            Ok(_) => {
                let _ = result_tx.send(DomainEvent::PostProcessingFailed {
                    error_key: "err.llm.provider".into(),
                    retryable: false,
                    detail: "respuesta vacía".into(),
                });
                let _ = result_tx.send(DomainEvent::PostProcessingCompleted {
                    text,
                    latency_ms: started.elapsed().as_millis() as u64,
                    degraded: true,
                });
            }
            Err(e) => {
                let _ = result_tx.send(DomainEvent::PostProcessingFailed {
                    error_key: crate::llm::error_key(&e).into(),
                    retryable: matches!(
                        e,
                        crate::speech::SpeechError::Network
                            | crate::speech::SpeechError::RateLimited
                    ),
                    detail: e.to_string(),
                });
                let _ = result_tx.send(DomainEvent::PostProcessingCompleted {
                    text,
                    latency_ms: started.elapsed().as_millis() as u64,
                    degraded: true,
                });
            }
        }
    });
}

/// Efecto asíncrono: transcripción vía provider. El resultado vuelve al
/// orquestador como evento — nunca toca la máquina de estados directamente.
fn start_transcription(app: &AppHandle, audio: AudioData, result_tx: Sender<DomainEvent>) {
    let state = app.state::<AppState>();
    let (provider_id, model, language, rate_per_min) = {
        let settings = state.settings.lock().expect("settings lock");
        let lang = match settings.stt.language.as_str() {
            "auto" => None,
            other => Some(other.to_string()),
        };
        // Tarifa vigente al momento del dictado: el gasto se acumula con ella
        // (ADR-0015) — editar una tarifa después no reescribe el histórico.
        let rate =
            crate::usage::effective_rate(&settings, &settings.stt.provider, &settings.stt.model);
        (
            settings.stt.provider.clone(),
            settings.stt.model.clone(),
            lang,
            rate,
        )
    };
    let config_dir = state.config_dir.clone();

    let api_key = match crate::persistence::get_api_key(&provider_id) {
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
        provider_id: provider_id.clone(),
        model: model.clone(),
    });

    tauri::async_runtime::spawn(async move {
        let audio_seconds = audio.duration_ms() as f64 / 1000.0;
        let provider = crate::providers::resolve(&provider_id, api_key);
        let opts = TranscribeOptions {
            language,
            model: model.clone(),
            timeout: PROVIDER_TIMEOUT,
        };
        let event = match provider.transcribe(audio, opts).await {
            Ok(transcript) => {
                // Ledger de gastos (ADR-0015): best-effort — un fallo al
                // escribir usage.json jamás afecta el dictado.
                if let Err(e) = crate::usage::record(
                    &config_dir,
                    &provider_id,
                    &model,
                    audio_seconds,
                    rate_per_min,
                ) {
                    tracing::warn!(error = %e, "no se pudo registrar el uso en usage.json");
                }
                DomainEvent::TranscriptionCompleted {
                    text: transcript.text,
                    latency_ms: transcript.latency.as_millis() as u64,
                    provider_id: provider_id.clone(),
                }
            }
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
