import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { DomainEvent } from "../shared/events";
import type { Settings } from "../shared/settings";
import "./Overlay.css";

type Phase = "idle" | "listening" | "transcribing" | "delivering" | "done" | "error";

/** Cuánto se muestra "Listo"/error antes de volver a reposo. */
const DWELL_MS = 900;

interface OverlayState {
  phase: Phase;
  message: string;
}

/** Amplitud sintética por fase: racimo central con envolvente gaussiana y
 *  colas que desaparecen. En v1.x-PR3 la fase `listening` pasará a usar el
 *  nivel real del micrófono. */
function amp(u: number, i: number, t: number, phase: Phase, doneAt: number): number {
  const env = Math.pow(Math.exp(-Math.pow((u - 0.5) / 0.17, 2)), 1.35);
  switch (phase) {
    case "listening": {
      const s1 = Math.sin(t * 0.13 + i * 0.9);
      const s2 = Math.sin(t * 0.31 + i * 2.3);
      const s3 = Math.sin(t * 0.07 + i * 0.31);
      return (0.2 + 0.8 * Math.abs(s1 * 0.6 + s2 * 0.35 + s3 * 0.4)) * env;
    }
    case "transcribing":
    case "delivering": {
      const p = Math.sin(t * 0.2 - u * 9);
      return (0.25 + 0.75 * Math.abs(p)) * env;
    }
    case "done": {
      const decay = Math.max(0, 1 - (t - doneAt) / 70);
      return env * (0.14 + 0.86 * decay * Math.abs(Math.sin(u * 22 + t * 0.05)));
    }
    case "error":
      return (0.08 + 0.15 * Math.abs(Math.sin(t * 0.5 + i * 2.7))) * (0.3 + 0.7 * env);
    case "idle":
      return (0.06 + 0.09 * Math.abs(Math.sin(t * 0.04 + i * 0.5))) * env;
  }
}

