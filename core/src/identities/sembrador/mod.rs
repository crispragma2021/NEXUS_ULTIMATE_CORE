pub mod database;
pub mod email_providers;
pub mod gemini_rotator;
pub mod identity_factory;
pub mod vault;

use std::sync::Arc;

use database::IdentidadDb;
use email_providers::EmailProviders;
use gemini_rotator::GeminiKeyRotator;
use identity_factory::{Identidad, IdentityFactory};
use std::path::PathBuf;
use tracing::{info, warn};
use vault::NexusVault;

use crate::identities::browser_profile::BrowserProfileManager;
use crate::identities::chrome_planter::ChromePlanter;
use crate::identities::types::{IdentityFingerprint, IdentityProfile, SyntheticIdentity};

// ─── Tipos de cuenta ────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TipoCuenta {
    Temporal,
    Gmail,
    Proton,
    Facebook,
    Twitter,
    Sintetico,
}

impl std::fmt::Display for TipoCuenta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TipoCuenta::Temporal => write!(f, "Temporal"),
            TipoCuenta::Gmail => write!(f, "Gmail"),
            TipoCuenta::Proton => write!(f, "Proton"),
            TipoCuenta::Facebook => write!(f, "Facebook"),
            TipoCuenta::Twitter => write!(f, "Twitter"),
            TipoCuenta::Sintetico => write!(f, "Sintetico"),
        }
    }
}

impl TipoCuenta {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "temporal" | "temp" => Some(TipoCuenta::Temporal),
            "gmail" | "google" => Some(TipoCuenta::Gmail),
            "proton" | "protonmail" => Some(TipoCuenta::Proton),
            "facebook" | "fb" => Some(TipoCuenta::Facebook),
            "twitter" | "x" => Some(TipoCuenta::Twitter),
            "sintetico" | "synthetic" | "local" => Some(TipoCuenta::Sintetico),
            _ => None,
        }
    }
}

// ─── Estados de identidad ───────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EstadoIdentidad {
    Creada,
    EmailVerificado,
    Activa,
    Bloqueada,
    EnProceso,
    Fallida,
}

impl std::fmt::Display for EstadoIdentidad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EstadoIdentidad::Creada => write!(f, "Creada"),
            EstadoIdentidad::EmailVerificado => write!(f, "EmailVerificado"),
            EstadoIdentidad::Activa => write!(f, "Activa"),
            EstadoIdentidad::Bloqueada => write!(f, "Bloqueada"),
            EstadoIdentidad::EnProceso => write!(f, "EnProceso"),
            EstadoIdentidad::Fallida => write!(f, "Fallida"),
        }
    }
}

impl EstadoIdentidad {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "creada" => Some(EstadoIdentidad::Creada),
            "emailverificado" | "email_verificado" => Some(EstadoIdentidad::EmailVerificado),
            "activa" => Some(EstadoIdentidad::Activa),
            "bloqueada" => Some(EstadoIdentidad::Bloqueada),
            "enproceso" | "en_proceso" => Some(EstadoIdentidad::EnProceso),
            "fallida" => Some(EstadoIdentidad::Fallida),
            _ => None,
        }
    }
}

// ─── SembradorOmega ─────────────────────────────────────────────────────────
pub struct SembradorOmega {
    pub factory: IdentityFactory,
    pub providers: EmailProviders,
    pub db: Arc<IdentidadDb>,
    pub vault: NexusVault,
    pub gemini: Option<GeminiKeyRotator>,
    /// Cuando está configurado, las cuentas reales (Gmail, FB, Proton)
    /// se crean AUTOMÁTICAMENTE vía ChromePlanter Rust nativo.
    /// Cuando es None, se marcan como "EnProceso" (comportamiento legacy).
    pub planter: Option<Arc<ChromePlanter>>,
}

impl SembradorOmega {
    pub fn new(db_path: &str, vault_key: &[u8; 32]) -> anyhow::Result<Self> {
        Ok(Self {
            factory: IdentityFactory::new(),
            providers: EmailProviders::new(),
            db: Arc::new(IdentidadDb::new(db_path)?),
            vault: NexusVault::new(vault_key),
            gemini: None,
            planter: None,
        })
    }

    /// Conecta el ChromePlanter para crear cuentas REALES vía headless Chromium.
    /// Sin esto, Gmail/Facebook/Proton se marcan como "EnProceso" (solo preparación de datos).
    pub fn conectar_planter(&mut self, profile_base_dir: PathBuf) {
        let browser_mgr = BrowserProfileManager::new(profile_base_dir);
        let planter = ChromePlanter::new(browser_mgr);
        self.planter = Some(Arc::new(planter));
    }

