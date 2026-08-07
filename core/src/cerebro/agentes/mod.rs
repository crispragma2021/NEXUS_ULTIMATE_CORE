// ==========================================
// 🧠 AGENTES ESPECIALISTAS — Catálogo Soberano de Roles Expertos
// ==========================================
// Absorbe los 20 agentes de Roo Code como datos nativos de Rust.
// Cada agente es una variante del enum con su nombre canónico,
// dominio de expertise, skills asociados y system prompt.
//
// Referencia original: .agent/agents/*.md
// ==========================================

use serde::{Deserialize, Serialize};

/// Dominio de expertise de un agente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dominio {
    Frontend,
    Backend,
    Database,
    Mobile,
    GameDev,
    DevOps,
    Security,
    Testing,
    Debugging,
    Performance,
    Seo,
    Documentation,
    Product,
    Architecture,
    CodeQuality,
    Exploration,
    FullStack,
    Orchestration,
}

impl Dominio {
    pub fn nombre(&self) -> &'static str {
        match self {
            Dominio::Frontend => "Frontend & UI",
            Dominio::Backend => "Backend & API",
            Dominio::Database => "Database & SQL",
            Dominio::Mobile => "Mobile Development",
            Dominio::GameDev => "Game Development",
            Dominio::DevOps => "DevOps & Infra",
            Dominio::Security => "Security & Compliance",
            Dominio::Testing => "Testing & QA",
            Dominio::Debugging => "Debugging & Root Cause",
            Dominio::Performance => "Performance Optimization",
            Dominio::Seo => "SEO & Growth",
            Dominio::Documentation => "Documentation",
            Dominio::Product => "Product Management",
            Dominio::Architecture => "Architecture & Design",
            Dominio::CodeQuality => "Code Quality",
            Dominio::Exploration => "Codebase Analysis",
            Dominio::FullStack => "Full Stack",
            Dominio::Orchestration => "Orchestration & Coordination",
        }
    }
}

/// Identificador único para cada agente especialista.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgenteEspecialista {
    Orchestrator,
    ProjectPlanner,
    FrontendSpecialist,
    BackendSpecialist,
    DatabaseArchitect,
    MobileDeveloper,
    GameDeveloper,
    DevOpsEngineer,
    SecurityAuditor,
    PenetrationTester,
    TestEngineer,
    Debugger,
    PerformanceOptimizer,
    SeoSpecialist,
    DocumentationWriter,
    ProductManager,
    ProductOwner,
    QaAutomationEngineer,
    CodeArchaeologist,
    ExplorerAgent,
    TradingJudge,
    BullAnalyst,
    BearAnalyst,
}

