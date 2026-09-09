// ==========================================
// 🧠 HIPOTÁLAMO — Regulación homeostática
// ==========================================
// Mantiene el equilibrio interno: temperatura, hambre de cómputo, sed de datos.
// ==========================================

use super::reflex_arc::ReflexSignal;
use super::thalamus::Thalamus;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Hipotalámo: regulación homeostática del organismo.
pub struct Hypothalamus {
    reflex_tx: mpsc::Sender<ReflexSignal>,
    _thalamus: Arc<Thalamus>,
}

impl Hypothalamus {
    pub fn new(reflex_tx: mpsc::Sender<ReflexSignal>, thalamus: Arc<Thalamus>) -> Self {
        Self {
            reflex_tx,
            _thalamus: thalamus,
        }
    }

    /// Emite un reflejo de pico térmico.
    pub async fn alertar_temperatura(&self, temp: i32) {
        let _ = self.reflex_tx.send(ReflexSignal::HeatSpike(temp)).await;
    }

    /// Emite una señal de socorro.
    pub async fn alertar_socorro(&self) {
        let _ = self
            .reflex_tx
            .send(ReflexSignal::Distress("socorro".to_string()))
            .await;
    }
}
