// ==========================================
// 🎯 SKILL CONOCIMIENTO — Catálogo Soberano de 47 Skills Reales
// ==========================================
// Absorbe todos los skills del ecosistema Roo Code y NEXUS propios
// como datos inmutables en Rust nativo.
//
// Referencia original: .agent/skills/*.md
// ==========================================

use serde::Serialize;
use std::sync::OnceLock;

/// Categoría funcional de un skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum CategoriaSkill {
    Frontend,
    Backend,
    Database,
    TypeScript,
    CloudInfra,
    Testing,
    Security,
    Architecture,
    Mobile,
    GameDev,
    Seo,
    Shell,
    Quality,
    Design,
    Productivity,
    NexusPropio,
    Otro,
}

impl CategoriaSkill {
    pub fn nombre(&self) -> &'static str {
        match self {
            Self::Frontend => "Frontend & UI",
            Self::Backend => "Backend & API",
            Self::Database => "Base de Datos",
            Self::TypeScript => "TypeScript/JavaScript",
            Self::CloudInfra => "Cloud & Infraestructura",
            Self::Testing => "Testing & Calidad",
            Self::Security => "Seguridad",
            Self::Architecture => "Arquitectura & Planeación",
            Self::Mobile => "Mobile",
            Self::GameDev => "Game Development",
            Self::Seo => "SEO & Growth",
            Self::Shell => "Shell/CLI",
            Self::Quality => "Calidad de Código",
            Self::Design => "Diseño",
            Self::Productivity => "Productividad",
            Self::NexusPropio => "NEXUS Propio",
            Self::Otro => "Otros",
        }
    }
}

/// Cada skill del catálogo.
#[derive(Debug, Clone, Serialize)]
pub struct FichaSkill {
    pub id: &'static str,
    pub categoria: CategoriaSkill,
    pub descripcion: &'static str,
    /// Path relativo al archivo original en .agent/skills/
    pub fuente: &'static str,
}

static CATALOGO: OnceLock<Vec<FichaSkill>> = OnceLock::new();

