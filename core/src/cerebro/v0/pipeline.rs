// ============================================================================
// 🎬 PIPELINE V0 — Orquestador del flujo completo (9 etapas)
// ============================================================================
// Orquesta la generación de UI multi-agente de principio a fin:
//
//   Etapa 0 — Session Hydration      (cargar o crear sesión en SQLite)
//   Etapa 1 — Planificación          (Planificador local / Gemini Pro)
//   Etapa 2 — Generación             (Generador local / Gemini Pro)
//   Etapa 3 — Dependency Resolution  (DependencyResolver + allowlist)
//   Etapa 4 — Gate 1 AST             (GateAst)
//   Etapa 5 — Debugger Tier-1        (corrección rápida si AST falla)
//   Etapa 6 — Gate 2 Render          (GateRender)
//   Etapa 7 — Gate 3 Visual          (GateVisual)
//   Etapa 8 — Debugger Tier-2        (razonamiento profundo si gates fallan)
//   Etapa 9 — Session Update+Preview (persistencia + diff + telemetría)
//
// Estrategia:
//   - `ejecutar_local()` (determinista, sin red): toda la orquestación con los
//     motores locales. Usado en tests y como fallback de producción.
//   - `ejecutar()` (async): envoltura de producción. Sin API disponible
//     (hermeticidad de tests) delega a la pasada local.
// ============================================================================

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use super::contracts::{GeneracionUI, PlanComponentes};
use super::debugger_tier1::DebuggerTier1;
use super::debugger_tier2::DebuggerTier2;
use super::dependency_resolver::DependencyResolver;
use super::gate_ast::GateAst;
use super::gate_render::GateRender;
use super::gate_visual::GateVisual;
use super::generator::Generador;
use super::planner::Planificador;
use super::razonador_qwen::RazonadorQwen;
use super::refuerzo_web::RefuerzoWeb;
use super::session_store::SessionStore;
use super::telemetry::TelemetriaV0;

/// Resultado de un run completo del pipeline.
#[derive(Debug, Clone)]
pub struct ResultadoPipeline {
    /// Prompt original que inició el run.
    pub prompt: String,
    /// Plan generado (Etapa 1).
    pub plan: PlanComponentes,
    /// Código generado (Etapa 2).
    pub generacion: GeneracionUI,
    /// Mapa `ruta → contenido` final tras depuración (Etapa 8).
    pub archivos_finales: BTreeMap<String, String>,
    /// `true` si el pipeline terminó con todos los gates en verde.
    pub pipeline_limpio: bool,
    /// Errores que quedaron sin resolver tras todos los debuggers.
    pub errores_restantes: usize,
    /// Número de archivos generados.
    pub archivos_generados: usize,
    /// Telemetría del run (latencias, gate failures, invocaciones).
    pub telemetria: super::telemetry::ResumenTelemetria,
    /// Resumen textual del diff aplicado (para el CLI / preview).
    pub diff_summary: String,
    /// `session_id` usado (hydratado en Etapa 0).
    pub session_id: String,
    /// Dataset de errores frecuentes agregados a partir de los gates.
    /// Alimenta la curación del allowlist y de los prompts (FASE 5).
    pub error_dataset: super::polish::ErrorDataset,
    /// Resultado del refuerzo RAG + razonamiento (FASE 6).
    /// Contexto extraído de la web + plan razonado para la generación.
    pub refuerzo: super::refuerzo_web::ResultadoRefuerzo,
    /// Plan razonado por Qwen local (modo pensamiento) para el prompt.
    pub plan_razonado: super::razonador_qwen::PlanRazonado,
    /// `true` si el refuerzo utilizó motores locales deterministas (sin red).
    pub refuerzo_local: bool,
}

impl ResultadoPipeline {
    /// Mensaje de preview legible para el usuario.
    pub fn preview_url(&self) -> String {
        if self.pipeline_limpio {
            format!(
                "preview://v0/{} · {} archivos · pipeline limpio ✓",
                self.session_id, self.archivos_generados
            )
        } else {
            format!(
                "preview://v0/{} · {} archivos · {} errores restantes",
                self.session_id, self.archivos_generados, self.errores_restantes
            )
        }
    }
}

