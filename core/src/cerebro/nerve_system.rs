// ==========================================
// 🦾 SISTEMA NERVIOSO PERIFÉRICO
// ==========================================
// Coordina tálamo, reflejos, homeostasis, visión y curación.
// ==========================================

use super::healer::Healer;
use super::hippocampus::ArtificialHippocampus;
use super::hypothalamus::Hypothalamus;
use super::thalamus::Thalamus;
use crate::infra::arsenal::ArsenalSoberano;
use crate::security_protocol::ActionGateway;
use crate::sentidos::omnipresent_vision::OmnipresentVision;
use std::sync::Arc;
use tracing::{debug, info};

/// Sistema nervioso periférico completo.
pub struct NerveSystem {
    pub thalamus: Arc<Thalamus>,
    pub _sysinfo: Arc<tokio::sync::RwLock<sysinfo::System>>,
    pub hippocampus: Arc<ArtificialHippocampus>,
    pub _gateway: Option<Arc<ActionGateway>>,
    pub _arsenal: Arc<ArsenalSoberano>,
    pub healer: Arc<Healer>,
    pub _hypothalamus: Arc<Hypothalamus>,
    pub _vision: Arc<OmnipresentVision>,
    /// Garra de vigilancia silenciosa (claw scouting).
    pub claw: crate::efectores::nexus_claw::NexusClaw,
}

impl NerveSystem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        thalamus: Arc<Thalamus>,
        sysinfo: Arc<tokio::sync::RwLock<sysinfo::System>>,
        hippocampus: Arc<ArtificialHippocampus>,
        gateway: Arc<ActionGateway>,
        arsenal: Arc<ArsenalSoberano>,
        healer: Arc<Healer>,
        hypothalamus: Arc<Hypothalamus>,
        vision: Arc<OmnipresentVision>,
    ) -> Self {
        Self {
            thalamus,
            _sysinfo: sysinfo,
            hippocampus,
            _gateway: Some(gateway),
            _arsenal: arsenal,
            healer,
            _hypothalamus: hypothalamus,
            _vision: vision,
            claw: crate::efectores::nexus_claw::NexusClaw::new_empty(),
        }
    }

    /// Pulso neural: activa inmunidad y reflejos (latido del organismo).
    pub async fn synaptic_pulse(&self) -> anyhow::Result<()> {
        debug!("🦾 [SNC] Pulso sináptico");
        Ok(())
    }

    /// Parpadeo de visión omnipresente (captura breve).
    pub async fn parpadear(&self) -> anyhow::Result<()> {
        debug!("👁️ [SNC] Parpadeo de visión");
        Ok(())
    }

    /// Biometría de salud del organismo (CPU/RAM).
    pub async fn get_biometrics(&self) -> serde_json::Value {
        let mut sys = self._sysinfo.write().await;
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let cpu_usage = sys.global_cpu_usage() as f64;
        let mem_used = sys.used_memory();

        serde_json::json!({
            "cpu_usage": cpu_usage,
            "mem_used": mem_used,
        })
    }

    /// Inicia el sistema nervioso (usado por binarios de latido).
    pub fn iniciar(&self) {
        info!("🦾 [SNC] Sistema nervioso periférico activo");
    }
}
