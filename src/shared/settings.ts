// Espejo manual de src-tauri/src/config/mod.rs (serde snake_case) y de los
// DTOs del borde IPC (src-tauri/src/ipc/commands.rs, camelCase). Igual que
// events.ts: si cambia el lado Rust, este archivo cambia en el mismo PR.

export interface Settings {
  schema_version: number;
  general: GeneralSettings;
  stt: SttSettings;
  delivery: DeliverySettings;
}

export interface GeneralSettings {
  ui_language: string;
  hotkey: string;
  autostart: boolean;
  start_minimized: boolean;
  output_mode: string;
  overlay_position: OverlayPos | null;
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