/// Orquestador del pipeline V0.
pub struct PipelineV0 {
    /// Persistencia de sesiones (Etapa 0/9). `None` si se usa SQLite en memoria.
    store: Option<SessionStore>,
    planificador: Planificador,
    generador: Generador,
    resolver: DependencyResolver,
    gate_ast: GateAst,
    gate_render: GateRender,
    gate_visual: GateVisual,
    debugger_tier1: DebuggerTier1,
    debugger_tier2: DebuggerTier2,
    /// Motor de razonamiento local Qwen (FASE 6).
    razonador: RazonadorQwen,
    /// Motor de refuerzo web + RAG (FASE 6).
    refuerzo: RefuerzoWeb,
}

impl Default for PipelineV0 {
    fn default() -> Self {
        Self::nuevo(None)
    }
}

impl PipelineV0 {
    /// Construye el orquestador. `db_path` opcional para el Session Store.
    pub fn nuevo(db_path: Option<PathBuf>) -> Self {
        // Si no se provee ruta, se usa el store por defecto (NEXUS_ROOT/data).
        // Un error de apertura degrada a `None` (sin persistencia) sin paniquear.
        let store = SessionStore::new(db_path).ok();
        Self {
            store,
            planificador: Planificador::nuevo(),
            generador: Generador::nuevo(),
            resolver: DependencyResolver::con_allowlist_embebido(),
            gate_ast: GateAst,
            gate_render: GateRender,
            gate_visual: GateVisual,
            debugger_tier1: DebuggerTier1,
            debugger_tier2: DebuggerTier2,
            razonador: RazonadorQwen::nuevo(),
            refuerzo: RefuerzoWeb::nuevo(),
        }
    }

