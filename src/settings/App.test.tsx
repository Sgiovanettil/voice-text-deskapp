import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

// jsdom no tiene el runtime de Tauri: se simula el borde IPC por comando.
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));
vi.mock("@tauri-apps/api/app", () => ({
  getVersion: vi.fn(() => Promise.resolve("1.0.0")),
}));
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    switch (cmd) {
      case "get_settings":
        return Promise.resolve({
          schema_version: 1,
          general: {
            ui_language: "es",
            hotkey: "Ctrl+Super+Space",
            autostart: false,
            start_minimized: true,
            output_mode: "insert",
            overlay_position: null,
          },
          stt: { provider: "openai", model: "gpt-4o-mini-transcribe", language: "auto" },
          delivery: { paste_combo_overrides: {}, fallback_typing: false },
          vad: { threshold: 0.5, silence_hangover_ms: 1200 },
          audio: { input_device: null },
        });
      case "get_api_key_status":
        return Promise.resolve({ isSet: true, masked: "…1234" });
      case "list_input_devices":
        return Promise.resolve(["Micrófono interno", "USB Mic"]);
      default:
        return Promise.reject(new Error(`comando no mockeado: ${cmd}`));
    }
  }),
}));

import "../i18n";
import App from "./App";

describe("Settings App", () => {
  it("muestra la navegación por secciones (es, idioma por defecto)", () => {
    render(<App />);
    for (const name of ["General", "Reconocimiento", "Atajos", "Acerca de"]) {
      expect(screen.getByRole("button", { name })).toBeInTheDocument();
    }
  });

  it("muestra el hotkey cargado al entrar a Atajos", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Atajos" }));
    await waitFor(() => {
      expect(screen.getByDisplayValue("Ctrl+Super+Space")).toBeInTheDocument();
    });
  });

  it("muestra la API key enmascarada en Reconocimiento", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Reconocimiento" }));
    await waitFor(() => {
      expect(screen.getByText(/…1234/)).toBeInTheDocument();
    });
  });
});
