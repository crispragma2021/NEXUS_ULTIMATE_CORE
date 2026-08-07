// ==========================================
// 🚀 INVOCACIÓN DE ESCUADRÓN — Gestión de Sub-Agentes
// ==========================================
// Permite al Orquestador instanciar y coordinar
// agentes especialistas para misiones complejas.
// ==========================================

use crate::cerebro::agentes::{catalogo_agentes, AgenteEspecialista, FichaAgente};
use crate::comms::bus_neuronal::{BusNeuronal, MensajeNeuronal, TipoMensaje};
use std::sync::Arc;
use tracing::info;

pub struct ComandanteEscuadron {
    bus: Arc<BusNeuronal>,
}

impl ComandanteEscuadron {
    pub fn new(bus: Arc<BusNeuronal>) -> Self {
        Self { bus }
    }

    /// Invoca a un agente para una misión específica
    pub async fn invocar_agente(
        &self,
        agente_id: AgenteEspecialista,
        mision: &str,
    ) -> Result<(), String> {
        let ficha = FichaAgente::from(agente_id);
        info!(
            "🪖 [ESCUADRÓN] Invocando a '{}' para misión: {}",
            ficha.nombre, mision
        );

        let msg = MensajeNeuronal::nuevo("orquestador", TipoMensaje::Delegacion, mision)
            .a_receptor(ficha.nombre);

        self.bus.enviar(msg).map(|_| ())
    }

    /// Analiza el prompt y decide qué agentes del escuadrón se necesitan
    pub fn seleccionar_especialistas(&self, prompt: &str) -> Vec<AgenteEspecialista> {
        let lower = prompt.to_lowercase();
        let mut elegidos = Vec::new();
        let catalogo = catalogo_agentes();

        for ficha in catalogo {
            // Heurística simple de detección de dominio por palabras clave
            let necesita_agente = match ficha.id {
                AgenteEspecialista::FrontendSpecialist => {
                    lower.contains("ui") || lower.contains("frontend") || lower.contains("css")
                }
                AgenteEspecialista::BackendSpecialist => {
                    lower.contains("api") || lower.contains("backend") || lower.contains("rust")
                }
                AgenteEspecialista::SecurityAuditor => {
                    lower.contains("seguridad")
                        || lower.contains("audit")
                        || lower.contains("vulnerabilidad")
                }
                AgenteEspecialista::PerformanceOptimizer => {
                    lower.contains("lento") || lower.contains("optimiza") || lower.contains("perf")
                }
                _ => false,
            };

            if necesita_agente {
                elegidos.push(ficha.id);
            }
        }

        elegidos
    }
}
