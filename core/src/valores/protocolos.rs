// ==========================================
// 🦾 PROTOCOLOS DE EJECUCIÓN — Reglas Físicas de NEXUS
// ==========================================
// Absorbe .agent/rules/protocolos_ejecucion.md como constantes nativas.
// Define niveles de seguridad, identificación de herramientas y
// umbrales de homeostasis para la ejecución física.
// ==========================================

use serde::Serialize;

/// Niveles de seguridad para ejecución de operaciones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum NivelSeguridad {
    /// Ejecución nativa directa, sin sandbox. Para ráfagas de alta prioridad.
    NativaDirecta,
    /// Ejecución con sandbox bwrap (por defecto).
    Sandboxeada,
    /// Solo lectura, sin escritura al sistema.
    SoloLectura,
    /// Simulación seca, sin efectos secundarios.
    DryRun,
}

impl NivelSeguridad {
    pub fn nombre(&self) -> &'static str {
        match self {
            Self::NativaDirecta => "Ejecución Nativa Directa",
            Self::Sandboxeada => "Sandbox bwrap",
            Self::SoloLectura => "Solo Lectura",
            Self::DryRun => "Simulación Seca (Dry Run)",
        }
    }
}

/// Identificación de herramientas de ejecución.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum HerramientaEjecucion {
    /// Agente Nativo: comandos de terminal, gestión de hilos y cgroups.
    AgenteNativo,
    /// NexusClaw Pro: manipulación de archivos (WRITE, READ) y navegación webclaw.
    NexusClawPro,
    /// Browser Native: scraping web con chromiumoxide.
    BrowserNative,
    /// Script Runner: ejecución de scripts shell.
    ScriptRunner,
}

impl HerramientaEjecucion {
    pub fn nombre(&self) -> &'static str {
        match self {
            Self::AgenteNativo => "AgenteNativo",
            Self::NexusClawPro => "NexusClaw Pro",
            Self::BrowserNative => "Browser Native",
            Self::ScriptRunner => "Script Runner",
        }
    }
}

// ── Constantes de Protocolo ──────────────────────────────────────────

/// Núcleos P (Performance) del i7-12700F para ráfagas de latencia ultra-baja.
pub const NUCLEOS_P: std::ops::Range<usize> = 0..8;

/// Umbral térmico para suspender ráfagas no críticas.
pub const UMBRAL_TERMICO_CRITICO: f64 = 80.0;

/// Nombre del ledger de registro atómico.
pub const LEDGER_DB: &str = "nexus_ledger.db";

/// Formato de log para registro atómico.
pub const FORMATO_LOG: &str = "[Timestamp] [Agente] [Acción] [Resultado]";

/// Nivel de seguridad por defecto.
pub const SEGURIDAD_POR_DEFECTO: NivelSeguridad = NivelSeguridad::Sandboxeada;

/// Indica si está permitido desactivar bwrap por orden del Arquitecto.
pub const PERMITIR_NATIVA_DIRECTA: bool = true;

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nucleos_p_rango_valido() {
        assert_eq!(NUCLEOS_P.start, 0);
        assert_eq!(NUCLEOS_P.end, 8);
    }

    #[test]
    fn test_umbral_termico_es_realista() {
        assert!(UMBRAL_TERMICO_CRITICO > 60.0 && UMBRAL_TERMICO_CRITICO < 100.0);
    }

    #[test]
    fn test_nivel_seguridad_tiene_nombre() {
        for nivel in [
            NivelSeguridad::NativaDirecta,
            NivelSeguridad::Sandboxeada,
            NivelSeguridad::SoloLectura,
            NivelSeguridad::DryRun,
        ] {
            assert!(!nivel.nombre().is_empty());
        }
    }

    #[test]
    fn test_herramienta_ejecucion_tiene_nombre() {
        for h in [
            HerramientaEjecucion::AgenteNativo,
            HerramientaEjecucion::NexusClawPro,
            HerramientaEjecucion::BrowserNative,
            HerramientaEjecucion::ScriptRunner,
        ] {
            assert!(!h.nombre().is_empty());
        }
    }

    #[test]
    fn test_formato_log_no_vacio() {
        assert!(!FORMATO_LOG.is_empty());
    }
}
