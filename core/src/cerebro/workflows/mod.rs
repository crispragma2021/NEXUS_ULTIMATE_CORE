// ==========================================
// 🔄 WORKFLOWS — Rutas de Ejecución del Orquestador
// ==========================================
// Absorbe los 12 slash commands del ecosistema Roo Code
// como rutas nativas del Orquestador.
//
// Referencia original: .agent/workflows/*.md
// ==========================================

use serde::Serialize;
use std::sync::OnceLock;

/// Comandos slash disponibles como workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum ComandoSlash {
    Brainstorm,
    Create,
    Debug,
    Deploy,
    Enhance,
    Orchestrate,
    Plan,
    Preview,
    Status,
    Test,
    UiUxProMax,
    SeguridadMapeo,
}

impl ComandoSlash {
    pub fn nombre(&self) -> &'static str {
        match self {
            Self::Brainstorm => "/brainstorm",
            Self::Create => "/create",
            Self::Debug => "/debug",
            Self::Deploy => "/deploy",
            Self::Enhance => "/enhance",
            Self::Orchestrate => "/orchestrate",
            Self::Plan => "/plan",
            Self::Preview => "/preview",
            Self::Status => "/status",
            Self::Test => "/test",
            Self::UiUxProMax => "/ui-ux-pro-max",
            Self::SeguridadMapeo => "/seguridad-mapeo",
        }
    }

    pub fn descripcion(&self) -> &'static str {
        match self {
            Self::Brainstorm => "Descubrimiento socrático y lluvia de ideas estructurada",
            Self::Create => "Creación de nuevas funcionalidades y componentes",
            Self::Debug => "Debuggear issues y análisis de causa raíz",
            Self::Deploy => "Despliegue de aplicación en producción",
            Self::Enhance => "Mejora de código existente con refactorización",
            Self::Orchestrate => "Coordinación multi-agente para tareas complejas",
            Self::Plan => "Desglose de tareas y planeación arquitectónica",
            Self::Preview => "Previsualización de cambios antes de aplicarlos",
            Self::Status => "Verificación del estado del proyecto y health checks",
            Self::Test => "Ejecución de tests y validación de calidad",
            Self::UiUxProMax => "Diseño de interfaces con 50 estilos predefinidos",
            Self::SeguridadMapeo => "Mapeo de seguridad y análisis de vulnerabilidades",
        }
    }

    /// Intenta parsear un comando slash desde una cadena.
    pub fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim().to_lowercase();
        // Acepta con o sin /
        let key = trimmed.strip_prefix('/').unwrap_or(&trimmed);
        match key {
            "brainstorm" => Some(Self::Brainstorm),
            "create" => Some(Self::Create),
            "debug" => Some(Self::Debug),
            "deploy" => Some(Self::Deploy),
            "enhance" => Some(Self::Enhance),
            "orchestrate" => Some(Self::Orchestrate),
            "plan" => Some(Self::Plan),
            "preview" => Some(Self::Preview),
            "status" => Some(Self::Status),
            "test" => Some(Self::Test),
            "ui-ux-pro-max" | "uiuxpromax" | "ui" => Some(Self::UiUxProMax),
            "seguridad-mapeo" | "seguridad" | "security-map" => Some(Self::SeguridadMapeo),
            _ => None,
        }
    }

    /// Agente recomendado para ejecutar este workflow.
    pub fn agente_recomendado(&self) -> &'static str {
        match self {
            Self::Brainstorm => "project-planner",
            Self::Create => "fullstack-developer",
            Self::Debug => "debugger",
            Self::Deploy => "devops-engineer",
            Self::Enhance => "code-archaeologist",
            Self::Orchestrate => "orchestrator",
            Self::Plan => "project-planner",
            Self::Preview => "frontend-specialist",
            Self::Status => "system-analyst",
            Self::Test => "test-engineer",
            Self::UiUxProMax => "frontend-specialist",
            Self::SeguridadMapeo => "security-auditor",
        }
    }
}

/// Metadatos completos de un workflow.
#[derive(Debug, Clone, Serialize)]
pub struct FichaWorkflow {
    pub comando: ComandoSlash,
    pub archivo_fuente: &'static str,
}

