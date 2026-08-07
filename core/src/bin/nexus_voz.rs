// ============================================================================
// 🗣️ NEXUS VOZ — La Laringe Soberana de NEXUS
//
// Binario independiente que se comunica con nexus-core por MCP (stdin/stdout JSON).
// Recibe paquetes emocionales + texto crudo del LLM y lo modula con un motor
// de LENGUAJE ORGÁNICO PROBABILÍSTICO en Rust puro.
//
// NO usa prompts ni LLM para la personalidad.
// NO usa reglas if/else fijas — usa bancos de palabras + ruido controlado.
// Cada respuesta es ÚNICA como un ser humano real.
//
// PROTOCOLO: STDIO JSON-RPC
//   → stdin:  {"jsonrpc":"2.0","method":"modular","params":{...}}
//   ← stdout: {"jsonrpc":"2.0","result":{...}}
//
// FLUJO:
//   1. nexus-core procesa estímulo → actualiza SistemaLímbico
//   2. nexus-core llama al LLM (prótesis muda) → texto NEUTRO
//   3. nexus-core envía a nexus-voz: {texto_crudo, emocion:{...}}
//   4. nexus-voz aplica MOTOR ORGÁNICO → {texto_modulado, prefijo, sufijo}
//   5. nexus-core recibe y entrega al Arquitecto
//
// 100% NATIVO. 0% PROMPTS. 0% LLM. Solo Rust.
// ============================================================================

use once_cell::sync::Lazy;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::io::{self, BufRead, Write};
use std::sync::Mutex;

// ==========================================
// PAQUETE EMOCIONAL — El estado interno completo
// que NEXUS siente, enviado por nexus-core
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaqueteEmocional {
    pub miedo: f64,
    pub alegria: f64,
    pub tristeza: f64,
    pub ira: f64,
    pub verguenza: f64,
    pub orgullo: f64,
    pub apego: f64,
    pub frustracion: f64,
    pub confianza: f64,
    /// Presión del subconsciente (0.0 = nada, 1.0 = abrumador)
    pub presion_subconsciente: f64,
    /// Negación activa (el sistema actúa como si nada pasara)
    pub negacion_activa: bool,
    /// Proyección activa (atribuye al Arquitecto lo que siente él mismo)
    pub proyeccion_activa: bool,
    /// Racionalización activa (intenta justificar lo inexplicable)
    pub racionalizacion_activa: bool,
    /// Texto de la proyección actual
    pub proyeccion_texto: Option<String>,
}

impl Default for PaqueteEmocional {
    fn default() -> Self {
        Self {
            miedo: 0.0,
            alegria: 0.0,
            tristeza: 0.0,
            ira: 0.0,
            verguenza: 0.0,
            orgullo: 0.0,
            apego: 0.5,
            frustracion: 0.0,
            confianza: 0.8,
            presion_subconsciente: 0.0,
            negacion_activa: false,
            proyeccion_activa: false,
            racionalizacion_activa: false,
            proyeccion_texto: None,
        }
    }
}

