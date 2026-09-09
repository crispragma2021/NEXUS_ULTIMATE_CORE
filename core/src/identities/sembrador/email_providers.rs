use reqwest::Client;
use serde::{Deserialize, Serialize};

// ─── Email temporal ─────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempEmail {
    pub email: String,
    pub token: Option<String>,
    pub id: Option<String>,
    pub provider: String,
}

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

// ─── EmailProviders ─────────────────────────────────────────────────────────
#[derive(Clone)]
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
                .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
                .build()
                .expect("Fallo al crear HTTP client"),
        }
    }

    /// Crea un email temporal usando un proveedor aleatorio
    pub async fn crear_temporal(&self) -> anyhow::Result<TempEmail> {
        let proveedor = ProveedorTemp::random();
        self.crear_con_proveedor(proveedor).await
    }

    /// Crea un email temporal con un proveedor específico
    pub async fn crear_con_proveedor(&self, proveedor: ProveedorTemp) -> anyhow::Result<TempEmail> {
        match proveedor {
            ProveedorTemp::MailTm => self.crear_mail_tm().await,
            ProveedorTemp::GuerrillaMail => self.crear_guerrilla_mail().await,
            ProveedorTemp::TempMail => self.crear_temp_mail().await,
        }
    }

    /// mail.tm — API moderna con autenticación
    async fn crear_mail_tm(&self) -> anyhow::Result<TempEmail> {
        // Primero obtener un dominio disponible
        let dominios: serde_json::Value = self
            .client
            .get("https://api.mail.tm/domains")
            .send()
            .await?
            .json()
            .await?;

        let domain = dominios["hydra:member"][0]["domain"]
            .as_str()
            .unwrap_or("cliptik.net");

        let addr = format!(
            "nexus.{}.{}@{}",
            chrono::Utc::now().timestamp_millis() % 100000,
            rand::random::<u16>(),
            domain
        );

        let create_resp: serde_json::Value = self
            .client
            .post("https://api.mail.tm/accounts")
            .json(&serde_json::json!({
                "address": addr,
                "password": "NexusTemp2025!"
            }))
            .send()
            .await?
            .json()
            .await?;

        let id = create_resp["id"].as_str().unwrap_or("").to_string();

        // Obtener token
        let token_resp: serde_json::Value = self
            .client
            .post("https://api.mail.tm/token")
            .json(&serde_json::json!({
                "address": addr,
                "password": "NexusTemp2025!"
            }))
            .send()
            .await?
            .json()
            .await?;

        let token = token_resp["token"].as_str().unwrap_or("").to_string();

        Ok(TempEmail {
            email: addr,
            token: Some(token),
            id: Some(id),
            provider: "mail.tm".to_string(),
        })
    }

    /// Guerrilla Mail — instantáneo, no requiere registro
    async fn crear_guerrilla_mail(&self) -> anyhow::Result<TempEmail> {
        let resp: serde_json::Value = self
            .client
            .get("https://api.guerrillamail.com/ajax.php?f=get_email_address&ip=127.0.0.1&agent=NEXUS_IMPLANT")
            .send()
            .await?
            .json()
            .await?;

        let email = resp["email_addr"]
            .as_str()
            .unwrap_or("error@guerrillamail.com");
        let sid = resp["sid"].as_str().unwrap_or("");

        Ok(TempEmail {
            email: email.to_string(),
            token: Some(sid.to_string()),
            id: None,
            provider: "guerrillamail".to_string(),
        })
    }

    /// TempMail — API simple basada en dominio
    async fn crear_temp_mail(&self) -> anyhow::Result<TempEmail> {
        let dominios = [
            "boxomail.live",
            "cobareta.com",
            "gmailos.com",
            "texasbcs.xyz",
            "xemaps.com",
        ];
        let dominio = dominios[rand::random::<usize>() % dominios.len()];
        let local = format!("nexus.{}", chrono::Utc::now().timestamp_millis() % 999999);
        let email = format!("{}@{}", local, dominio);

        Ok(TempEmail {
            email,
            token: None,
            id: None,
            provider: "tempmail".to_string(),
        })
    }

    /// Verifica la bandeja de entrada de un email temporal
    pub async fn verificar_inbox(&self, email: &TempEmail) -> anyhow::Result<Vec<EmailMessage>> {
        match email.provider.as_str() {
            "mail.tm" => self.verificar_mail_tm(email).await,
            "guerrillamail" => self.verificar_guerrilla(email).await,
            _ => Ok(Vec::new()),
        }
    }

    async fn verificar_mail_tm(&self, email: &TempEmail) -> anyhow::Result<Vec<EmailMessage>> {
        let token = match &email.token {
            Some(t) => t,
            None => return Ok(Vec::new()),
        };

        let resp: serde_json::Value = self
            .client
            .get("https://api.mail.tm/messages")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?
            .json()
            .await?;

        let messages = resp["hydra:member"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|msg| {
                        Some(EmailMessage {
                            id: msg["id"].as_str()?.to_string(),
                            from: msg["from"]["address"].as_str()?.to_string(),
                            subject: msg["subject"].as_str()?.to_string(),
                            intro: msg["intro"].as_str()?.to_string(),
                            received_at: msg["createdAt"].as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(messages)
    }

    async fn verificar_guerrilla(&self, email: &TempEmail) -> anyhow::Result<Vec<EmailMessage>> {
        let sid = match &email.token {
            Some(s) => s,
            None => return Ok(Vec::new()),
        };

        let resp: serde_json::Value = self
            .client
            .get(format!(
                "https://api.guerrillamail.com/ajax.php?f=get_email_list&sid={}",
                sid
            ))
            .send()
            .await?
            .json()
            .await?;

        let messages = resp["list"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|msg| {
                        Some(EmailMessage {
                            id: msg["mail_id"].as_str()?.to_string(),
                            from: msg["mail_from"].as_str()?.to_string(),
                            subject: msg["mail_subject"].as_str()?.to_string(),
                            intro: msg["mail_excerpt"].as_str()?.to_string(),
                            received_at: msg["mail_timestamp"].as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(messages)
    }
}

// ─── EmailMessage ───────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub id: String,
    pub from: String,
    pub subject: String,
    pub intro: String,
    pub received_at: String,
}
