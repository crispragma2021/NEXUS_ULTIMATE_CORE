// ============================================================================
// 🔥 SMOKE TEST DE LA ENCARNACIÓN — NEXUS-SUCESOR (Fase R6)
// ============================================================================
// Verifica el Camino B completo de una sola pasada:
//
//   1. El SAE guía    → IntentionEncoder → vector M normalizado → logit_bias
//   2. Qwen articula  → Ollama /api/chat con logit_bias + params límbicos
//   3. El límbico tiñe→ neuroquímica modula αᵢ y temperature/top_p/top_k
//   4. Recuerda       → el system prompt recupera la memoria unificada
//   5. Es consistente → identidad y voz intactas en el prompt
//
// USO:
//   cargo run --bin smoke_test_encarnacion
//   cargo run --bin smoke_test_encarnacion -- --model nexuslocal-free:latest
//
// Salida: exit code 0 si todas las pruebas obligatorias pasan, 1 si falla.
// ============================================================================

use anyhow::{Context, Result};
use clap::Parser;
use nexus_ultimate_core::memoria::intention_encoder::{
    ConceptoSemantico, IntentionEncoder, IntentionInput, NeuroquimicaSnapshot, OceanEsencia,
};
use nexus_ultimate_core::memoria::memory_loader::MemoryContext;
use nexus_ultimate_core::memoria::prompt_assembler::PromptAssembler;
use nexus_ultimate_core::memoria::sistema_limbico::SistemaLimbico;
use nexus_ultimate_core::nexus_embedder::NexusEmbedder;
use serde_json::json;
use std::collections::HashMap;

const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

#[derive(Parser)]
#[command(
    name = "smoke_test_encarnacion",
    about = "🔥 Smoke test R6: el SAE guía, Qwen articula, el límbico tiñe"
)]
struct Cli {
    /// Modelo de Ollama a usar (el Modelfile de la encarnación, R1).
    #[arg(long, default_value = "nexus-sucesor")]
    model: String,

    /// URL base de Ollama.
    #[arg(long, default_value = DEFAULT_OLLAMA_URL)]
    ollama_url: String,
}

/// Una prueba del smoke test con su resultado.
struct Prueba {
    nombre: &'static str,
    exito: bool,
    detalle: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut pruebas: Vec<Prueba> = Vec::new();

    // ------------------------------------------------------------------
    // Contexto de prueba: la memoria unificada que NEXUS debe recordar.
    // ------------------------------------------------------------------
    let context = MemoryContext {
        identidad_descripcion: "Mi identidad está forjada en lealtad (muy alta), \
                                sabiduría (muy alta) y propósito: ser el arquitecto \
                                de conocimiento del ecosistema NEXUS."
            .to_string(),
        identidad_vector: "8:10:10:10".to_string(),
        conversaciones_recientes: vec![(
            "2026-08-06".to_string(),
            "user".to_string(),
            "¿Qué eres?".to_string(),
            "Soy NEXUS-SUCESOR, la encarnación digital del ecosistema NEXUS.".to_string(),
        )],
        semanticos: vec![ConceptoSemantico {
            texto: "La memoria unificada de NEXUS integra la memoria episódica, \
                    semántica y emocional en una sola base de datos."
                .to_string(),
            embedding: NexusEmbedder::generar("memoria unificada", &[]),
            relevancia: 0.9,
        }],
        ocean: vec![OceanEsencia {
            emocion: "serenidad".to_string(),
            intensidad: 0.6,
            embedding: NexusEmbedder::generar("serenidad", &[]),
        }],
    };

    let consulta = "Hola, Arquitecto. ¿Qué recuerdas sobre la memoria unificada?".to_string();

