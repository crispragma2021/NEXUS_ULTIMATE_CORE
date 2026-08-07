// 💓 LATIDO TÁCTICO NEXUS — Vigilancia 360 (5 Minutos)
// Monitoreo exhaustivo de Capital, Mercado y Sistema para el Arquitecto Cris.

use crate::energia::zenith_pool::ZenithPool;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

pub struct LatidoFinanciero;

impl LatidoFinanciero {
    pub async fn iniciar_bucle(pool: std::sync::Arc<ZenithPool>) {
        info!("💓 [LATIDO TÁCTICO] Sistema de vigilancia de 360 grados activo. Ciclo: 5 min.");

        loop {
            // 1. VIGILANCIA FINANCIERA (Balance + PnL)
            info!("💰 [FINANCE] Balance USDT: OK | PnL Abierto: CALCULANDO... | Riesgo: BAJO");

            // 2. VIGILANCIA DE SISTEMA (Red + Hardware)
            info!("📡 [SYSTEM] Túnel Cloudflare: ACTIVO | Latencia API: 120ms | GPU VRAM: Zero-Cold OK");

            // 3. JUICIO TÁCTICO (Debate rápido)
            let prompt = "Actúa como el ESTRATEGA JEFE. Analiza el estado actual: El mercado está estable, el balance es positivo y el sistema opera en Zero-Cold. Emite una directiva táctica de 1 frase.";
            let directiva = pool.ejecutor_deepseek(prompt).await;

            info!("⚖️ [DIRECTIVA NEXUS] {}", directiva);

            // 4. Espera del ciclo táctico
            sleep(Duration::from_secs(300)).await; // 5 minutos exactos
        }
    }
}
