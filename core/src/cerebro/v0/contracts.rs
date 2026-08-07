// ============================================================================
// 📋 CONTRATOS JSON DEL PIPELINE V0
// ============================================================================
// Los 4 contratos que intercambian los agentes (Gemini planifica/genera,
// DeepSeek depura, pipeline valida):
//   1. PlanComponentes   — árbol de UI, layout, estado, dependencias
//   2. GeneracionUI      — archivos .tsx/.ts/.css + package.json
//   3. GateResult        — unificado para AST, render y crítica visual
//   4. SessionState      — proyecto completo + diff history entre turnos
//
// Todos son Serialize/Deserialize con serde para compatibilidad con
// `response_schema` de Gemini y persistencia en SQLite.
// ============================================================================

use serde::{Deserialize, Serialize};

/// Versiones de schema. Se exponen como constantes para los `$schema`.
pub const V0_SCHEMA_PLAN: &str = "nexus-v0-plan-v1";
pub const V0_SCHEMA_GENERATE: &str = "nexus-v0-generate-v1";
pub const V0_SCHEMA_GATE: &str = "nexus-v0-gate-v1";
pub const V0_SCHEMA_SESSION: &str = "nexus-v0-session-v1";

// ============================================================================
// CONTRATO 1 — PlanComponentes
// ============================================================================

/// Metadatos de la aplicación a generar.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanComponentes {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub app: AppSpec,
    pub page_tree: Vec<RoutePlan>,
    pub component_tree: NodoComponente,
    pub dependencies: DependenciasPlan,
    pub state_shape: StateShape,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSpec {
    pub name: String,
    pub description: String,
    pub framework: String,
    pub styling: String,
    #[serde(rename = "component_library")]
    pub component_library: String,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoutePlan {
    pub path: String,
    pub component: String,
    pub layout: String,
}

/// Un nodo del árbol de componentes. Soporta hijos recursivos.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodoComponente {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub props: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub children: Vec<NodoComponente>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependenciasPlan {
    pub runtime: Vec<String>,
    pub ui: Vec<String>,
    pub styling: Vec<String>,
    pub utils: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateShape {
    #[serde(rename = "useState")]
    #[serde(default)]
    pub use_state: Vec<StateVar>,
    #[serde(rename = "useReducer")]
    #[serde(default)]
    pub use_reducer: Vec<String>,
    #[serde(default)]
    pub context: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateVar {
    pub name: String,
    #[serde(rename = "type")]
    pub tipo: String,
    pub initial: serde_json::Value,
}

// ============================================================================
// CONTRATO 2 — GeneracionUI
// ============================================================================

/// Archivo de código fuente generado.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchivoGenerado {
    pub path: String,
    pub content: String,
    pub language: String,
}

/// Resultado de la generación de Gemini: todos los archivos + package.json.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneracionUI {
    #[serde(rename = "$schema")]
    pub schema: String,
    #[serde(rename = "plan_id")]
    pub plan_id: String,
    pub files: Vec<ArchivoGenerado>,
    pub package_json: serde_json::Map<String, serde_json::Value>,
    #[serde(rename = "entry_point")]
    pub entry_point: String,
}

// ============================================================================
// CONTRATO 3 — GateResult
// ============================================================================

/// Tipo de gate de validación.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GateKind {
    Ast,
    Render,
    Visual,
}

/// Severidad de un error detectado por un gate.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SeveridadError {
    Error,
    Warning,
}

/// Error de sintaxis/tipos detectado por el Gate AST.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorGate {
    pub severity: SeveridadError,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub suggestion: String,
}

/// Error de runtime capturado en el Gate Render.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricaError {
    #[serde(rename = "type")]
    pub tipo: String,
    pub message: String,
    #[serde(default)]
    pub stack: String,
}

/// Resultado unificado de los 3 gates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateResult {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub gate: GateKind,
    pub passed: bool,
    #[serde(default)]
    pub errors: Vec<ErrorGate>,
    #[serde(rename = "runtime_errors")]
    #[serde(default)]
    pub runtime_errors: Vec<MetricaError>,
    #[serde(rename = "visual_issues")]
    #[serde(default)]
    pub visual_issues: Vec<MetricaError>,
    #[serde(rename = "duration_ms")]
    pub duration_ms: u64,
}

