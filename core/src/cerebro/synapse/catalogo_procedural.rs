// ============================================================================
// core/src/cerebro/synapse/catalogo_procedural.rs — CATÁLOGO PROCEDURAL HIPOCÁMPICO
// ============================================================================
// Propósito: Indexación de artefactos procedimentales (código, parches, scripts, prompts)
//            con deduplicación basada en firmas SHA256 y relaciones conceptuales (HippoRAG).
// ============================================================================

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TipoArtefacto {
    ModuloRust,
    ScriptShell,
    ParcheCodigo,
    PromptPlantilla,
    HerramientaNativa,
}

impl std::fmt::Display for TipoArtefacto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TipoArtefacto::ModuloRust => write!(f, "ModuloRust"),
            TipoArtefacto::ScriptShell => write!(f, "ScriptShell"),
            TipoArtefacto::ParcheCodigo => write!(f, "ParcheCodigo"),
            TipoArtefacto::PromptPlantilla => write!(f, "PromptPlantilla"),
            TipoArtefacto::HerramientaNativa => write!(f, "HerramientaNativa"),
        }
    }
}

impl std::str::FromStr for TipoArtefacto {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ModuloRust" => Ok(TipoArtefacto::ModuloRust),
            "ScriptShell" => Ok(TipoArtefacto::ScriptShell),
            "ParcheCodigo" => Ok(TipoArtefacto::ParcheCodigo),
            "PromptPlantilla" => Ok(TipoArtefacto::PromptPlantilla),
            "HerramientaNativa" => Ok(TipoArtefacto::HerramientaNativa),
            _ => Ok(TipoArtefacto::ModuloRust),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtefactoProcedural {
    pub id: String,
    pub nombre: String,
    pub tipo: TipoArtefacto,
    pub firma_ast: String,
    pub hash_sha256: String,
    pub contenido: String,
    pub activo: bool,
    pub version: u32,
    pub conceptos_clave: Vec<String>,
    pub creado_en: String,
}

pub struct CatalogoHipocampico {
    pub artefactos: HashMap<String, ArtefactoProcedural>,
    pub hash_index: HashMap<String, String>, // sha256 -> artefacto_id
    pub db_path: Option<PathBuf>,
}

impl Default for CatalogoHipocampico {
    fn default() -> Self {
        Self::new()
    }
}

impl CatalogoHipocampico {
    pub fn new() -> Self {
        Self {
            artefactos: HashMap::new(),
            hash_index: HashMap::new(),
            db_path: None,
        }
    }

    pub fn set_db_path(&mut self, path: PathBuf) {
        self.db_path = Some(path);
    }

