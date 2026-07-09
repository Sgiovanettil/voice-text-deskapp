//! Máquina de estados del ciclo de dictado. Ver docs/2-arquitectura/ARCHITECTURE.md §3.
//!
//! Pura y sin I/O: recibe eventos de dominio (y ticks de reloj) y devuelve el
//! comando que el orquestador debe ejecutar contra audio/speech/delivery. El
//! tiempo entra siempre como parámetro (`now_ms`) para poder testear con
//! reloj simulado.

use crate::core::events::DomainEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreState {
    Idle,
    Recording,
    Transcribing,
    /// Etapa LLM opcional (ADR-0014); solo se pisa con `dictation_mode`
    /// distinto de `literal`.
    PostProcessing,
    Delivering,
    Error,
}

/// Modo de dictado (ADR-0014). `Literal` es el comportamiento original
/// (STT → inserción tal cual); los otros dos agregan la pasada LLM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DictationMode {
    #[default]
    Literal,
    /// Limpieza con prompt fijo versionado: muletillas, puntuación,
    /// redacción — sin cambiar el significado.
    Mejorado,
    /// La voz es una instrucción; el LLM genera el texto a insertar.
    Prompt,
}

impl DictationMode {
    /// Mapea el valor persistido en settings; desconocidos caen a literal
    /// (nunca activar el LLM por accidente).
    pub fn from_setting(value: &str) -> Self {
        match value {
            "mejorado" => Self::Mejorado,
            "prompt" => Self::Prompt,
            _ => Self::Literal,
        }
    }
}

/// Modo de activación del ciclo (ADR-0011). En `Ptt` la grabación vive
/// mientras el hotkey está presionado; en `Toggle` una pulsación inicia y
/// la termina otra pulsación, el corte del VAD o el tope de 120 s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActivationMode {
    #[default]
    Ptt,
    Toggle,
}

impl ActivationMode {
    /// Mapea el valor persistido en settings; desconocidos caen a PTT.
    pub fn from_setting(value: &str) -> Self {
        if value == "toggle" {
            Self::Toggle
        } else {
            Self::Ptt
        }
    }
}

/// Comando que la máquina ordena ejecutar tras procesar un evento. La máquina
/// decide; el orquestador (core) ejecuta el I/O y realimenta con el evento
/// resultante.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Iniciar captura de micrófono (entrando a `Recording`).
    StartRecording,
    /// Detener la captura; el módulo de audio responderá `RecordingStopped`.
    StopRecording,
    /// Enviar el audio capturado al proveedor STT.
    StartTranscription,
    /// Pasar la transcripción por el LLM (ADR-0014, modos mejorado/prompt).
    StartPostProcessing { text: String },
    /// Entregar el texto transcrito (clipboard/inserción).
    DeliverText { text: String },
    /// Nada que hacer (evento ignorado o transición sin efecto).
    None,
}

/// Grabaciones más cortas se descartan como pulsación accidental (§3).
pub const MIN_RECORDING_MS: u64 = 300;
/// Dictado máximo; al alcanzarlo se corta y se transcribe lo grabado (§3).
pub const MAX_RECORDING_MS: u64 = 120_000;
/// Permanencia en `Error` antes de volver a `Idle` (§3, "timeout ~3 s").
pub const ERROR_RESET_MS: u64 = 3_000;

