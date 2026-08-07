// ============================================================================
// 🧠 CEREBRO DIGITAL DINÁMICO v2 — Consola Interactiva + Feedback Tutor
// ============================================================================
// Arquitectura biológica con Hodgkin-Huxley, STDP real,
// memoria jerárquica (VRAM/RAM/SSD), 8 motores biológicos,
// trigramas Markov, mapa topológico semántico y feedback del Orquestador.
// ============================================================================

use cerebro_digital::cerebro::cerebro::CerebroAutoOptimizable;
use cerebro_digital::cerebro::estructuras::{Entrada, Estimulo};
use cerebro_digital::cerebro::tutor_openrouter::{FreeModel, TutorOpenRouter};
use std::io::{self, Write};

fn main() {
    // ─── Cargar API key de OpenRouter ──────────────────────────────────────
    let openrouter_key = std::env::var("OPENROUTER_API_KEY")
        .unwrap_or_else(|_| "sk-or-v1-REDACTADO".to_string());
    let tutor = std::sync::Mutex::new(TutorOpenRouter::new(openrouter_key));

    println!("\n{}", "=".repeat(60));
    println!("  🧠 CEREBRO DIGITAL DINÁMICO — Hodgkin-Huxley + STDP + Trigramas");
    println!("{}", "=".repeat(60));
    println!("  Comandos:");
    println!("    /exit              Salir");
    println!("    /stats             Estadísticas del cerebro");
    println!("    /emotion           Estado emocional detallado");
    println!("    /paso [N]          Ejecutar N pasos en ráfaga");
    println!("    /reset             Reiniciar neuronas");
    println!("    /tutor <txt>       Feedback manual del tutor (LTP dopaminérgico)");
    println!("    /autotutor         Feedback automático vía OpenRouter FREE (no gasta saldo)");
    println!("    /tutor_model [N]   Cambiar modelo gratuito (1-5). Sin args: listar");
    println!("    /tutor_stats       Estadísticas del tutor OpenRouter");
    println!("    /lexico            Estadísticas del léxico sensorial aprendido");
    println!("    <texto>            Estímulo sensorial como texto");
    println!();

    let mut cerebro = CerebroAutoOptimizable::nuevo();

    loop {
        print!("🧠 > ");
        io::stdout().flush().unwrap_or(());

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match input {
            "/exit" | "/quit" => {
                println!("🧠 Apagando cerebro digital...");
                break;
            }

            "/stats" | "/s" => {
                cerebro.resumen();
                continue;
            }

            "/emotion" | "/e" => {
                let a = &cerebro.motores.amigdala;
                println!("  🎭 ESTADO EMOCIONAL");
                println!("  Miedo:    {:.3}", a.miedo);
                println!("  Ansiedad: {:.3}", a.ansiedad);
                println!("  Ira:      {:.3}", a.ira);
                println!("  Alegría:  {:.3}", a.alegria);
                println!("  Dominante: {}", a.emocion_dominante());
                println!("  Dopamina: {:.3}", cerebro.motores.dopamina.nivel);
                println!("  Conciencia: {:.3}", cerebro.motores.conciencia.intensidad);
                continue;
            }

            "/lexico" | "/l" => {
                let sensorial = &cerebro.motor_sensorial;
                println!("  📚 LÉXICO SENSORIAL");
                println!("  Tokens (embeddings): {}", sensorial.total_embeddings());
                continue;
            }

            "/reset" | "/r" => {
                cerebro = CerebroAutoOptimizable::nuevo();
                println!("  🧹 Cerebro reiniciado completamente");
                continue;
            }

            cmd if cmd.starts_with("/tutor ") => {
                let respuesta_tutor = cmd.strip_prefix("/tutor ").unwrap_or("").trim();
                if respuesta_tutor.is_empty() {
                    println!("  ⚠️  Uso: /tutor <texto de respuesta del tutor>");
                } else {
                    cerebro.paso_tutor(respuesta_tutor);
                    println!(
                        "  🎓 [TUTOR] Feedback aplicado. Dopamina: {:.3}",
                        cerebro.motores.dopamina.nivel
                    );
                }
                continue;
            }

            "/autotutor" | "/a" => {
                // Obtener la última respuesta del cerebro como texto
                let respuesta = cerebro.ultima_salida.texto.clone();
                if respuesta.is_empty() || respuesta == "[silence]" {
                    println!("  ⚠️  No hay respuesta previa del cerebro. Primero escribe algo.");
                    continue;
                }

                println!("  🎓 Consultando tutor OpenRouter FREE... (modelo: {})",
                    tutor.lock().map(|t| t.model_name().to_string()).unwrap_or_default());
                println!("  ⚡ (Esto puede tomar 5-15 segundos — es un LLM gratuito)");

                match tutor.lock() {
                    Ok(t) => match t.evaluar(&respuesta) {
                        Ok(feedback) => {
                            println!("\n  🎓 FEEDBACK DEL TUTOR:\n  ─────────────────────");
                            for line in feedback.lines() {
                                println!("  {}", line);
                            }
                            println!("  ─────────────────────");

                            // Aplicar feedback al cerebro como aprendizaje LTP
                            cerebro.paso_tutor(&feedback);
                            println!(
                                "  ✅ Aprendizaje aplicado. Dopamina: {:.3}",
                                cerebro.motores.dopamina.nivel
                            );
                        }
                        Err(e) => {
                            println!("  ❌ {e}");
                            if e.contains("rate limit") || e.contains("429") {
                                println!("  💡 Consejo: espera 1 minuto y reintenta.");
                            } else if e.contains("402") {
                                println!("  💡 Este modelo free puede estar caído. Prueba: /tutor_model");
                            }
                        }
                    },
                    Err(e) => println!("  ❌ Error interno del tutor: {e}"),
                }
                continue;
            }

            cmd if cmd.starts_with("/tutor_model") => {
                let arg = cmd.strip_prefix("/tutor_model").map(|s| s.trim()).unwrap_or("");
                if arg.is_empty() {
                    println!("  📋 MODELOS GRATUITOS DISPONIBLES:");
                    for (idx, name) in FreeModel::listar() {
                        println!("    [{idx}] {name}");
                    }
                    println!("  Uso: /tutor_model <número>");
                } else if let Ok(n) = arg.parse::<usize>() {
                    match FreeModel::por_indice(n) {
                        Some(model) => {
                            let name = model.as_str().to_string();
                            match tutor.lock() {
                                Ok(mut t) => {
                                    t.set_model(model);
                                    println!("  ✅ Modelo cambiado a: {name}");
                                }
                                Err(e) => println!("  ❌ Error: {e}"),
                            }
                        }
                        None => println!("  ⚠️  Modelo no válido. Usa 1-5."),
                    }
                } else {
                    println!("  ⚠️  Uso: /tutor_model <1-5>  (sin argumento: lista modelos)");
                }
                continue;
            }

            "/tutor_stats" | "/ts" => {
                match tutor.lock() {
                    Ok(t) => println!("{}", t.stats()),
                    Err(e) => println!("  ❌ Error: {e}"),
                }
                continue;
            }

            cmd if cmd.starts_with("/paso ") || cmd == "/paso" => {
                let n: usize = cmd
                    .strip_prefix("/paso ")
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(1);
                let n = n.min(10_000);
                println!("  ⚡ Ejecutando {} pasos de simulación...", n);
                for i in 0..n {
                    let entrada = Entrada::vacía();
                    let _salida = cerebro.paso(0.001, entrada);
                    if (i + 1) % 1000 == 0 {
                        print!("\r  Progreso: {}/{} pasos", i + 1, n);
                        io::stdout().flush().unwrap_or(());
                    }
                }
                println!("\r  ✅ Simulación completada: {} pasos ejecutados", n);
                continue;
            }

            _ => {
                let amenaza = if input.contains("miedo") || input.contains("peligro") || input.contains("alerta") {
                    0.8
                } else {
                    0.1
                };
                let recompensa = if input.contains("gracias") || input.contains("bien") || input.contains("feliz") {
                    0.7
                } else {
                    0.2
                };

                let mut entrada = Entrada {
                    estimulos: Vec::new(),
                    texto: Some(input.to_string()),
                    recompensa,
                    amenaza,
                };

                if amenaza > 0.15 {
                    entrada.estimulos.push(Estimulo {
                        id: 9999,
                        intensidad: amenaza,
                        amenaza,
                        recompensa: 0.0,
                        valor: amenaza,
                    });
                }
                if recompensa > 0.25 {
                    entrada.estimulos.push(Estimulo {
                        id: 9998,
                        intensidad: recompensa,
                        amenaza: 0.0,
                        recompensa,
                        valor: recompensa,
                    });
                }

                let salida = cerebro.paso(0.001, entrada);
                println!("  🧠 [{:.3}s] {}", cerebro.tiempo, salida.texto);
            }
        }
    }

    println!("🧠 Cerebro digital apagado. Hasta la próxima.");
}
