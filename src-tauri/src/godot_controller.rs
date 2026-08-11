//! godot_controller.rs — Cliente Rust del puente LLM ↔ Godot (NexusLLMBridge).
//!
//! Da al backend (y por tanto al LLM) control directo sobre el juego NEXUS Protocol
//! que corre en el motor Godot. Habla el protocolo JSON sobre HTTP/WebSocket
//! en el puerto 8081 (ver game/autoload/LLMBridge.gd).
//!
//! No añade dependencias nuevas: usa tokio + reqwest + serde_json ya presentes
//! en el workspace (src-tauri/Cargo.toml).

use serde_json::{json, Value};
use std::time::Duration;

/// Cliente HTTP del puente. Pensado para ser usado desde un comando Tauri
/// o desde un daemon de backend (async).
#[derive(Debug, Clone)]
pub struct GodotController {
    base_url: String,
    client: reqwest::Client,
}

impl Default for GodotController {
    fn default() -> Self {
        Self::new("http://127.0.0.1:8081")
    }
}

impl GodotController {
    /// Crea un controlador apuntando a la base del puente (por defecto 127.0.0.1:8081).
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(8))
            .build()
            .expect("reqwest client debe construir");
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    /// Chequea la salud del puente.
    pub async fn health(&self) -> Result<Value, String> {
        self.get_json("/health").await
    }

    /// Obtiene información del jugador (posición, vida, grupo).
    pub async fn get_player(&self) -> Result<Value, String> {
        self.get_json("/player").await
    }

    /// Obtiene el árbol de escena serializado.
    pub async fn get_scene_tree(&self) -> Result<Value, String> {
        self.get_json("/scene/tree").await
    }

    /// Envía un comando arbitrario al puente.
    pub async fn command(&self, cmd: &str, args: Value) -> Result<Value, String> {
        let body = json!({ "cmd": cmd, "args": args });
        let url = format!("{}/command", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("error HTTP: {e}"))?;
        let status = resp.status();
        let v: Value = resp.json().await.map_err(|e| format!("JSON inválido: {e}"))?;
        if !status.is_success() {
            return Err(format!("puente respondió {status}: {v}"));
        }
        Ok(v)
    }

    /// Spawnea una bestia en el mundo. Especies: lobo, boar, spider, bat, golem.
    pub async fn spawn_beast(&self, species: &str, x: f32, z: f32) -> Result<Value, String> {
        self.command(
            "SPAWN_BEAST",
            json!({ "species": species, "x": x, "z": z }),
        )
        .await
    }

    /// Mata (libera) todas las bestias del grupo `enemies`.
    pub async fn kill_beasts(&self) -> Result<Value, String> {
        self.command("KILL_BEASTS", json!({})).await
    }

    /// Inflige daño al jugador.
    pub async fn damage_player(&self, amount: f32) -> Result<Value, String> {
        self.command("DAMAGE_PLAYER", json!({ "amount": amount })).await
    }

    /// Cura al jugador.
    pub async fn heal_player(&self, amount: f32) -> Result<Value, String> {
        self.command("HEAL_PLAYER", json!({ "amount": amount })).await
    }

    /// Teletransporta al jugador a (x, z) conservando su altura.
    pub async fn move_player(&self, x: f32, z: f32) -> Result<Value, String> {
        self.command("MOVE_PLAYER", json!({ "x": x, "z": z })).await
    }

    async fn get_json(&self, path: &str) -> Result<Value, String> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("error HTTP: {e}"))?;
        resp.json().await.map_err(|e| format!("JSON inválido: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_con_puente_apagado_falla_limpio() {
        let ctrl = GodotController::new("http://127.0.0.1:59999"); // puerto sin servicio
        let r = ctrl.health().await;
        assert!(r.is_err(), "debe fallar si no hay puente escuchando");
    }
}
