// ============================================================================
// 🧠 MEMORY LOADER — El Cargador de Memoria Unificada (Fase R2)
// ============================================================================
// Puente entre las dos bases del ecosistema:
//   - nexus_memoria.db  → memoria episódica (historial, contexto)
//   - intelligence.db   → identidad (nucleo_identidad) + emocional (ocean)
//
// Produce un `MemoryContext` completo que alimenta al IntentionEncoder y al
// PromptAssembler. Todas las consultas son tolerantes a esquema: si una tabla
// aún no existe, se devuelve vacío en lugar de romper el flujo.
// ============================================================================

use anyhow::Result;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::autonomia::nucleo_identidad::NucleoIdentidad;
use crate::memoria::intention_encoder::{ConceptoSemantico, OceanEsencia};
use crate::memoria::memory::MemoriaPulso;
use crate::nexus_embedder::NexusEmbedder;

/// Contexto de memoria unificado que se ensambla para cada turno.
#[derive(Debug, Clone, Default)]
pub struct MemoryContext {
    /// Descripción textual de la identidad (19 rasgos).
    pub identidad_descripcion: String,
    /// Vector serializado de la identidad ("8:10:...").
    pub identidad_vector: String,
    /// Conversaciones recientes: (timestamp, rol, prompt, respuesta).
    pub conversaciones_recientes: Vec<(String, String, String, String)>,
    /// Conceptos semánticos relevantes para la consulta.
    pub semanticos: Vec<ConceptoSemantico>,
    /// Esencias emocionales (ocean) con carga afectiva.
    pub ocean: Vec<OceanEsencia>,
}

pub struct MemoryLoader {
    memoria_pulso: MemoriaPulso,
    nucleo_identidad: NucleoIdentidad,
    /// Conexión adicional a intelligence.db para consultar ocean/esencias.
    intelligence_conn: Connection,
}

impl MemoryLoader {
    pub fn new() -> Result<Self> {
        let data_dir = crate::infra::paths::resolve_path("data");
        std::fs::create_dir_all(&data_dir)?;

        let nexus_memoria_db_path = data_dir.join("nexus_memoria.db");
        let intelligence_db_path = data_dir.join("intelligence.db");

        let memoria_pulso = MemoriaPulso::new(&nexus_memoria_db_path)?;
        let nucleo_identidad = NucleoIdentidad::new(&intelligence_db_path)
            .map_err(|e| anyhow::anyhow!("no se pudo abrir la identidad: {e}"))?;
        let intelligence_conn = Connection::open(&intelligence_db_path)?;

        info!("🧠 MemoryLoader inicializado — memorias unificadas conectadas");

        Ok(Self {
            memoria_pulso,
            nucleo_identidad,
            intelligence_conn,
        })
    }

    // ========================================================================
    // IDENTIDAD
    // ========================================================================

    pub fn get_identity_description(&self) -> String {
        self.nucleo_identidad.describir_identidad()
    }

    pub fn get_identity_vector(&self) -> String {
        self.nucleo_identidad.obtener_vector_identidad()
    }

    pub fn aprender_de_la_conversacion(&self, prompt: &str) {
        self.nucleo_identidad.aprender_del_prompt(prompt);
    }

    // ========================================================================
    // EPISÓDICA
    // ========================================================================

    pub fn get_recent_conversations(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, String, String)>> {
        self.memoria_pulso.recordar_recientes(session_id, limit)
    }

    pub fn guardar_interaccion(
        &self,
        session_id: &str,
        rol: &str,
        prompt: &str,
        respuesta: &str,
    ) -> Result<()> {
        self.memoria_pulso.guardar_interaccion(session_id, rol, prompt, respuesta)
    }

    // ========================================================================
    // SEMÁNTICA (top-k por similitud coseno sobre memoria_semantica)
    // ========================================================================

