// 🌱 NEXUS OMEGA — Tipos de Datos del Sembrador de Identidades
// ============================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Estado del ciclo de vida de una identidad plantada
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IdentityStatus {
    /// Generada pero aún no usada (en pool caliente)
    Pool,
    /// Actualmente en uso por una operación
    Active,
    /// En reposo, disponible para reutilizar
    Dormant,
    /// Operación completada, pendiente de destrucción
    Expired,
    /// Destruida permanentemente
    Destroyed,
}

impl std::fmt::Display for IdentityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityStatus::Pool => write!(f, "pool"),
            IdentityStatus::Active => write!(f, "active"),
            IdentityStatus::Dormant => write!(f, "dormant"),
            IdentityStatus::Expired => write!(f, "expired"),
            IdentityStatus::Destroyed => write!(f, "destroyed"),
        }
    }
}

/// Proveedor de correo electrónico
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmailProvider {
    MailTm,
    Gmail,
    Outlook,
    ProtonMail,
}

impl std::fmt::Display for EmailProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailProvider::MailTm => write!(f, "mail.tm"),
            EmailProvider::Gmail => write!(f, "gmail"),
            EmailProvider::Outlook => write!(f, "outlook"),
            EmailProvider::ProtonMail => write!(f, "protonmail"),
        }
    }
}

/// Proveedor de número telefónico
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PhoneProvider {
    SMSActivate,
    Twilio,
    Virtual,
}

impl std::fmt::Display for PhoneProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhoneProvider::SMSActivate => write!(f, "sms-activate"),
            PhoneProvider::Twilio => write!(f, "twilio"),
            PhoneProvider::Virtual => write!(f, "virtual"),
        }
    }
}

/// Perfil de persona sintética generado por Mistral
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProfile {
    pub full_name: String,
    pub gender: String,
    pub age: u8,
    pub nationality: String,
    pub occupation: String,
    pub city: String,
    pub country: String,
    pub bio: String,
    pub traits: Vec<String>,
    pub interests: Vec<String>,
}

impl Default for IdentityProfile {
    fn default() -> Self {
        Self {
            full_name: "Unknown".to_string(),
            gender: "N/A".to_string(),
            age: 0,
            nationality: "Unknown".to_string(),
            occupation: "Unknown".to_string(),
            city: "Unknown".to_string(),
            country: "Unknown".to_string(),
            bio: String::new(),
            traits: Vec::new(),
            interests: Vec::new(),
        }
    }
}

impl IdentityProfile {
    pub fn summary(&self) -> String {
        format!(
            "{} · {} años · {} · {} · {}",
            self.full_name, self.age, self.occupation, self.city, self.country
        )
    }
}

/// Cuenta de correo asociada a una identidad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAccount {
    pub address: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub provider: EmailProvider,
    pub verified: bool,
}

/// Cuenta telefónica asociada a una identidad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneAccount {
    pub number: String,
    pub provider: PhoneProvider,
    pub verified: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Cuenta en red social
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialAccount {
    pub platform: String,
    pub username: String,
    pub profile_url: String,
    pub verified: bool,
}

/// Huella digital técnica para evitar fingerprinting cruzado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityFingerprint {
    pub user_agent: String,
    pub screen_resolution: String,
    pub timezone: String,
    pub language: String,
    pub platform: String,
    pub browser_profile_dir: Option<String>,
}

impl Default for IdentityFingerprint {
    fn default() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            screen_resolution: "1920x1080".to_string(),
            timezone: "America/Asuncion".to_string(),
            language: "es-PY".to_string(),
            platform: "Linux x86_64".to_string(),
            browser_profile_dir: None,
        }
    }
}

/// Identidad sintética completa — la unidad fundamental del Sembrador
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticIdentity {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub status: IdentityStatus,

    // Datos de la persona sintética
    pub profile: IdentityProfile,

    // Canales de comunicación
    pub emails: Vec<EmailAccount>,
    pub phones: Vec<PhoneAccount>,

    // Redes sociales
    pub accounts: Vec<SocialAccount>,

    // Huella digital técnica
    pub fingerprint: IdentityFingerprint,

    // Metadatos operativos
    pub operation_id: Option<String>,
    pub last_used: Option<DateTime<Utc>>,
    pub notes: String,
}

impl SyntheticIdentity {
    /// Crear una identidad nueva con valores por defecto
    pub fn new(profile: IdentityProfile) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            status: IdentityStatus::Pool,
            profile,
            emails: Vec::new(),
            phones: Vec::new(),
            accounts: Vec::new(),
            fingerprint: IdentityFingerprint::default(),
            operation_id: None,
            last_used: None,
            notes: String::new(),
        }
    }

    /// Reconstruir desde fila de base de datos (sin correos/teléfonos/cuentas)
    pub fn load_from_row(
        id: String,
        created_at: String,
        status: String,
        profile_data: String,
        fingerprint_data: String,
        operation_id: Option<String>,
        last_used: Option<String>,
        notes: String,
    ) -> Self {
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        let last_used = last_used
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let status = match status.as_str() {
            "active" => IdentityStatus::Active,
            "dormant" => IdentityStatus::Dormant,
            "expired" => IdentityStatus::Expired,
            "destroyed" => IdentityStatus::Destroyed,
            _ => IdentityStatus::Pool,
        };

        let profile: IdentityProfile = serde_json::from_str(&profile_data).unwrap_or_default();
        let fingerprint: IdentityFingerprint =
            serde_json::from_str(&fingerprint_data).unwrap_or_default();

        Self {
            id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4()),
            created_at,
            status,
            profile,
            emails: Vec::new(),
            phones: Vec::new(),
            accounts: Vec::new(),
            fingerprint,
            operation_id,
            last_used,
            notes,
        }
    }

    /// Resumen de una línea para mostrar en CLI
    pub fn short_summary(&self) -> String {
        format!(
            "[{}] {} | {} | {} correo(s) | {} tel(s) | {}",
            &self.id.to_string()[..8],
            self.profile.summary(),
            self.status,
            self.emails.len(),
            self.phones.len(),
            &self.fingerprint.user_agent[..60.min(self.fingerprint.user_agent.len())],
        )
    }

    /// Marcar como en uso por una operación
    pub fn mark_active(&mut self, operation_id: &str) {
        self.status = IdentityStatus::Active;
        self.operation_id = Some(operation_id.to_string());
        self.last_used = Some(Utc::now());
    }

    /// Marcar como destruida
    pub fn mark_destroyed(&mut self) {
        self.status = IdentityStatus::Destroyed;
    }
}
