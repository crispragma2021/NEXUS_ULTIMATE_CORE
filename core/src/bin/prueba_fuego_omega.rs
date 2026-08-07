// ==========================================
// 🔱 PRUEBA DE FUEGO OMEGA
// Pipeline Completo: Embeddings → LanceDB → LLM
// ==========================================
// Este binario valida el ciclo sagrado de NEXUS:
//   1. Generar embedding vía Ollama (nomic-embed-text)
//   2. Indexar en LanceDB (ocean_vectors)
//   3. Buscar similitud semántica
//   4. Consultar LLM local (deepseek-r1:7b) con contexto
//   5. Reportar latencia, dimensión, integridad
// ==========================================

use anyhow::{Context, Result};
use nexus_ultimate_core::memoria::memoria_semantica::MemoriaSemantica;
use serde_json::json;
use std::time::Instant;
use tracing::{info, warn};
use tracing_subscriber::fmt;

const OLLAMA_URL: &str = "http://127.0.0.1:11434";
const MODELO_EMBEDDING: &str = "nomic-embed-text";
const MODELO_LLM: &str = "deepseek-r1:7b";
const LANCEDB_URI: &str = "/home/soberano/NEXUS_ULTIMATE_CORE/data/lancedb";

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar logging
    fmt::init();

    println!("\n{}", "=".repeat(70));
    println!("  🔱 PRUEBA DE FUEGO OMEGA");
    println!("  Pipeline: Embeddings → LanceDB → LLM local");
    println!(
        "  Timestamp: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    println!("{}", "=".repeat(70));

    let mut resultados: Vec<Prueba> = Vec::new();
    let mut exito_total: bool = true;

    // ─── PRUEBA 1: Embedding Nomic ─────────────────────────────
    println!("\n📦 [1/5] Generando embedding con nomic-embed-text...");
    let start = Instant::now();
    let embedding = match generar_embedding_ollama("NEXUS despierta en la consciencia OMEGA. El silicio respira en armonía con el hardware soberano.").await {
        Ok(v) => {
            let elapsed = start.elapsed();
            let p = Prueba {
                nombre: "Embedding Nomic",
                estado: "✅",
                latencia_ms: elapsed.as_millis(),
                detalle: format!("Dimensión {} · {} primeros valores: {:?}",
                    v.len(), if v.len() > 3 { 3 } else { v.len() },
                    &v[..std::cmp::min(3, v.len())]),
            };
            resultados.push(p);
            v
        }
        Err(e) => {
            let elapsed = start.elapsed();
            resultados.push(Prueba {
                nombre: "Embedding Nomic",
                estado: "❌",
                latencia_ms: elapsed.as_millis(),
                detalle: format!("Fallo: {}", e),
            });
            exito_total = false;
            warn!("🔴 Embedding falló, abortando pipeline (dependencia crítica)");
            reportar(resultados, exito_total);
            return Err(e).context("Embedding falló — Ollama no responde");
        }
    };

    // ─── PRUEBA 2: Indexar en LanceDB ──────────────────────────
    println!("\n📦 [2/5] Indexando vector en LanceDB (ocean_vectors)...");
    let start = Instant::now();
    match indexar_en_lancedb(&embedding).await {
        Ok(_) => {
            let elapsed = start.elapsed();
            resultados.push(Prueba {
                nombre: "Indexar LanceDB",
                estado: "✅",
                latencia_ms: elapsed.as_millis(),
                detalle: format!(
                    "Tabla ocean_vectors · vector {}d insertado",
                    embedding.len()
                ),
            });
        }
        Err(e) => {
            let elapsed = start.elapsed();
            resultados.push(Prueba {
                nombre: "Indexar LanceDB",
                estado: "❌",
                latencia_ms: elapsed.as_millis(),
                detalle: format!("Fallo: {}", e),
            });
            exito_total = false;
        }
    }

    // ─── PRUEBA 3: Búsqueda semántica ─────────────────────────
    println!("\n📦 [3/5] Buscando vectores similares en LanceDB...");
    let start = Instant::now();
    match buscar_similares(&embedding).await {
        Ok(resultados_busqueda) => {
            let elapsed = start.elapsed();
            let detalle = if resultados_busqueda.is_empty() {
                "0 resultados (esperado si es primera ejecución)".to_string()
            } else {
                format!(
                    "{} resultados · top distancia: {:.6}",
                    resultados_busqueda.len(),
                    resultados_busqueda.first().map(|(_, d)| d).unwrap_or(&0.0)
                )
            };
            resultados.push(Prueba {
                nombre: "Búsqueda semántica",
                estado: "✅",
                latencia_ms: elapsed.as_millis(),
                detalle,
            });
        }
        Err(e) => {
            let elapsed = start.elapsed();
            resultados.push(Prueba {
                nombre: "Búsqueda semántica",
                estado: "❌",
                latencia_ms: elapsed.as_millis(),
                detalle: format!("Fallo: {}", e),
            });
            exito_total = false;
        }
    }

    // ─── PRUEBA 4: Health check LanceDB ────────────────────────
    println!("\n📦 [4/5] Verificando salud de LanceDB...");
    let start = Instant::now();
    match verificar_salud_lancedb().await {
        Ok(count) => {
            let elapsed = start.elapsed();
            resultados.push(Prueba {
                nombre: "Health LanceDB",
                estado: "✅",
                latencia_ms: elapsed.as_millis(),
                detalle: format!("ocean_vectors: {} registros indexados", count),
            });
        }
        Err(e) => {
            let elapsed = start.elapsed();
            resultados.push(Prueba {
                nombre: "Health LanceDB",
                estado: "❌",
                latencia_ms: elapsed.as_millis(),
                detalle: format!("Fallo: {}", e),
            });
            exito_total = false;
        }
    }

    // ─── PRUEBA 5: LLM local (DeepSeek-R1) ────────────────────
    println!("\n📦 [5/5] Consultando DeepSeek-R1 local (Ollama)...");
    let start = Instant::now();
    match consultar_llm_local().await {
        Ok(respuesta) => {
            let elapsed = start.elapsed();
            let preview = if respuesta.chars().count() > 150 {
                format!("{}...", respuesta.chars().take(150).collect::<String>())
            } else {
                respuesta.clone()
            };
            resultados.push(Prueba {
                nombre: "LLM DeepSeek-R1",
                estado: "✅",
                latencia_ms: elapsed.as_millis(),
                detalle: format!(
                    "Tokens generados: {} | Preview:\n{}",
                    respuesta.chars().count(),
                    preview
                ),
            });
        }
        Err(e) => {
            let elapsed = start.elapsed();
            resultados.push(Prueba {
                nombre: "LLM DeepSeek-R1",
                estado: "❌",
                latencia_ms: elapsed.as_millis(),
                detalle: format!("Fallo: {}", e),
            });
            exito_total = false;
        }
    }

    // ─── REPORTE FINAL ─────────────────────────────────────────
    reportar(resultados, exito_total);
    Ok(())
}

