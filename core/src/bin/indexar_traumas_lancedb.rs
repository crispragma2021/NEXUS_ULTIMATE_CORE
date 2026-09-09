// ============================================================================
// 🔥 INDEXAR TRAUMAS REALES EN LANCEDB — Migración única
// ============================================================================
// Propósito: Leer los 3 traumas reales de intelligence.db (tabla ocean)
//   y migrarlos al índice semántico de LanceDB (tabla ocean_vectors).
//   Esto permite que Ocean::recordar_por_emocion() los encuentre
//   también mediante búsqueda vectorial, no solo por tono SQL.
//
// Contexto: Los traumas existentes NUNCA pasaron por Ocean::sumergir()
//   (que genera embedding e indexa en LanceDB). Fueron insertados
//   directamente en SQLite por sesiones pasadas.
//
// Diagnóstico:
//   - DB: data/intelligence.db (313MB)
//   - Tabla: ocean (102 registros, 3 con tono < -0.3)
//   - LanceDB: data/lancedb (tabla ocean_vectors)
// ============================================================================

use nexus_ultimate_core::emociones::ocean::Impresion;
use nexus_ultimate_core::memoria::memoria_semantica::MemoriaSemantica;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

/// Extrae TODAS las impresiones de la tabla ocean en intelligence.db
fn extraer_todas_impresiones(db_path: &PathBuf) -> Vec<Impresion> {
    use rusqlite::Connection;

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Error al abrir intelligence.db: {}", e);
            return vec![];
        }
    };

    // Obtener estadísticas
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM ocean", [], |row| row.get(0))
        .unwrap_or(0);
    println!(
        "   📊 Registros totales en Ocean: \x1b[1;36m{}\x1b[0m",
        total
    );

    let traumas_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM ocean WHERE tono_emocional < -0.3",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    println!(
        "   🩸 Traumas (tono < -0.3): \x1b[1;31m{}\x1b[0m",
        traumas_count
    );

    let neutras_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM ocean WHERE tono_emocional >= -0.3 AND tono_emocional <= 0.3",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    println!(
        "   😐 Neutras (-0.3 ≤ tono ≤ 0.3): \x1b[1;33m{}\x1b[0m",
        neutras_count
    );

    let positivas_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM ocean WHERE tono_emocional > 0.3",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    println!(
        "   😊 Positivas (tono > 0.3): \x1b[1;32m{}\x1b[0m",
        positivas_count
    );

    // Extraer TODAS las impresiones (no solo traumas) para tener índice completo
    let mut stmt = conn
        .prepare(
            "SELECT id, esencia, tono_emocional, tema, reflejo_arquitecto, timestamp \
             FROM ocean ORDER BY id ASC",
        )
        .expect("Error preparando query de ocean");

    let impresiones: Vec<Impresion> = stmt
        .query_map([], |row| {
            Ok(Impresion {
                id: row.get(0)?,
                esencia: row.get(1)?,
                tono_emocional: row.get(2)?,
                tema: row.get(3)?,
                reflejo_arquitecto: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })
        .expect("Error mappeando impresiones")
        .filter_map(|r| r.ok())
        .collect();

    println!(
        "   📦 Total extraídas: \x1b[1;36m{}\x1b[0m",
        impresiones.len()
    );
    impresiones
}

fn resumir_esencia(esencia: &str, max: usize) -> String {
    if esencia.len() <= max {
        esencia.to_string()
    } else {
        format!("{}...", &esencia[..max])
    }
}

#[tokio::main]
async fn main() {
    println!(
        "\n\x1b[1;35m══════════════════════════════════════════════════════════════════\x1b[0m"
    );
    println!("\x1b[1;35m  🔥 INDEXAR TRAUMAS EN LANCEDB — Migración Semántica\x1b[0m");
    println!(
        "\x1b[1;35m══════════════════════════════════════════════════════════════════\x1b[0m\n"
    );

    // ─── 1. Resolver rutas ───────────────────────────────────────────────────
    let db_path = nexus_ultimate_core::infra::paths::resolve_path("data/intelligence.db");
    let lancedb_uri = nexus_ultimate_core::infra::paths::resolve_path("data/lancedb");

    println!("📁 Intelligence DB: \x1b[1;36m{}\x1b[0m", db_path.display());
    println!(
        "📁 LanceDB URI:     \x1b[1;36m{}\x1b[0m",
        lancedb_uri.display()
    );

    // Verificar que intelligence.db existe
    if !db_path.exists() {
        eprintln!(
            "\n❌ ERROR FATAL: intelligence.db NO existe en {}",
            db_path.display()
        );
        std::process::exit(1);
    }

    // ─── 2. Extraer impresiones desde SQLite ────────────────────────────────
    println!(
        "\n\x1b[1;34m─── 1. Extrayendo impresiones de Ocean ───────────────────────────\x1b[0m"
    );
    let impresiones = extraer_todas_impresiones(&db_path);

    if impresiones.is_empty() {
        eprintln!("\n❌ No se encontraron impresiones en Ocean. Abortando.");
        std::process::exit(1);
    }

    // ─── 3. Conectar a LanceDB ──────────────────────────────────────────────
    println!(
        "\n\x1b[1;34m─── 2. Conectando a LanceDB ───────────────────────────────────────\x1b[0m"
    );
    let lancedb_str = lancedb_uri
        .to_str()
        .expect("Ruta LanceDB contiene caracteres no UTF-8");

    let semantica = Arc::new(match MemoriaSemantica::new(lancedb_str).await {
        Ok(s) => {
            println!("   ✅ MemoriaSemantica conectada a LanceDB");
            s
        }
        Err(e) => {
            eprintln!("❌ ERROR FATAL: no se pudo conectar a LanceDB: {}", e);
            std::process::exit(1);
        }
    });

    // Verificar estado actual del índice
    match semantica.contar_en_tabla("ocean_vectors").await {
        Ok(count) => println!(
            "   📊 Registros actuales en ocean_vectors: \x1b[1;36m{}\x1b[0m",
            count
        ),
        Err(_) => println!("   📊 Tabla ocean_vectors no existe aún — se creará"),
    }

    // ─── 4. Indexar cada impresión en LanceDB ───────────────────────────────
    println!(
        "\n\x1b[1;34m─── 3. Indexando impresiones en LanceDB ───────────────────────────\x1b[0m"
    );

    let mut indexados = 0u32;
    let mut errores = 0u32;
    let mut traumas_indexados = 0u32;
    let start = Instant::now();

    for (i, imp) in impresiones.iter().enumerate() {
        let es_trauma = imp.tono_emocional < -0.3;
        let label = if es_trauma { "🩸 TRAUMA" } else { "   📝" };
        let esencia_resumida = resumir_esencia(&imp.esencia, 60);

        print!(
            "\r   [{}/{}] {} id={} tono={:.1} «{}»",
            i + 1,
            impresiones.len(),
            label,
            imp.id,
            imp.tono_emocional,
            esencia_resumida
        );

        // Generar embedding
        let vector = match semantica.generar_embedding(&imp.esencia).await {
            Ok(v) => v,
            Err(e) => {
                println!(
                    "\n   ❌ Error generando embedding para id={}: {}",
                    imp.id, e
                );
                errores += 1;
                continue;
            }
        };

        // Indexar en LanceDB
        match semantica
            .indexar_impresion(imp.id, &imp.esencia, vector)
            .await
        {
            Ok(_) => {
                indexados += 1;
                if es_trauma {
                    traumas_indexados += 1;
                }
            }
            Err(e) => {
                println!("\n   ❌ Error indexando id={} en LanceDB: {}", imp.id, e);
                errores += 1;
            }
        }
    }

    println!(); // salto de línea después del último \r

    let elapsed = start.elapsed();

    // ─── 5. Reporte final ───────────────────────────────────────────────────
    println!(
        "\n\x1b[1;35m══════════════════════════════════════════════════════════════════\x1b[0m"
    );
    println!("\x1b[1;35m  📊 REPORTE DE MIGRACIÓN SEMÁNTICA\x1b[0m");
    println!("\x1b[1;35m══════════════════════════════════════════════════════════════════\x1b[0m");
    println!();
    println!(
        "   ✅ Indexados en LanceDB:  \x1b[1;32m{}\x1b[0m",
        indexados
    );
    println!(
        "   🩸 Traumas indexados:     \x1b[1;31m{}\x1b[0m",
        traumas_indexados
    );
    println!("   ❌ Errores:               \x1b[1;31m{}\x1b[0m", errores);
    println!(
        "   ⏱️  Tiempo total:          \x1b[1;36m{:.2}s\x1b[0m",
        elapsed.as_secs_f64()
    );

    // Verificar resultado final
    match semantica.contar_en_tabla("ocean_vectors").await {
        Ok(count) => {
            println!();
            println!(
                "   📊 Registros finales en ocean_vectors: \x1b[1;36m{}\x1b[0m",
                count
            );
            if count == impresiones.len() {
                println!(
                    "   🎯 \x1b[1;32mMIGRACIÓN COMPLETA: 100% de impresiones indexadas.\x1b[0m"
                );
            } else if count > 0 {
                println!(
                    "   ⚠️  Indexados {}/{} — puede haber duplicados si ya existían.",
                    count,
                    impresiones.len()
                );
            } else {
                println!("   ❌ No se indexó nada. Revisar errores.");
            }
        }
        Err(e) => {
            println!("   ❌ Error verificando resultado: {}", e);
        }
    }

    println!();
    println!("\x1b[1;35m──────────────────────────────────────────────────────────────────\x1b[0m");
    println!("\x1b[1;35m  🧬 Ahora el GOI encontrará traumas vía búsqueda semántica.\x1b[0m");
    println!("\x1b[1;35m  🚀 Ejecutar: cargo run -j 14 --bin indexar_traumas_lancedb\x1b[0m");
    println!(
        "\x1b[1;35m──────────────────────────────────────────────────────────────────\x1b[0m\n"
    );
}