    /// Conecta un ChromePlanter ya construido (para compartir entre sembradores)
    pub fn conectar_planter_existente(&mut self, planter: Arc<ChromePlanter>) {
        self.planter = Some(planter);
    }

    pub fn conectar_gemini(&mut self, cuentas: Vec<Vec<String>>) {
        self.gemini = Some(GeminiKeyRotator::new(cuentas));
    }

    /// Siembra una identidad del tipo especificado
    pub async fn sembrar(&self, tipo: TipoCuenta) -> anyhow::Result<Identidad> {
        match tipo {
            TipoCuenta::Sintetico => self.sembrar_sintetico().await,
            TipoCuenta::Temporal => self.sembrar_temporal().await,
            TipoCuenta::Gmail => self.sembrar_gmail().await,
            TipoCuenta::Proton => self.sembrar_proton().await,
            TipoCuenta::Facebook => self.sembrar_facebook().await,
            TipoCuenta::Twitter => self.sembrar_twitter().await,
        }
    }

    async fn sembrar_sintetico(&self) -> anyhow::Result<Identidad> {
        let mut identidad = self.factory.generar_identidad_base();
        identidad.tipo = "Sintetico".to_string();

        // Generar email sintético local
        let email_local = format!(
            "{}.{}@synthetic.nexus",
            identidad.nombre.to_lowercase(),
            identidad.apellido.to_lowercase()
        );
        identidad.email = email_local;
        identidad.estado = "Activa".to_string();

        // Guardar en base de datos
        self.db.registrar_identidad(
            &identidad.id,
            &identidad.nombre,
            &identidad.apellido,
            identidad.segundo_apellido.as_deref(),
            &identidad.email,
            &identidad.password,
            identidad.recovery_email.as_deref(),
            &identidad.fecha_nacimiento,
            &identidad.pais,
            &identidad.ciudad,
            &identidad.genero,
            identidad.telefono.as_deref(),
            identidad.foto_url.as_deref(),
            &identidad.tipo,
            &identidad.estado,
            None,
            None,
        )?;

        // Guardar en vault
        self.vault
            .guardar_credencial(&identidad.email, &identidad.password)?;

        Ok(identidad)
    }

    async fn sembrar_temporal(&self) -> anyhow::Result<Identidad> {
        let mut identidad = self.factory.generar_identidad_base();
        identidad.tipo = "Temporal".to_string();

        // Crear email temporal vía API
        let temp = match self.providers.crear_temporal().await {
            Ok(t) => t,
            Err(e) => {
                // Fallback: generar email sintético
                identidad.email = format!(
                    "temp.{}.{}@synthetic.nexus",
                    identidad.nombre.to_lowercase(),
                    identidad.apellido.to_lowercase()
                );
                identidad.email_provider = Some("fallback".to_string());
                identidad.estado = "Creada".to_string();
                self.db.registrar_identidad(
                    &identidad.id,
                    &identidad.nombre,
                    &identidad.apellido,
                    identidad.segundo_apellido.as_deref(),
                    &identidad.email,
                    &identidad.password,
                    identidad.recovery_email.as_deref(),
                    &identidad.fecha_nacimiento,
                    &identidad.pais,
                    &identidad.ciudad,
                    &identidad.genero,
                    identidad.telefono.as_deref(),
                    identidad.foto_url.as_deref(),
                    &identidad.tipo,
                    &identidad.estado,
                    identidad.email_provider.as_deref(),
                    None,
                )?;
                self.vault
                    .guardar_credencial(&identidad.email, &identidad.password)?;
                return Ok(identidad);
            }
        };

        identidad.email = temp.email.clone();
        identidad.email_provider = Some(temp.provider.clone());
        identidad.estado = "Creada".to_string();

        self.db.registrar_identidad(
            &identidad.id,
            &identidad.nombre,
            &identidad.apellido,
            identidad.segundo_apellido.as_deref(),
            &identidad.email,
            &identidad.password,
            identidad.recovery_email.as_deref(),
            &identidad.fecha_nacimiento,
            &identidad.pais,
            &identidad.ciudad,
            &identidad.genero,
            identidad.telefono.as_deref(),
            identidad.foto_url.as_deref(),
            &identidad.tipo,
            &identidad.estado,
            identidad.email_provider.as_deref(),
            Some(
                &serde_json::json!({
                    "temp_token": temp.token,
                    "temp_id": temp.id
                })
                .to_string(),
            ),
        )?;

        self.vault
            .guardar_credencial(&identidad.email, &identidad.password)?;

        Ok(identidad)
    }

