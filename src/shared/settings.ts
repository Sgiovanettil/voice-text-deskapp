// Espejo manual de src-tauri/src/config/mod.rs (serde snake_case) y de los
// DTOs del borde IPC (src-tauri/src/ipc/commands.rs, camelCase). Igual que
// events.ts: si cambia el lado Rust, este archivo cambia en el mismo PR.

export interface Settings {
  schema_version: number;
  general: GeneralSettings;
  stt: SttSettings;
  delivery: DeliverySettings;
  vad: VadSettings;
  audio: AudioSettings;
}

export interface AudioSettings {
  // Micrófono por nombre; null = default del SO.
  input_device: string | null;
}

export interface GeneralSettings {
  ui_language: string;
  hotkey: string;
  autostart: boolean;
  start_minimized: boolean;
  output_mode: string;
  activation_mode: string;
  overlay_position: OverlayPos | null;
}

export interface VadSettings {
  threshold: number;
  silence_hangover_ms: number;
}

export interface OverlayPos {
  x: number;
  y: number;
}

export interface SttSettings {
  provider: string;
  model: string;
  language: string;
}

export interface DeliverySettings {
  paste_combo_overrides: Record<string, string>;
  fallback_typing: boolean;
}

export interface ApiKeyStatus {
  isSet: boolean;
  masked: string | null;
}

export interface IpcError {
  code: string;
  errorKey: string;
}

// Espejo de providers::ModelCatalog (serde camelCase): modelos del proveedor
// clasificados en vivo. `chat` queda para el post-procesado LLM (ADR-0014).
export interface ModelCatalog {
  stt: string[];
  chat: string[];
}