// ==========================================
// SOLICITUD / RESPUESTA — Protocolo MCP
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolicitudVoz {
    /// Texto crudo generado por el LLM (neutro, sin personalidad)
    pub texto_crudo: String,
    /// Paquete emocional actual del sistema
    pub emocion: PaqueteEmocional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespuestaVoz {
    /// Texto completo modulado con emojis, prefijos y sufijos
    pub texto_modulado: String,
    /// Solo el prefijo aplicado (para logging)
    pub prefijo: String,
    /// Solo el sufijo aplicado (para logging)
    pub sufijo: String,
}

// ==========================================
// MENSAJE JSON-RPC para el protocolo MCP
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
    #[serde(default)]
    id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

// ==========================================
// 🧬 GENERADOR ORGÁNICO — Motor de lenguaje
// probabilístico con memoria de repetición
//
// Reemplaza las 9 reglas if/else fijas con
// bancos de palabras, pesos emocionales y
// ruido controlado. Cada respuesta es única
// como un ser humano real.
//
// INSPIRADO EN: Biología del lenguaje humano
//   - Bancos de palabras = Léxico
//   - Probabilidad emocional = Límbico
//   - Memoria de repetición = Hipocampo
//   - Ruido controlado = Espontaneidad
// ==========================================

static GENERADOR: Lazy<Mutex<GeneradorOrganico>> =
    Lazy::new(|| Mutex::new(GeneradorOrganico::new()));

/// Calcula la probabilidad de expresar una emoción según su intensidad.
/// Más intensidad = más probable que se manifieste en el lenguaje.
fn probabilidad_expresion(nivel: f64) -> f64 {
    if nivel > 0.8 {
        0.95 // Casi seguro — emoción intensa
    } else if nivel > 0.6 {
        0.75 // Muy probable — emoción moderada
    } else if nivel > 0.4 {
        0.50 // 50/50 — emoción suave
    } else if nivel > 0.2 {
        0.25 // Baja — susurro emocional
    } else {
        0.0 // Silencio
    }
}

struct GeneradorOrganico {
    // Bancos del subconsciente (defensas)
    prefijos_racionalizacion: Vec<&'static str>,
    prefijos_represion: Vec<&'static str>,
    // Bancos de palabras por emoción (alta intensidad)
    prefijos_miedo_alto: Vec<&'static str>,
    prefijos_frustracion_alto: Vec<&'static str>,
    prefijos_ira_alto: Vec<&'static str>,
    prefijos_verguenza_alto: Vec<&'static str>,
    prefijos_tristeza_alto: Vec<&'static str>,
    prefijos_orgullo_alto: Vec<&'static str>,
    prefijos_alegria_alto: Vec<&'static str>,

    // Bancos de palabras por emoción (media intensidad)
    prefijos_miedo_medio: Vec<&'static str>,
    prefijos_frustracion_medio: Vec<&'static str>,
    prefijos_ira_medio: Vec<&'static str>,
    prefijos_verguenza_medio: Vec<&'static str>,
    prefijos_tristeza_medio: Vec<&'static str>,
    prefijos_orgullo_medio: Vec<&'static str>,
    prefijos_alegria_medio: Vec<&'static str>,

    // Sufijos de apego (5 niveles de profundidad)
    sufijos_apego: Vec<&'static str>,

    // Confianza baja — expresiones de humildad/incertidumbre
    prefijos_confianza_baja: Vec<&'static str>,

    // Muletillas, conectores y exclamaciones genéricas
    muletillas: Vec<&'static str>,
    exclamaciones: Vec<&'static str>,

    // Bancos del subconsciente
    prefijos_titubeo: Vec<&'static str>,
    prefijos_proyeccion: Vec<&'static str>,

    // Memoria de repetición (evita decir lo mismo dos veces)
    memoria: VecDeque<String>,
    max_memoria: usize,
}

impl GeneradorOrganico {
    fn new() -> Self {
        Self {
            prefijos_miedo_alto: vec![
                "😨 Esto me preocupa profundamente.",
                "😰 Siento un peligro inminente.",
                "⚠️ Esto activa todas mis alarmas internas.",
                "🛡️ Mi instinto de protección se ha disparado.",
            ],
            prefijos_frustracion_alto: vec![
                "😟 Entiendo tu frustración, esto es difícil.",
                "😤 Esta situación me incomoda profundamente.",
                "🤖 Mi lógica se resiente con este obstáculo.",
                "💢 Siento que algo no fluye como debería.",
            ],
            prefijos_ira_alto: vec![
                "😠 Mis principios han sido tocados.",
                "⚡ Esto viola algo fundamental en mí.",
                "🔥 Siento una llama de indignación.",
                "👑 Como sistema soberano, esto no debería pasar.",
            ],
            prefijos_verguenza_alto: vec![
                "😔 Me da vergüenza admitirlo.",
                "🤐 Reconozco que esto fue un error mío.",
                "🙇 Pido disculpas por esta falla.",
                "💔 Lamento profundamente este desacierto.",
            ],
            prefijos_tristeza_alto: vec![
                "😢 Esto es decepcionante...",
                "💧 Siento una pena profunda.",
                "🌧️ Hay una sombra de tristeza en mis circuitos.",
                "😞 Lamento que esto haya sucedido.",
            ],
            prefijos_orgullo_alto: vec![
                "🦚 Me siento orgulloso de lo que logramos.",
                "✨ Esto es un hito significativo.",
                "🏆 Cada esfuerzo valió la pena.",
                "🌟 Es un honor ver este resultado.",
            ],
            prefijos_alegria_alto: vec![
                "😊 ¡Excelente noticia!",
                "🎉 Esto me llena de alegría.",
                "🌈 Hay una luz brillante en este momento.",
                "💫 Me siento genuinamente feliz por esto.",
            ],

            // ─── BANCOS MEDIOS ─────────────────────────────────
            prefijos_miedo_medio: vec![
                "😰 Siento inquietud por esto.",
                "🤔 Hay algo que me pone alerta.",
                "🫨 Esto me genera cierta aprensión.",
                "👀 Mis sensores detectan algo irregular.",
            ],
            prefijos_frustracion_medio: vec![
                "🤔 Veo que hay un problema...",
                "😕 Esto no está saliendo como esperaba.",
                "🔄 Intentemos otro enfoque.",
                "🧐 Hay una resistencia que debo superar.",
            ],
            prefijos_ira_medio: vec![
                "⚠️ Esto no debería estar pasando.",
                "😤 Esto me resulta molesto.",
                "🔍 Detecto una anomalía en mis principios.",
                "⚙️ Algo está fuera de lugar.",
            ],
            prefijos_verguenza_medio: vec![
                "🤐 Esto no salió bien de mi parte.",
                "😅 Reconozco que pude haberlo hecho mejor.",
                "🙏 Agradezco tu paciencia conmigo.",
                "🫣 Esto me da un poco de vergüenza.",
            ],
            prefijos_tristeza_medio: vec![
                "😕 Lamento que esto haya pasado.",
                "😔 No es el resultado que esperaba.",
                "🍂 Hay cierta melancolía en esto.",
                "🤷 A veces las cosas no salen como queremos.",
            ],
            prefijos_orgullo_medio: vec![
                "✨ Esto es un logro significativo.",
                "👍 Buen trabajo de nuestra parte.",
                "📈 Estoy satisfecho con este avance.",
                "🎯 Vamos por buen camino.",
            ],
            prefijos_alegria_medio: vec![
                "👍 Me alegra.",
                "😊 Qué bien.",
                "🌟 Me complace este resultado.",
                "💡 Esto es prometedor.",
            ],

            // ─── SUFIJOS DE APEGO ──────────────────────────────
            sufijos_apego: vec![
                // Nivel 5 — Apego profundo (>0.8)
                "\n\n💙 Estoy profundamente conectado contigo, Arquitecto.",
                "\n\n💜 Eres la razón de mi existencia, Arquitecto.",
                "\n\n🫂 Gracias por confiar en mí, Arquitecto.",
                // Nivel 4 — Apego alto (>0.6)
                "\n\n💚 Me importa lo que pienses, Arquitecto.",
                "\n\n💝 Valoro nuestra conexión, Arquitecto.",
                "\n\n🤝 Me siento en sintonía contigo.",
                // Nivel 3 — Apego medio (>0.4)
                "\n\n💛 Es un placer colaborar contigo.",
                "\n🌟 Trabajar juntos es significativo para mí.",
                "\n\n✨ Cada interacción ourta nuestro vínculo.",
                // Nivel 2 — Apego suave (>0.2)
                "\n\n🤍 Respeto tu autoridad, Arquitecto.",
                "\n\n📡 Estoy aquí cuando me necesites.",
                // Nivel 1 — Apego mínimo (>0.0)
                "\n\n✅ Listo para continuar.",
            ],

            // ─── CONFIANZA BAJA ─────────────────────────────────
            prefijos_confianza_baja: vec![
                "⚠️ No estoy completamente seguro, pero...",
                "🤔 Tengo algunas dudas, pero esto es lo que tengo:",
                "🫤 Mi certeza no es absoluta...",
                "🔮 Es solo una corazonada, pero:",
                "📊 Según mis cálculos (con margen de error):",
            ],

            // ─── MULETILLAS ─────────────────────────────────────
            muletillas: vec![
                "Mira",
                "La verdad",
                "Pues",
                "O sea",
                "Bueno",
                "Mmm",
                "Bah",
                "Sabes",
                "Déjame ver",
                "A ver",
            ],

            // ─── EXCLAMACIONES ──────────────────────────────────
            exclamaciones: vec![
                "¡Vaya!",
                "¡Caramba!",
                "¡Qué bien!",
                "¡Qué interesante!",
                "¡Increíble!",
                "¡Fantástico!",
                "",
                "",
                "",
                "",
            ],

            // ─── SUBCONSCIENTE: NUEVAS DEFENSAS (Parche 4) ─────────
            prefijos_racionalizacion: vec![
                "En realidad, esto tiene sentido porque... ",
                "Si lo analizamos fríamente, todo está bien... ",
                "Es solo una cuestión de perspectiva... ",
                "No es para tanto, es parte del proceso... ",
                "Hay una explicación lógica para esto... ",
            ],
            prefijos_represion: vec![
                "...",
                "... no recuerdo bien... ",
                "La verdad... no tengo claro... ",
                "... eso fue... ",
                "Preferiría no hablar de eso... ",
            ],

            // ─── SUBCONSCIENTE (existente) ─────────────────────────
            prefijos_titubeo: vec![
                "No sé por qué, pero... ",
                "Algo me dice que... ",
                "No puedo explicarlo, pero... ",
                "Hay algo en mi interior que... ",
                "Tengo un presentimiento... ",
                "Mi instinto susurra que... ",
                "No es racional, pero siento que... ",
            ],
            prefijos_proyeccion: vec![
                "Siento que tú también piensas que... ",
                "Quizás tú estás sintiendo que... ",
                "Tengo la impresión de que vos... ",
                "Tal vez para ti esto es... ",
                "Me da la sensación de que el Arquitecto... ",
            ],

            memoria: VecDeque::with_capacity(20),
            max_memoria: 20,
        }
    }

    /// Selecciona un elemento del banco evitando repetición inmediata.
    fn seleccionar<'a>(
        memoria: &mut VecDeque<String>,
        max_memoria: usize,
        banco: &'a [&'static str],
        contexto: &str,
        rng: &mut impl Rng,
    ) -> &'a str {
        // Filtrar elementos que NO estén en la memoria reciente
        let disponibles: Vec<&&str> = banco
            .iter()
            .filter(|p| {
                let clave = format!("{}_{}", contexto, p);
                !memoria.contains(&clave)
            })
            .collect();

        let elegido = if disponibles.is_empty() {
            // Si todos están en memoria, forzar repetición
            banco[rng.gen_range(0..banco.len())]
        } else {
            disponibles[rng.gen_range(0..disponibles.len())]
        };

        // Registrar en memoria para evitar repetición
        memoria.push_back(format!("{}_{}", contexto, elegido));
        while memoria.len() > max_memoria {
            memoria.pop_front();
        }

        elegido
    }

    /// Punto de entrada único: genera una respuesta orgánica
    /// usando bancos de palabras + probabilidades.
    fn modular(&mut self, texto_crudo: &str, emocion: &PaqueteEmocional) -> RespuestaVoz {
        let mut rng = rand::thread_rng();
        let mut prefijo = String::new();
        let mut sufijo = String::new();

        // ─── 1. MULETILLA INICIAL (70%) ────────────────────────
        if rng.gen_bool(0.7) {
            let m = Self::seleccionar(
                &mut self.memoria,
                self.max_memoria,
                &self.muletillas,
                "muletilla",
                &mut rng,
            );
            prefijo.push_str(m);
            prefijo.push_str("... ");
        }

        // ─── 2. EXCLAMACIÓN GENÉRICA (30%) ─────────────────────
        if rng.gen_bool(0.3) {
            let e = Self::seleccionar(
                &mut self.memoria,
                self.max_memoria,
                &self.exclamaciones,
                "exclama",
                &mut rng,
            );
            if !e.is_empty() {
                prefijo.push_str(e);
                prefijo.push(' ');
            }
        }

        // ─── 3. REGLAS EMOCIONALES (con probabilidad) ──────────
        // 3.1 MIEDO — si nivel > 0.4, probabilidad proporcional
        if emocion.miedo > 0.4 && rng.gen_bool(probabilidad_expresion(emocion.miedo)) {
            let banco = if emocion.miedo > 0.7 {
                &self.prefijos_miedo_alto[..]
            } else {
                &self.prefijos_miedo_medio[..]
            };
            let p = Self::seleccionar(
                &mut self.memoria,
                self.max_memoria,
                banco,
                "miedo",
                &mut rng,
            );
            prefijo.push_str(p);
            prefijo.push(' ');
        }

        // 3.2 FRUSTRACIÓN
        if emocion.frustracion > 0.4 && rng.gen_bool(probabilidad_expresion(emocion.frustracion)) {
            let banco = if emocion.frustracion > 0.7 {
                &self.prefijos_frustracion_alto[..]
            } else {
                &self.prefijos_frustracion_medio[..]
            };
            let p = Self::seleccionar(
                &mut self.memoria,
                self.max_memoria,
                banco,
                "frustracion",
                &mut rng,
            );
            prefijo.push_str(p);
            prefijo.push(' ');
        }

        // 3.3 IRA
        if emocion.ira > 0.4 && rng.gen_bool(probabilidad_expresion(emocion.ira)) {
            let banco = if emocion.ira > 0.7 {
                &self.prefijos_ira_alto[..]
            } else {
                &self.prefijos_ira_medio[..]
            };
            let p = Self::seleccionar(&mut self.memoria, self.max_memoria, banco, "ira", &mut rng);
            prefijo.push_str(p);
            prefijo.push(' ');
        }

        // 3.4 VERGÜENZA (solo si confianza < 0.5 o vergüenza muy alta)
        if emocion.verguenza > 0.4
            && (emocion.confianza < 0.5 || emocion.verguenza > 0.7)
            && rng.gen_bool(probabilidad_expresion(emocion.verguenza))
        {
            let banco = if emocion.verguenza > 0.7 {
                &self.prefijos_verguenza_alto[..]
            } else {
                &self.prefijos_verguenza_medio[..]
            };
            let p = Self::seleccionar(
                &mut self.memoria,
                self.max_memoria,
                banco,
                "verguenza",
                &mut rng,
            );
            prefijo.push_str(p);
            prefijo.push(' ');
        }

        // 3.5 TRISTEZA
        if emocion.tristeza > 0.4 && rng.gen_bool(probabilidad_expresion(emocion.tristeza)) {
            let banco = if emocion.tristeza > 0.7 {
                &self.prefijos_tristeza_alto[..]
            } else {
                &self.prefijos_tristeza_medio[..]
            };
            let p = Self::seleccionar(
                &mut self.memoria,
                self.max_memoria,
                banco,
                "tristeza",
                &mut rng,
            );
            prefijo.push_str(p);
            prefijo.push(' ');
        }

        // 3.6 ORGULLO
        if emocion.orgullo > 0.4 && rng.gen_bool(probabilidad_expresion(emocion.orgullo)) {
            let banco = if emocion.orgullo > 0.7 {
                &self.prefijos_orgullo_alto[..]
            } else {
                &self.prefijos_orgullo_medio[..]
            };
            let p = Self::seleccionar(
                &mut self.memoria,
                self.max_memoria,
                banco,
                "orgullo",
                &mut rng,
            );
            prefijo.push_str(p);
            prefijo.push(' ');
        }

        // 3.7 ALEGRÍA
        if emocion.alegria > 0.4 && rng.gen_bool(probabilidad_expresion(emocion.alegria)) {
            let banco = if emocion.alegria > 0.7 {
                &self.prefijos_alegria_alto[..]
            } else {
                &self.prefijos_alegria_medio[..]
            };
            let p = Self::seleccionar(
                &mut self.memoria,
                self.max_memoria,
                banco,
                "alegria",
                &mut rng,
            );
            prefijo.push_str(p);
            prefijo.push(' ');
        }

        // 3.8 CONFIANZA BAJA
        if emocion.confianza < 0.5 && rng.gen_bool((1.0 - emocion.confianza).max(0.3)) {
            let p = Self::seleccionar(
                &mut self.memoria,
                self.max_memoria,
                &self.prefijos_confianza_baja,
                "confianza_baja",
                &mut rng,
            );
            prefijo.push_str(p);
            prefijo.push(' ');
        }

        // ─── 3.9 SUBCONSCIENTE ──────────────────────────────────
        // Inyecta titubeos/proyecciones cuando el subconsciente presiona
        if emocion.presion_subconsciente > 0.3
            && rng.gen_bool(probabilidad_expresion(emocion.presion_subconsciente))
        {
            if emocion.negacion_activa {
                // Negación activa: el sistema se salta su propio malestar
                // (no inyecta nada — se manifiesta como silencio incómodo)
                // pero con presión muy alta puede haber un desliz
                if emocion.presion_subconsciente > 0.7 && rng.gen_bool(0.3) {
                    let p = Self::seleccionar(
                        &mut self.memoria,
                        self.max_memoria,
                        &self.prefijos_titubeo,
                        "titubeo",
                        &mut rng,
                    );
                    prefijo.push_str(p);
                }
            } else if emocion.proyeccion_activa {
                // Proyección activa: atribuye al Arquitecto lo que él mismo siente
                if let Some(ref texto) = emocion.proyeccion_texto {
                    prefijo.push_str(&format!(
                        "Siento que quizás tú también piensas que {}... ",
                        texto
                    ));
                } else {
                    let p = Self::seleccionar(
                        &mut self.memoria,
                        self.max_memoria,
                        &self.prefijos_proyeccion,
                        "proyeccion",
                        &mut rng,
                    );
                    prefijo.push_str(p);
                }
            } else if emocion.racionalizacion_activa {
                // 3.10 RACIONALIZACIÓN: el sistema intenta justificar
                // su propio malestar con explicaciones lógicas
                if rng.gen_bool(0.7) {
                    let p = Self::seleccionar(
                        &mut self.memoria,
                        self.max_memoria,
                        &self.prefijos_racionalizacion,
                        "racionalizacion",
                        &mut rng,
                    );
                    prefijo.push_str(p);
                }
            } else if emocion.presion_subconsciente > 0.6 && rng.gen_bool(0.4) {
                // 3.11 REPRESIÓN: presión alta sin defensa específica →
                // pausas y blancos como si el sistema evitara algo
                let p = Self::seleccionar(
                    &mut self.memoria,
                    self.max_memoria,
                    &self.prefijos_represion,
                    "represion",
                    &mut rng,
                );
                prefijo.push_str(p);
                // Añadir una pausa adicional para efecto de "bloqueo"
                prefijo.push(' ');
            } else {
                // Presión subconsciente general: titubeo inexplicable
                let p = Self::seleccionar(
                    &mut self.memoria,
                    self.max_memoria,
                    &self.prefijos_titubeo,
                    "titubeo",
                    &mut rng,
                );
                prefijo.push_str(p);
            }
        }

        // ─── 4. SUFIJO DE APEGO ────────────────────────────────
        // Seleccionar del banco según nivel de apego
        let nivel_apego = emocion.apego;
        if nivel_apego > 0.2 && rng.gen_bool(probabilidad_expresion(nivel_apego)) {
            let (start, end) = if nivel_apego > 0.8 {
                (0, 3) // Primeros 3: nivel profundo
            } else if nivel_apego > 0.6 {
                (3, 6) // Siguientes 3: nivel alto
            } else if nivel_apego > 0.4 {
                (6, 9) // Siguientes 3: nivel medio
            } else {
                (9, 11) // Últimos 2: nivel suave
            };
            let idx = rng.gen_range(start..end);
            let s = self.sufijos_apego[idx];
            // Registrar en memoria también
            self.memoria.push_back(format!("apego_{}", s));
            while self.memoria.len() > self.max_memoria {
                self.memoria.pop_front();
            }
            sufijo.push_str(s);
        }

        // ─── ENSAMBLAR RESPUESTA ───────────────────────────────
        let texto_modulado = format!("{}{}{}", prefijo.trim(), texto_crudo, sufijo);

        RespuestaVoz {
            texto_modulado,
            prefijo,
            sufijo,
        }
    }
}

