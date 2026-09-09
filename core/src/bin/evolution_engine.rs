use anyhow::Result;
use nexus_ultimate_core::evolution::EvolutionEngine;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧬 [EVOLUTION ENGINE] Iniciando ciclo de auto-sanación...");

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://nexus_admin:nexus_pass@localhost:5432/nexus_core".to_string()
    });

    // El motor de evolución necesita acceso al núcleo
    let engine = EvolutionEngine::new(&db_url, None);

    println!("[+] Escaneando vulnerabilidades de código y advertencias...");
    let fixed = engine.apply_optimizations("nexus_ultimate_core").await?;

    if fixed > 0 {
        println!(
            "✅ [EVOLUTION] {} optimizaciones aplicadas y persistidas.",
            fixed
        );
    } else {
        println!("✨ [EVOLUTION] Código prístino. No se requieren optimizaciones.");
    }

    Ok(())
}
