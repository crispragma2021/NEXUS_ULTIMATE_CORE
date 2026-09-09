// ==========================================
// 🧠 PENSAMIENTO ESTRATÉGICO ADAPTATIVO
// ==========================================
// Calcula el esfuerzo cognitivo según la complejidad
// de la tarea y el estado térmico del hardware.
//
// Legacy DNA: nexus-orquestador/src/thinking_strategy.rs
// Absorbido: 11-Jun-2026

use serde::{Deserialize, Serialize};

/// Nivel de esfuerzo cognitivo para procesar una tarea.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ThinkingEffort {
    Low,    // Respuesta casi instantánea para tareas triviales
    Medium, // Balance entre velocidad y razonamiento
    High,   // Razonamiento profundo para refactorizaciones
    Max,    // Modo "Guerra Total" para problemas críticos
}

/// Estrategia de pensamiento completa.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThinkingStrategy {
    pub effort: ThinkingEffort,
    pub interleaved: bool,
    pub budget_tokens: u32,
}

/// Planificador adaptativo de esfuerzo cognitivo.
pub struct AdaptiveThinking {
    pub current_effort: ThinkingEffort,
    pub interleaved_mode: bool,
}

impl Default for AdaptiveThinking {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveThinking {
    pub fn new() -> Self {
        Self {
            current_effort: ThinkingEffort::Medium,
            interleaved_mode: true,
        }
    }

    /// Calcula la estrategia basada en el prompt y el estado del hardware.
    pub fn get_strategy(
        &self,
        prompt: &str,
        thermal_core: Option<f64>,
        memoria_presion: Option<bool>,
    ) -> ThinkingStrategy {
        let complexity = Self::estimate_complexity(prompt);
        let mut effort = Self::calculate_effort(complexity);

        // Degradar esfuerzo si el hardware está bajo presión
        if let Some(temp) = thermal_core {
            if temp > 0.8 || memoria_presion.unwrap_or(false) {
                effort = match effort {
                    ThinkingEffort::Max => ThinkingEffort::High,
                    ThinkingEffort::High => ThinkingEffort::Medium,
                    _ => ThinkingEffort::Low,
                };
            }
        }

        ThinkingStrategy {
            effort,
            interleaved: self.interleaved_mode,
            budget_tokens: match effort {
                ThinkingEffort::Low => 512,
                ThinkingEffort::Medium => 2048,
                ThinkingEffort::High => 8192,
                ThinkingEffort::Max => 32768,
            },
        }
    }

    pub fn calculate_effort(task_complexity: f32) -> ThinkingEffort {
        if task_complexity < 0.3 {
            ThinkingEffort::Low
        } else if task_complexity < 0.6 {
            ThinkingEffort::Medium
        } else if task_complexity < 0.9 {
            ThinkingEffort::High
        } else {
            ThinkingEffort::Max
        }
    }

    pub fn estimate_complexity(prompt: &str) -> f32 {
        let mut score: f32 = 0.1;
        let keywords = [
            "refactor",
            "build",
            "create",
            "architect",
            "debug",
            "complex",
            "security",
            "impact",
            "analyze",
            "deep",
        ];
        let prompt_lower = prompt.to_lowercase();

        for kw in keywords {
            if prompt_lower.contains(kw) {
                score += 0.15;
            }
        }

        if prompt.len() > 1000 {
            score += 0.3;
        }

        score.min(1.0)
    }
}