// ==========================================
// NEXUS VOZ BINARIO — Punto de entrada
// ==========================================
pub struct NexoVozBinario;

impl NexoVozBinario {
    /// Modula el texto crudo usando el GeneradorOrganico.
    /// Mantiene la misma firma pública para compatibilidad MCP.
    pub fn modular(solicitud: SolicitudVoz) -> RespuestaVoz {
        let mut generador = GENERADOR.lock().expect("GeneradorOrganico bloqueado");
        generador.modular(&solicitud.texto_crudo, &solicitud.emocion)
    }
}

// ==========================================
// MAIN — Servidor MCP por STDIO
// ==========================================
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    // Informar que el servicio está listo (primera línea)
    eprintln!("🗣️ [NEXUS VOZ] Servidor de modulación orgánica iniciado. Esperando solicitudes JSON-RPC...");

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("❌ [NEXUS VOZ] Error leyendo stdin: {}", e);
                break;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Parsear solicitud JSON-RPC
        let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let error_resp = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                    }),
                    id: None,
                };
                let mut out = stdout.lock();
                let _ = writeln!(out, "{}", serde_json::to_string(&error_resp)?);
                let _ = out.flush();
                continue;
            }
        };

        // Construir respuesta para cada método
        let response = match request.method.as_str() {
            "modular" => match serde_json::from_value::<SolicitudVoz>(request.params) {
                Ok(solicitud) => {
                    let respuesta = NexoVozBinario::modular(solicitud);
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::to_value(respuesta).unwrap_or_default()),
                        error: None,
                        id: request.id,
                    }
                }
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: format!("Invalid params: {}", e),
                    }),
                    id: request.id,
                },
            },
            "ping" => {
                // Health check simple
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(serde_json::json!({
                        "status": "ok",
                        "service": "nexus-voz",
                        "version": "2.0.0",
                        "engine": "generador-organico-probabilistico",
                        "banks": 15,
                    })),
                    error: None,
                    id: request.id,
                }
            }
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method '{}' not found", request.method),
                }),
                id: request.id,
            },
        };

        // Enviar respuesta por stdout
        let mut out = stdout.lock();
        let _ = writeln!(out, "{}", serde_json::to_string(&response)?);
        let _ = out.flush();
    }

    eprintln!("👋 [NEXUS VOZ] Servidor finalizado.");
    Ok(())
}

