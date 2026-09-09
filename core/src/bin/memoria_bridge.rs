// ==========================================
// NEXUS MEMORIA BRIDGE — Puente de Consulta FTS5
// ==========================================
// Consulta directa a nexus_memoria.db con FTS5 (sin LanceDB, sin JSON legacy).
//
// USO:
//   cargo run --bin memoria_bridge snapshot
//   cargo run --bin memoria_bridge query "sembrador de identidades"
//   cargo run --bin memoria_bridge status
//   cargo run --bin memoria_bridge index
// ==========================================

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use nexus_ultimate_core::infra::paths::resolve_path;
use nexus_ultimate_core::memoria::memoria_semantica::MemoriaSemantica;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

// ── Constantes de ruta ──────────────────────────────────────────────────
const NEXUS_MEMORIA_DB: &str = "data/nexus_memoria.db";
const LOGROS_MD: &str = "memoria/logros.md";
const FRACASOS_MD: &str = "memoria/fracasos.md";

/// Trunca texto de forma segura por caracteres para evitar pánicos con emojis multibyte
fn truncar(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max_chars).collect::<String>())
    }
}

// ── CLI ─────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "memoria_bridge",
    about = "🧠 Puente de Memoria Multidimensional de NEXUS",
    version = "1.0.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Búsqueda semántica en toda la memoria disponible
    Query {
        /// Texto a buscar
        texto: String,
        /// Límite de resultados (default: 5)
        #[arg(short, long, default_value = "5")]
        limite: usize,
    },
    /// Snapshot completo del estado de memoria para inyectar en contexto
    Snapshot {
        /// Entradas recientes a incluir (default: 15)
        #[arg(short, long, default_value = "15")]
        recientes: usize,
        /// Texto de búsqueda semántica opcional para enriquecer el snapshot
        #[arg(short, long)]
        buscar: Option<String>,
    },
    /// Indexa logros.md y fracasos.md en FTS5 para búsqueda semántica
    Index,
    /// Verifica el estado de todos los órganos de memoria
    Status,
}

// ── SQLite helpers (nexus_memoria.db) ────────────────────────────────────

/// Abre conexión a nexus_memoria.db (reemplaza a la antigua abrir_pulso)
fn abrir_nexus_memoria() -> Result<Connection> {
    let path = resolve_path(NEXUS_MEMORIA_DB);
    if !path.exists() {
        anyhow::bail!("Base nexus_memoria.db no encontrada en {}", path.display());
    }
    let conn =
        Connection::open(&path).with_context(|| format!("No se pudo abrir {}", path.display()))?;
    Ok(conn)
}

/// Consulta historial reciente desde memoria_episodica (FTS5 content table)
fn consultar_historial_reciente(
    conn: &Connection,
    limite: usize,
) -> Result<Vec<(String, String, String)>> {
    let mut stmt = conn
        .prepare(
            "SELECT titulo, contenido, timestamp FROM memoria_episodica ORDER BY id DESC LIMIT ?1",
        )
        .context("Preparando consulta memoria_episodica")?;
    let rows = stmt
        .query_map([limite], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .context("Ejecutando consulta memoria_episodica")?;

    let mut resultados = Vec::new();
    for row in rows {
        resultados.push(row.context("Leyendo fila memoria_episodica")?);
    }
    Ok(resultados)
}

/// Consulta experiencias consolidadas desde memoria_semantica (FTS5 content table)
fn consultar_memoria_semantica(
    conn: &Connection,
    limite: usize,
) -> Result<Vec<(String, String, f64, f64)>> {
    let mut stmt = conn
        .prepare(
            "SELECT titulo, contenido, peso_permanencia, CAST(prioridad AS REAL) FROM memoria_semantica ORDER BY id DESC LIMIT ?1",
        )
        .context("Preparando consulta memoria_semantica")?;
    let rows = stmt
        .query_map([limite], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
            ))
        })
        .context("Ejecutando consulta memoria_semantica")?;

    let mut resultados = Vec::new();
    for row in rows {
        resultados.push(row.context("Leyendo fila memoria_semantica")?);
    }
    Ok(resultados)
}

/// Consulta contexto activo desde tabla contexto_activo
fn consultar_contexto(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt = conn
        .prepare(
            "SELECT clave, valor FROM contexto_activo ORDER BY ultima_actualizacion DESC LIMIT 30",
        )
        .context("Preparando consulta contexto_activo")?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .context("Ejecutando consulta contexto_activo")?;

    let mut resultados = Vec::new();
    for row in rows {
        resultados.push(row.context("Leyendo fila contexto_activo")?);
    }
    Ok(resultados)
}

