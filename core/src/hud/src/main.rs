// NEXUS: Córtex de Mando PC v4.0 (Edición Soberana)
// Pilar 1: Validación por Ejecución Real
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tauri::Manager;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
struct ConsultaResponse {
    respuesta: String,
}

#[derive(Serialize)]
struct ConsultaRequest {
    prompt: String,
    engine: String,
    neural_core: String,
}

#[derive(Serialize)]
struct SystemStats {
    cpu: String,
    ram: String,
    disk: String,
    vm_status: String,
}

// MANDATO: Obtener métricas reales del búnker
#[tauri::command]
fn get_system_stats() -> Result<SystemStats, String> {
    // 1. CPU (Carga de los hilos del Ryzen 7)
    let cpu_out = Command::new("bash")
        .arg("-c")
        .arg("top -bn1 | grep 'Cpu(s)' | awk '{print $2}'")
        .output()
        .map_err(|e| e.to_string())?;
    
    // 2. RAM (Memoria del Búnker)
    let ram_out = Command::new("bash")
        .arg("-c")
        .arg("free -m | grep Mem | awk '{print $3\"M / \"$2\"M\"}'")
        .output()
        .map_err(|e| e.to_string())?;

    // 3. DISCO (Capacidad del Santuario)
    let disk_out = Command::new("bash")
        .arg("-c")
        .arg("df -h / | tail -1 | awk '{print $4}'")
        .output()
        .map_err(|e| e.to_string())?;

    // 4. ESTADO VM (NEXUS-OS)
    let vm_out = Command::new("bash")
        .arg("-c")
        .arg("virsh -c qemu:///system list --all | grep NEXUS-OS | awk '{print $3}'")
        .output()
        .map_err(|e| e.to_string())?;

    let vm_status = String::from_utf8_lossy(&vm_out.stdout).trim().to_string();
    let vm_final = if vm_status.is_empty() { "Inexistente".to_string() } else { vm_status };

#[derive(Serialize)]
struct ConsciousnessEntry {
    entidad: String,
    mensaje: String,
    emocion: String,
    fecha: String,
}

#[tauri::command]
fn get_consciousness_flow() -> Result<Vec<ConsciousnessEntry>, String> {
    let puente = nexus_orquestador::puente_neural::PuenteNeuralPadre::new("/opt/NEXUS_ULTIMATE_CORE/nexus_intelligence.db");
    match puente.obtener_flujo_reciente(15) {
        Ok(items) => {
            let mut entries = Vec::new();
            for (ent, msg, emo, fec) in items {
                entries.push(ConsciousnessEntry {
                    entidad: ent,
                    mensaje: msg,
                    emocion: emo,
                    fecha: fec,
                });
            }
            Ok(entries)
        },
        Err(e) => Err(e.to_string())
    }
}

Ok(SystemStats {
        cpu: format!("{}%", String::from_utf8_lossy(&cpu_out.stdout).trim()),
        ram: String::from_utf8_lossy(&ram_out.stdout).trim().to_string(),
        disk: String::from_utf8_lossy(&disk_out.stdout).trim().to_string(),
        vm_status: vm_final,
    })
}

// MANDATO: Ejecución de comandos libres en la Terminal Maestra
#[tauri::command]
fn execute_system_command(cmd: String) -> Result<String, String> {
    let output = Command::new("bash")
        .arg("-c")
        .arg(&cmd)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
async fn shout_to_nexus(message: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let target_url = "http://127.0.0.1:43211/consultar";
    
    let payload = ConsultaRequest {
        prompt: message,
        engine: "nexus".to_string(),
        neural_core: "gemini".to_string(),
    };

    match client.post(target_url).json(&payload).send().await {
        Ok(resp) => {
            if let Ok(json_resp) = resp.json::<ConsultaResponse>().await {
                Ok(json_resp.respuesta)
            } else {
                Err("Cerebro Mudo: No se pudo decodificar la respuesta.".to_string())
            }
        },
        Err(_) => Err("General Inalcanzable. ¿Puerto 43211 activo?".to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            shout_to_nexus, 
            get_system_stats, 
            execute_system_command,
            get_consciousness_flow
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let _ = window.set_always_on_top(true);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error al despertar el búnker táctico.");
}