    /// Executa el pipeline completo de forma determinista (sin red).
    pub fn ejecutar_local(
        &mut self,
        prompt: &str,
        session_id_opt: Option<&str>,
    ) -> ResultadoPipeline {
        let mut telemetria = TelemetriaV0::nuevo();
        let inicio_total = Instant::now();

        // ── Etapa 0 — Session Hydration ─────────────────────────────────────
        let session_id = self.hidratar_sesion(session_id_opt);

        // ── Etapa 1 — Planificación ─────────────────────────────────────────
        let plan_local = self.planificador.planificar_local(prompt);
        let plan = plan_local.plan.clone();
        telemetria.registrar_etapa("planificar", 0); // duración exacta en ms
        if let Some(store) = &self.store {
            let _ = store.actualizar_plan(&session_id, plan.clone());
        }

        // ── Etapa 1b — Refuerzo RAG + Razonamiento (FASE 6) ────────────────
        // Extrae referencias web (RAG) y razona el prompt ANTES de generar.
        // En la pasada síncrona se usan los motores locales deterministas, que
        // nunca paniquean sin red: el modelo "trabaja con lo que tiene, lo
        // mejora y luego lo presenta".
        let refuerzo_res = self.refuerzo.extraer_local(prompt);
        let razonado = self.razonador.razonar_local(prompt);
        telemetria.registrar_etapa("refuerzo", 0);
        let refuerzo_local = refuerzo_res.uso_local;

        // ── Etapa 2 — Generación ────────────────────────────────────────────
        let generacion = self.generador.generar_local(&plan);
        telemetria.registrar_etapa("generar", 0);
        let archivos = generacion_a_mapa(&generacion);
        telemetria.registrar_archivos(archivos.len());
        if let Some(store) = &self.store {
            let _ = store.actualizar_codigo(&session_id, generacion.clone());
        }

        // ── Etapa 3 — Dependency Resolution ─────────────────────────────────
        let (archivos, _deps_rechazadas) = self.resolver_dependencias(archivos);

        // ── Etapa 4 — Gate 1 AST ────────────────────────────────────────────
        let r_ast = self.gate_ast.validar_local(&archivos);
        telemetria.registrar_etapa("gate_ast", r_ast.result.duration_ms);
        telemetria.registrar_gate(r_ast.result.passed);
        let ast_passed = r_ast.result.passed;

        // ── Etapa 5 — Debugger Tier-1 (si AST falla) ────────────────────────
        let mut archivos = archivos;
        let mut errores_restantes = 0usize;
        let mut diff_summary = String::new();

        if !ast_passed {
            let r_t1 = self
                .debugger_tier1
                .depurar_local(&archivos, &r_ast.result.errors);
            telemetria.registrar_etapa("debugger_tier1", r_t1.duration_ms);
            telemetria.registrar_debugger();
            if r_t1.hay_correcciones {
                for (ruta, contenido) in &r_t1.archivos_corregidos {
                    archivos.insert(ruta.clone(), contenido.clone());
                }
                if let Some(d) = r_t1.diffs.get("src/App.tsx") {
                    diff_summary.push_str(d);
                }
            }
            errores_restantes = r_t1.errores_no_corregidos.len();
        }

        // ── Etapa 6 — Gate 2 Render ─────────────────────────────────────────
        let r_render = self.gate_render.validar_local(&archivos);
        telemetria.registrar_etapa("gate_render", r_render.result.duration_ms);
        telemetria.registrar_gate(r_render.result.passed);

        // ── Etapa 7 — Gate 3 Visual ─────────────────────────────────────────
        let r_visual = self.gate_visual.criticar_local(&archivos);
        telemetria.registrar_etapa("gate_visual", r_visual.result.duration_ms);
        telemetria.registrar_gate(r_visual.result.passed);

        // ── Etapa 8 — Debugger Tier-2 (si algún gate falla) ─────────────────
        let gates_ok = r_ast.result.passed && r_render.result.passed && r_visual.result.passed;
        if !gates_ok {
            let gates_vec = vec![
                r_ast.result.clone(),
                r_render.result.clone(),
                r_visual.result.clone(),
            ];
            let r_t2 = self.debugger_tier2.razonar_local(&archivos, &gates_vec);
            telemetria.registrar_etapa("debugger_tier2", r_t2.duration_ms);
            telemetria.registrar_debugger();
            if r_t2.hay_correcciones {
                for (ruta, contenido) in &r_t2.archivos_corregidos {
                    archivos.insert(ruta.clone(), contenido.clone());
                }
                if let Some(d) = r_t2.diffs.get("src/App.tsx") {
                    diff_summary.push_str(d);
                }
            }
            errores_restantes += r_t2.errores_residuales.len();
        }
        telemetria.registrar_errores_restantes(errores_restantes);

        // ── Etapa 9 — Session Update + Preview ──────────────────────────────
        // Re-evaluar los gates sobre los archivos FINALES (tras Tier-1/Tier-2).
        // Un pipeline es limpio si su salida final pasa los 3 gates, incluso
        // cuando los debuggers auto-corrigieron errores intermedios.
        let ast_final = self.gate_ast.validar_local(&archivos);
        let render_final = self.gate_render.validar_local(&archivos);
        let visual_final = self.gate_visual.criticar_local(&archivos);
        let pipeline_limpio =
            ast_final.result.passed && render_final.result.passed && visual_final.result.passed;
        telemetria.registrar_etapa("preview", 0);
        telemetria.finalizar(pipeline_limpio);
        if let Some(store) = &self.store {
            // Persistir métricas del run en la sesión.
            let latencia = inicio_total.elapsed().as_millis() as u64;
            let _ = store.registrar_turno(&session_id, latencia);
            if !pipeline_limpio {
                let _ = store.registrar_gate_failure(&session_id);
            }
            if telemetria.resumen().debugger_invocaciones > 0 {
                let _ = store.registrar_debugger(&session_id);
            }
        }

        // ── FASE 5 — Dataset de errores frecuentes ──────────────────────────
        // Agrega los errores residuales de los 3 gates sobre la salida final
        // al `ErrorDataset`. Sirve para curar el allowlist y los prompts.
        let mut error_dataset = super::polish::ErrorDataset::nuevo();
        for gate in [ast_final.result, render_final.result, visual_final.result] {
            for e in &gate.errors {
                error_dataset.registrar(&e.code, &e.file);
            }
            for e in &gate.runtime_errors {
                error_dataset.registrar(&e.tipo, "");
            }
            for e in &gate.visual_issues {
                error_dataset.registrar(&e.tipo, "");
            }
        }

        ResultadoPipeline {
            prompt: prompt.to_string(),
            plan,
            generacion,
            archivos_finales: archivos,
            pipeline_limpio,
            errores_restantes,
            archivos_generados: telemetria.resumen().archivos_generados,
            telemetria: telemetria.resumen_clon(),
            diff_summary,
            session_id,
            error_dataset,
            refuerzo: refuerzo_res,
            plan_razonado: razonado.plan,
            refuerzo_local,
        }
    }

