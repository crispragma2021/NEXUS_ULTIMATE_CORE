// ============================================================================
// core/src/comms/protocolo_a2a.rs — PROTOCOLO AGENTE-A-AGENTE (A2A OMEGA)
// ============================================================================
// Propósito: Define la especificación estándar para la comunicación, apretón de
//            manos (handshake) y delegación transparente de tareas entre NEXUS CORE
//            y agentes externos o sub-agentes (LangGraph, CrewAI, AutoGen, Claude SDK).
//
// Protocolo:
//   - JSON-RPC 2.0 compatible over WebSocket/gRPC/BusNeuronal.
//   - A2A Handshake con intercambio de lista de herramientas/capacidades.
//   - Manejo de flujo streaming y resultados finales estructurados.
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

use crate::comms::bus_neuronal::{BusNeuronal, MensajeNeuronal, TipoMensaje};

/// Versión actual de la especificación del protocolo A2A.
pub const A2A_PROTOCOL_VERSION: &str = "2.0-omega";

/// Tipo de método JSON-RPC A2A
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum A2AMethod {
    Handshake,
    TaskDelegate,
    TaskStream,
    TaskCancel,
    Heartbeat,
}

/// Capacidades anunciadas por un agente A2A
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ACapabilities {
    pub agent_name: String,
    pub protocol_version: String,
    pub supported_tools: Vec<String>,
    pub supports_streaming: bool,
    pub max_parallel_tasks: usize,
}

/// Petición JSON-RPC 2.0 A2A
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ARequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: A2AMethod,
    pub sender_agent: String,
    pub params: serde_json::Value,
}

/// Respuesta JSON-RPC 2.0 A2A
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AResponse {
    pub jsonrpc: String,
    pub id: String,
    pub sender_agent: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<A2AError>,
}

/// Error normalizado en A2A
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AError {
    pub code: i32,
    pub message: String,
}

/// Pasarela de Protocolo A2A (A2AGateway)
pub struct A2AGateway {
    pub self_name: String,
    pub known_agents: HashMap<String, A2ACapabilities>,
}

impl Default for A2AGateway {
    fn default() -> Self {
        Self::new("NEXUS-MASTER")
    }
}

impl A2AGateway {
    pub fn new(self_name: &str) -> Self {
        Self {
            self_name: self_name.to_string(),
            known_agents: HashMap::new(),
        }
    }

    /// Genera la oferta de Handshake de NEXUS para anunciar sus capacidades a otro agente.
    pub fn build_handshake_request(&self, supported_tools: Vec<String>) -> A2ARequest {
        let caps = A2ACapabilities {
            agent_name: self.self_name.clone(),
            protocol_version: A2A_PROTOCOL_VERSION.to_string(),
            supported_tools,
            supports_streaming: true,
            max_parallel_tasks: 4,
        };

        A2ARequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string(),
            method: A2AMethod::Handshake,
            sender_agent: self.self_name.clone(),
            params: serde_json::to_value(caps).unwrap_or_default(),
        }
    }

    /// Procesa una respuesta de Handshake y registra al agente remoto en el mapa de capacidades.
    pub fn process_handshake_response(&mut self, response: &A2AResponse) -> anyhow::Result<String> {
        if let Some(ref err) = response.error {
            anyhow::bail!("A2A Handshake rechazado (código {}): {}", err.code, err.message);
        }

        let result_val = response
            .result
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Handshake sin payload de result"))?;

        let caps: A2ACapabilities = serde_json::from_value(result_val.clone())?;
        let remote_name = caps.agent_name.clone();

        info!(
            "🤝 [A2A] Handshake exitoso con agente '{}' (v{}). Tools: {:?}",
            remote_name, caps.protocol_version, caps.supported_tools
        );

        self.known_agents.insert(remote_name.clone(), caps);
        Ok(remote_name)
    }

    /// Construye una petición de Delegación de Tarea para enviar a un agente remoto subordinado.
    pub fn build_task_delegation(&self, target_agent: &str, task_prompt: &str, metadata: serde_json::Value) -> A2ARequest {
        A2ARequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string(),
            method: A2AMethod::TaskDelegate,
            sender_agent: self.self_name.clone(),
            params: serde_json::json!({
                "target_agent": target_agent,
                "task": task_prompt,
                "metadata": metadata,
            }),
        }
    }

    /// Convierte una petición A2A a un `MensajeNeuronal` para inyección en el `BusNeuronal`.
    pub fn to_neuronal_message(&self, req: &A2ARequest, target_agent: Option<&str>) -> anyhow::Result<MensajeNeuronal> {
        let payload_str = serde_json::to_string(req)?;
        let mut msg = MensajeNeuronal::nuevo(
            &self.self_name,
            TipoMensaje::Delegacion,
            &payload_str,
        );
        if let Some(target) = target_agent {
            msg = msg.a_receptor(target);
        }
        Ok(msg)
    }
}

// ============================================================================
// 🧪 PRUEBAS UNITARIAS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a2a_handshake_flow() {
        let mut master = A2AGateway::new("NEXUS-MASTER");
        let req = master.build_handshake_request(vec!["rust_compiler".to_string(), "web_search".to_string()]);

        assert_eq!(req.method, A2AMethod::Handshake);
        assert_eq!(req.sender_agent, "NEXUS-MASTER");

        // Simular respuesta del agente remoto
        let remote_caps = A2ACapabilities {
            agent_name: "SubAgent-CrewAI".to_string(),
            protocol_version: A2A_PROTOCOL_VERSION.to_string(),
            supported_tools: vec!["python_eval".to_string()],
            supports_streaming: true,
            max_parallel_tasks: 2,
        };

        let resp = A2AResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            sender_agent: "SubAgent-CrewAI".to_string(),
            result: Some(serde_json::to_value(remote_caps).unwrap()),
            error: None,
        };

        let name = master.process_handshake_response(&resp).expect("Handshake debe ser procesado");
        assert_eq!(name, "SubAgent-CrewAI");
        assert!(master.known_agents.contains_key("SubAgent-CrewAI"));
    }

    #[test]
    fn test_a2a_task_delegation_to_neuronal_msg() {
        let master = A2AGateway::new("NEXUS-MASTER");
        let req = master.build_task_delegation("Agent-Refactor", "Optimizar módulo bus", serde_json::json!({ "prioridad": 1 }));

        let neuronal_msg = master.to_neuronal_message(&req, Some("Agent-Refactor")).unwrap();

        assert_eq!(neuronal_msg.emisor, "NEXUS-MASTER");
        assert_eq!(neuronal_msg.receptor, Some("Agent-Refactor".to_string()));
        assert!(neuronal_msg.contenido.contains("Optimizar módulo bus"));
    }
}
