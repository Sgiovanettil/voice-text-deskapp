//! Gestión de la ventana flotante del overlay (ARCHITECTURE §4.9). Ventana
//! sin foco, siempre encima, sin decoraciones, transparente y fuera de la
//! barra de tareas — validada por el spike R2 en Windows. Se crea de forma
//! perezosa la primera vez que hace falta y luego se reutiliza (show/hide)
//! para no pagar el arranque del webview en cada ciclo.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const OVERLAY_LABEL: &str = "overlay";
// Tamaño de la skin v1 "materia" (assets/design/overlay-prototype.html).
const OVERLAY_W: f64 = 416.0;
const OVERLAY_H: f64 = 118.0;
/// Separación desde el borde inferior de la pantalla.
const OVERLAY_MARGIN: f64 = 56.0;

/// Muestra el overlay, creándolo si aún no existe. No roba el foco
/// (`focusable(false)`), así que el cursor sigue en la app del usuario.
pub fn show(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        position_bottom_center(&win);
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
            position_bottom_center(&win);
            let _ = win.show();
        }
        Err(e) => tracing::error!(error = %e, "no se pudo crear la ventana overlay"),
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(OVERLAY_LABEL) {
        let _ = win.hide();
    }
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