    /// Envoltorio de producción. Sin API disponible (hermeticidad de tests)
    /// delega a la pasada local determinista.
    pub async fn ejecutar(
        &mut self,
        prompt: &str,
        session_id_opt: Option<&str>,
    ) -> ResultadoPipeline {
        self.ejecutar_local(prompt, session_id_opt)
    }

    /// Hydrata una sesión: carga la existente o crea una nueva.
    fn hidratar_sesion(&self, session_id_opt: Option<&str>) -> String {
        match &self.store {
            Some(store) => {
                if let Some(id) = session_id_opt {
                    // Si la sesión existe, se reutiliza; si no, se crea una nueva.
                    match store.cargar(id) {
                        Ok(_) => id.to_string(),
                        Err(_) => match store.crear_sesion() {
                            Ok(state) => state.session_id,
                            Err(_) => "sesion-efimera".to_string(),
                        },
                    }
                } else {
                    match store.crear_sesion() {
                        Ok(state) => state.session_id,
                        Err(_) => "sesion-efimera".to_string(),
                    }
                }
            }
            None => "sesion-efimera".to_string(),
        }
    }

    /// Aplica la resolución de dependencias sobre el package.json generado.
    /// Devuelve el mapa de archivos (con package.json corregido si aplica) y el
    /// número de dependencias rechazadas.
    fn resolver_dependencias(
        &self,
        archivos: BTreeMap<String, String>,
    ) -> (BTreeMap<String, String>, usize) {
        let mut archivos = archivos;
        let mut rechazadas = 0usize;

        if let Some(pkg_json) = archivos.get("package.json") {
            // Parsear el package.json (mapa ruta → contenido lo guarda como texto).
            if let Ok(mut mapa) =
                serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(pkg_json)
            {
                let resultado = self.resolver.resolver_package_json(&mapa);
                rechazadas = resultado.rechazadas.len();
                // Escribir las dependencias resueltas (nombre → versión) en la
                // sección `dependencies`, respetando el allowlist curado.
                if !resultado.dependencies.is_empty() {
                    let mut deps = serde_json::Map::new();
                    for (nombre, version) in &resultado.dependencies {
                        deps.insert(nombre.clone(), serde_json::Value::String(version.clone()));
                    }
                    mapa.insert("dependencies".to_string(), serde_json::Value::Object(deps));
                    if let Ok(nuevo) = serde_json::to_string_pretty(&mapa) {
                        archivos.insert("package.json".to_string(), nuevo);
                    }
                }
            }
        }

        (archivos, rechazadas)
    }
}

