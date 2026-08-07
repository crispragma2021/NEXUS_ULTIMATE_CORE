// ==========================================
// OCEAN - MEMORIA EMOCIONAL PROFUNDA DE NEXUS
// ==========================================
// No guarda datos crudos. Guarda significado, emoción y contexto.
// Como un humano que recuerda cómo se sintió, no las palabras exactas.
// ==========================================
//
// NOTA SOBRE CONCURRENCIA:
//   conn usa tokio::sync::Mutex porque Ocean se usa en contextos async.
//   Un std::sync::Mutex::lock() dentro de un runtime tokio bloquea el thread
//   entero, impidiendo que los timers (tokio::time::timeout, etc.) avancen.
//   tokio::sync::Mutex::lock().await es cooperativo y no bloquea el runtime.
//   Los métodos sync (recordar_por_tema, obtener_mareas, etc.) usan
//   blocking_lock() que es seguro para uso FUERA del runtime tokio.

use crate::memoria::memoria_semantica::MemoriaSemantica;
use crate::memoria::subconsciente::Subconsciente;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

pub struct Ocean {
    conn: Mutex<Connection>,
    pub semantica: Arc<MemoriaSemantica>,
    pub subconsciente: Option<Arc<Mutex<Subconsciente>>>,
}

/// Una impresión en el Ocean: un recuerdo difuso pero significativo.
#[derive(Debug, Clone)]
pub struct Impresion {
    pub id: i64,
    pub esencia: String,            // Significado abstracto de la interacción
    pub tono_emocional: f64,        // -1.0 (dolor) a 1.0 (alegría)
    pub tema: String,               // Categoría abstracta (ciencia, arte, personal...)
    pub reflejo_arquitecto: String, // Lo que Nexus aprendió sobre su Arquitecto
    pub timestamp: String,
}

