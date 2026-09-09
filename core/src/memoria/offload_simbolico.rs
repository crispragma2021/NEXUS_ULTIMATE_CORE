// ============================================================================
// ⚡ OFFLOADING SIMBÓLICO — Mermaid Canvas + node_id (porte de TencentDB Agent Memory)
// ============================================================================
// El mayor consumidor de tokens en agentes de larga duración son los logs
// verbosos (resultados de búsqueda, código, trazas de error). TencentDB Agent
// Memory lo resuelve con MEMORIA SIMBÓLICA:
//
//   1. OFFLOAD: el texto completo se vuelca a un archivo externo (refs/*.md)
//   2. SIMBOLIZAR: se extrae un grafo de transición de estados en sintaxis
//      Mermaid, compacto (máximo ~200-300 tokens)
//   3. INYECCIÓN LIGERA: el agente solo ve el canvas Mermaid en contexto
//   4. DRILL-DOWN: para verificar un detalle, hace grep por node_id y recupera
//      el texto crudo completo
//
// Resultado medido por Tencent: -61.38% tokens en WideSearch, +51.52% tasa de
// éxito relativa, sin perder trazabilidad (cada nodo apunta a su evidencia).
//
// Implementación SOBERANA en Rust puro: el canvas Mermaid se genera con
// heurística determinista (sin LLM externo), los logs se persisten bajo
// data/refs/*.md y se indexan por node_id en SQLite.
// ============================================================================

use anyhow::{anyhow, Result};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

use crate::nexus_embedder::NexusEmbedder;

/// Directorio donde se vuelcan los logs crudos (evidencia de bajo nivel).
const REFS_DIR: &str = "data/refs";

/// Nodo del canvas simbólico.
#[derive(Debug, Clone)]
pub struct NodoSimbolico {
    pub node_id: String,
    pub etiqueta: String,
    pub estado: String,
    pub ref_archivo: String,
    pub token_estimados: usize,
    pub creado: String,
}

/// Transición entre nodos (arista del grafo Mermaid).
#[derive(Debug, Clone)]
pub struct Transicion {
    pub origen: String,
    pub destino: String,
    pub etiqueta: String,
}

/// Canvas Mermaid generado + metadatos para drill-down.
#[derive(Debug, Clone)]
pub struct CanvasMermaid {
    pub id: i64,
    pub titulo: String,
    pub mermaid: String,
    pub nodos: Vec<String>, // node_ids
    pub token_estimados: usize,
    pub creado: String,
}

/// Motor de offloading simbólico.
pub struct OffloadSimbolico {
    conn: Connection,
    refs_dir: PathBuf,
}

impl OffloadSimbolico {
    pub fn new() -> Result<Self> {
        let refs_dir = crate::infra::paths::resolve_path(REFS_DIR);
        let db_path = crate::infra::paths::resolve_path("data/nexus_memoria.db");
        Self::from_path(db_path, refs_dir)
    }

