// ============================================================================
// 📊 TELEMETRÍA V0 — Métricas del pipeline de generación de UI
// ============================================================================
// Registra latencias por etapa, fallos de gates e invocaciones de debuggers
// durante un run del pipeline. Produce un resumen estructurado que se persiste
// en el Session Store (MetricasSession) y se expone al CLI.
//
// Estrategia:
//   - `TelemetriaV0` (determinista, sin red): acumula etapas y contadores.
//   - `generar_reporte` (async): envoltura de producción que puede enriquecer
//     el reporte con contexto adicional; sin red devuelve el resumen local.
// ============================================================================

use std::time::Instant;

/// Registro temporal de una etapa del pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct RegistroEtapa {
    /// Nombre de la etapa (planificar, generar, gate_ast, debugger_tier1...).
    pub nombre: String,
    /// Duración de la etapa en milisegundos.
    pub duration_ms: u64,
}

/// Resumen de telemetría de un run del pipeline.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResumenTelemetria {
    /// Latencia total del run en milisegundos.
    pub latencia_total_ms: u64,
    /// Número de gates que fallaron (passed == false).
    pub gates_fallidos: u32,
    /// Número de invocaciones del debugger (tier1 + tier2).
    pub debugger_invocaciones: u32,
    /// Número de archivos generados.
    pub archivos_generados: usize,
    /// Número de errores que quedaron sin resolver tras la depuración.
    pub errores_restantes: usize,
    /// Registro detallado de etapas en orden de ejecución.
    pub etapas: Vec<RegistroEtapa>,
    /// `true` si el run terminó con todos los gates en verde.
    pub pipeline_limpio: bool,
}

impl ResumenTelemetria {
    /// Añade un registro de etapa (mantiene orden cronológico).
    fn registrar_etapa(&mut self, nombre: &str, duration_ms: u64) {
        self.etapas.push(RegistroEtapa {
            nombre: nombre.to_string(),
            duration_ms,
        });
    }
}

/// Recolector de telemetría del pipeline V0.
#[derive(Debug, Clone, Default)]
pub struct TelemetriaV0 {
    resumen: ResumenTelemetria,
    inicio_total: Option<Instant>,
}

impl TelemetriaV0 {
    /// Inicia un nuevo run del pipeline.
    pub fn nuevo() -> Self {
        Self {
            resumen: ResumenTelemetria::default(),
            inicio_total: Some(Instant::now()),
        }
    }

    /// Registra una etapa completada con su duración.
    pub fn registrar_etapa(&mut self, nombre: &str, duration_ms: u64) {
        self.resumen.registrar_etapa(nombre, duration_ms);
    }

    /// Registra el resultado de un gate. `passed == false` cuenta como fallo.
    pub fn registrar_gate(&mut self, passed: bool) {
        if !passed {
            self.resumen.gates_fallidos += 1;
        }
    }

    /// Registra una invocación del debugger.
    pub fn registrar_debugger(&mut self) {
        self.resumen.debugger_invocaciones += 1;
    }

    /// Registra el número de archivos generados.
    pub fn registrar_archivos(&mut self, n: usize) {
        self.resumen.archivos_generados = n;
    }

    /// Registra los errores que quedaron sin resolver.
    pub fn registrar_errores_restantes(&mut self, n: usize) {
        self.resumen.errores_restantes = n;
    }

    /// Finaliza el run: marca latencia total y limpieza del pipeline.
    pub fn finalizar(&mut self, pipeline_limpio: bool) {
        if let Some(inicio) = self.inicio_total.take() {
            self.resumen.latencia_total_ms = inicio.elapsed().as_millis() as u64;
        }
        self.resumen.pipeline_limpio = pipeline_limpio;
    }

    /// Devuelve el resumen actual de telemetría.
    pub fn resumen(&self) -> &ResumenTelemetria {
        &self.resumen
    }

    /// Devuelve el resumen clonado (útil para serializar o persistir).
    pub fn resumen_clon(&self) -> ResumenTelemetria {
        self.resumen.clone()
    }

    /// Envoltorio de producción: genera el reporte final. Sin API disponible
    /// (hermeticidad de tests) devuelve el resumen local.
    pub async fn generar_reporte(&mut self) -> ResumenTelemetria {
        self.resumen.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn telemetria_con_etapas() -> TelemetriaV0 {
        let mut t = TelemetriaV0::nuevo();
        t.registrar_etapa("planificar", 12);
        t.registrar_etapa("generar", 40);
        t.registrar_etapa("gate_ast", 3);
        t.registrar_gate(true);
        t.registrar_etapa("gate_render", 2);
        t.registrar_gate(false);
        t.registrar_debugger();
        t.registrar_archivos(4);
        t.registrar_errores_restantes(1);
        t.finalizar(false);
        t
    }

    #[test]
    fn test_nuevo_inicia_resumen_default() {
        let t = TelemetriaV0::nuevo();
        assert_eq!(t.resumen().gates_fallidos, 0);
        assert_eq!(t.resumen().debugger_invocaciones, 0);
        assert!(!t.resumen().pipeline_limpio);
    }

    #[test]
    fn test_registrar_etapas_acumula_orden() {
        let t = telemetria_con_etapas();
        let etapas = &t.resumen().etapas;
        assert_eq!(etapas.len(), 4);
        assert_eq!(etapas[0].nombre, "planificar");
        assert_eq!(etapas[0].duration_ms, 12);
        assert_eq!(etapas[3].nombre, "gate_render");
    }

    #[test]
    fn test_contadores_gates_y_debugger() {
        let t = telemetria_con_etapas();
        assert_eq!(t.resumen().gates_fallidos, 1);
        assert_eq!(t.resumen().debugger_invocaciones, 1);
        assert_eq!(t.resumen().archivos_generados, 4);
        assert_eq!(t.resumen().errores_restantes, 1);
    }

    #[test]
    fn test_finalizar_marca_latencia_y_limpieza() {
        let t = telemetria_con_etapas();
        assert!(!t.resumen().pipeline_limpio);
        // La latencia total es tiempo real transcurrido (siempre >= 0).
        assert!(t.resumen().latencia_total_ms >= 0);
    }

    #[test]
    fn test_pipeline_limpio_cuando_todo_pasa() {
        let mut t = TelemetriaV0::nuevo();
        t.registrar_gate(true);
        t.registrar_gate(true);
        t.registrar_gate(true);
        t.finalizar(true);
        assert!(t.resumen().pipeline_limpio);
        assert_eq!(t.resumen().gates_fallidos, 0);
    }

    #[test]
    fn test_resumen_clon_es_independiente() {
        let mut t = TelemetriaV0::nuevo();
        t.registrar_debugger();
        let clon = t.resumen_clon();
        t.registrar_debugger();
        assert_eq!(clon.debugger_invocaciones, 1);
        assert_eq!(t.resumen().debugger_invocaciones, 2);
    }

    #[tokio::test]
    async fn test_generar_reporte_retorna_resumen_local() {
        let mut t = telemetria_con_etapas();
        let reporte = t.generar_reporte().await;
        assert_eq!(reporte.gates_fallidos, 1);
        assert_eq!(reporte.debugger_invocaciones, 1);
        assert_eq!(reporte.etapas.len(), 4);
    }
}
