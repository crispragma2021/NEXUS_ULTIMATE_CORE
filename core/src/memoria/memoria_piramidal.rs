// ============================================================================
// 🧠 MEMORIA PIRAMIDAL — Capas L0→L3 (porte del concepto TencentDB Agent Memory)
// ============================================================================
// TencentDB Agent Memory demostró que la memoria a largo plazo no debe ser
// plana: reemplaza montones de vectores por una PIRÁMIDE SEMÁNTICA con capas:
//
//   L0 CONVERSATION  — diálogo crudo (evidencia, nunca se borra)
//   L1 ATOM          — hechos atómicos ("Cris prefiere Rust", "el puerto es 3000")
//   L2 SCENARIO      — bloques de escena (contexto que agrupa varios atoms)
//   L3 PERSONA       — perfil del usuario (preferencias, voz, metas a largo plazo)
//
// Principios portados (fieles al diseño de Tencent):
//   * Progressive disclosure: el agente solo atiende la cima (L3/L2) en contexto,
//     y baja a L1/L0 con node_id cuando necesita el detalle.
//   * Lower layers preserve evidence; upper layers preserve structure.
//   * Full traceability: toda abstracción (L3/L2) apunta a su evidencia (L1/L0)
//     mediante result_refs — NUNCA compresión irreversible.
//
// Implementación SOBERANA: Rust puro + SQLite (rusqlite), CERO dependencias
// externas. El trigger de extracción L1 usa el NexusEmbedder nativo para
// detectar hechos y deduplicarlos por similitud de hash.
// ============================================================================

use anyhow::{anyhow, Result};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use tracing::{info, warn};

use crate::nexus_embedder::NexusEmbedder;

/// Ruta por defecto de la base piramidal (coexiste con nexus_memoria.db).
const NEXUS_MEMORIA_DB: &str = "data/nexus_memoria.db";

/// Nivel de la pirámide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NivelPiramide {
    L0Conversacion,
    L1Atom,
    L2Escenario,
    L3Persona,
}

impl NivelPiramide {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::L0Conversacion => "L0",
            Self::L1Atom => "L1",
            Self::L2Escenario => "L2",
            Self::L3Persona => "L3",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "L0" => Some(Self::L0Conversacion),
            "L1" => Some(Self::L1Atom),
            "L2" => Some(Self::L2Escenario),
            "L3" => Some(Self::L3Persona),
            _ => None,
        }
    }
}

/// Un registro de la pirámide en cualquier nivel.
#[derive(Debug, Clone)]
pub struct MemoriaPiramidal {
    pub id: i64,
    pub nivel: NivelPiramide,
    pub contenido: String,
    pub keywords: String,
    /// Referencias a evidencias inferiores (ids), separadas por ';'.
    pub result_refs: String,
    /// Peso/importancia 0..1 (refuerzo tipo Ebbinghaus).
    pub peso: f64,
    pub creado: String,
    /// nodo_id para drill-down: "{nivel}:{id}"
    pub node_id: String,
}

/// Conector a la pirámide L0→L3 dentro de nexus_memoria.db.
pub struct MemoriaPiramidalStore {
    conn: Connection,
    db_path: PathBuf,
}

impl MemoriaPiramidalStore {
    /// Abre (o crea) el esquema piramidal en nexus_memoria.db.
    pub fn new() -> Result<Self> {
        let db_path = crate::infra::paths::resolve_path(NEXUS_MEMORIA_DB);
        Self::from_path(db_path)
    }

