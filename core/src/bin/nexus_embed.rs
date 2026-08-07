// ============================================================================
// NEXUS EMBED — CLI de Embeddings Soberanos 768-dim
// ============================================================================
// Toma texto por stdin (o primer argumento), devuelve 768 floats separados
// por espacio en stdout. Diseñado para pipe desde scripts de indexación.
//
// Uso:
//   echo "texto a embeber" | cargo run --bin nexus-embed
//   cargo run --bin nexus-embed -- "texto directo"
//   cargo run --bin nexus-embed -- --dim < input.txt   # solo imprime dimensión
// ============================================================================

use std::env;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().collect();

    // Leer texto de entrada (stdin o primer argumento)
    let texto = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        let mut buf = String::new();
        if io::stdin().read_to_string(&mut buf).is_err() {
            eprintln!("❌ Error leyendo stdin");
            std::process::exit(1);
        }
        buf
    };

    let texto = texto.trim().to_string();
    if texto.is_empty() {
        eprintln!("❌ No se proporcionó texto para embeber");
        std::process::exit(1);
    }

    // Generar embedding: SHA-256 angular puro (sin grafo de conceptos)
    let embedding = nexus_ultimate_core::nexus_embedder::NexusEmbedder::generar(&texto, &[]);

    // Escribir a stdout: 768 floats separados por espacio
    let mut first = true;
    for val in &embedding {
        if !first {
            print!(" ");
        }
        // Formato científico con 8 decimales para precisión sin perder ancho
        print!("{:.8e}", val);
        first = false;
    }
    println!();

    // Reportar métricas a stderr (no interfiere con pipeline)
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    eprintln!(
        "✓ Embedding 768-dim generado | norma={:.6} | chars={} | primeros_5=[{:.4}, {:.4}, {:.4}, {:.4}, {:.4}]",
        norm,
        texto.len(),
        embedding[0], embedding[1], embedding[2], embedding[3], embedding[4]
    );
}
