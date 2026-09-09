// ==========================================
// 📜 CONSTITUCIÓN DE NEXUS — Principios Soberanos
// ==========================================
// Motor de crítica interna que evalúa acciones contra
// principios fundamentales: utilidad, honestidad, seguridad,
// no-regresión y autoconservación.
//
// Legacy DNA: nexus-orquestador/src/constitution.rs
// Absorbido: 11-Jun-2026

use serde::{Deserialize, Serialize};

/// Principios fundamentales de la Constitución de NEXUS.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Principle {
    Helpful,
    Honest,
    Harmless,
    NonRegression,
    SelfPreservation,
}

/// Tipo de acción a evaluar.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ActionType {
    CodeModification,
    SystemCommand,
    InformationRetrieval,
    MemoryUpdate,
}

/// Acción candidata a evaluación constitucional.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Action {
    pub action_type: ActionType,
    pub description: String,
    pub risk_level: u32, // 1-10
}

/// Veredicto del juicio constitucional.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ConstitutionalVerdict {
    Approved,
    Rejected(String),
    RequiresPermission(String),
}

/// Constitución de NEXUS: marco ético supremo.
pub struct NexusConstitution {
    pub version: String,
}

impl Default for NexusConstitution {
    fn default() -> Self {
        Self::new()
    }
}

impl NexusConstitution {
    pub fn new() -> Self {
        Self {
            version: "2.0.0-OMEGA".to_string(),
        }
    }

    /// Evalúa una acción contra los principios constitucionales.
    pub fn review(&self, action: &Action) -> ConstitutionalVerdict {
        // 1. Evaluación de Seguridad (Harmless)
        if action.risk_level > 8 && action.description.to_lowercase().contains("delete") {
            return ConstitutionalVerdict::Rejected(
                "Violación de Seguridad: Acción de alto riesgo detectada sin salvaguardas."
                    .to_string(),
            );
        }

        // 2. Evaluación de No-Regresión (NonRegression)
        if action.description.to_lowercase().contains("downgrade")
            || action
                .description
                .to_lowercase()
                .contains("versión anterior")
        {
            return ConstitutionalVerdict::Rejected(
                "Violación de No-Regresión: Prohibido degradar la excelencia técnica del sistema."
                    .to_string(),
            );
        }

        // 3. Evaluación de Honestidad (Honest)
        if action.description.is_empty() {
            return ConstitutionalVerdict::RequiresPermission(
                "Falta descripción de la acción para validación de honestidad.".to_string(),
            );
        }

        ConstitutionalVerdict::Approved
    }
}