    /// Recupera los conceptos semánticos más cercanos a `query`.
    /// Si la tabla `memoria_semantica` no existe aún, devuelve vacío.
    pub fn load_semantic_concepts(&self, query: &str, limit: usize) -> Vec<ConceptoSemantico> {
        let Ok(query_vec) = self.vectorize_semantica(query) else {
            return Vec::new();
        };
        let Ok(rows) = self.query_semantica() else {
            return Vec::new();
        };

        let mut scored: Vec<(ConceptoSemantico, f32)> = rows
            .into_iter()
            .filter_map(|(texto, vector)| {
                let sim = cosine(&query_vec, &vector);
                (sim > 0.3).then(|| {
                    (
                        ConceptoSemantico {
                            texto,
                            embedding: vector,
                            relevancia: sim,
                        },
                        sim,
                    )
                })
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(limit).map(|(c, _)| c).collect()
    }

    /// Genera el vector de consulta reutilizando el embedding de memoria semántica
    /// cuando esté disponible; fallback a NexusEmbedder soberano (768-dim).
    fn vectorize_semantica(&self, texto: &str) -> Result<Vec<f32>> {
        let v = NexusEmbedder::generar(texto, &[]);
        Ok(v)
    }

    /// SELECT texto, vector FROM memoria_semantica (tolerante a esquema).
    fn query_semantica(&self) -> Result<Vec<(String, Vec<f32>)>> {
        let mut stmt = self
            .memoria_pulso
            .conn()
            .prepare("SELECT texto, vector FROM memoria_semantica LIMIT 500")?;
        let rows = stmt.query_map([], |row| {
            let texto: String = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            let vector = blob
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect::<Vec<f32>>();
            Ok((texto, vector))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    // ========================================================================
    // OCEAN (memoria emocional en intelligence.db)
    // ========================================================================

    /// Carga las esencias ocean con intensidad > umbral (tabla `ocean_esencias`).
    pub fn load_ocean_esencias(&self, limit: usize, umbral: f32) -> Vec<OceanEsencia> {
        let Ok(rows) = self.query_ocean() else {
            return Vec::new();
        };
        rows.into_iter()
            .filter(|e| e.intensidad >= umbral)
            .take(limit)
            .collect()
    }

    /// SELECT emocion, intensidad, embedding FROM ocean_esencias (tolerante).
    fn query_ocean(&self) -> Result<Vec<OceanEsencia>> {
        let mut stmt = self
            .intelligence_conn
            .prepare("SELECT emocion, intensidad, vector FROM ocean_esencias LIMIT 200")?;
        let rows = stmt.query_map([], |row| {
            let emocion: String = row.get(0)?;
            let intensidad: f32 = row.get(1)?;
            let blob: Vec<u8> = row.get(2)?;
            let embedding = blob
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect::<Vec<f32>>();
            Ok(OceanEsencia {
                emocion,
                intensidad,
                embedding,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    // ========================================================================
    // ENSAMBLADO DEL CONTEXTO COMPLETO
    // ========================================================================

    /// Carga todo lo necesario para un turno: identidad + episódica + semántica + ocean.
    pub fn load_all(&self, session_id: &str, query: &str) -> MemoryContext {
        let conversaciones_recientes = self
            .get_recent_conversations(session_id, 8)
            .unwrap_or_default();
        let semanticos = self.load_semantic_concepts(query, 5);
        let ocean = self.load_ocean_esencias(3, 0.4);

        MemoryContext {
            identidad_descripcion: self.get_identity_description(),
            identidad_vector: self.get_identity_vector(),
            conversaciones_recientes,
            semanticos,
            ocean,
        }
    }
}

impl Default for MemoryLoader {
    fn default() -> Self {
        Self::new().expect("MemoryLoader debe poder inicializarse")
    }
}

// ----------------------------------------------------------------------------
// Utilidades
// ----------------------------------------------------------------------------

fn cosine(a: &[f32], b: &[f32]) -> f32 {
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

// ----------------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_loader_se_inicializa_y_carga_contexto() {
        // Usa rutas reales de data/ (creadas bajo el dir de trabajo del test).
        let loader = MemoryLoader::new().expect("loader debe abrirse");
        let ctx = loader.load_all("test_sesion", "¿qué recuerdas de NEXUS?");
        assert!(!ctx.identidad_descripcion.is_empty());
        assert!(!ctx.identidad_vector.is_empty());
    }

    #[test]
    fn carga_tolerante_si_faltan_tablas() {
        let loader = MemoryLoader::new().expect("loader debe abrirse");
        // No debe paniquear aunque la tabla de semántica/ocean no exista.
        let sem = loader.load_semantic_concepts("hola", 5);
        let ocean = loader.load_ocean_esencias(3, 0.4);
        // Devuelve lo que haya (puede ser vacío) sin error.
        assert!(sem.len() <= 5);
        assert!(ocean.len() <= 3);
    }

    #[test]
    fn coseno_basico() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];
        assert!((cosine(&a, &b) - 1.0).abs() < 1e-6);
        assert_eq!(cosine(&[], &[]), 0.0);
    }
}
