// ============================================================================
// 🧠 SAE v2 — ENTRENADOR DEL NÚCLEO (Destilación desde NEXUS)
// ============================================================================
// Lee data/destilacion.jsonl (expectativas generadas por NEXUS a través de
// tutor_nexus.py) y entrena el BioTransformerCore por backpropagation real
// (candle autograd). Los pesos son obra propia del sistema.
//
// Uso:
//   cargo run --bin entrenar-nucleo -- [--epocas N] [--lr 0.003] [--max-len 64]
//
// Salida:
//   - data/nucleo_pesos.safetensors  (pesos guardados)
//   - data/nucleo_vocab.json         (vocabulario)
// ============================================================================

use cerebro_digital::cerebro::sae::nucleo_numerico::{
    EntrenadorBio, NucleoConfig, Vocabulario, dispositivo,
};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Normaliza un token: minúsculas + sin acentos (consistencia léxica).
fn normalizar(t: &str) -> String {
    const A: &[char] = &['á', 'à', 'ä', 'â', 'ã'];
    const E: &[char] = &['é', 'è', 'ë', 'ê'];
    const I: &[char] = &['í', 'ì', 'ï', 'î'];
    const O: &[char] = &['ó', 'ò', 'ö', 'ô', 'õ'];
    const U: &[char] = &['ú', 'ù', 'ü', 'û'];
    let mut out = String::with_capacity(t.len());
    for c in t.chars().flat_map(|c| c.to_lowercase()) {
        out.push(match c {
            'á' | 'à' | 'ä' | 'â' | 'ã' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            other => other,
        });
    }
    out
}

/// Un ejemplo de destilación: estímulo → respuesta esperada del tutor NEXUS.
#[derive(Debug, Deserialize, Clone)]
struct EjemploDestilacion {
    estimulo: String,
    respuesta: String,
}

/// Tokenizador simple por palabras (fase inicial). Cada palabra (y puntuación
/// separada) es un token. Suficiente para el arranque del Bio-Transformer;
/// un tokenizador BPE podrá sustituirlo sin cambiar el núcleo.
struct Tokenizador {
    vocabulario: Vocabulario,
}

impl Tokenizador {
    fn nuevo(ejemplos: &[EjemploDestilacion]) -> Self {
        let mut tokens: Vec<String> = Vec::new();
        let mut vistos: HashMap<String, bool> = HashMap::new();
        for e in ejemplos {
            for t in Self::tokenizar(&e.estimulo).into_iter().chain(Self::tokenizar(&e.respuesta)) {
                if !vistos.contains_key(&t) {
                    vistos.insert(t.clone(), true);
                    tokens.push(t);
                }
            }
        }
        Self {
            vocabulario: Vocabulario::nuevo(&tokens),
        }
    }

    fn tokenizar(texto: &str) -> Vec<String> {
        // Separa palabras y puntuación: "hola, mundo" → ["hola", ",", "mundo"].
        // Normaliza a minúsculas y sin acentos (consistencia léxica).
        texto
            .split_whitespace()
            .flat_map(|w| {
                let mut out = Vec::new();
                let mut buf = String::new();
                for c in w.chars() {
                    if c.is_alphanumeric() {
                        buf.push(c);
                    } else {
                        if !buf.is_empty() {
                            out.push(std::mem::take(&mut buf));
                        }
                        out.push(c.to_string());
                    }
                }
                if !buf.is_empty() {
                    out.push(buf);
                }
                out
            })
            .map(|t| normalizar(&t))
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn ids(&self, texto: &str) -> Vec<usize> {
        Self::tokenizar(texto)
            .iter()
            .map(|t| self.vocabulario.id_para(t))
            .collect()
    }
}

fn cargar_ejemplos(ruta: &Path) -> Vec<EjemploDestilacion> {
    if !ruta.exists() {
        eprintln!("❌ No existe {} — ejecuta primero tutor_nexus.py para acumular expectativas.", ruta.display());
        std::process::exit(1);
    }
    let contenido = fs::read_to_string(ruta).unwrap_or_default();
    contenido
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<EjemploDestilacion>(l).ok())
        .collect()
}