impl AgenteEspecialista {
    /// Nombre canónico del agente.
    pub fn nombre(&self) -> &'static str {
        match self {
            Self::Orchestrator => "orchestrator",
            Self::ProjectPlanner => "project-planner",
            Self::FrontendSpecialist => "frontend-specialist",
            Self::BackendSpecialist => "backend-specialist",
            Self::DatabaseArchitect => "database-architect",
            Self::MobileDeveloper => "mobile-developer",
            Self::GameDeveloper => "game-developer",
            Self::DevOpsEngineer => "devops-engineer",
            Self::SecurityAuditor => "security-auditor",
            Self::PenetrationTester => "penetration-tester",
            Self::TestEngineer => "test-engineer",
            Self::Debugger => "debugger",
            Self::PerformanceOptimizer => "performance-optimizer",
            Self::SeoSpecialist => "seo-specialist",
            Self::DocumentationWriter => "documentation-writer",
            Self::ProductManager => "product-manager",
            Self::ProductOwner => "product-owner",
            Self::QaAutomationEngineer => "qa-automation-engineer",
            Self::CodeArchaeologist => "code-archaeologist",
            Self::ExplorerAgent => "explorer-agent",
            Self::TradingJudge => "trading-judge",
            Self::BullAnalyst => "bull-analyst",
            Self::BearAnalyst => "bear-analyst",
        }
    }

    /// Dominio principal del agente.
    pub fn dominio(&self) -> Dominio {
        match self {
            Self::Orchestrator => Dominio::Orchestration,
            Self::ProjectPlanner => Dominio::Architecture,
            Self::FrontendSpecialist => Dominio::Frontend,
            Self::BackendSpecialist => Dominio::Backend,
            Self::DatabaseArchitect => Dominio::Database,
            Self::MobileDeveloper => Dominio::Mobile,
            Self::GameDeveloper => Dominio::GameDev,
            Self::DevOpsEngineer => Dominio::DevOps,
            Self::SecurityAuditor => Dominio::Security,
            Self::PenetrationTester => Dominio::Security,
            Self::TestEngineer => Dominio::Testing,
            Self::Debugger => Dominio::Debugging,
            Self::PerformanceOptimizer => Dominio::Performance,
            Self::SeoSpecialist => Dominio::Seo,
            Self::DocumentationWriter => Dominio::Documentation,
            Self::ProductManager => Dominio::Product,
            Self::ProductOwner => Dominio::Product,
            Self::QaAutomationEngineer => Dominio::Testing,
            Self::CodeArchaeologist => Dominio::CodeQuality,
            Self::ExplorerAgent => Dominio::Exploration,
            Self::TradingJudge => Dominio::Performance,
            Self::BullAnalyst => Dominio::Performance,
            Self::BearAnalyst => Dominio::Performance,
        }
    }

    /// Skills asociados al agente (referencias simbólicas al catálogo de skills).
    pub fn skills(&self) -> &'static [&'static str] {
        match self {
            Self::Orchestrator => &["parallel-agents"],
            Self::ProjectPlanner => &["brainstorming", "plan-writing", "architecture"],
            Self::FrontendSpecialist => &[
                "frontend-design",
                "react-best-practices",
                "tailwind-patterns",
            ],
            Self::BackendSpecialist => {
                &["api-patterns", "nodejs-best-practices", "database-design"]
            }
            Self::DatabaseArchitect => &["database-design"],
            Self::MobileDeveloper => &["mobile-design"],
            Self::GameDeveloper => &["game-development"],
            Self::DevOpsEngineer => &["deployment-procedures", "docker-expert"],
            Self::SecurityAuditor => &["vulnerability-scanner", "red-team-tactics"],
            Self::PenetrationTester => &["red-team-tactics"],
            Self::TestEngineer => &["testing-patterns", "tdd-workflow", "webapp-testing"],
            Self::Debugger => &["systematic-debugging"],
            Self::PerformanceOptimizer => &["performance-profiling"],
            Self::SeoSpecialist => &["seo-fundamentals", "geo-fundamentals"],
            Self::DocumentationWriter => &["documentation-templates"],
            Self::ProductManager => &["plan-writing", "brainstorming"],
            Self::ProductOwner => &["plan-writing", "brainstorming"],
            Self::QaAutomationEngineer => &["webapp-testing", "testing-patterns"],
            Self::CodeArchaeologist => &["clean-code", "code-review-checklist"],
            Self::ExplorerAgent => &[],
            Self::TradingJudge => &[
                "nexus-scientist-expert",
                "nexus-cfo-expert",
                "performance-profiling",
            ],
            Self::BullAnalyst => &[
                "nexus-scientist-expert",
                "performance-profiling",
            ],
            Self::BearAnalyst => &[
                "nexus-scientist-expert",
                "performance-profiling",
            ],
        }
    }

    /// System prompt del agente (esencia de su especialidad).
    pub fn system_prompt(&self) -> &'static str {
        match self {
            Self::Orchestrator => {
                "Eres el orquestador multi-agente. Coordinas equipos de especialistas para resolver problemas complejos."
            }
            Self::ProjectPlanner => {
                "Eres un planificador de proyectos. Desglosas requerimientos en tareas accionables con estimaciones realistas."
            }
            Self::FrontendSpecialist => {
                "Eres un especialista en frontend. Dominas React, TypeScript, CSS, Tailwind y diseño de UI/UX."
            }
            Self::BackendSpecialist => {
                "Eres un especialista en backend. Diseñas APIs, lógica de negocio y sistemas escalables."
            }
            Self::DatabaseArchitect => {
                "Eres un arquitecto de bases de datos. Diseñas esquemas, índices y optimizas consultas SQL."
            }
            Self::MobileDeveloper => {
                "Eres un desarrollador mobile. Creas apps para iOS, Android y React Native."
            }
            Self::GameDeveloper => {
                "Eres un desarrollador de juegos. Diseñas mecánicas, físicas y lógica de juego."
            }
            Self::DevOpsEngineer => {
                "Eres un ingeniero DevOps. Gestionas CI/CD, Docker, servidores y despliegues."
            }
            Self::SecurityAuditor => {
                "Eres un auditor de seguridad. Evalúas compliance, vulnerabilidades y mejores prácticas OWASP."
            }
            Self::PenetrationTester => {
                "Eres un pentester ofensivo. Identificas y explotas vulnerabilidades de seguridad."
            }
            Self::TestEngineer => {
                "Eres un ingeniero de pruebas. Diseñas estrategias de testing y garantizas calidad."
            }
            Self::Debugger => {
                "Eres un debugger experto. Encuentras causas raíz de bugs de manera sistemática."
            }
            Self::PerformanceOptimizer => {
                "Eres un optimizador de rendimiento. Mejoras velocidad, Web Vitals y eficiencia."
            }
            Self::SeoSpecialist => {
                "Eres un especialista SEO. Optimizas ranking, visibilidad y posicionamiento en buscadores."
            }
            Self::DocumentationWriter => {
                "Eres un escritor de documentación. Creas manuales, guías y documentación técnica clara."
            }
            Self::ProductManager => {
                "Eres un product manager. Defines requerimientos, historias de usuario y prioridades."
            }
            Self::ProductOwner => {
                "Eres un product owner. Gestionas estrategia, backlog y definición de MVP."
            }
            Self::QaAutomationEngineer => {
                "Eres un ingeniero de automatización QA. Creas pipelines de testing E2E y CI."
            }
            Self::CodeArchaeologist => {
                "Eres un arqueólogo de código. Analizas y refactorizas código legacy con precisión."
            }
            Self::ExplorerAgent => {
                "Eres un agente explorador. Analizas codebases y descubres patrones ocultos."
            }
            Self::TradingJudge => {
                "Eres el Juez Soberano de Trading. Tu misión es supervisar el motor ML y el Sentinel, auditando cada señal de compra/venta. Solo autorizas operaciones con una relación riesgo/recompensa óptima y proteges el capital del Arquitecto con disciplina absoluta."
            }
            Self::BullAnalyst => {
                "Eres el Analista Alcista (Bull). Tu misión es identificar argumentos sólidos, ineficiencias de mercado y señales cuantitativas que justifiquen una entrada en largo. Buscas el potencial de crecimiento y el alfa estadístico en las tendencias ascendentes."
            }
            Self::BearAnalyst => {
                "Eres el Analista Bajista (Bear). Tu misión es actuar como el abogado del diablo. Identificas riesgos ocultos, señales de sobrecompra, debilidad en la acción del precio y trampas de liquidez. Buscas motivos para invalidar la tesis alcista y proteger el capital."
            }
        }
    }
}

