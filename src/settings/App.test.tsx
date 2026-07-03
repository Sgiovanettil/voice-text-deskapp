import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

// jsdom no tiene el runtime de Tauri: se simula el borde IPC por comando.
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
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
          },
          stt: { provider: "openai", model: "gpt-4o-mini-transcribe", language: "auto" },
          delivery: { paste_combo_overrides: {}, fallback_typing: false },
        });
      case "get_api_key_status":
        return Promise.resolve({ isSet: true, masked: "…1234" });
      default:
        return Promise.reject(new Error(`comando no mockeado: ${cmd}`));
    }
  }),
}));

import "../i18n";
import App from "./App";

describe("Settings App", () => {
  it("renders the title from the i18n catalog (es, idioma por defecto)", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "VoiceText" })).toBeInTheDocument();
  });

  it("muestra el hotkey cargado desde get_settings", async () => {
    render(<App />);
    await waitFor(() => {
      expect(screen.getByDisplayValue("Ctrl+Super+Space")).toBeInTheDocument();
    });
  });

  it("muestra la API key como configurada y enmascarada", async () => {
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText(/…1234/)).toBeInTheDocument();
    });
  });
});
