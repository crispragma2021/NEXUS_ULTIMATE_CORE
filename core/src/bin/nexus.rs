// 🔱 NEXUS OMEGA — CLI Unificado (Orquestador Soberano)
// ============================================================
// Uso: nexus <subcomando>
//
// Subcomandos:
//   planter   — Sembrador de identidades sintéticas (FASE 1)
//   tor       — Control de Tor (verificar/reiniciar)
//   proxy     — Gestión de proxies (lista/test)
//   audit     — Auditoría de seguridad del sistema

use clap::{Parser, Subcommand};
use std::path::PathBuf;

// ── CLI Principal ──────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "nexus",
    version = "1.0.0",
    about = "🔱 NEXUS OMEGA — CLI Unificado del Orquestador Soberano"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 🌱 Sembrador de Identidades Sintéticas
    Planter {
        /// Acción: generate | list | activate | destroy | report | mail | plant-gmail
        #[clap(default_value = "generate")]
        action: String,

        /// Número de identidades a generar (solo para generate)
        #[clap(short = 'n', long = "count", default_value = "5")]
        count: usize,

        /// Filtrar por estado (list): pool | active | dormant | expired | destroyed
        #[clap(short = 's', long = "status")]
        status: Option<String>,

        /// ID de operación (activate)
        #[clap(short = 'o', long = "operation")]
        operation: Option<String>,

        /// ID de identidad (para activate/destroy)
        #[clap(short = 'i', long = "identity")]
        identity_id: Option<String>,

        /// Usar perfiles offline (sin Mistral API)
        #[clap(long = "offline")]
        offline: bool,
    },

    /// 📱 SMS Activate — Números virtuales para verificación
    Sms {
        /// Acción: balance | available | get | status | release | wait
        #[clap(default_value = "balance")]
        action: String,

        /// Servicio: google | telegram | whatsapp | facebook | twitter | instagram | outlook
        #[clap(short = 's', long = "service", default_value = "google")]
        service: String,

        /// País: paraguay | argentina | brazil | mexico | usa | spain | etc.
        #[clap(short = 'c', long = "country", default_value = "paraguay")]
        country: String,

        /// ID de activación (para status/release/wait/get)
        #[clap(short = 'i', long = "activation")]
        activation_id: Option<String>,

        /// Timeout en segundos para wait
        #[clap(short = 't', long = "timeout", default_value = "120")]
        timeout: u64,
    },

    /// 🔄 Control de Tor
    Tor {
        /// Acción: status | restart | new_circuit
        #[clap(default_value = "status")]
        action: String,
    },

    /// 🌐 Gestión de Proxies
    Proxy {
        /// Acción: list | test | switch
        #[clap(default_value = "list")]
        action: String,

        /// URL del proxy a probar
        #[clap(short = 'u', long = "url")]
        url: Option<String>,
    },

    /// 🛡️ Auditoría de Seguridad
    Audit {
        /// Tipo de auditoría: full | quick | network
        #[clap(default_value = "quick")]
        audit_type: String,
    },
}

// ── Main ───────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Planter {
            action,
            count,
            status,
            operation,
            identity_id,
            offline,
        } => cmd_planter(action, count, status, operation, identity_id, offline).await?,

        Commands::Sms {
            action,
            service,
            country,
            activation_id,
            timeout,
        } => cmd_sms(action, service, country, activation_id, timeout).await?,

        Commands::Tor { action } => cmd_tor(action).await?,

        Commands::Proxy { action, url } => cmd_proxy(action, url).await?,

        Commands::Audit { audit_type } => cmd_audit(audit_type).await?,
    }

    Ok(())
}

// ── Planter ────────────────────────────────────────────────────