fn main() {
    let mut epocas: usize = 3;
    let mut lr: f64 = 3e-3;
    let mut max_len: usize = 64;

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--epocas" => epocas = args.next().and_then(|v| v.parse().ok()).unwrap_or(epocas),
            "--lr" => lr = args.next().and_then(|v| v.parse().ok()).unwrap_or(lr),
            "--max-len" => max_len = args.next().and_then(|v| v.parse().ok()).unwrap_or(max_len),
            _ => {}
        }
    }

    println!("🧬 [SAE v2] Entrenando Bio-Transformer por backprop (destilación NEXUS)");
    println!("  ├─ épocas: {epocas} | lr: {lr} | max_len: {max_len}");

    // ── Cargar expectativas ───────────────────────────────────────────────
    let data_dir = Path::new("data");
    fs::create_dir_all(data_dir).expect("crear data/");
    let ejemplos = cargar_ejemplos(&data_dir.join("destilacion.jsonl"));
    println!("  ├─ ejemplos de destilación: {}", ejemplos.len());
    if ejemplos.is_empty() {
        eprintln!("❌ No hay ejemplos. Ejecuta tutor_nexus.py para que NEXUS genere expectativas.");
        std::process::exit(1);
    }

    // ── Vocabulario y tokenización ────────────────────────────────────────
    let tok = Tokenizador::nuevo(&ejemplos);
    let v = tok.vocabulario.tam();
    println!("  ├─ vocabulario: {v} tokens");

    let cfg = NucleoConfig {
        tam_vocabulario: v.max(64),
        max_len,
        ..NucleoConfig::default()
    };

    let device = dispositivo().expect("dispositivo candle");
    println!("  ├─ dispositivo: {device:?}");

    let mut entrenador = EntrenadorBio::nuevo(cfg, tok.vocabulario.clone(), device, lr)
        .expect("crear entrenador");

    // ── Preparar datos: cada ejemplo → (contexto, siguiente token) ────────
    // Autoregresivo con shift: entrada = ids[..t-1], objetivo = ids[1..]
    let mut secuencias: Vec<Vec<usize>> = Vec::new();
    for e in &ejemplos {
        let mut ids = tok.ids(&e.estimulo);
        ids.extend(tok.ids(&e.respuesta));
        if ids.len() > max_len {
            ids.truncate(max_len);
        }
        if ids.len() >= 2 {
            secuencias.push(ids);
        }
    }
    println!("  ├─ secuencias válidas: {}", secuencias.len());
    if secuencias.is_empty() {
        eprintln!("❌ No hay secuencias de longitud >= 2.");
        std::process::exit(1);
    }

    // ── Entrenamiento (backprop real) ─────────────────────────────────────
    let mut mejor_loss = f32::INFINITY;
    for ep in 0..epocas {
        let mut loss_acum = 0.0f32;
        let mut n = 0usize;
        for ids in &secuencias {
            let ids_u32: Vec<u32> = ids.iter().map(|&i| i as u32).collect();
            let tokens = candle_core::Tensor::new(ids_u32.as_slice(), &entrenador.modelo.device)
                .expect("tensor tokens")
                .unsqueeze(0)
                .expect("batch dim");
            let loss = entrenador.train_step(&tokens, &tokens, true).expect("train_step");
            loss_acum += loss;
            n += 1;
        }
        let media = if n > 0 { loss_acum / n as f32 } else { f32::INFINITY };
        println!("  ├─ época {}: loss medio = {:.4}", ep + 1, media);
        if media < mejor_loss {
            mejor_loss = media;
            let ruta_pesos = data_dir.join("nucleo_pesos.safetensors");
            entrenador.guardar(&ruta_pesos).expect("guardar pesos");
            println!("  │   💾 pesos guardados en {}", ruta_pesos.display());
        }
    }

    // ── Guardar vocabulario ───────────────────────────────────────────────
    let vocab_json = serde_json::to_string_pretty(&tok.vocabulario.id_a_token).expect("json");
    fs::write(data_dir.join("nucleo_vocab.json"), vocab_json).expect("guardar vocabulario");
    println!("  ├─ vocabulario guardado en data/nucleo_vocab.json");

    // ── Evaluación con el Juez E3 ─────────────────────────────────────────
    use cerebro_digital::cerebro::sae::juez_e3::{dictaminar, evaluar_secuencia, reportar};
    println!("\n🧑‍⚖️  Evaluando con el Juez E3...");
    let semillas = ["mente", "cerebro", "sistema", "vida", "luz", "puro", "bueno"];
    let mut evaluadas = Vec::new();
    for semilla in semillas {
        let ids_seed = tok.ids(semilla);
        if ids_seed.is_empty() {
            continue;
        }
        if let Ok(ids) = entrenador.modelo.sample(&ids_seed, 12, 0.8) {
            let texto = entrenador.modelo.ids_a_texto(&ids);
            println!("  Criatura 👶 > {texto}");
            evaluadas.push(evaluar_secuencia(&ids, &tok.vocabulario));
        }
    }

    let dictamen = dictaminar(&evaluadas, &tok.vocabulario);
    println!();
    println!("{}", reportar(&dictamen));

    println!("\n✅ Entrenamiento del núcleo completado (loss final {mejor_loss:.4}).");
    if dictamen.fase == cerebro_digital::cerebro::sae::juez_e3::FaseE3::Fase2 {
        println!("   🚀 Fase 2 alcanzada: aprendizaje semántico profundo habilitado.");
    } else {
        println!("   🔁 Acumula más destilación desde NEXUS (tutor_nexus.py) y re-entrena para subir de fase.");
    }
}
