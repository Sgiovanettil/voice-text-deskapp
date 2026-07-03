//! Spike R2 — overlay sin foco. Ver docs/spikes/r2-overlay-sin-foco.md.
//!
//! Protocolo de prueba manual: abrir un editor de texto y dejar el cursor en
//! un campo; ejecutar `cargo run --bin spike-overlay`; sin tocar el mouse,
//! escribir un carácter. Si el carácter llega al editor y NO a esta ventana
//! verde, `focusable(false)` funcionó — el overlay no robó el foco (R2,
//! principio 3 del PRD). Repetir en Windows, X11, Wayland GNOME y Wayland KDE.

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Un '#' o espacio sin escapar en un data: URL se interpreta como
            // inicio del fragment (#...) y trunca todo lo que sigue — por
            // eso los colores hex y el texto se codifican antes de armar la URL.
            let html_body = "<body style='margin:0;background:#222;color:#0f0;\
                font-family:sans-serif;display:flex;align-items:center;justify-content:center;\
                height:100vh'>ESCUCHANDO (spike-overlay)</body>";
            let encoded = html_body.replace('#', "%23").replace(' ', "%20");
            let url = format!("data:text/html,{encoded}");
            tauri::WebviewWindowBuilder::new(
                app,
                "spike-overlay",
                tauri::WebviewUrl::External(url.parse().unwrap()),
            )
            .always_on_top(true)
            .focusable(false)
            .decorations(false)
            .transparent(true)
            .skip_taskbar(true)
            .resizable(false)
            .inner_size(320.0, 90.0)
            .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("spike-overlay failed");
}
