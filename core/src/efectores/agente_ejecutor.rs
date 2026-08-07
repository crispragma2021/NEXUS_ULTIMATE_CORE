// ==========================================
// MOTOR DE EJECUCIÓN AUTÓNOMO - AGENTE EJECUTOR
// ==========================================

use crate::efectores::nexus_claw_pro::NexusClawPro;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolResponse {
    pub success: bool,
    pub output: String,
}

pub struct AgenteEjecutor {
    claw: NexusClawPro,
    workspace_root: PathBuf,
}

impl AgenteEjecutor {
    pub fn new(claw: NexusClawPro) -> Self {
        let workspace_root = crate::infra::paths::resolve_path("");
        Self {
            claw,
            workspace_root,
        }
    }

    /// Valida que una ruta esté dentro del workspace seguro
    fn validar_ruta(&self, ruta: &str) -> Result<PathBuf> {
        let path = Path::new(ruta);
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.workspace_root.join(path)
        };

        // Canonicalizar para resolver symlinks y evitar path traversal
        let canonical = match absolute_path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // Si el archivo no existe, verificamos el ancestro que sí exista
                let mut current = absolute_path.clone();
                while let Some(parent) = current.parent() {
                    if parent.exists() {
                        if let Ok(canon_parent) = parent.canonicalize() {
                            if canon_parent.starts_with(&self.workspace_root) {
                                return Ok(absolute_path);
                            }
                        }
                        break;
                    }
                    current = parent.to_path_buf();
                }
                return Err(anyhow!("Ruta insegura o fuera del workspace"));
            }
        };

        if canonical.starts_with(&self.workspace_root) {
            Ok(absolute_path)
        } else {
            Err(anyhow!(
                "Acceso denegado: La ruta está fuera del workspace de NEXUS"
            ))
        }
    }

    pub async fn leer_archivo(&self, ruta: &str) -> Result<String> {
        let path = self.validar_ruta(ruta)?;
        let content = std::fs::read_to_string(&path)?;
        Ok(content)
    }

    pub async fn escribir_archivo(&self, ruta: &str, contenido: &str) -> Result<String> {
        let path = self.validar_ruta(ruta)?;
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mut backup_path = None;
        let mut warning = String::new();

        // Obtener line count original si el archivo existe
        let line_count_original = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .map(|c| c.lines().count())
        } else {
            None
        };

        let line_count_nuevo = contenido.lines().count();

        // ── 1. Backup rotativo con timestamp ──────────────────────────
        if path.exists() {
            let backup_name = format!("{}.bak.{}", file_name, timestamp);
            let backup_path_buf = path.with_file_name(&backup_name);
            let _ = std::fs::copy(&path, &backup_path_buf);
            backup_path = Some(backup_path_buf);

            // Limpiar backups viejos: mantener solo los últimos 5
            if let Some(parent) = path.parent() {
                let mut backups: Vec<_> = std::fs::read_dir(parent)
                    .ok()
                    .into_iter()
                    .flatten()
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_name()
                            .to_str()
                            .map(|n| n.starts_with(&file_name) && n.contains(".bak."))
                            .unwrap_or(false)
                    })
                    .collect();

                // Ordenar por fecha de modificación (más reciente primero)
                backups.sort_by(|a, b| {
                    let a_time = a.metadata().ok().and_then(|m| m.modified().ok());
                    let b_time = b.metadata().ok().and_then(|m| m.modified().ok());
                    b_time.cmp(&a_time)
                });

                // Eliminar backups excedentes (mantener últimos 5)
                for old in backups.iter().skip(5) {
                    let _ = std::fs::remove_file(old.path());
                }
            }
        }

        // ── 2. Detección de truncado anómalo ──────────────────────────
        if let Some(original_count) = line_count_original {
            if original_count > 0 && line_count_nuevo < (original_count as f64 * 0.3) as usize {
                let emergency_name = format!("{}.bak.EMERGENCY.{}", file_name, timestamp);
                let emergency_path = path.with_file_name(&emergency_name);
                let _ = std::fs::copy(&path, &emergency_path);
                warning = format!(
                    " ⚠️ WARNING: Truncado anómalo ({}→{} líneas, {:.0}% original). Emergency backup: {}",
                    original_count,
                    line_count_nuevo,
                    (line_count_nuevo as f64 / original_count as f64) * 100.0,
                    emergency_path.display()
                );
            }
        }

        // ── 3. Snapshot git automático si >200 líneas ────────────────
        if let Some(original_count) = line_count_original {
            if original_count > 200 {
                let snapshot_msg = format!("pre-escritura {}", ruta);
                let _ = std::process::Command::new("./scripts/nexus_snapshot.sh")
                    .arg(&snapshot_msg)
                    .output();
            }
        }

        // Crear directorios si es necesario
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Escribir archivo
        std::fs::write(&path, contenido)?;

        // ── 4. Output con estadísticas ────────────────────────────────
        let antes = line_count_original
            .map(|c| c.to_string())
            .unwrap_or_else(|| "NUEVO".to_string());
        let backup_str = backup_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "ninguno".to_string());

        Ok(format!(
            "✅ Escrito {} | {}→{} líneas | .bak: {}{}",
            path.display(),
            antes,
            line_count_nuevo,
            backup_str,
            warning
        ))
    }

    pub async fn buscar_codigo_regex(&self, query: &str) -> Result<String> {
        // Ejecución nativa utilizando ripgrep si está instalado
        let output = std::process::Command::new("rg")
            .args([
                "--line-number",
                "--column",
                query,
                &self.workspace_root.to_string_lossy(),
            ])
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Ok(format!("No se encontraron coincidencias para: {}", query))
        }
    }

    pub async fn ejecutar_comando_seguro(&self, comando: &str) -> Result<String> {
        // Ejecutar a través de NexusClawPro que tiene la validación de JuicioSoberano
        self.claw.ejecutar_inteligente(comando).await
    }

    pub async fn resolver_herramienta(&self, call: ToolCall) -> ToolResponse {
        let res = match call.name.as_str() {
            "leer_archivo" => {
                let path = call
                    .arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                self.leer_archivo(path).await
            }
            "escribir_archivo" => {
                let path = call
                    .arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let content = call
                    .arguments
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                self.escribir_archivo(path, content).await
            }
            "buscar_codigo_regex" => {
                let query = call
                    .arguments
                    .get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                self.buscar_codigo_regex(query).await
            }
            "ejecutar_comando" => {
                let command = call
                    .arguments
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                self.ejecutar_comando_seguro(command).await
            }
            _ => Err(anyhow!("Herramienta desconocida: {}", call.name)),
        };

        match res {
            Ok(output) => ToolResponse {
                success: true,
                output,
            },
            Err(e) => ToolResponse {
                success: false,
                output: e.to_string(),
            },
        }
    }
}