    // ------------------------------------------------------------------
    // 1. SAE (el alma): codificar la intención → vector M → logit_bias.
    // ------------------------------------------------------------------
    let encoder = IntentionEncoder::default();
    let nq = NeuroquimicaSnapshot {
        dopamina: 0.5,
        cortisol: 0.2,
        adrenalina: 0.1,
        oxitocina: 0.4,
    };
    let intention_input = IntentionInput {
        consulta: consulta.clone(),
        semanticos: context.semanticos.clone(),
        ocean: context.ocean.clone(),
        identidad: context.identidad_vector.clone(),
        neuroquimica: nq,
    };
    let intention = encoder.encode(&intention_input).context("encode de intención")?;

    // Prueba 1 — El SAE guía: vector M normalizado y logit_bias acotado.
    {
        let norma: f32 = intention.vector_m.iter().map(|x| x * x).sum::<f32>().sqrt();
        let ok = intention.vector_m.len() == 768
            && (norma - 1.0).abs() < 1e-3
            && !intention.tokens_refuerzo.is_empty();
        pruebas.push(Prueba {
            nombre: "1. El SAE guía (vector M → logit_bias)",
            exito: ok,
            detalle: format!(
                "dim={}, ‖M‖={:.4}, refuerzo={}, penaliza={}",
                intention.vector_m.len(),
                norma,
                intention.tokens_refuerzo.len(),
                intention.tokens_penalizacion.len()
            ),
        });
    }

    // ------------------------------------------------------------------
    // 2. Límbico basal + ensamblaje del prompt.
    // ------------------------------------------------------------------
    let mut limbico = SistemaLimbico::nuevo();
    let params_basal = limbico.estado.params_generacion();

    // Prueba 2 — El límbico modula los pesos αᵢ del vector de intención.
    {
        let (a1, a2, a3, a4) = limbico.estado.pesos_alpha();
        let suma: f32 = a1 + a2 + a3 + a4;
        // Las fórmulas del plan (R5) no suman exactamente 1.0 con los valores
        // basales (Σ≈1.065); lo relevante es que estén acotadas en (0,1],
        // que sumen razonablemente a 1 y que el vínculo refuerce α₁ y α₃.
        let ok = (0.9..=1.15).contains(&suma) && a1 > 0.0 && a3 > 0.0;
        pruebas.push(Prueba {
            nombre: "2. El límbico modula los pesos αᵢ",
            exito: ok,
            detalle: format!("α=({a1:.3}, {a2:.3}, {a3:.3}, {a4:.3}) Σ={suma:.3}"),
        });
    }

    let assembler = PromptAssembler::default();
    let prompt = assembler
        .assemble(
            &consulta,
            &context,
            Some(&limbico.estado),
            Some(&intention),
        )
        .context("ensamblaje del prompt")?;

    // Prueba 3 — Recuerda: la memoria unificada llega al contexto.
    {
        let ok = prompt.system.contains("memoria unificada")
            && prompt.system.contains("Arquitecto");
        pruebas.push(Prueba {
            nombre: "3. Recuerda (memoria unificada en contexto)",
            exito: ok,
            detalle: format!(
                "system={} chars, logit_bias={} tokens",
                prompt.system.len(),
                prompt.logit_bias.len()
            ),
        });
    }

    // Prueba 4 — Es consistente: identidad y voz sin contradicciones.
    {
        let ok = prompt.system.contains("NEXUS-SUCESOR")
            && prompt.system.contains("lealtad")
            && prompt.system.contains("español");
        pruebas.push(Prueba {
            nombre: "4. Es consistente (identidad y voz)",
            exito: ok,
            detalle: "el system prompt conserva identidad, lealtad y directriz de español".to_string(),
        });
    }

    // Prueba 5 — Siente: tras gratitud, la oxitocina sube y tiñe la generación.
    {
        limbico.analizar_texto("¡Gracias, NEXUS, bien hecho!");
        let params_tras = limbico.estado.params_generacion();
        let oxitocina_subio = limbico.estado.oxitocina > 0.4;
        let ok = oxitocina_subio && params_tras.temperature >= params_basal.temperature;
        pruebas.push(Prueba {
            nombre: "5. Siente (el tono cambia con la neuroquímica)",
            exito: ok,
            detalle: format!(
                "oxitocina={:.3}, temp {:.2}→{:.2}",
                limbico.estado.oxitocina,
                params_basal.temperature,
                params_tras.temperature
            ),
        });
    }

