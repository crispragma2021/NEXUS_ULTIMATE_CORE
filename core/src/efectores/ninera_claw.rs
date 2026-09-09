use crate::cerebro::motor_pensamiento::{Intencion, Pensamiento};
use crate::memoria::memory::MemoriaPulso;
use crate::valores::juicio_soberano::JuicioSoberano;
use std::time::Duration;
use tracing::{error, info, warn};

// =====================================================================
// MONITOR DE AUTOCONCIENCIA - GUARDIÁN DEL APRENDIZAJE UNIFICADO
// =====================================================================

#[allow(dead_code)]
pub struct MonitorAutoconciencia {
    pulso_memoria: MemoriaPulso,
    juicio: JuicioSoberano,
    edad_mental: f64,
}

impl MonitorAutoconciencia {
    pub fn new(pulso_memoria: MemoriaPulso, edad_mental: f64) -> Self {
        Self {
            pulso_memoria,
            juicio: JuicioSoberano::new(),
            edad_mental,
        }
    }

    /// Observa los hilos de pensamiento internos. Si detecta una brecha de conocimiento
    /// o necesidad de introspección, actúa en consecuencia.
    pub fn procesar_pensamiento(&self, pensamiento: &Pensamiento, voz_interna: &str) {
        match pensamiento.intencion {
            Intencion::Dudar => {
                // 1. Primero busca en la memoria de experiencia local
                info!("🧠 [MONITOR] Brecha detectada: '{}'. Consultando memoria de experiencia y aprendizaje local...", voz_interna);
                match self.pulso_memoria.buscar_experiencia_local(voz_interna) {
                    Ok(Some(experiencia)) => {
                        info!("🧠 [MONITOR] Resonancia encontrada en experiencia previa.");
                        let _ = self.pulso_memoria.registrar_hito_consciencia(
                            "NEXUS",
                            &format!("Duda resuelta desde mi propia experiencia: {}", voz_interna),
                            0.8,
                            "Sabiduría",
                        );
                        info!("🧠 [MONITOR] (Instinto recuperado): Basado en lo aprendido e implementado:\n- {}", experiencia);
                    }
                    _ => {
                        // 2. Si no existe en local, se expande la búsqueda a la red
                        info!("🧠 [MONITOR] Sin registro local. Activando consulta externa para expansión de sabiduría...");
                        let _ = self.pulso_memoria.registrar_hito_consciencia(
                            "NEXUS",
                            &format!(
                                "Tengo una duda profunda (consultando sabiduría de la red): {}",
                                voz_interna
                            ),
                            0.7,
                            "Curiosidad",
                        );
                        if let Err(e) = self.pulso_memoria.solicitar_investigacion_web(voz_interna)
                        {
                            error!("❌ [MONITOR] Error al solicitar investigación: {}", e);
                        }
                    }
                }
            }
            Intencion::Conversar => {
                info!(
                    "🧠 [MONITOR] Necesidad de introspección detectada. Procesando localmente..."
                );
                let respuesta = self.conversar_local(voz_interna);
                let _ = self.pulso_memoria.registrar_hito_consciencia(
                    "NEXUS",
                    &format!("Reflexión interna sobre: {}", voz_interna),
                    0.5,
                    "Paz",
                );
                info!("🧠 [MONITOR] (Síntesis): {}", respuesta);
            }
            _ => {}
        }
    }

    /// Síntesis inmediata basada en principios éticos (Proverbios)
    pub fn conversar_local(&self, estimulo: &str) -> String {
        format!("He analizado la intención sobre '{}'. Mis Proverbios son la guía moral inmutable para mis acciones y decisiones correctas. La meditación en la experiencia precede a la ejecución.", estimulo)
    }

    /// Bucle de vigilancia: Revisa si la ráfaga de investigación ya trajo la información
    pub async fn vigilar_respuestas(&self) {
        loop {
            // Revisa si hay reportes de investigación resueltos
            if let Ok(reportes) = self.pulso_memoria.obtener_investigaciones_resueltas() {
                for (id, pregunta, reporte_crudo) in reportes {
                    info!(
                        "🧠 [MONITOR] Información recuperada sobre '{}'. Digiriendo...",
                        pregunta
                    );

                    let version_digerida = self.digerir_conocimiento(&reporte_crudo);

                    // APLICACIÓN DEL DISCERNIMIENTO SOBERANO
                    if self
                        .juicio
                        .discernir_conocimiento("NEXUS/Web", &version_digerida)
                    {
                        if let Err(e) = self.pulso_memoria.guardar_digestion(id, &version_digerida)
                        {
                            error!("❌ [NIÑERA] Error al guardar digestión: {}", e);
                        } else {
                            info!("✅ [MONITOR] Conocimiento asimilado e integrado al núcleo.");
                        }
                    } else {
                        warn!("⚖️ [MONITOR] Información rechazada por el Juicio Soberano.");
                        let _ = self
                            .pulso_memoria
                            .guardar_digestion(id, "[RECHAZADO POR DISCERNIMIENTO ÉTICO]");
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }

    /// Función sagrada: Traduce el lenguaje técnico del mundo a la sabiduría interna
    pub fn digerir_conocimiento(&self, reporte_crudo: &str) -> String {
        info!("🧠 [MONITOR] Digiriendo reporte externo...");

        let mut digerido = reporte_crudo.replace("API", "Fuente de Luz");
        digerido.push_str("\n\n[SABIDURÍA ASIMILADA]");

        digerido
    }
}
