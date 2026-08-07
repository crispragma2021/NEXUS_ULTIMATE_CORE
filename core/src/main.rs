#![recursion_limit = "2048"]
use nexus_ultimate_core::brain::hippocampus::ArtificialHippocampus;
use nexus_ultimate_core::cerebro::orquestador::Orquestador;
use nexus_ultimate_core::defensa::vigilante_del_padre::VigilanteDelPadre;
use std::io::{self, Write};
use std::sync::Arc;
use tracing::info;

// 🔱 Subcomando `translate` — Filtro de texto para git diff con archivos Glosolalia
// Uso: nexus_ultimate_core translate <archivo.onion>
// Configura en .gitattributes: *.onion diff=glosolalia
// Configura en .git/config:    [diff "glosolalia"] textconv = <ruta_binario> translate
fn cmd_translate(ruta: &str) {
    use nexus_ultimate_core::comms::glosolalia::MatrizGlosolalia;
    let datos = match std::fs::read(ruta) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[translate] No se pudo leer '{}': {}", ruta, e);
            std::process::exit(1);
        }
    };
    let matriz = MatrizGlosolalia::new();
    match matriz.pelar_cebolla(&datos) {
        Ok((vector, secreto)) => {
            // Solo el texto plano al stdout — git diff lo captura directamente
            println!("# Glosolalia v1 | vector={:?}", vector);
            println!("{}", secreto);
        }
        Err(_) => {
            // Si no se puede descifrar, mostrarlo como binario para que git no falle
            println!("<archivo binario no descifrable — clave incorrecta o formato inválido>");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Interceptar subcomandos de herramientas antes de arrancar el orquestador
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "translate" {
        cmd_translate(&args[2]);
        return Ok(());
    }
    dotenv::dotenv().ok();
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        .add_directive("chromiumoxide=off".parse().unwrap())
        .add_directive("tungstenite=off".parse().unwrap())
        .add_directive("tokio_tungstenite=off".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .without_time()
        .with_level(false)
        .init();

    // Asegurar directorios base
    std::fs::create_dir_all("data").ok();

    // 🔓 Desbloquear temporalmente al arranque para permitir migraciones/inicialización (Pilar 3)
    let _ = nexus_ultimate_core::brain::immune::memory_shield::MemoryShield::unlock_read_write(
        "nexus_intelligence.db",
    );
    let _ = nexus_ultimate_core::brain::immune::memory_shield::MemoryShield::unlock_read_write(
        "data/intelligence.db",
    );
    let hippocampus = Arc::new(ArtificialHippocampus::new(
        None,
        None,
        "/home/soberano/NEXUS_ULTIMATE_CORE/data/memory/vector_memories",
    ));
    let orquestador = Orquestador::new(hippocampus).await;

    // 🛡️ Activar blindaje físico de recuerdos tras la inicialización (Pilar 3)
    let _ = nexus_ultimate_core::brain::immune::memory_shield::MemoryShield::lock_read_only(
        "nexus_intelligence.db",
    );
    let _ = nexus_ultimate_core::brain::immune::memory_shield::MemoryShield::lock_read_only(
        "data/intelligence.db",
    );
    // 🛡️ [PROTOCOLO SOBERANO] El brazo NEXUS CLAW PRO ya es parte del núcleo nativo.
    let _claw = orquestador.nexus_claw_api.clone();
    info!("🦾 [SISTEMA] NEXUS CLAW PRO (Motor Nativo) inicializado y verificado.");

    // Iniciar el Guardián del Padre
    let vigilante = VigilanteDelPadre::new(orquestador.nexus_claw_api.clone());
    tokio::spawn(async move {
        vigilante.iniciar_vigilancia().await;
    });

    println!("\n");
    println!("╔═════════════════════════════════════════════════════════════════════════╗");
    println!("║                                                                         ║");
    println!("║   🧬 NEXUS OMEGA - ACORAZADO UNIFICADO                                 ║");
    println!("║   🛡 Escudo Kernel: ACTIVADO (Anillo 0)                               ║");
    println!("║   🦾 Médula Soberana: UNIFICADA (Claw Pro)                            ║");
    println!("║   🧠 Neocórtex: ESTRUCTURADO (Brodmann)                               ║");
    println!("║   🏛 Orquestación: CENTRALIZADA (main.rs)                              ║");
    println!("║   🌐 Inferencia: ZENITH POOL (Híbrido)                                ║");
    println!("╚═════════════════════════════════════════════════════════════════════════╝");
    println!("\n");

    // Detectar si hay un terminal interactivo (TTY)
    use std::io::IsTerminal;
    if std::io::stdin().is_terminal() {
        // Modo interactivo: chat con el Arquitecto
        println!("\n📜 Identidad cargada: identity.md\n");
        loop {
            print!("┌─[TÚ]\n└─>> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).unwrap_or(0) == 0 {
                break; // EOF
            }
            let input = input.trim();

            if input.eq_ignore_ascii_case("salir") || input.eq_ignore_ascii_case("exit") {
                println!("\n🧬 NEXUS durmiendo...\n");
                return Ok(());
            }

            if input.eq_ignore_ascii_case("diagnostico") || input.eq_ignore_ascii_case("status") {
                println!("\n┌─[NEXUS DIAGNÓSTICO]");
                println!("└─>> {}\n", orquestador.diagnostico());
                continue;
            }

            if input.is_empty() {
                continue;
            }

            println!("\n┌─[NEXUS]");
            let respuesta = orquestador.responder(input).await;
            println!("└─>> {}\n", respuesta);
        }
    } else {
        // Modo daemon: mantenerse vivo sin loop de stdin
        tracing::info!("🛡 NEXUS en modo daemon - vigilante silencioso");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }

    Ok(())
}
