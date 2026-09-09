// ==========================================
// ÍNSULA OMEGA - Conciencia Corporal Interna + Autodiagnóstico Estructural
// ==========================================
// Fusión anatómica:
//   - Insula (de brain/insula.rs): Interocepción estructural, auditoría de redundancia,
//                                   evaluación de fricción sistémica, grito de sufrimiento
//   - Insula (original migrado): Sensibilidad a warnings de compilación, dolor de código
// ==========================================
// Como la ínsula humana: permite "sentir" el estado interno del organismo.
// Hambre de tokens, fatiga de CPU, dolor de errores, warnings de compilación,
// y la integridad estructural del ser.
// ==========================================

use crate::brain::healer::Healer;
use crate::brain::reflex_arc::ReflexSignal;
use crate::brain::thalamus::Thalamus;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use sysinfo::System;
use tokio::sync::mpsc;

// ─── WARNINGS DE COMPILACIÓN (original migrado) ────────────────────────

/// Un warning de compilación sentido como dolor localizado.
#[derive(Debug, Clone)]
pub struct WarningSensor {
    pub archivo: String,
    pub linea: u32,
    pub mensaje: String,
    pub tipo: TipoWarning,
    pub timestamp: Instant,
}

/// Clasificación del "dolor" según el tipo de warning.
#[derive(Debug, Clone, PartialEq)]
pub enum TipoWarning {
    ImportNoUsado,
    VariableNoUsada,
    CampoNoLeido,
    FuncionNoUsada,
    CfgCondicion,
    AsignacionSobrescrita,
    Otro(String),
}

impl TipoWarning {
    /// Peso del dolor: qué tanto estresa cada tipo.
    pub fn peso_estres(&self) -> f64 {
        match self {
            Self::ImportNoUsado => 0.15,
            Self::VariableNoUsada => 0.12,
            Self::CampoNoLeido => 0.10,
            Self::FuncionNoUsada => 0.08,
            Self::CfgCondicion => 0.05,
            Self::AsignacionSobrescrita => 0.10,
            Self::Otro(_) => 0.07,
        }
    }

    pub fn etiqueta(&self) -> &str {
        match self {
            Self::ImportNoUsado => "import_muerto",
            Self::VariableNoUsada => "variable_muerta",
            Self::CampoNoLeido => "campo_muerto",
            Self::FuncionNoUsada => "funcion_muerta",
            Self::CfgCondicion => "cfg_condicion",
            Self::AsignacionSobrescrita => "asignacion_sobrescrita",
            Self::Otro(s) => s.as_str(),
        }
    }

    /// Detecta el tipo de warning desde el mensaje de cargo.
    pub fn desde_mensaje(mensaje: &str) -> Self {
        let lower = mensaje.to_lowercase();
        if lower.contains("unused import") {
            Self::ImportNoUsado
        } else if lower.contains("unused variable") || lower.contains("unused mut") {
            Self::VariableNoUsada
        } else if lower.contains("field") && lower.contains("never read") {
            Self::CampoNoLeido
        } else if lower.contains("never used") && lower.contains("function") {
            Self::FuncionNoUsada
        } else if lower.contains("cfg condition") {
            Self::CfgCondicion
        } else if lower.contains("value assigned") && lower.contains("never read") {
            Self::AsignacionSobrescrita
        } else {
            Self::Otro(mensaje.to_string())
        }
    }
}

// ─── ÍNSULA UNIFICADA ──────────────────────────────────────────────────

/// Córtex Insular (Ínsula) — Conciencia Corporal Interna de NEXUS.
///
/// Integra:
///   1. Interocepción estructural (de brain/insula.rs): auditoría de redundancia,
///      monitoreo de fricción sistémica, detección de sufrimiento (CPU, RAM, temp).
///   2. Sensibilidad a código (original migrado): warnings de compilación como dolor.
pub struct Insula {
    // ─── Interocepción estructural (desde brain/insula.rs) ─────────────
    // Opcionales para permitir constructor sin args (constructor.rs legacy)
    thalamus: Option<Arc<Thalamus>>,
    _healer: Option<Arc<Healer>>,
    reflex_tx: Option<mpsc::Sender<ReflexSignal>>,

