// ============================================================================
// 🧠 APRENDIZ AUTÓNOMO — Daemon de Aprendizaje Dirigido en Segundo Plano
// ============================================================================
// Corre SIN PARAR en background, ejecutando el puente completo de aprendizaje:
//
//   [RUMIAR]  el cerebro articula desde sus asambleas consolidadas
//   [GUÍA]    el LLM (Tutor Groq, modelo grande e inteligente) decide QUÉ
//             investigar, guiado por el estado y las últimas palabras del cerebro
//   [EXPLORAR] ExploradorWeb navega DuckDuckGo + Wikipedia/arXiv (JS activo)
//   [DESTILAR] el LLM convierte la síntesis web cruda en una lección clara
//   [APRENDER] paso_tutor() consolida la lección como asamblea semántica
//   [GUARDAR]  persistencia inmediata a SSD (memoria no volátil)
//
// Es un loop infinito: el cerebro aprende autónomamente de la web, dirigido
// por el modelo, sin intervención humana. Ideal para correr con
// service_manager.sh.
//
// 🎓 TUTOR: usa la API directa de Groq (llama-3.3-70b-versatile, gratuito),
// leída de la variable GROQ_API_KEY. Sin OpenRouter.
// ============================================================================

use cerebro_digital::cerebro::cerebro::CerebroAutoOptimizable;
use cerebro_digital::cerebro::estructuras::Entrada;
use cerebro_digital::cerebro::explorador::ExploradorWeb;
use cerebro_digital::cerebro::tutor_groq::TutorGroq;
use std::thread;
use std::time::Duration;

const CICLO_ESPERA_SEG: u64 = 60; // pausa entre ciclos (respetar rate limit)
const DOMINIOS: [&str; 14] = [
    "neurociencia y consciencia",
    "inteligencia artificial y cognición",
    "física cuántica y el universo",
    "filosofía de la mente",
    "biología evolutiva y vida",
    "matemáticas y lógica",
    "redes neuronales y plasticidad sináptica",
    "tecnología y sociedad",
    "psicología y emociones humanas",
    "ética, moral y libre albedrío",
    "lingüística y origen del lenguaje",
    "astronomía y astrofísica",
    "historia de la ciencia",
    "sistemas complejos y teoría del caos",
];

