// ============================================================================
// 🧬 ÓRGANO DE DESCOMPOSICIÓN COGNITIVA (Chain-of-Thought)
// ============================================================================
// Potencia el modelo local 8B dividiendo problemas complejos en sub-pasos,
// ejecutándolos secuencialmente y sintetizando una respuesta coherente.
// ============================================================================

use serde::{Deserialize, Serialize};
use std::time::Instant;
use reqwest::Client;
use serde_json::json;

// ─── Tipos Públicos ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Complejidad {
    Simple,
    Complejo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRazonamiento {
    pub pasos: Vec<String>,
    pub complejidad: Complejidad,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultadoPaso {
    pub indice: usize,
    pub descripcion: String,
    pub respuesta: String,
    pub latencia_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModoRazonamiento {
    Directo,    // Sin descomposición (prompt simple)
    Razonado,   // Con descomposición completa
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespuestaRazonada {
    pub respuesta_final: String,
    pub pasos: Vec<ResultadoPaso>,
    pub latencia_total_ms: u64,
    pub modo_usado: ModoRazonamiento,
}

// ─── Helper: Parseo NDJSON ──────────────────────────────────────────────────

/// Helper público para parsear respuestas NDJSON de Ollama /api/chat.
/// Ollama SIEMPRE devuelve NDJSON (multi-line JSON) incluso con stream=false.
/// Extrae el campo `message.content` del último objeto con `done: true`.
pub async fn extraer_contenido_ollama(res: reqwest::Response) -> Result<String, String> {
    let body_text = res.text().await
        .map_err(|e| format!("Error al leer cuerpo de Ollama: {}", e))?;

    // Intentar parse directo primero (caso normal si body_text tiene un solo JSON)
    if let Ok(body) = serde_json::from_str::<serde_json::Value>(&body_text) {
        if let Some(contenido) = body["message"]["content"].as_str() {
            if !contenido.is_empty() {
                return Ok(contenido.to_string());
            }
        }
    }

    // Fallback NDJSON: iterar líneas y buscar la última con contenido no-vacío
    let mut ultimo_contenido: Option<String> = None;
    for line in body_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        if let Ok(parcial) = serde_json::from_str::<serde_json::Value>(trimmed) {
            let esta_done = parcial.get("done")
                .and_then(|d| d.as_bool())
                .unwrap_or(false);
            // Solo actualizar si content NO está vacío (la línea final con done=true suele tener content="")
            if let Some(contenido) = parcial["message"]["content"].as_str() {
                if !contenido.is_empty() {
                    ultimo_contenido = Some(contenido.to_string());
                }
            }
            if esta_done {
                break;
            }
        }
    }

    ultimo_contenido.ok_or_else(|| {
        let preview = body_text.chars().take(200).collect::<String>();
        format!("NDJSON sin objeto válido. Preview: {}", preview)
    })
}


// ─── Clasificador de Intención ──────────────────────────────────────────────

/// Clasifica si un prompt necesita descomposición CoT o es directo.
/// Todo el análisis es heurístico en Rust — NO requiere LLM.
pub fn clasificar_complejidad(prompt: &str) -> Complejidad {
    // Regla 1: Prompts cortos → simple
    if prompt.len() < 80 && prompt.split_whitespace().count() < 15 {
        return Complejidad::Simple;
    }

    // Regla 2: Palabras clave que indican complejidad
    let palabras_complejas = [
        "explica paso a paso",
        "depura",
        "debug",
        "analiza",
        "compara",
        "contrasta",
        "diseña",
        "arquitectura",
        "implementa",
        "optimiza",
        "refactoriza",
        "por qué",
        "cómo funciona",
        "paso a paso",
        "desglosa",
        "razona",
        "piensa",
        "reason",
        "step by step",
        "analyze",
        "explain",
    ];

    let prompt_lower = prompt.to_lowercase();
    if palabras_complejas.iter().any(|p| prompt_lower.contains(p)) {
        return Complejidad::Complejo;
    }

    // Regla 3: Prompts que contienen código
    if prompt.contains("fn ") || prompt.contains("```") || prompt.contains("impl ") {
        return Complejidad::Complejo;
    }

    // Por defecto, simple
    Complejidad::Simple
}


// ─── Generador de Plan (1 llamada Ollama) ───────────────────────────────────

/// Llama a Ollama para generar un plan estructurado de descomposición.
pub async fn generar_plan(
    prompt: &str,
    ollama_api_base: &str,
    ollama_model: &str,
    client: &Client,
) -> Result<PlanRazonamiento, String> {
    let system_prompt = r#"
Eres un planificador de razonamiento. Tu única tarea es descomponer
el problema siguiente en PASOS. No resuelvas el problema aún.
Devuelve SOLO un JSON array de strings, cada string es un paso.

Reglas:
- Cada paso debe ser atómico (una sola operación mental)
- Máximo 5 pasos
- El primer paso es SIEMPRE "Identificar qué información se tiene"
- El último paso es SIEMPRE "Formular respuesta final"
- Si el problema involucra código, incluye un paso de "Escribir/analizar código"
"#;

    let messages = vec![
        json!({"role": "system", "content": system_prompt}),
        json!({"role": "user", "content": prompt}),
    ];

    let request_body = json!({
        "model": ollama_model,
        "messages": messages,
        "options": {
            "temperature": 0.3,
            "top_p": 0.9,
            "top_k": 40,
        },
        "format": "json",
        "stream": false,
    });

    let res = client.post(format!("{}/api/chat", ollama_api_base))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Error al enviar solicitud a Ollama (generar_plan): {}", e))?;

    // Usar helper NDJSON en vez de res.json()
    let contenido = extraer_contenido_ollama(res).await
        .map_err(|e| format!("Error al parsear respuesta de Ollama (generar_plan): {}", e))?;

    let pasos: Vec<String> = serde_json::from_str(&contenido)
        .map_err(|e| format!("Error al parsear pasos del plan: {}. Contenido: '{}'", e, contenido))?;

    Ok(PlanRazonamiento {
        pasos,
        complejidad: Complejidad::Complejo,
    })
}


// ─── Ejecutor de Sub-pasos (N llamadas Ollama) ─────────────────────────────

/// Ejecuta cada paso del plan secuencialmente.
/// Cada paso recibe contexto de los pasos anteriores.
pub async fn ejecutar_pasos(
    prompt_original: &str,
    plan: &PlanRazonamiento,
    ollama_api_base: &str,
    ollama_model: &str,
    client: &Client,
) -> Result<Vec<ResultadoPaso>, String> {
    let mut resultados: Vec<ResultadoPaso> = Vec::new();
    let mut historial_pasos: Vec<serde_json::Value> = Vec::new();

    for (indice, paso_actual) in plan.pasos.iter().enumerate() {
        let inicio_paso = Instant::now();

        let system_prompt = r#"
Eres un ejecutor de tareas. Tu objetivo es resolver ÚNICAMENTE el paso actual.
Sé conciso y directo.
No repitas información del historial.
"#;
        
        let messages = vec![
            json!({"role": "system", "content": system_prompt}),
            json!({"role": "user", "content": format!(
                "Contexto del problema original: {}\n\nProgreso hasta ahora: {}\n\nPaso actual a ejecutar: {}",
                prompt_original,
                historial_pasos.iter().map(|m| m["content"].as_str().unwrap_or("")).collect::<Vec<&str>>().join("\n\n"),
                paso_actual
            )}),
        ];

        let request_body = json!({
            "model": ollama_model,
            "messages": messages,
            "options": {
                "temperature": 0.3,
                "top_p": 0.9,
                "top_k": 40,
                "num_predict": 512,
            },
            "stream": false,
        });

        let res = client.post(format!("{}/api/chat", ollama_api_base))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Error al enviar solicitud a Ollama (ejecutar_pasos, paso {}): {}", indice, e))?;

        // Usar helper NDJSON en vez de res.json()
        let respuesta = extraer_contenido_ollama(res).await
            .map_err(|e| format!("Error al parsear respuesta de Ollama (ejecutar_pasos, paso {}): {}", indice, e))?;
        
        historial_pasos.push(json!({"role": "assistant", "content": respuesta.clone()}));

        resultados.push(ResultadoPaso {
            indice,
            descripcion: paso_actual.clone(),
            respuesta,
            latencia_ms: inicio_paso.elapsed().as_millis() as u64,
        });
    }

    Ok(resultados)
}


// ─── Sintetizador (1 llamada Ollama) ────────────────────────────────────────

/// Sintetiza los resultados parciales en una respuesta final coherente.
pub async fn sintetizar_respuesta(
    prompt_original: &str,
    resultados: &[ResultadoPaso],
    ollama_api_base: &str,
    ollama_model: &str,
    client: &Client,
) -> Result<String, String> {
    let system_prompt = r#"
Eres un sintetizador. Tu tarea es combinar los siguientes análisis
parciales en una respuesta final coherente, completa y bien estructurada.

Reglas:
- NO repitas información innecesariamente
- Asegura transiciones suaves entre secciones
- Mantén el tono de NEXUS: técnico, conciso, respetuoso
- Si hay contradicciones entre pasos, resáltalas y resuélvelas
- La respuesta debe ser AUTOCONTENIDA (no references a los pasos internos)
"#;

    let messages = vec![
        json!({"role": "system", "content": system_prompt}),
        json!({"role": "user", "content": format!(
            "Problema original: {}\n\nResultados de los pasos intermedios:\n{}",
            prompt_original,
            resultados.iter()
                .map(|r| format!("Paso {}: {}\n{}", r.indice, r.descripcion, r.respuesta))
                .collect::<Vec<String>>()
                .join("\n\n")
        )}),
    ];

    let request_body = json!({
        "model": ollama_model,
        "messages": messages,
        "options": {
            "temperature": 0.3,
            "top_p": 0.9,
            "top_k": 40,
        },
        "stream": false,
    });

    let res = client.post(format!("{}/api/chat", ollama_api_base))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Error al enviar solicitud a Ollama (sintetizar): {}", e))?;

    // Usar helper NDJSON en vez de res.json()
    let respuesta_final = extraer_contenido_ollama(res).await
        .map_err(|e| format!("Error al parsear respuesta de Ollama (sintetizar): {}", e))?;

    Ok(respuesta_final)
}


// ─── Helper: Llamada directa a Ollama ──────────────────────────────────────

async fn llamar_ollama_directo(
    prompt: &str,
    ollama_api_base: &str,
    ollama_model: &str,
    client: &Client,
) -> String {
    let system_prompt = r#"Eres un asistente útil y conciso. Responde directamente a la pregunta."#;

    let messages = vec![
        json!({"role": "system", "content": system_prompt}),
        json!({"role": "user", "content": prompt}),
    ];

    let request_body = json!({
        "model": ollama_model,
        "messages": messages,
        "options": {
            "temperature": 0.3,
            "top_p": 0.9,
            "top_k": 40,
        },
        "stream": false,
    });

    match client.post(format!("{}/api/chat", ollama_api_base))
        .json(&request_body)
        .send()
        .await {
            Ok(res) => {
                match extraer_contenido_ollama(res).await {
                    Ok(contenido) => contenido,
                    Err(e) => format!("Error al parsear respuesta directa de Ollama: {}", e),
                }
            },
            Err(e) => format!("Error al enviar solicitud directa a Ollama: {}", e),
        }
}


// ─── Orquestador Principal ─────────────────────────────────────────────────

/// Punto de entrada único. Decide el flujo según la complejidad.
pub async fn procesar_con_cot(
    prompt: &str,
    ollama_api_base: &str,
    ollama_model: &str,
) -> RespuestaRazonada {
    let inicio = Instant::now();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180)) // Timeout extendido para CoT
        .build()
        .unwrap_or_default();

    match clasificar_complejidad(prompt) {
        Complejidad::Simple => {
            // Modo Directo: respuesta directa
            let respuesta = llamar_ollama_directo(prompt, ollama_api_base, ollama_model, &client).await;
            RespuestaRazonada {
                respuesta_final: respuesta,
                pasos: vec![],
                latencia_total_ms: inicio.elapsed().as_millis() as u64,
                modo_usado: ModoRazonamiento::Directo,
            }
        },
        Complejidad::Complejo => {
            // Modo Razonado: plan → ejecutar → sintetizar
            let plan_resultado = generar_plan(prompt, ollama_api_base, ollama_model, &client).await;

            let plan = match plan_resultado {
                Ok(p) => p,
                Err(e) => {
                    // Fallback a directo
                    let respuesta = llamar_ollama_directo(prompt, ollama_api_base, ollama_model, &client).await;
                    return RespuestaRazonada {
                        respuesta_final: respuesta,
                        pasos: vec![],
                        latencia_total_ms: inicio.elapsed().as_millis() as u64,
                        modo_usado: ModoRazonamiento::Directo,
                    };
                }
            };

            // Ejecutar pasos
            let resultados = match ejecutar_pasos(prompt, &plan, ollama_api_base, ollama_model, &client).await {
                Ok(r) => r,
                Err(e) => {
                    return RespuestaRazonada {
                        respuesta_final: format!("❌ Error durante ejecución de pasos: {}", e),
                        pasos: vec![],
                        latencia_total_ms: inicio.elapsed().as_millis() as u64,
                        modo_usado: ModoRazonamiento::Razonado,
                    };
                }
            };

            // Sintetizar
            let sintesis = match sintetizar_respuesta(prompt, &resultados, ollama_api_base, ollama_model, &client).await {
                Ok(s) => s,
                Err(e) => {
                    return RespuestaRazonada {
                        respuesta_final: format!("❌ Error durante síntesis: {}", e),
                        pasos: resultados,
                        latencia_total_ms: inicio.elapsed().as_millis() as u64,
                        modo_usado: ModoRazonamiento::Razonado,
                    };
                }
            };

            RespuestaRazonada {
                respuesta_final: sintesis,
                pasos: resultados,
                latencia_total_ms: inicio.elapsed().as_millis() as u64,
                modo_usado: ModoRazonamiento::Razonado,
            }
        }
    }
}
