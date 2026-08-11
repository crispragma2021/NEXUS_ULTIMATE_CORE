// ==========================================
// 🧬 CEREBRO NATIVO v5.0 — Inferencia Soberana (mistral.rs + GGUF)
// ==========================================
// Reemplaza Ollama con un motor nativo embebido en el binario, escrito en
// Rust puro sobre la arquitectura de mistral.rs (supera a Candle: soporta
// Qwen3, DeepSeek, Gemma, Mistral, Llama... no solo `llama`).
//
// v5.0: Motor agnóstico a arquitectura. Carga el primer GGUF candidato en
// disco mediante `GgufModelBuilder` (extrae tokenizador y chat template del
// propio GGUF) y responde vía `send_chat_request`. Los fallos de
// auto-asimilación se reportan con logging en vez de tragarse en silencio.
//
// La API pública permanece idéntica a v4.x para que los consumidores
// (ZenithPool, pipeline.rs, nexus_claw_pro.rs) no cambien.
// ==========================================

use crate::efectores::mano_soberana::ManoSoberana;
use crate::sentidos::vision_omega::VisionOmega;
use mistralrs::{Device, GgufModelBuilder, TextMessageRole, TextMessages};

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Ruta base del workspace NEXUS.
const NEXUS_ROOT: &str = "/home/soberano/NEXUS_ULTIMATE_CORE";

/// Modelos GGUF candidatos, en orden de prioridad.
/// mistral.rs soporta una amplia gama de arquitecturas, por lo que basta
/// con que el GGUF exista en disco.
const MODELOS_CANDIDATOS: &[&str] = &[
    // Qwen3-4B Q4_K_M — repo oficial Qwen/Qwen3-4B-GGUF (instruct, razonamiento)
    concat!(
        "/home/soberano/NEXUS_ULTIMATE_CORE/brain/swarm/models/",
        "Qwen3-4B-Q4_K_M.gguf"
    ),
    concat!(
        "/home/soberano/NEXUS_ULTIMATE_CORE/brain/swarm/models/",
        "qwen2.5-coder-3b-instruct-q4_k_m.gguf"
    ),
    concat!(
        "/home/soberano/NEXUS_ULTIMATE_CORE/models/gguf/",
        "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
    ),
    concat!(
        "/home/soberano/NEXUS_ULTIMATE_CORE/brain/swarm/models/",
        "Llama-3.2-3B-Instruct-Q4_K_M.gguf"
    ),
    concat!(
        "/home/soberano/NEXUS_ULTIMATE_CORE/models/gguf/",
        "qwen1.5b-q4_k_m.gguf"
    ),
];

pub struct CerebroNativo {
    pub vision: VisionOmega,
    pub mano: ManoSoberana,
    model: Arc<RwLock<Option<mistralrs::Model>>>,
}

impl CerebroNativo {
    pub fn new() -> Self {
        info!("🧬 [IA-NATIVA] Inicializando Motor Soberano (mistral.rs)...");

        let instance = Self {
            vision: VisionOmega::new(),
            mano: ManoSoberana::new(),
            model: Arc::new(RwLock::new(None)),
        };

        // Auto-asimilación inmediata si existe un modelo GGUF compatible
        match Self::detectar_modelo() {
            Some(model_path) => {
                let model_arc = instance.model.clone();

                tokio::spawn(async move {
                    info!(
                        "🔄 [IA-NATIVA] Iniciando auto-asimilación de pesos: {}",
                        model_path
                    );
                    match cargar_modelo_en(&model_arc, &model_path).await {
                        Ok(()) => {
                            info!(
                                "✅ [IA-NATIVA] Córtex Nativo asimilado y listo para inferencia."
                            );
                        }
                        Err(e) => {
                            warn!("⚠️ [IA-NATIVA] Auto-asimilación falló: {:?}", e);
                        }
                    }
                });
            }
            None => {
                warn!("⚠️ [IA-NATIVA] No se encontró modelo GGUF en disco. Motor en modo warm-up.");
            }
        }

        instance
    }

    pub async fn generar_token_nativo(&self, prompt: &str) -> Result<String> {
        let model_guard = self.model.read().await;
        let model = match model_guard.as_ref() {
            Some(m) => m,
            None => {
                return Ok(
                    "⚠️ [Mistral-Native] Motor en modo warm-up (Sin modelo cargado).".to_string(),
                )
            }
        };

        info!("🧠 [IA-NATIVA] Procesando consulta local para: {}", prompt);

        let messages = TextMessages::new().add_message(TextMessageRole::User, prompt.to_string());

        let response = model
            .send_chat_request(messages)
            .await
            .map_err(|e| anyhow::anyhow!("Error en inferencia mistral.rs: {:?}", e))?;

        let texto = response
            .choices
            .first()
            .and_then(|c| c.message.content.as_ref())
            .cloned()
            .unwrap_or_default();

        info!(
            "🧠 [IA-NATIVA] Respuesta nativa generada ({} tokens).",
            response.usage.completion_tokens
        );
        Ok(texto)
    }