async fn cmd_planter(
    action: String,
    count: usize,
    status: Option<String>,
    operation: Option<String>,
    identity_id: Option<String>,
    offline: bool,
) -> anyhow::Result<()> {
    // Ruta de la DB
    let data_dir =
        PathBuf::from(std::env::var("NEXUS_DATA_DIR").unwrap_or_else(|_| "data".to_string()));
    let db_path = data_dir.join("identities.db");

    match action.as_str() {
        "generate" => {
            println!(
                "🌱 Sembrador de Identidades — Generando {} identidades...",
                count
            );
            let generator = nexus_ultimate_core::identities::IdentityGenerator::new();

            let identities = if offline {
                println!("📡 Usando modo offline (perfiles precargados)");
                generator.generate_offline(count)
            } else {
                generator.generate(count).await?
            };

            // Guardar en DB
            let store = nexus_ultimate_core::identities::IdentityStore::open(&db_path)?;
            for identity in &identities {
                store.save_identity(identity)?;
                println!(
                    "  ✅ {} [{}]",
                    identity.short_summary(),
                    &identity.id.to_string()[..8]
                );
            }

            println!(
                "\n📊 {} identidades generadas y almacenadas.",
                identities.len()
            );
        }

        "list" => {
            let store = nexus_ultimate_core::identities::IdentityStore::open(&db_path)?;
            let status_filter = status.as_deref().and_then(|s| match s {
                "pool" => Some(nexus_ultimate_core::identities::IdentityStatus::Pool),
                "active" => Some(nexus_ultimate_core::identities::IdentityStatus::Active),
                "dormant" => Some(nexus_ultimate_core::identities::IdentityStatus::Dormant),
                "expired" => Some(nexus_ultimate_core::identities::IdentityStatus::Expired),
                "destroyed" => Some(nexus_ultimate_core::identities::IdentityStatus::Destroyed),
                _ => None,
            });

            let identities = store.list_identities(status_filter)?;
            println!("📋 Identidades ({})", identities.len());
            for (i, id) in identities.iter().enumerate() {
                println!("  {}. {}", i + 1, id.short_summary());
            }
        }

        "activate" => {
            let id = identity_id.ok_or_else(|| anyhow::anyhow!("Se requiere --identity <UUID>"))?;
            let op = operation.unwrap_or_else(|| "default".to_string());
            let store = nexus_ultimate_core::identities::IdentityStore::open(&db_path)?;
            let mut all = store.list_identities(None)?;
            let identity = all
                .iter_mut()
                .find(|i| i.id.to_string().starts_with(&id))
                .ok_or_else(|| anyhow::anyhow!("Identidad no encontrada: {}", id))?;

            let rotator = nexus_ultimate_core::identities::IdentityRotator::new(store);
            rotator.activate(identity, &op)?;
            println!("✅ Identidad {} activada para operación '{}'", id, op);
        }

        "destroy" => {
            let id = identity_id.ok_or_else(|| anyhow::anyhow!("Se requiere --identity <UUID>"))?;
            let store = nexus_ultimate_core::identities::IdentityStore::open(&db_path)?;
            let browser_mgr = nexus_ultimate_core::identities::BrowserProfileManager::new(
                data_dir.join("browser_profiles"),
            );
            let destroyer =
                nexus_ultimate_core::identities::IdentityDestroyer::new(store, browser_mgr);

            let store2 = nexus_ultimate_core::identities::IdentityStore::open(&db_path)?;
            let all = store2.list_identities(None)?;
            let identity = all
                .into_iter()
                .find(|i| i.id.to_string().starts_with(&id))
                .ok_or_else(|| anyhow::anyhow!("Identidad no encontrada: {}", id))?;

            destroyer.destroy(identity)?;
        }

        "report" => {
            let store = nexus_ultimate_core::identities::IdentityStore::open(&db_path)?;
            let rotator = nexus_ultimate_core::identities::IdentityRotator::new(store);
            let report = rotator.pool_report()?;
            println!("{}", report);
        }

        "mail" => {
            let store = nexus_ultimate_core::identities::IdentityStore::open(&db_path)?;
            let all = store.list_identities(None)?;
            if all.is_empty() {
                println!(
                    "⚠️  No hay identidades. Genera algunas primero con `nexus planter generate`"
                );
                return Ok(());
            }

            let mail_factory = nexus_ultimate_core::identities::MailFactory::new();
            for identity in &all {
                println!("📧 Creando correo para {}...", identity.profile.full_name);
                match mail_factory.create_for_identity(identity).await {
                    Ok(email) => {
                        println!("  ✅ {} / {}", email.address, email.password);
                    }
                    Err(e) => {
                        eprintln!("  ⚠️  Error: {}", e);
                    }
                }
            }
        }

        "plant-gmail" => {
            println!(
                "🌱 Sembrador de Identidades — Iniciando flujo interactivo de registro Gmail..."
            );
            let store = nexus_ultimate_core::identities::IdentityStore::open(&db_path)?;
            let browser_mgr = nexus_ultimate_core::identities::BrowserProfileManager::new(
                data_dir.join("browser_profiles"),
            );
            let planter = nexus_ultimate_core::identities::ChromePlanter::new(browser_mgr);

            // Generar identidad sintética base offline
            let generator = nexus_ultimate_core::identities::IdentityGenerator::new();
            let mut identities = generator.generate_offline(1);
            let mut identity = identities.remove(0);

            let parts: Vec<&str> = identity.profile.full_name.split_whitespace().collect();
            let nombre = parts.first().copied().unwrap_or("Carlos");
            let apellido = parts.get(1).copied().unwrap_or("Mendoza");

            use rand::Rng;
            let password = format!("Nexus{}!", rand::thread_rng().gen_range(1000..9999));

            println!("📝 Creando cuenta para: {} {}", nombre, apellido);
            println!("🔑 Contraseña propuesta: {}", password);

            let result = planter
                .crear_cuenta_gmail(nombre, apellido, &password, None, &identity)
                .await;

            if result.success {
                if let Some(ref email_real) = result.email {
                    println!("\n🎉 [ÉXITO] Cuenta Gmail registrada: {}", email_real);

                    // Asignar el correo a la identidad sintética
                    identity
                        .emails
                        .push(nexus_ultimate_core::identities::EmailAccount {
                            address: email_real.clone(),
                            password: password.clone(),
                            provider: nexus_ultimate_core::identities::EmailProvider::Gmail,
                            verified: true,
                        });

                    // Guardar identidad en la DB
                    store.save_identity(&identity)?;
                    println!(
                        "💾 Identidad guardada en la base de datos con UUID: {}",
                        identity.id
                    );
                }
            } else {
                eprintln!(
                    "\n❌ [ERROR] No se pudo crear la cuenta Gmail: {:?}",
                    result.error
                );
            }
        }

        _ => {
            eprintln!("❌ Acción desconocida: {}. Usa: generate | list | activate | destroy | report | mail | plant-gmail", action);
        }
    }

    Ok(())
}

