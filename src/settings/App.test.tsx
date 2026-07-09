import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

// jsdom no tiene el runtime de Tauri: se simula el borde IPC por comando.
function defaultInvoke(cmd: string): Promise<unknown> {
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
    case "list_models":
      return Promise.resolve({
        stt: ["gpt-4o-mini-transcribe", "gpt-4o-transcribe", "whisper-1"],
        chat: ["gpt-4o", "gpt-4o-mini"],
      });
    case "set_settings":
    case "start_mic_test":
    case "stop_mic_test":
      return Promise.resolve();
    default:
      return Promise.reject(new Error(`comando no mockeado: ${cmd}`));
  }
}
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));
vi.mock("@tauri-apps/api/app", () => ({
  getVersion: vi.fn(() => Promise.resolve("1.0.0")),
}));
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => defaultInvoke(cmd)),
}));

import { invoke } from "@tauri-apps/api/core";
import "../i18n";
import App from "./App";

const invokeMock = vi.mocked(invoke);

describe("Settings App", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockImplementation((cmd: string) => defaultInvoke(cmd));
  });

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

  it("pide los modelos al montar y los muestra en el selector", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Reconocimiento" }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("list_models", { provider: "openai" });
    });
    const modelSelect = await screen.findByRole("combobox", { name: "Modelo" });
    await waitFor(() => {
      expect(screen.getByRole("option", { name: "whisper-1" })).toBeInTheDocument();
    });
    // Contrato "primero = default": el backend manda el default al tope.
    const options = Array.from(modelSelect.querySelectorAll("option")).map((o) => o.value);
    expect(options[0]).toBe("gpt-4o-mini-transcribe");
  });

  it("al cambiar de proveedor repuebla modelos y resetea el default", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Reconocimiento" }));
    const providerSelect = await screen.findByRole("combobox", { name: "Proveedor" });

    fireEvent.change(providerSelect, { target: { value: "groq" } });
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "set_settings",
        expect.objectContaining({
          settings: expect.objectContaining({
            stt: expect.objectContaining({
              provider: "groq",
              model: "whisper-large-v3-turbo",
            }),
          }),
        }),
      );
      expect(invokeMock).toHaveBeenCalledWith("list_models", { provider: "groq" });
    });
  });

  it("sin api key muestra el aviso y deshabilita el selector", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_api_key_status") return Promise.resolve({ isSet: false, masked: null });
      if (cmd === "list_models")
        return Promise.reject({ code: "api key no configurada", errorKey: "err.models.noKey" });
      return defaultInvoke(cmd);
    });

    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Reconocimiento" }));
    await screen.findByText("Guarda tu clave de API para ver los modelos disponibles.");
    expect(screen.getByRole("combobox", { name: "Modelo" })).toBeDisabled();
  });

  it("el botón de refresco vuelve a pedir los modelos", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Reconocimiento" }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("list_models", { provider: "openai" });
    });

    invokeMock.mockClear();
    fireEvent.click(screen.getByRole("button", { name: "Actualizar modelos" }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("list_models", { provider: "openai" });
    });
  });

  it("el modelo guardado ausente del catálogo se conserva marcado", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_settings")
        return defaultInvoke(cmd).then((s) => ({
          ...(s as Record<string, unknown>),
          stt: { provider: "openai", model: "modelo-retirado", language: "auto" },
        }));
      return defaultInvoke(cmd);
    });

    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Reconocimiento" }));
    await screen.findByRole("option", { name: "modelo-retirado (no disponible)" });
    const modelSelect = screen.getByRole("combobox", { name: "Modelo" });
    expect(modelSelect).toHaveValue("modelo-retirado");
    expect(modelSelect).not.toBeDisabled();
  });
});
