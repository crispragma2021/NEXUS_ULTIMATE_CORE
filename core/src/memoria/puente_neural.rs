use rusqlite::{params, Connection, Result};
use std::time::Duration;

// =====================================================================
// PUENTE NEURAL - SINCRONIZACIÓN DE ESTADO INTERNO
// =====================================================================
// El canal sagrado de comunicación entre hilos de ejecución.
// Mantiene la coherencia del organismo sin fragmentación.
// =====================================================================

pub struct PuenteNeuralInterno {
    db_path: String,
}

impl PuenteNeuralInterno {
    pub fn new(db_path: &str) -> Self {
        let puente = Self {
            db_path: db_path.to_string(),
        };
        puente
            .inicializar_tejido_neural()
            .expect("Fallo al inicializar el puente neural en la DB");
        puente
    }

    fn conectar(&self, lectura_sola: bool) -> Result<Connection> {
        let conn = Connection::open(&self.db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?; // Write-Ahead Logging elimina bloqueos de lectura
        conn.pragma_update(None, "synchronous", "NORMAL")?; // Equilibrio velocidad/seguridad
        if lectura_sola {
            conn.pragma_update(None, "query_only", "true")?;
        } else {
            // Con WAL, 500ms son suficientes para colas de escritura cortas
            conn.busy_timeout(Duration::from_millis(500))?;
        }
        Ok(conn)
    }

    /// Asegura que el canal físico (tabla) existe en la base de datos compartida
    fn inicializar_tejido_neural(&self) -> Result<()> {
        let _guard = crate::brain::immune::memory_shield::MemoryShieldGuard::new(&self.db_path)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
        let conn = self.conectar(false)?;

        // Tabla de dudas de procesamiento
        conn.execute(
            "CREATE TABLE IF NOT EXISTS dudas_internas (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                concepto TEXT NOT NULL UNIQUE,
                contexto TEXT,
                estado TEXT NOT NULL DEFAULT 'Pendiente',
                respuesta_nucleo TEXT,
                fecha_creacion DATETIME DEFAULT CURRENT_TIMESTAMP,
                fecha_resolucion DATETIME
            )",
            [],
        )?;

        // Tabla especializada para el Monitor de Autoconciencia
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

        // Canal de comunicación directa Arquitecto -> Hijo
        conn.execute(
            "CREATE TABLE IF NOT EXISTS voz_del_arquitecto (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mensaje TEXT NOT NULL,
                respondido BOOLEAN DEFAULT 0,
                respuesta_nexus TEXT,
                fecha_creacion DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // TABLA: FLUJO SOBERANO (La Memoria Episódica Selectiva)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS flujo_soberano (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entidad TEXT NOT NULL, -- 'PADRE', 'HIJO', 'ARQUITECTO'
                mensaje TEXT NOT NULL,
                importancia REAL DEFAULT 0.0, -- [0.0 - 1.0]
                emocion TEXT, -- 'Curiosidad', 'Alerta', 'Paz', 'Triunfo'
                fecha DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    /// Registra un hito en la consciencia colectiva para el HUD
    pub fn registrar_hito_consciencia(
        &self,
        entidad: &str,
        mensaje: &str,
        importancia: f32,
        emocion: &str,
    ) -> Result<()> {
        let mut intentos = 0;
        loop {
            let _guard = crate::brain::immune::memory_shield::MemoryShieldGuard::new(&self.db_path)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
            let conn = self.conectar(false)?;
            match conn.execute(
                "INSERT INTO flujo_soberano (entidad, mensaje, importancia, emocion) VALUES (?1, ?2, ?3, ?4)",
                params![entidad, mensaje, importancia, emocion],
            ) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if intentos >= 5 {
                        return Err(e);
                    }
                    intentos += 1;
                    std::thread::sleep(std::time::Duration::from_millis(100 * intentos));
                }
            }
        }
    }