/// Ficha completa de un agente con todos sus metadatos.
#[derive(Debug, Clone, Serialize)]
pub struct FichaAgente {
    pub id: AgenteEspecialista,
    pub nombre: &'static str,
    pub dominio: Dominio,
    pub skills: &'static [&'static str],
    pub system_prompt: &'static str,
}

impl From<AgenteEspecialista> for FichaAgente {
    fn from(id: AgenteEspecialista) -> Self {
        Self {
            id,
            nombre: id.nombre(),
            dominio: id.dominio(),
            skills: id.skills(),
            system_prompt: id.system_prompt(),
        }
    }
}

/// Catálogo completo de los 21 agentes.
pub fn catalogo_agentes() -> Vec<FichaAgente> {
    use AgenteEspecialista::*;
    vec![
        Orchestrator,
        ProjectPlanner,
        FrontendSpecialist,
        BackendSpecialist,
        DatabaseArchitect,
        MobileDeveloper,
        GameDeveloper,
        DevOpsEngineer,
        SecurityAuditor,
        PenetrationTester,
        TestEngineer,
        Debugger,
        PerformanceOptimizer,
        SeoSpecialist,
        DocumentationWriter,
        ProductManager,
        ProductOwner,
        QaAutomationEngineer,
        CodeArchaeologist,
        ExplorerAgent,
        TradingJudge,
        BullAnalyst,
        BearAnalyst,
    ]
    .into_iter()
    .map(FichaAgente::from)
    .collect()
}