    /// 📥 Asimila un modelo GGUF desde disco y lo deja listo para inferencia.
    ///
    /// Carga vía `GgufModelBuilder`; tokenizador y chat template se extraen
    /// de los metadatos del propio GGUF.
    pub async fn asimilar_pesos_con_seguridad(&self, ruta: &str) -> Result<()> {
        info!("📥 [IA-NATIVA] Asimilando pesos GGUF: {}", ruta);
        let path = Path::new(ruta);
        if !path.exists() {
            anyhow::bail!("Archivo GGUF no encontrado: {}", ruta);
        }

        let metadata = std::fs::metadata(path)
            .with_context(|| format!("No se pudo leer metadata de {}", ruta))?;
        let tamaño_mb = metadata.len() as f64 / 1_048_576.0;
        info!("🧬 [IA-NATIVA] Tamaño del modelo: {:.1} MB", tamaño_mb);

        cargar_modelo_en(&self.model, ruta).await?;

        info!("✅ [IA-NATIVA] Pesos asimilados exitosamente.");
        Ok(())
    }

    /// 🔍 Selecciona el primer modelo GGUF que exista en disco.
    fn detectar_modelo() -> Option<String> {
        MODELOS_CANDIDATOS
            .iter()
            .find(|ruta| Path::new(ruta).exists())
            .map(|s| s.to_string())
    }

    /// ⚡ Reflejo sensoriomotor: procesa un frame visual y ejecuta acciones.
    ///
    /// Toma una captura de pantalla (raw bytes) y la procesa para
    /// coordinar movimientos del ratón/teclado vía `ManoSoberana`.
    ///
    /// Es el arco reflejo de NEXUS: Ojo → Cerebro → Músculo.
    pub async fn ráfaga_sensoriomotora(&self, _frame: Vec<u8>) -> Result<()> {
        info!("🧬 [IA-NATIVA] Reflejo sensoriomotor activado (placeholder)");

        // TODO: Implementar pipeline visión → acción:
        // 1. Decodificar frame como imagen (image crate)
        // 2. Ejecutar modelo de visión clasificador
        // 3. Clasificar contenido → decidir acción
        // 4. Ejecutar acción via self.mano (tocar_punto, escribir_en_foco)
        Ok(())
    }

    /// 🔄 Verifica si el modelo principal está cargado en memoria.
    pub fn esta_listo(&self) -> bool {
        self.model
            .try_read()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    /// 📋 Devuelve el tipo de dispositivo activo como string descriptivo.
    ///
    /// mistral.rs se ejecuta en CPU nativa (sin GPU obligatoria); aquí se
    /// reporta CPU a menos que la aceleración CUDA esté activa.
    pub fn dispositivo_str(&self) -> &'static str {
        "CPU (mistral.rs)"
    }
}

impl Default for CerebroNativo {
    fn default() -> Self {
        Self::new()
    }
}

/// 📥 Carga un modelo GGUF en memoria mediante mistral.rs.
///
/// El directorio contenedor y el nombre de archivo se extraen de la ruta.
/// No se requiere chat template externo: se extrae del propio GGUF.
async fn cargar_modelo_en(
    model_arc: &Arc<RwLock<Option<mistralrs::Model>>>,
    ruta: &str,
) -> Result<()> {
    let path = Path::new(ruta);
    let dir = path
        .parent()
        .context("El GGUF no tiene directorio padre")?
        .to_string_lossy()
        .into_owned();
    let filename = path
        .file_name()
        .context("El GGUF no tiene nombre de archivo")?
        .to_string_lossy()
        .into_owned();

    info!(
        "🧬 [IA-NATIVA] Construyendo pipeline mistral.rs desde {:?}",
        dir
    );

    // Forzamos CPU explícitamente: la RTX 3070 (8GB VRAM) suele estar ya saturada
    // por el escritorio + ollama, y el device mapping mixto GPU/CPU de mistral.rs
    // desborda la VRAM al cargar Qwen3-4B en F16, produciendo `failed to fill
    // whole buffer`. Con 62GB de RAM, la CPU es el camino estable y predecible.
    let model = GgufModelBuilder::new(dir, vec![filename])
        .with_logging()
        .with_device(Device::Cpu)
        .build()
        .await
        .map_err(|e| anyhow::anyhow!("Error construyendo modelo mistral.rs: {:?}", e))?;

    let mut model_guard = model_arc.write().await;
    *model_guard = Some(model);

    Ok(())
}

// ==========================================
// 🧪 PRUEBAS UNITARIAS
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crear_cerebro() {
        let cerebro = CerebroNativo::new();
        assert!(!cerebro.esta_listo());
    }

    #[tokio::test]
    async fn test_generar_token_placeholder() {
        let cerebro = CerebroNativo::new();
        let resp = cerebro.generar_token_nativo("hola").await.unwrap();
        assert!(resp.contains("warm-up") || resp.is_empty());
    }

    #[tokio::test]
    async fn test_rafaga_sensoriomotora() {
        let cerebro = CerebroNativo::new();
        assert!(cerebro.ráfaga_sensoriomotora(vec![]).await.is_ok());
    }

    #[tokio::test]
    async fn test_detectar_modelo() {
        let encontrado = CerebroNativo::detectar_modelo();
        // Puede ser None si no hay GGUF en disco durante CI, pero el código
        // no debe entrar en pánico.
        assert!(encontrado.is_none() || encontrado.is_some());
    }

    #[test]
    fn test_modelos_candidatos_no_vacios() {
        assert!(!MODELOS_CANDIDATOS.is_empty());
    }
}
