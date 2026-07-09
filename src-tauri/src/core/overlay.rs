//! Gestión de la ventana flotante del overlay (ARCHITECTURE §4.9). Ventana
//! sin foco, siempre encima, sin decoraciones, transparente y fuera de la
//! barra de tareas — validada por el spike R2 en Windows. Desde v1.x el
//! overlay es residente: se crea al arranque, queda siempre visible y el
//! usuario lo arrastra a gusto (drag region en el frontend); su posición se
//! persiste en settings (`general.overlay_position`).

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::config::OverlayPos;

const OVERLAY_LABEL: &str = "overlay";
// Tamaño de la skin v1 "materia" (assets/design/overlay-prototype.html),
// ampliada ~12% sobre el prototipo para dar más aire a la onda y al texto.
const OVERLAY_W: f64 = 464.0;
const OVERLAY_H: f64 = 132.0;
/// Separación desde el borde inferior de la pantalla (posición por defecto).
const OVERLAY_MARGIN: f64 = 56.0;

/// Crea el overlay residente y lo muestra. `saved` es la posición persistida
/// (píxeles físicos); sin ella se posiciona abajo-centro del monitor primario.
pub fn show(app: &AppHandle, saved: Option<OverlayPos>) {
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        position(&win, saved);
        let _ = win.show();
        return;
    }
    match WebviewWindowBuilder::new(app, OVERLAY_LABEL, WebviewUrl::App("overlay.html".into()))
        .title("VoiceText")
        .inner_size(OVERLAY_W, OVERLAY_H)
        .decorations(false)
        .always_on_top(true)
        .focusable(false)
        .skip_taskbar(true)
        .resizable(false)
        .transparent(true)
        .shadow(false)
        .visible(false)
        .build()
    {
        Ok(win) => {
            position(&win, saved);
            let _ = win.show();
        }
        Err(e) => tracing::error!(error = %e, "no se pudo crear la ventana overlay"),
    }
}

fn position(win: &WebviewWindow, saved: Option<OverlayPos>) {
    if let Some(pos) = saved {
        let _ = win.set_position(tauri::PhysicalPosition::new(pos.x, pos.y));
        return;
    }
    position_bottom_center(win);
}

/// Centra el overlay horizontalmente sobre el monitor primario, cerca del
/// borde inferior. Trabaja en píxeles físicos (lo que espera `set_position`).
fn position_bottom_center(win: &WebviewWindow) {
    let Ok(Some(monitor)) = win.primary_monitor() else {
        return;
    };
    let scale = monitor.scale_factor();
    let size = monitor.size();
    let origin = monitor.position();
    let win_w = (OVERLAY_W * scale) as i32;
    let win_h = (OVERLAY_H * scale) as i32;
    let margin = (OVERLAY_MARGIN * scale) as i32;
    let x = origin.x + (size.width as i32 - win_w) / 2;
    let y = origin.y + size.height as i32 - win_h - margin;
    let _ = win.set_position(tauri::PhysicalPosition::new(x, y));
}
