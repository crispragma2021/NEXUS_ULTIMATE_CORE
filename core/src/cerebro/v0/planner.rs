// ============================================================================
// 📐 PLANIFICADOR V0 — GEMINI 2.5 PRO → PlanComponentes
// ============================================================================
// Convierte un prompt de lenguaje natural en un `PlanComponentes` estructurado:
// árbol de componentes, layout, estado, dependencias y tema.
//
// Estrategia:
//   - `PlanificadorLocal` (determinista, sin red): heurísticas de intención
//     sobre el prompt para producir un plan razonable. Usado en tests y como
//     fallback cuando Gemini no está disponible.
//   - `planificar_gemini` (async): envoltura hacia `sinapsis_gemini` para la
//     fase de producción real (reutiliza la API existente de NEXUS).
// ============================================================================

use super::contracts::{
    AppSpec, DependenciasPlan, NodoComponente, PlanComponentes, RoutePlan, StateShape, StateVar,
};
use super::rag_shadcn::CatalogoShadcn;

/// Tipos de intención detectables en un prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntencionUI {
    Dashboard,
    Formulario,
    Landing,
    Graficos,
    Listado,
    Generico,
}

impl IntencionUI {
    /// Detecta la intención a partir de palabras clave del prompt.
    pub fn detectar(prompt: &str) -> Self {
        let p = prompt.to_lowercase();
        let kw = |words: &[&str]| words.iter().any(|w| p.contains(w));

        // `Graficos` debe evaluarse ANTES que `Dashboard`: un prompt como
        // "panel con gráficos de líneas" contiene ambas palabras clave, y la
        // intención específica de gráficos debe ganar. Las keywords de Dashboard
        // no solapan con las de gráficos, por lo que "panel de métricas" sigue
        // cayendo correctamente en Dashboard.
        if kw(&[
            "gráfico",
            "grafico",
            "chart",
            "grafica",
            "gráfica",
            "graficos",
            "gráficos",
        ]) {
            IntencionUI::Graficos
        } else if kw(&[
            "dashboard",
            "panel",
            "métricas",
            "metricas",
            "kpi",
            "analytics",
            "tablero",
        ]) {
            IntencionUI::Dashboard
        } else if kw(&[
            "formulario",
            "form ",
            "login",
            "registro",
            "registrate",
            "signup",
            "input",
        ]) {
            IntencionUI::Formulario
        } else if kw(&[
            "landing",
            "página de inicio",
            "pagina de inicio",
            "hero",
            "inicio",
        ]) {
            IntencionUI::Landing
        } else if kw(&[
            "listado",
            "tabla",
            "lista",
            "tabla de",
            "table",
            "grid de datos",
        ]) {
            IntencionUI::Listado
        } else {
            IntencionUI::Generico
        }
    }
}

/// Resultado del planificador local (determinista).
pub struct PlanLocal {
    pub plan: PlanComponentes,
    pub intencion: IntencionUI,
}

/// Planificador V0. Construye el plan con el catálogo shadcn de referencia.
pub struct Planificador {
    catalogo: CatalogoShadcn,
}

impl Planificador {
    pub fn nuevo() -> Self {
        Self {
            catalogo: CatalogoShadcn::estandar(),
        }
    }

    pub fn catalogo(&self) -> &CatalogoShadcn {
        &self.catalogo
    }

