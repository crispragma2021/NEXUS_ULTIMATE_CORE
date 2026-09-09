// 🔱 EXECUTION LOOP — Bucle determinista de reintentos y escalación (Validator-Refiner)
// Orquesta el flujo completo de inferencia, validación, reintentos locales y escalación.
// Puro Rust, sin unwrap(), sin expect(), listo para producción.

use crate::efectores::agente_ejecutor::ToolResponse;
use crate::orquestador::cloud_fallback::CloudFallback;
use crate::orquestador::context_pruner::ContextPruner;
use crate::orquestador::sandbox::Sandbox;
use crate::orquestador::slm_dispatcher::SLMDispatcher;
use crate::orquestador::task_graph::TaskNode;
use crate::orquestador::validator::{ValidationResult, Validator};
use anyhow::{anyhow, Result};

pub struct ExecutionLoop {
    pub validator: Validator,
    pub dispatcher: SLMDispatcher,
    pub sandbox: Sandbox,
    pub cloud_fallback: CloudFallback,
}

impl ExecutionLoop {
    /// Crea el bucle de ejecución unificando los pilares de validación, sandbox y fallback
    pub fn new(
        validator: Validator,
        dispatcher: SLMDispatcher,
        sandbox: Sandbox,
        cloud_fallback: CloudFallback,
    ) -> Self {
        Self {
            validator,
            dispatcher,
            sandbox,
            cloud_fallback,
        }
    }

    /// Ejecuta una tarea atómica con un flujo hermético de reintentos y escalación
    pub async fn execute_task(&self, task: &mut TaskNode) -> Result<ToolResponse> {
        let mut attempts = 0;
        let max_attempts = task.max_retries.max(1);
        let mut last_error = String::new();

        // ─── 1. BUCLE DE REINTENTO LOCAL CON EL OPERADOR SLM ─────────────────────────
        while attempts < max_attempts {
            attempts += 1;
            tracing::info!(
                "🔄 [LOOP] Ejecutando intento {}/{} para la tarea '{}'",
                attempts,
                max_attempts,
                task.id
            );

            // Modificar el mensaje de error en el nodo para que el prunner lo inyecte
            if !last_error.is_empty() {
                task.error_msg = Some(last_error.clone());
            }

            // Construir contexto aislado
            let pruned_ctx = ContextPruner::build_prompt(task, None);

            // Inferencia local
            let raw_output = match self.dispatcher.infer(&pruned_ctx).await {
                Ok(out) => out,
                Err(e) => {
                    last_error = format!("Fallo en Ollama/Inferencia: {}", e);
                    continue;
                }
            };

            // Validación de salida determinista
            match self.validator.validate(&raw_output) {
                ValidationResult::Valid(tool_call) => {
                    tracing::info!(
                        "✅ [VALIDADOR] Formato JSON y firma de herramienta válidos para la tarea '{}'. Ejecutando en Sandbox...",
                        task.id
                    );
                    // Ejecutar en Sandbox seguro
                    let res = self.sandbox.execute(&tool_call).await;
                    return Ok(res);
                }
                ValidationResult::InvalidJson(err) => {
                    last_error = format!("JSON inválido: {}", err);
                }
                ValidationResult::InvalidAction(err) => {
                    last_error = format!("Acción no permitida: {}", err);
                }
                ValidationResult::InvalidParams(err) => {
                    last_error = format!("Parámetros incorrectos: {}", err);
                }
                ValidationResult::InvalidPath(err) => {
                    last_error = format!("Infracción de Filesystem (Traversal): {}", err);
                }
            }
        }

        // ─── 2. AGOTADOS LOS INTENTOS LOCALES -> ESCALACIÓN INTELIGENTE A NUBE ─────────
        tracing::warn!(
            "🚨 [ESCALACIÓN] SLM local falló consecutivamente en la tarea '{}'. Escalando...",
            task.id
        );

        let fallback_res = self
            .cloud_fallback
            .execute_fallback(task, &last_error)
            .await?;

        // Validar la salida del fallback de nube
        match self.validator.validate(&fallback_res.output) {
            ValidationResult::Valid(tool_call) => {
                tracing::info!(
                    "✅ [ESCALACIÓN] Salida de nube validada con éxito. Ejecutando en Sandbox..."
                );
                let res = self.sandbox.execute(&tool_call).await;
                Ok(res)
            }
            _ => {
                // Si la salida de la nube tampoco es parseable como JSON puro, ejecutamos como comando libre mitigando riesgos
                tracing::warn!("⚠️ [ESCALACIÓN] El modelo nube no devolvió un JSON perfectamente estructurado. Retornando output crudo.");
                Ok(ToolResponse {
                    success: false,
                    output: format!(
                        "La escalación a nube no pudo estructurar la acción. Output crudo del modelo grande:\n{}",
                        fallback_res.output
                    ),
                })
            }
        }
    }
}
