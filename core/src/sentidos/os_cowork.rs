// ==========================================
// 🖥️ OS COWORKER — Contexto de Sistema Operativo
// ==========================================
// Integración de contexto de ventana activa, portapapeles
// y acciones de sistema para el compañero digital.
//
// Legacy DNA: nexus-orquestador/src/sentidos_vision/os_cowork.rs
// Absorbido: 11-Jun-2026

use std::process::Command;
use tracing::{info, warn};

/// Compañero digital que integra contexto del Sistema Operativo.
/// Proporciona acceso a ventana activa, portapapeles y acciones de sistema.
pub struct OsCoworker;

impl Default for OsCoworker {
    fn default() -> Self {
        Self::new()
    }
}

impl OsCoworker {
    pub fn new() -> Self {
        Self
    }

    /// Obtiene el contexto de la ventana activa (Linux/X11 o Wayland).
    /// Requiere `xdotool` instalado en el sistema.
    pub fn get_active_window_context(&self) -> String {
        info!("🖥️ [COWORK] Capturando contexto de ventana activa...");
        let output = Command::new("xdotool")
            .args(["getactivewindow", "getwindowname"])
            .output();

        match output {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            _ => {
                warn!("⚠️ xdotool no disponible o fallo en captura.");
                "Unknown Window".to_string()
            }
        }
    }

    /// Lee el portapapeles de forma segura.
    /// Requiere `xclip` instalado en el sistema.
    pub fn read_clipboard(&self) -> String {
        let output = Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output();

        match output {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
            _ => "Clipboard Empty/Unavailable".to_string(),
        }
    }

    /// Abre una carpeta de proyecto en el gestor de archivos del sistema.
    /// Requiere `xdg-open` instalado en el sistema.
    pub fn open_project_folder(&self, path: &str) -> anyhow::Result<()> {
        info!("📂 [COWORK] Abriendo carpeta de proyecto: {}", path);
        Command::new("xdg-open").arg(path).spawn()?;
        Ok(())
    }
}