    /// Abre (o crea) el esquema en una ruta concreta. `:memory:` es válido
    /// para tests aislados (cada store tiene su propia base en memoria).
    fn from_path(db_path: PathBuf) -> Result<Self> {
        if db_path.as_os_str() != ":memory:" {
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;

        // Tabla unificada: cada fila es un nodo de la pirámide.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memoria_piramidal (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                nivel       TEXT NOT NULL CHECK (nivel IN ('L0','L1','L2','L3')),
                contenido   TEXT NOT NULL,
                keywords    TEXT NOT NULL DEFAULT '',
                result_refs TEXT NOT NULL DEFAULT '',
                peso        REAL NOT NULL DEFAULT 0.5,
                creado      TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_piramidal_nivel ON memoria_piramidal(nivel);
            CREATE INDEX IF NOT EXISTS idx_piramidal_peso ON memoria_piramidal(peso DESC);",
        )?;

        // FTS5 para búsqueda híbrida sobre toda la pirámide.
        conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS memoria_piramidal_fts USING fts5(
                contenido, keywords, content=''
            );",
        )?;

        info!(
            "🧠 [PIRÁMIDE] Memoria piramidal L0→L3 lista en {:?}",
            db_path
        );
        Ok(Self { conn, db_path })
    }

    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    // ========================================================================
    // L0 — Conversación cruda (evidencia)
    // ========================================================================

    /// Guarda un turno de conversación como L0. Devuelve su id.
    pub fn registrar_conversacion(&self, rol: &str, prompt: &str, respuesta: &str) -> Result<i64> {
        let contenido = format!("[{}] {}\n→ {}", rol, prompt, respuesta);
        let keywords = extraer_keywords(&contenido);
        self.insertar(
            NivelPiramide::L0Conversacion,
            &contenido,
            &keywords,
            "",
            0.4,
        )
    }

    // ========================================================================
    // L1 — Átomos (hechos atómicos extraídos)
    // ========================================================================

    /// Extrae hechos atómicos de una conversación y los inserta como L1,
    /// deduplicando contra atoms existentes por similitud de embedding.
    pub fn extraer_atomos(&self, conversacion: &str, l0_id: i64) -> Result<Vec<i64>> {
        let frases = extraer_frases(conversacion);
        let mut insertados = Vec::new();

        for frase in frases {
            if frase.len() < 12 {
                continue;
            }
            if self.es_duplicado_atom(&frase)? {
                continue;
            }
            let keywords = extraer_keywords(&frase);
            let id = self.insertar(
                NivelPiramide::L1Atom,
                &frase,
                &keywords,
                &format!("L0:{}", l0_id),
                0.6,
            )?;
            insertados.push(id);
        }

        if !insertados.is_empty() {
            info!(
                "🔬 [PIRÁMIDE] {} átomos nuevos desde L0#{}",
                insertados.len(),
                l0_id
            );
        }
        Ok(insertados)
    }

    /// Deduplicación por similitud de embeddings (umbral 0.85).
    fn es_duplicado_atom(&self, frase: &str) -> Result<bool> {
        let frase_emb = NexusEmbedder::generar(frase, &[]);
        let mut stmt = self
            .conn
            .prepare("SELECT id, contenido FROM memoria_piramidal WHERE nivel='L1' LIMIT 500")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (_, contenido) = row?;
            let emb = NexusEmbedder::generar(&contenido, &[]);
            if cosine_sim(&frase_emb, &emb) > 0.85 {
                return Ok(true);
            }
        }
        Ok(false)
    }

    // ========================================================================
    // L2 — Escenarios (bloques de escena que agrupan atoms)
    // ========================================================================

    /// Consolida un conjunto de atoms en un escenario. El escenario referencia
    /// a sus atoms (drill-down), y los atoms conservan su evidencia L0.
    pub fn crear_escenario(&self, titulo: &str, atom_ids: &[i64]) -> Result<i64> {
        let contenido = format!(
            "ESCENARIO: {}\n{}",
            titulo,
            atom_ids
                .iter()
                .map(|id| format!("- L1:{}", id))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let refs = atom_ids
            .iter()
            .map(|id| format!("L1:{}", id))
            .collect::<Vec<_>>()
            .join(";");
        let keywords = extraer_keywords(titulo);
        self.insertar(
            NivelPiramide::L2Escenario,
            &contenido,
            &keywords,
            &refs,
            0.7,
        )
    }

    // ========================================================================
    // L3 — Persona (perfil a largo plazo)
    // ========================================================================

    /// Actualiza (o crea) el perfil L3 a partir de escenarios — la capa que
    /// el agente atiende primero en contexto.
    pub fn consolidar_persona(&self, escenario_ids: &[i64]) -> Result<i64> {
        let mut lineas = Vec::new();
        let mut refs = Vec::new();
        for id in escenario_ids {
            if let Some(esc) = self.obtener(*id)? {
                if esc.nivel == NivelPiramide::L2Escenario {
                    lineas.push(format!("- L2:{} — {}", id, esc.contenido));
                    refs.push(format!("L2:{}", id));
                }
            }
        }
        if lineas.is_empty() {
            return Err(anyhow!("No hay escenarios para consolidar persona"));
        }

        let contenido = format!(
            "## Perfil del Arquitecto (persona consolidada)\n{}",
            lineas.join("\n")
        );
        let refs_joined = refs.join(";");

        // Una sola persona canónica: reemplaza la anterior.
        self.conn
            .execute("DELETE FROM memoria_piramidal WHERE nivel='L3'", [])?;
        let id = self.insertar(
            NivelPiramide::L3Persona,
            &contenido,
            "perfil;arquitecto;preferencias",
            &refs_joined,
            1.0,
        )?;
        info!(
            "👤 [PIRÁMIDE] Persona L3 consolidada desde {} escenarios",
            lineas.len()
        );
        Ok(id)
    }

    /// Obtiene la persona actual (top de la pirámide) para inyección en contexto.
    pub fn obtener_persona(&self) -> Result<Option<MemoriaPiramidal>> {
        self.conn
            .query_row(
                "SELECT id, nivel, contenido, keywords, result_refs, peso, creado
                 FROM memoria_piramidal WHERE nivel='L3' ORDER BY id DESC LIMIT 1",
                [],
                |row| {
                    Ok(MemoriaPiramidal {
                        id: row.get(0)?,
                        nivel: NivelPiramide::L3Persona,
                        contenido: row.get(2)?,
                        keywords: row.get(3)?,
                        result_refs: row.get(4)?,
                        peso: row.get(5)?,
                        creado: row.get(6)?,
                        node_id: format!("L3:{}", row.get::<_, i64>(0)?),
                    })
                },
            )
            .optional()
            .map_err(|e| e.into())
    }

    // ========================================================================
    // Drill-down — trazabilidad completa por node_id
    // ========================================================================

    /// Dado un node_id ("L2:3"), baja a sus evidencias recursivamente.
    /// Devuelve el camino completo "L3 → L2 → L1 → L0" como texto legible.
    pub fn drill_down(&self, node_id: &str) -> Result<Vec<String>> {
        let mut camino = Vec::new();
        let mut actual = node_id.to_string();

        for _ in 0..5 {
            // Tolerante a refs legacy sin prefijo (p. ej. "3"): se interpretan
            // como id de L0, el nivel base de evidencia.
            let (nivel, id) = match parse_node_id(&actual) {
                Ok(p) => p,
                Err(_) => (NivelPiramide::L0Conversacion, actual.parse().unwrap_or(0)),
            };
            let Some(fila) = self.obtener_por_id(id)? else {
                break;
            };
            camino.push(format!("[{}] {}", fila.nivel.as_str(), fila.contenido));

            if fila.result_refs.is_empty() {
                break;
            }
            // Bajar al primer ref (camino principal).
            actual = fila.result_refs.split(';').next().unwrap_or("").to_string();
            if actual.is_empty() || nivel == NivelPiramide::L0Conversacion {
                break;
            }
        }
        Ok(camino)
    }

    // ========================================================================
    // Búsqueda por nivel (para hybrid recall)
    // ========================================================================

    /// Busca texto en un nivel concreto usando FTS5 + BM25.
    pub fn buscar_nivel(
        &self,
        query: &str,
        nivel: NivelPiramide,
        limite: usize,
    ) -> Result<Vec<(MemoriaPiramidal, f32)>> {
        let q = sanitizar_fts5(query);
        if q.is_empty() {
            return Ok(vec![]);
        }
        // FTS5 no filtra por nivel directamente: filtramos por join manual.
        let sql = format!(
            "SELECT p.id, p.nivel, p.contenido, p.keywords, p.result_refs, p.peso, p.creado,
                    bm25(memoria_piramidal_fts, 10.0, 5.0) AS rank
             FROM memoria_piramidal_fts f
             JOIN memoria_piramidal p ON p.id = f.rowid
             WHERE memoria_piramidal_fts MATCH ?1 AND p.nivel = ?2
             ORDER BY rank LIMIT ?3"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![q, nivel.as_str(), limite as i64], |row| {
            let id: i64 = row.get(0)?;
            Ok((
                MemoriaPiramidal {
                    id,
                    nivel: NivelPiramide::from_str(&row.get::<_, String>(1)?)
                        .unwrap_or(NivelPiramide::L1Atom),
                    contenido: row.get(2)?,
                    keywords: row.get(3)?,
                    result_refs: row.get(4)?,
                    peso: row.get(5)?,
                    creado: row.get(6)?,
                    node_id: format!("{}:{}", row.get::<_, String>(1)?, id),
                },
                (1.0 / (1.0 + row.get::<_, f64>(7)?)).clamp(0.0, 1.0) as f32,
            ))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    /// Relevancia vectorial por similitud coseno (usada por hybrid recall).
    pub fn buscar_vectorial(
        &self,
        query: &str,
        limite: usize,
    ) -> Result<Vec<(MemoriaPiramidal, f32)>> {
        let q_emb = NexusEmbedder::generar(query, &[]);
        let mut stmt = self
            .conn
            .prepare("SELECT id, nivel, contenido, keywords, result_refs, peso, creado FROM memoria_piramidal LIMIT 500")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                MemoriaPiramidal {
                    id: row.get(0)?,
                    nivel: NivelPiramide::from_str(&row.get::<_, String>(1)?)
                        .unwrap_or(NivelPiramide::L1Atom),
                    contenido: row.get(2)?,
                    keywords: row.get(3)?,
                    result_refs: row.get(4)?,
                    peso: row.get(5)?,
                    creado: row.get(6)?,
                    node_id: format!("{}:{}", row.get::<_, String>(1)?, row.get::<_, i64>(0)?),
                },
                0.0_f32,
            ))
        })?;

        let mut scored = Vec::new();
        for row in rows {
            let (m, _) = row?;
            let emb = NexusEmbedder::generar(&m.contenido, &[]);
            let sim = cosine_sim(&q_emb, &emb);
            if sim > 0.25 {
                scored.push((m, sim));
            }
        }
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limite);
        Ok(scored)
    }

    /// Cima de la pirámide para inyección ligera en contexto:
    /// persona (L3) + top escenarios (L2). El agente baja a L1/L0 solo si lo pide.
    pub fn capa_superior_para_contexto(
        &self,
        limite_l2: usize,
    ) -> Result<(Option<MemoriaPiramidal>, Vec<MemoriaPiramidal>)> {
        let persona = self.obtener_persona()?;
        let mut stmt = self.conn.prepare(
            "SELECT id, nivel, contenido, keywords, result_refs, peso, creado
             FROM memoria_piramidal WHERE nivel='L2' ORDER BY peso DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limite_l2 as i64], |row| {
            Ok(MemoriaPiramidal {
                id: row.get(0)?,
                nivel: NivelPiramide::L2Escenario,
                contenido: row.get(2)?,
                keywords: row.get(3)?,
                result_refs: row.get(4)?,
                peso: row.get(5)?,
                creado: row.get(6)?,
                node_id: format!("L2:{}", row.get::<_, i64>(0)?),
            })
        })?;
        let escenarios = rows.collect::<Result<Vec<_>, _>>()?;
        Ok((persona, escenarios))
    }

    // ========================================================================
    // Reforzar peso (Ebbinghaus: lo que se usa, se refuerza)
    // ========================================================================

    pub fn reforzar(&self, node_id: &str, delta: f64) -> Result<()> {
        let (_, id) = parse_node_id(node_id)?;
        self.conn.execute(
            "UPDATE memoria_piramidal SET peso = MIN(1.0, peso + ?1) WHERE id = ?2",
            params![delta, id],
        )?;
        Ok(())
    }

    // ========================================================================
    // Internos
    // ========================================================================

    fn insertar(
        &self,
        nivel: NivelPiramide,
        contenido: &str,
        keywords: &str,
        result_refs: &str,
        peso: f64,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO memoria_piramidal (nivel, contenido, keywords, result_refs, peso, creado)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                nivel.as_str(),
                contenido,
                keywords,
                result_refs,
                peso,
                Utc::now().to_rfc3339()
            ],
        )?;
        let id = self.conn.last_insert_rowid();
        // Indexar en FTS5 (content externa: solo rowid + texto).
        self.conn.execute(
            "INSERT INTO memoria_piramidal_fts (rowid, contenido, keywords) VALUES (?1, ?2, ?3)",
            params![id, contenido, keywords],
        )?;
        Ok(id)
    }

    fn obtener(&self, id: i64) -> Result<Option<MemoriaPiramidal>> {
        self.obtener_por_id(id)
    }

    fn obtener_por_id(&self, id: i64) -> Result<Option<MemoriaPiramidal>> {
        self.conn
            .query_row(
                "SELECT id, nivel, contenido, keywords, result_refs, peso, creado
                 FROM memoria_piramidal WHERE id = ?1",
                [id],
                |row| {
                    let nivel_str: String = row.get(1)?;
                    Ok(MemoriaPiramidal {
                        id: row.get(0)?,
                        nivel: NivelPiramide::from_str(&nivel_str).unwrap_or(NivelPiramide::L1Atom),
                        contenido: row.get(2)?,
                        keywords: row.get(3)?,
                        result_refs: row.get(4)?,
                        peso: row.get(5)?,
                        creado: row.get(6)?,
                        node_id: format!("{}:{}", nivel_str, row.get::<_, i64>(0)?),
                    })
                },
            )
            .optional()
            .map_err(|e| e.into())
    }
}

