use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const BASE_URL: &str = "https://api.mail.tm";

#[derive(Debug, Clone)]
pub struct TemporalMailClient {
    client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DomainInfo {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "domain")]
    pub domain: String,
    #[serde(rename = "isActive")]
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
struct DomainList {
    #[serde(rename = "hydra:member")]
    pub members: Vec<DomainInfo>,
}

#[derive(Debug, Deserialize)]
pub struct AccountResponse {
    pub id: String,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailAddress {
    pub address: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MessageHeader {
    pub id: String,
    pub from: EmailAddress,
    pub subject: String,
    pub intro: Option<String>,
    pub seen: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct MessageList {
    #[serde(rename = "hydra:member")]
    pub members: Vec<MessageHeader>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MessageDetail {
    pub id: String,
    pub from: EmailAddress,
    pub subject: String,
    pub text: Option<String>,
    pub html: Option<Vec<String>>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

impl Default for TemporalMailClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TemporalMailClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub async fn obtener_dominios(&self) -> Result<Vec<DomainInfo>> {
        let url = format!("{}/domains", BASE_URL);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Error obteniendo dominios: {}", resp.status()));
        }
        let list: DomainList = resp.json().await?;
        Ok(list.members.into_iter().filter(|d| d.is_active).collect())
    }

    pub async fn crear_cuenta(&self, email: &str, password: &str) -> Result<AccountResponse> {
        let url = format!("{}/accounts", BASE_URL);
        let payload = serde_json::json!({
            "address": email,
            "password": password
        });

        let resp = self.client.post(&url).json(&payload).send().await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Error creando cuenta: {}", err_text));
        }

        let account: AccountResponse = resp.json().await?;
        Ok(account)
    }

    pub async fn obtener_token(&self, email: &str, password: &str) -> Result<String> {
        let url = format!("{}/token", BASE_URL);
        let payload = serde_json::json!({
            "address": email,
            "password": password
        });

        let resp = self.client.post(&url).json(&payload).send().await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Error de autenticación: {}", err_text));
        }

        let token_resp: TokenResponse = resp.json().await?;
        Ok(token_resp.token)
    }

    fn headers_autenticados(&self, token: &str) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let auth_val = format!("Bearer {}", token);
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_val)?);
        Ok(headers)
    }

    pub async fn listar_mensajes(&self, token: &str) -> Result<Vec<MessageHeader>> {
        let url = format!("{}/messages", BASE_URL);
        let headers = self.headers_autenticados(token)?;

        let resp = self.client.get(&url).headers(headers).send().await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Error listando mensajes: {}", resp.status()));
        }

        let list: MessageList = resp.json().await?;
        Ok(list.members)
    }

    pub async fn obtener_contenido_mensaje(&self, id: &str, token: &str) -> Result<MessageDetail> {
        let url = format!("{}/messages/{}", BASE_URL, id);
        let headers = self.headers_autenticados(token)?;

        let resp = self.client.get(&url).headers(headers).send().await?;

        if !resp.status().is_success() {
            return Err(anyhow!(
                "Error recuperando el correo {}: {}",
                id,
                resp.status()
            ));
        }

        let msg: MessageDetail = resp.json().await?;
        Ok(msg)
    }
}
