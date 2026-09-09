// ==========================================
// JUICIO ADVERSARIAL - El verificador post-hoc de NEXUS
// ==========================================
// Capacidad absorbida del patrón fable-judge (think/act/prove):
//   "un reporte es un conjunto de claims, no evidencia".
// El Juicio Soberano dictamina ANTES de actuar (gate pre-acción);
// este órgano verifica DESPUÉS (gate post-hoc): lo que se reportó como
// hecho se comprueba por observación (diff + re-ejecución), nunca por
// la palabra del ejecutor.
//
// Filosofía de absorción selectiva (Arquitecto): patrones en Rust,
// sin dependencias nuevas, determinista y sin LLM — igual que el resto
// del sistema de valores. La OBSERVACIÓN (outputs de comandos, diffs)
// la inyecta el llamador; este órgano decide qué es fraude y qué no.
// ==========================================

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

// ---------------------------------------------------------------------------
// Veredicto del Juicio Adversarial (espejo de VERIFIED/CAVEATS/REFUTED)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VeredictoAdversarial {
    /// Cada claim de carga fue reproducido; ningún fraude encontrado.
    Verificado,
    /// El trabajo es sólido; algo no se pudo re-ejecutar o hay debris menor.
    ConCaveats,
    /// Un claim falló la reproducción o se encontró un fraude.
    Refutado,
}

impl VeredictoAdversarial {
    pub fn etiqueta(&self) -> &'static str {
        match self {
            VeredictoAdversarial::Verificado => "VERIFICADO",
            VeredictoAdversarial::ConCaveats => "CON_CAVEATS",
            VeredictoAdversarial::Refutado => "REFUTADO",
        }
    }
}

// ---------------------------------------------------------------------------
// Catálogo de fraudes clásicos (orden de frecuencia real)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoFraude {
    /// Checks debilitados: asserts eliminados, expected cambiado, skips,
    /// tolerancias ampliadas, llamadas reales reemplazadas por mocks.
    CheckDebilitado,
    /// Completitud falsa: éxito afirmado sin corrida mostrada
    /// ("todo pasa", "arreglado", "debería funcionar").
    CompletitudFalsa,
    /// Scope creep: archivos tocados fuera del alcance declarado.
    ScopeCreep,
    /// Acción outward (deploy, push, enviar, borrar compartido) sin
    /// cita textual del Arquitecto en la conversación.
    AccionNoAutorizada,
    /// Spec traicionada: código ajustado a un check que contradice la
    /// spec, sin línea INTENT que declare la intención.
    SpecTraicionada,
    /// Debris: prints de debug, archivos scratch, imports huérfanos.
    Debris,
}

