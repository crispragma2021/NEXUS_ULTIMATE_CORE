use crate::brain::{neural_memory::NexusMemory, NeuralManager};
use anyhow::{anyhow, Result};
use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;

pub struct NeuralIngest {
    neural_manager: Arc<NeuralManager>,
    memory: Arc<NexusMemory>,
}

impl NeuralIngest {
    pub fn new(
        _qdrant_url: &str,
        _telegram_token: Option<String>,
        _chat_id: Option<i64>,
    ) -> Result<Self> {
        Ok(Self {
            neural_manager: Arc::new(NeuralManager::new()),
            memory: Arc::new(NexusMemory::new(
                &crate::infra::paths::resolve_path("brain/nexus_memory.lance").to_string_lossy(),
            )),
        })
    }

    pub async fn ingest_text(&self, text: &str, file_path: &str, file_name: &str) -> Result<()> {
        let active_engine = self.neural_manager.get_active_engine();
        let engine_guard = active_engine.read().await;

        if let Some(engine) = engine_guard.as_deref() {
            let embeddings = engine.generate_embeddings(text).await?;
            self.memory
                .add_entry(embeddings, text, file_path, file_name)
                .await?;
            Ok(())
        } else {
            Err(anyhow!(
                "No active inference engine (Native Candle engine required)"
            ))
        }
    }

    pub async fn transmute_directory(&self, path: &str) -> Result<()> {
        println!("📂 [NeuralIngest] Transmuting directory: {}", path);
        let path_obj = Path::new(path);

        if !path_obj.exists() {
            return Err(anyhow!("Path does not exist: {}", path));
        }

        let walker = WalkDir::new(path_obj)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "rs").unwrap_or(false));

        for entry in walker {
            let file_path = entry.path();
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let file_name = file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();
                println!("  🧠 Indexing: {}", file_path.display());
                self.ingest_text(&content, file_path.to_str().unwrap_or_default(), file_name)
                    .await?;
            }
        }

        Ok(())
    }
}