    async fn sembrar_gmail(&self) -> anyhow::Result<Identidad> {
        let mut identidad = self.factory.generar_identidad_base();
        identidad.tipo = "Gmail".to_string();

        // Generar email @gmail.com (se sobreescribe si ChromePlanter crea la cuenta real)
        let email_sugerido = identidad.generar_email_gmail();
        identidad.email = email_sugerido.clone();
        identidad.email_provider = Some("google".to_string());

        if let Some(ref planter) = self.planter {
            // 🔥 MODO OMEGA: Crear cuenta Gmail REAL via ChromePlanter Rust nativo
            let identidad_db = synthetic_profile_from_identidad(&identidad);
            let recovery = identidad.recovery_email.clone();

            let result = planter
                .crear_cuenta_gmail(
                    &identidad.nombre,
                    &identidad.apellido,
                    &identidad.password,
                    recovery.as_deref(),
                    &identidad_db,
                )
                .await;

            if result.success {
                // Usar el email real devuelto por ChromePlanter (o el generado)
                if let Some(ref email_real) = result.email {
                    identidad.email = email_real.clone();
                }
                identidad.estado = "Activa".to_string();
                info!("✅ [SEMBRADOR] Cuenta Gmail CREADA: {}", identidad.email);
            } else {
                identidad.estado = "Fallida".to_string();
                warn!("❌ [SEMBRADOR] Falló creación Gmail: {:?}", result.error);
            }

            self.db.registrar_identidad(
                &identidad.id,
                &identidad.nombre,
                &identidad.apellido,
                identidad.segundo_apellido.as_deref(),
                &identidad.email,
                &identidad.password,
                identidad.recovery_email.as_deref(),
                &identidad.fecha_nacimiento,
                &identidad.pais,
                &identidad.ciudad,
                &identidad.genero,
                identidad.telefono.as_deref(),
                identidad.foto_url.as_deref(),
                &identidad.tipo,
                &identidad.estado,
                identidad.email_provider.as_deref(),
                Some(
                    &serde_json::json!({
                        "via": "chrome_planter_rust",
                        "success": result.success,
                        "pending_verification": result.pending_verification,
                        "error": result.error,
                    })
                    .to_string(),
                ),
            )?;
        } else {
            // Modo legacy: solo preparar datos, marcar EnProceso
            identidad.estado = "EnProceso".to_string();

            self.db.registrar_identidad(
                &identidad.id,
                &identidad.nombre,
                &identidad.apellido,
                identidad.segundo_apellido.as_deref(),
                &identidad.email,
                &identidad.password,
                identidad.recovery_email.as_deref(),
                &identidad.fecha_nacimiento,
                &identidad.pais,
                &identidad.ciudad,
                &identidad.genero,
                identidad.telefono.as_deref(),
                identidad.foto_url.as_deref(),
                &identidad.tipo,
                &identidad.estado,
                identidad.email_provider.as_deref(),
                Some(
                    &serde_json::json!({
                        "via": "legacy_placeholder",
                        "script": "sembrador_chrome.js"
                    })
                    .to_string(),
                ),
            )?;
        }

        self.vault
            .guardar_credencial(&identidad.email, &identidad.password)?;
        Ok(identidad)
    }

    async fn sembrar_proton(&self) -> anyhow::Result<Identidad> {
        let mut identidad = self.factory.generar_identidad_base();
        identidad.tipo = "Proton".to_string();
        identidad.email = identidad.generar_email_proton();
        identidad.estado = "EnProceso".to_string();
        identidad.email_provider = Some("proton".to_string());

        self.db.registrar_identidad(
            &identidad.id,
            &identidad.nombre,
            &identidad.apellido,
            identidad.segundo_apellido.as_deref(),
            &identidad.email,
            &identidad.password,
            identidad.recovery_email.as_deref(),
            &identidad.fecha_nacimiento,
            &identidad.pais,
            &identidad.ciudad,
            &identidad.genero,
            identidad.telefono.as_deref(),
            identidad.foto_url.as_deref(),
            &identidad.tipo,
            &identidad.estado,
            identidad.email_provider.as_deref(),
            None,
        )?;

        self.vault
            .guardar_credencial(&identidad.email, &identidad.password)?;

        Ok(identidad)
    }