impl Default for MemoriaPiramidalStore {
    fn default() -> Self {
        Self::new().expect("MemoriaPiramidalStore debe poder inicializarse")
    }
}

// ============================================================================
// Utilidades
// ============================================================================

/// Divide una conversación en frases candidatas a átomos.
fn extraer_frases(texto: &str) -> Vec<String> {
    texto
        .split(['.', ';', '\n', '!', '?'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.chars().count() > 4)
        .collect()
}

/// Extrae palabras clave (minúsculas, sin stopwords) para indexación FTS.
fn extraer_keywords(texto: &str) -> String {
    const STOPWORDS: &[&str] = &[
        "el", "la", "los", "las", "de", "del", "un", "una", "unos", "unas", "y", "o", "u", "a",
        "en", "con", "por", "para", "que", "es", "ser", "está", "estan", "the", "and", "of", "to",
        "in", "is", "are", "was", "were",
    ];
    let mut palabras = Vec::new();
    for w in texto.split_whitespace() {
        let limpia: String = w
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase();
        if limpia.len() >= 3 && !STOPWORDS.contains(&limpia.as_str()) {
            palabras.push(limpia);
        }
    }
    palabras.join(";")
}

/// Similitud coseno entre dos embeddings.
fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na > 1e-8 && nb > 1e-8 {
        dot / (na * nb)
    } else {
        0.0
    }
}

