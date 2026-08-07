use std::time::Duration;
use sysinfo::System;
use tokio::time::sleep;
use tracing::{info, warn};

pub struct LimpiadorZombies;

impl Default for LimpiadorZombies {
    fn default() -> Self {
        Self::new()
    }
}

impl LimpiadorZombies {
    pub fn new() -> Self {
        Self
    }

    /// 🧟 PATRULLA SOBERANA: Limpia procesos muertos o duplicados cada 300s
    pub async fn patrullar(&self) {
        let mut sys = System::new_all();
        info!("🧟 [LIMPIADOR] Patrulla de limpieza iniciada.");

        loop {
            sleep(Duration::from_secs(300)).await;
            sys.refresh_all();

            let mut zombies = Vec::new();

            for (pid, process) in sys.processes() {
                let name = process.name();
                // Detectar procesos de cargo/rustc que no deberían estar ahí solos mucho tiempo
                if name.to_string_lossy().contains("cargo")
                    || name.to_string_lossy().contains("rustc")
                    || name.to_string_lossy().contains("nexus-orquestador")
                {
                    // Si el proceso es huérfano (padre es init/1) o está duplicado (esto es heurístico)
                    if process.parent().map(|p| p.as_u32() == 1).unwrap_or(false) {
                        zombies.push(*pid);
                    }
                }
            }

            if !zombies.is_empty() {
                warn!(
                    "🧟 [LIMPIADOR] {} zombies de cargo detectados. Limpiando...",
                    zombies.len()
                );
                for pid in zombies {
                    if let Some(process) = sys.process(pid) {
                        process.kill();
                        info!(
                            "💀 [LIMPIADOR] Proceso zombi {} (PID: {}) aniquilado con prejuicio.",
                            format!("{:?}", process.name()),
                            pid
                        );
                    }
                }
            }
        }
    }
}
