use anyhow::Result;
use dotenv::dotenv;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use nexus_ultimate_core::identities::browser_profile::BrowserProfileManager;
use nexus_ultimate_core::identities::chrome_planter::ChromePlanter;
use nexus_ultimate_core::identities::types::SyntheticIdentity;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Configurar logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("🔱 NEXUS API HARVESTER — Cosechador de Células Energéticas");
        println!("Uso:");
        println!("  nexus_harvester <email> <password>     # Cosechar de cuenta existente");
        println!("  nexus_harvester --generate             # Sembrar y cosechar cuenta nueva");
        return Ok(());
    }

    let browser_mgr = BrowserProfileManager::new(std::path::PathBuf::from("data/profiles"));
    let planter = ChromePlanter::new(browser_mgr);

    if args[1] == "--generate" {
        info!("🌱 Iniciando SIEMBRA de nueva identidad sintética (Prioridad: PROTON)...");

        let profile = nexus_ultimate_core::identities::types::IdentityProfile {
            full_name: "Gabriel Omega".to_string(),
            gender: "M".to_string(),
            age: 33,
            nationality: "Paraguayo".to_string(),
            city: "Asunción".to_string(),
            country: "Paraguay".to_string(),
            ..Default::default()
        };
        let mut identity = SyntheticIdentity::new(profile);
        let password = format!(
            "Omega_{}_2026",
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        );

        info!("🌐 Intentando crear cuenta de Proton Mail (Soberanía Total)...");
        let plant_res = planter
            .crear_cuenta_proton("Gabriel", "Omega", &password, None, &identity)
            .await;

        if plant_res.success {
            let email = plant_res.email.unwrap_or_default();
            info!("✅ Cuenta Proton Creada: {}", email);

            identity
                .emails
                .push(nexus_ultimate_core::identities::types::EmailAccount {
                    address: email.clone(),
                    password: password.clone(),
                    provider: nexus_ultimate_core::identities::types::EmailProvider::ProtonMail,
                    verified: true,
                });

            // Proton Mail puede usarse para registrarse en otros servicios o como identidad
            info!("🧬 Identidad generada y lista para operaciones.");
        } else {
            error!("❌ Fallo en la siembra de Proton: {:?}", plant_res.error);
        }
        return Ok(());
    }

    if args.len() < 3 {
        error!("❌ Error: Faltan credenciales (email y password)");
        return Ok(());
    }

    let email = &args[1];
    let password = &args[2];
    let mut identity = SyntheticIdentity::new(Default::default());
    identity
        .emails
        .push(nexus_ultimate_core::identities::types::EmailAccount {
            address: email.clone(),
            password: password.clone(),
            provider: nexus_ultimate_core::identities::types::EmailProvider::Gmail,
            verified: true,
        });

    info!("🚀 Iniciando proceso de cosecha para: {}", email);

    // Login
    info!("🔑 Iniciando sesión en Google...");
    let login_res = planter.login_gmail(email, password, Some(&identity)).await;

    if !login_res.success {
        error!("❌ Fallo en el login: {:?}", login_res.error);
        return Ok(());
    }

    // Cosechar API Key
    info!("💎 Cosechando API Key de Gemini...");
    match planter.cosechar_api_key_gemini(&identity).await {
        Ok(api_key) => {
            info!("✅ ÉXITO: API Key obtenida: {}", api_key);
            match inyectar_llave_en_env(&api_key) {
                Ok(_) => info!("💉 Llave inyectada en .env y lista para Zenith Pool."),
                Err(e) => error!("⚠️ No se pudo inyectar automáticamente: {}", e),
            }
        }
        Err(e) => {
            error!("❌ Error durante la cosecha: {}", e);
        }
    }

    Ok(())
}

fn inyectar_llave_en_env(key: &str) -> Result<()> {
    let env_path = "C:/Users/crisp/NEXUS_ULTIMATE_CORE/.env";
    let content = std::fs::read_to_string(env_path)?;

    if content.contains("GEMINI_ACCOUNT_3_KEYS") {
        let new_content = content.replace(
            "GEMINI_ACCOUNT_3_KEYS=\"",
            &format!("GEMINI_ACCOUNT_3_KEYS=\"{},", key),
        );
        std::fs::write(env_path, new_content)?;
    } else {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new().append(true).open(env_path)?;
        writeln!(
            file,
            "\n# Inyectada por NEXUS Harvester\nGEMINI_ACCOUNT_10_KEYS=\"{}\"",
            key
        )?;
    }
    Ok(())
}
