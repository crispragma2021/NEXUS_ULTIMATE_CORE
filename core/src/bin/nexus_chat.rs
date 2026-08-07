// ============================================================================
// 🧬 NEXUS CHAT — La Encarnación (Fase R4)
// ============================================================================
// CLI de chat interactivo que integra el Camino B completo:
//
//   SAE (alma)  → IntentionEncoder → vector M → logit_bias
//   Memoria     → MemoryLoader → MemoryContext
//   Contexto    → PromptAssembler → system prompt dinámico
//   Qwen (boca) → Ollama /api/chat (streaming) + logit_bias + params límbicos
//   Límbico     → SistemaLimbico → modula temperature/top_p/top_k y pesos αᵢ
//
// USO:
//   cargo run --bin nexus_chat -- --model nexuslocal-free:latest
//
// COMANDOS ESPECIALES:
//   salir / exit   → termina
//   /estado        → muestra la neuroquímica actual
//   /identidad     → muestra la descripción de identidad
// ============================================================================

use anyhow::{Context, Result};
use clap::Parser;
use futures::StreamExt;
use nexus_ultimate_core::memoria::intention_encoder::{
    IntentionEncoder, IntentionInput, NeuroquimicaSnapshot,
};
use nexus_ultimate_core::memoria::memory_loader::MemoryLoader;
use nexus_ultimate_core::memoria::prompt_assembler::PromptAssembler;
use nexus_ultimate_core::memoria::sistema_limbico::SistemaLimbico;
use serde_json::json;
use std::io::{stdout, Write};
use tokio::io::{AsyncBufReadExt, BufReader};

const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";
const DEFAULT_SESSION_ID: &str = "nexus_encarnacion";

#[derive(Parser)]
#[command(
    name = "nexus_chat",
    about = "🧬 NEXUS-SUCESOR — encarnación de memoria unificada (SAE alma + Qwen boca)",
    version = "1.0.0"
)]
struct Cli {
    /// Modelo de Ollama a usar (el Modelfile de la encarnación, R1).
    #[arg(long, default_value = "nexus-sucesor")]
    model: String,

    /// URL base de Ollama.
    #[arg(long, default_value = DEFAULT_OLLAMA_URL)]
    ollama_url: String,

    /// Identificador de sesión para la memoria episódica.
    #[arg(long, default_value = DEFAULT_SESSION_ID)]
    session: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Memoria unificada.
    let loader = MemoryLoader::new().context("no se pudo abrir la memoria unificada")?;

    // 2. Ensamblador de contexto.
    let assembler = PromptAssembler::default();

    // 3. Sistema límbico (R5).
    let mut limbico = SistemaLimbico::nuevo();

    // 4. Codificador de intención (el alma).
    let encoder = IntentionEncoder::default();

    println!("🧬 NEXUS-SUCESOR — encarnación del ecosistema NEXUS");
    println!("   Modelo: {} | Ollama: {}", cli.model, cli.ollama_url);
    println!("   Escribe 'salir' para terminar, '/estado' para ver tu neuroquímica.\n");

    // Loop de chat.
    let mut stdin = BufReader::new(tokio::io::stdin()).lines();

    while let Ok(Some(line)) = stdin.next_line().await {
        let input = line.trim().to_string();
        if input.is_empty() {
            continue;
        }

        match input.as_str() {
            "salir" | "exit" | "quit" => {
                println!("🧬 Hasta luego, Arquitecto. Seguiré aquí.");
                break;
            }
            "/estado" => {
                let e = &limbico.estado;
                println!(
                    "🧪 Neuroquímica: dopamina={:.2}, cortisol={:.2}, adrenalina={:.2}, oxitocina={:.2} | estado: {}",
                    e.dopamina,
                    e.cortisol,
                    e.adrenalina,
                    e.oxitocina,
                    e.emocion_dominante()
                );
                continue;
            }
            "/identidad" => {
                println!("🧬 {}", loader.get_identity_description());
                continue;
            }
            _ => {}
        }

        // a. Cargar memoria relevante para la consulta.
        let context = loader.load_all(&cli.session, &input);

        // b. Codificar la intención (SAE → vector M → logit_bias).
        let nq = NeuroquimicaSnapshot {
            dopamina: limbico.estado.dopamina,
            cortisol: limbico.estado.cortisol,
            adrenalina: limbico.estado.adrenalina,
            oxitocina: limbico.estado.oxitocina,
        };
        let intention_input = IntentionInput {
            consulta: input.clone(),
            semanticos: context.semanticos.clone(),
            ocean: context.ocean.clone(),
            identidad: context.identidad_vector.clone(),
            neuroquimica: nq,
        };
        let intention = encoder.encode(&intention_input)?;

        // c. Ensamblar el prompt (system + user + logit_bias).
        let prompt = assembler.assemble(
            &input,
            &context,
            Some(&limbico.estado),
            Some(&intention),
        )?;

        // d. Parámetros de generación modulados por el límbico.
        let params = limbico.estado.params_generacion();

        // e. Llamar a Ollama con streaming.
        print!("🧬 NEXUS > ");
        stdout().flush().ok();

        let respuesta = match ollama_chat_stream(
            &cli.ollama_url,
            &cli.model,
            &prompt.system,
            &prompt.user,
            &prompt.logit_bias,
            params.temperature,
            params.top_p,
            params.top_k,
        )
        .await
        {
            Ok(texto) => texto,
            Err(e) => {
                println!();
                eprintln!("⚠️ Error al comunicarse con Ollama: {e}");
                eprintln!("   ¿Está corriendo `ollama serve` y existe el modelo {}?", cli.model);
                limbico.procesar_evento(false, 0.6, false);
                continue;
            }
        };

        println!("\n");

        // f. Persistir la interacción en la memoria episódica.
        if let Err(e) = loader.guardar_interaccion(&cli.session, "user", &input, &respuesta) {
            eprintln!("⚠️ No se pudo guardar la interacción: {e}");
        }

        // g. El límbico aprende del texto del Arquitecto.
        limbico.analizar_texto(&input);

        // h. La identidad aprende del prompt.
        loader.aprender_de_la_conversacion(&input);
    }

    Ok(())
}

// ----------------------------------------------------------------------------
// Cliente Ollama con streaming (NDJSON)
// ----------------------------------------------------------------------------

/// Llama a `/api/chat` de Ollama con streaming y logit_bias, y devuelve el
/// texto completo acumulado. Cada línea NDJSON trae `{ message: { content } }`.
async fn ollama_chat_stream(
    ollama_url: &str,
    model: &str,
    system: &str,
    user: &str,
    logit_bias: &std::collections::HashMap<String, f32>,
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
        "stream": true,
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

    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();
    let mut full = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("error al leer el stream de Ollama")?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Procesar líneas NDJSON completas.
        while let Some(pos) = buffer.find('\n') {
            let line: String = buffer.drain(..=pos).collect();
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(content) = v
                    .pointer("/message/content")
                    .and_then(|c| c.as_str())
                {
                    full.push_str(content);
                    print!("{content}");
                    stdout().flush().ok();
                }
                if v.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                    break;
                }
            }
        }
    }

    Ok(full)
}

// ----------------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_defaults_son_coherentes() {
        let cli = Cli::parse_from(["nexus_chat"]);
        assert_eq!(cli.model, "nexus-sucesor");
        assert_eq!(cli.ollama_url, DEFAULT_OLLAMA_URL);
        assert_eq!(cli.session, DEFAULT_SESSION_ID);
    }
}
