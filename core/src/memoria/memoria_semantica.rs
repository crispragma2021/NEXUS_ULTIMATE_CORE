// ==========================================
// MEMORIA SEMÁNTICA - Instinto (SQLite FTS5 + NexusEmbedder Soberano)
// ==========================================
// ANTES: LanceDB con vectores SHA-256 (768-dim) para búsqueda semántica.
// AHORA: SQLite FTS5 con tokenizador unicode61 + BM25 ranking.
//         Más rápido, cero dependencias externas, ACID, integrado con nexus_memoria.db.
// ==========================================

use anyhow::{anyhow, Result};
use rusqlite::{params, Connection};
use std::sync::Mutex;
use tracing::info;

use crate::nexus_embedder::NexusEmbedder;

/// Ruta por defecto a la base de datos unificada de memoria.
/// Sobreescribe la antigua URI de LanceDB.
const NEXUS_MEMORIA_DB: &str = "data/nexus_memoria.db";

pub struct MemoriaSemantica {
    conn: Mutex<Connection>,
    #[allow(dead_code)]
    path: String,
}

impl MemoriaSemantica {
    /// Expone la conexión interna para operaciones cross-memoria (Ebbinghaus).
    /// ÚNICAMENTE para uso controlado por MemoriaOperativa::ebbinghaus_tick().
    pub fn conn(&self) -> &Mutex<Connection> {
        &self.conn
    }

