// 🌱 NEXUS OMEGA — Fábrica de Correos Electrónicos para Identidades
// ============================================================
// Reusa TemporalMailClient (mail.tm) para crear cuentas de correo
// por cada identidad sintética.

use crate::comms::correo_temporal::TemporalMailClient;
use crate::identities::types::{EmailAccount, EmailProvider, SyntheticIdentity};
use anyhow::Result;

/// Crea cuentas de correo temporales asociadas a identidades sintéticas
pub struct MailFactory {
    client: TemporalMailClient,
}

impl Default for MailFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl MailFactory {
    pub fn new() -> Self {
        Self {
            client: TemporalMailClient::new(),
        }
    }

    /// Crea una cuenta de correo temporal para la identidad usando mail.tm
    /// Retorna el EmailAccount creado
    pub async fn create_for_identity(&self, identity: &SyntheticIdentity) -> Result<EmailAccount> {
        let domains = self.client.obtener_dominios().await?;
        let domain = domains
            .first()
            .map(|d| d.domain.clone())
            .unwrap_or_else(|| "@nemob.com".to_string());

        // Sanitizar nombre para email: sin espacios, minúsculas
        let local_part = identity
            .profile
            .full_name
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(".")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '_')
            .collect::<String>();

        // Añadir sufijo único con 4 caracteres aleatorios
        use rand::Rng;
        let suffix: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(4)
            .map(|c| c as char)
            .collect();

        let email = format!("{}.{}@{}", local_part, suffix.to_lowercase(), domain);
        let password = format!("Nexus{}!", suffix.to_uppercase());

        // Crear la cuenta vía mail.tm API
        let account = self.client.crear_cuenta(&email, &password).await?;

        Ok(EmailAccount {
            address: account.address,
            password,
            provider: EmailProvider::MailTm,
            verified: false,
        })
    }

    /// Asigna cuentas de correo a múltiples identidades
    pub async fn populate_identities(
        &self,
        identities: &mut [SyntheticIdentity],
        emails_per_identity: usize,
    ) -> Result<()> {
        for identity in identities.iter_mut() {
            for _ in 0..emails_per_identity {
                match self.create_for_identity(identity).await {
                    Ok(email_acct) => identity.emails.push(email_acct),
                    Err(e) => {
                        eprintln!(
                            "⚠️  No se pudo crear correo para {}: {}",
                            identity.profile.full_name, e
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Verifica bandeja de entrada de una cuenta (para confirmación 2FA, etc.)
    pub async fn check_inbox(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Vec<crate::comms::correo_temporal::MessageHeader>> {
        let token = self.client.obtener_token(email, password).await?;
        let messages = self.client.listar_mensajes(&token).await?;
        Ok(messages)
    }
}
