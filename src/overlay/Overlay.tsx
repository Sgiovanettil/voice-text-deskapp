import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";

import type { DomainEvent } from "../shared/events";

type Phase = "listening" | "transcribing" | "delivering" | "done" | "error";

interface OverlayState {
  phase: Phase;
  message: string;
}

function Overlay() {
  const { t } = useTranslation();
  const [state, setState] = useState<OverlayState>({
    phase: "listening",
    message: t("overlay.listening"),
  });

  useEffect(() => {
    const unlisten = listen<DomainEvent>("domain-event", ({ payload: ev }) => {
      switch (ev.event) {
        case "overlayOpened":
        case "recordingStarted":
          setState({ phase: "listening", message: t("overlay.listening") });
          break;
        case "recordingStopped":
        case "transcriptionStarted":
          setState({ phase: "transcribing", message: t("overlay.transcribing") });
          break;
        case "textDeliveryStarted":
          setState({ phase: "delivering", message: t("overlay.delivering") });
          break;
        case "textDeliveryCompleted":
          setState({ phase: "done", message: t("overlay.done") });
          break;
        case "recordingFailed":
        case "transcriptionFailed":
        case "textDeliveryFailed":
          setState({ phase: "error", message: t(ev.payload.errorKey) });
          break;
        default:
          break;
      }
    });
    return () => {
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, [t]);

  return (
    <div className={`overlay-card overlay-${state.phase}`}>
      <span className="overlay-indicator" aria-hidden="true" />
      <span className="overlay-message">{state.message}</span>
    </div>
  );
}

export default Overlay;