// ============================================================================
// CONTRATO 4 — SessionState
// ============================================================================

/// Design tokens del sistema visual impuesto por el pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DesignTokens {
    #[serde(default)]
    pub colors: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub typography: std::collections::HashMap<String, serde_json::Value>,
    #[serde(rename = "borderRadius")]
    #[serde(default)]
    pub border_radius: String,
}

impl Default for DesignTokens {
    fn default() -> Self {
        let mut colors = std::collections::HashMap::new();
        colors.insert("primary".to_string(), "#3B82F6".to_string());
        colors.insert("background".to_string(), "#FFFFFF".to_string());
        colors.insert("text".to_string(), "#111827".to_string());
        Self {
            colors,
            typography: std::collections::HashMap::new(),
            border_radius: "0.5rem".to_string(),
        }
    }
}

/// Entrada del historial de diffs entre turnos.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiffEntry {
    pub turn: u32,
    #[serde(rename = "user_prompt")]
    pub user_prompt: String,
    #[serde(rename = "plan_snapshot")]
    pub plan_snapshot: Option<PlanComponentes>,
    #[serde(rename = "applied_diff")]
    pub applied_diff: String,
    pub timestamp: String,
}

/// Métricas acumuladas de la sesión.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MetricasSession {
    #[serde(rename = "total_turns")]
    pub total_turns: u32,
    #[serde(rename = "total_gate_failures")]
    pub total_gate_failures: u32,
    #[serde(rename = "total_debugger_invocations")]
    pub total_debugger_invocations: u32,
    #[serde(rename = "avg_latency_ms")]
    pub avg_latency_ms: u64,
}

