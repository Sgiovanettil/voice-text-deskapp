// Módulos de negocio por capacidad: se agregan en PR4 (esqueleto) y en M1+
// (implementación). Ver docs/ARCHITECTURE.md §4 y §8.4.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
