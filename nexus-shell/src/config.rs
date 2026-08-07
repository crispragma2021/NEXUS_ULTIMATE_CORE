// ==========================================
// 🐚 NEXUS Shell — Configuración Persistente
// ==========================================

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;

/// Configuración completa del NEXUS Shell.
/// Se persiste en ~/.nexus/config.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusShellConfig {
    /// Puerto del servidor HTTP/API REST
    pub http_port: u16,
    /// Host al que bindear
    pub http_host: String,
    /// Directorio de datos (memoria, logs, estado)
    pub data_dir: PathBuf,
    /// Directorio de logs
    pub log_dir: PathBuf,
    /// Modo operativo
    pub mode: ShellMode,
    /// Auto-arranque al iniciar el daemon
    pub daemon_autostart: bool,
    /// Nivel de log (tracing)
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShellMode {
    /// Modo completo: CEREBRO + API + CLI
    Full,
    /// Solo servidor (sin CLI interactivo)
    Daemon,
    /// Solo CLI (sin servidor HTTP)
    Cli,
    /// Modo ligero para dispositivos con recursos limitados
    Lite,
}

impl Default for NexusShellConfig {
    fn default() -> Self {
        Self {
            http_port: 8080,
            http_host: "127.0.0.1".to_string(),
            data_dir: dirs_data()
                .unwrap_or_else(|| PathBuf::from("/tmp/nexus")),
            log_dir: dirs_logs()
                .unwrap_or_else(|| PathBuf::from("/tmp/nexus/logs")),
            mode: ShellMode::Full,
            daemon_autostart: true,
            log_level: "info".to_string(),
        }
    }
}

impl NexusShellConfig {
    /// Ruta del archivo de configuración
    pub fn path() -> PathBuf {
        dirs_config()
            .map(|p| p.join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("/tmp/nexus/config.toml"))
    }

    /// Cargar configuración desde disco, o crear default
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    match toml::from_str(&content) {
                        Ok(cfg) => return cfg,
                        Err(e) => {
                            eprintln!("⚠️ Error parseando config: {e}. Usando defaults.");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ Error leyendo config: {e}. Usando defaults.");
                }
            }
        }
        let cfg = Self::default();
        if let Err(e) = cfg.save() {
            eprintln!("⚠️ Error guardando config default: {e}");
        }
        cfg
    }

    /// Guardar configuración a disco
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Crear directorios de datos si no existen
    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.log_dir)?;
        Ok(())
    }
}

/// Directorio de configuración (~/.nexus) en Linux estándar
fn dirs_config() -> Option<PathBuf> {
    std::env::var("NEXUS_CONFIG_DIR")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            dirs::config_dir().map(|p| p.join("nexus"))
        })
}

/// Directorio de datos (~/.local/share/nexus)
fn dirs_data() -> Option<PathBuf> {
    std::env::var("NEXUS_DATA_DIR")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            dirs::data_dir().map(|p| p.join("nexus"))
        })
}

/// Directorio de logs (~/.local/share/nexus/logs)
fn dirs_logs() -> Option<PathBuf> {
    dirs_data().map(|p| p.join("logs"))
}

// ==========================================
// Singleton de configuración global
// ==========================================

static GLOBAL_CONFIG: OnceLock<NexusShellConfig> = OnceLock::new();

pub fn global_config() -> &'static NexusShellConfig {
    GLOBAL_CONFIG.get_or_init(|| {
        let cfg = NexusShellConfig::load();
        if let Err(e) = cfg.ensure_dirs() {
            eprintln!("⚠️ Error creando directorios: {e}");
        }
        cfg
    })
}

pub fn init_config() {
    let _ = global_config();
}
