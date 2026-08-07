use crate::memoria::aprendizaje_profundo::DuelingQNetwork;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyStats {
    pub consecutive_failures: usize,
    pub total_failures: usize,
    #[serde(default)]
    pub total_successes: usize,
    pub deadline: u64,
    pub last_failure: u64,
    pub last_success: u64,
    pub punishment_factor: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TatuajeNeural {
    pub version: u32,
    pub timestamp: u64,
    pub network_architecture: String,
    pub total_weights: usize,
    pub key_stats: HashMap<String, KeyStats>,
    pub official_threshold: usize,
    pub openrouter_threshold: usize,
    pub network: Option<DuelingQNetwork>,
}

impl TatuajeNeural {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let tatuaje: Self = serde_json::from_str(&content)?;
        Ok(tatuaje)
    }

    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
