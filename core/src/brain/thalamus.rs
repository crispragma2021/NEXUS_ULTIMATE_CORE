// ==========================================
// 🧠 TÁLAMO — Relé sensorial central
// ==========================================
// Enruta señales sensoriales entre sentidos y corteza.
// ==========================================

use std::sync::Arc;
use tokio::sync::broadcast;

/// Señal tálamo-cortical enrutada.
#[derive(Debug, Clone)]
pub struct ThalamicSignal {
    pub origen: String,
    pub payload: String,
}

/// Tálamo: relé central de señales sensoriales.
#[derive(Clone)]
pub struct Thalamus {
    /// Canal broadcast de señales sensoriales.
    tx: broadcast::Sender<ThalamicSignal>,
}

impl Thalamus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { tx }
    }

    /// Suscribe un órgano cortical a señales sensoriales.
    pub fn subscribe(&self) -> broadcast::Receiver<ThalamicSignal> {
        self.tx.subscribe()
    }

    /// Enruta una señal sensorial hacia la corteza.
    pub fn enrutar(&self, origen: impl Into<String>, payload: impl Into<String>) {
        let _ = self.tx.send(ThalamicSignal {
            origen: origen.into(),
            payload: payload.into(),
        });
    }

    /// Latencia de polling simulada (ms). La ínsula la usa para detectar
    /// fricción sistémica.
    pub fn get_polling_ms(&self, _umbral: u64) -> u64 {
        25
    }
}

impl Default for Thalamus {
    fn default() -> Self {
        Self::new()
    }
}

// Mantiene el Arc importado (usado en firmas públicas).
#[allow(unused)]
fn _keep_arc(_: Arc<Thalamus>) {}