/// Estado completo de una sesión de generación de UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionState {
    #[serde(rename = "$schema")]
    pub schema: String,
    #[serde(rename = "session_id")]
    pub session_id: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "current_plan")]
    pub current_plan: Option<PlanComponentes>,
    #[serde(rename = "current_code")]
    pub current_code: Option<GeneracionUI>,
    #[serde(rename = "diff_history")]
    #[serde(default)]
    pub diff_history: Vec<DiffEntry>,
    #[serde(rename = "design_tokens")]
    pub design_tokens: DesignTokens,
    pub metrics: MetricasSession,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan_ejemplo() -> PlanComponentes {
        PlanComponentes {
            schema: V0_SCHEMA_PLAN.to_string(),
            app: AppSpec {
                name: "dashboard".into(),
                description: "Panel de métricas".into(),
                framework: "react".into(),
                styling: "tailwind".into(),
                component_library: "shadcn/ui".into(),
                theme: "light".into(),
            },
            page_tree: vec![RoutePlan {
                path: "/".into(),
                component: "DashboardPage".into(),
                layout: "default".into(),
            }],
            component_tree: NodoComponente {
                name: "App".into(),
                source: "local".into(),
                props: Default::default(),
                children: vec![NodoComponente {
                    name: "Button".into(),
                    source: "shadcn/ui".into(),
                    props: Default::default(),
                    children: vec![],
                }],
            },
            dependencies: DependenciasPlan {
                runtime: vec!["react".into(), "react-dom".into()],
                ui: vec!["lucide-react".into()],
                styling: vec!["tailwindcss".into()],
                utils: vec!["clsx".into()],
            },
            state_shape: StateShape {
                use_state: vec![StateVar {
                    name: "count".into(),
                    tipo: "number".into(),
                    initial: serde_json::json!(0),
                }],
                use_reducer: vec![],
                context: vec![],
            },
        }
    }

    #[test]
    fn test_plan_roundtrip_json() {
        let plan = plan_ejemplo();
        let json = serde_json::to_string(&plan).unwrap();
        let plan2: PlanComponentes = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, plan2);
    }

    #[test]
    fn test_plan_json_tiene_schema() {
        let plan = plan_ejemplo();
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains(V0_SCHEMA_PLAN));
        assert!(json.contains("component_library"));
        assert!(json.contains("useState"));
    }

    #[test]
    fn test_generacion_roundtrip_json() {
        let gen = GeneracionUI {
            schema: V0_SCHEMA_GENERATE.into(),
            plan_id: "plan-1".into(),
            files: vec![ArchivoGenerado {
                path: "src/App.tsx".into(),
                content: "export default function App() { return <div/>; }".into(),
                language: "tsx".into(),
            }],
            package_json: serde_json::json!({
                "name": "nexus-v0-app",
                "dependencies": {}
            })
            .as_object()
            .unwrap()
            .clone(),
            entry_point: "src/App.tsx".into(),
        };
        let json = serde_json::to_string(&gen).unwrap();
        let gen2: GeneracionUI = serde_json::from_str(&json).unwrap();
        assert_eq!(gen, gen2);
        assert!(json.contains(V0_SCHEMA_GENERATE));
    }

    #[test]
    fn test_gate_result_roundtrip_json() {
        let gate = GateResult {
            schema: V0_SCHEMA_GATE.into(),
            gate: GateKind::Ast,
            passed: false,
            errors: vec![ErrorGate {
                severity: SeveridadError::Error,
                file: "src/App.tsx".into(),
                line: 5,
                column: 12,
                message: "Type 'string' is not assignable to type 'number'".into(),
                code: "TS2322".into(),
                suggestion: "Consider parseInt()".into(),
            }],
            runtime_errors: vec![],
            visual_issues: vec![],
            duration_ms: 42,
        };
        let json = serde_json::to_string(&gate).unwrap();
        let gate2: GateResult = serde_json::from_str(&json).unwrap();
        assert_eq!(gate, gate2);
        // gate enum serializa en lowercase
        assert!(json.contains("\"ast\""));
    }

    #[test]
    fn test_gate_kind_serde() {
        assert_eq!(
            serde_json::to_string(&GateKind::Render).unwrap(),
            "\"render\""
        );
        assert_eq!(
            serde_json::from_str::<GateKind>("\"visual\"").unwrap(),
            GateKind::Visual
        );
    }

    #[test]
    fn test_severidad_serde() {
        assert_eq!(
            serde_json::to_string(&SeveridadError::Warning).unwrap(),
            "\"warning\""
        );
    }

    #[test]
    fn test_session_state_roundtrip() {
        let state = SessionState {
            schema: V0_SCHEMA_SESSION.into(),
            session_id: "uuid-123".into(),
            created_at: "2026-08-03T00:00:00Z".into(),
            updated_at: "2026-08-03T00:00:00Z".into(),
            current_plan: Some(plan_ejemplo()),
            current_code: None,
            diff_history: vec![],
            design_tokens: DesignTokens::default(),
            metrics: MetricasSession::default(),
        };
        let json = serde_json::to_string(&state).unwrap();
        let state2: SessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, state2);
        assert!(json.contains(V0_SCHEMA_SESSION));
    }

    #[test]
    fn test_design_tokens_default() {
        let dt = DesignTokens::default();
        assert_eq!(dt.colors.get("primary").unwrap(), "#3B82F6");
        assert_eq!(dt.border_radius, "0.5rem");
    }

    #[test]
    fn test_campos_ausentes_con_default() {
        // Sin `children`, `props`, `useState`, `diff_history` → default vacío
        let json = r#"{
            "$schema": "nexus-v0-plan-v1",
            "app": {"name":"x","description":"","framework":"react","styling":"tailwind","component_library":"shadcn/ui","theme":"light"},
            "page_tree": [],
            "component_tree": {"name":"App","source":"local"},
            "dependencies": {"runtime":[],"ui":[],"styling":[],"utils":[]},
            "state_shape": {}
        }"#;
        let plan: PlanComponentes = serde_json::from_str(json).unwrap();
        assert!(plan.component_tree.children.is_empty());
        assert!(plan.state_shape.use_state.is_empty());
        assert!(plan.component_tree.props.is_empty());
    }
}