    async fn sembrar_facebook(&self) -> anyhow::Result<Identidad> {
        let mut identidad = self.factory.generar_identidad_base();
        identidad.tipo = "Facebook".to_string();
        // Facebook usa email temporal o existente
        identidad.email = format!(
            "fb.{}.{}@synthetic.nexus",
            identidad.nombre.to_lowercase(),
            identidad.apellido.to_lowercase()
        );
        identidad.estado = "EnProceso".to_string();

        self.db.registrar_identidad(
            &identidad.id,
            &identidad.nombre,
            &identidad.apellido,
            identidad.segundo_apellido.as_deref(),
            &identidad.email,
            &identidad.password,
            identidad.recovery_email.as_deref(),
            &identidad.fecha_nacimiento,
            &identidad.pais,
            &identidad.ciudad,
            &identidad.genero,
            identidad.telefono.as_deref(),
            identidad.foto_url.as_deref(),
            &identidad.tipo,
            &identidad.estado,
            None,
            None,
        )?;

        self.vault
            .guardar_credencial(&identidad.email, &identidad.password)?;

        Ok(identidad)
    }

    async fn sembrar_twitter(&self) -> anyhow::Result<Identidad> {
        let mut identidad = self.factory.generar_identidad_base();
        identidad.tipo = "Twitter".to_string();
        identidad.email = format!(
            "tw.{}.{}@synthetic.nexus",
            identidad.nombre.to_lowercase(),
            identidad.apellido.to_lowercase()
        );
        identidad.estado = "EnProceso".to_string();

        self.db.registrar_identidad(
            &identidad.id,
            &identidad.nombre,
            &identidad.apellido,
            identidad.segundo_apellido.as_deref(),
            &identidad.email,
            &identidad.password,
            identidad.recovery_email.as_deref(),
            &identidad.fecha_nacimiento,
            &identidad.pais,
            &identidad.ciudad,
            &identidad.genero,
            identidad.telefono.as_deref(),
            identidad.foto_url.as_deref(),
            &identidad.tipo,
            &identidad.estado,
            None,
            None,
        )?;

        self.vault
            .guardar_credencial(&identidad.email, &identidad.password)?;

        Ok(identidad)
    }

    /// Lista identidades registradas
    pub fn listar_identidades(&self, limit: usize) -> anyhow::Result<Vec<Identidad>> {
        self.db.listar_identidades(limit)
    }

    /// Obtiene una identidad por email
    pub fn obtener_identidad(&self, email: &str) -> anyhow::Result<Option<Identidad>> {
        self.db.obtener_identidad(email)
    }

    /// Actualiza el estado de una identidad
    pub fn actualizar_estado(&self, email: &str, estado: &str) -> anyhow::Result<()> {
        self.db.actualizar_estado(email, estado)
    }

    /// Reporte estadístico de identidades
    pub fn reporte(&self) -> anyhow::Result<serde_json::Value> {
        self.db.reporte_estadistico()
    }
}

impl std::fmt::Debug for SembradorOmega {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SembradorOmega").finish()
    }
}

// ─── Conversor Identidad → SyntheticIdentity ────────────────────────────────

/// Convierte la estructura interna `Identidad` (sembrador/identity_factory)
/// a `SyntheticIdentity` (types.rs) para usar con ChromePlanter y otros módulos
/// que operan sobre el modelo de datos unificado.
fn synthetic_profile_from_identidad(id: &Identidad) -> SyntheticIdentity {
    let profile = IdentityProfile {
        full_name: format!("{} {}", id.nombre, id.apellido),
        gender: id.genero.clone(),
        age: calcular_edad(&id.fecha_nacimiento),
        nationality: id.pais.clone(),
        occupation: "Profesional".to_string(),
        city: id.ciudad.clone(),
        country: id.pais.clone(),
        bio: format!("Identidad sintética tipo {}", id.tipo),
        traits: Vec::new(),
        interests: Vec::new(),
    };

    let fingerprint = IdentityFingerprint {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36".to_string(),
        screen_resolution: "1366x768".to_string(),
        timezone: "America/Asuncion".to_string(),
        language: "es-PY".to_string(),
        platform: "Linux x86_64".to_string(),
        browser_profile_dir: None,
    };

    let mut identity = SyntheticIdentity::new(profile);
    identity.fingerprint = fingerprint;

    if !id.email.is_empty() {
        identity
            .emails
            .push(crate::identities::types::EmailAccount {
                address: id.email.clone(),
                password: id.password.clone(),
                provider: crate::identities::types::EmailProvider::Gmail,
                verified: false,
            });
    }

    identity
}

fn calcular_edad(fecha_nacimiento: &str) -> u8 {
    // Formato esperado: YYYY-MM-DD
    let parts: Vec<&str> = fecha_nacimiento.split('-').collect();
    if parts.len() == 3 {
        if let Ok(year) = parts[0].parse::<i32>() {
            let current_year = 2025;
            let edad = (current_year - year) as u8;
            return edad.clamp(18, 99);
        }
    }
    30 // default
}
