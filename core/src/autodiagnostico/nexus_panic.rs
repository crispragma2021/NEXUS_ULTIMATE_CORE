use std::fs;
use std::process::Command;

pub fn engage_panic_defense(target: &str) {
    if target.contains(".") {
        // Es una IP: Bloqueo de Red
        println!("🚨 NEXUS PÁNICO: Bloqueando IP externa: {}", target);
        let _ = Command::new("sudo")
            .arg("iptables")
            .arg("-A")
            .arg("INPUT")
            .arg("-s")
            .arg(target)
            .arg("-j")
            .arg("DROP")
            .spawn();
    } else {
        // Es una amenaza interna: Cuarentena de Archivos
        println!(
            "☣️ NEXUS PÁNICO: Amenaza interna detectada: {}. Aplicando Cuarentena.",
            target
        );
        let _ = fs::create_dir_all("/home/soberano/NEXUS_ULTIMATE_CORE/archive/quarantine");
        // Aquí moveríamos el archivo sospechoso si tuviéramos la ruta exacta
    }

    let msg = format!("🚨 NEXUS PÁNICO: Defensa activa contra {}.", target);
    pollster::block_on(crate::nexus_telegram::send_alert(&msg));
}
