use crate::brain::hippocampus::ArtificialHippocampus;
use crate::brain::NeuralManager;
use anyhow::Result;
use std::sync::Arc;

pub struct Session {
    hippocampus: Arc<ArtificialHippocampus>,
    _neural_manager: Arc<NeuralManager>,
}

impl Session {
    pub fn new(_qdrant_url: &str, neural_manager: Arc<NeuralManager>) -> Result<Self> {
        let hippocampus = Arc::new(ArtificialHippocampus::new(
            None,
            None,
            "C:/Users/crisp/NEXUS_ULTIMATE_CORE/data/nexus_memory",
        ));

        Ok(Self {
            hippocampus,
            _neural_manager: neural_manager,
        })
    }

    pub fn hippocampus(&self) -> Arc<ArtificialHippocampus> {
        self.hippocampus.clone()
    }
}