/// Consulta flujo soberano
fn consultar_flujo_soberano(
    conn: &Connection,
    limite: usize,
) -> Result<Vec<(String, String, f64)>> {
    let mut stmt = conn
        .prepare(
            "SELECT entidad, mensaje, importancia FROM flujo_soberano ORDER BY id DESC LIMIT ?1",
        )
        .context("Preparando consulta flujo_soberano")?;
    let rows = stmt
        .query_map([limite], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
            ))
        })
        .context("Ejecutando consulta flujo_soberano")?;

    let mut resultados = Vec::new();
    for row in rows {
        resultados.push(row.context("Leyendo fila flujo_soberano")?);
    }
    Ok(resultados)
}

// ── Comandos ────────────────────────────────────────────────────────────

async fn cmd_query(texto: &str, limite: usize) -> Result<()> {
    let start = Instant::now();

    // 1. Buscar directamente en FTS5 (sin embedding vectorial)
    let mem = MemoriaSemantica::new(NEXUS_MEMORIA_DB).await?;

    let resultados_episodica = mem.buscar_fts5(texto, "memoria_episodica", limite)?;
    let resultados_semantica = mem.buscar_fts5(texto, "memoria_semantica", limite)?;

    // 2. Resultados
    println!("🔍 RESULTADOS DE BÚSQUEDA FTS5: \"{}\"", texto);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    if resultados_episodica.is_empty() && resultados_semantica.is_empty() {
        println!("📭 No se encontraron resultados en FTS5.");
        println!("   → Usa 'memoria_bridge index' para indexar logros.md");
    }

    if !resultados_episodica.is_empty() {
        println!("🌊 MEMORIA_EPISÓDICA (FTS5):");
        for (id, esencia, score) in &resultados_episodica {
            println!(
                "  [{id}] relevancia={:.1}% | {:.120}",
                score * 100.0,
                esencia
            );
        }
        println!();
    }

    if !resultados_semantica.is_empty() {
        println!("🧠 MEMORIA_SEMÁNTICA (FTS5):");
        for (id, esencia, score) in &resultados_semantica {
            println!(
                "  [{id}] relevancia={:.1}% | {:.120}",
                score * 100.0,
                esencia
            );
        }
        println!();
    }

    eprintln!("⏱️  Tiempo total: {:?}", start.elapsed());
    Ok(())
}