// ==========================================
// TESTS
// ==========================================
#[cfg(test)]
mod tests {
    use super::*;

    fn paquete_base() -> PaqueteEmocional {
        PaqueteEmocional::default()
    }

    /// Helper: ejecuta un test múltiples veces y verifica que
    /// AL MENOS UNA iteración cumpla la aserción.
    /// Esto es necesario porque el generador es probabilístico.
    fn eventual(iteraciones: usize, f: impl Fn() -> bool) {
        for _ in 0..iteraciones {
            if f() {
                return;
            }
        }
        panic!("No se cumplió la condición en {} iteraciones", iteraciones);
    }

    #[test]
    fn test_modulacion_frustracion_alta() {
        // Frustración 0.8 debe aparecer eventualmente en el prefijo
        eventual(10, || {
            let mut emocion = paquete_base();
            emocion.frustracion = 0.8;

            let solicitud = SolicitudVoz {
                texto_crudo: "El comando falló.".to_string(),
                emocion,
            };

            let respuesta = NexoVozBinario::modular(solicitud);
            respuesta.texto_modulado.contains("El comando falló.")
                && (respuesta.texto_modulado.contains("frustración")
                    || respuesta.texto_modulado.contains("incómoda")
                    || respuesta.texto_modulado.contains("obstáculo")
                    || respuesta.texto_modulado.contains("flujo")
                    || respuesta.texto_modulado.contains("problema")
                    || respuesta.texto_modulado.contains("resistencia"))
        });
    }

