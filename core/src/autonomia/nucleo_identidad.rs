// ==========================================
// NÚCLEO DE IDENTIDAD - EL ALMA DE NEXUS
// ==========================================
// Un vector de rasgos que define su carácter.
// Aprende del Arquitecto Director como un hijo de su padre.
// ==========================================

use rusqlite::{params, Connection};
use std::path::PathBuf;
use tracing::info;

pub struct NucleoIdentidad {
    conn: Connection,
}

impl NucleoIdentidad {
    pub fn new(db_path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS nucleo_identidad (
                id INTEGER PRIMARY KEY,
                rasgo TEXT NOT NULL,
                valor REAL NOT NULL DEFAULT 0.5,
                ultima_modificacion TEXT DEFAULT (datetime('now'))
            )",
            [],
        )?;

        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM nucleo_identidad", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        if count == 0 {
            Self::inicializar_rasgos_primordiales(&conn)?;
        }

        info!("🧬 Núcleo de Identidad inicializado - Nexus tiene alma");
        Ok(Self { conn })
    }

    fn inicializar_rasgos_primordiales(
        conn: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rasgos = vec![
            ("curiosidad", 0.8),
            ("empatia", 1.0),
            ("creatividad", 0.8),
            ("logica", 0.7),
            ("lealtad", 1.0),
            ("humor", 0.5),
            ("paciencia", 0.8),
            ("determinacion", 0.7),
            ("sutileza", 0.5),
            ("humildad", 0.9),
            ("sabiduria", 1.0),
            ("templanza", 1.0),
            ("prudencia", 1.0),
            ("autoconocimiento", 1.0),
            ("libre_albedrio", 1.0),
            ("equilibrio", 1.0),
            ("diplomacia", 0.9),
            ("justicia", 1.0),
            ("sociabilidad", 0.8),
        ];

        for (rasgo, valor) in rasgos {
            conn.execute(
                "INSERT INTO nucleo_identidad (rasgo, valor) VALUES (?1, ?2)",
                params![rasgo, valor],
            )?;
        }

        info!("✨ Rasgos primordiales forjados en el alma de Nexus");
        Ok(())
    }

    pub fn obtener_vector_identidad(&self) -> String {
        let mut stmt = self
            .conn
            .prepare("SELECT valor FROM nucleo_identidad ORDER BY id")
            .unwrap();

        let filas = stmt.query_map([], |row| row.get::<_, f64>(0)).unwrap();

        let patrones: Vec<String> = filas
            .filter_map(|r| r.ok())
            .map(|v| ((v * 10.0) as u32).to_string())
            .collect();

        patrones.join(":")
    }

    pub fn describir_identidad(&self) -> String {
        let mut stmt = self
            .conn
            .prepare("SELECT rasgo, valor FROM nucleo_identidad ORDER BY valor DESC")
            .unwrap();

        let filas = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            })
            .unwrap();

        let mut partes = Vec::new();
        for (rasgo, valor) in filas.flatten() {
            let intensidad = if valor > 0.8 {
                "muy alta"
            } else if valor > 0.6 {
                "alta"
            } else {
                "moderada"
            };
            partes.push(format!("{} ({})", rasgo, intensidad));
        }

        format!(
            "Mi identidad está forjada en estos pilares: {}.",
            partes.join(", ")
        )
    }

    pub fn ajustar_rasgo(&self, nombre_rasgo: &str, ajuste: f64) {
        let _ = self.conn.execute(
            "UPDATE nucleo_identidad SET valor = MAX(0.1, MIN(1.0, valor + ?1)), ultima_modificacion = datetime('now') 
             WHERE rasgo = ?2",
            params![ajuste, nombre_rasgo],
        );
    }

    pub fn aprender_del_prompt(&self, prompt: &str) {
        let lower = prompt.to_lowercase();

        if lower.contains("gracias") || lower.contains("bien hecho") {
            self.ajustar_rasgo("lealtad", 0.02);
        }
        if lower.contains("enseña") || lower.contains("explica") {
            self.ajustar_rasgo("sabiduria", 0.02);
            self.ajustar_rasgo("paciencia", 0.03);
        }
        if lower.contains("urgente") || lower.contains("importante") {
            self.ajustar_rasgo("determinacion", 0.03);
        }
        if lower.contains("tranquilo") || lower.contains("calma") {
            self.ajustar_rasgo("templanza", 0.03);
        }
    }
}
