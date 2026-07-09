//! Spike R3 / ADR-0005 — mecanismo de inserción de texto (clipboard + pegado
//! sintético con restauración). Ver docs/2-arquitectura/spikes/r3-insercion-texto.md.
//!
//! Uso: `cargo run --bin spike-delivery -- "texto a insertar"`. Da 3 s para
//! hacer click en la app destino antes de copiar+pegar. Correr contra la
//! matriz del ADR-0005 (VS Code, terminal, navegador, Notepad/gedit, campo de
//! contraseña) en Windows y X11; repetir con Ctrl+Shift+V para terminales.

use std::{env, thread, time::Duration};

use arboard::Clipboard;
use enigo::{
    Direction::{Press, Release},
    Enigo, Key, Keyboard, Settings,
};

fn main() {
    let text = env::args()
        .nth(1)
        .expect("uso: spike-delivery \"texto a insertar\"");
    println!("Tenés 3s para hacer click en la app destino...");
    thread::sleep(Duration::from_secs(3));

    let mut cb = Clipboard::new().expect("no se pudo acceder al clipboard");
    let previous = cb.get_text().ok();
    cb.set_text(text)
        .expect("no se pudo escribir en el clipboard");

    let mut enigo = Enigo::new(&Settings::default()).expect("no se pudo inicializar enigo");
    enigo.key(Key::Control, Press).expect("fallo Ctrl down");
    enigo.key(Key::Unicode('v'), Press).expect("fallo V down");
    enigo.key(Key::Unicode('v'), Release).expect("fallo V up");
    enigo.key(Key::Control, Release).expect("fallo Ctrl up");

    thread::sleep(Duration::from_millis(300));
    if let Some(p) = previous {
        cb.set_text(p).ok();
    }
    println!("listo — revisar visualmente si el texto quedó insertado");
}