fn main() {
    // La API key de Groq se lee del entorno (GROQ_API_KEY). Si no existe, se
    // imprime aviso y el daemon seguirá usando el fallback simulado en cada ciclo.
    let api_key = std::env::var("GROQ_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        eprintln!("⚠️  GROQ_API_KEY no definida — usando modo degradado (sin LLM tutor).");
    }
    let tutor = TutorGroq::new(api_key);

    println!("🧠 APRENDIZ AUTÓNOMO iniciado — aprendizaje dirigido sin parar");
    println!("═══════════════════════════════════════════════════════════════");
    println!("  🌐 Fuentes: Wikipedia / arXiv / DuckDuckGo (JS activo)");
    println!("  🎓 Tutor: Groq (modelo: {})", tutor.model_name());
    println!("  ⏱️  Ciclo cada {}s", CICLO_ESPERA_SEG);

    let mut cerebro = CerebroAutoOptimizable::nuevo();
    let mut indice_dominio = 0usize;
    let mut ciclo = 0u64;

    loop {
        ciclo += 1;
        println!("\n── CICLO #{ciclo} ──────────────────────────────────────────");

        // ── [RUMIAR] dejar que el cerebro articule espontáneamente ──
        let mut salida_anterior = String::new();
        for _ in 0..5 {
            let entrada = Entrada {
                estimulos: Vec::new(),
                texto: None,
                recompensa: 0.0,
                amenaza: 0.0,
            };
            let salida = cerebro.paso(0.001, entrada);
            let t = salida.texto.trim().to_string();
            if !t.is_empty() && t != "..." {
                salida_anterior = t;
                break;
            }
        }
        let contexto = if salida_anterior.is_empty() {
            "el cerebro está en silencio, en estado contemplativo".to_string()
        } else {
            format!("el cerebro articuló: \"{salida_anterior}\"")
        };

        // ── [GUÍA] el LLM decide QUÉ investigar ──
        let dominio = DOMINIOS[indice_dominio % DOMINIOS.len()];
        indice_dominio += 1;

        println!("  🧭 [GUÍA] Dominio: {dominio}");
        let pregunta = match tutor.consultar(
            &format!("Contexto del cerebro: {contexto}"),
            &format!(
                "Actúa como guía de estudio de un cerebro artificial biológico. \
                 Proponme UNA pregunta concreta, específica y respondible sobre \
                 '{dominio}', ideal para buscar en Wikipedia. Devuelve SOLO la \
                 pregunta, sin explicaciones ni comillas. Máximo 15 palabras."
            ),
        ) {
            Ok(p) => p.trim().trim_matches('"').to_string(),
            Err(e) => {
                eprintln!("  ⚠️ [GUÍA] error LLM ({e}); usando pregunta genérica");
                format!("¿Qué es {dominio}?")
            }
        };
        println!("  ❓ Pregunta guiada: \"{pregunta}\"");

        // ── [EXPLORAR] navegar la web ──
        let (sintesis, paginas) = match ExploradorWeb::explorar(&pregunta, 2) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("  🌐⚠️ [EXPLORAR] falló web ({e}), usando simulado");
                ExploradorWeb::explorar_simulado(&pregunta, 2)
            }
        };
        println!(
            "  🌐 [EXPLORAR] {}{}",
            if sintesis.is_empty() {
                "síntesis vacía".to_string()
            } else {
                format!("síntesis de {} car", sintesis.chars().count())
            },
            if paginas.is_empty() {
                String::new()
            } else {
                format!(" | {} páginas", paginas.len())
            }
        );

        // ── [DESTILAR] el LLM convierte el ruido en lección ──
        let recorte: String = sintesis.chars().take(3000).collect();
        let leccion = match tutor.consultar(
            &recorte,
            "Destila este contenido en UNA lección de máximo 3 oraciones, \
             clara y precisa, adecuada para que un cerebro biológico la \
             asocie y aprenda. Devuelve SOLO la lección, sin prefijos ni \
             comillas.",
        ) {
            Ok(l) => l.trim().trim_matches('"').to_string(),
            Err(e) => {
                eprintln!("  ⚠️ [DESTILAR] error LLM ({e}); usando síntesis cruda");
                recorte.chars().take(300).collect()
            }
        };

        // ── [APRENDER] consolidar como asamblea semántica ──
        if !leccion.is_empty() {
            cerebro.paso_tutor(&leccion);
            println!("  🎓 [APRENDER] lección consolidada: \"{}\"", trunca(&leccion, 90));
        } else {
            println!("  ⏭️  [APRENDER] lección vacía, omitiendo este ciclo");
        }

        // ── [DIÁLOGO] conversación bidireccional con el modelo nube ──
        // El cerebro intenta articular algo sobre lo aprendido; Groq responde
        // como interlocutor; el cerebro integra la respuesta (asamblea + vínculo).
        if !leccion.is_empty() {
            // Turno 1: el cerebro intenta hablar estimulado por la lección
            let articulacion = {
                let entrada = Entrada {
                    estimulos: Vec::new(),
                    texto: Some(trunca(&leccion, 300)),
                    recompensa: 0.2,
                    amenaza: 0.0,
                };
                let salida = cerebro.paso(0.001, entrada);
                salida.texto.trim().to_string()
            };
            let mensaje_cerebro = if articulacion.is_empty() || articulacion == "..." {
                format!("Acabo de aprender esto: \"{}\". ¿Qué opinás?", trunca(&leccion, 120))
            } else {
                articulacion
            };

            match tutor.consultar(
                &mensaje_cerebro,
                "Eres el interlocutor de un cerebro artificial que está aprendiendo \
                 y tratando de pensar. Responde de forma breve (máximo 2 oraciones), \
                 cálida y estimulante, como un compañero de conversación. Plantéale \
                 una idea o pregunta que lo haga reflexionar más.",
            ) {
                Ok(respuesta) => {
                    let respuesta = respuesta.trim().trim_matches('"').to_string();
                    if !respuesta.is_empty() {
                        cerebro.paso_tutor(&respuesta);
                        println!("  💬 [DIÁLOGO] cerebro: \"{}\"", trunca(&mensaje_cerebro, 80));
                        println!("  💬 [DIÁLOGO] modelo:   \"{}\"", trunca(&respuesta, 80));
                    }
                }
                Err(e) => eprintln!("  ⚠️ [DIÁLOGO] error LLM ({e}); omitiendo diálogo"),
            }
        }

        // ── [GUARDAR] persistencia inmediata ──
        match cerebro.guardar_a_disco() {
            Ok(_) => println!("  💾 [GUARDAR] estado persistido a SSD"),
            Err(e) => eprintln!("  ⚠️ [GUARDAR] error: {e}"),
        }

        println!("  ✔️ Ciclo #{ciclo} completado. Siguiente en {}s...", CICLO_ESPERA_SEG);
        thread::sleep(Duration::from_secs(CICLO_ESPERA_SEG));
    }
}

fn trunca(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n).collect::<String>() + "..."
    }
}
