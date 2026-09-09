use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[async_trait]
pub trait HealthProbe: Send + Sync {
    async fn check(&self) -> ProbeResult;
    fn tier(&self) -> ProbeTier;
    fn nombre(&self) -> &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub nombre: String,
    pub tier: ProbeTier,
    pub passed: bool,
    pub mensaje: String,
    pub detalles: Option<serde_json::Value>,
    pub latencia_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ProbeTier {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub timestamp: String,
    pub score_global: f32,
    pub estado: HealthStatus,
    pub probes: Vec<ProbeResult>,
    pub resumen: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Degraded => write!(f, "Degraded"),
            HealthStatus::Critical => write!(f, "Critical"),
        }
    }
}

pub struct SentinelCore {
    probes: Vec<Box<dyn HealthProbe>>,
}

impl SentinelCore {
    pub fn new() -> Self {
        Self { probes: Vec::new() }
    }

    pub fn registrar_probe(&mut self, probe: Box<dyn HealthProbe>) {
        self.probes.push(probe);
    }

    pub async fn run_full_diagnostic(&self) -> HealthReport {
        let mut tasks = Vec::new();
        for probe in &self.probes {
            tasks.push(async move {
                let probe_start = Instant::now();
                let result = probe.check().await;
                let latencia_ms = probe_start.elapsed().as_millis() as u64;
                ProbeResult {
                    latencia_ms,
                    ..result
                }
            });
        }

        let results = futures::future::join_all(tasks).await;

        let score = Self::calcular_score(&results);
        let estado = Self::estado_desde_score(score);
        let resumen = Self::generar_resumen(&results, &estado);

        HealthReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            score_global: score,
            estado,
            probes: results,
            resumen,
        }
    }

    pub async fn run_tier(&self, target_tier: ProbeTier) -> Vec<ProbeResult> {
        let mut tasks = Vec::new();

        for probe in &self.probes {
            if probe.tier() == target_tier {
                tasks.push(async move {
                    let probe_start = Instant::now();
                    let result = probe.check().await;
                    let latencia_ms = probe_start.elapsed().as_millis() as u64;
                    ProbeResult {
                        latencia_ms,
                        ..result
                    }
                });
            }
        }

        futures::future::join_all(tasks).await
    }

    fn calcular_score(probes: &[ProbeResult]) -> f32 {
        let mut total_weight = 0.0;
        let mut passed_weight = 0.0;

        for probe in probes {
            let weight = match probe.tier {
                ProbeTier::Critical => 0.6,
                ProbeTier::Warning => 0.3,
                ProbeTier::Info => 0.1,
            };
            total_weight += weight;
            if probe.passed {
                passed_weight += weight;
            }
        }

        if total_weight == 0.0 {
            1.0
        } else {
            passed_weight / total_weight
        }
    }

    fn estado_desde_score(score: f32) -> HealthStatus {
        if score >= 0.9 {
            HealthStatus::Healthy
        } else if score >= 0.5 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Critical
        }
    }

    fn generar_resumen(probes: &[ProbeResult], estado: &HealthStatus) -> String {
        let failed_count = probes.iter().filter(|p| !p.passed).count();
        let total_count = probes.len();
        match estado {
            HealthStatus::Healthy => format!(
                "Sistema operativo normal. {}/{} probes pasaron.",
                total_count, total_count
            ),
            HealthStatus::Degraded => format!(
                "Degradado. {}/{} probes fallaron. Posible impacto en funcionalidad parcial.",
                failed_count, total_count
            ),
            HealthStatus::Critical => format!(
                "Crítico. {}/{} probes fallaron. Se requiere intervención inmediata.",
                failed_count, total_count
            ),
        }
    }
}