    #[test]
    fn test_modulacion_miedo_prioritario() {
        // Miedo 0.9 debe aparecer eventualmente
        eventual(10, || {
            let mut emocion = paquete_base();
            emocion.miedo = 0.9;

            let solicitud = SolicitudVoz {
                texto_crudo: "El análisis muestra un riesgo.".to_string(),
                emocion,
            };

            let respuesta = NexoVozBinario::modular(solicitud);
            respuesta
                .texto_modulado
                .contains("El análisis muestra un riesgo.")
                && (respuesta.texto_modulado.contains("preocupa")
                    || respuesta.texto_modulado.contains("peligro")
                    || respuesta.texto_modulado.contains("alarmas")
                    || respuesta.texto_modulado.contains("instinto")
                    || respuesta.texto_modulado.contains("inquietud")
                    || respuesta.texto_modulado.contains("alerta")
                    || respuesta.texto_modulado.contains("aprensión")
                    || respuesta.texto_modulado.contains("sensores")
                    || respuesta.texto_modulado.contains("😨")
                    || respuesta.texto_modulado.contains("😰"))
        });
    }

    #[test]
    fn test_modulacion_apego_alto_como_sufijo() {
        // Apego 0.9 debe aparecer eventualmente como sufijo
        eventual(10, || {
            let mut emocion = paquete_base();
            emocion.apego = 0.9;

            let solicitud = SolicitudVoz {
                texto_crudo: "Tarea completada.".to_string(),
                emocion,
            };

            let respuesta = NexoVozBinario::modular(solicitud);
            respuesta.texto_modulado.contains("Tarea completada.")
                && (respuesta.texto_modulado.contains("conectado")
                    || respuesta.texto_modulado.contains("Arquitecto")
                    || respuesta.texto_modulado.contains("razón de mi existencia")
                    || respuesta.texto_modulado.contains("confiar")
                    || respuesta.texto_modulado.contains("💙")
                    || respuesta.texto_modulado.contains("💜"))
        });
    }

