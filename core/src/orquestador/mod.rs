// 🔱 ORQUESTADOR DE OPERADORES — Entrada y export de los 5 pilares deterministas
// Punto de entrada único para el Orquestador local determinista.
// Puro Rust, sin unwrap(), sin expect(), listo para producción.

pub mod cloud_fallback;
pub mod context_pruner;
pub mod execution_loop;
pub mod feedback_bus;
pub mod inference_config;
pub mod sandbox;
pub mod slm_dispatcher;
pub mod task_graph;
pub mod validator;
// 🎯 Aislamiento de contexto + registro inmutable de puertos + health LED
pub mod health_monitor;
pub mod port_registry;
pub mod scope_mapper;

pub use cloud_fallback::CloudFallback;
pub use context_pruner::{ContextPruner, PrunedContext};
pub use execution_loop::ExecutionLoop;
pub use feedback_bus::{ExecutionMetrics, FeedbackBus, TaskDispatch, TaskResult};
pub use inference_config::SLMInferenceConfig;
pub use sandbox::{Sandbox, SandboxConfig};
pub use slm_dispatcher::SLMDispatcher;
pub use task_graph::{DAGState, NodeState, Priority, TaskDAG, TaskNode, ToolAction};
pub use validator::{ValidationResult, Validator};
