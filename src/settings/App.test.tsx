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
            activation_mode: "toggle",
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
      case "set_settings":
      case "start_mic_test":
      case "stop_mic_test":
        return Promise.resolve();
      default:
        return Promise.reject(new Error(`comando no mockeado: ${cmd}`));
    }
  }),
}));

import { invoke } from "@tauri-apps/api/core";
import "../i18n";
import App from "./App";

const invokeMock = vi.mocked(invoke);

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

  it("arranca y detiene la prueba de micrófono desde Atajos", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Atajos" }));
    const start = await screen.findByRole("button", { name: "Probar micrófono" });

    invokeMock.mockClear();
    fireEvent.click(start);
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("start_mic_test");
    });
    // Con la prueba activa aparecen el medidor y el veredicto de voz.
    expect(screen.getByRole("meter")).toBeInTheDocument();

    invokeMock.mockClear();
    fireEvent.click(screen.getByRole("button", { name: "Detener prueba" }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("stop_mic_test");
    });
  });

  it("detiene la prueba al salir de la sección Atajos", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Atajos" }));
    fireEvent.click(await screen.findByRole("button", { name: "Probar micrófono" }));
    await waitFor(() => {
      expect(screen.getByRole("meter")).toBeInTheDocument();
    });

    invokeMock.mockClear();
    fireEvent.click(screen.getByRole("button", { name: "General" }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("stop_mic_test");
    });
  });
});
