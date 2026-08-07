// ============================================================================
// 🧠 SAE v2 — CAPA BIOLÓGICA (Neuromodulación + Homeostasis + IGG sobre el núcleo)
// ============================================================================
// ADR SAE-002: la capa biológica del SAE v1 se CONSERVA, pero ahora actúa
// SOBRE el núcleo numérico (BioTransformerCore) en vez de sobre spikes puros.
//
//  1. NEUROMODULACIÓN: la Neuroquimica (DA/5HT/NA/CORT/OXI) modula en tiempo
//     real los parámetros de generación del núcleo:
//       - Dopamina alta  → mayor consumo de intención (mensaje directo) y
//                          mayor factor de aprendizaje (refuerzo).
//       - Cortisol alto  → umbral ε más alto (se expresa menos, túnel).
//       - Adrenalina alta→ más ruido/exploración (asociaciones remotas).
//       - Serotonina alta→ estabilidad (menos ruido, muestreo más agudo).
//       - Oxitocina alta → refuerzo del vínculo (escala el feedback del Juez).
//
//  2. HOMEOSTASIS: cada paso de generación regula el umbral de parada según
//     la tasa de disparo estimada; evita bucle o silencio patológico.
//
//  3. IGG (Generador Guiado por Intención): el núcleo produce la preferencia
//     del token p_t; la capa biológica la integra con el residual R_t de la
//     intención M (descomposición tipo Gram-Schmidt) — el mensaje se REDUCE
//     hasta ‖R‖ < ε, no se "predice" hasta agotar tokens.
// ============================================================================

use crate::cerebro::sae::nucleo_numerico::Vocabulario;
use crate::cerebro::sistema_limbico::Neuroquimica;

/// Configuración de la capa biológica.
#[derive(Debug, Clone)]
pub struct CapaBiologicaConfig {
    /// α base de consumo de intención por token.
    pub alfa_base: f32,
    /// ε base (homeostasis): ‖R‖ bajo esto → silencio.
    pub epsilon_base: f32,
    /// Longitud máxima de secuencia (cortafuegos).
    pub max_tokens: usize,
    /// Ruido base de exploración.
    pub ruido_base: f32,
    /// Mínimo de α (evita bucles infinitos).
    pub alfa_min: f32,
    /// Máximo de ε (evita silencio prematuro).
    pub epsilon_max: f32,
}

impl Default for CapaBiologicaConfig {
    fn default() -> Self {
        Self {
            alfa_base: 0.35,
            epsilon_base: 0.08,
            max_tokens: 24,
            ruido_base: 0.05,
            alfa_min: 0.10,
            epsilon_max: 0.30,
        }
    }
}

/// Parámetros de generación ya modulados por la neuroquímica.
#[derive(Debug, Clone)]
pub struct ParametrosModulados {
    /// Consumo de intención efectivo (α').
    pub alfa: f32,
    /// Umbral de homeostasis efectivo (ε').
    pub epsilon: f32,
    /// Temperatura del muestreo del núcleo.
    pub temperatura: f32,
    /// Ruido de selección.
    pub ruido: f32,
    /// Factor de aprendizaje (para escalar el lr en backprop).
    pub factor_aprendizaje: f32,
}

/// Capa biológica: modula el núcleo numérico y orquesta el IGG.
#[derive(Debug, Clone)]
pub struct CapaBiologica {
    pub config: CapaBiologicaConfig,
}

impl Default for CapaBiologica {
    fn default() -> Self {
        Self::new(CapaBiologicaConfig::default())
    }
}

impl CapaBiologica {
    pub fn new(config: CapaBiologicaConfig) -> Self {
        Self { config }
    }

    /// Aplica neuromodulación sobre los parámetros base.
    pub fn modular(&self, neuro: &Neuroquimica) -> ParametrosModulados {
        let cfg = &self.config;

        // α': dopamina acelera el consumo (0.35 base → hasta ~0.55).
        let alfa =
            (cfg.alfa_base * (1.0 + 0.6 * neuro.dopamina)).clamp(cfg.alfa_min, 0.9);

        // ε': cortisol sube el umbral (túnel cognitivo → se expresa menos);
        // oxitocina baja el umbral (confianza → se explaya más).
        let epsilon = (cfg.epsilon_base * (1.0 + 0.5 * neuro.cortisol)
            - 0.02 * neuro.oxitocina)
            .clamp(0.01, cfg.epsilon_max);

        // Temperatura: adrenalina aumenta exploración; serotonina la estabiliza.
        let temperatura = (0.8 + 0.6 * neuro.adrenalina - 0.4 * neuro.serotonina)
            .clamp(0.1, 2.0);

        // Ruido noradrenérgico.
        let ruido = (cfg.ruido_base + 0.35 * neuro.adrenalina).min(1.0);

        // Factor de aprendizaje: dopamina + oxitocina aceleran el backprop
        // (refuerzo) — conecta el sistema límbico con la velocidad de
        // aprendizaje del núcleo (equivalente a una escala de lr dinámica).
        let factor_aprendizaje = (0.5 + 0.5 * neuro.dopamina + 0.3 * neuro.oxitocina)
            .clamp(0.1, 2.0);

        ParametrosModulados {
            alfa,
            epsilon,
            temperatura,
            ruido,
            factor_aprendizaje,
        }
    }