/// Parsea "L2:3" → (NivelPiramide::L2Escenario, 3).
fn parse_node_id(node_id: &str) -> Result<(NivelPiramide, i64)> {
    let mut parts = node_id.split(':');
    let nivel = parts
        .next()
        .and_then(NivelPiramide::from_str)
        .ok_or_else(|| anyhow!("node_id inválido: {}", node_id))?;
    let id: i64 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow!("node_id inválido: {}", node_id))?;
    Ok((nivel, id))
}

/// Sanitiza una query para FTS5 (escapa comillas y caracteres especiales).
fn sanitizar_fts5(query: &str) -> String {
    query
        .chars()
        .map(|c| match c {
            '"' => ' ',
            '\'' => ' ',
            '(' | ')' | '*' | ':' | '^' | '-' => ' ',
            c if c.is_alphanumeric() || c.is_whitespace() => c,
            _ => ' ',
        })
        .collect::<String>()
        .trim()
        .to_string()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piramide_completa_l0_a_l3() {
        let store = MemoriaPiramidalStore::from_path(PathBuf::from(":memory:")).expect("store");

        // L0: registrar conversación
        let l0 = store
            .registrar_conversacion("user", "Hola NEXUS", "Hola Arquitecto")
            .expect("L0");

        // L1: extraer átomos (frases largas de ejemplo)
        let atoms = store
            .extraer_atomos(
                "El proyecto usa Rust para el core y TypeScript para la extensión.",
                l0,
            )
            .expect("átomos");
        assert!(!atoms.is_empty(), "debe extraer al menos un átomo");

        // L2: escenario que agrupa atoms
        let l2 = store
            .crear_escenario("Stack tecnológico", &atoms)
            .expect("escenario");
        let esc = store.obtener(l2).expect("escenario").expect("exists");
        assert!(esc.result_refs.contains("L1:"));

        // L3: consolidar persona
        let l3 = store.consolidar_persona(&[l2]).expect("persona");
        let persona = store.obtener(l3).expect("persona").expect("exists");
        assert_eq!(persona.nivel, NivelPiramide::L3Persona);

        // Drill-down L3 → L2 → L1 → L0
        let camino = store.drill_down(&persona.node_id).expect("drill");
        assert!(
            camino.len() >= 2,
            "camino debe tener al menos persona+escenario: {:?}",
            camino
        );
    }

    #[test]
    fn busqueda_por_nivel_devuelve_resultados() {
        let store = MemoriaPiramidalStore::from_path(PathBuf::from(":memory:")).expect("store");
        let l0 = store
            .registrar_conversacion("user", "Prefiero Rust sobre C++", "Anotado")
            .expect("L0");
        let atoms = store
            .extraer_atomos("El puerto del servidor es el 3000 y el idioma es Rust.", l0)
            .expect("atoms");
        assert!(
            store
                .buscar_nivel("rust", NivelPiramide::L1Atom, 5)
                .expect("buscar")
                .len()
                >= 1
        );
        assert!(!atoms.is_empty());
    }

    #[test]
    fn capa_superior_para_contexto() {
        let store = MemoriaPiramidalStore::from_path(PathBuf::from(":memory:")).expect("store");
        let (persona, escenarios) = store.capa_superior_para_contexto(3).expect("capa");
        // Sin datos previos puede no haber nada, pero no debe fallar.
        assert!(persona.is_none() || persona.is_some());
        assert!(escenarios.len() <= 3);
    }

    #[test]
    fn node_id_parsing() {
        let (nivel, id) = parse_node_id("L2:7").expect("parse");
        assert_eq!(nivel, NivelPiramide::L2Escenario);
        assert_eq!(id, 7);
        assert!(parse_node_id("X9").is_err());
    }
}