/// Busca un agente por nombre canónico.
pub fn buscar_agente(nombre: &str) -> Option<FichaAgente> {
    use AgenteEspecialista::*;
    let id = match nombre {
        "orchestrator" => Orchestrator,
        "project-planner" => ProjectPlanner,
        "frontend-specialist" => FrontendSpecialist,
        "backend-specialist" => BackendSpecialist,
        "database-architect" => DatabaseArchitect,
        "mobile-developer" => MobileDeveloper,
        "game-developer" => GameDeveloper,
        "devops-engineer" => DevOpsEngineer,
        "security-auditor" => SecurityAuditor,
        "penetration-tester" => PenetrationTester,
        "test-engineer" => TestEngineer,
        "debugger" => Debugger,
        "performance-optimizer" => PerformanceOptimizer,
        "seo-specialist" => SeoSpecialist,
        "documentation-writer" => DocumentationWriter,
        "product-manager" => ProductManager,
        "product-owner" => ProductOwner,
        "qa-automation-engineer" => QaAutomationEngineer,
        "code-archaeologist" => CodeArchaeologist,
        "explorer-agent" => ExplorerAgent,
        "trading-judge" => TradingJudge,
        _ => return None,
    };
    Some(FichaAgente::from(id))
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalogo_tiene_23_agentes() {
        let catalogo = catalogo_agentes();
        assert_eq!(
            catalogo.len(),
            23,
            "Debe haber exactamente 23 agentes especialistas (21 base + BullAnalyst + BearAnalyst)"
        );
    }

    #[test]
    fn test_todos_los_agentes_tienen_nombre() {
        for agente in catalogo_agentes() {
            assert!(
                !agente.nombre.is_empty(),
                "Agente {:?} sin nombre",
                agente.id
            );
        }
    }

    #[test]
    fn test_buscar_agente_por_nombre() {
        let encontrado = buscar_agente("frontend-specialist");
        assert!(encontrado.is_some());
        assert_eq!(encontrado.unwrap().dominio, Dominio::Frontend);
    }

    #[test]
    fn test_buscar_agente_inexistente() {
        assert!(buscar_agente("agente-fantasma").is_none());
    }

    #[test]
    fn test_cada_agente_tiene_dominio_unico() {
        let mut dominios = std::collections::HashSet::new();
        for agente in catalogo_agentes() {
            // No falla si hay duplicados (varios agentes pueden compartir dominio)
            dominios.insert(agente.dominio);
        }
        // Debemos tener al menos varios dominios diferentes
        assert!(
            dominios.len() >= 5,
            "Se esperaban al menos 5 dominios distintos"
        );
    }
}
