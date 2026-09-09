pub mod database;
pub mod email_providers;
pub mod gemini_rotator;
pub mod identity_factory;
pub mod vault;

use database::IdentidadDb;
use email_providers::EmailProviders;
use gemini_rotator::GeminiKeyRotator;
use identity_factory::{Identidad, IdentityFactory};
use std::sync::Arc;
use tracing::{error, info};

// ============================================================================
// SEMBRADOR OMEGA — Motor Unificado de Generación de Identidades
// ============================================================================
// Fusión de:
//   - legacy/nexus-orquestador/src/sembrador/
//   - legacy/nexus-orquestador/src/motor_identidad.rs
//   - legacy/nexus-orquestador/src/hipocampo_cognitivo.rs
//   - legacy/reliquias/nexus-vision-hud/cortex_ejecutivo.js
//   - scripts/gabriel_birth.js
// ============================================================================

/// Tipo de cuenta a generar
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TipoCuenta {
    /// Cuenta temporal vía mail.tm / guerrilla mail
    Temporal,
    /// Gmail real vía Chrome headless automation
    Gmail,
    /// Proton Mail real vía Chrome headless automation
    Proton,
    /// Facebook (hereda patrón gabriel_birth.js)
    Facebook,
    /// Twitter/X real
    Twitter,
    /// Solo datos sintéticos sin email real
    Sintetico,
}

impl std::fmt::Display for TipoCuenta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TipoCuenta::Temporal => write!(f, "temporal"),
            TipoCuenta::Gmail => write!(f, "gmail"),
            TipoCuenta::Proton => write!(f, "proton"),
            TipoCuenta::Facebook => write!(f, "facebook"),
            TipoCuenta::Twitter => write!(f, "twitter"),
            TipoCuenta::Sintetico => write!(f, "sintetico"),
        }
    }
}

impl TipoCuenta {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "temporal" => Some(Self::Temporal),
            "gmail" => Some(Self::Gmail),
            "proton" => Some(Self::Proton),
            "facebook" => Some(Self::Facebook),
            "twitter" => Some(Self::Twitter),
            "sintetico" => Some(Self::Sintetico),
            _ => None,
        }
    }
}

/// Estados de una identidad durante su ciclo de vida
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum EstadoIdentidad {
    /// Recién creada, datos generados
    Creada,
    /// Email temporal verificado
    EmailVerificado,
    /// Cuenta Gmail/Proton creada exitosamente
    Activa,
    /// Cuenta bloqueada o desactivada
    Bloqueada,
    /// En proceso de creación (Chrome automation corriendo)
    EnProceso,
    /// Falló la creación
    Fallida,
}

impl std::fmt::Display for EstadoIdentidad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EstadoIdentidad::Creada => write!(f, "creada"),
            EstadoIdentidad::EmailVerificado => write!(f, "email_verificado"),
            EstadoIdentidad::Activa => write!(f, "activa"),
            EstadoIdentidad::Bloqueada => write!(f, "bloqueada"),
            EstadoIdentidad::EnProceso => write!(f, "en_proceso"),
            EstadoIdentidad::Fallida => write!(f, "fallida"),
        }
    }
}

impl EstadoIdentidad {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "creada" => Some(Self::Creada),
            "email_verificado" => Some(Self::EmailVerificado),
            "activa" => Some(Self::Activa),
            "bloqueada" => Some(Self::Bloqueada),
            "en_proceso" => Some(Self::EnProceso),
            "fallida" => Some(Self::Fallida),
            _ => None,
        }
    }
}

// ============================================================================
// ORQUESTADOR PRINCIPAL
// ============================================================================

pub struct SembradorOmega {
    pub factory: IdentityFactory,
    pub providers: EmailProviders,
    pub db: Arc<IdentidadDb>,
    pub vault: vault::NexusVault,
    pub gemini: Option<GeminiKeyRotator>,
}

