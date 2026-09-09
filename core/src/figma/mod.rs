//! Cliente de Figma y Sistema de Eventos / Webhooks.

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// Errores específicos del cliente Figma.
#[derive(Debug, thiserror::Error)]
pub enum FigmaClientError {
    #[error("Error de red: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Respuesta inesperada de la API de Figma: {0}")]
    ApiResponse(String),
    #[error("Token de acceso de Figma no configurado.")]
    TokenNotConfigured,
    #[error("Error al parsear JSON: {0}")]
    JsonParse(#[from] serde_json::Error),
}

/// Cliente de la API de Figma.
pub struct FigmaClient {
    client: Client,
    personal_access_token: String,
}

impl FigmaClient {
    /// Crea una nueva instancia del cliente Figma.
    /// Lee el token de acceso personal de Figma de la variable de entorno FIGMA_PERSONAL_ACCESS_TOKEN.
    pub fn new() -> Result<Self, FigmaClientError> {
        let personal_access_token = env::var("FIGMA_PERSONAL_ACCESS_TOKEN")
            .map_err(|_| FigmaClientError::TokenNotConfigured)?;

        Ok(Self {
            client: Client::new(),
            personal_access_token,
        })
    }

    /// Obtiene la estructura de un archivo de Figma.
    /// file_key: La clave del archivo Figma (parte de la URL).
    pub async fn get_file(&self, file_key: &str) -> Result<FigmaFile, FigmaClientError> {
        let url = format!("https://api.figma.com/v1/files/{}", file_key);
        let response = self
            .client
            .get(&url)
            .header("X-Figma-Token", &self.personal_access_token)
            .send()
            .await?;

        if response.status().is_success() {
            let figma_file: FigmaFile = response.json().await?;
            Ok(figma_file)
        } else {
            let status = response.status();
            let text = response.text().await?;
            Err(FigmaClientError::ApiResponse(format!(
                "{}: {}",
                status, text
            )))
        }
    }

    /// Registra un nuevo Webhook en la API de Figma.
    /// - team_id: El ID del equipo de Figma.
    /// - event_type: Tipo de evento (ej. "FILE_UPDATE", "FILE_VERSION_UPDATE").
    /// - url: La URL pública de destino de tu servidor.
    /// - passcode: Una clave secreta para verificar que el evento venga realmente de Figma.
    pub async fn create_webhook(
        &self,
        team_id: &str,
        event_type: &str,
        webhook_url: &str,
        passcode: &str,
    ) -> Result<FigmaWebhookResponse, FigmaClientError> {
        let url = "https://api.figma.com/v2/webhooks";
        let payload = json_webhook_payload(team_id, event_type, webhook_url, passcode);

        let response = self
            .client
            .post(url)
            .header("X-Figma-Token", &self.personal_access_token)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let webhook_res: FigmaWebhookResponse = response.json().await?;
            Ok(webhook_res)
        } else {
            let status = response.status();
            let text = response.text().await?;
            Err(FigmaClientError::ApiResponse(format!(
                "{}: {}",
                status, text
            )))
        }
    }
}

fn json_webhook_payload(
    team_id: &str,
    event_type: &str,
    url: &str,
    passcode: &str,
) -> serde_json::Value {
    serde_json::json!({
        "event_type": event_type,
        "team_id": team_id,
        "url": url,
        "passcode": passcode
    })
}

// ==========================================================================
// Estructuras de datos para la API de Figma
// ==========================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct FigmaFile {
    pub name: String,
    pub document: Document,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub name: String,
    pub children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub name: String,
    #[serde(default)]
    pub children: Option<Vec<Node>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FigmaWebhookResponse {
    pub id: String,
    pub event_type: String,
    pub team_id: String,
    pub url: String,
    pub client_id: String,
    pub status: String,
}

// ==========================================================================
// Payload del Webhook recibido de Figma
// ==========================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FigmaWebhookPayload {
    pub event_type: String,
    pub file_key: String,
    pub file_name: String,
    pub timestamp: String,
    pub passcode: String,
    pub webhook_id: String,
    pub triggered_by: TriggeredBy,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TriggeredBy {
    pub id: String,
    pub handle: String,
    pub email: String,
}
