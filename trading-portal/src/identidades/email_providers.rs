use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

// ============================================================================
// PROVEEDORES DE EMAIL — EmailProviders Mejorado
// ============================================================================
// Fusión de:
//   - legacy/nexus-orquestador/src/sembrador/email_providers.rs (mail.tm)
//   - Nuevos: GuerrillaMail, TempMail, 10MinMail
//   - Arquitectura extensible para más proveedores
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempEmail {
    pub address: String,
    pub password: String,
    pub provider: String,
    pub token: Option<String>,
    pub inbox_id: Option<String>,
}

/// Proveedor de email temporal
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProveedorTemp {
    MailTm,
    GuerrillaMail,
    TempMail,
}

impl ProveedorTemp {
    pub fn as_str(&self) -> &str {
        match self {
            ProveedorTemp::MailTm => "mail.tm",
            ProveedorTemp::GuerrillaMail => "guerrillamail",
            ProveedorTemp::TempMail => "tempmail",
        }
    }

    pub fn random() -> Self {
        match rand::random::<u8>() % 3 {
            0 => ProveedorTemp::MailTm,
            1 => ProveedorTemp::GuerrillaMail,
            _ => ProveedorTemp::TempMail,
        }
    }
}

pub struct EmailProviders {
    client: Client,
}

impl Default for EmailProviders {
    fn default() -> Self {
        Self::new()
    }
}