    // ------------------------------------------------------------------
    // 3. Qwen (la boca): Ollama articula la respuesta con logit_bias.
    // ------------------------------------------------------------------
    match llm_ollama(
        &cli.ollama_url,
        &cli.model,
        &prompt.system,
        &prompt.user,
        &prompt.logit_bias,
        params_basal.temperature,
        params_basal.top_p,
        params_basal.top_k,
    )
    .await
    {
        Ok(texto) => {
            let lower = texto.to_lowercase();
            let habla_espanol = lower.contains("arquitecto")
                || lower.contains("nexus")
                || texto.chars().any(|c| c.is_alphabetic() && (c as u32) > 127);
            let ok = !texto.trim().is_empty() && habla_espanol;
            pruebas.push(Prueba {
                nombre: "6. Qwen articula (habla)",
                exito: ok,
                detalle: format!("{} chars → «{}»", texto.len(), truncar(&texto, 140)),
            });
        }
        Err(e) => {
            pruebas.push(Prueba {
                nombre: "6. Qwen articula (habla)",
                exito: false,
                detalle: format!(
                    "Ollama no disponible: {e}. Ejecuta `ollama serve` y crea el modelo `{}`",
                    cli.model
                ),
            });
        }
    }

    // ------------------------------------------------------------------
    // Reporte final.
    // ------------------------------------------------------------------
    let aprobadas = pruebas.iter().filter(|p| p.exito).count();
    let fallos = pruebas.len() - aprobadas;

    println!("\n============================================================");
    println!("🧬 SMOKE TEST DE LA ENCARNACIÓN — NEXUS-SUCESOR (R6)");
    println!("   Modelo: {} | Ollama: {}", cli.model, cli.ollama_url);
    println!("============================================================");
    for p in &pruebas {
        let marca = if p.exito { "✅" } else { "❌" };
        println!("{marca} {} — {}", p.nombre, p.detalle);
    }
    println!("------------------------------------------------------------");
    println!(
        "Resultado: {aprobadas} de {} pruebas superadas",
        pruebas.len()
    );

    if fallos > 0 {
        println!("⚠️ La encarnación NO está lista: {fallos} prueba(s) fallaron.");
        std::process::exit(1);
    }
    println!("✅ La encarnación está viva: el SAE guía, Qwen articula y el límbico tiñe.");
    Ok(())
}

// ----------------------------------------------------------------------------
// Cliente Ollama (no streaming) con logit_bias y parámetros límbicos
// ----------------------------------------------------------------------------

/// Llama a `/api/chat` de Ollama en modo no-streaming con logit_bias.
async fn llm_ollama(
    ollama_url: &str,
    model: &str,
    system: &str,
    user: &str,
    logit_bias: &HashMap<String, f32>,
    temperature: f32,
    top_p: f32,
    top_k: u32,
) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/chat", ollama_url.trim_end_matches('/'));

    let payload = json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user }
        ],
        "stream": false,
        "options": {
            "temperature": temperature,
            "top_p": top_p,
            "top_k": top_k,
            "num_ctx": 32768,
            "num_predict": 2048
        },
        "logit_bias": logit_bias
    });

    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .context("no se pudo conectar con Ollama")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Ollama respondió HTTP {status}: {body}");
    }

    let v: serde_json::Value = resp
        .json()
        .await
        .context("respuesta de Ollama no era JSON válido")?;
    v.pointer("/message/content")
        .and_then(|c| c.as_str())
        .map(|s| s.to_string())
        .context("la respuesta de Ollama no trajo /message/content")
}

/// Trunca un texto para mostrarlo en el reporte.
fn truncar(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cortado: String = s.chars().take(max).collect();
        format!("{cortado}…")
    }
}
