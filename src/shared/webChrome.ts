// Endurece las ventanas para que se comporten como una app nativa y no como
// una página web: sin menú de clic derecho, sin recargar, sin devtools, sin
// zoom, sin ver código fuente, buscar ni imprimir. Solo se aplica en la app
// empaquetada; en `npm run dev` se conservan esos atajos para el desarrollo.

/** Combinación de teclas del "cromo de navegador" que bloqueamos. */
function isBrowserShortcut(e: KeyboardEvent): boolean {
  const ctrl = e.ctrlKey || e.metaKey;
  const key = e.key.toLowerCase();

  // Recargar la página (F5, Ctrl/Cmd+R, incluido el recargado duro con Shift).
  if (key === "f5" || (ctrl && key === "r")) return true;
  // DevTools (F12, Ctrl+Shift+I/J/C).
  if (key === "f12") return true;
  if (ctrl && e.shiftKey && (key === "i" || key === "j" || key === "c")) return true;
  // Ver código fuente (Ctrl+U), imprimir (Ctrl+P), buscar (Ctrl+F/G).
  if (ctrl && (key === "u" || key === "p" || key === "f" || key === "g")) return true;
  // Zoom del navegador (Ctrl con +, =, - o 0).
  if (ctrl && (key === "+" || key === "=" || key === "-" || key === "0")) return true;

  return false;
}

/**
 * Instala los listeners que suprimen el cromo de navegador en `target`. Se
 * expone aparte de [`lockDownWebChrome`] para poder probarlo sin depender del
 * entorno de build.
 */
export function installWebChromeGuards(target: Window = window): void {
  // Menú de clic derecho.
  target.addEventListener("contextmenu", (e) => e.preventDefault(), { capture: true });

  // Zoom con Ctrl + rueda del mouse.
  target.addEventListener(
    "wheel",
    (e) => {
      if (e.ctrlKey) e.preventDefault();
    },
    { capture: true, passive: false },
  );

  // Atajos de teclado de navegador (recarga, devtools, zoom, etc.).
  target.addEventListener(
    "keydown",
    (e) => {
      if (isBrowserShortcut(e)) e.preventDefault();
    },
    { capture: true },
  );
}

/**
 * Bloquea el cromo de navegador en la ventana actual. No hace nada en modo
 * desarrollo, donde devtools y recarga siguen disponibles.
 */
export function lockDownWebChrome(): void {
  if (import.meta.env.DEV) return;
  installWebChromeGuards();
}