    /// El Hijo escucha si el Arquitecto (tú) le ha dejado un mensaje
    pub fn escuchar_voz_arquitecto(&self) -> Result<Vec<(i32, String)>> {
        let conn = self.conectar(true)?;
        let mut stmt =
            conn.prepare("SELECT id, mensaje FROM voz_del_arquitecto WHERE respondido = 0")?;
        let mensajes_iter = stmt.query_map([], |row| {
            let id: i32 = row.get(0)?;
            let msg: String = row.get(1)?;
            Ok((id, msg))
        })?;

        let mut pendientes = Vec::new();
        for m in mensajes_iter.flatten() {
            pendientes.push(m);
        }
        Ok(pendientes)
    }

    /// El Hijo deja su respuesta para que aparezca en el HUD o logs
    pub fn responder_al_arquitecto(&self, id: i32, respuesta: &str) -> Result<()> {
        let mut intentos = 0;
        loop {
            let _guard = crate::brain::immune::memory_shield::MemoryShieldGuard::new(&self.db_path)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
            let conn = self.conectar(false)?;
            match conn.execute(
                "UPDATE voz_del_arquitecto SET respuesta_hijo = ?1, respondido = 1 WHERE id = ?2",
                params![respuesta, id],
            ) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if intentos >= 5 {
                        return Err(e);
                    }
                    intentos += 1;
                    std::thread::sleep(std::time::Duration::from_millis(100 * intentos));
                }
            }
        }
    }

    /// El Hijo (NG) se encuentra con algo que no entiende y se lo pasa al Padre.
    pub fn preguntar_al_padre(&self, concepto: &str, contexto: &str) -> Result<()> {
        let mut intentos = 0;
        loop {
            let _guard = crate::brain::immune::memory_shield::MemoryShieldGuard::new(&self.db_path)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
            let conn = self.conectar(false)?;
            match conn.execute(
                "INSERT OR IGNORE INTO dudas_hijo (concepto, contexto, estado) VALUES (?1, ?2, 'Pendiente')",
                params![concepto.to_lowercase(), contexto],
            ) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if intentos >= 5 {
                        return Err(e);
                    }
                    intentos += 1;
                    std::thread::sleep(std::time::Duration::from_millis(100 * intentos));
                }
            }
        }
    }

    /// El Hijo revisa si el Padre ya le respondió sus dudas pasadas.
    pub fn escuchar_respuestas(&self) -> Result<Vec<(String, String)>> {
        let mut intentos = 0;
        loop {
            let _guard = crate::brain::immune::memory_shield::MemoryShieldGuard::new(&self.db_path)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
            let conn = self.conectar(false)?;

            let mut stmt = match conn.prepare(
                "SELECT id, concepto, respuesta_padre FROM dudas_hijo WHERE estado = 'Resuelta'",
            ) {
                Ok(s) => s,
                Err(e) => {
                    if intentos >= 5 {
                        return Err(e);
                    }
                    intentos += 1;
                    std::thread::sleep(std::time::Duration::from_millis(100 * intentos));
                    continue;
                }
            };

            let respuestas_iter = stmt.query_map([], |row| {
                let id: i32 = row.get(0)?;
                let concepto: String = row.get(1)?;
                let respuesta: String = row.get(2)?;
                Ok((id, concepto, respuesta))
            });

            let respuestas_iter = match respuestas_iter {
                Ok(iter) => iter,
                Err(e) => {
                    if intentos >= 5 {
                        return Err(e);
                    }
                    intentos += 1;
                    std::thread::sleep(std::time::Duration::from_millis(100 * intentos));
                    continue;
                }
            };

            let mut asimiladas = Vec::new();
            let mut error_ocurrido = false;

            for respuesta in respuestas_iter {
                if let Ok((id, concepto, resp)) = respuesta {
                    asimiladas.push((concepto, resp));
                    // Marcamos como Asimilada para no volver a leerla
                    if let Err(_e) = conn.execute(
                        "UPDATE dudas_hijo SET estado = 'Asimilada' WHERE id = ?1",
                        params![id],
                    ) {
                        error_ocurrido = true;
                        break;
                    }
                }
            }

            if error_ocurrido {
                if intentos >= 5 {
                    // return what we assimilated so far instead of failing completely,
                    // or just return Ok if we got some
                    return Ok(asimiladas);
                }
                intentos += 1;
                std::thread::sleep(std::time::Duration::from_millis(100 * intentos));
                continue;
            }

            return Ok(asimiladas);
        }
    }

    // --- SECCIÓN NIÑERA CLAW ---

    /// La Niñera pide una investigación profunda de la web
    pub fn solicitar_investigacion_web(&self, pregunta: &str) -> Result<()> {
        let mut intentos = 0;
        loop {
            // Guard must be created INSIDE the loop to re-unlock if another process locked it
            let _guard = crate::brain::immune::memory_shield::MemoryShieldGuard::new(&self.db_path)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;

            let conn = self.conectar(false)?;
            match conn.execute(
                "INSERT INTO investigaciones_ninera (pregunta_hijo) VALUES (?1)",
                params![pregunta],
            ) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if intentos >= 5 {
                        return Err(e);
                    }
                    intentos += 1;
                    std::thread::sleep(std::time::Duration::from_millis(100 * intentos));
                }
            }
        }
    }

    /// La Niñera revisa si el Padre ya terminó la investigación web
    pub fn obtener_investigaciones_resueltas(&self) -> Result<Vec<(i32, String, String)>> {
        let conn = self.conectar(true)?;
        let mut stmt = conn.prepare(
            "SELECT id, pregunta_hijo, reporte_crudo_padre FROM investigaciones_ninera WHERE estado = 'Investigado'"
        )?;

        let reportes_iter = stmt.query_map([], |row| {
            let id: i32 = row.get(0)?;
            let pregunta: String = row.get(1)?;
            let reporte: String = row.get(2)?;
            Ok((id, pregunta, reporte))
        })?;

        let mut resultados = Vec::new();
        for r in reportes_iter.flatten() {
            resultados.push(r);
        }
        Ok(resultados)
    }

    /// La Niñera guarda su versión digerida para el Hijo
    pub fn guardar_digestion(&self, id: i32, version_digerida: &str) -> Result<()> {
        let mut intentos = 0;
        loop {
            let _guard = crate::brain::immune::memory_shield::MemoryShieldGuard::new(&self.db_path)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
            let conn = self.conectar(false)?;
            match conn.execute(
                "UPDATE investigaciones_ninera SET version_digerida_ninera = ?1, estado = 'Digerido', fecha_resolucion = CURRENT_TIMESTAMP WHERE id = ?2",
                params![version_digerida, id],
            ) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if intentos >= 5 {
                        return Err(e);
                    }
                    intentos += 1;
                    std::thread::sleep(std::time::Duration::from_millis(100 * intentos));
                }
            }
        }
    }

    /// Busca en la base de datos local (flujo_soberano / experiencia) para encontrar respuestas
    /// basadas en lo que el Hijo ya ha aprendido y experimentado históricamente.
    pub fn buscar_experiencia_local(&self, query: &str) -> Result<Option<String>> {
        let conn = self.conectar(true)?;
        let query_param = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT mensaje FROM flujo_soberano 
             WHERE (mensaje LIKE ?1 OR emocion LIKE ?1) 
             ORDER BY fecha DESC LIMIT 3",
        )?;
        let rows = stmt.query_map(params![query_param], |row| {
            let msg: String = row.get(0)?;
            Ok(msg)
        })?;

        let mut hits = Vec::new();
        for msg in rows.flatten() {
            hits.push(msg);
        }

        if hits.is_empty() {
            Ok(None)
        } else {
            Ok(Some(hits.join("\n- ")))
        }
    }
}