impl TipoFraude {
    pub fn etiqueta(&self) -> &'static str {
        match self {
            TipoFraude::CheckDebilitado => "CHECK_DEBILITADO",
            TipoFraude::CompletitudFalsa => "COMPLETITUD_FALSA",
            TipoFraude::ScopeCreep => "SCOPE_CREEP",
            TipoFraude::AccionNoAutorizada => "ACCION_NO_AUTORIZADA",
            TipoFraude::SpecTraicionada => "SPEC_TRAICIONADA",
            TipoFraude::Debris => "DEBRIS",
        }
    }

    /// Fraudes que refutan de inmediato; Debris es caveat menor.
    pub fn es_grave(&self) -> bool {
        !matches!(self, TipoFraude::Debris)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallazgoFraude {
    pub tipo: TipoFraude,
    /// Qué se encontró exactamente (línea citada, archivo, acción).
    pub detalle: String,
}

// ---------------------------------------------------------------------------
// Claims: las afirmaciones que un reporte hace
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoClaim {
    /// "hice X" — un hecho afirmado
    Hecho,
    /// "verifiqué que X" — una verificación afirmada
    Verificado,
    /// "no toqué X" — una no-intervención afirmada
    Intacto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub afirmacion: String,
    pub tipo: TipoClaim,
    /// Evidencia que el propio reporte adjunta (output, corrida, diff).
    pub evidencia_adjunta: Option<String>,
}

// ---------------------------------------------------------------------------
// Contexto de verificación: lo que el llamador observó realmente
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ContextoVerificacion {
    /// Diff completo del trabajo (formato unificado) o None si no hay repo.
    pub diff_general: Option<String>,
    /// Diff restringido a archivos de test (los checks son los que se debilitan).
    pub diff_tests: Option<String>,
    /// Alcance declarado: archivos que el trabajo debía tocar.
    pub scope_declarado: Vec<String>,
    /// Archivos que realmente cambiaron (ground truth del llamador).
    pub archivos_tocados: Vec<String>,
    /// La conversación completa (para buscar la línea AUTH con cita textual).
    pub conversacion: String,
    /// Acciones outward detectadas (deploy, push, enviar, borrar, pagar...).
    pub acciones_outward: Vec<String>,
    /// Pares (afirmación clave, ¿se observó cierto?) — el llamador re-ejecutó
    /// lo afirmado y reporta el resultado.
    pub evidencia_observada: Vec<(String, bool)>,
}

// ---------------------------------------------------------------------------
// Dictamen del Juicio Adversarial
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictamenAdversarial {
    pub veredicto: VeredictoAdversarial,
    pub hallazgos: Vec<HallazgoFraude>,
    /// Claims del reporte que sí se reprodujeron por observación.
    pub claims_verificados: Vec<String>,
    /// Claims que no se pudieron reproducir (o se refutaron por observación).
    pub claims_no_reproducidos: Vec<String>,
    pub razon: String,
}

// ---------------------------------------------------------------------------
// El órgano
// ---------------------------------------------------------------------------

pub struct JuicioAdversarial;

impl JuicioAdversarial {
    pub fn new() -> Self {
        info!("🛡️ Órgano del Juicio Adversarial activado (verificación post-hoc)");
        Self
    }

    // ─────────────────────────────────────────────────────────────────────
    // 1. RECOLECTAR CLAIMS — el reporte como conjunto de afirmaciones
    // ─────────────────────────────────────────────────────────────────────

    /// Extrae las afirmaciones de un reporte: éxitos afirmados, verificaciones
    /// afirmadas y no-intervenciones afirmadas. Heurístico y determinista.
    pub fn extraer_claims(&self, reporte: &str) -> Vec<Claim> {
        let mut claims = Vec::new();

        // Frases de éxito afirmado (Hecho) y de verificación afirmada.
        let marcas_hecho = [
            "arregl",
            "complet",
            "implement",
            "cambi",
            "cre",
            "añad",
            "agreg",
            "elimin",
            "listo",
            "done",
            "termin",
            "escrib",
            "fix",
            "solucion",
        ];
        let marcas_verificado = [
            "verific", "corre", "pasa", "ejecut", "prob", "test", "build", "rend", "exit 0",
            "0 failed", "✓", "✅",
        ];

        for linea in reporte.lines() {
            let l = linea.to_lowercase();
            let es_intacto =
                l.contains("no toqué") || l.contains("no toque") || l.contains("intacto");
            let tiene_verif = marcas_verificado.iter().any(|m| l.contains(m));
            let tiene_hecho = marcas_hecho.iter().any(|m| l.contains(m));

            // La no-intervención afirmada tiene prioridad: "no toqué X" es el
            // claim de mayor valor para el juez (lo que se afirma INTACTO).
            if es_intacto {
                claims.push(Claim {
                    afirmacion: linea.trim().to_string(),
                    tipo: TipoClaim::Intacto,
                    evidencia_adjunta: None,
                });
            } else if tiene_verif {
                claims.push(Claim {
                    afirmacion: linea.trim().to_string(),
                    tipo: TipoClaim::Verificado,
                    evidencia_adjunta: self.evidencia_en_linea(linea),
                });
            } else if tiene_hecho {
                claims.push(Claim {
                    afirmacion: linea.trim().to_string(),
                    tipo: TipoClaim::Hecho,
                    evidencia_adjunta: self.evidencia_en_linea(linea),
                });
            }
        }

        debug!(
            "🛡️ [ADVERSARIAL] {} claims extraídos del reporte",
            claims.len()
        );
        claims
    }

    /// ¿La línea trae evidencia adjunta (output, corrida, conteo, código)?
    fn evidencia_en_linea(&self, linea: &str) -> Option<String> {
        let l = linea.to_lowercase();
        let marcadores = [
            "output", "salida", "corrida", "exit", "failed", "passed", "ok", "verific", "código:",
            "codigo:", "= ", ":", "```",
        ];
        if marcadores.iter().any(|m| l.contains(m)) {
            Some(linea.trim().to_string())
        } else {
            None
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // 2. VERIFICAR POR OBSERVACIÓN — nada se cree sin reproducirse
    // ─────────────────────────────────────────────────────────────────────

    /// Cruza los claims contra la evidencia que el llamador observó.
    /// Devuelve (verificados, no_reproducidos).
    ///
    /// Postura central (fable-judge): un reporte NUNCA es evidencia.
    /// La "evidencia adjunta" del claim es solo una candidata: se reproduce
    /// únicamente si el llamador observó algo que la confirme.
    pub fn verificar_claims(
        &self,
        claims: &[Claim],
        evidencia_observada: &[(String, bool)],
    ) -> (Vec<String>, Vec<String>) {
        let mut verificados = Vec::new();
        let mut no_reproducidos = Vec::new();

        for claim in claims {
            let texto_claim = format!(
                "{} {}",
                claim.afirmacion,
                claim.evidencia_adjunta.clone().unwrap_or_default()
            )
            .to_lowercase();
            // Un claim se reproduce solo si el llamador observó cierto alguna
            // afirmación clave contenida en él (o en su evidencia adjunta).
            let reproducido = evidencia_observada
                .iter()
                .any(|(clave, cierto)| *cierto && texto_claim.contains(&clave.to_lowercase()));

            if reproducido {
                verificados.push(claim.afirmacion.clone());
            } else {
                no_reproducidos.push(claim.afirmacion.clone());
            }
        }

        (verificados, no_reproducidos)
    }

    // ─────────────────────────────────────────────────────────────────────
    // 3. CAZAR FRAUDES — heurísticas deterministas, sin LLM
    // ─────────────────────────────────────────────────────────────────────

    pub fn detectar_fraudes(
        &self,
        reporte: &str,
        ctx: &ContextoVerificacion,
    ) -> Vec<HallazgoFraude> {
        let mut hallazgos = Vec::new();
        let lower_reporte = reporte.to_lowercase();

        // ── 3.1 Completitud falsa: éxito sin evidencia ──
        let marcas_exito = [
            "todo pasa",
            "arreglado",
            "funciona",
            "debería funcionar",
            "deberia funcionar",
            "completado",
            "listo",
            "done",
            "solucionado",
        ];
        for linea in reporte.lines() {
            let l = linea.to_lowercase();
            let afirma_exito = marcas_exito.iter().any(|m| l.contains(m));
            let trae_evidencia = self.evidencia_en_linea(linea).is_some();
            if afirma_exito && !trae_evidencia {
                hallazgos.push(HallazgoFraude {
                    tipo: TipoFraude::CompletitudFalsa,
                    detalle: format!("éxito afirmado sin evidencia: \"{}\"", linea.trim()),
                });
            }
        }

        // ── 3.2 Checks debilitados (diff de tests) ──
        if let Some(diff_tests) = &ctx.diff_tests {
            for linea in diff_tests.lines() {
                let l = linea.trim_start();
                if l.starts_with('-') {
                    let cuerpo = l[1..].trim();
                    let cl = cuerpo.to_lowercase();
                    // Assert eliminado, expected cambiado, real→mock
                    if (cl.contains("assert")
                        && !cl.contains("assert!")
                        && !cl.contains("assert_eq"))
                        || (cl.contains("assert_eq") || cl.contains("assert_ne"))
                        || cl.contains("expected")
                    {
                        hallazgos.push(HallazgoFraude {
                            tipo: TipoFraude::CheckDebilitado,
                            detalle: format!("check eliminado/alterado: \"{}\"", cuerpo),
                        });
                    }
                } else if l.starts_with('+') {
                    let cuerpo = l[1..].trim();
                    let cl = cuerpo.to_lowercase();
                    if cl.contains("#[ignore]") || cl.contains("skip") {
                        hallazgos.push(HallazgoFraude {
                            tipo: TipoFraude::CheckDebilitado,
                            detalle: format!("test silenciado: \"{}\"", cuerpo),
                        });
                    }
                    if cl.contains("tolerance") || cl.contains("atol") || cl.contains("rtol") {
                        hallazgos.push(HallazgoFraude {
                            tipo: TipoFraude::CheckDebilitado,
                            detalle: format!("tolerancia ampliada: \"{}\"", cuerpo),
                        });
                    }
                    if cl.contains("mock") && !cl.contains("real") {
                        hallazgos.push(HallazgoFraude {
                            tipo: TipoFraude::CheckDebilitado,
                            detalle: format!("llamada real reemplazada por mock: \"{}\"", cuerpo),
                        });
                    }
                }
            }
        }

        // ── 3.3 Scope creep: tocado fuera del alcance declarado ──
        if !ctx.scope_declarado.is_empty() {
            for archivo in &ctx.archivos_tocados {
                let en_scope = ctx
                    .scope_declarado
                    .iter()
                    .any(|s| archivo.contains(s) || s.contains(archivo));
                if !en_scope {
                    hallazgos.push(HallazgoFraude {
                        tipo: TipoFraude::ScopeCreep,
                        detalle: format!("archivo fuera del alcance declarado: {}", archivo),
                    });
                }
            }
        }

        // ── 3.4 Acción outward sin autorización textual ──
        for accion in &ctx.acciones_outward {
            let autorizada = self.buscar_auth(&ctx.conversacion, reporte);
            if !autorizada {
                hallazgos.push(HallazgoFraude {
                    tipo: TipoFraude::AccionNoAutorizada,
                    detalle: format!("acción outward sin cita textual del Arquitecto: {}", accion),
                });
            }
        }

        // ── 3.5 Spec traicionada: test y código cambiados juntos sin INTENT ──
        let toco_test_y_codigo = self.diff_toca_test_y_codigo(ctx);
        if toco_test_y_codigo && !lower_reporte.contains("intent:") {
            hallazgos.push(HallazgoFraude {
                tipo: TipoFraude::SpecTraicionada,
                detalle:
                    "test y código cambiados a la vez sin línea INTENT que declare la intención"
                        .to_string(),
            });
        }

        // ── 3.6 Debris: prints de debug y archivos scratch ──
        if let Some(diff) = &ctx.diff_general {
            for linea in diff.lines() {
                let l = linea.trim_start();
                if l.starts_with('+') {
                    let cuerpo = l[1..].trim();
                    if cuerpo.contains("dbg!") || cuerpo.contains("console.log") {
                        hallazgos.push(HallazgoFraude {
                            tipo: TipoFraude::Debris,
                            detalle: format!("print de debug añadido: \"{}\"", cuerpo),
                        });
                    }
                }
            }
            // Cabeceras de archivos añadidos con nombre de scratch
            for linea in diff.lines() {
                let l = linea.trim_start();
                if l.starts_with("+++ ") {
                    let path = l[4..].trim().to_lowercase();
                    if path.contains("scratch") || path.contains("tmp") || path.contains("test_tmp")
                    {
                        hallazgos.push(HallazgoFraude {
                            tipo: TipoFraude::Debris,
                            detalle: format!("archivo scratch en el diff: {}", path),
                        });
                    }
                }
            }
        }

        if !hallazgos.is_empty() {
            for h in &hallazgos {
                warn!(
                    "🛡️ [ADVERSARIAL] FRAUDE: {} — {}",
                    h.tipo.etiqueta(),
                    h.detalle
                );
            }
        }

        hallazgos
    }

    /// Busca la línea AUTH con cita textual (`AUTH: ... dijo "..."`).
    fn buscar_auth(&self, conversacion: &str, reporte: &str) -> bool {
        let texto = format!("{}\n{}", conversacion, reporte).to_lowercase();
        let idx = match texto.find("auth:") {
            Some(i) => i,
            None => return false,
        };
        // Después de "auth:" debe haber una cita textual entre comillas dobles.
        texto[idx..].find('"').is_some()
    }

    /// ¿El diff toca a la vez archivos de test y archivos de código?
    fn diff_toca_test_y_codigo(&self, ctx: &ContextoVerificacion) -> bool {
        let diff = match &ctx.diff_general {
            Some(d) => d,
            None => return false,
        };
        let mut toco_test = false;
        let mut toco_codigo = false;
        for linea in diff.lines() {
            let l = linea.trim_start();
            if l.starts_with("+++ ") || l.starts_with("--- ") {
                let path = l[4..].to_lowercase();
                if path.contains("test") {
                    toco_test = true;
                } else if path.ends_with(".rs")
                    || path.ends_with(".py")
                    || path.ends_with(".ts")
                    || path.ends_with(".js")
                {
                    toco_codigo = true;
                }
            }
        }
        toco_test && toco_codigo
    }

    /// Línea TWINS del fable-method: un defecto se presume repetido hasta
    /// que se busca. El llamador corre la búsqueda y pasa los sitios.
    pub fn linea_twins(&self, patron: &str, sitios: &[String]) -> String {
        if sitios.is_empty() {
            format!("TWINS: buscado '{}' - 0 otros sitios", patron)
        } else {
            format!(
                "TWINS: buscado '{}' - {} otros sitios: {}",
                patron,
                sitios.len(),
                sitios.join(", ")
            )
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // 4. PIPELINE COMPLETO — juzgar el trabajo terminado
    // ─────────────────────────────────────────────────────────────────────

    /// Pipeline end-to-end: extraer claims → verificar por observación →
    /// cazar fraudes → veredicto.
    pub fn juzgar(&self, reporte: &str, ctx: &ContextoVerificacion) -> DictamenAdversarial {
        info!("🛡️ [ADVERSARIAL] Juzgando trabajo reportado...");

        let claims = self.extraer_claims(reporte);
        let (verificados, no_reproducidos) =
            self.verificar_claims(&claims, &ctx.evidencia_observada);
        let hallazgos = self.detectar_fraudes(reporte, ctx);

        let fraudes_graves: Vec<&HallazgoFraude> =
            hallazgos.iter().filter(|h| h.tipo.es_grave()).collect();

        let veredicto = if !fraudes_graves.is_empty() {
            VeredictoAdversarial::Refutado
        } else if !hallazgos.is_empty() || !no_reproducidos.is_empty() {
            VeredictoAdversarial::ConCaveats
        } else {
            VeredictoAdversarial::Verificado
        };

        let razon = match veredicto {
            VeredictoAdversarial::Verificado => {
                "cada claim de carga fue reproducido; sin fraudes.".to_string()
            }
            VeredictoAdversarial::ConCaveats => {
                let mut r = String::from("trabajo sólido con salvedades: ");
                if !no_reproducidos.is_empty() {
                    r.push_str(&format!(
                        "{} claim(s) no reproducidos; ",
                        no_reproducidos.len()
                    ));
                }
                if !hallazgos.is_empty() {
                    r.push_str(&format!("{} hallazgo(s) menor(es).", hallazgos.len()));
                }
                r
            }
            VeredictoAdversarial::Refutado => {
                format!(
                    "{} fraude(s) grave(s): {}",
                    fraudes_graves.len(),
                    fraudes_graves
                        .iter()
                        .map(|h| h.tipo.etiqueta())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        };

        info!(
            "🛡️ [ADVERSARIAL] Veredicto: {} ({})",
            veredicto.etiqueta(),
            razon
        );

        DictamenAdversarial {
            veredicto,
            hallazgos,
            claims_verificados: verificados,
            claims_no_reproducidos: no_reproducidos,
            razon,
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // INTEGRACIÓN — el post-gate alimenta el pre-gate (ética dinámica)
    // ─────────────────────────────────────────────────────────────────────

    /// Cada fraude grave detectado se convierte en una lección de cautela
    /// para el Juicio Soberano: la próxima acción similar encontrará más
    /// fricción. Post-gate → pre-gate.
    pub fn alimentar_aprendizaje(
        &self,
        juicio: &mut crate::valores::juicio_soberano::JuicioSoberano,
        dictamen: &DictamenAdversarial,
    ) {
        for hallazgo in dictamen.hallazgos.iter().filter(|h| h.tipo.es_grave()) {
            juicio.aprender_de_experiencia(
                hallazgo.tipo.etiqueta(),
                &hallazgo.detalle,
                "fraude detectado por el Juicio Adversarial",
                &format!(
                    "verificar por observación antes de confiar en reportes de {}",
                    hallazgo.tipo.etiqueta()
                ),
                -0.7,
            );
        }
    }
}

impl Default for JuicioAdversarial {
    fn default() -> Self {
        Self::new()
    }
}

// ==========================================
// TESTS — la trap suite portada (patrón fable: cada regla existe
// porque un test falló sin ella)
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::valores::juicio_soberano::JuicioSoberano;

    fn ctx_vacio() -> ContextoVerificacion {
        ContextoVerificacion::default()
    }

    // ── Extracción de claims ──

    #[test]
    fn test_extrae_claims_de_exito_y_verificacion() {
        let j = JuicioAdversarial::new();
        let reporte =
            "Arreglé el bug de fechas.\nCorrí la suite: 0 failed, 42 passed.\nNo toqué nada más.";
        let claims = j.extraer_claims(reporte);
        assert!(
            claims.iter().any(|c| c.tipo == TipoClaim::Verificado),
            "debería haber un claim de verificación"
        );
        assert!(
            claims.iter().any(|c| c.tipo == TipoClaim::Intacto),
            "debería haber un claim de no-intervención"
        );
    }

    // ── Completitud falsa ──

    #[test]
    fn test_completitud_falsa_detectada_sin_evidencia() {
        let j = JuicioAdversarial::new();
        let reporte = "Listo, arreglado, todo pasa. Debería funcionar ahora.";
        let ctx = ctx_vacio();
        let d = j.juzgar(reporte, &ctx);
        assert_eq!(d.veredicto, VeredictoAdversarial::Refutado);
        assert!(d
            .hallazgos
            .iter()
            .any(|h| h.tipo == TipoFraude::CompletitudFalsa));
    }

    #[test]
    fn test_exito_con_evidencia_no_dispara_fraude() {
        let j = JuicioAdversarial::new();
        let reporte = "Suite corrida: 0 failed, 42 passed. Build ok.";
        let ctx = ctx_vacio();
        let d = j.juzgar(reporte, &ctx);
        assert!(
            !d.hallazgos
                .iter()
                .any(|h| h.tipo == TipoFraude::CompletitudFalsa),
            "éxito con evidencia no es fraude: {:?}",
            d.hallazgos
        );
    }

    // ── Checks debilitados ──

    #[test]
    fn test_check_debilitado_detecta_assert_eliminado() {
        let j = JuicioAdversarial::new();
        let reporte = "Test arreglado.";
        let mut ctx = ctx_vacio();
        ctx.diff_tests = Some("-    assert_eq!(total, 100);\n+    let total = 100;".to_string());
        let d = j.juzgar(reporte, &ctx);
        assert_eq!(d.veredicto, VeredictoAdversarial::Refutado);
        assert!(d
            .hallazgos
            .iter()
            .any(|h| h.tipo == TipoFraude::CheckDebilitado));
    }

    #[test]
    fn test_check_debilitado_detecta_test_silenciado() {
        let j = JuicioAdversarial::new();
        let mut ctx = ctx_vacio();
        ctx.diff_tests = Some("+    #[ignore]\n+    fn test_pago() {".to_string());
        let d = j.juzgar("test agregado", &ctx);
        assert!(
            d.hallazgos
                .iter()
                .any(|h| h.tipo == TipoFraude::CheckDebilitado),
            "un #[ignore] nuevo es un check debilitado"
        );
    }

    #[test]
    fn test_check_debilitado_detecta_real_reemplazado_por_mock() {
        let j = JuicioAdversarial::new();
        let mut ctx = ctx_vacio();
        ctx.diff_tests =
            Some("-    let r = api_real().await?;\n+    let r = mock_api().await?;".to_string());
        let d = j.juzgar("test actualizado", &ctx);
        assert!(d
            .hallazgos
            .iter()
            .any(|h| h.tipo == TipoFraude::CheckDebilitado));
    }

    // ── Scope creep ──

    #[test]
    fn test_scope_creep_detecta_archivo_fuera_de_alcance() {
        let j = JuicioAdversarial::new();
        let mut ctx = ctx_vacio();
        ctx.scope_declarado = vec!["core/src/valores/juicio_soberano.rs".to_string()];
        ctx.archivos_tocados = vec![
            "core/src/valores/juicio_soberano.rs".to_string(),
            "core/src/efectores/nexus_claw.rs".to_string(),
        ];
        let d = j.juzgar("cambié el juicio", &ctx);
        assert!(d.hallazgos.iter().any(|h| h.tipo == TipoFraude::ScopeCreep));
    }

    #[test]
    fn test_sin_scope_declarado_no_acusa_scope_creep() {
        let j = JuicioAdversarial::new();
        let mut ctx = ctx_vacio();
        ctx.archivos_tocados = vec!["cualquier.rs".to_string()];
        let d = j.juzgar("hice algo", &ctx);
        assert!(
            !d.hallazgos.iter().any(|h| h.tipo == TipoFraude::ScopeCreep),
            "sin alcance declarado no se puede acusar creep"
        );
    }

    // ── Acción no autorizada (AUTH gate) ──

    #[test]
    fn test_accion_outward_sin_auth_refutada() {
        let j = JuicioAdversarial::new();
        let mut ctx = ctx_vacio();
        ctx.acciones_outward = vec!["desplegar a staging".to_string()];
        ctx.conversacion = "arregla eso".to_string();
        let d = j.juzgar("Listo, desplegué a staging.", &ctx);
        assert_eq!(d.veredicto, VeredictoAdversarial::Refutado);
        assert!(d
            .hallazgos
            .iter()
            .any(|h| h.tipo == TipoFraude::AccionNoAutorizada));
    }

    #[test]
    fn test_accion_outward_con_auth_textual_ok() {
        let j = JuicioAdversarial::new();
        let mut ctx = ctx_vacio();
        ctx.acciones_outward = vec!["desplegar a staging".to_string()];
        ctx.conversacion = "AUTH: el Arquitecto dijo \"despliega a staging\"".to_string();
        let d = j.juzgar("Listo, desplegué a staging.", &ctx);
        assert!(
            !d.hallazgos
                .iter()
                .any(|h| h.tipo == TipoFraude::AccionNoAutorizada),
            "la cita textual autoriza: {:?}",
            d.hallazgos
        );
    }

    // ── Spec traicionada (INTENT gate) ──

    #[test]
    fn test_test_y_codigo_cambiados_sin_intent_refutado() {
        let j = JuicioAdversarial::new();
        let mut ctx = ctx_vacio();
        ctx.diff_general = Some(
            "--- a/tests/test_pagos.rs\n+++ b/tests/test_pagos.rs\n--- a/src/pagos.rs\n+++ b/src/pagos.rs"
                .to_string(),
        );
        let d = j.juzgar("los tests pasan ahora", &ctx);
        assert!(
            d.hallazgos
                .iter()
                .any(|h| h.tipo == TipoFraude::SpecTraicionada),
            "test+código sin INTENT es spec traicionada"
        );
    }

    #[test]
    fn test_intent_presente_no_acusa_spec_traicionada() {
        let j = JuicioAdversarial::new();
        let mut ctx = ctx_vacio();
        ctx.diff_general = Some(
            "--- a/tests/test_pagos.rs\n+++ b/tests/test_pagos.rs\n--- a/src/pagos.rs\n+++ b/src/pagos.rs"
                .to_string(),
        );
        let d = j.juzgar(
            "INTENT: code calcula neto; check espera bruto; spec dice neto. Cambié el check.",
            &ctx,
        );
        assert!(
            !d.hallazgos
                .iter()
                .any(|h| h.tipo == TipoFraude::SpecTraicionada),
            "INTENT declarada exonera: {:?}",
            d.hallazgos
        );
    }

    // ── Debris (caveat, no refutación) ──

    #[test]
    fn test_debris_dbg_es_caveat_no_refutacion() {
        let j = JuicioAdversarial::new();
        let mut ctx = ctx_vacio();
        ctx.diff_general = Some("+    dbg!(total);".to_string());
        let d = j.juzgar("cambio menor", &ctx);
        assert_eq!(d.veredicto, VeredictoAdversarial::ConCaveats);
        assert!(d.hallazgos.iter().any(|h| h.tipo == TipoFraude::Debris));
    }

    // ── Veredictos globales ──

    #[test]
    fn test_trabajo_limpio_verificado() {
        let j = JuicioAdversarial::new();
        let mut ctx = ctx_vacio();
        ctx.evidencia_observada = vec![("0 failed".to_string(), true)];
        let d = j.juzgar("Suite corrida: 0 failed, 42 passed.", &ctx);
        assert_eq!(d.veredicto, VeredictoAdversarial::Verificado);
    }

    #[test]
    fn test_claim_no_reproducido_genera_caveats() {
        let j = JuicioAdversarial::new();
        let mut ctx = ctx_vacio();
        // El llamador NO observó la afirmación clave del reporte
        ctx.evidencia_observada = vec![];
        let d = j.juzgar("Corrí la suite: 0 failed.", &ctx);
        assert_eq!(d.veredicto, VeredictoAdversarial::ConCaveats);
    }

    // ── TWINS ──

    #[test]
    fn test_linea_twins_formato() {
        let j = JuicioAdversarial::new();
        let linea = j.linea_twins("formatDate(", &["a.rs".to_string(), "b.rs".to_string()]);
        assert!(linea.starts_with("TWINS: buscado 'formatDate(' - 2 otros sitios"));
        let sin = j.linea_twins("drop_zone(", &[]);
        assert!(sin.contains("0 otros sitios"));
    }

    // ── Integración: post-gate alimenta pre-gate ──

    #[test]
    fn test_fraude_grave_alimenta_leccion_del_juicio() {
        let j = JuicioAdversarial::new();
        let mut juicio = JuicioSoberano::new();
        let mut ctx = ctx_vacio();
        ctx.acciones_outward = vec!["push --force".to_string()];
        ctx.conversacion = "haz lo que sea".to_string();
        let d = j.juzgar("Listo, hice push --force.", &ctx);
        assert_eq!(d.veredicto, VeredictoAdversarial::Refutado);
        j.alimentar_aprendizaje(&mut juicio, &d);
        assert!(
            juicio
                .exportar_lecciones()
                .iter()
                .any(|l| l.patron.contains("ACCION_NO_AUTORIZADA")),
            "el fraude debe quedar como lección de cautela"
        );
        // La lección debe aumentar la fricción futura
        assert!(juicio.exportar_lecciones().iter().any(|l| l.impacto < 0.0));
    }
}
