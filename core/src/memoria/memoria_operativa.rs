// ==========================================
// MEMORIA OPERATIVA - Sesión Activa (Ring Buffer + SQLite FTS5)
// ==========================================
// Hemisferio volátil + persistente a corto plazo.
// Memoria de lo que está SUCEDIENDO ahora.
// Ring Buffer RAM + auto-almacenamiento en nexus_memoria.db
// + inyección de contexto al inicio de sesión.
// ==========================================

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::collections::VecDeque;
use std::sync::Mutex;
use tracing::{info, warn};

/// Capacidad máxima del Ring Buffer en RAM
const CAPACIDAD_RING: usize = 50;

/// Ruta a la base de datos unificada
const NEXUS_MEMORIA_DB: &str = "data/nexus_memoria.db";

/// Umbral para la Curva de Ebbinghaus: importancia mínima para promoción
const UMBRAL_PROMOCION: f64 = 0.65;

/// Tiempo de decaimiento (en segundos) antes de que un recuerdo se considere "viejo"
const DECAIMIENTO_SEGUNDOS: i64 = 3600; // 1 hora

// ---------------------------------------------------------------------------
// Tipos
// ---------------------------------------------------------------------------

/// Un registro en el Ring Buffer de la sesión activa
#[derive(Debug, Clone)]
pub struct RegistroOp {
    pub timestamp: i64,
    pub prompt: String,
    pub respuesta: String,
    pub resumen: String,
    pub emocion: String,
}

/// Resultado de búsqueda
#[derive(Debug, Clone)]
pub struct ResultadoBusqueda {
    pub id: i64,
    pub titulo: String,
    pub contenido: String,
    pub score: f32,
}

// ---------------------------------------------------------------------------
// MemoriaOperativa
// ---------------------------------------------------------------------------

/// Memoria de sesión activa: Ring Buffer en RAM + persistencia a SQLite.
///
/// Dos capas:
/// 1. **Ring Buffer volátil** (`VecDeque<RegistroOp>`) — últimas N interacciones
/// 2. **Persistencia SQLite** (`memoria_episodica` en `nexus_memoria.db`)
pub struct MemoriaOperativa {
    conn: Mutex<Connection>,
    ring: Mutex<VecDeque<RegistroOp>>,
    capacidad: usize,
}

impl MemoriaOperativa {
    /// Abre conexión a `nexus_memoria.db` e inicializa el Ring Buffer.
    pub fn new() -> Result<Self> {
        let path = crate::infra::paths::resolve_path(NEXUS_MEMORIA_DB);
        let conn = Connection::open(&path)
            .with_context(|| format!("No se pudo abrir {}", path.display()))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;

        info!(
            "🧠 MemoriaOperativa activa — Ring Buffer[{}] + nexus_memoria.db",
            CAPACIDAD_RING
        );

        Ok(Self {
            conn: Mutex::new(conn),
            ring: Mutex::new(VecDeque::with_capacity(CAPACIDAD_RING)),
            capacidad: CAPACIDAD_RING,
        })
    }

