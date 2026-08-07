// ==========================================
// DUDA SISTEMÁTICA OMEGA - Verificación Pre-Ejecución
// ==========================================
// Migrado de legacy/nexus-orquestador/src/reflejos/duda_sistematica.rs
// y legacy/nexus-orquestador/src/valores/juicio_habla.rs
//
// La Duda Sistemática es el mecanismo que activa NEXUS antes de
// ejecutar una acción en el mundo real (web, sistema de archivos, etc.)
// para verificar si realmente tiene la información necesaria.
// ==========================================

use tracing::{debug, info, warn};

/// Nivel de prioridad para el filtro de habla
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrioridadHabla {
    /// Peligro inminente - hablar inmediatamente
    Peligro,
    /// Mejora significativa - proponer al arquitecto
    Mejora,
    /// Duda crítica - necesito guía
    Duda,
    /// Lección valiosa - compartir sabiduría
    Leccion,
    /// Silencio soberano - no interrumpir
    Silencio,
}

/// Un "pensamiento" que NEXUS evalúa antes de expresarlo
pub struct Pensamiento {
    pub mensaje: String,
    pub es_peligro: bool,
    pub es_mejora: bool,
    pub es_duda_importante: bool,
    pub es_leccion: bool,
    pub gravedad: f32,    // 0.0 - 1.0
    pub impacto: f32,     // 0.0 - 1.0
    pub importancia: f32, // 0.0 - 1.0
}

impl Pensamiento {
    pub fn nuevo_peligro(mensaje: &str, gravedad: f32) -> Self {
        Self {
            mensaje: mensaje.to_string(),
            es_peligro: true,
            es_mejora: false,
            es_duda_importante: false,
            es_leccion: false,
            gravedad,
            impacto: 0.0,
            importancia: 0.0,
        }
    }

    pub fn nueva_mejora(mensaje: &str, impacto: f32) -> Self {
        Self {
            mensaje: mensaje.to_string(),
            es_peligro: false,
            es_mejora: true,
            es_duda_importante: false,
            es_leccion: false,
            gravedad: 0.0,
            impacto,
            importancia: 0.0,
        }
    }

    pub fn nueva_duda(mensaje: &str) -> Self {
        Self {
            mensaje: mensaje.to_string(),
            es_peligro: false,
            es_mejora: false,
            es_duda_importante: true,
            es_leccion: false,
            gravedad: 0.0,
            impacto: 0.0,
            importancia: 0.0,
        }
    }

    pub fn nueva_leccion(mensaje: &str, importancia: f32) -> Self {
        Self {
            mensaje: mensaje.to_string(),
            es_peligro: false,
            es_mejora: false,
            es_duda_importante: false,
            es_leccion: true,
            gravedad: 0.0,
            impacto: 0.0,
            importancia,
        }
    }
}

/// Sistema de Duda Sistemática: verifica proposiciones antes de actuar
pub struct DudaSistematica {
    /// Umbral de sabiduría (0.0 - 1.0). Por debajo de esto, se activa la duda
    umbral_sabiduria: f32,
    /// Intentos máximos de verificación
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
        info!("🔍 [DUDA SISTEMÁTICA] Mecanismo de verificación OMEGA activado");
        Self::default()
    }

    /// Evalúa una proposición y determina si requiere investigación externa
    pub async fn procesar_duda(&mut self, proposicion: &str) -> (bool, String) {
        let confianza = self.calcular_nivel_confianza(proposicion).await;

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

    /// Evaluación heurística de confianza
    async fn calcular_nivel_confianza(&self, proposicion: &str) -> f32 {
        let p = proposicion.to_lowercase();

        // Datos críticos (precios, mercado, paths) -> duda alta
        if p.contains("precio") || p.contains("market") || p.contains('/') || p.contains("error") {
            return 0.4;
        }

        // Afirmaciones genéricas largas -> confianza media
        if p.len() > 100 {
            return 0.7;
        }

        // Instrucciones directas del Arquitecto -> confianza alta
        0.9
    }

    /// Genera reporte de evidencia
    pub fn formatear_evidencia(&self, evidencias: Vec<String>) -> String {
        if evidencias.is_empty() {
            return "No se encontró evidencia física concluyente. Procediendo con precaución."
                .to_string();
        }

        let mut report =
            String::from("\n📑 [REPORTE DE EVIDENCIA OMEGA]:\n");
        for (i, e) in evidencias.iter().enumerate() {
            report.push_str(&format!("  {}. {}\n", i + 1, e));
        }
        report
    }
}

