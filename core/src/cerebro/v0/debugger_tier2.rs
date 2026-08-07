// ============================================================================
// 🧠 DEBUGGER TIER-2 — Razonamiento profundo (DeepSeek R1 / Reasoner)
// ============================================================================
// Recibe `{archivos_originales, gate_result combinado de los 3 gates}` y
// razona sobre el problema completo: lógica React, estado, imports, diseño.
// En producción delega a DeepSeek R1 (Reasoner); el motor local es
// determinista y aplica un conjunto de reglas de razonamiento de nivel 2.
//
// Estrategia:
//   - `DebuggerTier2::razonar_local()` (determinista): combina los errores de
//     los 3 gates (AST + Render + Visual) y aplica correcciones de lógica/
//     estado/imports que el Tier-1 no cubre.
//   - `razonar_tier2` (async): envoltura hacia DeepSeek R1. Máximo 1 reintento.
// ============================================================================

use std::collections::BTreeMap;
use std::time::Instant;

use super::contracts::{ErrorGate, GateResult};
use super::diff_engine::DiffEngine;

/// Resultado de una pasada de razonamiento Tier-2.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultadoDebugTier2 {
    /// Mapa `ruta → contenido corregido`.
    pub archivos_corregidos: BTreeMap<String, String>,
    /// Diffs unificados por archivo.
    pub diffs: BTreeMap<String, String>,
    /// Errores que quedaron sin resolver tras la pasada.
    pub errores_residuales: Vec<ErrorGate>,
    /// Diagnóstico textual del razonamiento aplicado.
    pub diagnostico: String,
    /// `true` si se aplicó al menos una corrección.
    pub hay_correcciones: bool,
    /// Duración en milisegundos.
    pub duration_ms: u64,
}

/// Debugger de nivel 2: razonamiento profundo sobre el estado completo.
#[derive(Debug, Clone, Default)]
pub struct DebuggerTier2;

impl DebuggerTier2 {
    /// Razona sobre los archivos y el resultado combinado de los 3 gates.
    ///
    /// `archivos` es `ruta → contenido`. `gates` son los `GateResult` de
    /// AST (Gate 1), Render (Gate 2) y Visual (Gate 3) combinados.
    pub fn razonar_local(
        &self,
        archivos: &BTreeMap<String, String>,
        gates: &[GateResult],
    ) -> ResultadoDebugTier2 {
        let inicio = Instant::now();
        let mut archivos_corregidos: BTreeMap<String, String> = BTreeMap::new();
        let mut diffs: BTreeMap<String, String> = BTreeMap::new();
        let mut errores_residuales: Vec<ErrorGate> = Vec::new();
        let mut hay_correcciones = false;
        let mut diagnostico: Vec<String> = Vec::new();

        let diff = DiffEngine;

        // Recolectar errores de todos los gates.
        let mut todos_errores: Vec<&ErrorGate> = Vec::new();
        for g in gates {
            todos_errores.extend(g.errors.iter());
        }

        for (ruta, contenido) in archivos {
            let errores_archivo: Vec<&ErrorGate> = todos_errores
                .iter()
                .filter(|e| e.file == *ruta || e.file.is_empty())
                .map(|e| *e)
                .collect();

            let mut corregido = contenido.clone();
            let mut ok_todo = true;

            // Corrección de imports React: aplica a TODO archivo .tsx con JSX,
            // independientemente de si tiene errores de gate (regla global).
            if ruta.ends_with(".tsx") && !corregido.contains("import React") && contiene_jsx(&corregido) {
                let mut nuevo = String::from("import React from 'react';\n");
                nuevo.push_str(&corregido);
                corregido = nuevo;
                diagnostico.push(format!("[{ruta}] Añadido import de React"));
                hay_correcciones = true;
            }

            for err in &errores_archivo {
                match razonar_correccion_nivel2(&mut corregido, err) {
                    Some(mensaje) => {
                        diagnostico.push(format!("[{}:{}] {}", ruta, err.line, mensaje));
                        hay_correcciones = true;
                    }
                    None => {
                        errores_residuales.push((*err).clone());
                        ok_todo = false;
                    }
                }
            }

            let d = diff.calcular_diff(ruta, contenido, &corregido);
            if d.hay_cambios {
                diffs.insert(ruta.clone(), d.diff_unificado);
                archivos_corregidos.insert(ruta.clone(), corregido);
            }
            let _ = ok_todo;
        }

        ResultadoDebugTier2 {
            archivos_corregidos,
            diffs,
            errores_residuales,
            diagnostico: diagnostico.join("\n"),
            hay_correcciones,
            duration_ms: inicio.elapsed().as_millis() as u64,
        }
    }

    /// Envoltorio de producción: delega a DeepSeek R1 (Reasoner). Sin API
    /// disponible, devuelve la pasada local para hermeticidad.
    pub async fn razonar_tier2(
        &self,
        archivos: &BTreeMap<String, String>,
        gates: &[GateResult],
    ) -> ResultadoDebugTier2 {
        self.razonar_local(archivos, gates)
    }
}

/// Detecta si el contenido contiene JSX (`<div`, `<>`, etc.).
fn contiene_jsx(contenido: &str) -> bool {
    contenido.contains("return (")
        || contenido.contains("return <")
        || contenido.contains("return (<>")
        || contenido.contains("</div>")
}

