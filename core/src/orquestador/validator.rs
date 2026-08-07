// 🔱 DETERMINISTIC VALIDATOR — Validador y parser estricto determinista
// Valida la salida JSON del SLM, verifica la estructura y previene Path Traversal.
// Puro Rust, sin unwrap(), sin expect(), listo para producción.

use crate::efectores::agente_ejecutor::ToolCall;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum ValidationResult {
    Valid(ToolCall),
    InvalidJson(String),
    InvalidAction(String),
    InvalidParams(String),
    InvalidPath(String),
}

pub struct Validator {
    pub allowed_actions: Vec<String>,
    pub workspace_root: PathBuf,
}

impl Validator {
    /// Crea un validador con la configuración de seguridad de NEXUS
    pub fn new(allowed_actions: Vec<String>) -> Self {
        let workspace_root = crate::infra::paths::resolve_path("");
        Self {
            allowed_actions,
            workspace_root,
        }
    }

    /// Valida de manera determinista la respuesta del SLM
    pub fn validate(&self, raw_output: &str) -> ValidationResult {
        // 1. Eliminar cualquier residuo de markdown para robustez en el parseo
        let cleaned = Self::clean_markdown_json(raw_output);

        // 2. Parseo JSON
        let parsed: serde_json::Value = match serde_json::from_str(&cleaned) {
            Ok(val) => val,
            Err(e) => return ValidationResult::InvalidJson(format!("JSON inválido: {}", e)),
        };

        // 3. Validar campo "action"
        let action = match parsed.get("action").and_then(|a| a.as_str()) {
            Some(act) => act.to_string(),
            None => return ValidationResult::InvalidAction("Falta el campo obligatorio 'action'".into()),
        };

        if !self.allowed_actions.contains(&action) {
            return ValidationResult::InvalidAction(format!("Acción '{}' no se encuentra en la whitelist", action));
        }

        // 4. Validar campo "params"
        let params = match parsed.get("params") {
            Some(p) if p.is_object() => p,
            _ => return ValidationResult::InvalidParams("Falta el objeto obligatorio 'params'".into()),
        };

        // 5. Validar "target" (el destino de la acción)
        let target = match params.get("target").and_then(|t| t.as_str()) {
            Some(t) => t,
            None => return ValidationResult::InvalidParams("Falta el parámetro 'target' dentro de params".into()),
        };

        // 6. Validar seguridad de rutas para acciones que accedan al Filesystem
        if action == "read_file" || action == "write_file" || action == "list_dir" {
            if !self.is_path_safe(target) {
                return ValidationResult::InvalidPath(format!("Acceso denegado: El target '{}' está fuera del workspace", target));
            }
        }

        ValidationResult::Valid(ToolCall {
            name: action,
            arguments: params.clone(),
        })
    }

    /// Limpia bloques de código markdown si el SLM los incluyó por error (ej: ```json ... ```)
    fn clean_markdown_json(input: &str) -> String {
        let trimmed = input.trim();
        if trimmed.starts_with("```") {
            let mut lines = trimmed.lines();
            // Descartar primera línea (```json o ```)
            lines.next();
            let mut cleaned = String::new();
            for line in lines {
                if line.trim().starts_with("```") {
                    break; // Fin del bloque
                }
                cleaned.push_str(line);
                cleaned.push('\n');
            }
            cleaned.trim().to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Verifica de manera hermética que una ruta esté dentro del workspace seguro
    pub fn is_path_safe(&self, path_str: &str) -> bool {
        let path = Path::new(path_str);
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.workspace_root.join(path)
        };

        // Intentar canonicalizar. Si el archivo no existe, verificar la ruta de sus ancestros existentes
        match absolute_path.canonicalize() {
            Ok(canon) => canon.starts_with(&self.workspace_root),
            Err(_) => {
                let mut current = absolute_path.clone();
                while let Some(parent) = current.parent() {
                    if parent.exists() {
                        if let Ok(canon_parent) = parent.canonicalize() {
                            return canon_parent.starts_with(&self.workspace_root);
                        }
                        break;
                    }
                    current = parent.to_path_buf();
                }
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_with_json_and_markdown() {
        let validator = Validator::new(vec!["read_file".to_string(), "write_file".to_string()]);

        // JSON normal
        let raw = r#"{"action": "read_file", "params": {"target": "Cargo.toml"}}"#;
        let res = validator.validate(raw);
        assert!(matches!(res, ValidationResult::Valid(_)));

        // JSON envuelto en Markdown ```json
        let raw_md = r#"```json
{
  "action": "write_file",
  "params": {
    "target": "tmp.txt",
    "payload": "hola"
  }
}
```"#;
        let res_md = validator.validate(raw_md);
        assert!(matches!(res_md, ValidationResult::Valid(_)));
    }

    #[test]
    fn test_validator_path_traversal_detection() {
        let validator = Validator::new(vec!["read_file".to_string()]);

        // Intento de Path Traversal
        let traversal = r#"{"action": "read_file", "params": {"target": "../../../etc/passwd"}}"#;
        let res = validator.validate(traversal);
        assert!(matches!(res, ValidationResult::InvalidPath(_)));
    }
}