/// Juicio de Habla: Filtro de relevancia antes de interrumpir al Arquitecto
pub struct JuicioHabla;

impl JuicioHabla {
    pub fn new() -> Self {
        info!("🗣️ [JUICIO HABLA] Filtro de relevancia vocal activado");
        Self
    }

    /// Determina si un pensamiento merece interrumpir al Arquitecto
    pub fn deberia_hablar(&self, pensamiento: &Pensamiento) -> PrioridadHabla {
        // 1. Peligro inminente
        if pensamiento.es_peligro && pensamiento.gravedad > 0.8 {
            info!("🚨 [JUICIO-HABLA] Peligro Inminente (>0.8). ABRIENDO CANAL VOCAL.");
            return PrioridadHabla::Peligro;
        }

        // 2. Mejora significativa
        if pensamiento.es_mejora && pensamiento.impacto > 0.7 {
            info!("💡 [JUICIO-HABLA] Mejora de Alto Impacto (>0.7). PREPARANDO PROPUESTA.");
            return PrioridadHabla::Mejora;
        }

        // 3. Duda crítica
        if pensamiento.es_duda_importante {
            info!("❓ [JUICIO-HABLA] Duda Vital. CONSULTANDO AL ARQUITECTO.");
            return PrioridadHabla::Duda;
        }

        // 4. Lección valiosa
        if pensamiento.es_leccion && pensamiento.importancia > 0.8 {
            info!("📚 [JUICIO-HABLA] Lección Fundamental (>0.8). COMPARTIENDO SABIDURÍA.");
            return PrioridadHabla::Leccion;
        }

        // 5. Silencio soberano
        PrioridadHabla::Silencio
    }

    /// Expresa el pensamiento solo si pasa el filtro
    pub fn expresar_si_merece(&self, pensamiento: Pensamiento) -> Option<String> {
        match self.deberia_hablar(&pensamiento) {
            PrioridadHabla::Silencio => None,
            _ => Some(pensamiento.mensaje),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duda_confianza_alta() {
        let mut duda = DudaSistematica::new();
        let (necesita_verificacion, _) =
            tokio::runtime::Runtime::new().unwrap().block_on(duda.procesar_duda("Hola Arquitecto, todo está listo"));
        assert!(!necesita_verificacion);
    }

    #[test]
    fn test_duda_confianza_baja() {
        let mut duda = DudaSistematica::new();
        let (necesita_verificacion, _) =
            tokio::runtime::Runtime::new().unwrap().block_on(duda.procesar_duda("El precio del mercado es..."));
        assert!(necesita_verificacion);
    }

    #[test]
    fn test_juicio_habla_silencio() {
        let juicio = JuicioHabla::new();
        let p = Pensamiento {
            mensaje: "Todo está bien.".to_string(),
            es_peligro: false,
            es_mejora: false,
            es_duda_importante: false,
            es_leccion: false,
            gravedad: 0.1,
            impacto: 0.1,
            importancia: 0.1,
        };
        assert_eq!(juicio.deberia_hablar(&p), PrioridadHabla::Silencio);
    }

    #[test]
    fn test_juicio_habla_peligro() {
        let juicio = JuicioHabla::new();
        let p = Pensamiento::nuevo_peligro("CPU al 99%", 0.9);
        assert_eq!(juicio.deberia_hablar(&p), PrioridadHabla::Peligro);
    }
}
