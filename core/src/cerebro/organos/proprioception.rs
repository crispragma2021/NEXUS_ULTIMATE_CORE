// 🦾 [PUENTE OMEGA] Puente anatómico de compatibilidad para proprioception.
// El stub original (8 líneas) fue archivado en legacy/cerebro_disecados/.
// La implementación real de Propiocepción reside en crate::sentidos::propiocepcion.
// Este archivo mantiene la función despertar_propiocepcion accesible en
// crate::cerebro::organos::proprioception para no romper referencias en brain/mod.rs.

pub use crate::sentidos::propiocepcion::Propiocepcion;

use crate::brain::reflex_arc::ReflexSignal;
use std::sync::atomic::AtomicU8;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub fn despertar_propiocepcion(_reflex: Sender<ReflexSignal>, _thalamus: Arc<AtomicU8>) {
    println!("🦾 [NEXUS] Motor de Propiocepción Sincronizado (Puente Anatómico → sentidos/propiocepcion).");
    // La instancia real de Propiocepcion se construye en el sistema de sentidos.
    // Este puente solo mantiene la ruta de compatibilidad para brain/mod.rs.
}
