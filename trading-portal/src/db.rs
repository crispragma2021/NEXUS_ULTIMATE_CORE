use rusqlite::{params, Connection, Result};
use crate::state::{Orden, Cartera};
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    // Basic global connection for synchronous rusqlite wrapped in a Mutex for simple persistence.
    // In a fully robust HFT system, we would use a connection pool (e.g. sqlx + sqlite)
    // but for now, we will use this to prevent data loss across restarts.
    static ref DB_CONN: Mutex<Option<Connection>> = Mutex::new(None);
}

pub fn init_db() -> Result<()> {
    let conn = Connection::open("nexus_trading.db")?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ordenes (
            id TEXT PRIMARY KEY,
            simbolo TEXT NOT NULL,
            lado TEXT NOT NULL,
            tipo TEXT NOT NULL,
            cantidad REAL NOT NULL,
            precio REAL,
            estado TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            razon_nexus TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS cartera (
            id INTEGER PRIMARY KEY DEFAULT 1,
            usd REAL NOT NULL,
            nvda REAL NOT NULL,
            aapl REAL NOT NULL
        )",
        [],
    )?;

    // Insert default wallet if not exists
    conn.execute(
        "INSERT OR IGNORE INTO cartera (id, usd, nvda, aapl) VALUES (1, 10000.0, 0.0, 0.0)",
        [],
    )?;

    *DB_CONN.lock().unwrap() = Some(conn);
    Ok(())
}

pub fn save_order(orden: &Orden) -> Result<()> {
    if let Some(conn) = DB_CONN.lock().unwrap().as_ref() {
        conn.execute(
            "INSERT OR REPLACE INTO ordenes (id, simbolo, lado, tipo, cantidad, precio, estado, timestamp, razon_nexus)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                orden.id,
                orden.simbolo,
                orden.lado,
                orden.tipo,
                orden.cantidad,
                orden.precio,
                orden.estado,
                orden.timestamp,
                orden.razon_nexus
            ],
        )?;
    }
    Ok(())
}

pub fn save_cartera(cartera: &Cartera) -> Result<()> {
    if let Some(conn) = DB_CONN.lock().unwrap().as_ref() {
        conn.execute(
            "UPDATE cartera SET usd = ?1, nvda = ?2, aapl = ?3 WHERE id = 1",
            params![cartera.usd, cartera.nvda, cartera.aapl],
        )?;
    }
    Ok(())
}

pub fn load_cartera() -> Result<Option<Cartera>> {
    if let Some(conn) = DB_CONN.lock().unwrap().as_ref() {
        let mut stmt = conn.prepare("SELECT usd, nvda, aapl FROM cartera WHERE id = 1")?;
        let mut iter = stmt.query_map([], |row| {
            Ok(Cartera {
                usd: row.get(0)?,
                nvda: row.get(1)?,
                aapl: row.get(2)?,
            })
        })?;
        
        if let Some(result) = iter.next() {
            return Ok(Some(result?));
        }
    }
    Ok(None)
}

pub fn load_orders() -> Result<Vec<Orden>> {
    let mut ordenes = Vec::new();
    if let Some(conn) = DB_CONN.lock().unwrap().as_ref() {
        let mut stmt = conn.prepare("SELECT id, simbolo, lado, tipo, cantidad, precio, estado, timestamp, razon_nexus FROM ordenes ORDER BY timestamp DESC")?;
        let iter = stmt.query_map([], |row| {
            Ok(Orden {
                id: row.get(0)?,
                simbolo: row.get(1)?,
                lado: row.get(2)?,
                tipo: row.get(3)?,
                cantidad: row.get(4)?,
                precio: row.get(5)?,
                estado: row.get(6)?,
                timestamp: row.get(7)?,
                razon_nexus: row.get(8)?,
            })
        })?;
        
        for result in iter {
            ordenes.push(result?);
        }
    }
    Ok(ordenes)
}