    /// Abre el motor en una ruta de DB concreta y un directorio de refs.
    /// `:memory:` es válido para tests aislados.
    fn from_path(db_path: PathBuf, refs_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&refs_dir)?;
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS canvas_mermaid (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                titulo     TEXT NOT NULL,
                mermaid    TEXT NOT NULL,
                token_est  INTEGER NOT NULL DEFAULT 0,
                creado     TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS nodos_simbolicos (
                node_id     TEXT PRIMARY KEY,
                canvas_id   INTEGER NOT NULL,
                etiqueta    TEXT NOT NULL,
                estado      TEXT NOT NULL DEFAULT 'activo',
                ref_archivo TEXT NOT NULL,
                token_est   INTEGER NOT NULL DEFAULT 0,
                creado      TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS transiciones_simbolicas (
                id       INTEGER PRIMARY KEY AUTOINCREMENT,
                canvas_id INTEGER NOT NULL,
                origen   TEXT NOT NULL,
                destino  TEXT NOT NULL,
                etiqueta TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_nodos_canvas ON nodos_simbolicos(canvas_id);",
        )?;

        info!(
            "⚡ [OFFLOAD] Motor simbólico Mermaid listo (refs: {:?})",
            refs_dir
        );
        Ok(Self { conn, refs_dir })
    }

    // ========================================================================
    // OFFLOAD — volcar log crudo a disco + registrar nodo
    // ========================================================================

    /// Vuelca un log/traz a disco bajo data/refs/ y registra un nodo simbólico.
    /// Devuelve el node_id generado (e.g. "n_3").
    pub fn offload_log(&self, canvas_id: i64, etiqueta: &str, log_crudo: &str) -> Result<String> {
        let ts = Utc::now().format("%Y%m%d%H%M%S%3f").to_string();
        let slug = slugificar(etiqueta);
        let filename = format!("{}_{}.md", slug, ts);
        let ruta = self.refs_dir.join(&filename);

        fs::write(&ruta, log_crudo)?;

        let node_id = format!("n_{}", self.contar_nodos() + 1);
        let tokens = estimar_tokens(log_crudo);

        self.conn.execute(
            "INSERT INTO nodos_simbolicos (node_id, canvas_id, etiqueta, estado, ref_archivo, token_est, creado)
             VALUES (?1, ?2, ?3, 'activo', ?4, ?5, ?6)",
            params![node_id, canvas_id, etiqueta, filename, tokens, Utc::now().to_rfc3339()],
        )?;

        info!(
            "📤 [OFFLOAD] nodo {} ({}, {} tok) → refs/{}",
            node_id, etiqueta, tokens, filename
        );
        Ok(node_id)
    }

    // ========================================================================
    // SIMBOLIZAR — construir el canvas Mermaid a partir de nodos y transiciones
    // ========================================================================

    /// Genera el texto Mermaid (grafo de estados) y lo persiste como canvas.
    /// Devuelve el canvas completo.
    pub fn simbolizar(
        &self,
        titulo: &str,
        nodos: &[String],
        transiciones: &[Transicion],
    ) -> Result<CanvasMermaid> {
        if nodos.is_empty() {
            return Err(anyhow!("No hay nodos para simbolizar"));
        }

        let mut mermaid = String::from("graph LR\n");
        for n in nodos {
            // Etiqueta corta (máx 40 chars) para máxima densidad de información.
            let etiqueta = self
                .etiqueta_corta(n)?
                .chars()
                .take(40)
                .collect::<String>()
                .replace(['[', ']', '(', ')', '{', '}'], "");
            mermaid.push_str(&format!("    {}[\"{}\"]\n", n, etiqueta));
        }
        for t in transiciones {
            mermaid.push_str(&format!(
                "    {} -->|{}| {}\n",
                t.origen,
                t.etiqueta.chars().take(20).collect::<String>(),
                t.destino
            ));
        }

        let token_est = estimar_tokens(&mermaid);

        self.conn.execute(
            "INSERT INTO canvas_mermaid (titulo, mermaid, token_est, creado) VALUES (?1, ?2, ?3, ?4)",
            params![titulo, mermaid, token_est as i64, Utc::now().to_rfc3339()],
        )?;
        let id = self.conn.last_insert_rowid();

        info!(
            "🎨 [OFFLOAD] Canvas '{}' generado: {} nodos, {} aristas, {} tok",
            titulo,
            nodos.len(),
            transiciones.len(),
            token_est
        );

        Ok(CanvasMermaid {
            id,
            titulo: titulo.to_string(),
            mermaid,
            nodos: nodos.to_vec(),
            token_estimados: token_est,
            creado: Utc::now().to_rfc3339(),
        })
    }

    /// Crea el canvas de forma declarativa: dado un log gigante, genera nodos
    /// por secciones (separadas por '###' o '---') + transición entre nodos
    /// consecutivos. Es la versión heurística del "pipeline" de Tencent.
    pub fn procesar_log_largo(&self, titulo: &str, log: &str) -> Result<CanvasMermaid> {
        let canvas_id = self.crear_canvas_vacio(titulo)?;

        // Dividir en secciones por marcadores de sección.
        let secciones: Vec<String> = log
            .split("###")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let secciones = if secciones.is_empty() {
            // Fallback: dividir por líneas agrupadas de 20.
            let lineas: Vec<&str> = log.lines().collect();
            lineas.chunks(20).map(|c| c.join("\n")).collect()
        } else {
            secciones
        };

        let mut nodos = Vec::new();
        let mut transiciones = Vec::new();
        let mut anterior: Option<String> = None;

        for (i, seccion) in secciones.iter().enumerate() {
            let etiqueta = format!("Etapa {}", i + 1);
            let node_id = self.offload_log(canvas_id, &etiqueta, seccion)?;
            nodos.push(node_id.clone());
            if let Some(prev) = &anterior {
                transiciones.push(Transicion {
                    origen: prev.clone(),
                    destino: node_id.clone(),
                    etiqueta: "sigue".to_string(),
                });
            }
            anterior = Some(node_id);
        }

        // Si solo hay una sección, forzar al menos 2 nodos para que el grafo sea útil.
        if nodos.len() == 1 {
            let extra = self.offload_log(canvas_id, "Detalle", "Continuación del log.")?;
            nodos.push(extra.clone());
            transiciones.push(Transicion {
                origen: nodos[0].clone(),
                destino: extra,
                etiqueta: "más".to_string(),
            });
        }

        self.simbolizar(titulo, &nodos, &transiciones)
    }

    fn crear_canvas_vacio(&self, titulo: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO canvas_mermaid (titulo, mermaid, token_est, creado) VALUES (?1, '', 0, ?2)",
            params![titulo, Utc::now().to_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    // ========================================================================
    // DRILL-DOWN — recuperar evidencia por node_id
    // ========================================================================

    /// Dado un node_id, devuelve el contenido crudo del archivo ref.
    pub fn recuperar_evidencia(&self, node_id: &str) -> Result<String> {
        let filename: String = self
            .conn
            .query_row(
                "SELECT ref_archivo FROM nodos_simbolicos WHERE node_id = ?1",
                [node_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| anyhow!("{e}"))?
            .ok_or_else(|| anyhow!("node_id no encontrado: {}", node_id))?;

        let ruta = self.refs_dir.join(&filename);
        fs::read_to_string(&ruta).map_err(|e| anyhow!("No se pudo leer {}: {}", ruta.display(), e))
    }

    /// Busca un node_id por texto (grep sobre nodos registrados).
    pub fn buscar_nodo_por_texto(&self, texto: &str) -> Result<Vec<NodoSimbolico>> {
        let q = format!("%{}%", texto);
        let mut stmt = self.conn.prepare(
            "SELECT node_id, canvas_id, etiqueta, estado, ref_archivo, token_est, creado
             FROM nodos_simbolicos WHERE etiqueta LIKE ?1 OR node_id LIKE ?1",
        )?;
        let rows = stmt.query_map([q], |row| {
            Ok(NodoSimbolico {
                node_id: row.get(0)?,
                etiqueta: row.get(2)?,
                estado: row.get(3)?,
                ref_archivo: row.get(4)?,
                token_estimados: row.get(5)?,
                creado: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    /// Obtiene el canvas Mermaid más reciente (para inyección ligera en contexto).
    pub fn canvas_reciente(&self) -> Result<Option<CanvasMermaid>> {
        let base = self
            .conn
            .query_row(
                "SELECT id, titulo, mermaid, token_est, creado
                 FROM canvas_mermaid ORDER BY id DESC LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| anyhow!("{e}"))?;

        let Some((id, titulo, mermaid, token_est, creado)) = base else {
            return Ok(None);
        };

        // Consultar nodos por separado; el iterador se consume dentro del closure
        // antes de que `stmt` se destruya (evita E0515).
        let nodos = self
            .conn
            .prepare("SELECT node_id FROM nodos_simbolicos WHERE canvas_id = ?1 ORDER BY rowid")
            .and_then(|mut stmt| {
                let rows = stmt.query_map([id], |r| r.get::<_, String>(0))?;
                rows.collect::<Result<Vec<_>, _>>()
            })
            .map_err(|e| anyhow!("{e}"))
            .unwrap_or_default();

        Ok(Some(CanvasMermaid {
            id,
            titulo,
            mermaid,
            nodos,
            token_estimados: token_est as usize,
            creado,
        }))
    }

    // ========================================================================
    // Utilidades
    // ========================================================================

    fn contar_nodos(&self) -> i64 {
        self.conn
            .query_row("SELECT COUNT(*) FROM nodos_simbolicos", [], |r| r.get(0))
            .unwrap_or(0)
    }

    fn etiqueta_corta(&self, node_id: &str) -> Result<String> {
        self.conn
            .query_row(
                "SELECT etiqueta FROM nodos_simbolicos WHERE node_id = ?1",
                [node_id],
                |row| row.get(0),
            )
            .map_err(|e| anyhow!("{e}"))
    }
}

impl Default for OffloadSimbolico {
    fn default() -> Self {
        Self::new().expect("OffloadSimbolico debe poder inicializarse")
    }
}

// ============================================================================
// Utilidades puras
// ============================================================================

/// Slug seguro para nombres de archivo.
fn slugificar(texto: &str) -> String {
    let mut slug = String::new();
    let mut separador = false;
    for c in texto.chars() {
        if c.is_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            separador = false;
        } else if !separador {
            slug.push('_');
            separador = true;
        }
    }
    slug.trim_matches('_').chars().take(32).collect()
}

/// Estimación de tokens: ~4 chars/token (español) con piso de 1.
pub fn estimar_tokens(texto: &str) -> usize {
    (texto.len() / 4).max(1)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offload_y_simbolizar() {
        let off =
            OffloadSimbolico::from_path(PathBuf::from(":memory:"), PathBuf::from("data/refs"))
                .expect("offload");
        let canvas_id = off.crear_canvas_vacio("Prueba").expect("canvas");

        let n1 = off
            .offload_log(
                canvas_id,
                "Búsqueda inicial",
                "resultados de web search...\nlinea1\nlinea2",
            )
            .expect("n1");
        let n2 = off
            .offload_log(canvas_id, "Extracción", "contenido extraído del HTML...")
            .expect("n2");

        let canvas = off
            .simbolizar(
                "Flujo de scraping",
                &[n1.clone(), n2.clone()],
                &[Transicion {
                    origen: n1.clone(),
                    destino: n2.clone(),
                    etiqueta: "parsea".to_string(),
                }],
            )
            .expect("simbolizar");

        assert!(canvas.mermaid.contains("graph LR"));
        assert!(canvas.mermaid.contains(&n1));
        assert!(canvas.token_estimados > 0);

        // Drill-down: recuperar evidencia cruda
        let evidencia = off.recuperar_evidencia(&n1).expect("evidencia");
        assert!(evidencia.contains("web search"));

        // Canvas reciente disponible
        assert!(off.canvas_reciente().expect("reciente").is_some());
    }

    #[test]
    fn procesar_log_largo_genera_canvas() {
        let off =
            OffloadSimbolico::from_path(PathBuf::from(":memory:"), PathBuf::from("data/refs"))
                .expect("offload");
        let log = "### Paso 1: fetch\nHTML recibido...\n### Paso 2: parse\nDOM tree...\n### Paso 3: extract\ndatos listos...";
        let canvas = off
            .procesar_log_largo("Pipeline scraping", log)
            .expect("canvas");
        assert!(
            canvas.nodos.len() >= 3,
            "debe haber 3 nodos: {:?}",
            canvas.nodos
        );
        assert!(canvas.mermaid.contains("graph LR"));
    }

    #[test]
    fn estimacion_tokens_simple() {
        assert_eq!(estimar_tokens("hola"), 1);
        assert!(estimar_tokens(&"a".repeat(400)) >= 100);
    }

    #[test]
    fn slugify_basico() {
        assert_eq!(slugificar("Paso 1: fetch!"), "paso_1_fetch");
    }
}