// ── SMS Activate ──────────────────────────────────────────────

async fn cmd_sms(
    action: String,
    service: String,
    country: String,
    activation_id: Option<String>,
    timeout: u64,
) -> anyhow::Result<()> {
    let client = match nexus_ultimate_core::identities::SmsActivateClient::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ {}. Configura SMS_ACTIVATE_API_KEY en .env", e);
            eprintln!("   Obtén tu API key en https://sms-activate.org");
            return Ok(());
        }
    };

    match action.as_str() {
        "balance" => match client.get_balance().await {
            Ok(balance) => println!("💰 Saldo SMS Activate: ${:.2}", balance),
            Err(e) => eprintln!("❌ Error: {}", e),
        },

        "available" => {
            let parsed_country =
                nexus_ultimate_core::identities::sms_activate::SmsCountry::from_name(&country);
            match client.get_numbers_status(parsed_country).await {
                Ok(statuses) => {
                    println!("📊 Números disponibles:");
                    if statuses.is_empty() {
                        println!("  (ninguno)");
                    }
                    let mut sorted: Vec<_> = statuses.into_iter().collect();
                    sorted.sort_by(|a, b| b.1.cmp(&a.1));
                    for (key, count) in sorted.iter().take(20) {
                        if *count > 0 {
                            println!("  ✅ {}: {} números", key, count);
                        }
                    }
                }
                Err(e) => eprintln!("❌ Error: {}", e),
            }
        }

        "get" => {
            let parsed_service =
                nexus_ultimate_core::identities::sms_activate::SmsService::from_name(&service)
                    .unwrap_or(nexus_ultimate_core::identities::sms_activate::SmsService::Google);
            let parsed_country =
                nexus_ultimate_core::identities::sms_activate::SmsCountry::from_name(&country)
                    .unwrap_or(nexus_ultimate_core::identities::sms_activate::SmsCountry::Paraguay);

            match client.get_number(parsed_service, parsed_country).await {
                Ok(activation) => {
                    println!("✅ Número obtenido!");
                    println!("   ID Activación: {}", activation.activation_id);
                    println!("   Número:        {}", activation.phone_display());
                    println!("   Servicio:      {}", activation.service);
                }
                Err(e) => eprintln!("❌ Error: {}", e),
            }
        }

        "status" => {
            let id =
                activation_id.ok_or_else(|| anyhow::anyhow!("Se requiere --activation <ID>"))?;
            match client.get_status(&id).await {
                Ok(status) => {
                    let msg = match &status {
                        nexus_ultimate_core::identities::sms_activate::ActivationStatus::Pending =>
                            "⏳ Esperando SMS...".to_string(),
                        nexus_ultimate_core::identities::sms_activate::ActivationStatus::SmsReceived(code) =>
                            format!("✅ SMS recibido: {}", code),
                        nexus_ultimate_core::identities::sms_activate::ActivationStatus::Canceled =>
                            "❌ Cancelada".to_string(),
                        nexus_ultimate_core::identities::sms_activate::ActivationStatus::Finished =>
                            "✅ Finalizada".to_string(),
                    };
                    println!("{}", msg);
                }
                Err(e) => eprintln!("❌ Error: {}", e),
            }
        }

        "wait" => {
            let id =
                activation_id.ok_or_else(|| anyhow::anyhow!("Se requiere --activation <ID>"))?;
            println!("⏳ Esperando SMS (timeout: {}s)...", timeout);
            match client.wait_for_sms(&id, timeout).await {
                Ok(code) => println!("✅ Código SMS recibido: {}", code),
                Err(e) => eprintln!("❌ {}", e),
            }
        }

        "release" => {
            let id =
                activation_id.ok_or_else(|| anyhow::anyhow!("Se requiere --activation <ID>"))?;
            match client.release_number(&id).await {
                Ok(_) => println!("✅ Número liberado: {}", id),
                Err(e) => eprintln!("❌ Error: {}", e),
            }
        }

        _ => {
            eprintln!(
                "❌ Acción desconocida. Usa: balance | available | get | status | wait | release"
            );
        }
    }

    Ok(())
}

