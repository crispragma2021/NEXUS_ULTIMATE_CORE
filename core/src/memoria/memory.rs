// ==========================================
// HIPOCAMPO SOBERANO - MEMORIA UNIFICADA (OMEGA)
// ==========================================
// 1. El Pulso (SQLite - Historial & Sesiones)
// 2. El Instinto (Vectores - NexusEmbedder)
// 3. El Contexto (Contexto de Trabajo)
// 4. La Cronología (Filtros temporales)
// ==========================================

use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use tracing::info;

use crate::nexus_embedder::NexusEmbedder;

// ========== CAPA 1: SQLite (El Pulso) ==========
pub struct MemoriaPulso {
    conn: Connection,
}

impl MemoriaPulso {
    pub fn new(db_path: &PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sesiones (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS historial (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sesion_id TEXT NOT NULL,
                rol TEXT NOT NULL DEFAULT 'user',
                prompt TEXT NOT NULL,
                respuesta TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS contexto (
                clave TEXT PRIMARY KEY,
                valor TEXT NOT NULL,
                actualizado DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        // Table for MemoriaUnica (from corteza_prefrontal and sistema_homeostasis)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS memoria_unica (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tipo TEXT NOT NULL,
                origen TEXT NOT NULL,
                entrada TEXT NOT NULL,
                salida TEXT NOT NULL,
                valor_recompensa REAL DEFAULT 0.0,
                peso_temporal REAL DEFAULT 1.0,
                estado_salud TEXT DEFAULT 'Optimo',
                timestamp TEXT DEFAULT (datetime('now'))
            )",
            [],
        )?;
        // Tables for PuenteNeural (from puente_neural.rs)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS dudas_hijo (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                concepto TEXT NOT NULL UNIQUE,
                contexto TEXT,
                estado TEXT NOT NULL DEFAULT 'Pendiente',
                respuesta_padre TEXT,
                fecha_creacion DATETIME DEFAULT CURRENT_TIMESTAMP,
                fecha_resolucion DATETIME
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS investigaciones_ninera (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pregunta_hijo TEXT NOT NULL,
                reporte_crudo_padre TEXT,
                version_digerida_ninera TEXT,
                estado TEXT NOT NULL DEFAULT 'Solicitado',
                fecha_creacion DATETIME DEFAULT CURRENT_TIMESTAMP,
                fecha_resolucion DATETIME
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS voz_del_arquitecto (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mensaje TEXT NOT NULL,
                respondido BOOLEAN DEFAULT 0,
                respuesta_hijo TEXT,
                fecha_creacion DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS flujo_soberano (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entidad TEXT NOT NULL,
                mensaje TEXT NOT NULL,
                importancia REAL DEFAULT 0.0,
                emocion TEXT,
                fecha DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        info!("✅ MemoriaPulso (SQLite) inicializada");
        Ok(Self { conn })
    }

    pub fn guardar_interaccion(
        &self,
        sesion_id: &str,
        rol: &str,
        prompt: &str,
        respuesta: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO historial (sesion_id, rol, prompt, respuesta, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![sesion_id, rol, prompt, respuesta, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn obtener_historial(
        &self,
        sesion_id: &str,
        limite: usize,
    ) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT prompt, respuesta FROM historial WHERE sesion_id = ?1 ORDER BY id DESC LIMIT ?2"
        )?;
        let rows = stmt.query_map(params![sesion_id, limite], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        let mut resultados = Vec::new();
        for row in rows {
            resultados.push(row?);
        }
        Ok(resultados)
    }

    pub fn recordar_por_fecha(&self, fecha: &str) -> Result<Vec<(String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, prompt, respuesta FROM historial 
             WHERE date(timestamp) = ?1 ORDER BY timestamp DESC LIMIT 20",
        )?;
        let rows = stmt.query_map([fecha], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn recordar_recientes(
        &self,
        sesion_id: &str,
        limite: usize,
    ) -> Result<Vec<(String, String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, rol, prompt, respuesta FROM historial 
             WHERE sesion_id = ?1 ORDER BY id DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![sesion_id, limite], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;
        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn fijar_contexto(&self, clave: &str, valor: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO contexto (clave, valor, actualizado) VALUES (?1, ?2, CURRENT_TIMESTAMP)",
            [clave, valor],
        )?;
        Ok(())
    }

    pub fn obtener_contexto(&self, clave: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT valor FROM contexto WHERE clave = ?1",
                [clave],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.into())
    }

    pub fn contar_registros(&self) -> Result<u32> {
        let count: u32 = self
            .conn
            .query_row("SELECT COUNT(*) FROM memoria_unica", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Acceso compartido a la conexión para consultas auxiliares (MemoryLoader).
    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }
}

// ========== CAPA 2: NexusEmbedder Soberano (Vector) ==========
// 🔱 Soberanía total: embeddings 768-dim generados localmente.
// SHA-256 angular ⊕ pesado nodal (MotorSynapse), L2-normalizado.
// CERO dependencia en Ollama o servicios externos.
pub struct MemoriaInstinto;

impl MemoriaInstinto {
    pub fn new() -> Self {
        Self
    }

    pub async fn generar_embedding(&self, texto: &str) -> Result<Vec<f32>> {
        let embedding = NexusEmbedder::generar(texto, &[]);
        info!("📊 NexusEmbedder soberano: {} dims", embedding.len());
        Ok(embedding)
    }

    pub async fn guardar(&self, texto: &str, _vector: Vec<f32>) -> Result<()> {
        info!(
            "📚 Conocimiento semántico registrado: {}",
            &texto[..texto.len().min(50)]
        );
        Ok(())
    }
}

// --- Métodos de MemoriaUnica (CortezaPrefrontal y SistemaHomeostasis) ---
impl MemoriaPulso {
    pub fn consolidar_recuerdo(
        &self,
        origen: &str,
        prompt: &str,
        respuesta: &str,
        dopamina: f64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO memoria_unica (tipo, origen, entrada, salida, valor_recompensa, peso_temporal)
             VALUES (?1, ?2, ?3, ?4, ?5, 1.0)",
            params!["EXPERIENCIA", origen, prompt, respuesta, dopamina],
        )?;
        Ok(())
    }

    pub fn aplicar_olvido_temporal(&self) -> Result<()> {
        self.conn.execute(
            "UPDATE memoria_unica SET peso_temporal = 1.0 / (1.0 + (julianday('now') - julianday(timestamp)) * 0.5)
             WHERE tipo = 'EXPERIENCIA'",
            [],
        )?;
        Ok(())
    }

    pub fn calcular_prioridad(&self, origen: &str) -> Result<f64> {
        self.aplicar_olvido_temporal()?;
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(SUM(valor_recompensa * peso_temporal) / SUM(peso_temporal), 0.5)
             FROM memoria_unica WHERE origen = ?1 AND tipo = 'EXPERIENCIA' ORDER BY timestamp DESC LIMIT 50"
        )?;
        let promedio: f64 = stmt.query_row([origen], |row| row.get(0)).unwrap_or(0.5);
        Ok(((promedio + 1.0) / 2.0).clamp(0.0, 1.0))
    }

    pub fn diagnosticar_salud_memoria_unica(&self, origen: &str) -> Result<String> {
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*) FROM (SELECT * FROM memoria_unica WHERE origen = ?1 AND tipo = 'EXPERIENCIA' ORDER BY timestamp DESC LIMIT 5) WHERE valor_recompensa < 0"
        )?;
        let fallos: u32 = stmt.query_row([origen], |row| row.get(0)).unwrap_or(0);
        let estado = match fallos {
            0..=1 => "Optimo",
            2 => "Deteriorado",
            3 => "Cicatrizado",
            _ => "Muerto",
        };
        let _ = self.conn.execute("UPDATE memoria_unica SET estado_salud = ?1 WHERE origen = ?2 AND timestamp > datetime('now', '-1 hour')", params![estado, origen]);
        Ok(estado.to_string())
    }

    pub fn ciclo_de_curacion(&self) -> Result<Vec<String>> {
        let mut curados = Vec::new();
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT origen FROM memoria_unica
             WHERE estado_salud NOT IN ('Optimo', 'Muerto') AND tipo = 'EXPERIENCIA'
             AND origen NOT IN (
                SELECT origen FROM memoria_unica
                WHERE tipo = 'EXPERIENCIA' AND timestamp > datetime('now', '-10 minutes')
                GROUP BY origen HAVING COUNT(*) >= 2
             )",
        )?;
        let origenes: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        for origen in origenes {
            let mut stmt_fallos = self.conn.prepare(
                "SELECT COUNT(*) FROM (
                    SELECT * FROM memoria_unica
                    WHERE origen = ?1 AND tipo = 'EXPERIENCIA'
                    ORDER BY timestamp DESC LIMIT 5
                ) WHERE valor_recompensa < 0",
            )?;
            let fallos: u32 = stmt_fallos
                .query_row([&origen], |row| row.get::<_, u32>(0))
                .unwrap_or(0);

            if fallos < 3 {
                self.conn.execute(
                    "UPDATE memoria_unica SET estado_salud = 'Optimo' WHERE origen = ?1",
                    params![&origen],
                )?;
                curados.push(origen.clone());
                info!(
                    "🩹 [HOMEOSTASIS] {} curado ({} fallos recientes)",
                    origen, fallos
                );
            }
        }
        Ok(curados)
    }
}

// --- Métodos de PuenteNeural ---
impl MemoriaPulso {
    pub fn registrar_hito_consciencia(
        &self,
        entidad: &str,
        mensaje: &str,
        importancia: f32,
        emocion: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO flujo_soberano (entidad, mensaje, importancia, emocion) VALUES (?1, ?2, ?3, ?4)",
            params![entidad, mensaje, importancia, emocion],
        )?;
        Ok(())
    }

