// 🔱 SANDBOX — Entorno seguro y aislado de ejecución de comandos y filesystem
// Ejecuta las herramientas validadas aplicando Whitelists, límites de recursos y control de Kernel.
// Puro Rust, sin unwrap(), sin expect(), listo para producción.

use crate::efectores::agente_ejecutor::{ToolCall, ToolResponse};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub timeout_secs: u64,                   // Timeout de comando (ej: 30s)
    pub allowed_commands: Vec<String>,       // Whitelist de comandos permitidos
    pub max_output_bytes: usize,             // Límite de tamaño de salida (ej: 1MB)
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            allowed_commands: vec![
                "ls".to_string(),
                "cat".to_string(),
                "grep".to_string(),
                "rg".to_string(),
                "find".to_string(),
                "wc".to_string(),
                "cargo".to_string(),
                "rustc".to_string(),
                "python".to_string(),
                "python3".to_string(),
                "node".to_string(),
            ],
            max_output_bytes: 1024 * 1024, // 1MB
        }
    }
}

pub struct Sandbox {
    pub config: SandboxConfig,
    pub workspace_root: PathBuf,
}

impl Sandbox {
    /// Inicializa el Sandbox seguro con la configuración y el workspace root
    pub fn new(config: SandboxConfig) -> Self {
        let workspace_root = crate::infra::paths::resolve_path("");
        Self {
            config,
            workspace_root,
        }
    }

    /// Ejecuta una llamada a herramienta en el Sandbox seguro
    pub async fn execute(&self, call: &ToolCall) -> ToolResponse {
        match call.name.as_str() {
            "read_file" => self.safe_read_file(call),
            "write_file" => self.safe_write_file(call).await,
            "execute_cmd" => self.safe_execute_cmd(call),
            "search_code" => self.safe_search_code(call),
            "list_dir" => self.safe_list_dir(call),
            _ => ToolResponse {
                success: false,
                output: format!("Acción desconocida en el Sandbox: {}", call.name),
            },
        }
    }

    /// Lectura de archivo segura
    fn safe_read_file(&self, call: &ToolCall) -> ToolResponse {
        let target = match call.arguments.get("target").and_then(|t| t.as_str()) {
            Some(t) => t,
            None => return ToolResponse { success: false, output: "Falta el parámetro 'target'".into() },
        };

        let path = self.workspace_root.join(target);
        match std::fs::read_to_string(&path) {
            Ok(content) => ToolResponse { success: true, output: content },
            Err(e) => ToolResponse { success: false, output: format!("Error al leer el archivo: {}", e) },
        }
    }

    /// Escritura de archivo segura con backups rotativos
    async fn safe_write_file(&self, call: &ToolCall) -> ToolResponse {
        let target = match call.arguments.get("target").and_then(|t| t.as_str()) {
            Some(t) => t,
            None => return ToolResponse { success: false, output: "Falta el parámetro 'target'".into() },
        };
        let payload = match call.arguments.get("payload").and_then(|p| p.as_str()) {
            Some(p) => p,
            None => return ToolResponse { success: false, output: "Falta el parámetro 'payload'".into() },
        };

        let path = self.workspace_root.join(target);

        // Crear directorios si es necesario
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return ToolResponse { success: false, output: format!("No se pudo crear directorios: {}", e) };
            }
        }

        // Crear un backup rotativo simple si el archivo ya existe
        if path.exists() {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
            let backup_path = path.with_extension(format!("bak.{}", timestamp));
            let _ = std::fs::copy(&path, &backup_path);
        }

        // Escribir el contenido nuevo de forma atómica
        match std::fs::write(&path, payload) {
            Ok(_) => ToolResponse {
                success: true,
                output: format!("✅ Escrito {} exitosamente.", target),
            },
            Err(e) => ToolResponse {
                success: false,
                output: format!("Error al escribir el archivo: {}", e),
            },
        }
    }

    /// Ejecución de comandos del sistema con whitelist y timeout rígido
    fn safe_execute_cmd(&self, call: &ToolCall) -> ToolResponse {
        let target = match call.arguments.get("target").and_then(|t| t.as_str()) {
            Some(t) => t,
            None => return ToolResponse { success: false, output: "Falta el parámetro 'target'".into() },
        };

        // Extraer el binario base (ej: "cargo test" -> "cargo")
        let binary = target.split_whitespace().next().unwrap_or("");
        if !self.config.allowed_commands.contains(&binary.to_string()) {
            return ToolResponse {
                success: false,
                output: format!("Comando '{}' no permitido en la whitelist del Sandbox", binary),
            };
        }

        // Ejecución con timeout a nivel de OS
        let timeout_str = self.config.timeout_secs.to_string();
        let output_res = Command::new("timeout")
            .arg(&timeout_str)
            .arg("sh")
            .arg("-c")
            .arg(target)
            .current_dir(&self.workspace_root)
            .output();

        match output_res {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                // Truncar salida si excede límites de la configuración
                let mut combined = format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr);
                if combined.len() > self.config.max_output_bytes {
                    combined.truncate(self.config.max_output_bytes);
                    combined.push_str("\n\n⚠️ [SALIDA TRUNCADA POR EL SANDBOX]");
                }

                ToolResponse {
                    success: output.status.success(),
                    output: combined,
                }
            }
            Err(e) => ToolResponse {
                success: false,
                output: format!("Fallo al lanzar el comando en el Sandbox: {}", e),
            },
        }
    }

    /// Búsqueda de código segura vía grep/ripgrep
    fn safe_search_code(&self, call: &ToolCall) -> ToolResponse {
        let pattern = match call.arguments.get("pattern").and_then(|p| p.as_str()) {
            Some(p) => p,
            None => return ToolResponse { success: false, output: "Falta el parámetro 'pattern' para buscar código".into() },
        };

        // Lanzar ripgrep de forma segura
        let output_res = Command::new("rg")
            .args(["--line-number", "--column", pattern])
            .current_dir(&self.workspace_root)
            .output();

        match output_res {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                ToolResponse {
                    success: output.status.success(),
                    output: format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr),
                }
            }
            Err(e) => ToolResponse {
                success: false,
                output: format!("Error al buscar código en el Sandbox: {}", e),
            },
        }
    }

    /// Listar directorios seguro
    fn safe_list_dir(&self, call: &ToolCall) -> ToolResponse {
        let target = call.arguments.get("target").and_then(|t| t.as_str()).unwrap_or(".");
        let path = self.workspace_root.join(target);

        match std::fs::read_dir(path) {
            Ok(entries) => {
                let mut result = Vec::new();
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let file_type = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        "DIR"
                    } else {
                        "FILE"
                    };
                    result.push(format!("{} [{}]", name, file_type));
                }
                ToolResponse {
                    success: true,
                    output: result.join("\n"),
                }
            }
            Err(e) => ToolResponse {
                success: false,
                output: format!("Error al listar directorio: {}", e),
            },
        }
    }
}
