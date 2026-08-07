use tracing::{info, warn};

/// 🧠 MECANISMO DE DUDA SISTEMÁTICA (MDS - NIVEL OMEGA)
/// Evolución del pensamiento crítico: De la precaución a la Verificación Soberana.
pub struct DudaSistematica {
    umbral_sabiduria: f32,
    intentos_maximos: u8,
}

impl Default for DudaSistematica {
    fn default() -> Self {
        Self {
            umbral_sabiduria: 0.85,
            intentos_maximos: 3,
        }
    }
}

impl DudaSistematica {
    pub fn new() -> Self {
        Self::default()
    }

    /// 🧪 EL MÉTODO DEL SABIO OMEGA
    /// Analiza una proposición y decide si requiere investigación externa o interna.
    pub async fn procesar_duda(&mut self, proposicion: &str) -> (bool, String) {
        let confianza = self.calcular_nivel_confianza(proposicion);

        if confianza < self.umbral_sabiduria {
            warn!(
                "🔍 [DUDA OMEGA] Confianza baja ({:.2}). Iniciando protocolo de verificación...",
                confianza
            );
            (
                true,
                format!("Requiere investigación profunda: {}", proposicion),
            )
        } else {
            info!(
                "✅ [DUDA OMEGA] Confianza sólida ({:.2}). Procediendo.",
                confianza
            );
            (false, proposicion.to_string())
        }
    }

    /// 🔍 EVALUACIÓN HEURÍSTICA DE CONFIANZA
    fn calcular_nivel_confianza(&self, proposicion: &str) -> f32 {
        let mut confianza: f32 = 0.85;

        // Paths absolutos conocidos = alta confianza
        if proposicion.contains("/home/soberano/NEXUS_ULTIMATE_CORE") {
            confianza += 0.10;
        }
        // Datos técnicos específicos
        if proposicion.contains("Ryzen")
            || proposicion.contains("RTX")
            || proposicion.contains("i7-12700F")
        {
            confianza += 0.05;
        }
        // Afirmaciones vagas = baja confianza
        if proposicion.len() < 20 {
            confianza -= 0.20;
        }
        // Timestamps recientes implícitos
        if proposicion.contains("2025") || proposicion.contains("2026") {
            confianza += 0.03;
        }

        confianza.clamp(0.0, 1.0)
    }
}