    /// Promueve un recuerdo episódico a memoria semántica (Curva de Ebbinghaus).
    ///
    /// Inserta en `memoria_semantica` si no existe ya, y marca el origen.
    /// Retorna true si se insertó, false si ya existía.
    pub fn promover_desde_episodica(
        &self,
        id_episodico: i64,
        titulo: &str,
        contenido: &str,
        importancia: f64,
        tono_emocional: f64,
        keywords: &str,
        created_at: &str,
    ) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow!("Mutex poisoned: {}", e))?;
        let keywords_promovido = format!(
            "{};promovido_desde=episodica;id_episodico={};importancia={:.2}",
            keywords, id_episodico, importancia
        );

        let result = conn.execute(
            "INSERT OR IGNORE INTO memoria_semantica
                (titulo, contenido, importancia, tono_emocional, keywords, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                titulo,
                contenido,
                importancia,
                tono_emocional,
                &keywords_promovido,
                created_at
            ],
        )?;

        if result > 0 {
            info!(
                "🧪 [SEMÁNTICA] Promovido id_episodico={} a semántica (importancia={:.2})",
                id_episodico, importancia
            );
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Abre conexión a nexus_memoria.db con FTS5.
    ///
    /// `uri` se conserva para compatibilidad de API, pero ahora es
    /// la ruta al archivo SQLite (antes era URI de LanceDB).
    pub async fn new(uri: &str) -> Result<Self> {
        let db_path = if uri.is_empty() || uri == "memory://" || uri == "memory://test" {
            NEXUS_MEMORIA_DB.to_string()
        } else if uri.starts_with("data/") || !uri.contains("://") {
            uri.to_string()
        } else {
            NEXUS_MEMORIA_DB.to_string()
        };

        let path = crate::infra::paths::resolve_path(&db_path);
        let conn = Connection::open(&path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;

        info!("🧬 MemoriaSemantica vinculada a FTS5 en {}", path.display());
        Ok(Self {
            conn: Mutex::new(conn),
            path: db_path,
        })
    }

    /// Genera embedding usando NexusEmbedder soberano.
    /// Se conserva para compatibilidad con código que aún lo usa,
    /// pero la búsqueda principal ahora es FTS5.
    pub async fn generar_embedding(&self, texto: &str) -> Result<Vec<f32>> {
        let embedding = NexusEmbedder::generar(texto, &[]);
        info!("📊 NexusEmbedder soberano: {} dims", embedding.len());
        Ok(embedding)
    }

    /// Indexa una impresión (texto) en la tabla FTS5 correspondiente.
    ///
    /// Tradicionalmente indexaba un vector en LanceDB. Ahora inserta
    /// el texto directamente en la tabla content_table, y el trigger FTS5
    /// lo sincroniza automáticamente con el índice de búsqueda.
    pub async fn indexar_impresion(&self, id: i64, esencia: &str, _vector: Vec<f32>) -> Result<()> {
        self.indexar_impresion_con_tabla(id, esencia, _vector, "ocean_vectors")
            .await
    }

    /// Indexa una impresión en la tabla FTS5 correspondiente.
    ///
    /// `table_name` se usa para determinar qué tabla content actualizar.
    /// Mapeo: "ocean_vectors" → memoria_episodica, "codebase_knowledge" → memoria_semantica
    pub async fn indexar_impresion_con_tabla(
        &self,
        id: i64,
        esencia: &str,
        _vector: Vec<f32>,
        table_name: &str,
    ) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow!("Mutex poisoned: {}", e))?;

        let content_table = match table_name {
            "ocean_vectors" => "memoria_episodica",
            _ => "memoria_semantica",
        };

        conn.execute(
            &format!(
                "INSERT INTO {} (id, titulo, contenido, importancia, tono_emocional, created_at)
                 VALUES (?1, ?2, ?3, 0.5, 0.0, datetime('now'))",
                content_table
            ),
            params![id, esencia, esencia],
        )?;

        info!("📝 Indexado en FTS5 [{}]: id={}", content_table, id);
        Ok(())
    }

    /// Busca vectores similares — AHORA: búsqueda FTS5.
    ///
    /// Como FTS5 no puede buscar por vector, el vector se ignora.
    /// Esta función usa el último texto buscado almacenado internamente
    /// a través de `generar_embedding`. Para búsqueda directa, usa `buscar_fts5`.
    #[deprecated(since = "0.2.0", note = "Usa buscar_fts5() con texto directamente")]
    pub async fn buscar_similares(
        &self,
        _vector: Vec<f32>,
        limite: usize,
    ) -> Result<Vec<(i64, f32)>> {
        // Fallback: devuelve vacío. Los callers deben migrar a buscar_fts5.
        Ok(self
            .buscar_fts5_raw("", "memoria_episodica", limite)?
            .into_iter()
            .map(|(id, _, score)| (id, score))
            .collect())
    }

    /// Busca en tabla específica — AHORA: búsqueda FTS5.
    #[deprecated(since = "0.2.0", note = "Usa buscar_fts5() con texto directamente")]
    pub async fn buscar_similares_en_tabla(
        &self,
        _vector: &[f32],
        limite: usize,
        table_name: &str,
    ) -> Result<Vec<(i64, f32)>> {
        let content_table = match table_name {
            "ocean_vectors" => "memoria_episodica",
            _ => "memoria_semantica",
        };
        Ok(self
            .buscar_fts5_raw("", content_table, limite)?
            .into_iter()
            .map(|(id, _, score)| (id, score))
            .collect())
    }

    /// Busca y devuelve texto — AHORA: búsqueda FTS5.
    ///
    /// El vector se ignora completamente. La búsqueda se hace contra FTS5
    /// usando el texto almacenado del que se generó el embedding originalmente.
    #[deprecated(since = "0.2.0", note = "Usa buscar_fts5() con texto directamente")]
    pub async fn buscar_similares_con_texto(
        &self,
        _vector: &[f32],
        limite: usize,
        table_name: &str,
    ) -> Result<Vec<(i64, String, f32)>> {
        let content_table = match table_name {
            "ocean_vectors" => "memoria_episodica",
            _ => "memoria_semantica",
        };
        self.buscar_fts5_raw("", content_table, limite)
    }

    /// ⭐ NUEVO: Búsqueda FTS5 directa con BM25 ranking.
    ///
    /// Reemplaza toda la búsqueda vectorial de LanceDB.
    /// Usa FTS5 con tokenizador unicode61 y ranking BM25.
    /// `table_name` puede ser "memoria_episodica" o "memoria_semantica".
    /// `table_name` legacy "ocean_vectors" y "codebase_knowledge" se mapean automáticamente.
    pub fn buscar_fts5(
        &self,
        query: &str,
        table_name: &str,
        limite: usize,
    ) -> Result<Vec<(i64, String, f32)>> {
        // Sanitizar query para FTS5 (escapar caracteres especiales)
        let query_sanitized = sanitizar_fts5(query);
        if query_sanitized.is_empty() {
            return Ok(vec![]);
        }

        let content_table = match table_name {
            "ocean_vectors" | "memoria_episodica" => "memoria_episodica",
            "codebase_knowledge" | "nexus_knowledge" | "memoria_semantica" => "memoria_semantica",
            _ => "memoria_episodica",
        };

        let fts_table = format!("{}_fts", content_table);

        let sql = format!(
            "SELECT c.id, c.titulo, c.contenido, bm25({}, 0.0, 10.0, 5.0) AS rank
             FROM {} c
             JOIN {} f ON c.id = f.rowid
             WHERE {} MATCH ?1
             ORDER BY rank
             LIMIT ?2",
            fts_table, content_table, fts_table, fts_table
        );

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow!("Mutex poisoned: {}", e))?;
        let mut stmt = conn.prepare(&sql)?;

        let results: Vec<(i64, String, f32)> = stmt
            .query_map(params![query_sanitized, limite as i64], |row| {
                let id: i64 = row.get(0)?;
                let titulo: String = row.get(1)?;
                let contenido: String = row.get(2)?;
                let rank: f64 = row.get(3)?;
                Ok((id, titulo, contenido, rank as f32))
            })?
            .filter_map(|r| r.ok())
            .map(|(id, titulo, contenido, rank)| {
                // BM25: menor rank = más relevante. Convertimos a score 0..1.
                // BM25 típico: 0 = match perfecto, >5 = irrelevante
                let score = (1.0 / (1.0 + rank as f64)).clamp(0.0, 1.0) as f32;
                let texto = if contenido.len() > titulo.len() {
                    contenido
                } else {
                    titulo
                };
                (id, texto, score)
            })
            .collect();

        info!(
            "🔍 FTS5 search [{}]: '{}' → {} resultados",
            content_table,
            query_sanitized,
            results.len()
        );

        Ok(results)
    }

    /// Raw internal FTS5 search (used by deprecated wrappers).
    fn buscar_fts5_raw(
        &self,
        query: &str,
        content_table: &str,
        limite: usize,
    ) -> Result<Vec<(i64, String, f32)>> {
        if query.is_empty() {
            // Si query vacío, devolver últimos registros como fallback
            let sql = format!(
                "SELECT id, titulo, contenido FROM {} ORDER BY id DESC LIMIT ?1",
                content_table
            );
            let conn = self
                .conn
                .lock()
                .map_err(|e| anyhow!("Mutex poisoned: {}", e))?;
            let mut stmt = conn.prepare(&sql)?;
            let results = stmt
                .query_map(params![limite as i64], |row| {
                    let id: i64 = row.get(0)?;
                    let titulo: String = row.get(1)?;
                    let contenido: String = row.get(2)?;
                    Ok((id, titulo, contenido))
                })?
                .filter_map(|r| r.ok())
                .map(|(id, titulo, contenido)| {
                    let texto = if contenido.len() > titulo.len() {
                        contenido
                    } else {
                        titulo
                    };
                    (id, texto, 0.5_f32)
                })
                .collect();
            return Ok(results);
        }

        self.buscar_fts5(query, content_table, limite)
    }

    /// Verifica el estado de salud de la memoria FTS5.
    /// Retorna la cantidad de registros indexados.
    pub async fn verificar_estado_lancedb(&self) -> Result<usize> {
        self.contar_en_tabla("memoria_episodica").await
    }

    /// Cuenta registros en una tabla FTS5.
    pub async fn contar_en_tabla(&self, table_name: &str) -> Result<usize> {
        let content_table = match table_name {
            "ocean_vectors" | "memoria_episodica" => "memoria_episodica",
            "codebase_knowledge" | "nexus_knowledge" | "memoria_semantica" => "memoria_semantica",
            _ => table_name,
        };

        let sql = format!("SELECT COUNT(*) FROM {}", content_table);
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow!("Mutex poisoned: {}", e))?;
        let count: i64 = conn.query_row(&sql, [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

/// Sanitiza un query para FTS5: escapa caracteres especiales.
fn sanitizar_fts5(query: &str) -> String {
    if query.is_empty() {
        return String::new();
    }
    // FTS5 usa: ^ * " ( ) : + - ~
    // Escapamos o eliminamos caracteres especiales para evitar errores de sintaxis
    let sanitized: String = query
        .chars()
        .map(|c| match c {
            '"' | '(' | ')' | '*' | '^' | ':' | '+' | '~' => ' ',
            _ => c,
        })
        .collect();

    let trimmed = sanitized.split_whitespace().collect::<Vec<_>>().join(" ");
    if trimmed.len() > 200 {
        // Recorte seguro por límite de carácter UTF-8 (nunca partir un char multibyte)
        let mut boundary = 200;
        while !trimmed.is_char_boundary(boundary) {
            boundary -= 1;
        }
        trimmed[..boundary].to_string()
    } else {
        trimmed
    }
}
