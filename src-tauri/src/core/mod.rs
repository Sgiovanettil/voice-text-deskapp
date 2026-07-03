//! Orquestador: máquina de estados del ciclo de dictado dirigida por eventos
//! de dominio. No conoce proveedores ni SO. Ver docs/ARCHITECTURE.md §3, §4.1.

pub mod events;
pub mod orchestrator;
pub mod overlay;
pub mod state_machine;