static CATALOGO_WORKFLOWS: OnceLock<Vec<FichaWorkflow>> = OnceLock::new();

fn init_workflows() -> &'static Vec<FichaWorkflow> {
    CATALOGO_WORKFLOWS.get_or_init(|| {
        vec![
            FichaWorkflow {
                comando: ComandoSlash::Brainstorm,
                archivo_fuente: ".agent/workflows/brainstorm.md",
            },
            FichaWorkflow {
                comando: ComandoSlash::Create,
                archivo_fuente: ".agent/workflows/create.md",
            },
            FichaWorkflow {
                comando: ComandoSlash::Debug,
                archivo_fuente: ".agent/workflows/debug.md",
            },
            FichaWorkflow {
                comando: ComandoSlash::Deploy,
                archivo_fuente: ".agent/workflows/deploy.md",
            },
            FichaWorkflow {
                comando: ComandoSlash::Enhance,
                archivo_fuente: ".agent/workflows/enhance.md",
            },
            FichaWorkflow {
                comando: ComandoSlash::Orchestrate,
                archivo_fuente: ".agent/workflows/orchestrate.md",
            },
            FichaWorkflow {
                comando: ComandoSlash::Plan,
                archivo_fuente: ".agent/workflows/plan.md",
            },
            FichaWorkflow {
                comando: ComandoSlash::Preview,
                archivo_fuente: ".agent/workflows/preview.md",
            },
            FichaWorkflow {
                comando: ComandoSlash::Status,
                archivo_fuente: ".agent/workflows/status.md",
            },
            FichaWorkflow {
                comando: ComandoSlash::Test,
                archivo_fuente: ".agent/workflows/test.md",
            },
            FichaWorkflow {
                comando: ComandoSlash::UiUxProMax,
                archivo_fuente: ".agent/workflows/ui-ux-pro-max.md",
            },
            FichaWorkflow {
                comando: ComandoSlash::SeguridadMapeo,
                archivo_fuente: ".agent/workflows/seguridad_mapeo.md",
            },
        ]
    })
}

/// Catálogo completo de workflows.
pub fn catalogo_workflows() -> &'static Vec<FichaWorkflow> {
    init_workflows()
}

/// Busca un workflow por comando.
pub fn buscar_workflow(comando: ComandoSlash) -> Option<&'static FichaWorkflow> {
    init_workflows().iter().find(|w| w.comando == comando)
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hay_12_workflows() {
        assert_eq!(
            catalogo_workflows().len(),
            12,
            "Debe haber exactamente 12 workflows"
        );
    }

    #[test]
    fn test_todos_los_comandos_tienen_nombre() {
        for w in catalogo_workflows().iter() {
            assert!(!w.comando.nombre().is_empty());
        }
    }

    #[test]
    fn test_parsear_comando_con_slash() {
        assert_eq!(
            ComandoSlash::parse("/brainstorm"),
            Some(ComandoSlash::Brainstorm)
        );
        assert_eq!(ComandoSlash::parse("/debug"), Some(ComandoSlash::Debug));
        assert_eq!(
            ComandoSlash::parse("/ui-ux-pro-max"),
            Some(ComandoSlash::UiUxProMax)
        );
    }

    #[test]
    fn test_parsear_comando_sin_slash() {
        assert_eq!(ComandoSlash::parse("plan"), Some(ComandoSlash::Plan));
        assert_eq!(ComandoSlash::parse("test"), Some(ComandoSlash::Test));
    }

    #[test]
    fn test_parsear_comando_invalido() {
        assert!(ComandoSlash::parse("/comando-fantasma").is_none());
        assert!(ComandoSlash::parse("xyz").is_none());
    }

    #[test]
    fn test_todos_los_workflows_tienen_agente_recomendado() {
        for w in catalogo_workflows().iter() {
            let agente = w.comando.agente_recomendado();
            assert!(
                !agente.is_empty(),
                "Workflow {:?} sin agente recomendado",
                w.comando
            );
        }
    }

    #[test]
    fn test_cada_comando_tiene_descripcion_unica() {
        let mut descs = std::collections::HashSet::new();
        for w in catalogo_workflows().iter() {
            assert!(
                descs.insert(w.comando.descripcion()),
                "Descripción duplicada para {:?}",
                w.comando
            );
        }
    }
}
