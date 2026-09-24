// ==========================================
// 🧠 CORTEZA PREFRONTAL — Planificación y experiencia táctica
// ==========================================
// Registra experiencias tácticas con contexto de sistema para
// aprendizaje por refuerzo (post-evaluación).
// ==========================================

use std::time::SystemTime;

/// Resultado de una acción táctica.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionOutcome {
    Success,
    Failure,
}

/// Experiencia táctica acumulada con contexto de carga.
#[derive(Debug, Clone)]
pub struct TacticalExperience {
    pub action_id: String,
    pub module: String,
    pub outcome: ActionOutcome,
    pub failure_point: Option<String>,
    pub context: Option<String>,
    pub cpu_load: f32,
    pub ram_load: f32,
    pub timestamp: SystemTime,
}

impl TacticalExperience {
    pub fn new(action_id: impl Into<String>, module: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            module: module.into(),
            outcome: ActionOutcome::Success,
            failure_point: None,
            context: None,
            cpu_load: 0.0,
            ram_load: 0.0,
            timestamp: SystemTime::now(),
        }
    }

    pub fn registrar(&mut self, outcome: ActionOutcome) {
        self.outcome = outcome;
    }

    pub fn exitoso(&self) -> bool {
        self.outcome == ActionOutcome::Success
    }
}

/// Corteza prefrontal principal.
#[derive(Default)]
pub struct PrefrontalCortex;

impl PrefrontalCortex {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl crate::brain::CognitiveCortex for PrefrontalCortex {
    async fn reason(&self, req: crate::cerebro::reptilian::InferenceRequest) -> anyhow::Result<String> {
        Ok(format!("Razonamiento completado: {}", req.prompt))
    }
    fn set_personality(&self, _p: crate::cerebro::affective_engine::Personality) -> anyhow::Result<()> {
        Ok(())
    }
}
