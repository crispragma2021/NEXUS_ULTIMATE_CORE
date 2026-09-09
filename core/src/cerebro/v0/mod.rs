// ============================================================================
// 🧬 NEXUS V0 — Generador de UI multi-agente estilo v0.app
// ============================================================================
// Propósito: Pipeline de 9 etapas donde Gemini planifica/genera UI y DeepSeek
// depura. Contratos JSON, Session Store, Gates (AST→Render→Visual).
//
//   FASE 0 — contratos + session store        (completada)
//   FASE 1 — planificación + generación       (completada)
//   FASE 2 — pipeline de gates                (completada)
//   FASE 3 — debugger multi-nivel             (completada)
//   FASE 4 — orquestación + integración       (completada)
//   FASE 5 — pulido v0-real                    (completada)
//   FASE 6 — refuerzo RAG + razonamiento local (Qwen/Ollama + web) (completada)
//   FASE 6.1 — memoria de contexto (hipocampo): recuperación selectiva con
//              presupuesto de tokens para ventanas de contexto pequeñas (completada)
// ============================================================================

pub mod contracts;
pub mod debugger_tier1;
pub mod debugger_tier2;
pub mod dependency_resolver;
pub mod diff_engine;
pub mod gate_ast;
pub mod gate_render;
pub mod gate_visual;
pub mod generator;
pub mod memoria_contexto;
pub mod pipeline;
pub mod planner;
pub mod polish;
pub mod rag_shadcn;
pub mod razonador_qwen;
pub mod refuerzo_web;
pub mod session_store;
pub mod telemetry;

pub use contracts::{
    ArchivoGenerado, DependenciasPlan, DesignTokens, DiffEntry, ErrorGate, GateKind, GateResult,
    GeneracionUI, MetricaError, MetricasSession, NodoComponente, PlanComponentes, RoutePlan,
    SessionState, SeveridadError, StateShape, StateVar, V0_SCHEMA_GATE, V0_SCHEMA_GENERATE,
    V0_SCHEMA_PLAN, V0_SCHEMA_SESSION,
};
pub use debugger_tier1::{DebuggerTier1, ResultadoDebugTier1};
pub use debugger_tier2::{DebuggerTier2, ResultadoDebugTier2};
pub use dependency_resolver::{
    Allowlist, DependencyResolver, PaquetePermitido, ResolucionDep, ResultadoResolucion,
};
pub use diff_engine::{DiffEngine, ResultadoDiff};
pub use gate_ast::{GateAst, ResultadoGateAst};
pub use gate_render::{GateRender, ResultadoGateRender};
pub use gate_visual::{GateVisual, ResultadoGateVisual};
pub use generator::Generador;
pub use memoria_contexto::{FragmentoContexto, MemoriaContexto, ResultadoRecuperacion};
pub use pipeline::{PipelineV0, ResultadoPipeline};
pub use planner::{IntencionUI, PlanLocal, Planificador};
pub use polish::{
    exportar_codesandbox, exportar_stackblitz, validar_tokens, ErrorDataset, ErrorFrecuente,
    HallazgoToken, PayloadCodeSandbox, PayloadStackBlitz, ResultadoTokens,
};
pub use rag_shadcn::{CatalogoShadcn, ComponenteShadcn};
pub use razonador_qwen::{PlanRazonado, RazonadorQwen, ResultadoRazonamiento};
pub use refuerzo_web::{ReferenciaWeb, RefuerzoWeb, ResultadoRefuerzo};
pub use session_store::{SessionStore, SessionStoreError};
pub use telemetry::{ResumenTelemetria, TelemetriaV0};

/// Versión del módulo V0.
pub const VERSION_V0: &str = "0.7.0";