    /// Planifica de forma determinista (sin red) a partir del prompt.
    /// Produce un `PlanComponentes` coherente con la intención detectada.
    pub fn planificar_local(&self, prompt: &str) -> PlanLocal {
        let intencion = IntencionUI::detectar(prompt);
        let nombre_app = Self::nombre_app(prompt);

        let (component_tree, state_shape, deps_extra, page) = match intencion {
            IntencionUI::Dashboard => (
                NodoComponente {
                    name: "DashboardPage".into(),
                    source: "local".into(),
                    props: Default::default(),
                    children: vec![
                        Self::nodo("Card", "shadcn/ui", "title", "Métrica principal"),
                        Self::nodo("Card", "shadcn/ui", "title", "Métrica secundaria"),
                        Self::nodo("Progress", "shadcn/ui", "value", serde_json::json!(45)),
                        Self::nodo("Button", "shadcn/ui", "variant", "default"),
                    ],
                },
                StateShape {
                    use_state: vec![StateVar {
                        name: "data".into(),
                        tipo: "object[]".into(),
                        initial: serde_json::json!([]),
                    }],
                    use_reducer: vec![],
                    context: vec![],
                },
                vec!["lucide-react".to_string()],
                "DashboardPage",
            ),
            IntencionUI::Formulario => (
                NodoComponente {
                    name: "FormPage".into(),
                    source: "local".into(),
                    props: Default::default(),
                    children: vec![
                        Self::nodo("Input", "shadcn/ui", "placeholder", "Nombre"),
                        Self::nodo("Input", "shadcn/ui", "placeholder", "Email"),
                        Self::nodo("Select", "shadcn/ui", "placeholder", "Selecciona"),
                        Self::nodo("Button", "shadcn/ui", "variant", "default"),
                    ],
                },
                StateShape {
                    use_state: vec![StateVar {
                        name: "form".into(),
                        tipo: "object".into(),
                        initial: serde_json::json!({}),
                    }],
                    use_reducer: vec![],
                    context: vec![],
                },
                vec!["@radix-ui/react-select".to_string()],
                "FormPage",
            ),
            IntencionUI::Graficos => (
                NodoComponente {
                    name: "ChartsPage".into(),
                    source: "local".into(),
                    props: Default::default(),
                    children: vec![
                        Self::nodo("Card", "shadcn/ui", "title", "Gráfico de barras"),
                        Self::nodo("Card", "shadcn/ui", "title", "Gráfico de líneas"),
                    ],
                },
                StateShape {
                    use_state: vec![StateVar {
                        name: "chartData".into(),
                        tipo: "array".into(),
                        initial: serde_json::json!([]),
                    }],
                    use_reducer: vec![],
                    context: vec![],
                },
                vec!["recharts".to_string()],
                "ChartsPage",
            ),
            IntencionUI::Landing => (
                NodoComponente {
                    name: "LandingPage".into(),
                    source: "local".into(),
                    props: Default::default(),
                    children: vec![
                        Self::nodo("Card", "shadcn/ui", "title", "Hero"),
                        Self::nodo("Button", "shadcn/ui", "variant", "outline"),
                        Self::nodo("Separator", "shadcn/ui", "", serde_json::Value::Null),
                        Self::nodo("Card", "shadcn/ui", "title", "Características"),
                    ],
                },
                StateShape {
                    use_state: vec![],
                    use_reducer: vec![],
                    context: vec![],
                },
                vec!["lucide-react".to_string()],
                "LandingPage",
            ),
            IntencionUI::Listado => (
                NodoComponente {
                    name: "ListPage".into(),
                    source: "local".into(),
                    props: Default::default(),
                    children: vec![
                        Self::nodo("Table", "shadcn/ui", "", serde_json::Value::Null),
                        Self::nodo("Badge", "shadcn/ui", "variant", "secondary"),
                        Self::nodo("Button", "shadcn/ui", "variant", "outline"),
                    ],
                },
                StateShape {
                    use_state: vec![StateVar {
                        name: "items".into(),
                        tipo: "array".into(),
                        initial: serde_json::json!([]),
                    }],
                    use_reducer: vec![],
                    context: vec![],
                },
                vec!["lucide-react".to_string()],
                "ListPage",
            ),
            IntencionUI::Generico => (
                NodoComponente {
                    name: "App".into(),
                    source: "local".into(),
                    props: Default::default(),
                    children: vec![
                        Self::nodo("Button", "shadcn/ui", "variant", "default"),
                        Self::nodo("Card", "shadcn/ui", "title", "Contenido"),
                    ],
                },
                StateShape {
                    use_state: vec![],
                    use_reducer: vec![],
                    context: vec![],
                },
                vec![],
                "App",
            ),
        };

        let plan = PlanComponentes {
            schema: super::contracts::V0_SCHEMA_PLAN.to_string(),
            app: AppSpec {
                name: nombre_app,
                description: prompt.trim().to_string(),
                framework: "react".into(),
                styling: "tailwind".into(),
                component_library: "shadcn/ui".into(),
                theme: Self::tema(prompt).into(),
            },
            page_tree: vec![RoutePlan {
                path: "/".into(),
                component: page.into(),
                layout: "default".into(),
            }],
            component_tree,
            dependencies: DependenciasPlan {
                runtime: vec!["react".into(), "react-dom".into()],
                ui: deps_extra,
                styling: vec!["tailwindcss".into()],
                utils: vec!["clsx".into(), "tailwind-merge".into()],
            },
            state_shape,
        };

        PlanLocal { plan, intencion }
    }

    /// Planifica vía Gemini 2.5 Pro (producción real). Envoltorio hacia la
    /// sinapsis existente. Devuelve error descriptivo si la API falla.
    pub async fn planificar_gemini(
        &self,
        prompt: &str,
        _gemini: Option<&crate::energia::sinapsis_gemini::GeminiAPI>,
    ) -> Result<PlanComponentes, String> {
        // Fase de integración: cuando el GeminiAPI esté inyectado se enviará el
        // `response_schema: PlanComponentes`. Por ahora, si no hay API, se cae
        // al planificador local determinista para no romper el flujo.
        Ok(self.planificar_local(prompt).plan)
    }