#[derive(Debug)]
pub struct StateMachine {
    state: CoreState,
    /// Instante (ms, reloj del llamador) en que se entró al estado actual.
    entered_at_ms: u64,
    /// Modo vigente al iniciar el ciclo; el orquestador lo refresca desde
    /// settings antes de cada evento (cambiarlo a mitad de ciclo es seguro:
    /// solo altera qué disparadores cortan la grabación).
    mode: ActivationMode,
    /// Modo de dictado vigente (ADR-0014); el orquestador lo refresca junto
    /// con el de activación. Decide si la transcripción pasa por el LLM.
    dictation_mode: DictationMode,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: CoreState::Idle,
            entered_at_ms: 0,
            mode: ActivationMode::Ptt,
            dictation_mode: DictationMode::Literal,
        }
    }

    pub fn state(&self) -> CoreState {
        self.state
    }

    pub fn set_mode(&mut self, mode: ActivationMode) {
        self.mode = mode;
    }

    pub fn set_dictation_mode(&mut self, mode: DictationMode) {
        self.dictation_mode = mode;
    }

    fn transition(&mut self, next: CoreState, now_ms: u64) {
        self.state = next;
        self.entered_at_ms = now_ms;
    }

    /// Procesa un evento de dominio y devuelve el comando a ejecutar.
    /// Eventos que no aplican al estado actual se ignoran (`Command::None`) —
    /// incluye la regla de reentrada: `HotkeyPressed` durante un ciclo activo.
    pub fn handle(&mut self, event: &DomainEvent, now_ms: u64) -> Command {
        match (self.state, event) {
            (CoreState::Idle, DomainEvent::HotkeyPressed { .. }) => {
                self.transition(CoreState::Recording, now_ms);
                Command::StartRecording
            }
            // PTT: soltar el hotkey corta. En toggle la liberación que sigue a
            // la pulsación inicial se ignora — el ciclo sigue grabando.
            (CoreState::Recording, DomainEvent::HotkeyReleased { .. }) => match self.mode {
                ActivationMode::Ptt => Command::StopRecording,
                ActivationMode::Toggle => Command::None,
            },
            // Toggle: la segunda pulsación corta (en PTT la reentrada cae al
            // catch-all y se ignora, regla original).
            (CoreState::Recording, DomainEvent::HotkeyPressed { .. })
                if self.mode == ActivationMode::Toggle =>
            {
                Command::StopRecording
            }
            // Toggle: silencio sostenido del VAD corta igual que el hotkey.
            (CoreState::Recording, DomainEvent::SilenceDetected { .. })
                if self.mode == ActivationMode::Toggle =>
            {
                Command::StopRecording
            }
            (CoreState::Recording, DomainEvent::RecordingStopped { duration_ms, .. }) => {
                if *duration_ms < MIN_RECORDING_MS {
                    // Pulsación accidental: a Idle sin llamar al proveedor.
                    self.transition(CoreState::Idle, now_ms);
                    Command::None
                } else {
                    self.transition(CoreState::Transcribing, now_ms);
                    Command::StartTranscription
                }
            }
            (CoreState::Recording, DomainEvent::RecordingFailed { .. }) => {
                self.transition(CoreState::Error, now_ms);
                Command::None
            }
            (CoreState::Transcribing, DomainEvent::TranscriptionCompleted { text, .. }) => {
                match self.dictation_mode {
                    DictationMode::Literal => {
                        self.transition(CoreState::Delivering, now_ms);
                        Command::DeliverText { text: text.clone() }
                    }
                    // Modos LLM (ADR-0014): pasada extra antes de entregar.
                    DictationMode::Mejorado | DictationMode::Prompt => {
                        self.transition(CoreState::PostProcessing, now_ms);
                        Command::StartPostProcessing { text: text.clone() }
                    }
                }
            }
            (CoreState::Transcribing, DomainEvent::TranscriptionFailed { .. }) => {
                self.transition(CoreState::Error, now_ms);
                Command::None
            }
            // La etapa LLM siempre termina en Completed (ante fallo llega
            // degradado con el texto literal); Failed es solo aviso.
            (CoreState::PostProcessing, DomainEvent::PostProcessingCompleted { text, .. }) => {
                self.transition(CoreState::Delivering, now_ms);
                Command::DeliverText { text: text.clone() }
            }
            (CoreState::PostProcessing, DomainEvent::PostProcessingFailed { .. }) => Command::None,
            (CoreState::Delivering, DomainEvent::TextDeliveryCompleted { .. }) => {
                self.transition(CoreState::Idle, now_ms);
                Command::None
            }
            (CoreState::Delivering, DomainEvent::TextDeliveryFailed { .. }) => {
                self.transition(CoreState::Error, now_ms);
                Command::None
            }
            // Todo lo demás (incluida la reentrada de HotkeyPressed en un
            // ciclo activo) se ignora sin cambiar de estado.
            _ => Command::None,
        }
    }

    /// Tick periódico del orquestador: aplica los límites temporales de §3.
    pub fn tick(&mut self, now_ms: u64) -> Command {
        let elapsed = now_ms.saturating_sub(self.entered_at_ms);
        match self.state {
            CoreState::Recording if elapsed >= MAX_RECORDING_MS => Command::StopRecording,
            CoreState::Error if elapsed >= ERROR_RESET_MS => {
                self.transition(CoreState::Idle, now_ms);
                Command::None
            }
            _ => Command::None,
        }
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hotkey_pressed() -> DomainEvent {
        DomainEvent::HotkeyPressed { timestamp: 0 }
    }

    fn recording_stopped(duration_ms: u64) -> DomainEvent {
        DomainEvent::RecordingStopped {
            duration_ms,
            samples: 1_000,
        }
    }

    fn transcription_completed(text: &str) -> DomainEvent {
        DomainEvent::TranscriptionCompleted {
            text: text.into(),
            latency_ms: 500,
            provider_id: "openai".into(),
        }
    }

    #[test]
    fn ciclo_feliz_completo() {
        let mut sm = StateMachine::new();
        assert_eq!(sm.state(), CoreState::Idle);

        assert_eq!(sm.handle(&hotkey_pressed(), 0), Command::StartRecording);
        assert_eq!(sm.state(), CoreState::Recording);

        assert_eq!(
            sm.handle(&DomainEvent::HotkeyReleased { timestamp: 4_000 }, 4_000),
            Command::StopRecording
        );
        // Sigue grabando hasta que audio confirme el corte.
        assert_eq!(sm.state(), CoreState::Recording);

        assert_eq!(
            sm.handle(&recording_stopped(4_000), 4_050),
            Command::StartTranscription
        );
        assert_eq!(sm.state(), CoreState::Transcribing);

        assert_eq!(
            sm.handle(&transcription_completed("hola mundo"), 5_000),
            Command::DeliverText {
                text: "hola mundo".into()
            }
        );
        assert_eq!(sm.state(), CoreState::Delivering);

        assert_eq!(
            sm.handle(
                &DomainEvent::TextDeliveryCompleted {
                    mode: crate::delivery::DeliveryMode::Clipboard,
                    chars: 10,
                },
                5_100,
            ),
            Command::None
        );
        assert_eq!(sm.state(), CoreState::Idle);
    }

    #[test]
    fn grabacion_corta_se_descarta_sin_transcribir() {
        let mut sm = StateMachine::new();
        sm.handle(&hotkey_pressed(), 0);
        assert_eq!(
            sm.handle(&recording_stopped(MIN_RECORDING_MS - 1), 300),
            Command::None
        );
        assert_eq!(sm.state(), CoreState::Idle);
    }

    #[test]
    fn hotkey_durante_ciclo_activo_se_ignora() {
        let mut sm = StateMachine::new();
        sm.handle(&hotkey_pressed(), 0);
        assert_eq!(sm.handle(&hotkey_pressed(), 100), Command::None);
        assert_eq!(sm.state(), CoreState::Recording);

        sm.handle(&recording_stopped(1_000), 1_000);
        assert_eq!(sm.handle(&hotkey_pressed(), 1_100), Command::None);
        assert_eq!(sm.state(), CoreState::Transcribing);
    }

    #[test]
    fn fallo_de_grabacion_va_a_error_y_vuelve_a_idle_por_timeout() {
        let mut sm = StateMachine::new();
        sm.handle(&hotkey_pressed(), 0);
        sm.handle(
            &DomainEvent::RecordingFailed {
                error_key: "err.audio.device".into(),
                detail: "no device".into(),
            },
            1_000,
        );
        assert_eq!(sm.state(), CoreState::Error);

        // Antes del timeout sigue en Error.
        assert_eq!(sm.tick(1_000 + ERROR_RESET_MS - 1), Command::None);
        assert_eq!(sm.state(), CoreState::Error);

        // Cumplido el timeout vuelve a Idle y acepta un ciclo nuevo.
        sm.tick(1_000 + ERROR_RESET_MS);
        assert_eq!(sm.state(), CoreState::Idle);
        assert_eq!(
            sm.handle(&hotkey_pressed(), 10_000),
            Command::StartRecording
        );
    }

    #[test]
    fn fallo_de_transcripcion_va_a_error() {
        let mut sm = StateMachine::new();
        sm.handle(&hotkey_pressed(), 0);
        sm.handle(&recording_stopped(2_000), 2_000);
        sm.handle(
            &DomainEvent::TranscriptionFailed {
                error_key: "err.stt.network".into(),
                retryable: true,
                detail: "timeout".into(),
            },
            3_000,
        );
        assert_eq!(sm.state(), CoreState::Error);
    }

    #[test]
    fn fallo_de_entrega_va_a_error() {
        let mut sm = StateMachine::new();
        sm.handle(&hotkey_pressed(), 0);
        sm.handle(&recording_stopped(2_000), 2_000);
        sm.handle(&transcription_completed("hola"), 3_000);
        sm.handle(
            &DomainEvent::TextDeliveryFailed {
                error_key: "err.delivery.blocked".into(),
                fallback_used: false,
            },
            3_100,
        );
        assert_eq!(sm.state(), CoreState::Error);
    }

    #[test]
    fn dictado_maximo_corta_la_grabacion() {
        let mut sm = StateMachine::new();
        sm.handle(&hotkey_pressed(), 0);

        assert_eq!(sm.tick(MAX_RECORDING_MS - 1), Command::None);
        assert_eq!(sm.tick(MAX_RECORDING_MS), Command::StopRecording);
        // El estado no cambia hasta que audio confirme RecordingStopped:
        // lo grabado hasta aquí sí se transcribe (corte, no descarte).
        assert_eq!(sm.state(), CoreState::Recording);
        assert_eq!(
            sm.handle(&recording_stopped(MAX_RECORDING_MS), MAX_RECORDING_MS + 50),
            Command::StartTranscription
        );
    }

    #[test]
    fn toggle_ignora_release_y_corta_con_segunda_pulsacion() {
        let mut sm = StateMachine::new();
        sm.set_mode(ActivationMode::Toggle);

        assert_eq!(sm.handle(&hotkey_pressed(), 0), Command::StartRecording);
        // La liberación inmediata tras la pulsación inicial no corta.
        assert_eq!(
            sm.handle(&DomainEvent::HotkeyReleased { timestamp: 120 }, 120),
            Command::None
        );
        assert_eq!(sm.state(), CoreState::Recording);

        // La segunda pulsación sí corta.
        assert_eq!(sm.handle(&hotkey_pressed(), 5_000), Command::StopRecording);
        assert_eq!(sm.state(), CoreState::Recording);
        assert_eq!(
            sm.handle(&recording_stopped(5_000), 5_050),
            Command::StartTranscription
        );
    }

    #[test]
    fn toggle_corta_por_silencio_del_vad() {
        let mut sm = StateMachine::new();
        sm.set_mode(ActivationMode::Toggle);
        sm.handle(&hotkey_pressed(), 0);

        assert_eq!(
            sm.handle(&DomainEvent::SilenceDetected { silence_ms: 1_200 }, 6_000),
            Command::StopRecording
        );
    }

    #[test]
    fn ptt_ignora_silencio_del_vad() {
        let mut sm = StateMachine::new();
        sm.handle(&hotkey_pressed(), 0);
        assert_eq!(
            sm.handle(&DomainEvent::SilenceDetected { silence_ms: 1_200 }, 6_000),
            Command::None
        );
        assert_eq!(sm.state(), CoreState::Recording);
    }

    #[test]
    fn from_setting_mapea_toggle_y_cae_a_ptt() {
        assert_eq!(
            ActivationMode::from_setting("toggle"),
            ActivationMode::Toggle
        );
        assert_eq!(ActivationMode::from_setting("ptt"), ActivationMode::Ptt);
        assert_eq!(ActivationMode::from_setting("otro"), ActivationMode::Ptt);
    }

    #[test]
    fn modo_mejorado_pasa_por_post_procesado() {
        let mut sm = StateMachine::new();
        sm.set_dictation_mode(DictationMode::Mejorado);
        sm.handle(&hotkey_pressed(), 0);
        sm.handle(&recording_stopped(2_000), 2_000);

        assert_eq!(
            sm.handle(&transcription_completed("eh… hola mundo"), 3_000),
            Command::StartPostProcessing {
                text: "eh… hola mundo".into()
            }
        );
        assert_eq!(sm.state(), CoreState::PostProcessing);

        assert_eq!(
            sm.handle(
                &DomainEvent::PostProcessingCompleted {
                    text: "Hola, mundo.".into(),
                    latency_ms: 640,
                    degraded: false,
                },
                4_000,
            ),
            Command::DeliverText {
                text: "Hola, mundo.".into()
            }
        );
        assert_eq!(sm.state(), CoreState::Delivering);
    }

    #[test]
    fn fallo_del_llm_no_corta_el_ciclo_y_entrega_el_degradado() {
        let mut sm = StateMachine::new();
        sm.set_dictation_mode(DictationMode::Prompt);
        sm.handle(&hotkey_pressed(), 0);
        sm.handle(&recording_stopped(2_000), 2_000);
        sm.handle(&transcription_completed("redacta un saludo"), 3_000);
        assert_eq!(sm.state(), CoreState::PostProcessing);

        // El aviso de fallo no transiciona: la degradación llega después
        // como PostProcessingCompleted con el texto literal.
        assert_eq!(
            sm.handle(
                &DomainEvent::PostProcessingFailed {
                    error_key: "err.llm.network".into(),
                    retryable: true,
                    detail: "timeout".into(),
                },
                4_000,
            ),
            Command::None
        );
        assert_eq!(sm.state(), CoreState::PostProcessing);

        assert_eq!(
            sm.handle(
                &DomainEvent::PostProcessingCompleted {
                    text: "redacta un saludo".into(),
                    latency_ms: 30_000,
                    degraded: true,
                },
                34_000,
            ),
            Command::DeliverText {
                text: "redacta un saludo".into()
            }
        );
        assert_eq!(sm.state(), CoreState::Delivering);
    }

    #[test]
    fn modo_literal_no_pasa_por_el_llm() {
        let mut sm = StateMachine::new();
        sm.set_dictation_mode(DictationMode::Literal);
        sm.handle(&hotkey_pressed(), 0);
        sm.handle(&recording_stopped(2_000), 2_000);
        assert_eq!(
            sm.handle(&transcription_completed("hola"), 3_000),
            Command::DeliverText {
                text: "hola".into()
            }
        );
        assert_eq!(sm.state(), CoreState::Delivering);
    }

    #[test]
    fn dictation_mode_from_setting_cae_a_literal() {
        assert_eq!(
            DictationMode::from_setting("mejorado"),
            DictationMode::Mejorado
        );
        assert_eq!(DictationMode::from_setting("prompt"), DictationMode::Prompt);
        assert_eq!(
            DictationMode::from_setting("literal"),
            DictationMode::Literal
        );
        assert_eq!(DictationMode::from_setting("otro"), DictationMode::Literal);
    }

    #[test]
    fn eventos_fuera_de_estado_se_ignoran() {
        let mut sm = StateMachine::new();
        // En Idle, nada de esto aplica.
        assert_eq!(sm.handle(&recording_stopped(5_000), 0), Command::None);
        assert_eq!(sm.handle(&transcription_completed("x"), 0), Command::None);
        assert_eq!(sm.state(), CoreState::Idle);
    }
}
