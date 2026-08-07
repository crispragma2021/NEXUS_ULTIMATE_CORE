// 🔱 FEEDBACK BUS — Canal de eventos asíncronos entre Planificador y Orquestador
// Permite que el Planificador (en nube) envíe misiones y reciba las métricas y feedbacks de los operadores.
// Puro Rust, sin unwrap(), sin expect(), listo para producción.

use crate::efectores::agente_ejecutor::ToolResponse;
use crate::orquestador::task_graph::TaskNode;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, Receiver, Sender};

/// Evento de despacho: Planificador -> Orquestador
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDispatch {
    pub dag_id: String,
    pub task: TaskNode,
    pub relevant_files: Vec<String>,
}

/// Evento de resultado: Orquestador -> Planificador
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub dag_id: String,
    pub task_id: String,
    pub success: bool,
    pub output: ToolResponse,
    pub metrics: ExecutionMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub inference_time_ms: u64,
    pub total_attempts: u8,
}

pub struct FeedbackBus {
    dispatch_tx: Sender<TaskDispatch>,
    result_tx: Sender<TaskResult>,
}

impl FeedbackBus {
    /// Inicializa un bus de comunicación bidireccional asíncrono
    pub fn new(buffer_size: usize) -> (Self, Receiver<TaskDispatch>, Receiver<TaskResult>) {
        let (dispatch_tx, dispatch_rx) = mpsc::channel(buffer_size);
        let (result_tx, result_rx) = mpsc::channel(buffer_size);

        let bus = Self {
            dispatch_tx,
            result_tx,
        };

        (bus, dispatch_rx, result_rx)
    }

    /// Envía una tarea de despacho desde el Planificador
    pub async fn dispatch_task(&self, dispatch: TaskDispatch) -> anyhow::Result<()> {
        self.dispatch_tx.send(dispatch).await
            .map_err(|e| anyhow::anyhow!("Fallo al enviar despacho por el FeedbackBus: {}", e))
    }

    /// Envía un resultado de ejecución hacia el Planificador
    pub async fn send_result(&self, result: TaskResult) -> anyhow::Result<()> {
        self.result_tx.send(result).await
            .map_err(|e| anyhow::anyhow!("Fallo al enviar resultado por el FeedbackBus: {}", e))
    }
}