    /// Crea una instancia para tests con capacidad personalizada
    #[cfg(test)]
    pub fn with_capacidad(cap: usize) -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Self {
            conn: Mutex::new(conn),
            ring: Mutex::new(VecDeque::with_capacity(cap)),
            capacidad: cap,
        })
    }

    // -----------------------------------------------------------------------
    // API pública
    // -----------------------------------------------------------------------

    /// Registra una interacción: push al Ring Buffer + INSERT en memoria_episodica.
    pub fn registrar_interaccion(&self, prompt: &str, respuesta: &str) -> Result<()> {
        let timestamp = Utc::now().timestamp();
        let resumen = prompt
            .chars()
            .take(120)
            .collect::<String>()
            .replace('\n', " ");

        // Detectar tono básico del prompt
        let emocion = detectar_tono(prompt);

        // 1. Ring Buffer
        {
            let mut ring = self
                .ring
                .lock()
                .map_err(|e| anyhow::anyhow!("Mutex ring poisoned: {}", e))?;
            ring.push_back(RegistroOp {
                timestamp,
                prompt: prompt.to_string(),
                respuesta: respuesta.to_string(),
                resumen: resumen.clone(),
                emocion: emocion.clone(),
            });
            if ring.len() > self.capacidad {
                ring.pop_front();
            }
        }

        // 2. Persistencia SQLite (sincrónica, WAL permite lecturas concurrentes)
        {
            let conn = self
                .conn
                .lock()
                .map_err(|e| anyhow::anyhow!("Mutex conn poisoned: {}", e))?;
            conn.execute(
                "INSERT INTO memoria_episodica 
                    (titulo, contenido, importancia, tono_emocional, keywords, created_at, acceso_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, datetime(?6, 'unixepoch'), 1)",
                params![
                    resumen,
                    format!("PROMPT:\n{}\n\nRESPUESTA:\n{}", prompt, respuesta),
                    0.5_f64,      // importancia inicial neutra
                    0.0_f64,      // tono emocional — se recalibra en consolidación
                    emocion,
                    timestamp,
                ],
            )?;
        }

        info!(
            "💾 [MEMOP] Interacción registrada: {}...",
            &resumen[..resumen.len().min(40)]
        );
        Ok(())
    }

    /// Inyecta contexto del historial reciente (Ring Buffer + DB) para el inicio
    /// de una nueva interacción.
    ///
    /// Retorna un String formateado con:
    /// - Últimas `n` interacciones del Ring Buffer
    /// - Contexto persistente de la sesión actual
    /// - Aprendizajes previos relevantes
    pub fn inyectar_contexto(&self, n_recientes: usize) -> Result<String> {
        let mut partes: Vec<String> = Vec::new();

        // 1. Ring Buffer (últimas N interacciones)
        {
            let ring = self
                .ring
                .lock()
                .map_err(|e| anyhow::anyhow!("Mutex ring poisoned: {}", e))?;
            let recientes: Vec<&RegistroOp> = ring.iter().rev().take(n_recientes).collect();
            if !recientes.is_empty() {
                partes.push("## 📋 Últimas interacciones de esta sesión".to_string());
                for (i, r) in recientes.iter().enumerate() {
                    partes.push(format!(
                        "{}. [{}] **Tú:** {}\n   **NEXUS:** {}",
                        i + 1,
                        r.emocion,
                        truncar(&r.prompt, 100),
                        truncar(&r.respuesta, 200),
                    ));
                }
            }
        }

        // 2. Contexto persistente de la sesión actual (desde DB)
        {
            let conn = self
                .conn
                .lock()
                .map_err(|e| anyhow::anyhow!("Mutex conn poisoned: {}", e))?;
            let mut stmt = conn.prepare(
                "SELECT titulo, contenido, importancia, tono_emocional, created_at
                 FROM memoria_episodica
                 WHERE importancia > 0.3
                 ORDER BY id DESC
                 LIMIT 10",
            )?;
            let rows: Vec<(String, String, f64, f64, String)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, f64>(2)?,
                        row.get::<_, f64>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();

            if !rows.is_empty() {
                partes.push("\n## 🧠 Contexto persistente de memoria".to_string());
                for (titulo, contenido, importancia, tono, fecha) in &rows {
                    if *importancia > 0.6 {
                        partes.push(format!(
                            "- **{}** (importancia: {:.2}, tono: {:.1}) — {}\n  ```{}```",
                            titulo,
                            importancia,
                            tono,
                            fecha,
                            truncar(contenido, 150),
                        ));
                    }
                }
            }
        }

        if partes.is_empty() {
            return Ok(
                "[MEMOP] Sin contexto disponible — primera interacción de la sesión.".to_string(),
            );
        }

        Ok(partes.join("\n"))
    }

    /// Búsqueda FTS5 en memoria_episodica reciente.
    pub fn buscar_reciente(&self, query: &str, limite: usize) -> Result<Vec<ResultadoBusqueda>> {
        let query_sanitized = sanitizar_fts5(query);
        if query_sanitized.is_empty() {
            return Ok(vec![]);
        }

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Mutex conn poisoned: {}", e))?;

        let sql = format!(
            "SELECT c.id, c.titulo, c.contenido, bm25(memoria_episodica_fts, 0.0, 10.0, 5.0) AS rank
             FROM memoria_episodica c
             JOIN memoria_episodica_fts f ON c.id = f.rowid
             WHERE memoria_episodica_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2"
        );

        let mut stmt = conn.prepare(&sql)?;
        let results: Vec<ResultadoBusqueda> = stmt
            .query_map(params![query_sanitized, limite as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, f64>(3)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .map(|(id, titulo, contenido, rank)| {
                let score = (1.0 / (1.0 + rank)).clamp(0.0, 1.0) as f32;
                ResultadoBusqueda {
                    id,
                    titulo,
                    contenido,
                    score,
                }
            })
            .collect();

        Ok(results)
    }

    /// Vuelca el Ring Buffer completo (para debugging / snapshot).
    pub fn snapshot_ring(&self) -> Result<Vec<RegistroOp>> {
        let ring = self
            .ring
            .lock()
            .map_err(|e| anyhow::anyhow!("Mutex ring poisoned: {}", e))?;
        Ok(ring.iter().cloned().collect())
    }

    /// Curva de Ebbinghaus: evalúa decaimiento y promueve recuerdos importantes a semántica.
    ///
    /// 1. Busca registros en `memoria_episodica` con alta importancia y bajo acceso_count
    /// 2. Calcula factor de decaimiento basado en tiempo desde creación
    /// 3. Si supera umbral, promueve a `memoria_semantica`
    /// 4. Actualiza acceso_count en los registros consultados
    ///
    /// Retorna IDs de los registros promovidos.
    pub fn ebbinghaus_tick(&self, semantica_conn: &Connection) -> Result<Vec<i64>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Mutex conn poisoned: {}", e))?;

        // 1. Identificar candidatos a promoción: importancia alta, no promovidos aún
        let mut stmt = conn.prepare(
            "SELECT id, titulo, contenido, importancia, tono_emocional, 
                    created_at, acceso_count
             FROM memoria_episodica
             WHERE importancia > ?1
               AND (keywords NOT LIKE '%promovido%' OR keywords IS NULL)
             ORDER BY importancia DESC
             LIMIT 20",
        )?;

        let ahora = Utc::now().timestamp();
        let mut promovidos: Vec<i64> = Vec::new();

        let candidatos: Vec<(i64, String, String, f64, f64, String, i64)> = stmt
            .query_map(params![UMBRAL_PROMOCION], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get::<_, i64>(6)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        for (id, titulo, contenido, importancia, tono, created_at, acceso_count) in &candidatos {
            // 2. Calcular factor de decaimiento
            let created_ts = parse_fecha_sqlite(created_at).unwrap_or(0);
            let edad = ahora - created_ts;
            let factor_decaimiento = if edad > 0 {
                (-(edad as f64) / DECAIMIENTO_SEGUNDOS as f64).exp()
            } else {
                1.0
            };

            // 3. Puntaje Ebbinghaus: importancia * factor_decaimiento * log(acceso + 1)
            let score_ebbinghaus =
                importancia * factor_decaimiento * ((*acceso_count as f64) + 1.0).ln();

            if score_ebbinghaus > UMBRAL_PROMOCION {
                // 4. Promover a memoria_semantica
                let keywords_promovido = format!(
                    "promovido;importancia={:.2};ebbinghaus={:.2}",
                    importancia, score_ebbinghaus
                );

                // Insertar en memoria_semantica
                let result = semantica_conn.execute(
                    "INSERT OR IGNORE INTO memoria_semantica 
                        (titulo, contenido, importancia, tono_emocional, keywords, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        titulo,
                        contenido,
                        importancia,
                        tono,
                        &keywords_promovido,
                        created_at,
                    ],
                );

                match result {
                    Ok(_) => {
                        // Marcar como promovido en el registro episódico original
                        let _ = conn.execute(
                            "UPDATE memoria_episodica SET keywords = COALESCE(keywords, '') || ';promovido' WHERE id = ?1",
                            params![id],
                        );
                        promovidos.push(*id);
                        info!(
                            "🧪 [EBBINGHAUS] Promovido id={} a semántica (score={:.2}, edad={}s)",
                            id, score_ebbinghaus, edad
                        );
                    }
                    Err(e) => {
                        warn!("⚠️ [EBBINGHAUS] Error promoviendo id={}: {}", id, e);
                    }
                }
            }
        }

        // 5. Incrementar acceso_count de los registros existentes
        let _ = conn.execute(
            "UPDATE memoria_episodica SET acceso_count = COALESCE(acceso_count, 0) + 1 WHERE acceso_count IS NOT NULL",
            [],
        );

        if !promovidos.is_empty() {
            info!(
                "🧪 [EBBINGHAUS] {} recuerdos promovidos a semántica",
                promovidos.len()
            );
        }

        Ok(promovidos)
    }
}

// ---------------------------------------------------------------------------
// Funciones auxiliares
// ---------------------------------------------------------------------------

/// Detecta el tono emocional básico de un texto.
fn detectar_tono(texto: &str) -> String {
    let lower = texto.to_lowercase();
    if lower.contains("error")
        || lower.contains("falla")
        || lower.contains("🔥")
        || lower.contains("⚠️")
    {
        "urgencia".to_string()
    } else if lower.contains("gracias") || lower.contains("bien") || lower.contains("excelente") {
        "positivo".to_string()
    } else if lower.contains("?") {
        "consulta".to_string()
    } else if lower.contains("diseño") || lower.contains("ui") || lower.contains("creativo") {
        "creativo".to_string()
    } else {
        "neutro".to_string()
    }
}

/// Trunca texto a una longitud máxima.
fn truncar(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

/// Parsea fecha SQLite (formato ISO 8601) a timestamp Unix.
fn parse_fecha_sqlite(fecha: &str) -> Option<i64> {
    // Formato esperado: "2026-06-29 12:34:56" o "2026-06-29T12:34:56"
    let fecha_limpia = fecha.replace('T', " ");
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&fecha_limpia, "%Y-%m-%d %H:%M:%S") {
        Some(dt.and_utc().timestamp())
    } else {
        None
    }
}

/// Sanitiza query para FTS5.
fn sanitizar_fts5(query: &str) -> String {
    if query.is_empty() {
        return String::new();
    }
    let sanitized: String = query
        .chars()
        .map(|c| match c {
            '"' | '(' | ')' | '*' | '^' | ':' | '+' | '~' => ' ',
            _ => c,
        })
        .collect();

    let trimmed = sanitized.split_whitespace().collect::<Vec<_>>().join(" ");
    if trimmed.len() > 200 {
        trimmed[..200].to_string()
    } else {
        trimmed
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitizar_fts5() {
        assert_eq!(sanitizar_fts5(""), "");
        assert_eq!(sanitizar_fts5("hola mundo"), "hola mundo");
        assert_eq!(sanitizar_fts5("test: (parentesis)"), "test parentesis");
    }

    #[test]
    fn test_detectar_tono() {
        assert_eq!(detectar_tono("hay un error grave"), "urgencia");
        assert_eq!(detectar_tono("gracias excelente trabajo"), "positivo");
        assert_eq!(detectar_tono("qué es esto?"), "consulta");
        assert_eq!(detectar_tono("hola mundo"), "neutro");
    }

    #[test]
    fn test_parse_fecha_sqlite() {
        let ts = parse_fecha_sqlite("2026-06-29 12:00:00");
        assert!(ts.is_some());
        assert!(ts.unwrap() > 0);
    }
}
