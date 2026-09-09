use crate::autodiagnostico::sentinel_core::{HealthProbe, ProbeResult, ProbeTier};
use async_trait::async_trait;
use std::time::Instant;
use tokio::fs;

pub struct ProbeMemory;

impl ProbeMemory {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HealthProbe for ProbeMemory {
    async fn check(&self) -> ProbeResult {
        let start_time = Instant::now();
        let mut passed = true;
        let mut mensaje = String::new();
        let mut detalles = serde_json::json!({});

        // 1. Read /proc/meminfo for RAM and Swap
        let mut ram_used_pct = 0.0;
        let mut swap_used_pct = 0.0;
        if let Ok(content) = fs::read_to_string("/proc/meminfo").await {
            let mut total_ram: u64 = 0;
            let mut free_ram: u64 = 0;
            let mut total_swap: u64 = 0;
            let mut free_swap: u64 = 0;

            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    total_ram = line
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                } else if line.starts_with("MemAvailable:") {
                    free_ram = line
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                } else if line.starts_with("SwapTotal:") {
                    total_swap = line
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                } else if line.starts_with("SwapFree:") {
                    free_swap = line
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                }
            }

            // MemAvailable cuenta la memoria realmente libre (excluye cache/buffers
            // recuperables). Esto evita falsos positivos: MemFree no restaba el caché.
            if total_ram > 0 {
                let used_ram = total_ram.saturating_sub(free_ram);
                ram_used_pct = (used_ram as f32 / total_ram as f32) * 100.0;
            }
            if total_swap > 0 {
                swap_used_pct = ((total_swap - free_swap) as f32 / total_swap as f32) * 100.0;
            }

            detalles["ram_used_pct"] = serde_json::json!(ram_used_pct);
            detalles["swap_used_pct"] = serde_json::json!(swap_used_pct);

            if ram_used_pct > 90.0 {
                passed = false;
                mensaje.push_str(&format!("Uso de RAM crítico: {:.1}%. ", ram_used_pct));
            }
            if swap_used_pct > 80.0 {
                passed = false;
                mensaje.push_str(&format!("Uso de SWAP crítico: {:.1}%. ", swap_used_pct));
            }
        } else {
            passed = false;
            mensaje.push_str("No se pudo leer /proc/meminfo. ");
        }

        // 2. Read CPU Temperature (Linux specific)
        let mut cpu_temp_c = 0.0;
        if let Ok(content) = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").await {
            if let Ok(temp_raw) = content.trim().parse::<f32>() {
                cpu_temp_c = temp_raw / 1000.0;
                detalles["cpu_temp_c"] = serde_json::json!(cpu_temp_c);
                if cpu_temp_c > 85.0 {
                    passed = false;
                    mensaje.push_str(&format!(
                        "Temperatura de CPU crítica: {:.1}°C. ",
                        cpu_temp_c
                    ));
                }
            }
        } else {
            // Try alternative path for CPU temp
            if let Ok(content) = fs::read_to_string("/sys/class/hwmon/hwmon0/temp1_input").await {
                if let Ok(temp_raw) = content.trim().parse::<f32>() {
                    cpu_temp_c = temp_raw / 1000.0;
                    detalles["cpu_temp_c"] = serde_json::json!(cpu_temp_c);
                    if cpu_temp_c > 85.0 {
                        passed = false;
                        mensaje.push_str(&format!(
                            "Temperatura de CPU crítica: {:.1}°C. ",
                            cpu_temp_c
                        ));
                    }
                }
            } else {
                mensaje.push_str("No se pudo leer la temperatura de CPU. ");
            }
        }

        if mensaje.is_empty() {
            mensaje.push_str("Uso de recursos óptimo.");
        }

        ProbeResult {
            nombre: self.nombre().to_string(),
            tier: self.tier(),
            passed,
            mensaje,
            detalles: Some(detalles),
            latencia_ms: start_time.elapsed().as_millis() as u64,
        }
    }

    fn tier(&self) -> ProbeTier {
        ProbeTier::Warning
    }

    fn nombre(&self) -> &'static str {
        "System Resources"
    }
}
