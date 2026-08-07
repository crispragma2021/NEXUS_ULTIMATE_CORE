// ==========================================
// 🛰️ BUS NEURONAL — Comunicación Inter-Agente
// ==========================================
// El sistema de mensajería asíncrona que permite
// al Escuadrón NEXUS colaborar en tiempo real.
// ==========================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

/// Tipos de mensajes que fluyen por el bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TipoMensaje {
    Comando,
    Reporte,
    Alerta,
    Sincronizacion,
    Delegacion,
}

/// Mensaje neuronal básico
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MensajeNeuronal {
    pub id: Uuid,
    pub emisor: String,
    pub receptor: Option<String>, // None = Broadcast
    pub tipo: TipoMensaje,
    pub contenido: String,
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl MensajeNeuronal {
    pub fn nuevo(emisor: &str, tipo: TipoMensaje, contenido: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            emisor: emisor.to_string(),
            receptor: None,
            tipo,
            contenido: contenido.to_string(),
            metadata: serde_json::json!({}),
            timestamp: Utc::now(),
        }
    }

    pub fn a_receptor(mut self, receptor: &str) -> Self {
        self.receptor = Some(receptor.to_string());
        self
    }
}

/// El Bus Neuronal propiamente dicho
pub struct BusNeuronal {
    tx: broadcast::Sender<MensajeNeuronal>,
}

impl BusNeuronal {
    pub fn new(capacidad: usize) -> Self {
        let (tx, _) = broadcast::channel(capacidad);
        Self { tx }
    }

    pub fn enviar(&self, mensaje: MensajeNeuronal) -> Result<usize, String> {
        self.tx.send(mensaje).map_err(|e| e.to_string())
    }

    pub fn subscribirse(&self) -> broadcast::Receiver<MensajeNeuronal> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bus_neuronal_broadcast() {
        let bus = BusNeuronal::new(10);
        let mut rx1 = bus.subscribirse();
        let mut rx2 = bus.subscribirse();

        let msg = MensajeNeuronal::nuevo("orquestador", TipoMensaje::Comando, "iniciar_mision");
        bus.enviar(msg).unwrap();

        let res1 = rx1.recv().await.unwrap();
        let res2 = rx2.recv().await.unwrap();

        assert_eq!(res1.emisor, "orquestador");
        assert_eq!(res2.emisor, "orquestador");
        assert_eq!(res1.contenido, "iniciar_mision");
    }
}