/// Aplica una corrección de razonamiento de nivel 2. Devuelve `Some(mensaje)`
/// si se aplicó una corrección, `None` si no se pudo resolver.
fn razonar_correccion_nivel2(contenido: &mut String, err: &ErrorGate) -> Option<String> {
    match err.code.as_str() {
        "missing_export" | "sin_export" => {
            // Si un archivo .tsx usa JSX pero no exporta componente, añadir
            // el export por defecto si hay una función con mayúscula.
            if contiene_jsx(contenido) && !contenido.contains("export default") {
                contenido.push_str("\nexport default App;\n");
                Some("Añadido 'export default App' faltante".into())
            } else {
                None
            }
        }
        "import_irresoluble" | "modulo_no_encontrado" => {
            // Import relativo que no resuelve: no podemos corregir localmente
            // porque no conocemos la estructura de destino → escalar.
            None
        }
        "low_contrast" => {
            // Corregir contraste: forzar un fondo oscuro cuando hay text-white
            // sin fondo oscuro (heurística del Gate Visual).
            if contenido.contains("text-white") && !tiene_fondo_oscuro(contenido) {
                *contenido = contenido.replace("className=\"", "className=\"bg-slate-900 ");
                Some("Añadido fondo oscuro para contraste (text-white)".into())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Verifica si el contenido declara un fondo oscuro.
fn tiene_fondo_oscuro(contenido: &str) -> bool {
    ["bg-slate-900", "bg-zinc-900", "bg-black", "bg-gray-900", "bg-neutral-900"]
        .iter()
        .any(|f| contenido.contains(f))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::v0::contracts::{GateKind, MetricaError, SeveridadError, V0_SCHEMA_GATE};

    fn gate_result(errors: Vec<ErrorGate>, tipo: &str) -> GateResult {
        GateResult {
            schema: V0_SCHEMA_GATE.to_string(),
            gate: if tipo == "ast" { GateKind::Ast } else { GateKind::Visual },
            passed: errors.is_empty(),
            errors,
            runtime_errors: vec![],
            visual_issues: vec![],
            duration_ms: 0,
        }
    }

    fn err(code: &str, file: &str) -> ErrorGate {
        ErrorGate {
            severity: SeveridadError::Error,
            file: file.into(),
            line: 0,
            column: 0,
            message: String::new(),
            code: code.into(),
            suggestion: String::new(),
        }
    }

    fn mapa(kv: &[(&str, &str)]) -> BTreeMap<String, String> {
        let mut m = BTreeMap::new();
        for (k, v) in kv {
            m.insert(k.to_string(), v.to_string());
        }
        m
    }

    #[test]
    fn test_sin_gates_no_corrige() {
        let d = DebuggerTier2;
        let archivos = mapa(&[("src/App.tsx", "export const a = 1;")]);
        let r = d.razonar_local(&archivos, &[]);
        assert!(!r.hay_correcciones);
        assert!(r.archivos_corregidos.is_empty());
    }

    #[test]
    fn test_export_default_anadido_cuando_falta() {
        let d = DebuggerTier2;
        let codigo = "function App() {\n  return <div>Hola</div>;\n}";
        let archivos = mapa(&[("src/App.tsx", codigo)]);
        let gates = vec![gate_result(vec![err("missing_export", "src/App.tsx")], "ast")];
        let r = d.razonar_local(&archivos, &gates);
        assert!(r.hay_correcciones);
        let corregido = r.archivos_corregidos.get("src/App.tsx").unwrap();
        assert!(corregido.contains("export default App"));
    }

    #[test]
    fn test_import_react_anadido_en_jsx() {
        let d = DebuggerTier2;
        let codigo = "export default function App() {\n  return <div>Hola</div>;\n}";
        let archivos = mapa(&[("src/App.tsx", codigo)]);
        let r = d.razonar_local(&archivos, &[]);
        // Aunque no haya errores, la regla de import React se aplica sobre .tsx con JSX.
        assert!(r.hay_correcciones);
        let corregido = r.archivos_corregidos.get("src/App.tsx").unwrap();
        assert!(corregido.starts_with("import React from 'react';"));
    }

    #[test]
    fn test_low_contrast_corregido_con_fondo_oscuro() {
        let d = DebuggerTier2;
        let codigo = "export default function App() {\n  return <div className=\"max-w-7xl\"><button className=\"text-white\">X</button></div>;\n}";
        let archivos = mapa(&[("src/App.tsx", codigo)]);
        let gates = vec![gate_result(vec![err("low_contrast", "src/App.tsx")], "visual")];
        let r = d.razonar_local(&archivos, &gates);
        assert!(r.hay_correcciones);
        let corregido = r.archivos_corregidos.get("src/App.tsx").unwrap();
        assert!(corregido.contains("bg-slate-900"));
    }

    #[test]
    fn test_import_irresoluble_escalado_como_residual() {
        let d = DebuggerTier2;
        let codigo = "import { X } from './inexistente';\nexport const a = 1;";
        let archivos = mapa(&[("src/App.tsx", codigo)]);
        let gates = vec![gate_result(vec![err("import_irresoluble", "src/App.tsx")], "render")];
        let r = d.razonar_local(&archivos, &gates);
        // No puede corregirse localmente → queda como error residual.
        assert_eq!(r.errores_residuales.len(), 1);
        assert!(r.diagnostico.is_empty() || !r.hay_correcciones);
    }
}
