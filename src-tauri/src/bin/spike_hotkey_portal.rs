//! Spike R1 / ADR-0004 — hotkey global vía XDG Desktop Portal
//! `GlobalShortcuts` (Wayland GNOME/KDE). Ver docs/2-arquitectura/spikes/r1-hotkey-portal.md.
//!
//! Uso: `cargo run --bin spike-hotkey-portal` en una sesión GNOME/KDE Wayland
//! real. Debería aparecer el diálogo de aprobación del portal la primera vez;
//! confirmar y luego presionar/soltar el atajo asignado para ver los eventos.

#[cfg(target_os = "linux")]
mod linux {
    use futures_util::StreamExt;

    pub async fn run() -> ashpd::Result<()> {
        use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};

        let proxy = GlobalShortcuts::new().await?;
        let session = proxy.create_session(Default::default()).await?;
        let shortcut = NewShortcut::new("dictate", "VoiceText — spike de dictado");
        proxy
            .bind_shortcuts(&session, &[shortcut], None, Default::default())
            .await?;
        println!(
            "Diálogo de aprobación del portal debería haber aparecido. \
             Presioná/soltá el atajo asignado (Ctrl+C para salir)."
        );

        let mut activated = proxy.receive_activated().await?;
        while let Some(signal) = activated.next().await {
            println!("HotkeyPressed (portal): {signal:?}");
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() -> ashpd::Result<()> {
    linux::run().await
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!(
        "spike-hotkey-portal solo aplica a Linux/Wayland \
         (XDG Desktop Portal GlobalShortcuts, R1/ADR-0004)."
    );
}
