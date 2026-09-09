// ============================================================================
// 🔒 PORT REGISTRY — Registro Permanente de Puertos (Regla 2)
// ============================================================================
// INMUTABILIDAD: ningún agente/IA puede asignar puertos aleatorios ni
// sobreescribir puertos existentes.
//
// - Registro persistente (SQLite) administrado EXCLUSIVAMENTE por Rust.
// - Al crear un proyecto, se le asigna el siguiente puerto libre en el rango
//   reservado (8000-8999) y se BLOQUEA PERMANENTEMENTE.
// - Al arrancar un servicio, el agente LEE el puerto asignado desde el registro.
// - Si el puerto está ocupado por un proceso colgado, el orquestador debe
//   MATAR ese proceso, NUNCA cambiarle el puerto al proyecto.
// ============================================================================

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

/// Rango reservado de puertos de proyecto.
pub const PORT_RANGE_START: u16 = 8000;
pub const PORT_RANGE_END: u16 = 8999;

/// Registro persistente de puertos.
pub struct PortRegistry {
    conn: Mutex<Connection>,
}

impl PortRegistry {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let r = Self {
            conn: Mutex::new(conn),
        };
        r.init()?;
        Ok(r)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let r = Self {
            conn: Mutex::new(conn),
        };
        r.init()?;
        Ok(r)
    }

    fn init(&self) -> Result<()> {
        self.conn.lock().unwrap().execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS port_assignments (
                project_id TEXT PRIMARY KEY,
                port       INTEGER NOT NULL UNIQUE,
                assigned_at TEXT NOT NULL DEFAULT (datetime('now')),
                locked     INTEGER NOT NULL DEFAULT 1
            );
            "#,
        )?;
        Ok(())
    }

    /// Asigna el siguiente puerto libre del rango reservado a un proyecto.
    ///
    /// El puerto queda BLOQUEADO PERMANENTEMENTE (locked=1). Si el proyecto ya
    /// tenía puerto, devuelve el mismo (inmutabilidad: nunca se reasigna).
    pub fn assign_port(&self, project_id: &str) -> Result<u16> {
        let conn = self.conn.lock().unwrap();

        // 1. ¿Ya tiene puerto? → devolverlo (INMUTABLE).
        if let Ok(port) = conn.query_row(
            "SELECT port FROM port_assignments WHERE project_id = ?1",
            rusqlite::params![project_id],
            |r| r.get::<_, i64>(0),
        ) {
            return Ok(port as u16);
        }

        // 2. Buscar el siguiente puerto libre.
        let used: Vec<i64> = {
            let mut stmt = conn.prepare("SELECT port FROM port_assignments")?;
            let rows = stmt.query_map([], |r| r.get::<_, i64>(0))?;
            let mut v = Vec::new();
            for r in rows {
                v.push(r?);
            }
            v
        };

        let next = (PORT_RANGE_START..=PORT_RANGE_END)
            .find(|p| !used.contains(&(*p as i64)))
            .context("rango de puertos agotado (8000-8999)")?;

        // 3. Bloquear permanentemente.
        conn.execute(
            "INSERT INTO port_assignments (project_id, port, locked) VALUES (?1, ?2, 1)",
            rusqlite::params![project_id, next],
        )?;
        Ok(next)
    }

    /// Lee el puerto asignado a un proyecto (para arrancar su servicio).
    ///
    /// NO reasigna: si el proyecto no tiene puerto, error.
    pub fn get_port(&self, project_id: &str) -> Result<u16> {
        let conn = self.conn.lock().unwrap();
        let port: i64 = conn
            .query_row(
                "SELECT port FROM port_assignments WHERE project_id = ?1",
                rusqlite::params![project_id],
                |r| r.get(0),
            )
            .context("proyecto sin puerto asignado")?;
        Ok(port as u16)
    }

    /// Devuelve los proyectos con su puerto (para la UI).
    pub fn list_assignments(&self) -> Result<Vec<(String, u16)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT project_id, port FROM port_assignments ORDER BY port")?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u16))
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// ¿El puerto está en el rango reservado?
    pub fn is_reserved(port: u16) -> bool {
        (PORT_RANGE_START..=PORT_RANGE_END).contains(&port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asigna_puertos_incrementales() {
        let r = PortRegistry::open_in_memory().unwrap();
        let p1 = r.assign_port("a").unwrap();
        let p2 = r.assign_port("b").unwrap();
        assert_eq!(p1, PORT_RANGE_START);
        assert_eq!(p2, PORT_RANGE_START + 1);
    }

    #[test]
    fn puerto_es_inmutable_para_mismo_proyecto() {
        let r = PortRegistry::open_in_memory().unwrap();
        let p1 = r.assign_port("a").unwrap();
        let p2 = r.assign_port("a").unwrap();
        assert_eq!(p1, p2); // nunca se reasigna
    }

    #[test]
    fn get_port_despues_de_asignar() {
        let r = PortRegistry::open_in_memory().unwrap();
        r.assign_port("trader").unwrap();
        assert_eq!(r.get_port("trader").unwrap(), PORT_RANGE_START);
    }

    #[test]
    fn get_port_sin_asignar_error() {
        let r = PortRegistry::open_in_memory().unwrap();
        assert!(r.get_port("nope").is_err());
    }

    #[test]
    fn rango_reservado_correcto() {
        assert!(PortRegistry::is_reserved(8000));
        assert!(PortRegistry::is_reserved(8999));
        assert!(!PortRegistry::is_reserved(7000));
        assert!(!PortRegistry::is_reserved(9000));
    }

    #[test]
    fn lista_asignaciones() {
        let r = PortRegistry::open_in_memory().unwrap();
        r.assign_port("x").unwrap();
        r.assign_port("y").unwrap();
        let all = r.list_assignments().unwrap();
        assert_eq!(all.len(), 2);
    }
}