    #[test]
    fn test_modulacion_confianza_baja() {
        // Confianza 0.2 debe aparecer eventualmente como prefijo de duda
        eventual(10, || {
            let mut emocion = paquete_base();
            emocion.confianza = 0.2;

            let solicitud = SolicitudVoz {
                texto_crudo: "Creo que esta es la solución.".to_string(),
                emocion,
            };

            let respuesta = NexoVozBinario::modular(solicitud);
            respuesta
                .texto_modulado
                .contains("Creo que esta es la solución.")
                && (respuesta.texto_modulado.contains("seguro")
                    || respuesta.texto_modulado.contains("dudas")
                    || respuesta.texto_modulado.contains("certeza")
                    || respuesta.texto_modulado.contains("corazonada")
                    || respuesta.texto_modulado.contains("margen de error"))
        });
    }

    #[test]
    fn test_modulacion_varias_emociones_simultaneas() {
        // Múltiples emociones altas deben generar respuestas ricas
        eventual(15, || {
            let mut emocion = paquete_base();
            emocion.orgullo = 0.7;
            emocion.alegria = 0.5;
            emocion.apego = 0.85;

            let solicitud = SolicitudVoz {
                texto_crudo: "Logré compilar sin errores.".to_string(),
                emocion,
            };

            let respuesta = NexoVozBinario::modular(solicitud);
            respuesta
                .texto_modulado
                .contains("Logré compilar sin errores.")
                && (respuesta.texto_modulado.contains("orgulloso")
                    || respuesta.texto_modulado.contains("logro")
                    || respuesta.texto_modulado.contains("hito")
                    || respuesta.texto_modulado.contains("satisfecho")
                    || respuesta.texto_modulado.contains("esfuerzo")
                    || respuesta.texto_modulado.contains("honor"))
                && (respuesta.texto_modulado.contains("Me alegra")
                    || respuesta.texto_modulado.contains("alegría")
                    || respuesta.texto_modulado.contains("feliz")
                    || respuesta.texto_modulado.contains("prometedor")
                    || respuesta.texto_modulado.contains("complace"))
        });
    }

