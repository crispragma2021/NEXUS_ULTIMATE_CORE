use anyhow::Result;
use sysinfo::{Pid, System};
use tracing::{error, info, warn};

/// 🛡️ ÓRGANO: SISTEMA INMUNE OMEGA
/// Encargado de la detección de procesos extraños (Virus, Antiespionaje)
/// y la protección de la integridad del Ryzen 7 5700U.
pub struct SistemaInmune {
    sys: System,
    whitelist: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FaseInmune {
    Reconocimiento, // Detectado pero bajo observación.
    Opsonizacion,   // Marcado como proceso sospechoso / Amenaza potencial.
    Lisis,          // Acción ofensiva: Terminar proceso.
    Homeostasis,    // Entorno limpio.
}

impl Default for SistemaInmune {
    fn default() -> Self {
        Self::new()
    }
}

impl SistemaInmune {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        Self {
            sys,
            whitelist: vec![
                "nexus_daemon".to_string(),
                "despertar".to_string(),
                "cargo".to_string(),
                "rustc".to_string(),
                "python".to_string(),
                "python3".to_string(),
                "code".to_string(), // VS Code / Cursor
                "bash".to_string(),
                "antigravity".to_string(),
                "rust".to_string(),
                "systemd".to_string(),
                "dbus".to_string(),
                "gnome".to_string(),
                "x11".to_string(),
                "firefox".to_string(),
                "discord".to_string(),
                "telegram".to_string(),
                "spotify".to_string(),
                "qemu".to_string(),
                "kvm".to_string(),
                "node".to_string(),
                "npm".to_string(),
                "yarn".to_string(),
                "chrome".to_string(),
                "chromium".to_string(),
                "vbox".to_string(),
                "docker".to_string(),
                "libvirt".to_string(),
            ],
        }
    }

    /// 🔍 PATRULLAR: Escaneo de procesos en busca de antígenos (amenazas)
    pub fn patrullar(&mut self) -> Vec<(Pid, String, FaseInmune)> {
        self.sys.refresh_all();
        let mut hallazgos = Vec::new();

        for (pid, process) in self.sys.processes() {
            let name = process.name().to_string_lossy().to_lowercase();

            // 🛡️ REGLA 1: Detección de Procesos No Whitelisted con Carga Anómala
            if !self.es_aliado(&name) {
                let cpu = process.cpu_usage();
                if cpu > 10.0 {
                    // Uso sospechoso persistente
                    hallazgos.push((*pid, name.clone(), FaseInmune::Lisis));
                } else if cpu > 2.0 {
                    hallazgos.push((*pid, name.clone(), FaseInmune::Opsonizacion));
                }
            }
        }

        if hallazgos.is_empty() {
            info!("🛡️ [INMUNE] Patrulla completada. El entorno está en Homeostasis.");
        } else {
            warn!(
                "🚨 [INMUNE] Detectados {} procesos extraños fuera de la whitelist.",
                hallazgos.len()
            );
        }

        hallazgos
    }

    /// 🗡️ EJECUTAR_LISIS: Eliminar una amenaza detectada
    pub fn ejecutar_lisis(&mut self, pid: Pid) -> Result<()> {
        if let Some(process) = self.sys.process(pid) {
            let name = process.name().to_string_lossy().to_string();
            warn!(
                "🗡️ [INMUNE] Ejecutando LISIS sobre el proceso: {} (PID: {})",
                name, pid
            );
            process.kill();
            Ok(())
        } else {
            error!(
                "⚠️ [INMUNE] No se pudo encontrar el antígeno (PID: {}) para Lisis.",
                pid
            );
            Err(anyhow::anyhow!("Proceso no encontrado"))
        }
    }

    fn es_aliado(&self, name: &str) -> bool {
        self.whitelist.iter().any(|allowed| name.contains(allowed)) || name.is_empty()
    }
}
