// ============================================================================
// 🗣️ DEMO DE EXPRESIÓN — Validación de la Cirugía Expresiva A+B+C
// ============================================================================
// Tras el desacoplamiento triple de espacios de ID, el cerebro respondía
// siempre con "...". Esta demo enseña lecciones vía paso_tutor (que consolida
// asambleas con IDs semánticos base_neurona+dimensión), y luego re-expone el
// mismo estímulo textual para demostrar que el cerebro YA articula ideas.
// ============================================================================

use cerebro_digital::cerebro::cerebro::CerebroAutoOptimizable;
use cerebro_digital::cerebro::estructuras::Entrada;

fn main() {
    println!("{}", "=".repeat(60));
    println!("  🗣️  DEMO DE EXPRESIÓN — el cerebro aprende y luego habla");
    println!("{}", "=".repeat(60));

    let mut cerebro = CerebroAutoOptimizable::nuevo();

    let lecciones = [
        "exocortex cuántica ignición",
        "red neuronal sincrónica pulsante",
        "hormiga digital arquitecto",
        "sembrador de identidades digitales",
    ];

    println!("\n── FASE 1: Enseñanza (paso_tutor consolida asambleas semánticas) ──");
    for leccion in &lecciones {
        cerebro.paso_tutor(leccion);
        println!("  📚 Enseñado: \"{}\"", leccion);
    }

    println!("\n── FASE 2: Re-exposición al estímulo (el cerebro debe articular) ──");
    let mut articulo = false;
    for leccion in &lecciones {
        let entrada = Entrada {
            estimulos: Vec::new(),
            texto: Some(leccion.to_string()),
            recompensa: 0.0,
            amenaza: 0.0,
        };
        let salida = cerebro.paso(0.001, entrada);
        let texto = salida.texto.trim();
        if !texto.is_empty() && texto != "..." {
            println!("  🧠 [{:>3}] \"{}\"  →  dice: \"{}\"", cerebro.paso_actual, leccion, texto);
            articulo = true;
        } else {
            println!("  🧠 [{:>3}] \"{}\"  →  silencio (...) ", cerebro.paso_actual, leccion);
        }
    }

    println!("\n── FASE 3: Rumiación sin entrada (volición espontánea) ──");
    let mut espontaneo = 0;
    for _ in 0..20 {
        let entrada = Entrada {
            estimulos: Vec::new(),
            texto: None,
            recompensa: 0.0,
            amenaza: 0.0,
        };
        let salida = cerebro.paso(0.001, entrada);
        let texto = salida.texto.trim();
        if !texto.is_empty() && texto != "..." {
            println!("  🧠 [rumia] dice: \"{}\"", texto);
            espontaneo += 1;
            if espontaneo >= 3 {
                break;
            }
        }
    }

    println!("{}", "=".repeat(60));
    if articulo {
        println!("  ✅ RESULTADO: el cerebro articula ideas tras aprender. Cirugía expresiva validada.");
    } else {
        println!("  ⚠️  El cerebro aún no articuló por re-exposición directa.");
    }
    println!("{}", "=".repeat(60));
}