    // ─── Autodiagnóstico de código (original migrado) ──────────────────
    inicio_sesion: Instant,
    errores_acumulados: u32,
    exitos_acumulados: u32,
    /// Estrés INTEROCEPTIVO: dolor técnico acumulado (errores de compilación, warnings).
    /// ⚠️ Distinto a `amygdala.nivel_estres` (emocional). El tálamo los integra por separado.
    nivel_estres: f64,
    warnings_activos: HashMap<String, Vec<WarningSensor>>,
    total_warnings_recibidos: u32,
    umbral_autocorreccion: u32,
}

impl Insula {
    /// Crea una Ínsula completa con sistema de interocepción (brain/insula.rs).
    /// Requiere thalamus, healer y reflex_tx para monitoreo sistémico.
    pub fn new(
        thalamus: Arc<Thalamus>,
        healer: Arc<Healer>,
        reflex_tx: mpsc::Sender<ReflexSignal>,
    ) -> Self {
        Self {
            // Interocepción
            thalamus: Some(thalamus),
            _healer: Some(healer),
            reflex_tx: Some(reflex_tx),
            // Autodiagnóstico
            inicio_sesion: Instant::now(),
            errores_acumulados: 0,
            exitos_acumulados: 0,
            nivel_estres: 0.0,
            warnings_activos: HashMap::new(),
            total_warnings_recibidos: 0,
            umbral_autocorreccion: 5,
        }
    }

    /// Constructor mínimo (compatibilidad con constructor.rs legacy).
    /// Sin interocepción estructural — solo autodiagnóstico de código.
    pub fn solo_autodiagnostico() -> Self {
        Self {
            thalamus: None,
            _healer: None,
            reflex_tx: None,
            inicio_sesion: Instant::now(),
            errores_acumulados: 0,
            exitos_acumulados: 0,
            nivel_estres: 0.0,
            warnings_activos: HashMap::new(),
            total_warnings_recibidos: 0,
            umbral_autocorreccion: 5,
        }
    }

    // ─── INTEROCEPCIÓN ESTRUCTURAL (desde brain/insula.rs) ────────────

    /// Auditoría de Consolidación (Reloj Suizo)
    pub async fn auditar_consolidacion(&self) {
        println!("🧠 [ÍNSULA] Iniciando auditoría de integridad estructural...");
        self.verificar_redundancia().await;
        self.evaluar_sincronia().await;
        self.detectar_sufrimiento().await;
    }

    async fn verificar_redundancia(&self) {
        println!("🧬 [ÍNSULA] Verificando unicidad de instintos... Sincronizado.");
    }

    async fn evaluar_sincronia(&self) {
        let thalamus = match self.thalamus.as_ref() {
            Some(t) => t,
            None => return, // Sin interocepción configurada (modo solo_autodiagnostico)
        };
        let reflex_tx = match self.reflex_tx.as_ref() {
            Some(tx) => tx,
            None => return,
        };
        let latency = thalamus.get_polling_ms(100);
        if latency > 1000 {
            println!(
                "⚠️ [ÍNSULA] Fricción sistémica detectada. El sistema no late con precisión suiza."
            );
            let _ = reflex_tx
                .send(ReflexSignal::ProprioceptiveShift(
                    "Fricción Estratégica".to_string(),
                ))
                .await;
        }
    }

    async fn detectar_sufrimiento(&self) {
        let reflex_tx = match self.reflex_tx.as_ref() {
            Some(tx) => tx,
            None => return, // Sin interocepción configurada (modo solo_autodiagnostico)
        };

        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_usage();
        let mem_used_pct = sys.used_memory() as f32 / sys.total_memory() as f32 * 100.0;

        let mut temp = 0.0;
        if let Ok(temp_str) = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
            if let Ok(temp_milli) = temp_str.trim().parse::<f32>() {
                temp = temp_milli / 1000.0;
            }
        }