function Overlay() {
  const { t } = useTranslation();
  const [state, setState] = useState<OverlayState>({
    phase: "idle",
    message: t("overlay.idle"),
  });
  const [provider, setProvider] = useState<{ name: string; model: string } | null>(null);

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const phaseRef = useRef<Phase>("idle");
  const doneAtRef = useRef(0);
  // Nivel real del micrófono ("audio-level", ~30 Hz): valor actual, historia
  // que se desplaza bajo la onda y timestamp del último dato (para caer a la
  // animación sintética si el backend no reporta niveles).
  const levelRef = useRef(0);
  const levelHistoryRef = useRef<number[]>([]);
  const lastLevelAtRef = useRef(0);

  useEffect(() => {
    phaseRef.current = state.phase;
  }, [state.phase]);

  useEffect(() => {
    const unlisten = listen<number>("audio-level", ({ payload }) => {
      levelRef.current = payload;
      lastLevelAtRef.current = performance.now();
    });
    return () => {
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, []);

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then((s) => setProvider({ name: s.stt.provider, model: s.stt.model }))
      .catch(() => {});
  }, []);

  useEffect(() => {
    // El overlay es residente: tras cerrar el ciclo (OverlayClosed) se
    // muestra el desenlace un instante y se vuelve a reposo, salvo que otro
    // ciclo haya arrancado entretanto (el timer se cancela con cada evento).
    let dwell: ReturnType<typeof setTimeout> | undefined;
    const unlisten = listen<DomainEvent>("domain-event", ({ payload: ev }) => {
      clearTimeout(dwell);
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
        case "overlayClosed":
          dwell = setTimeout(() => {
            setState({ phase: "idle", message: t("overlay.idle") });
          }, DWELL_MS);
          break;
        default:
          break;
      }
    });
    return () => {
      clearTimeout(dwell);
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, [t]);

  // Onda: doble pasada (baño de glow difuso + trazo fino con picos quemados).
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;

    const resize = () => {
      const r = canvas.getBoundingClientRect();
      const dpr = Math.min(devicePixelRatio || 1, 2);
      canvas.width = Math.max(1, r.width * dpr);
      canvas.height = Math.max(1, r.height * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    };
    const ro = new ResizeObserver(resize);
    ro.observe(canvas);
    resize();

    let time = 0;
    let raf = 0;
    let lastPhase: Phase = phaseRef.current;

    const draw = () => {
      time += reduce ? 0.15 : 1;
      const phase = phaseRef.current;
      if (phase === "done" && lastPhase !== "done") doneAtRef.current = time;
      lastPhase = phase;

      const dpr = Math.min(devicePixelRatio || 1, 2);
      const w = canvas.width / dpr;
      const h = canvas.height / dpr;
      ctx.clearRect(0, 0, w, h);
      const col = getComputedStyle(canvas).getPropertyValue("--core").trim() || "#2fe8b4";
      const n = Math.floor(w / 3.5);
      const cy = h / 2;

      // En "listening" con niveles reales frescos (<500 ms), la onda es la
      // historia del micrófono desplazándose bajo la envolvente gaussiana;
      // sin datos frescos se cae a la animación sintética por fase.
      const live = phase === "listening" && performance.now() - lastLevelAtRef.current < 500;
      const history = levelHistoryRef.current;
      if (live) {
        history.push(levelRef.current);
        while (history.length > n) history.shift();
      } else if (history.length > 0) {
        history.length = 0;
      }

      const heights: number[] = [];
      for (let i = 0; i < n; i++) {
        const u = i / (n - 1);
        if (live) {
          const env = Math.pow(Math.exp(-Math.pow((u - 0.5) / 0.17, 2)), 1.35);
          const idx = history.length - n + i;
          const lvl = idx >= 0 ? history[idx] : 0;
          heights.push(Math.min(1, (0.06 + lvl * 1.35) * env));
        } else {
          heights.push(Math.min(1, amp(u, i, time, phase, doneAtRef.current)));
        }
      }
      ctx.shadowBlur = 16;
      ctx.shadowColor = col;
      ctx.fillStyle = col;
      for (let i = 0; i < n; i++) {
        const a = heights[i];
        if (a < 0.05) continue;
        const bh = a * h * 0.95;
        ctx.globalAlpha = a * 0.22;
        ctx.fillRect(i * 3.5, cy - bh / 2, 3, bh);
      }
      ctx.shadowBlur = 6;
      for (let i = 0; i < n; i++) {
        const a = heights[i];
        const bh = Math.max(a > 0.04 ? 1.5 : 0.6, a * h * 0.95);
        ctx.globalAlpha = 0.12 + a * 0.88;
        ctx.fillStyle = a > 0.72 ? "#eafffa" : col;
        ctx.fillRect(i * 3.5 + 0.75, cy - bh / 2, 1.6, bh);
      }
      ctx.globalAlpha = 1;
      ctx.shadowBlur = 0;
      raf = requestAnimationFrame(draw);
    };
    raf = requestAnimationFrame(draw);
    return () => {
      cancelAnimationFrame(raf);
      ro.disconnect();
    };
  }, []);

  return (
    <div className="overlay" data-phase={state.phase}>
      {/* Zona de arrastre nativa: toda la superficie mueve la ventana. */}
      <div className="drag" data-tauri-drag-region aria-hidden="true" />
      <div className="chassis" aria-hidden="true" />
      <div className="socket" aria-hidden="true">
        <div className="core">
          <div className="heart" />
          <div className="neb a" />
          <div className="neb b" />
          <div className="glass" />
        </div>
      </div>
      <div className="screen">
        <div className="topline">
          <span className="label" role="status">
            {state.message}
          </span>
          <span className="bars" aria-hidden="true">
            <i />
            <i />
            <i />
          </span>
        </div>
        <div className="wave" aria-hidden="true">
          <canvas ref={canvasRef} />
        </div>
        <div className="readout">
          {provider !== null && (
            <span className="badge">
              <span className="hex" aria-hidden="true" />
              <span>
                <b>{provider.name}</b>
                {provider.model}
              </span>
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

export default Overlay;