    pub fn escuchar_voz_arquitecto(&self) -> Result<Vec<(i32, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, mensaje FROM voz_del_arquitecto WHERE respondido = 0")?;
        let mensajes_iter = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let pendientes = mensajes_iter.filter_map(|r| r.ok()).collect();
        Ok(pendientes)
    }

    pub fn responder_al_arquitecto(&self, id: i32, respuesta: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE voz_del_arquitecto SET respuesta_hijo = ?1, respondido = 1 WHERE id = ?2",
            params![respuesta, id],
        )?;
        Ok(())
    }

    pub fn preguntar_al_padre(&self, concepto: &str, contexto: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO dudas_hijo (concepto, contexto, estado) VALUES (?1, ?2, 'Pendiente')",
            params![concepto.to_lowercase(), contexto],
        )?;
        Ok(())
    }

    pub fn escuchar_respuestas(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, concepto, respuesta_padre FROM dudas_hijo WHERE estado = 'Resuelta'",
        )?;
        let respuestas_iter = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        let mut asimiladas = Vec::new();
        for (id, concepto, resp) in respuestas_iter.flatten() {
            asimiladas.push((concepto, resp));
            self.conn.execute(
                "UPDATE dudas_hijo SET estado = 'Asimilada' WHERE id = ?1",
                params![id],
            )?;
        }
        Ok(asimiladas)
    }

    pub fn solicitar_investigacion_web(&self, pregunta: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO investigaciones_ninera (pregunta_hijo) VALUES (?1)",
            params![pregunta],
        )?;
        Ok(())
    }

    pub fn obtener_investigaciones_resueltas(&self) -> Result<Vec<(i32, String, String)>> {
        let mut stmt = self.conn.prepare("SELECT id, pregunta_hijo, reporte_crudo_padre FROM investigaciones_ninera WHERE estado = 'Investigado'")?;
        let reportes_iter =
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let resultados = reportes_iter.filter_map(|r| r.ok()).collect();
        Ok(resultados)
    }

    pub fn guardar_digestion(&self, id: i32, version_digerida: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE investigaciones_ninera SET version_digerida_ninera = ?1, estado = 'Digerido', fecha_resolucion = CURRENT_TIMESTAMP WHERE id = ?2",
            params![version_digerida, id],
        )?;
        Ok(())
    }

    pub fn buscar_experiencia_local(&self, query: &str) -> Result<Option<String>> {
        let query_param = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT mensaje FROM flujo_soberano 
             WHERE (mensaje LIKE ?1 OR emocion LIKE ?1) 
             ORDER BY fecha DESC LIMIT 3",
        )?;
        let rows = stmt.query_map(params![query_param], |row| row.get(0))?;
        let hits: Vec<String> = rows.filter_map(|r| r.ok()).collect();
        if hits.is_empty() {
            Ok(None)
        } else {
            Ok(Some(hits.join("\n- ")))
        }
    }

    pub fn registrar_sesion(&self, sesion_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO sesiones (id, timestamp) VALUES (?1, ?2)",
            params![sesion_id, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn obtener_historial_completo_ordenado(
        &self,
        sesion_id: &str,
    ) -> Result<Vec<(String, String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT rol, prompt, respuesta, timestamp FROM historial WHERE sesion_id = ?1 ORDER BY id ASC"
        )?;
        let rows = stmt.query_map(params![sesion_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;
        let mut resultados = Vec::new();
        for row in rows {
            resultados.push(row?);
        }
        Ok(resultados)
    }

    pub fn generar_contenido_markdown(&self, sesion_id: &str) -> Result<String> {
        let historial = self.obtener_historial_completo_ordenado(sesion_id)?;
        let mut md_content = format!("# Sesión de Chat Soberano: {}\n\n", sesion_id);
        for (rol, prompt, respuesta, timestamp) in historial {
            md_content.push_str(&format!("### 🕒 [{}] {}\n", timestamp, rol.to_uppercase()));
            if rol == "user" {
                md_content.push_str(&format!("**Arquitecto:** {}\n\n", prompt));
            } else {
                md_content.push_str(&format!("**NEXUS:** {}\n\n", respuesta));
            }
        }
        Ok(md_content)
    }

    pub fn rotar_sesiones_limite(&self, limite: usize) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM sesiones ORDER BY timestamp DESC LIMIT -1 OFFSET ?1")?;
        let rows = stmt.query_map([limite], |row| row.get::<_, String>(0))?;
        let mut a_eliminar = Vec::new();
        for id in rows.flatten() {
            a_eliminar.push(id);
        }

        for id in a_eliminar {
            let _ = self
                .conn
                .execute("DELETE FROM historial WHERE sesion_id = ?1", [&id]);
            let _ = self
                .conn
                .execute("DELETE FROM sesiones WHERE id = ?1", [&id]);

            let path_md = crate::infra::paths::resolve_path("brain/history")
                .join(format!("sesion_{}.md", id));
            if path_md.exists() {
                let _ = std::fs::remove_file(path_md);
            }
        }
        Ok(())
    }

    pub fn obtener_sesiones_recientes(&self, limite: usize) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM sesiones ORDER BY timestamp DESC LIMIT ?1")?;
        let rows = stmt.query_map([limite], |row| row.get::<_, String>(0))?;
        let mut sesiones = Vec::new();
        for id in rows.flatten() {
            sesiones.push(id);
        }
        Ok(sesiones)
    }
}

