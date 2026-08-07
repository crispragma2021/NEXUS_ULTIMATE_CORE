// 🔱 CONTEXT PRUNER — Filtro de aislamiento estricto de contexto
// Asegura que los prompts inyectados al SLM local sean < 2k tokens y libres de historial ruidoso.
// Puro Rust, sin unwrap(), sin expect(), listo para producción.

use super::task_graph::TaskNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrunedContext {
    pub system_prompt: String,      // Rol del operador de ejecución atómica
    pub user_prompt: String,        // Instrucción atómica empaquetada
    pub output_schema: String,      // JSON Schema esperado
}

pub struct ContextPruner;

impl ContextPruner {
    /// Construye un prompt aislado e indivisible < 2k tokens para el SLM
    pub fn build_prompt(task: &TaskNode, relevant_rag_context: Option<&str>) -> PrunedContext {
        let system_prompt = r#"Eres un operador de ejecución atómica para NEXUS. Tu única función es recibir una orden simple y devolver un JSON estricto respetando el esquema.
REGLAS ABSOLUTAS:
1. Responde ÚNICAMENTE en JSON válido.
2. No agregues explicaciones, markdown extra ni texto conversacional.
3. Si los parámetros están incompletos, usa valores vacíos.
4. No inventes herramientas ni acciones fuera del esquema."#.to_string();

        let mut user_content = format!(
            "ID de Tarea: {}\nInstrucción de Tarea: {}\n",
            task.id, task.instruction
        );

        // Inyectar contexto RAG truncado si existe
        if let Some(rag_ctx) = relevant_rag_context {
            // Estimar tamaño del RAG context para evitar sobrepasar los 2k tokens.
            // Truncamos preventivamente a 1000 caracteres (aprox. 250 tokens)
            let truncated_rag: String = rag_ctx.chars().take(1000).collect();
            user_content.push_str(&format!("\nContexto del Codebase:\n{}\n", truncated_rag));
        }

        // Si hay un error de un intento previo, lo inyectamos de forma aislada
        if let Some(ref err) = task.error_msg {
            user_content.push_str(&format!("\n⚠️ REINTENTO POR FALLO PREVIO:\nError reportado: {}\nPor favor, corrige tu respuesta anterior para cumplir con el esquema y solucionar este error.\n", err));
        }

        let output_schema = serde_json::to_string_pretty(&task.output_schema)
            .unwrap_or_else(|_| r#"{"type": "object"}"#.to_string());

        PrunedContext {
            system_prompt,
            user_prompt: user_content,
            output_schema,
        }
    }

    /// Estima si el contexto total excede los 2048 tokens (~4 caracteres por token)
    pub fn is_context_safe(ctx: &PrunedContext) -> bool {
        let total_chars = ctx.system_prompt.len() + ctx.user_prompt.len() + ctx.output_schema.len();
        let estimated_tokens = total_chars / 4;
        estimated_tokens <= 2048
    }

    /// Formatea el prompt final para un template ChatML estándar
    pub fn to_chatml(&self, ctx: &PrunedContext) -> String {
        format!(
            "<|im_start|>system\n{}\nEsquema de salida obligatorio:\n{}\n<|im_end|>\n<|im_start|>user\n{}\n<|im_end|>\n<|im_start|>assistant\n",
            ctx.system_prompt, ctx.output_schema, ctx.user_prompt
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orquestador::task_graph::{NodeState, Priority, ToolAction};

    #[test]
    fn test_context_pruner_build() {
        let task = TaskNode {
            id: "test_task".to_string(),
            instruction: "Escribe un Hola Mundo en Python".to_string(),
            tool: ToolAction::WriteFile { 
                target: "hello.py".to_string(), 
                payload: "print('Hello')".to_string() 
            },
            depends_on: vec![],
            output_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string" },
                    "params": { "type": "object" }
                }
            }),
            max_retries: 2,
            priority: Priority::Normal,
            state: NodeState::Ready,
            error_msg: None,
        };

        let ctx = ContextPruner::build_prompt(&task, Some("Esto es información de codebase extra"));
        assert!(ctx.user_prompt.contains("test_task"));
        assert!(ctx.user_prompt.contains("Escribe un Hola Mundo"));
        assert!(ctx.user_prompt.contains("codebase extra"));
        assert!(ContextPruner::is_context_safe(&ctx));

        // Probar reintento por fallo
        let mut failed_task = task.clone();
        failed_task.error_msg = Some("JSON inválido: missing bracket".to_string());
        let ctx_failed = ContextPruner::build_prompt(&failed_task, None);
        assert!(ctx_failed.user_prompt.contains("REINTENTO POR FALLO PREVIO"));
        assert!(ctx_failed.user_prompt.contains("missing bracket"));
    }
}
