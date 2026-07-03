//! Spike R2 — overlay sin foco. Ver docs/spikes/r2-overlay-sin-foco.md.
//!
//! Protocolo de prueba manual: abrir un editor de texto y dejar el cursor en
//! un campo; ejecutar `cargo run --bin spike-overlay`; sin tocar el mouse,
//! escribir un carácter. Si el carácter llega al editor y NO a esta ventana
//! verde, `focusable(false)` funcionó — el overlay no robó el foco (R2,
//! principio 3 del PRD). Repetir en Windows, X11, Wayland GNOME y Wayland KDE.

fn main() {
    // WebView2 en Windows no renderiza navegaciones a data: URLs (ventana
    // queda en blanco), así que el spike carga los assets embebidos de la app
    // (frontendDist compilado dentro del binario) y reemplaza el contenido
    // con el aviso verde vía initialization script, antes de que cargue React.
    let overlay_script = "\
        window.addEventListener('DOMContentLoaded', () => {\
            document.documentElement.innerHTML =\
                \"<body style='margin:0;background:#222;color:#0f0;\
                font-family:sans-serif;display:flex;align-items:center;\
                justify-content:center;height:100vh'>\
                ESCUCHANDO (spike-overlay)</body>\";\
        });";
    tauri::Builder::default()
        .setup(move |app| {
            tauri::WebviewWindowBuilder::new(
                app,
                "spike-overlay",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .initialization_script(overlay_script)
            .always_on_top(true)
            .focusable(false)
            .decorations(false)
            .skip_taskbar(true)
            .resizable(false)
            .inner_size(320.0, 90.0)
            .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("spike-overlay failed");
}