    /// Calcula la firma hash SHA256 única del contenido del artefacto.
    pub fn calcular_hash(contenido: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(contenido.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Registra un artefacto procedimental con deduplicación garantizada (Zero-Duplicates).
    /// Retorna `(id_artefacto, es_nuevo)`. Si ya existía, retorna `(id_existente, false)`.
    pub fn registrar(
        &mut self,
        nombre: &str,
        tipo: TipoArtefacto,
        firma_ast: &str,
        contenido: &str,
        conceptos: &[&str],
    ) -> anyhow::Result<(String, bool)> {
        let hash = Self::calcular_hash(contenido);

        // 1. Control Anti-Duplicados (Ley de Oro): Si el hash ya existe, rechazar duplicación
        if let Some(id_existente) = self.hash_index.get(&hash) {
            info!(
                "♻️ [HIPPORAG-CATALOGO] Artefacto duplicado detectado ('{}', hash: {}...). Reusando id: {}",
                nombre,
                &hash[..8.min(hash.len())],
                id_existente
            );
            return Ok((id_existente.clone(), false));
        }

        // 2. Crear nuevo artefacto procedural
        let id = format!("art_{}", uuid::Uuid::new_v4().simple());
        let artefacto = ArtefactoProcedural {
            id: id.clone(),
            nombre: nombre.to_string(),
            tipo,
            firma_ast: firma_ast.to_string(),
            hash_sha256: hash.clone(),
            contenido: contenido.to_string(),
            activo: true,
            version: 1,
            conceptos_clave: conceptos.iter().map(|s| s.to_string()).collect(),
            creado_en: chrono::Utc::now().to_rfc3339(),
        };

        // 3. Insertar en índices en memoria
        self.hash_index.insert(hash, id.clone());
        self.artefactos.insert(id.clone(), artefacto.clone());

        // 4. Persistir en DB si la ruta está configurada
        if let Err(e) = self.guardar_artefacto_db(&artefacto) {
            warn!("⚠️ [HIPPORAG-CATALOGO] No se pudo guardar artefacto en DB: {}", e);
        }

        info!(
            "📦 [HIPPORAG-CATALOGO] Nuevo artefacto procedimental registrado: '{}' (ID: {})",
            nombre, id
        );

        Ok((id, true))
    }

    /// Busca un artefacto existente por su hash SHA256.
    pub fn buscar_por_hash(&self, hash: &str) -> Option<&ArtefactoProcedural> {
        self.hash_index.get(hash).and_then(|id| self.artefactos.get(id))
    }

    /// Busca artefactos asociados a un concepto clave determinado.
    pub fn buscar_por_concepto(&self, concepto: &str) -> Vec<&ArtefactoProcedural> {
        self.artefactos
            .values()
            .filter(|a| a.activo && a.conceptos_clave.iter().any(|c| c.eq_ignore_ascii_case(concepto)))
            .collect()
    }

    /// Búsqueda relacional HippoRAG 2 basada en propagación de activación (Personalized PageRank).
    /// Calcula un puntaje relacional para cada artefacto activo ponderando la coincidencia
    /// con los conceptos semilla y sus enlaces asociativos.
    pub fn buscar_relacional_pagerank(
        &self,
        conceptos_semilla: &[&str],
        amortiguacion: f32,
    ) -> Vec<(&ArtefactoProcedural, f32)> {
        if conceptos_semilla.is_empty() {
            return Vec::new();
        }

        let mut resultados = Vec::new();

        for art in self.artefactos.values() {
            if !art.activo {
                continue;
            }

            let mut ppr_score = 0.0f32;
            for semilla in conceptos_semilla {
                // Coincidencia directa con conceptos clave
                if art.conceptos_clave.iter().any(|c| c.eq_ignore_ascii_case(semilla)) {
                    ppr_score += 1.0 * amortiguacion;
                }
                // Coincidencia parcial con el nombre o firma AST
                if art.nombre.to_lowercase().contains(&semilla.to_lowercase()) {
                    ppr_score += 0.5 * amortiguacion;
                }
            }

            if ppr_score > 0.0 {
                resultados.push((art, ppr_score));
            }
        }

        resultados.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        resultados
    }

    /// Desactiva una versión obsoleta cuando es reemplazada por un nuevo parche.
    pub fn invalidar_version_anterior(&mut self, id: &str) -> bool {
        if let Some(art) = self.artefactos.get_mut(id) {
            art.activo = false;
            info!("🗑️ [HIPPORAG-CATALOGO] Version obsoleta de artefacto declarada inactiva: {}", id);
            true
        } else {
            false
        }
    }

    // ─── Persistencia SQLite ────────────────────────────────────────────────

    fn inicializar_tabla(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS synapse_artefactos (
                id             TEXT PRIMARY KEY,
                nombre         TEXT NOT NULL,
                tipo           TEXT NOT NULL,
                firma_ast      TEXT NOT NULL,
                hash_sha256    TEXT NOT NULL UNIQUE,
                contenido      TEXT NOT NULL,
                activo         INTEGER NOT NULL DEFAULT 1,
                version        INTEGER NOT NULL DEFAULT 1,
                conceptos_clave TEXT NOT NULL DEFAULT '[]',
                creado_en      TEXT NOT NULL
            );",
        )
    }

    fn guardar_artefacto_db(&self, art: &ArtefactoProcedural) -> rusqlite::Result<()> {
        let db_path = match &self.db_path {
            Some(p) => p,
            None => return Ok(()),
        };
        let conn = rusqlite::Connection::open(db_path)?;
        Self::inicializar_tabla(&conn)?;

        let conceptos_json = serde_json::to_string(&art.conceptos_clave).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT OR REPLACE INTO synapse_artefactos 
             (id, nombre, tipo, firma_ast, hash_sha256, contenido, activo, version, conceptos_clave, creado_en)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                art.id,
                art.nombre,
                art.tipo.to_string(),
                art.firma_ast,
                art.hash_sha256,
                art.contenido,
                if art.activo { 1 } else { 0 },
                art.version,
                conceptos_json,
                art.creado_en,
            ],
        )?;

