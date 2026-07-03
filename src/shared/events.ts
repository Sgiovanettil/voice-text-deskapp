// Espejo manual de src-tauri/src/core/events.rs (catálogo v1, ARCHITECTURE.md
// §2). ES el contrato IPC. Regla de ADR-0009: se pueden añadir eventos y
// campos; nunca renombrar ni cambiar la semántica de los existentes — si el
// catálogo crece mucho post-MVP, evaluar generación automática (ts-rs/specta)
// en vez de mantener este archivo a mano.

export type OverlayMode = "listening";
export type DeliveryMode = "insert" | "clipboard";
export type OverlayOutcome = "ok" | "error" | "cancelled";

export type DomainEvent =
  | { event: "HotkeyPressed"; payload: { timestamp: number } }
  | { event: "HotkeyReleased"; payload: { timestamp: number } }
  | { event: "OverlayOpened"; payload: { mode: OverlayMode } }
  | { event: "RecordingStarted"; payload: { deviceId: string; sampleRate: number } }
  | { event: "RecordingStopped"; payload: { durationMs: number; samples: number } }
  | { event: "RecordingFailed"; payload: { errorKey: string; detail: string } }
  | { event: "TranscriptionStarted"; payload: { providerId: string; model: string } }
  | {
      event: "TranscriptionCompleted";
      payload: { text: string; latencyMs: number; providerId: string };
    }
  | {
      event: "TranscriptionFailed";
      payload: { errorKey: string; retryable: boolean; detail: string };
    }
  | { event: "TextDeliveryStarted"; payload: { mode: DeliveryMode } }
  | { event: "TextDeliveryCompleted"; payload: { mode: DeliveryMode; chars: number } }
  | { event: "TextDeliveryFailed"; payload: { errorKey: string; fallbackUsed: boolean } }
  | { event: "OverlayClosed"; payload: { outcome: OverlayOutcome } }
  | { event: "ConfigChanged"; payload: { changedKeys: string[] } };
