// ==========================================
// HERRAMIENTA DE DIAGNÓSTICO OCEAN - NEXUS
// ==========================================
// Verifica y sincroniza las tablas ocean/mareas
// en nexus_intelligence.db
// ==========================================

use rusqlite::{params, Connection};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from("/home/soberano/NEXUS_ULTIMATE_CORE");

    // --- 1. DIAGNÓSTICO: nexus_intelligence.db ---
    let nexus_db = root.join("nexus_intelligence.db");
    println!("🔍 Analizando: {:?}", nexus_db);
    let conn_nexus = Connection::open(&nexus_db)?;

    let tablas_nexus: Vec<String> = conn_nexus
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    println!("   Tablas en nexus_intelligence.db: {} tablas", tablas_nexus.len());
    let tiene_ocean = tablas_nexus.contains(&"ocean".to_string());
    let tiene_mareas = tablas_nexus.contains(&"mareas".to_string());
    println!("   ocean: {}  |  mareas: {}", tiene_ocean, tiene_mareas);

    // --- 2. DIAGNÓSTICO: data/intelligence.db ---
    let data_db = root.join("data/intelligence.db");
    println!("\n🔍 Analizando: {:?}", data_db);
    let conn_data = Connection::open(&data_db)?;

    let tablas_data: Vec<String> = conn_data
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    println!("   Tablas en data/intelligence.db: {} tablas", tablas_data.len());
    let data_tiene_ocean = tablas_data.contains(&"ocean".to_string());
    let data_tiene_mareas = tablas_data.contains(&"mareas".to_string());
    println!("   ocean: {}  |  mareas: {}", data_tiene_ocean, data_tiene_mareas);

    // --- 3. Sincronizar si falta ---
    if !tiene_ocean || !tiene_mareas {
        println!("\n🔄 Sincronizando tablas faltantes en nexus_intelligence.db...");

        if !tiene_ocean {
            conn_nexus.execute(
                "CREATE TABLE IF NOT EXISTS ocean (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    esencia TEXT NOT NULL,
                    tono_emocional REAL NOT NULL DEFAULT 0.0,
                    tema TEXT,
                    reflejo_arquitecto TEXT,
                    intensidad REAL DEFAULT 0.5,
                    timestamp TEXT DEFAULT (datetime('now'))
                )",
                [],
            )?;
            println!("   ✅ Tabla 'ocean' creada en nexus_intelligence.db");
        }

        if !tiene_mareas {
            conn_nexus.execute(
                "CREATE TABLE IF NOT EXISTS mareas (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    tema TEXT NOT NULL,
                    tono_promedio REAL DEFAULT 0.0,
                    frecuencia INTEGER DEFAULT 0,
                    ultima_marea TEXT DEFAULT (datetime('now'))
                )",
                [],
            )?;
            println!("   ✅ Tabla 'mareas' creada en nexus_intelligence.db");
        }
    } else {
        println!("\n✅ nexus_intelligence.db ya tiene ocean y mareas");
    }

    // --- 4. CONTEO DE REGISTROS ---
    if data_tiene_ocean {
        let count: i64 = conn_data.query_row("SELECT COUNT(*) FROM ocean", [], |r| r.get(0))?;
        println!("\n📊 data/intelligence.db - ocean: {} registros", count);
    }
    if data_tiene_mareas {
        let count: i64 = conn_data.query_row("SELECT COUNT(*) FROM mareas", [], |r| r.get(0))?;
        println!("📊 data/intelligence.db - mareas: {} registros", count);
    }

    if tiene_ocean {
        let count: i64 = conn_nexus.query_row("SELECT COUNT(*) FROM ocean", [], |r| r.get(0))?;
        println!("📊 nexus_intelligence.db - ocean: {} registros", count);
    }
    if tiene_mareas {
        let count: i64 = conn_nexus.query_row("SELECT COUNT(*) FROM mareas", [], |r| r.get(0))?;
        println!("📊 nexus_intelligence.db - mareas: {} registros", count);
    }

    // --- 5. VERIFICAR ESQUEMA ---
    println!("\n📋 Schema ocean (nexus):");
    let schema: String = conn_nexus
        .prepare("SELECT sql FROM sqlite_master WHERE name='ocean'")?
        .query_row([], |r| r.get(0))?;
    println!("   {}", schema);

    println!("\n📋 Schema mareas (nexus):");
    let schema: String = conn_nexus
        .prepare("SELECT sql FROM sqlite_master WHERE name='mareas'")?
        .query_row([], |r| r.get(0))?;
    println!("   {}", schema);

    println!("\n✅ Diagnóstico y sincronización completados.");
    Ok(())
}
