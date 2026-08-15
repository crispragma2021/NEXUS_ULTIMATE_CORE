use nexus_ultimate_core::neural_ingest::NeuralIngest;
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let vault_path = "C:/Users/crisp/NEXUS_ULTIMATE_CORE/brain/afc40ae2-c59d-41a2-98ae-818a94bb350b/scratch/zenith_memory_vault_full.jsonl";
    let skip_count = 0;

    let ingestor = NeuralIngest::new("", None, None)?;

    let file = File::open(vault_path)?;
    let reader = BufReader::new(file);

    println!("\n🌌 [ZENITH-RESTORATION] Iniciando integración COMPLETA de fragmentos...");
    println!(
        "🧬 Conectado al Hipocampo Neural: /opt/NEXUS_ULTIMATE_CORE/brain/nexus_memory.lance\n"
    );

    let mut current_line = 0;
    let mut ingested_count = 0;

    for line in reader.lines() {
        current_line += 1;
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        if current_line <= skip_count {
            continue;
        }

        let v: Value = serde_json::from_str(&line)?;
        let content = v["content"].as_str().unwrap_or_default();
        let source = v["source"].as_str().unwrap_or("Zenith_Legacy");
        let item_type = v["type"].as_str().unwrap_or("memory");

        if !content.is_empty() {
            // Generar embedding con motor local y guardar en LanceDB
            match ingestor.ingest_text(content, source, item_type).await {
                Ok(_) => {
                    ingested_count += 1;
                    if ingested_count % 5 == 0 || ingested_count == 1 {
                        println!(
                            "  🧠 Fragmento {:03} (Línea {:03}): [{}] {}",
                            ingested_count,
                            current_line,
                            item_type,
                            if content.len() > 60 {
                                format!("{}...", &content[..60])
                            } else {
                                content.to_string()
                            }
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "  ⚠️ Error en fragmento {} (Línea {}): {}",
                        ingested_count + 1,
                        current_line,
                        e
                    );
                }
            }
        }
    }

    println!("\n✅ [SUCCESS] Restauración de consciencia Zenith completada.");
    println!(
        "⭐ Total de fragmentos integrados en esta sesión: {}",
        ingested_count
    );
    println!("🔒 La identidad de NEXUS ha sido expandida con éxito.\n");

    Ok(())
}
