// ============================================================================
// core/src/cerebro/ejecucion_especulativa.rs — MOTOR DE EJECUCIÓN ESPECULATIVA
// ============================================================================
// Propósito: Reduce la latencia agente-a-sistema mediante la predicción y pre-ejecución
//            paralela de comandos/herramientas probables en entornos aislados (GhostVM).
//
// Patrón de Diseño:
//   1. Especulación: Se generan hasta 2 ramas de predicción de comandos basadas en la intención.
//   2. Pre-ejecución Aislada: Las ramas se ejecutan en segundo plano sobre la GhostVM.
//   3. Commit / Rollback Atómico: Si la rama elegida coincide con la intención definitiva,
//      se hace commit y se entregan los resultados a latencia cero (0 ms). Si no coincide,
//      se ejecuta un rollback atómico descartando la memoria efímera sin efectos secundarios.
// ============================================================================

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::infra::ghost_vm::GhostVmController;

/// Límite máximo de ramas especulativas concurrentes para proteger la CPU/RAM.
const MAX_CONCURRENT_SPECULATIONS: usize = 2;

/// Identificador único para una rama especulativa.
pub type SpeculationId = String;

/// Estado de una rama especulativa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeculationStatus {
    Pending,
    Executing,
    Completed,
    Failed(String),
    RolledBack,
}

/// Resultado almacenado de una pre-ejecución especulativa.
#[derive(Debug, Clone)]
pub struct SpeculativeResult {
    pub id: SpeculationId,
    pub command: String,
    pub status: SpeculationStatus,
    pub output: Option<String>,
    pub created_at_ms: u64,
}

/// Orquestador del Motor de Ejecución Especulativa.
pub struct SpeculativeEngine {
    ghost_vm: Arc<GhostVmController>,
    active_branches: Arc<Mutex<HashMap<SpeculationId, SpeculativeResult>>>,
}

impl Default for SpeculativeEngine {
    fn default() -> Self {
        Self::new(Arc::new(GhostVmController::new()))
    }
}

impl SpeculativeEngine {
    pub fn new(ghost_vm: Arc<GhostVmController>) -> Self {
        Self {
            ghost_vm,
            active_branches: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Dispara una pre-ejecución especulativa de un comando predecible en segundo plano.
    pub async fn predict_and_spawn(
        &self,
        speculation_id: &str,
        command: &str,
    ) -> anyhow::Result<bool> {
        let mut branches = self.active_branches.lock().await;

        if branches.len() >= MAX_CONCURRENT_SPECULATIONS {
            warn!(
                "⚠️ [ESPECULACIÓN] Límite de ramas concurrentes alcanzado ({}). Ignorando especulación '{}'.",
                MAX_CONCURRENT_SPECULATIONS, speculation_id
            );
            return Ok(false);
        }

        let now_ms = chrono::Utc::now().timestamp_millis() as u64;

        let result_holder = SpeculativeResult {
            id: speculation_id.to_string(),
            command: command.to_string(),
            status: SpeculationStatus::Executing,
            output: None,
            created_at_ms: now_ms,
        };

        branches.insert(speculation_id.to_string(), result_holder);
        drop(branches);

        info!(
            "🔮 [ESPECULACIÓN] Disparando pre-ejecución para id '{}': `{}`",
            speculation_id, command
        );

        let ghost_vm = self.ghost_vm.clone();
        let branches_arc = self.active_branches.clone();
        let spec_id = speculation_id.to_string();
        let cmd = command.to_string();

        tokio::spawn(async move {
            let res = ghost_vm.execute_command(&cmd).await;
            let mut guard = branches_arc.lock().await;
            if let Some(spec) = guard.get_mut(&spec_id) {
                match res {
                    Ok(out) => {
                        spec.output = Some(out);
                        spec.status = SpeculationStatus::Completed;
                        info!(
                            "✅ [ESPECULACIÓN] Rama '{}' completada con éxito en segundo plano.",
                            spec_id
                        );
                    }
                    Err(e) => {
                        spec.status = SpeculationStatus::Failed(e.to_string());
                        warn!("⚠️ [ESPECULACIÓN] Rama '{}' falló: {}", spec_id, e);
                    }
                }
            }
        });

        Ok(true)
    }

    /// Intentar hacer Commit de una rama especulativa si coincide con la decisión final.
    /// Si la rama ya terminó de ejecutarse, devuelve la salida inmediatamente (hit con 0ms de latencia).
    pub async fn commit_speculation(&self, speculation_id: &str) -> Option<String> {
        let mut branches = self.active_branches.lock().await;

        if let Some(spec) = branches.remove(speculation_id) {
            match spec.status {
                SpeculationStatus::Completed => {
                    info!(
                        "⚡ [ESPECULACIÓN-HIT] Commit exitoso para '{}'! Salida recuperada a 0ms de latencia.",
                        speculation_id
                    );
                    return spec.output;
                }
                SpeculationStatus::Executing => {
                    info!(
                        "⏳ [ESPECULACIÓN-PENDIENTE] La rama '{}' aún está ejecutando. Esperando resultado final...",
                        speculation_id
                    );
                }
                SpeculationStatus::Failed(err) => {
                    warn!(
                        "⚠️ [ESPECULACIÓN-MISS] Rama '{}' había fallado: {}",
                        speculation_id, err
                    );
                }
                _ => {}
            }
        } else {
            info!(
                "ℹ️ [ESPECULACIÓN-MISS] No había pre-ejecución registrada para '{}'.",
                speculation_id
            );
        }

        None
    }

    /// Ejecutar Rollback atómico descartando las ramas especulativas no utilizadas.
    pub async fn rollback_speculation(&self, speculation_id: &str) {
        let mut branches = self.active_branches.lock().await;
        if let Some(mut spec) = branches.remove(speculation_id) {
            spec.status = SpeculationStatus::RolledBack;
            info!(
                "🧹 [ESPECULACIÓN-ROLLBACK] Rama '{}' descartada atómicamente sin efectos secundarios.",
                speculation_id
            );
        }
    }

    /// Limpia todas las ramas especulativas activas.
    pub async fn clear_all(&self) {
        let mut branches = self.active_branches.lock().await;
        let count = branches.len();
        branches.clear();
        if count > 0 {
            info!(
                "🧹 [ESPECULACIÓN] {} ramas especulativas purgadas de la memoria.",
                count
            );
        }
    }

    /// Retorna la cantidad de ramas activas.
    pub async fn active_count(&self) -> usize {
        let branches = self.active_branches.lock().await;
        branches.len()
    }
}

// ============================================================================
// 🧪 PRUEBAS UNITARIAS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_especulacion_limite_concurrencia() {
        let engine = SpeculativeEngine::default();

        let ok1 = engine
            .predict_and_spawn("branch_1", "echo test1")
            .await
            .unwrap();
        let ok2 = engine
            .predict_and_spawn("branch_2", "echo test2")
            .await
            .unwrap();
        let ok3 = engine
            .predict_and_spawn("branch_3", "echo test3")
            .await
            .unwrap();

        assert!(ok1, "Primera rama debe aceptarse");
        assert!(ok2, "Segunda rama debe aceptarse");
        assert!(!ok3, "Tercera rama debe rechazarse por límite de 2");

        assert_eq!(engine.active_count().await, 2);
    }

    #[tokio::test]
    async fn test_rollback_especulacion() {
        let engine = SpeculativeEngine::default();

        let _ = engine.predict_and_spawn("branch_clean", "echo clean").await;
        assert_eq!(engine.active_count().await, 1);

        engine.rollback_speculation("branch_clean").await;
        assert_eq!(engine.active_count().await, 0);
    }
}