// ── Tor ────────────────────────────────────────────────────────

async fn cmd_tor(action: String) -> anyhow::Result<()> {
    match action.as_str() {
        "status" => {
            println!("🔍 Verificando estado de Tor...");
            match tokio::net::TcpStream::connect("127.0.0.1:9050").await {
                Ok(_) => println!("✅ Tor SOCKS5 proxy activo en localhost:9050"),
                Err(e) => println!("❌ Tor NO disponible: {}", e),
            }
        }
        "restart" => {
            println!("🔄 Reiniciando Tor...");
            let output = std::process::Command::new("systemctl")
                .args(["restart", "tor"])
                .output()?;
            if output.status.success() {
                println!("✅ Tor reiniciado correctamente");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("❌ Error al reiniciar Tor: {}", stderr);
            }
        }
        "new_circuit" => {
            println!("🔄 Solicitando nuevo circuito Tor...");
            // Usar señal SIGHUP a Tor o puerto de control
            let output = std::process::Command::new("tor")
                .args(["--control-port", "9051"])
                .arg("--new-circuit")
                .output()?;
            if output.status.success() {
                println!("✅ Nuevo circuito solicitado");
            } else {
                // Fallback: reiniciar Tor
                println!("⚠️  No se pudo enviar señal. Intentando restart...");
                std::process::Command::new("systemctl")
                    .args(["restart", "tor"])
                    .output()?;
            }
        }
        _ => {
            eprintln!("❌ Acción desconocida. Usa: status | restart | new_circuit");
        }
    }
    Ok(())
}

// ── Proxy ──────────────────────────────────────────────────────

async fn cmd_proxy(action: String, _url: Option<String>) -> anyhow::Result<()> {
    match action.as_str() {
        "list" => {
            println!("🌐 Listando proxies disponibles...");
            // Lista de proxies conocidos del sistema
            let proxies = vec![("Tor (SOCKS5)", "socks5://127.0.0.1:9050")];
            for (name, addr) in &proxies {
                let status = tokio::net::TcpStream::connect(addr.trim_start_matches("socks5://"))
                    .await
                    .is_ok();
                println!("  {} {} — {}", if status { "✅" } else { "❌" }, name, addr);
            }
        }
        "test" => {
            println!("🌐 Testeando conectividad de proxy...");
            let client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all("socks5://127.0.0.1:9050")?)
                .timeout(std::time::Duration::from_secs(10))
                .build()?;
            match client
                .get("https://check.torproject.org/api/ip")
                .send()
                .await
            {
                Ok(resp) => {
                    let body = resp.text().await.unwrap_or_default();
                    println!("✅ Proxy responde: {}", body);
                }
                Err(e) => println!("❌ Proxy falló: {}", e),
            }
        }
        _ => {
            eprintln!("❌ Acción desconocida. Usa: list | test");
        }
    }
    Ok(())
}

// ── Audit ──────────────────────────────────────────────────────

