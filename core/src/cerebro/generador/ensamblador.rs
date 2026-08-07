// ============================================================================
// 🧠 CORTEZA MOTORA — Ensamblador de Voz
// ============================================================================
// Propósito: Toma una ruta narrativa y produce texto listo para modulación.
//
// Capa 4 del GOI: después de seleccionar la ruta (Capa 3), esta capa
//   ensambla el texto final según el tipo de ruta:
//   - Directa: usa el fragmento tal cual
//   - Síntesis: une fragmentos con transiciones
//   - Exploración: usa SintetizadorBroca expandido
//   - Silencio: frase predefinida
// ============================================================================

use crate::cerebro::generador::selector_ruta::RutaNarrativa;
use crate::cerebro::synapse::SintetizadorBroca;

/// Ensambla texto a partir de una ruta narrativa y un nivel de restricción.
///
/// El `nivel_restriccion` (0.0–0.9) representa fricción semántica:
///   - ≤ 0.3 → tono neutro o positivo (libre)
///   - 0.3–0.6 → tono cauto, analítico
///   - ≥ 0.6 → tono serio, directo, sintético (trauma activo)
pub struct EnsambladorVoz {
    /// Sintetizador Broca (reutilizado para la ruta de Exploración y safety net).
    pub broca: SintetizadorBroca,
}

impl EnsambladorVoz {
    /// Crea una nueva instancia del ensamblador.
    pub fn new() -> Self {
        Self {
            broca: SintetizadorBroca::new(),
        }
    }

    /// Toma una ruta narrativa y un nivel de restricción y produce texto
    /// listo para modulación. Cuanto mayor la restricción, más cauto y
    /// sintético el tono.
    ///
    /// # Parámetros
    /// - `ruta`: La ruta narrativa seleccionada por los Ganglios Basales.
    /// - `nivel_restriccion`: Fricción semántica 0.0–0.9.
    ///
    /// # Retorna
    /// Texto crudo listo para pasar a VozMCP::modular().
    pub fn ensamblar(&self, ruta: RutaNarrativa, nivel_restriccion: f64) -> String {
        let tono = if nivel_restriccion >= 0.5 {
            TonoVoz::Serio
        } else if nivel_restriccion >= 0.2 {
            TonoVoz::Cauto
        } else if nivel_restriccion <= 0.01 {
            // Sin restricción → tono ligero (éxito o normalidad)
            TonoVoz::Alegre
        } else {
            TonoVoz::Neutro
        };

        match ruta {
            RutaNarrativa::Directa(fragmento) => {
                if fragmento.texto.is_empty() {
                    String::new()
                } else {
                    match tono {
                        TonoVoz::Serio => {
                            format!("⚠️ {}. Sugiero revisar con cuidado.", fragmento.texto)
                        }
                        TonoVoz::Cauto => {
                            format!("🤔 {}. Tenemos que evaluarlo.", fragmento.texto)
                        }
                        TonoVoz::Alegre => {
                            format!("✨ {}. ¡Me gusta!", fragmento.texto)
                        }
                        TonoVoz::Neutro => fragmento.texto,
                    }
                }
            }
            RutaNarrativa::Sintesis(fragmentos, _hilo) => {
                if fragmentos.is_empty() {
                    return String::new();
                }
                let mut partes: Vec<String> = fragmentos
                    .into_iter()
                    .filter(|f| !f.texto.is_empty())
                    .map(|f| f.texto)
                    .collect();
                partes.dedup();
                if partes.is_empty() {
                    String::new()
                } else {
                    match tono {
                        TonoVoz::Serio => partes.join(" → "),
                        TonoVoz::Cauto => partes.join("... "),
                        TonoVoz::Alegre => partes.join(" ¡y "),
                        TonoVoz::Neutro => partes.join("... "),
                    }
                }
            }
            RutaNarrativa::Exploracion(raiz) => {
                let mut concepto = raiz;
                if matches!(tono, TonoVoz::Serio) {
                    concepto = format!("análisis de {}", concepto);
                } else if matches!(tono, TonoVoz::Alegre) {
                    concepto = format!("{} creativo", concepto);
                }
                let conceptos = vec![(concepto, 0.85)];
                self.broca.sintetizar(&conceptos)
            }
            RutaNarrativa::Silencio(frase) => frase.to_string(),
        }
    }
}

/// Nivel de tono que refleja la fricción semántica.
enum TonoVoz {
    /// Restricción ≥ 0.5: trauma activo, tono serio y sintético
    Serio,
    /// Restricción 0.2–0.5: precaución, tono cauto
    Cauto,
    /// Restricción ≤ 0.01: éxito, tono alegre y ligero
    Alegre,
    /// Sin fricción: tono neutro
    Neutro,
}

impl Default for EnsambladorVoz {
    fn default() -> Self {
        Self::new()
    }
}