        Ok(())
    }

    pub fn cargar_desde_db(&mut self) -> rusqlite::Result<usize> {
        let db_path = match &self.db_path {
            Some(p) => p,
            None => return Ok(0),
        };
        if !db_path.exists() {
            return Ok(0);
        }

        let conn = rusqlite::Connection::open(db_path)?;
        Self::inicializar_tabla(&conn)?;

        let mut stmt = conn.prepare(
            "SELECT id, nombre, tipo, firma_ast, hash_sha256, contenido, activo, version, conceptos_clave, creado_en
             FROM synapse_artefactos",
        )?;

        let rows = stmt.query_map([], |row| {
            let tipo_str: String = row.get(2)?;
            let tipo = tipo_str.parse().unwrap_or(TipoArtefacto::ModuloRust);
            let conceptos_json: String = row.get(8)?;
            let conceptos: Vec<String> = serde_json::from_str(&conceptos_json).unwrap_or_default();

            Ok(ArtefactoProcedural {
                id: row.get(0)?,
                nombre: row.get(1)?,
                tipo,
                firma_ast: row.get(3)?,
                hash_sha256: row.get(4)?,
                contenido: row.get(5)?,
                activo: row.get::<_, i32>(6)? != 0,
                version: row.get(7)?,
                conceptos_clave: conceptos,
                creado_en: row.get(9)?,
            })
        })?;

        let mut cargados = 0;
        for r in rows {
            if let Ok(art) = r {
                self.hash_index.insert(art.hash_sha256.clone(), art.id.clone());
                self.artefactos.insert(art.id.clone(), art);
                cargados += 1;
            }
        }

        if cargados > 0 {
            info!("🧠 [HIPPORAG-CATALOGO] {} artefactos procedimentales restaurados desde DB", cargados);
        }

        Ok(cargados)
    }
}

// ============================================================================
// 🧪 PRUEBAS UNITARIAS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registrar_y_deduplicar_artefacto() {
        let mut cat = CatalogoHipocampico::new();
        let code = "pub fn suma(a: i32, b: i32) -> i32 { a + b }";

        // Registro inicial
        let (id1, es_nuevo1) = cat
            .registrar(
                "suma",
                TipoArtefacto::ModuloRust,
                "fn(i32, i32) -> i32",
                code,
                &["rust", "matematica"],
            )
            .unwrap();

        assert!(es_nuevo1);
        assert!(!id1.is_empty());

        // Registro duplicado (mismo contenido)
        let (id2, es_nuevo2) = cat
            .registrar(
                "suma_duplicada",
                TipoArtefacto::ModuloRust,
                "fn(i32, i32) -> i32",
                code,
                &["rust"],
            )
            .unwrap();

        assert!(!es_nuevo2);
        assert_eq!(id1, id2); // Debe reusar el id exactamente
    }

    #[test]
    fn test_buscar_por_concepto() {
        let mut cat = CatalogoHipocampico::new();
        cat.registrar(
            "patch_a",
            TipoArtefacto::ParcheCodigo,
            "fn()",
            "let a = 1;",
            &["optimizacion"],
        )
        .unwrap();

        let res = cat.buscar_por_concepto("optimizacion");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].nombre, "patch_a");
    }

    #[test]
    fn test_buscar_relacional_pagerank() {
        let mut cat = CatalogoHipocampico::new();
        cat.registrar(
            "modulo_red",
            TipoArtefacto::ModuloRust,
            "fn connect()",
            "pub fn connect() {}",
            &["red", "tcp"],
        )
        .unwrap();

        let ranking = cat.buscar_relacional_pagerank(&["red", "tcp"], 0.85);
        assert_eq!(ranking.len(), 1);
        assert!(ranking[0].1 > 1.0); // Debería acumular puntaje por múltiples coincidencias
    }
}