fn init_catalogo() -> &'static Vec<FichaSkill> {
    CATALOGO.get_or_init(|| {
        vec![
            // ── Frontend & UI ──
            FichaSkill {
                id: "frontend-design",
                categoria: CategoriaSkill::Frontend,
                descripcion: "Diseño y maquetación frontend con HTML/CSS/JS moderno",
                fuente: ".agent/skills/frontend-design",
            },
            FichaSkill {
                id: "web-design-guidelines",
                categoria: CategoriaSkill::Frontend,
                descripcion: "Guías de diseño web responsivo y accesible",
                fuente: ".agent/skills/web-design-guidelines",
            },
            FichaSkill {
                id: "tailwind-patterns",
                categoria: CategoriaSkill::Frontend,
                descripcion: "Patrones y utilidades con Tailwind CSS",
                fuente: ".agent/skills/tailwind-patterns",
            },
            FichaSkill {
                id: "nextjs-react-expert",
                categoria: CategoriaSkill::Frontend,
                descripcion: "Experto en Next.js, React y SSR",
                fuente: ".agent/skills/nextjs-react-expert",
            },
            FichaSkill {
                id: "ui-ux-pro-max",
                categoria: CategoriaSkill::Design,
                descripcion: "Diseño UX/UI avanzado con Figma y prototipado",
                fuente: ".agent/skills/doc.md",
            },
            // ── Backend & API ──
            FichaSkill {
                id: "api-patterns",
                categoria: CategoriaSkill::Backend,
                descripcion: "Patrones de diseño de APIs REST, GraphQL y gRPC",
                fuente: ".agent/skills/doc.md",
            },
            FichaSkill {
                id: "nodejs-best-practices",
                categoria: CategoriaSkill::Backend,
                descripcion: "Mejores prácticas Node.js para producción",
                fuente: ".agent/skills/nodejs-best-practices",
            },
            FichaSkill {
                id: "python-patterns",
                categoria: CategoriaSkill::Backend,
                descripcion: "Patrones y buenas prácticas en Python",
                fuente: ".agent/skills/python-patterns",
            },
            FichaSkill {
                id: "rust-pro",
                categoria: CategoriaSkill::Backend,
                descripcion: "Rust avanzado: unsafe, FFI, concurrencia y optimización",
                fuente: ".agent/skills/rust-pro",
            },
            // ── Database ──
            FichaSkill {
                id: "database-design",
                categoria: CategoriaSkill::Database,
                descripcion: "Diseño de esquemas SQL y NoSQL",
                fuente: ".agent/skills/database-design",
            },
            FichaSkill {
                id: "prisma-expert",
                categoria: CategoriaSkill::Database,
                descripcion: "ORM Prisma: modelos, migraciones, queries avanzadas",
                fuente: ".agent/skills/doc.md",
            },
            // ── TypeScript ──
            FichaSkill {
                id: "typescript-expert",
                categoria: CategoriaSkill::TypeScript,
                descripcion: "TypeScript avanzado: tipos genéricos, inferencia, utility types",
                fuente: ".agent/skills/doc.md",
            },
            // ── Cloud & Infra ──
            FichaSkill {
                id: "docker-expert",
                categoria: CategoriaSkill::CloudInfra,
                descripcion: "Docker: Dockerfile, compose, multi-stage builds, networking",
                fuente: ".agent/skills/doc.md",
            },
            FichaSkill {
                id: "deployment-procedures",
                categoria: CategoriaSkill::CloudInfra,
                descripcion: "Procedimientos de deploy: CI/CD, rollback, health checks",
                fuente: ".agent/skills/deployment-procedures",
            },
            FichaSkill {
                id: "server-management",
                categoria: CategoriaSkill::CloudInfra,
                descripcion: "Gestión de servidores Linux: monitoreo, hardening, backups",
                fuente: ".agent/skills/server-management",
            },
            // ── Testing & Calidad ──
            FichaSkill {
                id: "testing-patterns",
                categoria: CategoriaSkill::Testing,
                descripcion: "Patrones de testing: unitario, integración, e2e",
                fuente: ".agent/skills/testing-patterns",
            },
            FichaSkill {
                id: "webapp-testing",
                categoria: CategoriaSkill::Testing,
                descripcion: "Testing de aplicaciones web: Playwright, Cypress, Vitest",
                fuente: ".agent/skills/webapp-testing",
            },
            FichaSkill {
                id: "tdd-workflow",
                categoria: CategoriaSkill::Testing,
                descripcion: "Flujo TDD: red-green-refactor, coverage, mutation testing",
                fuente: ".agent/skills/tdd-workflow",
            },
            FichaSkill {
                id: "code-review-checklist",
                categoria: CategoriaSkill::Testing,
                descripcion: "Checklist de code review por tipo de proyecto",
                fuente: ".agent/skills/code-review-checklist",
            },
            FichaSkill {
                id: "lint-and-validate",
                categoria: CategoriaSkill::Testing,
                descripcion: "Linting y validación: ESLint, Prettier, Biome, Clippy",
                fuente: ".agent/skills/lint-and-validate",
            },
            // ── Security ──
            FichaSkill {
                id: "vulnerability-scanner",
                categoria: CategoriaSkill::Security,
                descripcion: "Escaneo de vulnerabilidades: OWASP, SAST, DAST",
                fuente: ".agent/skills/vulnerability-scanner",
            },
            FichaSkill {
                id: "red-team-tactics",
                categoria: CategoriaSkill::Security,
                descripcion: "Tácticas de red team: reconocimiento, explotación, post-explotación",
                fuente: ".agent/skills/red-team-tactics",
            },
            FichaSkill {
                id: "forensic-investigator",
                categoria: CategoriaSkill::Security,
                descripcion: "Investigación forense digital: análisis de logs, memoria y disco",
                fuente: ".agent/skills/forensic-investigator",
            },
            // ── Architecture & Planning ──
            FichaSkill {
                id: "app-builder",
                categoria: CategoriaSkill::Architecture,
                descripcion: "Constructor de aplicaciones: desde idea hasta MVP estructurado",
                fuente: ".agent/skills/app-builder",
            },
            FichaSkill {
                id: "architecture",
                categoria: CategoriaSkill::Architecture,
                descripcion: "Arquitectura de software: patrones, estilos, decisiones técnicas",
                fuente: ".agent/skills/software-architect",
            },
            FichaSkill {
                id: "plan-writing",
                categoria: CategoriaSkill::Architecture,
                descripcion: "Redacción de planes técnicos y documentación arquitectónica",
                fuente: ".agent/skills/plan-writing",
            },
            FichaSkill {
                id: "brainstorming",
                categoria: CategoriaSkill::Architecture,
                descripcion: "Lluvia de ideas estructurada para resolver problemas complejos",
                fuente: ".agent/skills/brainstorming",
            },
            FichaSkill {
                id: "intelligent-routing",
                categoria: CategoriaSkill::Architecture,
                descripcion: "Enrutamiento inteligente de tareas entre agentes especializados",
                fuente: ".agent/skills/intelligent-routing",
            },
            // ── Mobile ──
            FichaSkill {
                id: "mobile-design",
                categoria: CategoriaSkill::Mobile,
                descripcion: "Diseño mobile: iOS HIG, Material Design, responsive mobile-first",
                fuente: ".agent/skills/mobile-design",
            },
            // ── Game Dev ──
            FichaSkill {
                id: "game-development",
                categoria: CategoriaSkill::GameDev,
                descripcion: "Desarrollo de juegos: Unity, Unreal, Godot, mecánicas y físicas",
                fuente: ".agent/skills/game-development",
            },
            // ── SEO ──
            FichaSkill {
                id: "seo-fundamentals",
                categoria: CategoriaSkill::Seo,
                descripcion: "Fundamentos SEO: on-page, off-page, técnico, Core Web Vitals",
                fuente: ".agent/skills/seo-fundamentals",
            },
            FichaSkill {
                id: "geo-fundamentals",
                categoria: CategoriaSkill::Seo,
                descripcion: "SEO geolocalizado: Google My Business, mapas, reseñas locales",
                fuente: ".agent/skills/geo-fundamentals",
            },
            // ── Shell ──
            FichaSkill {
                id: "bash-linux",
                categoria: CategoriaSkill::Shell,
                descripcion: "Scripting avanzado en Bash/Linux: sed, awk, pipes, procesos",
                fuente: ".agent/skills/doc.md",
            },
            FichaSkill {
                id: "powershell-windows",
                categoria: CategoriaSkill::Shell,
                descripcion: "PowerShell: scripting, automatización, administración Windows",
                fuente: ".agent/skills/powershell-windows",
            },
            // ── Quality ──
            FichaSkill {
                id: "clean-code",
                categoria: CategoriaSkill::Quality,
                descripcion: "Principios de código limpio: SOLID, KISS, DRY, YAGNI",
                fuente: ".agent/skills/clean-code",
            },
            FichaSkill {
                id: "systematic-debugging",
                categoria: CategoriaSkill::Quality,
                descripcion: "Debug sistemático: bisect, logging, profiling, tracing",
                fuente: ".agent/skills/systematic-debugging",
            },
            FichaSkill {
                id: "performance-profiling",
                categoria: CategoriaSkill::Quality,
                descripcion: "Perfilamiento de rendimiento: CPU, memoria, I/O, red",
                fuente: ".agent/skills/performance-profiling",
            },
            // ── Productivity ──
            FichaSkill {
                id: "behavioral-modes",
                categoria: CategoriaSkill::Productivity,
                descripcion: "Modos de comportamiento: deep work, revisión, exploración",
                fuente: ".agent/skills/behavioral-modes",
            },
            FichaSkill {
                id: "parallel-agents",
                categoria: CategoriaSkill::Productivity,
                descripcion: "Agentes paralelos: ejecución concurrente de tareas independientes",
                fuente: ".agent/skills/parallel-agents",
            },
            FichaSkill {
                id: "documentation-templates",
                categoria: CategoriaSkill::Productivity,
                descripcion: "Plantillas de documentación: README, API docs, changelogs",
                fuente: ".agent/skills/documentation-templates",
            },
            FichaSkill {
                id: "i18n-localization",
                categoria: CategoriaSkill::Productivity,
                descripcion: "Internacionalización y localización: i18n, l10n, pluralización",
                fuente: ".agent/skills/i18n-localization",
            },
            FichaSkill {
                id: "mcp-builder",
                categoria: CategoriaSkill::Productivity,
                descripcion: "Constructor de servidores MCP: protocolo, herramientas, recursos",
                fuente: ".agent/skills/mcp-builder",
            },
            FichaSkill {
                id: "os-vm-orchestrator",
                categoria: CategoriaSkill::Productivity,
                descripcion:
                    "Orquestación de sistemas virtuales: VMs, contenedores, entornos aislados",
                fuente: ".agent/skills/os-vm-orchestrator",
            },
            // ── NEXUS Propios ──
            FichaSkill {
                id: "antigravity-design-expert",
                categoria: CategoriaSkill::NexusPropio,
                descripcion: "Experto en diseño Antigravity: UI/UX soberano del ecosistema NEXUS",
                fuente: ".agent/skills/antigravity-design-expert",
            },
            FichaSkill {
                id: "cred-omega",
                categoria: CategoriaSkill::NexusPropio,
                descripcion: "Credenciales OMEGA: identidad, firma y autenticación soberana",
                fuente: ".agent/skills/cred-omega",
            },
            FichaSkill {
                id: "elite-discipline",
                categoria: CategoriaSkill::NexusPropio,
                descripcion: "Disciplina Élite: estándares de excelencia en ejecución",
                fuente: ".agent/skills/elite-discipline",
            },
            FichaSkill {
                id: "product-auditor",
                categoria: CategoriaSkill::NexusPropio,
                descripcion: "Auditor de producto: revisión de calidad, usabilidad y viabilidad",
                fuente: ".agent/skills/product-auditor",
            },
            FichaSkill {
                id: "nexus-awesome-skills",
                categoria: CategoriaSkill::NexusPropio,
                descripcion: "Skills NEXUS Awesome: catálogo de habilidades extraordinarias",
                fuente: ".agent/skills/nexus_awesome_skills",
            },
            FichaSkill {
                id: "nexus-cfo-expert",
                categoria: CategoriaSkill::NexusPropio,
                descripcion: "CFO NEXUS: finanzas, presupuesto y contabilidad automatizada",
                fuente: ".agent/skills/nexus_cfo_expert",
            },
            FichaSkill {
                id: "nexus-legal-expert",
                categoria: CategoriaSkill::NexusPropio,
                descripcion: "Asesor legal NEXUS: contratos, compliance, derechos digitales",
                fuente: ".agent/skills/nexus_legal_expert",
            },
            FichaSkill {
                id: "nexus-marketing-expert",
                categoria: CategoriaSkill::NexusPropio,
                descripcion: "Marketing NEXUS: estrategia, embudos, copywriting, analítica",
                fuente: ".agent/skills/nexus_marketing_expert",
            },
            FichaSkill {
                id: "nexus-notary-expert",
                categoria: CategoriaSkill::NexusPropio,
                descripcion: "Notario NEXUS: certificación, sellado temporal y verificación",
                fuente: ".agent/skills/nexus_notary_expert",
            },
            FichaSkill {
                id: "nexus-scientist-expert",
                categoria: CategoriaSkill::NexusPropio,
                descripcion: "Científico NEXUS: análisis de datos, modelos estadísticos, ML",
                fuente: ".agent/skills/nexus_scientist_expert",
            },
        ]
    })
}