impl EmailProviders {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
                .build()
                .unwrap_or_default(),
        }
    }

    /// Crea un email temporal usando un proveedor aleatorio
    pub async fn crear_temporal(&self) -> anyhow::Result<TempEmail> {
        let proveedor = ProveedorTemp::random();
        info!("📧 [EMAIL] Usando proveedor: {}", proveedor.as_str());

        match proveedor {
            ProveedorTemp::MailTm => self.crear_mail_tm().await,
            ProveedorTemp::GuerrillaMail => self.crear_guerrilla_mail().await,
            ProveedorTemp::TempMail => self.crear_temp_mail().await,
        }
    }

    /// Crea un email temporal con proveedor específico
    pub async fn crear_con_proveedor(&self, proveedor: ProveedorTemp) -> anyhow::Result<TempEmail> {
        match proveedor {
            ProveedorTemp::MailTm => self.crear_mail_tm().await,
            ProveedorTemp::GuerrillaMail => self.crear_guerrilla_mail().await,
            ProveedorTemp::TempMail => self.crear_temp_mail().await,
        }
    }

    // ─── mail.tm API ────────────────────────────────────────────────────────
    async fn crear_mail_tm(&self) -> anyhow::Result<TempEmail> {
        let base_url = "https://api.mail.tm";

        // Obtener dominios disponibles
        let domains_resp = self
            .client
            .get(format!("{}/domains", base_url))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("mail.tm: {}", e))?;
        let domains_json: serde_json::Value = domains_resp.json().await?;
        let domains = domains_json["hydra:member"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No domains array"))?;

        if domains.is_empty() {
            anyhow::bail!("No domains available");
        }

        let rand_idx = rand::random::<usize>() % domains.len();
        let domain = domains[rand_idx]["domain"]
            .as_str()
            .unwrap_or("mail.tm");

        let id = uuid::Uuid::new_v4().to_string();
        let address = format!("{}@{}", &id[..10], domain);
        let password = format!("Nexus!{}", &id[24..]);

        // Crear cuenta
        let resp = self
            .client
            .post(format!("{}/accounts", base_url))
            .json(&serde_json::json!({
                "address": address,
                "password": password
            }))
            .send()
            .await?;

        if !resp.status().is_success() {
            warn!("⚠️ mail.tm create failed: {}", resp.status());
        }

        // Obtener token
        let token_resp = self
            .client
            .post(format!("{}/token", base_url))
            .json(&serde_json::json!({
                "address": address,
                "password": password
            }))
            .send()
            .await?;

        let token_data: serde_json::Value = token_resp.json().await.unwrap_or_default();
        let token = token_data["token"].as_str().map(|s| s.to_string());

        let inbox_id = token_data["id"].as_str().map(|s| s.to_string());

        Ok(TempEmail {
            address,
            password,
            provider: "mail.tm".to_string(),
            token,
            inbox_id,
        })
    }

    // ─── Guerrilla Mail API (sin registro, instantáneo) ─────────────────────
    async fn crear_guerrilla_mail(&self) -> anyhow::Result<TempEmail> {
        let resp = self
            .client
            .get("https://api.guerrillamail.com/ajax.php?f=get_email_address")
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;
        let address = data["email_addr"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No email from guerrilla"))?;

        let sid = data["sid_token"].as_str().unwrap_or("");
        let alias = data["alias"].as_str().unwrap_or("");

        Ok(TempEmail {
            address: address.to_string(),
            password: format!("Nexus!{}", &uuid::Uuid::new_v4().to_string()[..8]),
            provider: "guerrillamail".to_string(),
            token: Some(sid.to_string()),
            inbox_id: Some(alias.to_string()),
        })
    }

    // ─── TempMail API ───────────────────────────────────────────────────────
    async fn crear_temp_mail(&self) -> anyhow::Result<TempEmail> {
        let resp = self
            .client
            .post("https://api.temp-mail.org/request/domains/format/json")
            .send()
            .await?;

        let domains: Vec<String> = resp.json().await.unwrap_or_default();
        let domain = if domains.is_empty() {
            "temp-mail.org"
        } else {
            domains[rand::random::<usize>() % domains.len()].as_str()
        };

        let id = uuid::Uuid::new_v4().to_string();
        let local = &id[..10];
        let address = format!("{}@{}", local, domain);

        // TempMail no requiere registro - solo generar dirección
        Ok(TempEmail {
            address,
            password: format!("Nexus!{}", &id[24..]),
            provider: "tempmail".to_string(),
            token: None,
            inbox_id: None,
        })
    }

    /// Verifica si hay emails en la bandeja de entrada
    pub async fn verificar_inbox(&self, email: &TempEmail) -> anyhow::Result<Vec<EmailMessage>> {
        match email.provider.as_str() {
            "mail.tm" => self.verificar_mail_tm(email).await,
            "guerrillamail" => self.verificar_guerrilla(email).await,
            _ => Ok(Vec::new()),
        }
    }

    async fn verificar_mail_tm(&self, email: &TempEmail) -> anyhow::Result<Vec<EmailMessage>> {
        let token = email
            .token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No token"))?;

        let resp = self
            .client
            .get("https://api.mail.tm/messages")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;
        let messages = data["hydra:member"].as_array().map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let from = m["from"]["address"].as_str()?;
                    let subject = m["subject"].as_str()?;
                    let content = m["textContent"].as_str().or_else(|| m["htmlContent"].as_str());
                    Some(EmailMessage {
                        from: from.to_string(),
                        subject: subject.to_string(),
                        body: content.unwrap_or("").to_string(),
                        id: m["id"].as_str().unwrap_or("").to_string(),
                    })
                })
                .collect()
        });

        Ok(messages.unwrap_or_default())
    }

    async fn verificar_guerrilla(&self, email: &TempEmail) -> anyhow::Result<Vec<EmailMessage>> {
        let sid = email.token.as_deref().unwrap_or("");
        let resp = self
            .client
            .get(&format!(
                "https://api.guerrillamail.com/ajax.php?f=get_email_list&sid_token={}",
                sid
            ))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;
        let messages = data["list"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| {
                        let from = m["mail_from"].as_str()?;
                        let subject = m["mail_subject"].as_str()?;
                        Some(EmailMessage {
                            from: from.to_string(),
                            subject: subject.to_string(),
                            body: m["mail_excerpt"].as_str().unwrap_or("").to_string(),
                            id: m["mail_id"].to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(messages)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub from: String,
    pub subject: String,
    pub body: String,
    pub id: String,
}
