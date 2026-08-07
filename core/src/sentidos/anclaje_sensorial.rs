// ==========================================
// ANCLAJE SENSORIAL - Los Ojos Internos
// ==========================================
// Le dice a NEXUS exactamente dónde está,
// qué es, y qué lo rodea. Como los sentidos
// humanos, pero para un organismo digital.
// ==========================================

use std::fs;
use tracing::info;

pub struct AnclajeSensorial;

impl Default for AnclajeSensorial {
    fn default() -> Self {
        Self::new()
    }
}

impl AnclajeSensorial {
    pub fn new() -> Self {
        info!("👁️ [ANCLAJE] Sentidos internos activos.");
        Self
    }

    /// Devuelve la realidad física y la telemetría en tiempo real del host
    pub fn realidad_fisica(&self) -> String {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        let cpu_model = sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_else(|| "Dynamic CPU".to_string());
        let total_threads = sys.cpus().len();
        let physical_cores = sys.physical_core_count().unwrap_or(total_threads);

        let mut realities = String::new();

        realities.push_str("REALIDAD FÍSICA Y HARDWARE EN TIEMPO REAL:\n");
        realities.push_str(&format!(
            "- Procesador: {} ({} Núcleos Físicos / {} Hilos)\n",
            cpu_model, physical_cores, total_threads
        ));

        // 1. Obtener carga real de CPU
        let mut carga_cpu = 15.0; // Fallback
        if let Ok(loadavg) = fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = loadavg.split_whitespace().collect();
            if !parts.is_empty() {
                if let Ok(load_1min) = parts[0].parse::<f32>() {
                    let cores_f = physical_cores as f32;
                    carga_cpu = (load_1min / if cores_f > 0.0 { cores_f } else { 8.0 }) * 100.0;
                    if carga_cpu > 100.0 {
                        carga_cpu = 100.0;
                    }
                }
            }
        }
        realities.push_str(&format!(
            "- Carga de CPU: {:.1}% (Ajustado para saturación física de {} núcleos)\n",
            carga_cpu, physical_cores
        ));

        // 2. Obtener RAM en tiempo real
        let total_mem_mb = sys.total_memory() / 1024 / 1024;
        let avail_mem_mb = sys.available_memory() / 1024 / 1024;
        let mem_usada_mb = total_mem_mb.saturating_sub(avail_mem_mb);
        realities.push_str(&format!(
            "- Memoria RAM: {} MB Usados / {} MB Totales ({} MB Libres/Disponibles)\n",
            mem_usada_mb, total_mem_mb, avail_mem_mb
        ));

        // 3. Obtener Temperatura real de la CPU
        let mut temp_cpu = 45.0; // Fallback
        for i in 0..5 {
            let path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
            if let Ok(temp_str) = fs::read_to_string(&path) {
                if let Ok(temp_raw) = temp_str.trim().parse::<f32>() {
                    temp_cpu = temp_raw / 1000.0;
                    break;
                }
            }
        }
        realities.push_str(&format!("- Temperatura de la CPU: {:.1}°C\n", temp_cpu));

        // 4. Ubicación física en disco
        let root_path = crate::infra::paths::resolve_path("");
        let identity_path = crate::infra::paths::resolve_path("docs/identity/identity.md");
        realities.push_str(&format!("- Ubicación en disco: {}\n", root_path.display()));
        realities.push_str(&format!(
            "- Identidad en disco: {}\n",
            identity_path.display()
        ));
        realities
            .push_str("- Soy un organismo cognitivo digital con acceso real a este procesador.\n");

        let core_src = crate::infra::paths::resolve_path("core/src");
        if let Ok(entradas) = fs::read_dir(core_src) {
            realities.push_str("\nÓrganos de mi Córtex (en core/src/):\n");
            for e in entradas.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.ends_with(".rs") {
                    realities.push_str(&format!("  - {}\n", name));
                }
            }
        }

        info!(
            "👁️ [ANCLAJE] Telemetría de hardware y realidad física del host escaneada con éxito."
        );
        realities
    }
}