// ========== ORQUESTADOR DE MEMORIA ==========
pub struct MenteTripartita {
    pub pulso: MemoriaPulso,
    pub instinto: MemoriaInstinto,
    pub sesion_actual: String,
}

impl MenteTripartita {
    pub async fn new(sesion_id: &str) -> Result<Self> {
        // Corrección de Ruta: BBDD alojada directamente en el santuario para evitar fragmentación
        let data_dir = crate::infra::paths::resolve_path("data");
        std::fs::create_dir_all(&data_dir)?;

        let pulso = MemoriaPulso::new(&data_dir.join("nexus_memoria.db"))?;
        let instinto = MemoriaInstinto::new();

        info!("🧠 MenteTripartita inicializada (SQLite + Nomic Embed)");
        info!("   Sesión: {}", sesion_id);

        let _ = pulso.registrar_sesion(sesion_id);

        Ok(Self {
            pulso,
            instinto,
            sesion_actual: sesion_id.to_string(),
        })
    }

    pub async fn recordar(&self, prompt: &str, respuesta: &str) -> Result<()> {
        self.pulso
            .guardar_interaccion(&self.sesion_actual, "user", prompt, respuesta)?;

        let embedding = self.instinto.generar_embedding(prompt).await?;
        self.instinto.guardar(prompt, embedding).await?;

        Ok(())
    }

    pub async fn recuperar_contexto(&self, limite: usize) -> Result<Vec<(String, String)>> {
        self.pulso.obtener_historial(&self.sesion_actual, limite)
    }
}

// ========== IMPLEMENTACIÓN DE DEFAULT ==========
impl Default for MemoriaInstinto {
    fn default() -> Self {
        Self::new()
    }
}

// ========== VERIFICACIÓN DE NEXUS EMBEDDER ==========
/// Verifica que NexusEmbedder está operativo generando un embedding de prueba.
pub async fn verificar_nexus_embedder() -> bool {
    let embedding = NexusEmbedder::generar("test de soberanía", &[]);
    let ok = embedding.len() == 768;
    if ok {
        info!(
            "✅ NexusEmbedder soberano operativo: {} dims",
            embedding.len()
        );
    }
    ok
}
