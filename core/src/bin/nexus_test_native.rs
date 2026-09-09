// 🧪 Binario de prueba para validar la inferencia nativa Candle.
// Carga el modelo GGUF de forma síncrona (diagnóstico explícito)
// y genera texto real midiendo la latencia.
use nexus_ultimate_core::energia::ia_nativa::CerebroNativo;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🧬 [TEST] Creando CerebroNativo...");
    let cerebro = CerebroNativo::new();

    // Validar el modelo soberano Qwen3-4B Q4_K_M (repo Qwen/Qwen3-4B-GGUF)
    let ruta = "C:/Users/crisp/NEXUS_ULTIMATE_CORE/brain/swarm/models/Qwen3-4B-Q4_K_M.gguf";
    println!("📥 [TEST] Asimilando modelo GGUF (Qwen3-4B)...");
    match cerebro.asimilar_pesos_con_seguridad(ruta).await {
        Ok(()) => println!("✅ [TEST] Modelo asimilado correctamente."),
        Err(e) => {
            eprintln!("❌ [TEST] Error al asimilar modelo: {:?}", e);
            std::process::exit(1);
        }
    }

    println!(
        "✅ [TEST] Modelo listo. Dispositivo: {}",
        cerebro.dispositivo_str()
    );

    // Medir latencia de inferencia real
    let prompt = "Responde en una frase: que es NEXUS";
    println!("🧠 [TEST] Prompt: {}", prompt);

    let start = std::time::Instant::now();
    match cerebro.generar_token_nativo(prompt).await {
        Ok(resp) => {
            let elapsed = start.elapsed();
            println!("⏱️  [TEST] Latencia: {:.2?}", elapsed);
            println!("💬 [TEST] Respuesta: {}", resp);
        }
        Err(e) => {
            eprintln!("❌ [TEST] Error en inferencia: {:?}", e);
            std::process::exit(1);
        }
    }
}