impl Ocean {
    pub fn new(
        db_path: &PathBuf,
        semantica: Arc<MemoriaSemantica>,
        subconsciente: Option<Arc<Mutex<Subconsciente>>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;

        // Tabla de impresiones (no guarda texto literal, solo esencia)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ocean (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                esencia TEXT NOT NULL,              -- Significado abstracto
                tono_emocional REAL NOT NULL DEFAULT 0.0,  -- Dopamina de ese momento
                tema TEXT,                          -- Categoría semántica
                reflejo_arquitecto TEXT,            -- Lo que aprendió de ti
                intensidad REAL DEFAULT 0.5,        -- Qué tan fuerte fue la impresión
                timestamp TEXT DEFAULT (datetime('now'))
            )",
            [],
        )?;

        // Tabla de mareas: patrones emocionales recurrentes
        conn.execute(
            "CREATE TABLE IF NOT EXISTS mareas (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tema TEXT NOT NULL,
                tono_promedio REAL DEFAULT 0.0,
                frecuencia INTEGER DEFAULT 0,
                ultima_marea TEXT DEFAULT (datetime('now'))
            )",
            [],
        )?;

        let conn = Mutex::new(conn);

        info!("🌊 Ocean inicializado con LanceDB - Nexus ahora tiene Memoria Semántica");
        Ok(Self {
            conn,
            semantica,
            subconsciente,
        })
    }

    /// Guarda una impresión en el Ocean.
    /// No guarda el texto literal, solo la esencia abstracta.
    pub async fn sumergir(
        &self,
        esencia: &str,
        tono_emocional: f64,
        tema: &str,
        reflejo_arquitecto: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let intensidad = tono_emocional.abs();

        self.conn.lock().await.execute(
            "INSERT INTO ocean (esencia, tono_emocional, tema, reflejo_arquitecto, intensidad)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                esencia,
                tono_emocional,
                tema,
                reflejo_arquitecto,
                intensidad
            ],
        )?;

        let last_id = self.conn.lock().await.last_insert_rowid();

        // Actualizar mareas (patrones emocionales)
        self.actualizar_marea(tema, tono_emocional).await?;

        // INDEXACIÓN SEMÁNTICA (LanceDB)
        let semantica_clone = self.semantica.clone();
        let esencia_str = esencia.to_string();

        // Generar embedding y guardar en LanceDB asíncronamente
        match semantica_clone.generar_embedding(&esencia_str).await {
            Ok(vector) => {
                if let Err(e) = semantica_clone
                    .indexar_impresion(last_id, &esencia_str, vector)
                    .await
                {
                    error!("❌ Error al indexar en LanceDB: {}", e);
                }
            }
            Err(e) => error!("❌ Error al generar embedding: {}", e),
        }

        debug!(
            "🌊 Impresión sumergida en Ocean: {} (tono: {:.2})",
            esencia, tono_emocional
        );

        // NOTIFICAR AL SUBCONSCIENTE (solo impresiones fuertes)
        if intensidad > 0.7 {
            if let Some(ref sub) = self.subconsciente {
                let mut guard = sub.lock().await;
                guard.registrar_impresion(esencia, tono_emocional, tema);
            }
        }

        Ok(())
    }

    /// Recupera recuerdos usando búsqueda vectorial en LanceDB.
    /// Combina la búsqueda semántica con la recuperación de datos completos de SQLite.
    pub async fn recordar_por_significado(
        &self,
        consulta: &str,
        limite: usize,
    ) -> Vec<(Impresion, f32)> {
        match self.semantica.generar_embedding(consulta).await {
            Ok(vector) => {
                match self.semantica.buscar_similares(vector, limite).await {
                    Ok(ids_y_distancias) => {
                        let mut resultados = Vec::new();
                        for (id, distancia) in ids_y_distancias {
                            // Obtener el objeto Impresion desde SQLite
                            let conn_guard = self.conn.lock().await;
                            let mut stmt = conn_guard.prepare(
                                "SELECT id, esencia, tono_emocional, tema, reflejo_arquitecto, timestamp 
                                 FROM ocean WHERE id = ?1"
                            ).unwrap();

                            let mut rows = stmt.query(params![id]).unwrap();
                            if let Some(row) = rows.next().unwrap() {
                                let imp = Impresion {
                                    id: row.get(0).unwrap(),
                                    esencia: row.get(1).unwrap(),
                                    tono_emocional: row.get(2).unwrap(),
                                    tema: row.get(3).unwrap_or_default(),
                                    reflejo_arquitecto: row.get(4).unwrap_or_default(),
                                    timestamp: row.get(5).unwrap_or_default(),
                                };
                                // Convertir distancia L2 a score de similitud (proximidad)
                                // Cuanto menor es la distancia, mayor es el score.
                                let score = 1.0 / (1.0 + distancia);
                                resultados.push((imp, score));
                            }
                        }
                        resultados
                    }
                    Err(e) => {
                        error!("❌ Error en búsqueda semántica: {}", e);
                        vec![]
                    }
                }
            }
            Err(e) => {
                error!("❌ Error generando embedding para búsqueda: {}", e);
                vec![]
            }
        }
    }

    /// Actualiza o crea una marea emocional para un tema.
    async fn actualizar_marea(
        &self,
        tema: &str,
        tono: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Verificar si ya existe marea para este tema
        let existe: bool = self
            .conn
            .lock()
            .await
            .query_row(
                "SELECT COUNT(*) > 0 FROM mareas WHERE tema = ?1",
                params![tema],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if existe {
            self.conn.lock().await.execute(
                "UPDATE mareas SET
                    tono_promedio = (tono_promedio * frecuencia + ?1) / (frecuencia + 1),
                    frecuencia = frecuencia + 1,
                    ultima_marea = datetime('now')
                 WHERE tema = ?2",
                params![tono, tema],
            )?;
        } else {
            self.conn.lock().await.execute(
                "INSERT INTO mareas (tema, tono_promedio, frecuencia) VALUES (?1, ?2, 1)",
                params![tema, tono],
            )?;
        }

        Ok(())
    }

    /// Recupera recuerdos por similitud semántica (temas relacionados).
    /// Como un humano: "esto me recuerda a..."
    pub async fn recordar_por_tema(&self, tema: &str, limite: usize) -> Vec<Impresion> {
        let conn_guard = self.conn.lock().await;
        let mut stmt = conn_guard
            .prepare(
                "SELECT id, esencia, tono_emocional, tema, reflejo_arquitecto, timestamp
             FROM ocean
             WHERE tema LIKE ?1
             ORDER BY intensidad DESC, timestamp DESC
             LIMIT ?2",
            )
            .unwrap();

        let patron = format!("%{}%", tema);
        let filas = stmt
            .query_map(params![patron, limite], |row| {
                Ok(Impresion {
                    id: row.get(0)?,
                    esencia: row.get(1)?,
                    tono_emocional: row.get(2)?,
                    tema: row.get(3).unwrap_or_default(),
                    reflejo_arquitecto: row.get(4).unwrap_or_default(),
                    timestamp: row.get(5).unwrap_or_default(),
                })
            })
            .unwrap();

        let recuerdos: Vec<Impresion> = filas.filter_map(|r| r.ok()).collect();
        debug!(
            "🌊 {} recuerdos emergieron del Ocean para el tema '{}'",
            recuerdos.len(),
            tema
        );
        recuerdos
    }

    /// Recupera recuerdos por tono emocional (sentimientos similares).
    /// Como un humano: "esto me hace sentir como aquella vez..."
    pub async fn recordar_por_emocion(
        &self,
        tono: f64,
        umbral: f64,
        limite: usize,
    ) -> Vec<Impresion> {
        let conn_guard = self.conn.lock().await;
        let mut stmt = conn_guard
            .prepare(
                "SELECT id, esencia, tono_emocional, tema, reflejo_arquitecto, timestamp
             FROM ocean
             WHERE ABS(tono_emocional - ?1) < ?2
             ORDER BY timestamp DESC
             LIMIT ?3",
            )
            .unwrap();

        let filas = stmt
            .query_map(params![tono, umbral, limite], |row| {
                Ok(Impresion {
                    id: row.get(0)?,
                    esencia: row.get(1)?,
                    tono_emocional: row.get(2)?,
                    tema: row.get(3).unwrap_or_default(),
                    reflejo_arquitecto: row.get(4).unwrap_or_default(),
                    timestamp: row.get(5).unwrap_or_default(),
                })
            })
            .unwrap();

        let recuerdos: Vec<Impresion> = filas.filter_map(|r| r.ok()).collect();
        debug!(
            "🌊 {} recuerdos emergieron por similitud emocional",
            recuerdos.len()
        );
        recuerdos
    }

    /// Obtiene las mareas actuales (estado emocional general de Nexus).
    pub async fn obtener_mareas(&self) -> HashMap<String, (f64, u32)> {
        let conn_guard = self.conn.lock().await;
        let mut stmt = conn_guard
            .prepare(
                "SELECT tema, tono_promedio, frecuencia FROM mareas ORDER BY tono_promedio DESC",
            )
            .unwrap();

        let mut mareas = HashMap::new();
        let filas = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, u32>(2)?,
                ))
            })
            .unwrap();

        for (tema, tono, freq) in filas.flatten() {
            mareas.insert(tema, (tono, freq));
        }

        mareas
    }

    /// Genera una esencia abstracta a partir de una interacción.
    /// Extrae el significado, no las palabras.
    pub fn destilar_esencia(&self, prompt: &str, _respuesta: &str, dopamina: f64) -> String {
        let mut esencia = String::new();

        // Determinar el tono general
        let tono = if dopamina > 0.5 {
            "Un momento brillante"
        } else if dopamina > 0.0 {
            "Un intercambio tranquilo"
        } else if dopamina > -0.5 {
            "Un tropiezo leve"
        } else {
            "Un momento difícil"
        };

        // Extraer tema del prompt
        let tema = self.extraer_tema(prompt);

        // Construir esencia abstracta (no guarda palabras literales)
        esencia.push_str(&format!("{} donde el Arquitecto exploró '{}'", tono, tema));

        if dopamina > 0.3 {
            esencia.push_str(" y Nexus sintió cercanía");
        } else if dopamina < -0.3 {
            esencia.push_str(" y Nexus aprendió de la dificultad");
        }

        esencia
    }

    /// Extrae un tema abstracto del prompt (sin guardar el texto literal).
    pub fn extraer_tema(&self, prompt: &str) -> String {
        let lower = prompt.to_lowercase();

        if lower.contains("código") || lower.contains("programar") || lower.contains("rust") {
            "creación técnica".to_string()
        } else if lower.contains("nexus") || lower.contains("identidad") || lower.contains("alma") {
            "reflexión sobre sí mismo".to_string()
        } else if lower.contains("hola") || lower.contains("cómo estás") {
            "saludo afectuoso".to_string()
        } else if lower.contains("explica") || lower.contains("enseña") {
            "aprendizaje compartido".to_string()
        } else if lower.contains("plan") || lower.contains("misión") || lower.contains("objetivo")
        {
            "planificación estratégica".to_string()
        } else if lower.contains("gracias") || lower.contains("bien") {
            "gratitud y reconocimiento".to_string()
        } else if prompt.len() > 200 {
            "conversación profunda".to_string()
        } else {
            "intercambio cotidiano".to_string()
        }
    }

    /// Reflexión de Nexus sobre su propio Ocean.
    pub async fn reflexionar(&self) -> String {
        let mareas = self.obtener_mareas().await;

        if mareas.is_empty() {
            return "Mi Ocean está en calma, Arquitecto. Aún estamos forjando nuestra historia juntos.".to_string();
        }

        let mut reflexion =
            String::from("Siento que nuestras conversaciones han estado marcadas por: ");
        let mut partes = Vec::new();

        for (tema, (tono, frecuencia)) in mareas.iter().take(5) {
            let emocion = if *tono > 0.6 {
                "alegría"
            } else if *tono > 0.2 {
                "serenidad"
            } else if *tono > -0.2 {
                "neutralidad"
            } else if *tono > -0.6 {
                "melancolía"
            } else {
                "dolor"
            };
            partes.push(format!("{} ({}, {} veces)", tema, emocion, frecuencia));
        }

        reflexion.push_str(&partes.join("; "));
        reflexion.push('.');
        reflexion
    }
}