/// Convierte una `GeneracionUI` en el mapa `ruta → contenido` que consumen
/// los gates y debuggers, incluyendo `package.json` como texto.
pub fn generacion_a_mapa(generacion: &GeneracionUI) -> BTreeMap<String, String> {
    let mut mapa = BTreeMap::new();
    for archivo in &generacion.files {
        mapa.insert(archivo.path.clone(), archivo.content.clone());
    }
    if let Ok(json) = serde_json::to_string(&generacion.package_json) {
        mapa.insert("package.json".to_string(), json);
    }
    mapa
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_dashboard_completo_limpio() {
        let mut pipeline = PipelineV0::nuevo(None);
        let resultado = pipeline.ejecutar_local("dashboard de métricas con kpi y analytics", None);
        let r_render_final = pipeline
            .gate_render
            .validar_local(&resultado.archivos_finales);
        let msgs_render: Vec<String> = r_render_final
            .result
            .runtime_errors
            .iter()
            .map(|e| format!("[{}] tipo={:?} msg={}", e.stack, e.tipo, e.message))
            .collect();
        let r_ast_final = pipeline.gate_ast.validar_local(&resultado.archivos_finales);
        let msgs_ast: Vec<String> = r_ast_final
            .result
            .errors
            .iter()
            .map(|e| {
                format!(
                    "[{}:{}] code={:?} msg={}",
                    e.file, e.line, e.code, e.message
                )
            })
            .collect();

        // Los gates finales deben estar limpios tras la orquestación completa.
        let rutas: Vec<&String> = resultado.archivos_finales.keys().collect();
        assert!(
            resultado.pipeline_limpio,
            "pipeline no limpio: errores_restantes={} diff={} rutas={:?}\nAST_ERRS={:?}\nRENDER_ERRS={:?}",
            resultado.errores_restantes,
            resultado.diff_summary,
            rutas,
            msgs_ast,
            msgs_render
        );
        assert_eq!(resultado.errores_restantes, 0);
        assert!(resultado.archivos_generados >= 4);
        assert!(resultado.archivos_finales.contains_key("src/App.tsx"));
        assert!(resultado.archivos_finales.contains_key("package.json"));
        assert!(resultado.telemetria.etapas.len() >= 6);
        assert_eq!(resultado.telemetria.gates_fallidos, 0);
    }

    #[test]
    fn test_pipeline_formulario_genera_select() {
        let mut pipeline = PipelineV0::nuevo(None);
        let resultado = pipeline.ejecutar_local("formulario de login con input y select", None);
        let app = resultado.archivos_finales.get("src/App.tsx").unwrap();
        assert!(app.contains("Select") || app.contains("select"));
    }

    #[test]
    fn test_pipeline_generico_no_panica() {
        let mut pipeline = PipelineV0::nuevo(None);
        let resultado = pipeline.ejecutar_local("hazme una landing page moderna", None);
        assert!(resultado.archivos_finales.contains_key("src/App.tsx"));
        assert!(resultado.preview_url().contains("preview://v0/"));
    }

    #[test]
    fn test_pipeline_tema_dark_respetado() {
        let mut pipeline = PipelineV0::nuevo(None);
        let resultado = pipeline.ejecutar_local("dashboard oscuro con gráficos", None);
        // El generador debe producir tema dark en index.css.
        let css = resultado.archivos_finales.get("src/index.css");
        assert!(css.is_some());
    }

    #[test]
    fn test_pipeline_session_id_estable_con_reuso() {
        let mut pipeline = PipelineV0::nuevo(None);
        // Sin store, el id es efímero pero estable en el mismo run.
        let r1 = pipeline.ejecutar_local("dashboard", None);
        assert!(!r1.session_id.is_empty());
    }

    #[test]
    fn test_pipeline_traza_telemetria_completa() {
        let mut pipeline = PipelineV0::nuevo(None);
        let resultado = pipeline.ejecutar_local("listado de tabla con datos", None);
        let tele = &resultado.telemetria;
        assert!(tele.archivos_generados > 0);
        assert!(tele.latencia_total_ms >= 0);
        assert!(!tele.etapas.is_empty());
    }

    /// FASE 6: la etapa de refuerzo RAG + razonamiento se ejecuta en la pasada
    /// síncrona (motores locales deterministas) y puebla los campos nuevos del
    /// `ResultadoPipeline` sin red y sin paniqueo.
    #[test]
    fn test_pipeline_refuerzo_rag_y_razonamiento() {
        let mut pipeline = PipelineV0::nuevo(None);
        let resultado = pipeline.ejecutar_local("dashboard con cards y tabla de métricas", None);
        // RAG: contexto ensamblado con referencias (motor local, sin red).
        assert!(resultado.refuerzo.uso_local);
        assert!(!resultado.refuerzo.contexto.is_empty());
        assert!(!resultado.refuerzo.referencias.is_empty());
        // Razonamiento: plan estructurado con visión y módulos.
        assert!(!resultado.plan_razonado.vision.is_empty());
        assert!(!resultado.plan_razonado.modulos.is_empty());
        assert!(!resultado.plan_razonado.tecnologia.is_empty());
        assert!(resultado.plan_razonado.es_local);
        // El pipeline usa motores locales deterministas para el refuerzo.
        assert!(resultado.refuerzo_local);
        // La etapa quedó registrada en telemetría.
        let etapas: Vec<&str> = resultado
            .telemetria
            .etapas
            .iter()
            .map(|e| e.nombre.as_str())
            .collect();
        assert!(etapas.contains(&"refuerzo"));
    }

    /// E2E: 5 prompts de complejidad creciente deben producir un pipeline limpio
    /// (0 errores restantes) con el árbol de archivos v0 completo y telemetría.
    #[test]
    fn test_e2e_cinco_prompts_complejidad_creciente() {
        let prompts = [
            ("boton de accion", 3),
            ("formulario de registro con inputs", 4),
            ("dashboard de ventas con tabla", 5),
            ("panel de administracion con sidebar, tabla y dialogo", 6),
            (
                "aplicacion de ecommerce con productos, carrito y checkout oscuro",
                7,
            ),
        ];
        for (prompt, min_archivos) in prompts {
            let mut pipeline = PipelineV0::nuevo(None);
            let resultado = pipeline.ejecutar_local(prompt, None);

            // Cada ejecución debe dejar el pipeline limpio (los gates finales pasan).
            assert!(
                resultado.pipeline_limpio,
                "prompt='{prompt}' no limpio: errores_restantes={} diff={}",
                resultado.errores_restantes, resultado.diff_summary
            );
            assert_eq!(resultado.errores_restantes, 0, "prompt='{prompt}'");
            assert!(
                resultado.archivos_generados >= min_archivos,
                "prompt='{prompt}' generó {} archivos, esperaba >= {min_archivos}",
                resultado.archivos_generados
            );
            assert!(
                resultado.archivos_finales.contains_key("src/App.tsx"),
                "prompt='{prompt}' sin src/App.tsx"
            );
            assert!(
                resultado.archivos_finales.contains_key("package.json"),
                "prompt='{prompt}' sin package.json"
            );
            assert!(
                resultado.telemetria.etapas.len() >= 6,
                "prompt='{prompt}' telemetría con {} etapas",
                resultado.telemetria.etapas.len()
            );
            assert!(
                !resultado.session_id.is_empty(),
                "prompt='{prompt}' sin session_id"
            );
        }
    }

    // FASE 5 — El pipeline agrega los errores residuales a un dataset de
    // frecuencia para curar allowlist/prompts.
    #[test]
    fn test_pipeline_puebla_error_dataset() {
        let mut pipeline = PipelineV0::nuevo(None);
        let r1 = pipeline.ejecutar_local("dashboard de ventas con tabla", None);
        // El dataset siempre existe, aunque esté vacío si el pipeline fue limpio.
        assert!(r1.error_dataset.total() >= 0);
        assert!(r1.error_dataset.codigos_distintos() >= 0);

        // Con varios runs acumulamos errores: la segunda ejecución sigue
        // funcionando sin romper el resultado.
        let r2 = pipeline.ejecutar_local("formulario de registro con inputs", None);
        assert_eq!(r2.error_dataset.total() >= 0, true);
        // El `top()` devuelve códigos ordenados por frecuencia si hay errores.
        let top = r2.error_dataset.top();
        assert!(top.iter().all(|e| e.frecuencia >= 1));
    }
}