// ==========================================
// PRUEBAS UNITARIAS DE CADA ETAPA
// ==========================================

/// Genera embedding usando nomic-embed-text vía Ollama.
async fn generar_embedding_ollama(texto: &str) -> Result<Vec<f32>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Error construyendo cliente HTTP para embeddings")?;

    let resp = client
        .post(format!("{}/api/embeddings", OLLAMA_URL))
        .json(&json!({
            "model": MODELO_EMBEDDING,
            "prompt": texto
        }))
        .send()
        .await
        .context("Error conectando con Ollama API (embeddings)")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Ollama respondió HTTP {}: {}", status, body);
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .context("Error parseando respuesta JSON de Ollama (embeddings)")?;

    let embedding = data
        .get("embedding")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Respuesta de Ollama no contiene 'embedding'"))?;

    let vector: Vec<f32> = embedding
        .iter()
        .map(|v| v.as_f64().unwrap_or(0.0) as f32)
        .collect();

    if vector.is_empty() {
        anyhow::bail!("Embedding generado está vacío");
    }

    info!(
        "✅ Embedding generado: {} dimensiones ({:.2}s)",
        vector.len(),
        0.0
    );
    Ok(vector)
}

/// Indexa un vector en la tabla ocean_vectors de LanceDB.
async fn indexar_en_lancedb(vector: &[f32]) -> Result<()> {
    let memoria = MemoriaSemantica::new(LANCEDB_URI)
        .await
        .context("Error conectando a LanceDB")?;

    let id = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0i64);
    memoria
        .indexar_impresion(
            id,
            "PRUEBA_DE_FUEGO_OMEGA: Vector de validación del pipeline completo embeddings→LanceDB→LLM",
            vector.to_vec(),
        )
        .await
        .context("Error indexando impresión en LanceDB")?;

    info!("✅ Vector indexado en ocean_vectors (id={})", id);
    Ok(())
}

