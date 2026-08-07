// =====================================================================
// NEXUS RESCATE — Binario de Emergencia para Descifrado de Glosolalia
// =====================================================================
// Propósito: Herramienta autónoma de recuperación de secretos cifrados
// con AES-256-GCM sin depender de ninguna librería del sistema operativo.
// Compilar con musl para binario 100% estático:
//   cargo build --release --target x86_64-unknown-linux-musl --bin nexus_rescate
// =====================================================================

use nexus_ultimate_core::comms::glosolalia::MatrizGlosolalia;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("🔱 [NEXUS RESCATE] Descifrador soberano de Glosolalia");
        eprintln!("Uso:   nexus_rescate <archivo.onion>");
        eprintln!("       nexus_rescate --verify <archivo.onion>");
        eprintln!();
        eprintln!("Modos:");
        eprintln!("  <archivo.onion>           Descifrar y mostrar el contenido en stdout");
        eprintln!("  --verify <archivo.onion>  Verificar integridad sin mostrar el secreto");
        process::exit(1);
    }

    let (verificar, ruta) = if args[1] == "--verify" {
        if args.len() < 3 {
            eprintln!("❌ [RESCATE] Especifica un archivo para verificar.");
            process::exit(1);
        }
        (true, args[2].as_str())
    } else {
        (false, args[1].as_str())
    };

    // Leer archivo binario cifrado
    let datos = match std::fs::read(ruta) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("❌ [RESCATE] No se pudo leer el archivo '{}': {}", ruta, e);
            process::exit(2);
        }
    };

    // Instanciar el módulo de Glosolalia con la llave maestra
    let matriz = MatrizGlosolalia::new();

    // Intentar pelar la cebolla
    match matriz.pelar_cebolla(&datos) {
        Ok((vector, secreto)) => {
            if verificar {
                println!("✅ [RESCATE] Integridad verificada.");
                println!("   Archivo : {}", ruta);
                println!("   Tamaño  : {} bytes", datos.len());
                println!("   Vector  : {:?}", vector);
                println!("   Contenido validado (AES-GCM tag OK) — no mostrado en modo --verify.");
            } else {
                println!("✅ [RESCATE OK] Vector superficial: {:?}", vector);
                println!("───────────────────────────────────────────────────────────");
                println!("{}", secreto);
                println!("───────────────────────────────────────────────────────────");
            }
        }
        Err(e) => {
            eprintln!("❌ [RESCATE FALLIDO] {}", e);
            eprintln!("   Posibles causas:");
            eprintln!("   1. La llave maestra no coincide con la usada al cifrar.");
            eprintln!("   2. El archivo está corrupto o no fue generado por la Glosolalia.");
            eprintln!("   3. El nonce fue reutilizado (violación del protocolo).");
            process::exit(3);
        }
    }
}
