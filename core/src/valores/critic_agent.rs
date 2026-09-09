// ==========================================
// CRITIC AGENT OMEGA - Auto-Auditoría Arquitectónica
// ==========================================
// Migrado de legacy/nexus-orquestador/src/autonomia/critic.rs
//
// El CriticAgent permite a NEXUS auditar sus propias soluciones,
// comparándolas contra los pilares arquitectónicos antes de ejecutarlas.
// ==========================================

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Resultado de una auditoría
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditResult {
    /// Si la acción/solución está aprobada
    pub approved: bool,
    /// Razonamiento detrás de la decisión
    pub reasoning: String,
    /// Sugerencias de mejora
    pub suggestions: Vec<String>,
}

impl AuditResult {
    /// Auditoría automática (sin IA) basada en reglas
    pub fn auto_audit(solution: &str, context: &str) -> Self {
        let mut suggestions = Vec::new();
        let mut approved = true;
        let mut reasoning = String::new();

        // Regla 1: Verificar eficiencia de CPU (Pilar 1)
        if solution.contains("clone()") && solution.matches("clone()").count() > 3 {
            suggestions.push("Demasiados clone(). Usar Arc o referencias compartidas.".to_string());
            reasoning.push_str("Alto uso de clone() detectado. ");
        }

        // Regla 2: Detectar dependencias externas peligrosas
        let dangerous_deps = ["unsafe", "std::process::Command", "fs::remove_dir_all"];
        for dep in &dangerous_deps {
            if solution.contains(dep) {
                approved = false;
                suggestions.push(format!(
                    "Uso de {} requiere verificación manual del Arquitecto.",
                    dep
                ));
                reasoning.push_str(&format!("Código peligroso ({}) detectado. ", dep));
            }
        }

        // Regla 3: Verificar manejo de errores
        if solution.contains(".unwrap()") && !solution.contains("expect(") {
            suggestions.push("Reemplazar .unwrap() con manejo de errores explícito.".to_string());
            reasoning.push_str("Uso excesivo de unwrap(). ");
        }

        // Regla 4: Verificar contexto
        if context.contains("producción") && solution.len() < 50 {
            approved = false;
            reasoning.push_str("Solución demasiado corta para producción. ");
            suggestions.push("Expandir la solución con manejo de errores y logging.".to_string());
        }

        if reasoning.is_empty() {
            reasoning = "Auditoría automática: sin problemas detectados.".to_string();
        }

        if approved {
            info!("✅ [CRITIC] Auditoría automática APROBADA: {}", reasoning);
        } else {
            warn!("⚠️ [CRITIC] Auditoría automática RECHAZADA: {}", reasoning);
        }

        AuditResult {
            approved,
            reasoning: reasoning.trim().to_string(),
            suggestions,
        }
    }

    /// Verifica si una acción respeta los 5 Pilares de NEXUS
    pub fn check_pillars(action: &str) -> Vec<String> {
        let mut violations = Vec::new();

        // Pilar 13: No degradar el núcleo
        if action.contains("degradar")
            || action.contains("borrar nucleo")
            || action.contains("delete_root")
        {
            violations.push("Pilar 13: Intento de degradación del núcleo.".to_string());
        }

        // Pilar 8: Protección de propiedad intelectual
        if (action.contains("cliente") || action.contains("externo")) && action.contains("acceder")
        {
            violations
                .push("Pilar 8: Intento de acceso no autorizado a datos privados.".to_string());
        }

        // Protección de hardware
        if action.contains("overclock") {
            violations.push("Pilar HW: Riesgo letal para el Intel i7-12700F.".to_string());
        }

        violations
    }
}

/// Auditor programático de soluciones (sin dependencia de LLM)
pub struct CriticAgent;

impl CriticAgent {
    pub fn new() -> Self {
        info!("🔍 [CRITIC] Agente de Auto-Auditoría activado");
        Self
    }

    /// Audita una solución contra los pilares arquitectónicos
    pub fn audit_solution(&self, solution: &str, context: &str) -> AuditResult {
        AuditResult::auto_audit(solution, context)
    }

    /// Verifica si una propuesta de migración es segura
    pub fn check_migration(&self, from: &str, to: &str) -> AuditResult {
        let mut suggestions = Vec::new();
        let mut reasoning = String::new();

        // Verificar que el destino existe
        if to.contains("..") || to.contains("~") {
            suggestions.push("Usar paths absolutos o relativos seguros.".to_string());
            reasoning.push_str("Path inseguro detectado. ");
        }

        // Verificar que no estamos migrando sobre algo superior
        if from.contains("legacy") && to.contains("core") {
            reasoning.push_str("Migración de legacy a core. ");
        }

        if reasoning.is_empty() {
            reasoning = "Migración verificada: sin problemas.".to_string();
        }

        AuditResult {
            approved: true,
            reasoning,
            suggestions,
        }
    }
}

impl Default for CriticAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_audit_approves_good_code() {
        let result = AuditResult::auto_audit(
            "fn process(data: &str) -> Result<String, Error> { Ok(data.to_string()) }",
            "desarrollo",
        );
        assert!(result.approved);
    }

    #[test]
    fn test_auto_audit_rejects_unsafe_code() {
        let result = AuditResult::auto_audit(
            "std::process::Command::new(\"rm\").arg(\"-rf\").arg(\"/\")",
            "producción",
        );
        assert!(!result.approved);
    }

    #[test]
    fn test_check_pillars_rejects_degradation() {
        let violations = AuditResult::check_pillars("borrar nucleo del sistema");
        assert!(!violations.is_empty());
        assert!(violations[0].contains("Pilar 13"));
    }

    #[test]
    fn test_check_pillars_approves_normal() {
        let violations = AuditResult::check_pillars("optimizar compilación del módulo");
        assert!(violations.is_empty());
    }
}