async fn cmd_audit(audit_type: String) -> anyhow::Result<()> {
    match audit_type.as_str() {
        "full" => {
            println!("🛡️ Auditoría completa de seguridad...");
            let report = audit_security().await?;
            println!("{}", report);
            // Guardar reporte
            let report_path = PathBuf::from("reports/audit/nexus_audit_report.txt");
            if let Some(parent) = report_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&report_path, &report).await?;
            println!("\n📄 Reporte guardado en: {}", report_path.display());
        }
        "quick" | _ => {
            println!("🛡️ Auditoría rápida...");
            println!("{}", audit_quick().await?);
        }
    }
    Ok(())
}

async fn audit_quick() -> anyhow::Result<String> {
    let mut report = String::new();
    report.push_str("╔══════════════════════════════════════╗\n");
    report.push_str("║   🛡️ NEXUS AUDIT — RÁPIDA           ║\n");
    report.push_str("╚══════════════════════════════════════╝\n\n");

    // Tor
    match tokio::net::TcpStream::connect("127.0.0.1:9050").await {
        Ok(_) => report.push_str("✅ Tor SOCKS5: Activo (localhost:9050)\n"),
        Err(_) => report.push_str("❌ Tor SOCKS5: No disponible\n"),
    }

    // Disco
    let disk_usage = std::fs::read_to_string("/proc/diskstats").unwrap_or_default();
    let disk_lines = disk_usage.lines().count();
    report.push_str(&format!("💾 Disco: {} líneas de stats\n", disk_lines));

    // Memoria
    let mem_info = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    if let Some(line) = mem_info.lines().find(|l| l.starts_with("MemAvailable:")) {
        report.push_str(&format!("🧠 Memoria disponible: {}\n", line.trim()));
    }

    // Tiempo del sistema
    let uptime = std::fs::read_to_string("/proc/uptime").unwrap_or_default();
    let uptime_secs: f64 = uptime
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let uptime_hours = uptime_secs / 3600.0;
    report.push_str(&format!("⏱️  Uptime: {:.1} horas\n", uptime_hours));

    Ok(report)
}

async fn audit_security() -> anyhow::Result<String> {
    let mut report = String::new();
    report.push_str("╔══════════════════════════════════════╗\n");
    report.push_str("║   🛡️ NEXUS AUDIT — COMPLETA         ║\n");
    report.push_str("╚══════════════════════════════════════╝\n\n");

    // 1. Tor
    report.push_str("── 🔄 Tor ──\n");
    match tokio::net::TcpStream::connect("127.0.0.1:9050").await {
        Ok(_) => report.push_str("  ✅ SOCKS5: localhost:9050 — Activo\n"),
        Err(e) => report.push_str(&format!("  ❌ SOCKS5: localhost:9050 — {}\n", e)),
    }

    // 2. Procesos sensibles
    report.push_str("\n── 📋 Procesos Sensibles ──\n");
    for proc_name in &["tor", "ssh", "docker", "nginx", "apache2"] {
        let output = std::process::Command::new("pgrep")
            .args(["-x", proc_name])
            .output()?;
        if output.status.success() {
            report.push_str(&format!("  ✅ {}: Activo\n", proc_name));
        } else {
            report.push_str(&format!("  ⚪ {}: No detectado\n", proc_name));
        }
    }

    // 3. Puertos críticos
    report.push_str("\n── 🔌 Puertos ──\n");
    let ports_to_check = [9050, 22, 80, 443, 3000, 8080];
    for port in &ports_to_check {
        if let Ok(_) = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await {
            report.push_str(&format!("  🔴 Puerto {}: Abierto\n", port))
        }
    }

    // 4. Espacio en disco
    report.push_str("\n── 💾 Disco ──\n");
    match std::fs::read_to_string("/proc/meminfo") {
        Ok(info) => {
            for line in info.lines().take(5) {
                report.push_str(&format!("  {}\n", line.trim()));
            }
        }
        Err(e) => report.push_str(&format!("  ❌ Error: {}\n", e)),
    }

    // 5. Conexiones de red (netstat simplificado)
    report.push_str("\n── 🌐 Conexiones de Red ──\n");
    let netstat = std::process::Command::new("ss").args(["-tlnp"]).output()?;
    let output_str = String::from_utf8_lossy(&netstat.stdout);
    for line in output_str.lines().skip(1).take(20) {
        report.push_str(&format!("  {}\n", line));
    }

    Ok(report)
}
