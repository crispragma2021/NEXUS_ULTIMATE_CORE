// ==========================================
// POLÍTICA SOBERANA DE NEXUS
// ==========================================
// Carga y valida el archivo nexus_asa_policy.yaml
// ==========================================

use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGovernor {
    pub cpu_max_percent: u8,
    pub mem_vector_max_mb: u16,
    pub net_requests_per_sec: u8,
    pub net_jitter_ms: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionSandbox {
    pub root_partition: String,
    pub overlay_partition: String,
    pub pre_commit_validation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealityAnchor {
    pub fundamental_truths: Vec<String>,
    pub semantic_distance_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticalFallback {
    pub token_verification_url: String,
    pub token_check_interval_minutes: u64,
    pub fallback_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusPolicy {
    pub version: String,
    pub sovereign_seal: String,
    pub resource_governor: ResourceGovernor,
    pub evolution_sandbox: EvolutionSandbox,
    pub reality_anchor: RealityAnchor,
    pub tactical_fallback: TacticalFallback,
}

impl NexusPolicy {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let policy_path = "nexus_asa_policy.yaml";
        let contents = fs::read_to_string(policy_path)?;
        let policy: NexusPolicy = serde_yaml::from_str(&contents)?;
        Ok(policy)
    }

    pub fn validate(&self) -> bool {
        // Verificar que el sello soberano tiene el formato correcto
        if !self.sovereign_seal.starts_with("0x") || self.sovereign_seal.len() < 10 {
            return false;
        }

        // Verificar verdades fundamentales
        if self.reality_anchor.fundamental_truths.is_empty() {
            return false;
        }

        true
    }
}
