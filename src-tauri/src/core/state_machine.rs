//! Máquina de estados del ciclo de dictado. Ver docs/ARCHITECTURE.md §3.
//! Transiciones dirigidas por eventos: implementación en M1.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreState {
    Idle,
    Recording,
    Transcribing,
    Delivering,
    Error,
}

// TODO(M1): transiciones dirigidas por eventos de dominio (core/events.rs, PR5),
// timeouts de seguridad por estado, reentradas ignoradas durante ciclo activo.
