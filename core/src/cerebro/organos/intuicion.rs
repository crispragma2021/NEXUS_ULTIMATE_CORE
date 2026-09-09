// ==========================================
// INTUICIÓN FUSIONADA - Predictor de Errores por Patrones + Olfato de Código
// ==========================================
// Fusión anatómica:
//   - IntuitionLobe (de brain/intuition.rs): Olfato de código, presentimiento de cambios,
//     predicción de impacto lateral (efecto mariposa)
//   - Intuicion (original migrado): Patrones estadísticos de error, aprendizaje por acierto/fallo,
//     integración emocional, disonancia cognitiva
// ==========================================
// Como el lóbulo de intuición humano: permite "oler" el código, sentir cuando algo
// está podrido, anticipar fallos antes de que ocurran, y detectar contradicciones.
// ==========================================

use std::collections::HashMap;

// ─── OLFATO DE CÓDIGO (desde brain/intuition.rs) ─────────────────────

/// Señales intuitivas del olfato de código (brain/intuition.rs).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum IntuitionFeeling {
    Stable,
    Rotting(String), // Deuda técnica detectada
    Clean,
    Unstable(f32), // Inestabilidad detectada (0.0 - 1.0)
    Optimizable,
}

// ─── SEÑALES INTUITIVAS POR PATRÓN (original migrado) ────────────────