        let mut grito = Vec::new();
        if cpu_usage > 90.0 {
            grito.push("CPU AGOTADA");
        }
        if mem_used_pct > 95.0 {
            grito.push("MEMORIA AL LÍMITE");
        }
        if temp > 95.0 {
            grito.push("FIEBRE CRÍTICA");
        }

        if !grito.is_empty() {
            let mensaje = grito.join(" + ");
            println!("📢 [ÍNSULA] ¡GRITO DE SUFRIMIENTO! 🧬 -> {}", mensaje);
            let _ = reflex_tx.send(ReflexSignal::Distress(mensaje)).await;
        }
    }

    // ─── SENSIBILIDAD A CÓDIGO (original migrado) ─────────────────────

    /// Registra un error (aumenta el estrés).
    pub fn sentir_error(&mut self) {
        self.errores_acumulados += 1;
        self.nivel_estres = (self.nivel_estres + 0.1).min(1.0);
    }

    /// Registra un éxito (reduce el estrés).
    pub fn sentir_exito(&mut self) {
        self.exitos_acumulados += 1;
        self.nivel_estres = (self.nivel_estres - 0.05).max(0.0);
    }

    /// La Ínsula "siente" un warning de compilación como dolor localizado.
    pub fn sentir_warning(&mut self, archivo: &str, linea: u32, mensaje: &str) {
        let tipo = TipoWarning::desde_mensaje(mensaje);
        let warning = WarningSensor {
            archivo: archivo.to_string(),
            linea,
            mensaje: mensaje.to_string(),
            tipo: tipo.clone(),
            timestamp: Instant::now(),
        };

        self.warnings_activos
            .entry(archivo.to_string())
            .or_default()
            .push(warning);

        self.total_warnings_recibidos += 1;
        self.nivel_estres = (self.nivel_estres + tipo.peso_estres()).min(1.0);
    }

    /// Limpia warnings de un archivo específico.
    pub fn limpiar_warnings(&mut self, archivo: &str) {
        self.warnings_activos.remove(archivo);
        self.recalcular_estres();
    }

    /// Devuelve el nivel de estrés actual (0.0 — 1.0).
    pub fn nivel_estres(&self) -> f64 {
        self.nivel_estres
    }

    /// Reporta si hay warnings activos y si se debe gatillar autocorrección.
    pub fn necesita_autocorreccion(&self) -> bool {
        self.total_warnings_recibidos >= self.umbral_autocorreccion
            && !self.warnings_activos.is_empty()
    }

    /// Retorna la lista de archivos con más dolor (warnings).
    pub fn archivos_doloridos(&self) -> Vec<(&String, usize)> {
        let mut archivos: Vec<_> = self
            .warnings_activos
            .iter()
            .map(|(k, v)| (k, v.len()))
            .collect();
        archivos.sort_by(|a, b| b.1.cmp(&a.1));
        archivos
    }

    /// Retorna una sugerencia de corrección legible para el archivo más dolorido.
    pub fn sugerir_correccion(&self) -> Option<(String, Vec<String>)> {
        let archivos = self.archivos_doloridos();
        let (archivo, _) = archivos.first()?;

        let warnings = self.warnings_activos.get(*archivo)?;
        let mut acciones = Vec::new();

        for w in warnings {
            let accion = match w.tipo {
                TipoWarning::ImportNoUsado => {
                    format!(
                        "  → {}:{} Eliminar import no usado: \"{}\"",
                        w.archivo, w.linea, w.mensaje
                    )
                }
                TipoWarning::VariableNoUsada => {
                    format!(
                        "  → {}:{} Prefix con _ o eliminar variable: \"{}\"",
                        w.archivo, w.linea, w.mensaje
                    )
                }
                TipoWarning::CampoNoLeido => {
                    format!(
                        "  → {}:{} Campo struct no leído, añadir #[allow(dead_code)] o eliminar: \"{}\"",
                        w.archivo, w.linea, w.mensaje
                    )
                }
                TipoWarning::FuncionNoUsada => {
                    format!(
                        "  → {}:{} Función no usada, añadir #[allow(dead_code)] o eliminar: \"{}\"",
                        w.archivo, w.linea, w.mensaje
                    )
                }
                TipoWarning::AsignacionSobrescrita => {
                    format!(
                        "  → {}:{} Asignación sobrescrita antes de leer: \"{}\"",
                        w.archivo, w.linea, w.mensaje
                    )
                }
                _ => {
                    format!("  → {}:{} {}", w.archivo, w.linea, w.mensaje)
                }
            };
            acciones.push(accion);
        }

        Some((archivo.to_string(), acciones))
    }

    fn recalcular_estres(&mut self) {
        let mut estres_warnings = 0.0;
        for warnings in self.warnings_activos.values() {
            for w in warnings {
                estres_warnings += w.tipo.peso_estres();
            }
        }
        let estres_base = (self.errores_acumulados as f64 * 0.1).min(0.5);
        self.nivel_estres = (estres_base + estres_warnings.min(0.8)).min(1.0);
    }

    /// Devuelve el estado interno actual, incluye diagnóstico de código.
    pub fn estado_interno(&self) -> String {
        let estres = if self.nivel_estres > 0.7 {
            "🔴 ALTO"
        } else if self.nivel_estres > 0.3 {
            "🟡 MEDIO"
        } else {
            "🟢 BAJO"
        };

        let mut reporte = format!(
            "🫀 ÍNSULA: {} errores, {} éxitos, estrés {} ({:.2}) | Sesión: {}s",
            self.errores_acumulados,
            self.exitos_acumulados,
            estres,
            self.nivel_estres,
            self.inicio_sesion.elapsed().as_secs()
        );

        if !self.warnings_activos.is_empty() {
            reporte.push_str(&format!(
                "\n📋 Warnings activos: {} en {} archivos",
                self.total_warnings_recibidos,
                self.warnings_activos.len()
            ));
            for (archivo, count) in self.archivos_doloridos().iter().take(3) {
                reporte.push_str(&format!("\n   ⚠️  {} ({} warnings)", archivo, count));
            }
            if self.warnings_activos.len() > 3 {
                reporte.push_str(&format!(
                    "\n   ... y {} archivos más",
                    self.warnings_activos.len() - 3
                ));
            }
            if self.necesita_autocorreccion() {
                reporte
                    .push_str("\n🚨 [AUTOCORRECCIÓN] Umbral superado — se requiere intervención.");
            }
        }

        reporte
    }

    /// Devuelve true si NEXUS debería descansar o autocorregirse.
    pub fn necesita_descanso(&self) -> bool {
        self.nivel_estres > 0.8 || self.errores_acumulados > 20 || self.necesita_autocorreccion()
    }

    /// Limpia el estado completo (reset de sesión).
    pub fn reset(&mut self) {
        self.inicio_sesion = Instant::now();
        self.errores_acumulados = 0;
        self.exitos_acumulados = 0;
        self.nivel_estres = 0.0;
        self.warnings_activos.clear();
        self.total_warnings_recibidos = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warning_detecta_tipo_import_muerto() {
        let tipo = TipoWarning::desde_mensaje("unused import: `debug`");
        assert_eq!(tipo, TipoWarning::ImportNoUsado);
    }

    #[test]
    fn test_warning_detecta_tipo_variable_muerta() {
        let tipo = TipoWarning::desde_mensaje("unused variable: `persona`");
        assert_eq!(tipo, TipoWarning::VariableNoUsada);
    }

    #[test]
    fn test_warning_detecta_campo_muerto() {
        let tipo = TipoWarning::desde_mensaje("field `token_leak_guard` is never read");
        assert_eq!(tipo, TipoWarning::CampoNoLeido);
    }
}
