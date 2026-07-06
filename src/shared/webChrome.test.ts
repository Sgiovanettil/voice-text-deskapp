import { afterEach, describe, expect, it } from "vitest";

import { installWebChromeGuards } from "./webChrome";

// Los guards se instalan sobre un contenedor propio para no filtrar listeners
// entre casos; se limpia tras cada prueba.
let host: HTMLDivElement | null = null;

function guardedWindow(): Window {
  installWebChromeGuards(window);
  host = document.createElement("div");
  document.body.appendChild(host);
  return window;
}

afterEach(() => {
  host?.remove();
  host = null;
});

function keydown(init: KeyboardEventInit): boolean {
  const ev = new KeyboardEvent("keydown", { cancelable: true, bubbles: true, ...init });
  window.dispatchEvent(ev);
  return ev.defaultPrevented;
}

describe("installWebChromeGuards", () => {
  it("suprime el menú de clic derecho", () => {
    guardedWindow();
    const ev = new MouseEvent("contextmenu", { cancelable: true, bubbles: true });
    window.dispatchEvent(ev);
    expect(ev.defaultPrevented).toBe(true);
  });

  it("bloquea recargar la página (F5 y Ctrl+R)", () => {
    guardedWindow();
    expect(keydown({ key: "F5" })).toBe(true);
    expect(keydown({ key: "r", ctrlKey: true })).toBe(true);
    expect(keydown({ key: "R", ctrlKey: true, shiftKey: true })).toBe(true);
  });

  it("bloquea devtools, ver fuente, imprimir, buscar y zoom", () => {
    guardedWindow();
    expect(keydown({ key: "F12" })).toBe(true);
    expect(keydown({ key: "I", ctrlKey: true, shiftKey: true })).toBe(true);
    expect(keydown({ key: "u", ctrlKey: true })).toBe(true);
    expect(keydown({ key: "p", ctrlKey: true })).toBe(true);
    expect(keydown({ key: "f", ctrlKey: true })).toBe(true);
    expect(keydown({ key: "=", ctrlKey: true })).toBe(true);
    expect(keydown({ key: "-", ctrlKey: true })).toBe(true);
  });

  it("no interfiere con la escritura ni con copiar/pegar en inputs", () => {
    guardedWindow();
    expect(keydown({ key: "a" })).toBe(false);
    expect(keydown({ key: "c", ctrlKey: true })).toBe(false);
    expect(keydown({ key: "v", ctrlKey: true })).toBe(false);
    expect(keydown({ key: "x", ctrlKey: true })).toBe(false);
    expect(keydown({ key: "a", ctrlKey: true })).toBe(false);
  });
});
