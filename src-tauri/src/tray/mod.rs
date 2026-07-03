//! Icono de bandeja del sistema (residencia). La app vive en la bandeja: el
//! menú permite abrir la configuración o salir, y el click izquierdo abre la
//! configuración. Con esto la ventana de settings puede cerrarse a la bandeja
//! sin terminar el proceso (ver `on_window_event` en `lib.rs`).
//!
//! El tray es un plugin multiplataforma de Tauri (ARCHITECTURE.md §1): no
//! lleva `#[cfg(target_os)]`.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

const MENU_SETTINGS: &str = "tray_settings";
const MENU_QUIT: &str = "tray_quit";

/// Etiquetas del menú según el idioma de la UI. El menú de la bandeja es
/// nativo (no pasa por react-i18next), así que se localiza mínimamente aquí.
/// Se fija al construir la bandeja en el arranque; cambiar el idioma en
/// settings recién surte efecto tras reiniciar la app.
struct TrayLabels {
    settings: &'static str,
    quit: &'static str,
}

fn labels(ui_language: &str) -> TrayLabels {
    match ui_language {
        "en" => TrayLabels {
            settings: "Settings",
            quit: "Quit",
        },
        _ => TrayLabels {
            settings: "Configuración",
            quit: "Salir",
        },
    }
}

/// Muestra y enfoca la ventana de configuración (la des-oculta si estaba en
/// la bandeja). Punto único usado por el menú y el click del icono.
pub fn show_settings<R: Runtime>(app: &AppHandle<R>) {
    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

/// Construye el icono de bandeja con su menú. Se llama una vez en el setup.
pub fn build<R: Runtime>(app: &AppHandle<R>, ui_language: &str) -> tauri::Result<()> {
    let l = labels(ui_language);
    let settings_item = MenuItem::with_id(app, MENU_SETTINGS, l.settings, true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, MENU_QUIT, l.quit, true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&settings_item, &sep, &quit_item])?;

    TrayIconBuilder::with_id("main")
        .icon(
            app.default_window_icon()
                .cloned()
                .expect("la app siempre trae un icono por defecto"),
        )
        .tooltip("VoiceText")
        .menu(&menu)
        // El menú aparece con click derecho; el izquierdo abre configuración.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            MENU_SETTINGS => show_settings(app),
            MENU_QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_settings(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
