pub mod nexus_telemetry {
    // tonic::include_proto!("nexus_telemetry");
}

/*
use nexus_telemetry::telemetry_orchestrator_server::{
    TelemetryOrchestrator, TelemetryOrchestratorServer,
};
use nexus_telemetry::{Ack, HardwareState, NexusCommand, Notification};
*/

// Stub para compilación mientras se generan los protos
pub struct HardwareState {
    pub cpu_usage: f32,
    pub ram_usage: f32,
}
pub struct NexusCommand {
    pub command_type: String,
    pub payload: String,
}
pub struct Notification {
    pub title: String,
    pub body: String,
}
pub struct Ack {
    pub success: bool,
}
use std::sync::Arc;
use tokio::sync::RwLock;

// 🔮 LIQUID TELEMETRY: Adaptive Anomaly Detection
pub struct LiquidAnomalyDetector {
    cpu_history: Vec<f32>,
    ram_history: Vec<f32>,
    window_size: usize,
    threshold: f32, // Z-score threshold (e.g., 3.0)
}

impl LiquidAnomalyDetector {
    pub fn new(window_size: usize, threshold: f32) -> Self {
        Self {
            cpu_history: Vec::with_capacity(window_size),
            ram_history: Vec::with_capacity(window_size),
            window_size,
            threshold,
        }
    }

    pub fn analyze(&mut self, cpu: f32, ram: f32) -> Option<String> {
        let cpu_anomaly =
            Self::check_value(cpu, &mut self.cpu_history, self.window_size, self.threshold);
        let ram_anomaly =
            Self::check_value(ram, &mut self.ram_history, self.window_size, self.threshold);

        if cpu_anomaly {
            return Some(format!(
                "🚨 [LIQUID] CPU Anomaly Detected: {:.2}% (Z > {})",
                cpu, self.threshold
            ));
        }
        if ram_anomaly {
            return Some(format!(
                "🚨 [LIQUID] RAM Anomaly Detected: {:.2}% (Z > {})",
                ram, self.threshold
            ));
        }
        None
    }

    fn check_value(val: f32, history: &mut Vec<f32>, window_size: usize, threshold: f32) -> bool {
        if history.len() < window_size / 2 {
            history.push(val);
            return false;
        }

        let mean = history.iter().sum::<f32>() / history.len() as f32;
        let variance =
            history.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / history.len() as f32;
        let std_dev = variance.sqrt().max(0.1); // Avoid div by zero

        let z_score = (val - mean).abs() / std_dev;

        // Update window
        if history.len() >= window_size {
            history.remove(0);
        }
        history.push(val);

        z_score > threshold
    }
}

pub struct TelemetryService {
    detector: Arc<RwLock<LiquidAnomalyDetector>>,
}

/*
#[tonic::async_trait]
impl TelemetryOrchestrator for TelemetryService {
    type SyncHardwareStream =
        Pin<Box<dyn FuturesStream<Item = Result<NexusCommand, Status>> + Send>>;

    async fn sync_hardware(
        &self,
        request: Request<tonic::Streaming<HardwareState>>,
    ) -> std::result::Result<Response<<Self as TelemetryOrchestrator>::SyncHardwareStream>, Status> {
        let mut stream = request.into_inner();
        let detector = self.detector.clone();

        let output_stream = async_stream::try_stream! {
            while let Some(state) = stream.message().await? {
                println!("🔱 [TELEMETRY] Received hardware state: CPU={}, RAM={}", state.cpu_usage, state.ram_usage);

                // Analyze for anomalies
                let mut guard = detector.write().await;
                if let Some(anomaly) = guard.analyze(state.cpu_usage, state.ram_usage) {
                    println!("{}", anomaly);
                    yield NexusCommand {
                        command_type: "ALERT".to_string(),
                        payload: anomaly,
                    };
                }

                // Standard heartbeat
                yield NexusCommand {
                    command_type: "HEARTBEAT".to_string(),
                    payload: "Core is alive".to_string(),
                };
            }
        };

        Ok(Response::new(Box::pin(output_stream)))
    }

    async fn push_notification(
        &self,
        request: Request<Notification>,
    ) -> Result<Response<Ack>, Status> {
        let note = request.into_inner();
        println!(
            "🔱 [TELEMETRY] Mobile Notification Mirrored: {} - {}",
            note.title, note.body
        );

        Ok(Response::new(Ack { success: true }))
    }
}
*/

pub async fn start_telemetry_server(addr: &str) -> anyhow::Result<()> {
    let addr_parsed: std::net::SocketAddr = addr.parse()?;
    let _telemetry = TelemetryService {
        detector: Arc::new(RwLock::new(LiquidAnomalyDetector::new(50, 3.0))),
    };

    println!(
        "🔱 [TELEMETRY] Mobile Confluence Bridge starting on {}",
        addr_parsed
    );

    // tonic::transport::Server::builder()
    //    .add_service(TelemetryOrchestratorServer::new(telemetry))
    //    .serve(addr)
    //    .await?;

    Ok(())
}
