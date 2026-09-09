// 👁️ VISIÓN GRÁFICA NEXUS — Análisis Multimodal de Trading
// Gemini 1.5 Pro analizando tu HUD en tiempo real.

use crate::energia::zenith_pool::ZenithPool;
use std::sync::Arc;
use tracing::{error, info};

pub struct VisionGrafica {
    pool: Arc<ZenithPool>,
}

impl VisionGrafica {
    pub fn new(pool: Arc<ZenithPool>) -> Self {
        Self { pool }
    }

    pub async fn analizar_hud(&self, image_path: &str) -> String {
        info!("👁️ [VISIÓN] Analizando captura del HUD: {}", image_path);

        let image_bytes = match std::fs::read(image_path) {
            Ok(b) => b,
            Err(e) => {
                error!("❌ Error leyendo captura: {}", e);
                return "Error de visión".to_string();
            }
        };

        let prompt = "Actúa como el ANALISTA TÉCNICO MAESTRO. Observa este gráfico de trading. Identifica: 1. Tendencia actual. 2. Patrones de velas críticos. 3. Niveles visuales de soporte/resistencia. 4. Veredicto visual rápido.";

        // Usamos el método analizar_imagen de ZenithPool
        self.pool
            .analizar_imagen(&image_bytes, "image/png", prompt)
            .await
    }
}
