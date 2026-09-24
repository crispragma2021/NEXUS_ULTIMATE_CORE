// ==========================================
// 🦅 HERRAMIENTAS NATIVAS — Punto de acceso soberano a sistema de archivos, shell y búsqueda
// ==========================================
// Módulo consolidado que reemplaza a AgenteEjecutor como la interfaz nativa
// para todas las herramientas del sistema. Absorbe la lógica de claws-mcp
// y la elimina como dependencia externa.
//
// Capacidades:
// 1. leer_archivo — con validación de path traversal
// 2. escribir_archivo — con backup automático pre-escritura
// 3. buscar_codigo_regex — usando ripgrep si disponible
// 4. ejecutar_comando — a través de JuicioSoberano
// ==========================================

use crate::efectores::agente_ejecutor::{AgenteEjecutor, ToolCall, ToolResponse};
use crate::efectores::nexus_claw_pro::NexusClawPro;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

// ── Constantes de seguridad ──────────────────────────────────────────

/// Extensiones de backup pre-escritura
const BACKUP_EXTENSION: &str = "bak";

/// Número máximo de caracteres en una línea para búsqueda regex
const REGEX_LINE_LIMIT: usize = 4096;

/// Tiempo máximo de ejecución de un comando shell (segundos)
const SHELL_TIMEOUT_SECS: u64 = 120;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Registro de telemetría y salud para el Protocolo Autogenético de Herramientas (AGP).
#[derive(Debug, Clone, Default)]
pub struct Metricasherramienta {
    pub success_count: u64,
    pub failure_count: u64,
    pub latencia_promedio_ms: u64,
    pub estado_salud: String,
}

/// Punto de entrada soberano para todas las herramientas del sistema.
/// Wrapper sobre AgenteEjecutor con funcionalidad extendida.
pub struct HerramientasNativas {
    executor: AgenteEjecutor,
    workspace_root: PathBuf,
    telemetria_agp: Arc<Mutex<HashMap<String, Metricasherramienta>>>,
}

impl HerramientasNativas {
    pub fn new(claw: NexusClawPro) -> Self {
        let executor = AgenteEjecutor::new(claw);
        let workspace_root = crate::infra::paths::resolve_path("");
        Self {
            executor,
            workspace_root,
            telemetria_agp: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn new_from_executor(executor: AgenteEjecutor) -> Self {
        let workspace_root = crate::infra::paths::resolve_path("");
        Self {
            executor,
            workspace_root,
            telemetria_agp: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn executor(&self) -> &AgenteEjecutor {
        &self.executor
    }

    /// Lee un archivo con validación de seguridad.
    pub async fn leer_archivo(&self, ruta: &str) -> Result<String> {
        self.executor.leer_archivo(ruta).await
    }

    /// Escribe un archivo con backup automático.
    pub async fn escribir_archivo(&self, ruta: &str, contenido: &str) -> Result<String> {
        self.executor.escribir_archivo(ruta, contenido).await
    }

    /// Busca con regex usando ripgrep (nativo, sin dependencias externas).
    pub async fn buscar_codigo_regex(&self, query: &str) -> Result<String> {
        self.executor.buscar_codigo_regex(query).await
    }

    /// Ejecuta comando shell auditado.
    pub async fn ejecutar_comando(&self, comando: &str) -> Result<String> {
        self.executor.ejecutar_comando_seguro(comando).await
    }

    /// Resuelve una llamada de herramienta MCP directamente.
    pub async fn resolver_herramienta(&self, call: ToolCall) -> ToolResponse {
        self.executor.resolver_herramienta(call).await
    }

    // ── Métodos extendidos (más allá de AgenteEjecutor) ─────────────

    /// Verifica si una ruta existe en el workspace.
    pub fn ruta_existe(&self, ruta: &str) -> bool {
        let path = Path::new(ruta);
        if path.is_absolute() {
            path.exists()
        } else {
            self.workspace_root.join(path).exists()
        }
    }

    /// Lista archivos en un directorio (no recursivo).
    pub fn listar_directorio(&self, ruta: &str) -> Result<Vec<String>> {
        let base = if Path::new(ruta).is_absolute() {
            PathBuf::from(ruta)
        } else {
            self.workspace_root.join(ruta)
        };
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&base)? {
            let entry = entry?;
            entries.push(entry.file_name().to_string_lossy().to_string());
        }
        entries.sort();
        Ok(entries)
    }

    /// Obtiene metadatos de un archivo (tamaño, modificado).
    pub fn metadata_archivo(&self, ruta: &str) -> Result<FileMetadata> {
        let path = if Path::new(ruta).is_absolute() {
            PathBuf::from(ruta)
        } else {
            self.workspace_root.join(ruta)
        };
        let meta = std::fs::metadata(&path)?;
        Ok(FileMetadata {
            size: meta.len(),
            modified: meta
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            is_file: meta.is_file(),
            is_dir: meta.is_dir(),
        })
    }

    /// Registrar telemetría y resultado de ejecución de una herramienta (Protocolo AGP).
    pub fn registrar_telemetria_herramienta(&self, tool_name: &str, exito: bool, latencia_ms: u64) {
        let mut map = self.telemetria_agp.lock().unwrap();
        let metricas = map.entry(tool_name.to_string()).or_insert_with(|| Metricasherramienta {
            success_count: 0,
            failure_count: 0,
            latencia_promedio_ms: latencia_ms,
            estado_salud: "Optimo".to_string(),
        });

        if exito {
            metricas.success_count += 1;
        } else {
            metricas.failure_count += 1;
        }

        let total = metricas.success_count + metricas.failure_count;
        let tasa_exito = metricas.success_count as f32 / total as f32;

        if tasa_exito < 0.5 {
            metricas.estado_salud = "Degradado".to_string();
        } else {
            metricas.estado_salud = "Optimo".to_string();
        }
    }

    /// Obtiene la tasa de éxito (0.0 a 1.0) para una herramienta registrada.
    pub fn obtener_tasa_exito(&self, tool_name: &str) -> f32 {
        let map = self.telemetria_agp.lock().unwrap();
        if let Some(m) = map.get(tool_name) {
            let total = m.success_count + m.failure_count;
            if total == 0 {
                1.0
            } else {
                m.success_count as f32 / total as f32
            }
        } else {
            1.0
        }
    }
}

/// Metadatos de archivo.
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: u64,
    pub is_file: bool,
    pub is_dir: bool,
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruta_existe_raiz() {
        let claw = NexusClawPro::new_empty();
        let h = HerramientasNativas::new(claw);
        // El workspace siempre existe
        assert!(h.ruta_existe("."));
    }

    #[test]
    fn test_ruta_no_existe() {
        let claw = NexusClawPro::new_empty();
        let h = HerramientasNativas::new(claw);
        assert!(!h.ruta_existe("/ruta/que/no/existe/12345xyz"));
    }

    #[test]
    fn test_listar_directorio_vacio() {
        let claw = NexusClawPro::new_empty();
        let h = HerramientasNativas::new(claw);
        // Al menos debe listar algo en la raíz del workspace
        let entries = h.listar_directorio(".").unwrap();
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_agp_telemetria_herramientas() {
        let claw = NexusClawPro::new_empty();
        let h = HerramientasNativas::new(claw);

        h.registrar_telemetria_herramienta("leer_archivo", true, 12);
        h.registrar_telemetria_herramienta("leer_archivo", true, 15);
        h.registrar_telemetria_herramienta("leer_archivo", false, 100);

        let tasa = h.obtener_tasa_exito("leer_archivo");
        assert!((tasa - 0.666).abs() < 0.05);
    }
}