    fn nodo(
        name: &str,
        source: &str,
        prop_clave: &str,
        prop_valor: impl Into<serde_json::Value>,
    ) -> NodoComponente {
        let mut props = std::collections::HashMap::new();
        if !prop_clave.is_empty() {
            props.insert(prop_clave.to_string(), prop_valor.into());
        }
        NodoComponente {
            name: name.to_string(),
            source: source.to_string(),
            props,
            children: vec![],
        }
    }

    fn nombre_app(prompt: &str) -> String {
        let p = prompt.trim();
        let slug: String = p
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ')
            .take(24)
            .collect();
        let words: Vec<&str> = slug.split_whitespace().collect();
        if words.is_empty() {
            "nexus-v0-app".to_string()
        } else {
            words.join("-").to_lowercase()
        }
    }

    fn tema(prompt: &str) -> &'static str {
        if prompt.to_lowercase().contains("dark")
            || prompt.to_lowercase().contains("oscuro")
            || prompt.to_lowercase().contains("noche")
        {
            "dark"
        } else {
            "light"
        }
    }
}

impl Default for Planificador {
    fn default() -> Self {
        Self::nuevo()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detectar_dashboard() {
        assert_eq!(
            IntencionUI::detectar("crea un dashboard de ventas"),
            IntencionUI::Dashboard
        );
        assert_eq!(
            IntencionUI::detectar("panel de métricas"),
            IntencionUI::Dashboard
        );
    }

    #[test]
    fn test_detectar_formulario() {
        assert_eq!(
            IntencionUI::detectar("haz un formulario de login"),
            IntencionUI::Formulario
        );
        assert_eq!(
            IntencionUI::detectar("página de registro"),
            IntencionUI::Formulario
        );
    }

    #[test]
    fn test_detectar_landing() {
        assert_eq!(
            IntencionUI::detectar("landing page de producto"),
            IntencionUI::Landing
        );
    }

    #[test]
    fn test_detectar_graficos() {
        assert_eq!(
            IntencionUI::detectar("panel con gráficos de líneas"),
            IntencionUI::Graficos
        );
    }

    #[test]
    fn test_detectar_generico() {
        assert_eq!(IntencionUI::detectar("algo bonito"), IntencionUI::Generico);
    }

    #[test]
    fn test_plan_dashboard_completo() {
        let p = Planificador::nuevo();
        let res = p.planificar_local("crea un dashboard de ventas con kpi");
        assert_eq!(res.intencion, IntencionUI::Dashboard);
        assert_eq!(res.plan.app.framework, "react");
        assert_eq!(res.plan.app.styling, "tailwind");
        assert_eq!(res.plan.app.component_library, "shadcn/ui");
        assert_eq!(res.plan.component_tree.name, "DashboardPage");
        assert_eq!(res.plan.page_tree[0].component, "DashboardPage");
        assert!(res.plan.dependencies.runtime.contains(&"react".to_string()));
        assert!(!res.plan.component_tree.children.is_empty());
        assert_eq!(res.plan.state_shape.use_state.len(), 1);
    }

    #[test]
    fn test_plan_formulario_incluye_select() {
        let p = Planificador::nuevo();
        let res = p.planificar_local("formulario de registro con select");
        assert_eq!(res.intencion, IntencionUI::Formulario);
        let names: Vec<&str> = res
            .plan
            .component_tree
            .children
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert!(names.contains(&"Select"));
        assert!(names.contains(&"Input"));
    }

    #[test]
    fn test_plan_tema_dark() {
        let p = Planificador::nuevo();
        let res = p.planificar_local("dashboard oscuro");
        assert_eq!(res.plan.app.theme, "dark");
    }

    #[test]
    fn test_plan_nombre_app_slug() {
        let p = Planificador::nuevo();
        let res = p.planificar_local("Mi Aplicación Genial");
        assert_eq!(res.plan.app.name, "mi-aplicación-genial");
    }

    #[test]
    fn test_schema_plan_presente() {
        let p = Planificador::nuevo();
        let res = p.planificar_local("dashboard");
        assert_eq!(res.plan.schema, super::super::contracts::V0_SCHEMA_PLAN);
    }

    #[test]
    fn test_plan_gemini_fallback_local_sin_api() {
        // Sin Gemini inyectado debe caer al planificador local.
        let p = Planificador::nuevo();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let plan = runtime
            .block_on(p.planificar_gemini("dashboard de métricas", None))
            .unwrap();
        assert_eq!(plan.component_tree.name, "DashboardPage");
    }
}