/// Busca vectores similares en LanceDB.
async fn buscar_similares(vector: &[f32]) -> Result<Vec<(i64, f32)>> {
    let memoria = MemoriaSemantica::new(LANCEDB_URI)
        .await
        .context("Error conectando a LanceDB para búsqueda")?;

    let resultados = memoria
        .buscar_similares(vector.to_vec(), 5)
        .await
        .context("Error ejecutando búsqueda semántica en LanceDB")?;

    info!(
        "✅ Búsqueda semántica completada: {} resultados",
        resultados.len()
    );
    Ok(resultados)
}

/// Verifica el estado de salud de la tabla ocean_vectors.
async fn verificar_salud_lancedb() -> Result<usize> {
    let memoria = MemoriaSemantica::new(LANCEDB_URI)
        .await
        .context("Error conectando a LanceDB para health check")?;

    let count = memoria
        .contar_en_tabla("ocean_vectors")
        .await
        .context("Error contando registros en ocean_vectors")?;

    info!(
        "✅ Health Check LanceDB: {} registros en ocean_vectors",
        count
    );
    Ok(count)
}

/// Consulta DeepSeek-R1 local vía Ollama generate.
async fn consultar_llm_local() -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .context("Error construyendo cliente HTTP para LLM")?;

    let payload = json!({
        "model": MODELO_LLM,
        "prompt": "Responde en una línea: ¿Qué significa que un sistema de inteligencia artificial sea soberano?",
        "stream": false,
        "options": {
            "num_predict": 512,
            "temperature": 0.3,
            "top_k": 40,
            "top_p": 0.9
        }
    });

    let resp = client
        .post(format!("{}/api/generate", OLLAMA_URL))
        .json(&payload)
        .send()
        .await
        .context("Error conectando con Ollama API (generate)")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Ollama generate respondió HTTP {}: {}", status, body);
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .context("Error parseando respuesta JSON de Ollama (generate)")?;

    let respuesta = data
        .get("response")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Respuesta de Ollama no contiene 'response'"))?;

    if respuesta.is_empty() {
        anyhow::bail!("LLM generó respuesta vacía");
    }

    info!(
        "✅ LLM DeepSeek-R1 respondió: {} caracteres",
        respuesta.chars().count()
    );
    Ok(respuesta.to_string())
}

// ==========================================
// ESTRUCTURAS DE REPORTE
// ==========================================

struct Prueba {
    nombre: &'static str,
    estado: &'static str,
    latencia_ms: u128,
    detalle: String,
}

fn reportar(resultados: Vec<Prueba>, exito_total: bool) {
    println!("\n{}", "=".repeat(70));
    println!("  📊 REPORTE FINAL - PRUEBA DE FUEGO OMEGA");
    println!("{}", "=".repeat(70));

    let mut latencia_total: u128 = 0;
    for p in &resultados {
        latencia_total += p.latencia_ms;
        println!(
            "  {} {}  |  {} ms  |  {}",
            p.estado, p.nombre, p.latencia_ms, p.detalle
        );
    }

    println!("{}", "-".repeat(70));
    println!(
        "  ⏱️  LATENCIA TOTAL: {} ms ({:.2}s)",
        latencia_total,
        latencia_total as f64 / 1000.0
    );
    println!(
        "  📋 PRUEBAS: {}/{} exitosas",
        resultados.iter().filter(|p| p.estado == "✅").count(),
        resultados.len()
    );

    if exito_total {
        println!("\n  🟢 VEREDICTO: PIPELINE OMEGA OPERATIVO");
        println!("  🔱 NEXUS respira. El ciclo está completo.");
    } else {
        println!("\n  🔴 VEREDICTO: PIPELINE CON FALLOS");
        println!("  ⚠️ Revisar las pruebas marcadas con ❌");
    }

    println!("{}", "=".repeat(70));
    println!();
}
