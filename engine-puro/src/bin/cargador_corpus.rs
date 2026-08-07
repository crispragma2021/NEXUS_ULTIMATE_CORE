// ============================================================================
// 📚 CARGADOR DE CORPUS — Herramienta de Carga de Conocimiento Primario
// ============================================================================
// Uso: cargo run --bin cargador-corpus
//
// Educa al engine-puro inyectando ~500 frases de alta calidad en 4 dominios
// (conversación, ciencia, tecnología, filosofía) usando el CargadorConocimiento.
//
// Sin LLM. Solo STDP + Markov 4º orden + importancia gramatical.
// ============================================================================

use std::time::Instant;
use cerebro_digital::cerebro::cerebro::CerebroAutoOptimizable;
use cerebro_digital::cerebro::estructuras::Entrada;

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║   📚 CARGADOR DE CORPUS SEMILLA — Conocimiento Primario ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // 1. Inicializar cerebro (carga automática de estado previo)
    println!("🧠 Inicializando cerebro...");
    let inicio = Instant::now();
    let mut cerebro = CerebroAutoOptimizable::nuevo();
    let init_ms = inicio.elapsed().as_millis();
    println!("   ✅ Cerebro listo en {} ms", init_ms);
    println!("   🧬 Neuronas iniciales: {}", {
        let (vram, ram, total, _) = cerebro.memoria.estadisticas();
        format!("{} (VRAM: {}, RAM: {})", total, vram, ram)
    });
    println!("   🔤 Tokens en léxico: {}", cerebro.motor_sensorial.total_embeddings());
    println!();

    // 2. Cargar corpus semilla
    println!("📚 Cargando corpus semilla de conocimiento primario...");
    let carga_inicio = Instant::now();
    let stats = cerebro.cargar_corpus_semilla();
    let carga_ms = carga_inicio.elapsed().as_millis();
    println!();

    // 3. Mostrar resultado
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║   📊 ESTADÍSTICAS FINALES DE CARGA                     ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    println!("   Frases procesadas    : {}", stats.frases_procesadas);
    println!("   Tokens nuevos        : {}", stats.tokens_aprendidos);
    println!("   Conexiones neurales  : {}", stats.neuronas_conectadas);
    println!("   Tiempo de carga      : {} ms", stats.tiempo_total_ms);
    println!("   Rendimiento          : {:.1} frases/s", stats.frases_por_segundo);
    println!();
    println!("   ⏱️  Tiempo total       : {} ms", carga_ms);
    println!();

    // 4. Mostrar estado del cerebro post-carga
    println!("📋 Estado del cerebro tras la carga:");
    println!("   🔤 Tokens totales en léxico: {}", cerebro.motor_sensorial.total_embeddings());
    let (vram, ram, total, episodios) = cerebro.memoria.estadisticas();
    println!("   🧬 Neuronas: {} (VRAM: {}, RAM: {})", total, vram, ram);
    println!("   💾 Episodios en SSD: {}", episodios);
    println!("   🎭 Emoción: {}", cerebro.motores.amigdala.emocion_dominante());
    println!();

    // 5. Prueba de generación (opcional)
    println!("🧪 Generando respuesta de prueba...");
    let entrada = Entrada {
        estimulos: vec![],
        texto: Some("hola como estas".to_string()),
        recompensa: 0.0,
        amenaza: 0.0,
    };
    let salida = cerebro.paso(0.01, entrada);
    println!("   💬 Salida: \"{}\"", salida.texto);
    println!("   🎭 Emoción: {:.2}", salida.emocion);
    println!("   🧠 Conciencia: {:.2}", salida.conciencia);
    println!();

    // 6. Guardar estado final
    println!("💾 Persistiendo cerebro a disco...");
    match cerebro.guardar_a_disco() {
        Ok(_) => println!("   ✅ Estado guardado exitosamente"),
        Err(e) => eprintln!("   ⚠️  Error al guardar: {}", e),
    }
    println!();

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║   ✅ CARGA DE CONOCIMIENTO PRIMARIO COMPLETADA          ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    println!("Próximos pasos:");
    println!("  • Ejecutar: cargo run --bin cerebro-digital  (para interactuar con el engine educado)");
    println!("  • Repetir carga: cargo run --bin cargador-corpus");
    println!();
}