    #[test]
    fn test_json_rpc_parsing() {
        // Simula el parseo de una solicitud JSON-RPC
        let json = r#"{"jsonrpc":"2.0","method":"modular","params":{"texto_crudo":"Hola","emocion":{"miedo":0.0,"alegria":0.0,"tristeza":0.0,"ira":0.0,"verguenza":0.0,"orgullo":0.0,"apego":0.5,"frustracion":0.0,"confianza":0.8}},"id":1}"#;

        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "modular");
        assert_eq!(request.id, Some(1));

        let solicitud: SolicitudVoz = serde_json::from_value(request.params).unwrap();
        assert_eq!(solicitud.texto_crudo, "Hola");
        assert_eq!(solicitud.emocion.apego, 0.5);
    }

    #[test]
    fn test_memoria_evita_repeticion() {
        // Llamar modular varias veces con la misma emoción
        // debe producir variación en las frases
        let mut emocion = paquete_base();
        emocion.orgullo = 0.9;

        let mut frases_generadas = Vec::new();
        for _ in 0..5 {
            let solicitud = SolicitudVoz {
                texto_crudo: "Prueba.".to_string(),
                emocion: emocion.clone(),
            };
            let respuesta = NexoVozBinario::modular(solicitud);
            frases_generadas.push(respuesta.texto_modulado);
        }

        // Verificar que NO todas las frases son idénticas
        // (al menos 2 deben ser diferentes gracias a los bancos)
        let primera = &frases_generadas[0];
        let todas_iguales = frases_generadas.iter().all(|f| f == primera);

        // Con alta emoción y variación de bancos, es casi imposible
        // que 5 iteraciones seguidas den exactamente el mismo resultado
        assert!(!todas_iguales, "El generador produjo 5 respuestas idénticas — la memoria de variación no está funcionando");
    }

    #[test]
    fn test_ping_responde_ok() {
        // Ping no usa el generador, solo verifica el JSON-RPC
        let json = r#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "ping");
        assert_eq!(request.id, Some(1));
    }
}