    /// Genera una secuencia guiada por intención usando el núcleo numérico.
    ///
    /// - `intencion`: vector M (estado + memoria) — la intención completa.
    /// - `semilla`: ids iniciales (p. ej. BOS) para el núcleo.
    /// - `vocabulario`: para convertir ids ↔ tokens.
    /// - `nucleo_generar`: cierre que pide al núcleo el siguiente id dado el
    ///   contexto y la temperatura (evita acoplar candle aquí).
    /// - `neuro`: neuroquímica actual.
    ///
    /// Algoritmo IGG:
    ///   R₀ = M (normalizado)
    ///   por cada paso:
    ///     p_t = preferencia del token (del núcleo, ya con temperatura)
    ///     g   = ⟨R_t, p_t⟩  (cuánto expresa el token de la intención)
    ///     R_{t+1} = R_t − α·g·p_t   (Gram-Schmidt: consumo del significado)
    ///     STOP si ‖R_{t+1}‖ < ε
    ///   Cada token además recibe ruido noradrenérgico en la selección.
    pub fn generar_con_igg<F>(
        &self,
        intencion: &[f32],
        semilla: &[usize],
        vocabulario: &Vocabulario,
        neuro: &Neuroquimica,
        mut nucleo_generar: F,
    ) -> Result<SecuenciaIgg, String>
    where
        F: FnMut(&[usize], f32) -> Result<usize, String>,
    {
        let p = self.modular(neuro);
        let cfg = &self.config;

        // R₀ = M normalizado.
        let mut r: Vec<f32> = intencion.to_vec();
        let norm = r.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 {
            for x in r.iter_mut() {
                *x /= norm;
            }
        }

        let mut ids = semilla.to_vec();
        let mut steps = Vec::new();
        let mut residual_final = 0.0f32;

        for _ in 0..cfg.max_tokens {
            // Pedir el siguiente token al núcleo (temperatura modulada + ruido).
            let temp = p.temperatura * (1.0 + p.ruido);
            let id = nucleo_generar(&ids, temp)?;
            let token = vocabulario.token_para(id).to_string();

            // Preferencia p_t del token: embedding aproximado por one-hot
            // normalizado. En la implementación completa, el núcleo expone el
            // embedding aprendido; aquí usamos one-hot para mantener la capa
            // desacoplada de candle.
            let dim = r.len().max(1);
            let mut pref = vec![0.0f32; dim];
            if dim > 0 {
                pref[id % dim] = 1.0;
            }

            // g = ⟨R_t, p_t⟩ — cuánto del mensaje expresa este token.
            let g: f32 = r
                .iter()
                .zip(pref.iter())
                .map(|(a, b)| a * b)
                .sum::<f32>()
                .max(0.0);

            // R_{t+1} = R_t − α·g·p_t
            for i in 0..r.len() {
                r[i] -= p.alfa * g * pref[i];
            }

            let residual = r.iter().map(|x| x * x).sum::<f32>().sqrt();
            residual_final = residual;
            steps.push(PasoIgg {
                token: token.clone(),
                residual_norm: residual,
            });
            ids.push(id);

            // Homeostasis: ¿mensaje expresado?
            if residual < p.epsilon {
                break;
            }
        }

        Ok(SecuenciaIgg {
            tokens: ids,
            steps,
            residual_final,
            terminated_by_homeostasis: residual_final < p.epsilon,
        })
    }

    /// Regula la tasa de disparo (homeostasis clásica): devuelve un factor
    /// 0..1 que suaviza la actividad si la tasa media supera el objetivo.
    pub fn homeostasis_tasa(&self, tasas: &[f32], objetivo: f32) -> f32 {
        if tasas.is_empty() {
            return 1.0;
        }
        let media: f32 = tasas.iter().sum::<f32>() / tasas.len() as f32;
        if media <= objetivo {
            1.0
        } else {
            (objetivo / media).clamp(0.0, 1.0)
        }
    }

    /// Escala el lr del optimizador según el factor de aprendizaje
    /// (dopamina + oxitocina). `lr_base` es el lr configurado del AdamW.
    pub fn lr_efectivo(&self, lr_base: f64, neuro: &Neuroquimica) -> f64 {
        lr_base * self.modular(neuro).factor_aprendizaje as f64
    }
}

/// Un paso de la generación IGG.
#[derive(Debug, Clone)]
pub struct PasoIgg {
    pub token: String,
    pub residual_norm: f32,
}

/// Secuencia generada por el IGG sobre el núcleo numérico.
#[derive(Debug, Clone)]
pub struct SecuenciaIgg {
    /// Ids generados (semilla incluida).
    pub tokens: Vec<usize>,
    pub steps: Vec<PasoIgg>,
    pub residual_final: f32,
    pub terminated_by_homeostasis: bool,
}

impl SecuenciaIgg {
    /// Convierte ids → texto uniendo tokens.
    pub fn a_texto(&self, vocabulario: &Vocabulario) -> String {
        let mut out = String::new();
        for id in &self.tokens {
            let tok = vocabulario.token_para(*id);
            if out.is_empty() {
                out.push_str(tok);
            } else if tok.chars().next().is_some_and(|c| c.is_alphanumeric()) {
                out.push(' ');
                out.push_str(tok);
            } else {
                out.push_str(tok);
            }
        }
        out
    }
}