async fn cmd_snapshot(recientes: usize, buscar: Option<String>) -> Result<()> {
    let start = Instant::now();
    let conn = abrir_nexus_memoria()?;

    // Encabezado
    println!("🧠 SNAPSHOT DE MEMORIA — NEXUS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "Generado: {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    // ── 1. Contexto activo ──
    let contexto = consultar_contexto(&conn)?;
    if !contexto.is_empty() {
        println!("## 🏛️ CONTEXTO ACTIVO");
        for (clave, valor) in &contexto {
            println!("- **{clave}**: {valor}");
        }
        println!();
    }

    // ── 2. Memoria episódica reciente ──
    let historial = consultar_historial_reciente(&conn, recientes)?;
    if !historial.is_empty() {
        println!(
            "## 🌊 MEMORIA EPISÓDICA RECIENTE (últimas {})",
            historial.len()
        );
        for (i, (titulo, contenido, _created_at)) in historial.iter().enumerate().rev() {
            let titulo_short = truncar(titulo, 80);
            let cont_short = truncar(contenido, 120);
            println!("{}. **{titulo_short}**", i + 1);
            println!("   → {cont_short}");
        }
        println!();
    }

    // ── 3. Memoria Semántica (experiencias consolidadas) ──
    let memoria_semantica = consultar_memoria_semantica(&conn, recientes)?;
    if !memoria_semantica.is_empty() {
        println!("## ⭐ EXPERIENCIAS (memoria_semantica)");
        for (titulo, contenido, importancia, tono) in &memoria_semantica {
            let titulo_short = truncar(titulo, 100);
            println!("- **{titulo_short}** (I={importancia:.2}, T={tono:.2})");
            if !contenido.is_empty() && contenido.len() < 200 {
                println!("  ↳ {contenido}");
            }
        }
        println!();
    } else {
        println!("## ⭐ EXPERIENCIAS");
        println!("_No hay experiencias consolidadas en memoria_semantica._");
        println!("  → Usa 'memoria_bridge index' para indexar logros.md\n");
    }

    // ── 4. Flujo Soberano ──
    let flujo = consultar_flujo_soberano(&conn, 10)?;
    if !flujo.is_empty() {
        println!("## 👑 FLUJO SOBERANO");
        for (entidad, mensaje, importancia) in &flujo {
            if *importancia > 0.5 {
                let msg_short = truncar(mensaje, 100);
                println!("- **[{entidad}]** (I={importancia:.2}) {msg_short}");
            }
        }
        println!();
    }

    // ── 5. Búsqueda FTS5 opcional ──
    if let Some(texto_buscar) = buscar {
        if !texto_buscar.is_empty() {
            println!("## 🔍 BÚSQUEDA FTS5: \"{texto_buscar}\"");
            let mem = MemoriaSemantica::new(NEXUS_MEMORIA_DB).await?;

            let resultados = mem.buscar_fts5(&texto_buscar, "memoria_episodica", 5)?;

            if resultados.is_empty() {
                println!("_Sin resultados FTS5._");
            } else {
                for (id, texto, score) in &resultados {
                    let relevancia = score * 100.0;
                    println!("- [{id}] ({relevancia:.0}%) {texto}");
                }
            }
            println!();
        }
    }

    // ── 6. Estado de nexus_memoria.db ──
    println!("## 📊 ESTADO DE NEXUS_MEMORIA.DB (FTS5)");
    let mem = MemoriaSemantica::new(NEXUS_MEMORIA_DB).await?;
    match mem.contar_en_tabla("memoria_episodica").await {
        Ok(n) => println!("- 🌊 memoria_episodica: {n} registros (FTS5)"),
        Err(e) => println!("- 🌊 memoria_episodica: ERROR — {e}"),
    }
    match mem.contar_en_tabla("memoria_semantica").await {
        Ok(n) => println!("- 🧠 memoria_semantica: {n} registros (FTS5)"),
        Err(e) => println!("- 🧠 memoria_semantica: ERROR — {e}"),
    }
    println!();

    eprintln!("⏱️  Snapshot generado en {:?}", start.elapsed());
    Ok(())
}

/// Indexa las secciones (delimitadas por "\n## ") de un archivo markdown
/// en la tabla memoria_semantica con el tipo indicado.
fn indexar_markdown(
    conn: &Connection,
    ruta: &str,
    tipo: &str,
    peso: f64,
    prioridad: u8,
) -> Result<usize> {
    let path = PathBuf::from(ruta);
    if !path.exists() {
        anyhow::bail!("{ruta} no encontrado");
    }
    let content = fs::read_to_string(&path).with_context(|| format!("Leyendo {ruta}"))?;

    // Calcular hash SHA256 del archivo para asegurar integridad (Evolución Cognitiva)
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    let fuente_info = format!("{} | sha256:{}", ruta, hash);

    let secciones: Vec<&str> = content.split("\n## ").collect();
    eprintln!("📄 {} secciones encontradas en {ruta}", secciones.len());

    let mut indexados = 0usize;
    for (i, seccion) in secciones.iter().enumerate() {
        let titulo = seccion.lines().next().unwrap_or("sin título").trim();
        let texto_completo = format!("## {}", seccion.trim());

        conn.execute(
            "INSERT INTO memoria_semantica (tipo, titulo, contenido, peso_permanencia, prioridad, archivos_fuente)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![tipo, titulo, texto_completo, peso, prioridad, fuente_info],
        )?;

        indexados += 1;
        if indexados <= 3 || i % 10 == 0 {
            eprintln!("  ✓ [{i}] {titulo}");
        }
    }
    Ok(indexados)
}

async fn cmd_index() -> Result<()> {
    let start = Instant::now();
    println!("📦 Indexando logros.md y fracasos.md en nexus_memoria.db (FTS5)...\n");

    let conn = Connection::open(resolve_path(NEXUS_MEMORIA_DB))?;

    // 1. Indexar logros.md (tipo 'Logro', peso 0.7, prioridad 1)
    eprintln!("\n📗 [LOGROS]");
    let logros = indexar_markdown(&conn, LOGROS_MD, "Logro", 0.7, 1)?;

    // 2. Indexar fracasos.md (tipo 'Fracaso', peso 0.8 — mayor por su valor de aprendizaje)
    eprintln!("\n📕 [FRACASOS]");
    let fracasos = indexar_markdown(&conn, FRACASOS_MD, "Fracaso", 0.8, 2)?;

    let total = logros + fracasos;
    println!(
        "\n✅ Indexación completada: {total} secciones ({logros} logros, {fracasos} fracasos) → memoria_semantica (FTS5)"
    );
    eprintln!("⏱️  Tiempo total: {:?}", start.elapsed());
    Ok(())
}

async fn cmd_status() -> Result<()> {
    println!("🧠 DIAGNÓSTICO DE ÓRGANOS DE MEMORIA (FTS5)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // ── 1. Tablas en nexus_memoria.db ──
    println!("1. 🧠 NEXUS_MEMORIA.DB (FTS5)");
    let conn = Connection::open(resolve_path(NEXUS_MEMORIA_DB))?;
    let tablas = [
        ("memoria_episodica", "🌊 Memoria Episódica"),
        ("memoria_semantica", "🧠 Memoria Semántica"),
        ("memoria_procedural", "⚡ Memoria Procedural"),
        ("memoria_emocional", "💭 Memoria Emocional"),
        ("sesiones", "💬 Sesiones"),
        ("contexto_activo", "🏛️ Contexto Activo"),
        ("identidades_sembradas", "🎭 Identidades Sembradas"),
        ("errores_soluciones", "🛠️ Errores y Soluciones"),
        ("flujo_soberano", "👑 Flujo Soberano"),
        ("grafo_semantico_nodos", "🔗 Nodos Grafo"),
        ("grafo_semantico_sinapsis", "⚡ Sinapsis Grafo"),
        ("sinapsis_legado", "📜 Sinapsis Legado"),
    ];
    for (tabla, nombre) in &tablas {
        let count: Result<i64, _> = conn.query_row(
            &format!("SELECT COUNT(*) FROM {tabla} WHERE id > 0"),
            [],
            |row| row.get(0),
        );
        match count {
            Ok(n) => println!("   {nombre}: {n} registros"),
            Err(e) => println!("   {nombre}: ⚠️  {e}"),
        }
    }
    println!();

    // ── 2. Tablas FTS5 virtuales ──
    println!("2. 🔍 TABLAS FTS5 (índices de búsqueda)");
    let fts_tablas = [
        ("memoria_episodica_fts", "🌊 FTS5 Episódica"),
        ("memoria_semantica_fts", "🧠 FTS5 Semántica"),
        ("memoria_procedural_fts", "⚡ FTS5 Procedural"),
        ("memoria_emocional_fts", "💭 FTS5 Emocional"),
    ];
    for (tabla, nombre) in &fts_tablas {
        let count: Result<i64, _> =
            conn.query_row(&format!("SELECT COUNT(*) FROM {tabla}"), [], |row| {
                row.get(0)
            });
        match count {
            Ok(n) => println!("   {nombre}: {n} registros indexados"),
            Err(e) => println!("   {nombre}: ⚠️  {e}"),
        }
    }
    println!();

    // ── 3. logros.md ──
    println!("3. 📗 LOGROS.MD ({LOGROS_MD})");
    let logros_path = PathBuf::from(LOGROS_MD);
    if logros_path.exists() {
        match fs::read_to_string(&logros_path) {
            Ok(content) => {
                let lineas = content.lines().count();
                let secciones: Vec<&str> = content.split("\n## ").collect();
                println!(
                    "   Estado: ✅ PRESENTE ({lineas} líneas, {} secciones)",
                    secciones.len()
                );
            }
            Err(e) => println!("   Estado: ⚠️  Error de lectura — {e}"),
        }
    } else {
        println!("   Estado: ❌ NO ENCONTRADO");
    }
    println!();

    // ── 4. agente_memoria.md ──
    println!("4. 📓 AGENTE_MEMORIA.MD");
    let agente_path = PathBuf::from("memoria/agente_memoria.md");
    if agente_path.exists() {
        match fs::read_to_string(&agente_path) {
            Ok(content) => {
                println!(
                    "   Estado: ✅ PRESENTE ({} líneas)",
                    content.lines().count()
                );
            }
            Err(e) => println!("   Estado: ⚠️  Error de lectura — {e}"),
        }
    } else {
        println!("   Estado: ❌ NO ENCONTRADO");
    }
    println!();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "🔧 Resumen: El sistema de memoria tiene TODOS los órganos unificados en nexus_memoria.db."
    );
    println!("   Gap principal: agente_memoria.md requiere actualización manual.");
    println!("   → Usa 'memoria_bridge snapshot > memoria/agente_memoria.md' para actualizar.");
    Ok(())
}

// ── Entry point ─────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Query { texto, limite } => cmd_query(&texto, limite).await,
        Commands::Snapshot { recientes, buscar } => cmd_snapshot(recientes, buscar).await,
        Commands::Index => cmd_index().await,
        Commands::Status => cmd_status().await,
    }
}