/// Catálogo completo de skills.
pub fn catalogo_skills() -> &'static Vec<FichaSkill> {
    init_catalogo()
}

/// Busca un skill por su ID.
pub fn buscar_skill(id: &str) -> Option<&'static FichaSkill> {
    init_catalogo().iter().find(|s| s.id == id)
}

/// Filtra skills por categoría.
pub fn skills_por_categoria(categoria: CategoriaSkill) -> Vec<&'static FichaSkill> {
    init_catalogo()
        .iter()
        .filter(|s| s.categoria == categoria)
        .collect()
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalogo_tiene_todos_los_skills() {
        let catalogo = catalogo_skills();
        assert!(
            catalogo.len() >= 47,
            "Debe haber al menos 47 skills, se encontraron {}",
            catalogo.len()
        );
    }

    #[test]
    fn test_todos_los_skills_tienen_id() {
        for skill in catalogo_skills().iter() {
            assert!(!skill.id.is_empty(), "Skill sin ID");
        }
    }

    #[test]
    fn test_buscar_skill_por_id() {
        let skill = buscar_skill("rust-pro");
        assert!(skill.is_some());
        assert_eq!(skill.unwrap().categoria, CategoriaSkill::Backend);
    }

    #[test]
    fn test_skills_por_categoria() {
        let frontend = skills_por_categoria(CategoriaSkill::Frontend);
        assert!(frontend.len() >= 3, "Debe haber al menos 3 skills frontend");
    }

    #[test]
    fn test_buscar_skill_inexistente() {
        assert!(buscar_skill("skill-fantasma").is_none());
    }

    #[test]
    fn test_todas_las_categorias_tienen_al_menos_un_skill() {
        use CategoriaSkill::*;
        let todas = vec![
            Frontend,
            Backend,
            Database,
            TypeScript,
            CloudInfra,
            Testing,
            Security,
            Architecture,
            Mobile,
            GameDev,
            Seo,
            Shell,
            Quality,
            Design,
            Productivity,
            NexusPropio,
        ];
        for cat in todas {
            let skills = skills_por_categoria(cat);
            assert!(!skills.is_empty(), "Categoría {:?} no tiene skills", cat);
        }
    }
}
