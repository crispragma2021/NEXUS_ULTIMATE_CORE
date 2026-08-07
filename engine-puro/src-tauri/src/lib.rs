// ============================================================================
// 🧬 Cerebro Digital — Tauri UI v1 (Librería)
// ============================================================================

use cerebro_digital::cerebro::cerebro::CerebroAutoOptimizable;
use cerebro_digital::cerebro::estructuras::Entrada;
use std::sync::Mutex;

pub struct CerebroState {
    pub cerebro: Mutex<CerebroAutoOptimizable>,
    pub paso_actual: Mutex<u64>,
}

#[tauri::command]
fn simular_paso(
    state: tauri::State<'_, CerebroState>,
    texto: Option<String>,
    _intensidad: f32,
) -> Result<serde_json::Value, String> {
    let mut cerebro = state.cerebro.lock().map_err(|e| format!("Lock: {}", e))?;
    let mut paso = state.paso_actual.lock().map_err(|e| format!("Lock: {}", e))?;

    let estimulos = Vec::new();
    let entrada = Entrada { estimulos, texto };
    let dt = 0.001;
    let salida = cerebro.paso(dt, entrada);

    // Cada 100 pasos, auto-alimentar con texto generado (retroalimentación cognitiva interna)
    if *paso % 100 == 0 && !salida.texto.is_empty() && salida.texto != "escucho" && salida.texto != "escucho..." {
        let retro_entrada = Entrada {
            estimulos: Vec::new(),
            texto: Some(salida.texto.clone()),
        };
        let _ = cerebro.paso(dt * 0.5, retro_entrada);
    }

    *paso += 1;
    Ok(serde_json::json!({
        "texto": salida.texto,
        "emocion": salida.emocion,
        "conciencia": salida.conciencia,
        "actividad": salida.actividad
    }))
}

#[tauri::command]
fn obtener_estado_sistema(state: tauri::State<'_, CerebroState>) -> Result<serde_json::Value, String> {
    let cerebro = state.cerebro.lock().map_err(|e| format!("Lock: {}", e))?;
    let paso = state.paso_actual.lock().map_err(|e| format!("Lock: {}", e))?;

    let (vram_n, ram_n, total_n, ssd_e) = cerebro.memoria.estadisticas();

    Ok(serde_json::json!({
        "paso": *paso,
        "tiempo": cerebro.tiempo,
        "neuronas": {
            "vram": vram_n,
            "ram": ram_n,
            "total": total_n,
            "ssd_episodios": ssd_e
        },
        "emocion": {
            "dominante": cerebro.motores.amigdala.emocion_dominante(),
            "valencia": cerebro.motores.amigdala.alegria - cerebro.motores.amigdala.miedo,
            "alegria": cerebro.motores.amigdala.alegria,
            "miedo": cerebro.motores.amigdala.miedo,
            "ira": cerebro.motores.amigdala.ira
        },
        "conciencia": cerebro.motores.conciencia.intensidad,
        "dopamina": cerebro.motores.dopamina.nivel,
        "config": {
            "max_neuronas_ram": cerebro.config.max_neuronas_ram,
            "max_neuronas_vram": cerebro.config.max_neuronas_vram,
            "hilos_cpu": cerebro.config.hilos_cpu,
            "memoria_episodica_max": cerebro.config.memoria_episodica_max
        }
    }))
}

#[tauri::command]
fn reiniciar_cerebro(state: tauri::State<'_, CerebroState>) -> Result<(), String> {
    let mut cerebro = state.cerebro.lock().map_err(|e| format!("Lock: {}", e))?;
    let mut paso = state.paso_actual.lock().map_err(|e| format!("Lock: {}", e))?;
    *cerebro = CerebroAutoOptimizable::nuevo();
    *paso = 0;
    Ok(())
}

#[tauri::command]
fn obtener_historial_emocional(state: tauri::State<'_, CerebroState>) -> Result<Vec<f32>, String> {
    let cerebro = state.cerebro.lock().map_err(|e| format!("Lock: {}", e))?;
    Ok(cerebro.historial_emocional.clone())
}

#[tauri::command]
fn guardar_cerebro(state: tauri::State<'_, CerebroState>) -> Result<(), String> {
    let cerebro = state.cerebro.lock().map_err(|e| format!("Lock: {}", e))?;
    cerebro.guardar_a_disco()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("🧬 Inicializando Cerebro Digital UI...");
    let cerebro = CerebroAutoOptimizable::nuevo(); // ← auto-carga persistencia si existe

    tauri::Builder::default()
        .manage(CerebroState {
            cerebro: Mutex::new(cerebro),
            paso_actual: Mutex::new(0),
        })
        .invoke_handler(tauri::generate_handler![
            simular_paso,
            obtener_estado_sistema,
            reiniciar_cerebro,
            obtener_historial_emocional,
            guardar_cerebro,
        ])
        .run(tauri::generate_context!())
        .expect("Error al lanzar Cerebro Digital UI");
}
