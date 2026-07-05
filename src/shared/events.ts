// Espejo manual de src-tauri/src/core/events.rs (catálogo v1, ARCHITECTURE.md
// §2). ES el contrato IPC. Regla de ADR-0009: se pueden añadir eventos y
// campos; nunca renombrar ni cambiar la semántica de los existentes — si el
// catálogo crece mucho post-MVP, evaluar generación automática (ts-rs/specta)
// en vez de mantener este archivo a mano.

export type OverlayMode = "listening";
export type DeliveryMode = "insert" | "clipboard";
export type OverlayOutcome = "ok" | "error" | "cancelled";

export type DomainEvent =
  | { event: "hotkeyPressed"; payload: { timestamp: number } }
  | { event: "hotkeyReleased"; payload: { timestamp: number } }
  | { event: "overlayOpened"; payload: { mode: OverlayMode } }
  | { event: "recordingStarted"; payload: { deviceId: string; sampleRate: number } }
  | { event: "recordingStopped"; payload: { durationMs: number; samples: number } }
  | { event: "recordingFailed"; payload: { errorKey: string; detail: string } }
  | { event: "transcriptionStarted"; payload: { providerId: string; model: string } }
  | {
      event: "transcriptionCompleted";
      payload: { text: string; latencyMs: number; providerId: string };
    }
  | {
      event: "transcriptionFailed";
      payload: { errorKey: string; retryable: boolean; detail: string };
    }
  | { event: "textDeliveryStarted"; payload: { mode: DeliveryMode } }
  | { event: "textDeliveryCompleted"; payload: { mode: DeliveryMode; chars: number } }
  | { event: "textDeliveryFailed"; payload: { errorKey: string; fallbackUsed: boolean } }
  | { event: "overlayClosed"; payload: { outcome: OverlayOutcome } }
  | { event: "configChanged"; payload: { changedKeys: string[] } }
  | {
      event: "updateAvailable";
      payload: { version: string; notes: string; pubDate: string | null };
    }
  | {
      event: "updateDownloadProgress";
      payload: { downloaded: number; contentLength: number | null };
    }
  | { event: "updateFailed"; payload: { errorKey: string; detail: string } };

// Info de una actualización disponible (retorno de `check_for_update`).
export type UpdateInfo = { version: string; notes: string; pubDate: string | null };
