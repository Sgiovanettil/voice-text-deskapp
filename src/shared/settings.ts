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
  pricing: PricingSettings;
  llm: LlmSettings;
}

// Proveedor y modelo del post-procesado LLM (ADR-0014); solo aplica cuando
// general.dictation_mode no es "literal".
export interface LlmSettings {
  provider: string;
  model: string;
}

// Overrides de tarifas para la estimación de gastos (ADR-0015); los defaults
// viven en el backend (usage::default_rate). Clave "proveedor/modelo".
export interface PricingSettings {
  rates: Record<string, number>;
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
  // "literal" | "mejorado" | "prompt" (ADR-0014); desconocidos = literal.
  dictation_mode: string;
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

// Espejo de usage::UsageEntry / usage::UsageLedger (ADR-0015, snake_case como
// Settings): acumulado de gastos estimados por (proveedor, modelo, mes).
export interface UsageEntry {
  provider: string;
  model: string;
  month: string;
  transcriptions: number;
  audio_seconds: number;
  estimated_cost_usd: number;
}

export interface UsageLedger {
  schema_version: number;
  entries: UsageEntry[];
}