impl SembradorOmega {
    pub fn new(db_path: &str, vault_key: &[u8; 32]) -> anyhow::Result<Self> {
        let db = Arc::new(IdentidadDb::new(db_path)?);
        let vault = vault::NexusVault::new(vault_key);
        
        Ok(Self {
            factory: IdentityFactory::new(),
            providers: EmailProviders::new(),
            db,
            vault,
            gemini: None,
        })
    }

    /// Vincula el rotador de claves Gemini (4 cuentas Google existentes)
    pub fn conectar_gemini(&mut self, cuentas: Vec<Vec<String>>) {
        self.gemini = Some(GeminiKeyRotator::new(cuentas));
        info!("🔑 Gemini rotator conectado");
    }

    /// [NÚCLEO]: Genera una identidad completa según el tipo solicitado
    pub async fn sembrar(&self, tipo: TipoCuenta) -> anyhow::Result<Identidad> {
        let mut identidad = self.factory.generar_identidad_base();

        match tipo {
            TipoCuenta::Temporal => {
                let temp = self.providers.crear_temporal().await?;
                identidad.email = temp.address;
                identidad.password = temp.password;
                identidad.email_provider = "mail.tm".to_string();
                identidad.tipo = tipo.to_string();
                identidad.estado = EstadoIdentidad::EmailVerificado.to_string();
            }
            TipoCuenta::Gmail => {
                let temp = self.providers.crear_temporal().await?;
                identidad.email = identidad.generar_email_gmail();
                identidad.password = self.factory.generar_password(24);
                identidad.email_provider = "gmail".to_string();
                identidad.tipo = tipo.to_string();
                identidad.estado = EstadoIdentidad::Creada.to_string();
                // El email temporal servirá como recovery
                identidad.recovery_email = Some(temp.address);
            }
            TipoCuenta::Proton => {
                let temp = self.providers.crear_temporal().await?;
                identidad.email = identidad.generar_email_proton();
                identidad.password = self.factory.generar_password(24);
                identidad.email_provider = "proton".to_string();
                identidad.tipo = tipo.to_string();
                identidad.estado = EstadoIdentidad::Creada.to_string();
                identidad.recovery_email = Some(temp.address);
            }
            TipoCuenta::Facebook | TipoCuenta::Twitter => {
                let temp = self.providers.crear_temporal().await?;
                identidad.email = temp.address;
                identidad.password = temp.password;
                identidad.email_provider = "mail.tm".to_string();
                identidad.tipo = tipo.to_string();
                identidad.estado = EstadoIdentidad::Creada.to_string();
            }
            TipoCuenta::Sintetico => {
                identidad.email = format!("{}@local.nexus", &identidad.nombre.to_lowercase());
                identidad.password = self.factory.generar_password(16);
                identidad.email_provider = "local".to_string();
                identidad.tipo = tipo.to_string();
                identidad.estado = EstadoIdentidad::Activa.to_string();
            }
        }

        // Registrar en base de datos
        self.db.registrar_identidad(
            &identidad.email,
            &identidad.password,
            identidad.recovery_email.as_deref(),
            &identidad.estado,
            identidad.email_provider.as_str(),
            &serde_json::to_value(&identidad)?,
        )?;

        info!(
            "🧬 [SEMBRADOR] Identidad sembrada: {} <{}> [{}]",
            identidad.nombre_completo(),
            identidad.email,
            tipo
        );

        // Cifrar en vault
        if let Err(e) = self.vault.guardar_credencial(&identidad.email, &identidad.password) {
            error!("❌ [VAULT] Error al cifrar credencial: {}", e);
        }

        Ok(identidad)
    }

    /// Lista todas las identidades registradas
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

    /// Obtiene reporte estadístico
    pub fn reporte(&self) -> anyhow::Result<serde_json::Value> {
        self.db.reporte_estadistico()
    }
}

impl std::fmt::Debug for SembradorOmega {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SembradorOmega").finish()
    }
}
