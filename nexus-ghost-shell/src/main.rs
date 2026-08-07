// ==========================================
// NEXUS GHOST-SHELL - CORE SYSTEM (RUST)
// ==========================================

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, WindowEvent};
use std::fs;

// Comando para leer archivos
#[tauri::command]
fn read_nexus_file(path: String) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| e.to_string())
}

// Comando para guardar archivos
#[tauri::command]
fn save_nexus_file(path: String, content: String) -> Result<(), String> {
    fs::write(path, content).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(tauri::generate_handler![read_nexus_file, save_nexus_file])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // Configuración OMEGA: Siempre encima y sin bordes
            let _ = window.set_always_on_top(true);
            let _ = window.set_decorations(false);

            println!("🛰️ GHOST-SHELL: Núcleo Rust inicializado.");
            Ok(())
        })
        .on_window_event(|_window, event| {
            if let WindowEvent::Focused(false) = event {
                // Opcional: Auto-ocultar
            }
        })
        .run(tauri::generate_context!())
        .expect("Error al ejecutar NEXUS GHOST-SHELL");
}
