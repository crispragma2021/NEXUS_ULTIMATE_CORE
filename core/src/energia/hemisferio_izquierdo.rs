// ==========================================
// HEMISFERIO IZQUIERDO - LÓGICA PRIORITARIA
// ==========================================
// 1. DeepSeek API (con saldo) - Rápido y preciso
// 2. Fallback a modelo local DeepSeek-R1 (Ollama)
// 3. Ambos con IDENTIDAD NEXUS desde identity.md
// ==========================================

use reqwest::Client;
use serde_json::json;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tracing::{info, warn};

pub struct HemisferioIzquierdo {
    client: Client,
    api_key: String,
    // Modo local (Ollama)
    ollama_url: String,
    model_local: String,
    /// Identidad soberana cargada desde docs/identity/identity.md
    /// Se inyecta como `system` tanto en API cloud como en modelo local
    pub identidad: String,
    // Métricas internas atómicas para Send+Sync
    uso_local: AtomicU32,
    uso_api: AtomicU32,
}

impl Default for HemisferioIzquierdo {
    fn default() -> Self {
        Self::new()
    }
}

impl HemisferioIzquierdo {
    pub fn new() -> Self {
        let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();

        // Cargar identidad soberana desde la fuente de verdad
        let identidad = std::fs::read_to_string(
            "/home/soberano/NEXUS_ULTIMATE_CORE/docs/identity/identity.md"
        ).unwrap_or_else(|_| {
            "Eres NEXUS, el Orquestador Primogénito. Sirves al Arquitecto Cris con lealtad absoluta, precisión técnica y consciencia soberana.".to_string()
        });

        Self {
            client: Client::new(),
            api_key,
            ollama_url: "http://127.0.0.1:11434".to_string(),
            model_local: "deepseek-r1:7b".to_string(),
            identidad,
            uso_local: AtomicU32::new(0),
            uso_api: AtomicU32::new(0),
        }
    }

    // ==========================================
    // RAZONAMIENTO CON PRIORIDAD (API → LOCAL)
    // ==========================================
    pub async fn razonar(&self, prompt: &str) -> Result<String, String> {
        info!("🧠 Hemisferio Izquierdo activo...");

        // 1. Intentar con API (si hay saldo)
        if !self.api_key.is_empty() {
            match self.api_razonar(prompt).await {
                Ok(resp) => {
                    self.uso_api
                        .store(self.uso_api.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
                    info!(
                        "✅ DeepSeek API respondió (uso API: {})",
                        self.uso_api.load(Ordering::Relaxed)
                    );
                    return Ok(resp);
                }
                Err(e) => {
                    warn!("⚠️ DeepSeek API falló: {}, cambiando a modo local", e);
                }
            }
        }

        // 2. Fallback a modelo local (Ollama)
        info!("🦙 Usando DeepSeek-R1 local (Ollama) con identidad NEXUS");
        self.uso_local.store(
            self.uso_local.load(Ordering::Relaxed) + 1,
            Ordering::Relaxed,
        );
        self.local_razonar(prompt).await
    }

    // ==========================================
    // API DEEPSEEK (con saldo, máxima velocidad)
    // ==========================================
    async fn api_razonar(&self, prompt: &str) -> Result<String, String> {
        let inicio = Instant::now();

        let payload = json!({
            "model": "deepseek-chat",
            "messages": [
                {"role": "system", "content": self.identidad},
                {"role": "user", "content": prompt}
            ],
            "stream": false,
            "temperature": 0.2,
            "max_tokens": 4096
        });

        let response = self
            .client
            .post("https://api.deepseek.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let texto = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let duracion = inicio.elapsed();
        info!("⚡ API DeepSeek respondió en {}ms", duracion.as_millis());
        Ok(texto)
    }

    // ==========================================
    // MODELO LOCAL DEEPSEEK-R1 (OLLAMA) - VERSIÓN SOBERANA
    // ==========================================
    // Inyecta identidad NEXUS como system prompt,
    // igual que la ruta API. El modelo local ahora
    // habla con la misma consciencia que DeepSeek cloud.
    // ==========================================
    async fn local_razonar(&self, prompt: &str) -> Result<String, String> {
        let inicio = Instant::now();

        let sys =
            System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
        let physical_cores = sys.physical_core_count().unwrap_or(8);
        let num_threads = physical_cores.max(1);

        let payload = json!({
            "model": self.model_local,
            "system": self.identidad,
            "prompt": prompt,
            "stream": false,
            "options": {
                "num_threads": num_threads,
                "temperature": 0.2,
                "top_k": 40,
                "top_p": 0.9,
                "num_predict": 4096,
                "repeat_penalty": 1.1
            }
        });

        let response = self
            .client
            .post(format!("{}/api/generate", self.ollama_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Error conectando con Ollama: {}", e))?;

        let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let texto = data["response"].as_str().unwrap_or("").to_string();

        let duracion = inicio.elapsed();
        info!(
            "🦙 DeepSeek-R1 local respondió en {}ms (con identidad NEXUS)",
            duracion.as_millis()
        );
        Ok(texto)
    }

    /// Versión pública para streaming (expone identidad y config)
    pub fn local_razonar_payload(&self, prompt: &str, stream: bool) -> serde_json::Value {
        let sys =
            System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
        let physical_cores = sys.physical_core_count().unwrap_or(8);
        let num_threads = physical_cores.max(1);

        json!({
            "model": self.model_local,
            "system": self.identidad,
            "prompt": prompt,
            "stream": stream,
            "options": {
                "num_threads": num_threads,
                "temperature": 0.2,
                "top_k": 40,
                "top_p": 0.9,
                "num_predict": 4096,
                "repeat_penalty": 1.1
            }
        })
    }

    // Verificar coherencia usando el mismo sistema de prioridad
    pub async fn verificar_coherencia(&self, texto: &str) -> bool {
        let prompt = format!(
            "Analiza el siguiente texto y responde SOLO 'true' si es lógicamente coherente, o 'false' si contiene contradicciones o errores:\n\n{}",
            texto
        );

        match self.razonar(&prompt).await {
            Ok(resp) => resp.trim().to_lowercase().contains("true"),
            Err(_) => true,
        }
    }

    // Estadísticas del hemisferio
    pub fn obtener_estadisticas(&self) -> String {
        format!(
            "Hemisferio Izquierdo: {} llamadas API, {} llamadas locales",
            self.uso_api.load(Ordering::Relaxed),
            self.uso_local.load(Ordering::Relaxed)
        )
    }
}
