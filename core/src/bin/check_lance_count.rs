use rusqlite::Connection;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 Investigando el estado de nexus_memoria.db (FTS5)...\n");

    let conn = Connection::open("data/nexus_memoria.db")?;
    let tablas = [
        ("memoria_episodica", "🌊 Memoria Episódica"),
        ("memoria_semantica", "🧠 Memoria Semántica"),
        ("memoria_emocional", "💭 Memoria Emocional"),
        ("memoria_procedural", "⚡ Memoria Procedural"),
        ("sesiones", "💬 Sesiones"),
        ("contexto_activo", "🏛️ Contexto Activo"),
        ("grafo_semantico_sinapsis", "🔗 Sinapsis"),
    ];

    for (tabla, nombre) in &tablas {
        let count: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {}", tabla), [], |row| {
            row.get(0)
        })?;
        println!("📊 '{}': {} registros encontrados.", nombre, count);
    }

    // Verificar FTS5
    let fts_tablas = ["memoria_episodica_fts", "memoria_semantica_fts"];
    for fts in &fts_tablas {
        let count: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {}", fts), [], |row| {
            row.get(0)
        })?;
        println!("🔍 Índice FTS5 '{}': {} entradas.", fts, count);
    }

    println!("\n✅ Diagnóstico completado. Todas las tablas FTS5 operativas.");
    Ok(())
}