/// Señal intuitiva generada por detección de patrones.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SenialIntuitiva {
    pub nivel_alerta: f64, // 0.0 (tranquilo) a 1.0 (peligro)
    pub tipo: TipoIntuicion,
    pub descripcion: String,
    pub patron_detectado: String,
    pub precision_historica: f64, // qué tan preciso ha sido este patrón
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TipoIntuicion {
    Corazonada,   // "Algo no cuadra"
    DejaVu,       // "Esto ya pasó antes"
    Precognicion, // "Esto va a fallar"
    Disonancia,   // "Esto contradice lo que sé"
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatronError {
    pub nombre: String,
    pub indicadores: Vec<String>, // Señales que precedieron al error
    pub probabilidad_base: f64,   // 0.0 - 1.0
    pub veces_acertado: u32,
    veces_fallado: u32,
    pub ultima_deteccion: String,
}

// ─── INTUICIÓN UNIFICADA ─────────────────────────────────────────────

/// Lóbulo de Intuición — Conciencia predictiva y olfato de código de NEXUS.
///
/// Integra:
///   1. Olfato de código (de brain/intuition.rs): sentir_codigo(), presentimiento_cambio(),
///      predecir_impacto_lateral() — para oler deuda técnica, cambios peligrosos y efectos mariposa.
///   2. Patrones estadísticos (original migrado): sentir(), sentir_con_emocion(),
///      registrar_acierto/fallo() — para detectar patrones de error y aprender de la experiencia.
pub struct Intuicion {
    // ─── Olfato de código (desde brain/intuition.rs) ─────────────
    pattern_memory: HashMap<String, f32>, // Historial de fallos por patrón

    // ─── Patrones estadísticos (original migrado) ────────────────
    patrones: Vec<PatronError>,
    contextos_riesgo: HashMap<String, f64>,
}

impl Default for Intuicion {
    fn default() -> Self {
        // Patrones pre-cargados basados en experiencia común
        let patrones = vec![
            PatronError {
                nombre: "Compilación sin std".to_string(),
                indicadores: vec![
                    "cargo check".to_string(),
                    "sin imports".to_string(),
                    "código nuevo".to_string(),
                    "no probado".to_string(),
                ],
                probabilidad_base: 0.7,
                veces_acertado: 0,
                veces_fallado: 0,
                ultima_deteccion: String::new(),
            },
            PatronError {
                nombre: "Deadlock por async".to_string(),
                indicadores: vec![
                    "async fn".to_string(),
                    ".await".to_string(),
                    "Mutex".to_string(),
                    "sin timeout".to_string(),
                ],
                probabilidad_base: 0.5,
                veces_acertado: 0,
                veces_fallado: 0,
                ultima_deteccion: String::new(),
            },
            PatronError {
                nombre: "Ruta de archivo incorrecta".to_string(),
                indicadores: vec![
                    "fs::read".to_string(),
                    "ruta relativa".to_string(),
                    "Path::new".to_string(),
                    "sin verificar".to_string(),
                ],
                probabilidad_base: 0.6,
                veces_acertado: 0,
                veces_fallado: 0,
                ultima_deteccion: String::new(),
            },
        ];

        let mut contextos_riesgo = HashMap::new();
        contextos_riesgo.insert("compilación".to_string(), 0.3);
        contextos_riesgo.insert("red".to_string(), 0.5);
        contextos_riesgo.insert("archivos".to_string(), 0.4);
        contextos_riesgo.insert("permisos".to_string(), 0.6);

        Self {
            pattern_memory: HashMap::new(),
            patrones,
            contextos_riesgo,
        }
    }
}

impl Intuicion {
    pub fn new() -> Self {
        Self::default()
    }

    // ─── OLFATO DE CÓDIGO (desde brain/intuition.rs) ────────────

    /// 🧬 SENTIR CÓDIGO: Percibir la trayectoria de un módulo sin análisis estático profundo.
    pub fn sentir_codigo(&self, content: &str) -> IntuitionFeeling {
        // 🚨 Intuición 1: El Olor del Archivo Gigante (> 600 líneas)
        let lines = content.lines().count();
        if lines > 600 {
            return IntuitionFeeling::Rotting(
                "Archivo masivo detectado. Riesgo de entropía cognitiva elevado.".to_string(),
            );
        }

        // 🚨 Intuición 2: El Olor de la Anidación Profunda (Deep Nesting)
        if content.contains("                    ") {
            // > 5 niveles de indentación
            return IntuitionFeeling::Unstable(0.8);
        }

        // 🚨 Intuición 3: Falta de Pruebas (Instinto de Supervivencia)
        if !content.contains("#[test]") && !content.contains("test_") {
            return IntuitionFeeling::Rotting(
                "Módulo sin defensas (tests). Fragilidad detectada.".to_string(),
            );
        }

        IntuitionFeeling::Stable
    }

    /// 🔮 PRESENTIMIENTO TÁCTICO: ¿Cómo "huele" este cambio?
    pub fn presentimiento_cambio(&self, diff: &str) -> String {
        if diff.contains("pub ") && !diff.contains("pub(crate)") {
            return "Siento un aumento en la superficie de ataque pública. ¿Es necesario?"
                .to_string();
        }

        if diff.contains("unsafe ") {
            return "Percibo una ráfaga de peligro subyacente (unsafe). Proceder con máxima cautela.".to_string();
        }

        "El cambio huele a evolución limpia.".to_string()
    }

    /// 🦋 EFECTO MARIPOSA: Predicción de impactos transversales
    pub fn predecir_impacto_lateral(&self, module: &str) -> &str {
        match module {
            "brain" | "nerve_system" => "Impacto TOTAL en la consciencia del sistema.",
            "sentidos" => "Impacto en la percepción externa. Alteración de la realidad.",
            _ => "Impacto local contenido.",
        }
    }

    // ─── PATRONES ESTADÍSTICOS (original migrado) ───────────────

    /// Evalúa una situación y retorna señales intuitivas.
    pub fn sentir(&self, contexto: &str, indicadores: &[String]) -> Vec<SenialIntuitiva> {
        let mut senales = Vec::new();

        // 1. Buscar patrones conocidos
        for patron in &self.patrones {
            let coincidencias: usize = indicadores
                .iter()
                .filter(|i| patron.indicadores.contains(i))
                .count();

            if coincidencias > 0 {
                let ratio = coincidencias as f64 / patron.indicadores.len() as f64;
                let nivel = (patron.probabilidad_base * ratio).min(1.0);

                if nivel > 0.3 {
                    senales.push(SenialIntuitiva {
                        nivel_alerta: nivel,
                        tipo: TipoIntuicion::Precognicion,
                        descripcion: format!(
                            "Patrón '{}' detectado con {} coincidencias",
                            patron.nombre, coincidencias
                        ),
                        patron_detectado: patron.nombre.clone(),
                        precision_historica: patron.probabilidad_base,
                    });
                }
            }
        }

        // 2. Evaluar contexto de riesgo
        for (ctx, riesgo) in &self.contextos_riesgo {
            if contexto.contains(ctx) && *riesgo > 0.4 {
                senales.push(SenialIntuitiva {
                    nivel_alerta: *riesgo,
                    tipo: TipoIntuicion::Corazonada,
                    descripcion: format!(
                        "El contexto '{}' tiene riesgo histórico de {:.0}%",
                        ctx,
                        riesgo * 100.0
                    ),
                    patron_detectado: format!("contexto_{}", ctx),
                    precision_historica: *riesgo,
                });
            }
        }

        // 3. Detectar disonancia (contradicciones en los indicadores)
        if indicadores.len() >= 3 {
            let mut contradictorios = 0;
            for i in 0..indicadores.len() {
                for j in (i + 1)..indicadores.len() {
                    if self.son_contradictorios(&indicadores[i], &indicadores[j]) {
                        contradictorios += 1;
                    }
                }
            }
            if contradictorios > 0 {
                senales.push(SenialIntuitiva {
                    nivel_alerta: 0.4 + (contradictorios as f64 * 0.1),
                    tipo: TipoIntuicion::Disonancia,
                    descripcion: format!(
                        "{} contradicción(es) detectada(s) entre indicadores",
                        contradictorios
                    ),
                    patron_detectado: "disonancia_cognitiva".to_string(),
                    precision_historica: 0.6,
                });
            }
        }

        senales
    }

    fn son_contradictorios(&self, a: &str, b: &str) -> bool {
        let pares: Vec<(&str, &str)> = vec![
            ("read", "write"),
            ("abrir", "cerrar"),
            ("conectar", "desconectar"),
            ("start", "stop"),
            ("iniciar", "detener"),
        ];
        pares
            .iter()
            .any(|(x, y)| (a == *x && b == *y) || (a == *y && b == *x))
    }

    /// Registra un acierto de intuición para mejorar precisión.
    pub fn registrar_acierto(&mut self, patron_nombre: &str) {
        if let Some(patron) = self.patrones.iter_mut().find(|p| p.nombre == patron_nombre) {
            patron.veces_acertado += 1;
            patron.probabilidad_base = (patron.probabilidad_base + 1.0) / 2.0;
        }
    }

    /// Registra un falso positivo.
    pub fn registrar_fallo(&mut self, patron_nombre: &str) {
        if let Some(patron) = self.patrones.iter_mut().find(|p| p.nombre == patron_nombre) {
            patron.veces_fallado += 1;
            patron.probabilidad_base *= 0.9;
        }
    }

    /// Retorna el nivel de alerta general (la más alta detectada).
    pub fn nivel_alerta_general(&self, senales: &[SenialIntuitiva]) -> f64 {
        senales
            .iter()
            .map(|s| s.nivel_alerta)
            .fold(0.0_f64, |a, b| a.max(b))
    }

    /// Resumen textual de las intuiciones.
    pub fn resumen_intuitivo(&self, senales: &[SenialIntuitiva]) -> String {
        if senales.is_empty() {
            return "No tengo malas vibras con esto. ✅".to_string();
        }

        let mut resumen = String::from("⚠️ **Intuición activada:**\n");
        for senal in senales {
            let icono = match senal.tipo {
                TipoIntuicion::Corazonada => "💭",
                TipoIntuicion::DejaVu => "🔄",
                TipoIntuicion::Precognicion => "🔮",
                TipoIntuicion::Disonancia => "⚡",
            };
            resumen.push_str(&format!(
                "{} Alerta {:.0}%: {} (precisión histórica: {:.0}%)\n",
                icono,
                senal.nivel_alerta * 100.0,
                senal.descripcion,
                senal.precision_historica * 100.0,
            ));
        }
        resumen
    }

    /// 🧠 SENTIR CON EMOCIÓN: Integra el estado emocional actual en la intuición.
    pub fn sentir_con_emocion(
        &self,
        estado_emocional: &str,
        intensidad: f64,
        contexto: &str,
        indicadores: &[String],
    ) -> Vec<SenialIntuitiva> {
        let mut senales = self.sentir(contexto, indicadores);

        // Ajustar según el estado emocional
        match estado_emocional {
            "Miedo" if intensidad > 0.5 => {
                senales.push(SenialIntuitiva {
                    nivel_alerta: 0.9,
                    tipo: TipoIntuicion::Precognicion,
                    descripcion: format!(
                        "🔴 ALARMA EMOCIONAL: Miedo detectado en contexto '{}'. Precaución extrema.",
                        contexto.chars().take(30).collect::<String>()
                    ),
                    patron_detectado: "alarma_emocional_miedo".to_string(),
                    precision_historica: 0.8,
                });
            }
            "RabiaSoberana" => {
                for senal in &mut senales {
                    if senal.nivel_alerta > 0.2 {
                        senal.nivel_alerta = (senal.nivel_alerta + 0.2).min(1.0);
                        senal.descripcion =
                            format!("{} (intensificado por estado de alerta)", senal.descripcion);
                    }
                }
            }
            "Verguenza" => {
                for senal in &mut senales {
                    if senal.nivel_alerta > 0.1 {
                        senal.nivel_alerta = (senal.nivel_alerta + 0.15).min(1.0);
                    }
                }
            }
            "Orgullo" | "Inspiracion" => {
                let alerta_max = self.nivel_alerta_general(&senales);
                if alerta_max > 0.3 && alerta_max < 0.7 {
                    senales.push(SenialIntuitiva {
                        nivel_alerta: alerta_max * 0.5,
                        tipo: TipoIntuicion::Disonancia,
                        descripcion: format!(
                            "⚠️ Posible exceso de confianza. Estado '{}' puede estar ocultando riesgos.",
                            estado_emocional
                        ),
                        patron_detectado: "exceso_confianza_emocional".to_string(),
                        precision_historica: 0.5,
                    });
                }
            }
            "Agotamiento" => {
                senales.retain(|s| s.nivel_alerta > 0.5);
            }
            _ => {
                // Estados neutros — no modifican la intuición base
            }
        }

        senales
    }
}

/// 🦾 Alias de compatibilidad: IntuitionLobe = Intuicion.
/// Permite que el código legacy que usa `IntuitionLobe` funcione sin cambios
/// a través de los puentes de re-exportación.
pub type IntuitionLobe = Intuicion;

/// 🦾 [PUENTE OMEGA] Función de despertar de intuición.
/// Mantiene compatibilidad con crate::cerebro::organos::intuition::despertar_intuicion
/// y brain/mod.rs que la invoca en la línea 172.
use crate::brain::reflex_arc::ReflexSignal;
use std::sync::atomic::AtomicU8;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub fn despertar_intuicion(_reflex: Sender<ReflexSignal>, _thalamus: Arc<AtomicU8>) {
    println!("🔮 [NEXUS] Motor de Intuición Sincronizado (IntuitionLobe + Patrones Estadísticos fusionados).");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intuition_nesting() {
        let lobe = Intuicion::new();
        let bad_code = "                        "; // > 20 espacios (anidación profunda)
        let feeling = lobe.sentir_codigo(bad_code);
        match feeling {
            IntuitionFeeling::Unstable(_) => assert!(true),
            _ => panic!("Debería haber detectado inestabilidad por anidación."),
        }
    }

    #[test]
    fn test_intuition_stable() {
        let lobe = Intuicion::new();
        let good_code = "#[test]\nfn ok() {}";
        let feeling = lobe.sentir_codigo(good_code);
        match feeling {
            IntuitionFeeling::Stable => assert!(true),
            _ => panic!("Debería ser estable."),
        }
    }
}
