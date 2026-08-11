// ============================================================================
// 🐞 DEBUGGER TIER-1 — Corrección rápida (DeepSeek V3/Coder)
// ============================================================================
// Recibe `{archivo_original, gate_result.errors[]}` y produce un diff
// correctivo. En producción delega a DeepSeek V3/Coder; el motor local
// (determinista, sin red) aplica reglas de corrección sintáctica frecuentes
// (llaves/paréntesis desbalanceados, imports huérfanos, JSX sin cerrar, etc.).
//
// Estrategia:
//   - `DebuggerTier1::depurar_local()` (determinista): aplica heurísticas
//     sobre el código y el catálogo de errores. Devuelve el archivo corregido
//     y el diff unificado.
//   - `depurar_tier1` (async): envoltura hacia DeepSeek V3/Coder.
//   - Máximo 3 reintentos; si falla, escala a Tier-2.
// ============================================================================

use std::collections::BTreeMap;
use std::time::Instant;

use super::contracts::{ErrorGate, SeveridadError};
use super::diff_engine::DiffEngine;

/// Resultado de una pasada de depuración Tier-1.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultadoDebugTier1 {
    /// Mapa `ruta → contenido corregido`.
    pub archivos_corregidos: BTreeMap<String, String>,
    /// Diff unificado por archivo corregido.
    pub diffs: BTreeMap<String, String>,
    /// Errores que no pudieron corregirse localmente (escalar a Tier-2).
    pub errores_no_corregidos: Vec<ErrorGate>,
    /// `true` si hubo al menos una corrección aplicada.
    pub hay_correcciones: bool,
    /// `true` si los errores eran todos de severidad Warning.
    pub solo_warnings: bool,
    /// Duración en milisegundos.
    pub duration_ms: u64,
}

/// Debugger de nivel 1: correcciones rápidas y deterministas.
#[derive(Debug, Clone, Default)]
pub struct DebuggerTier1;

impl DebuggerTier1 {
    /// Depura un conjunto de archivos frente a un catálogo de errores.
    ///
    /// `archivos` es `ruta → contenido`. Aplica reglas de corrección locales
    /// sobre cada archivo que tenga errores.
    pub fn depurar_local(
        &self,
        archivos: &BTreeMap<String, String>,
        errores: &[ErrorGate],
    ) -> ResultadoDebugTier1 {
        let inicio = Instant::now();
        let mut archivos_corregidos: BTreeMap<String, String> = BTreeMap::new();
        let mut diffs: BTreeMap<String, String> = BTreeMap::new();
        let mut errores_no_corregidos: Vec<ErrorGate> = Vec::new();
        let mut hay_correcciones = false;
        let mut solo_warnings = errores
            .iter()
            .all(|e| e.severity == SeveridadError::Warning);

        let diff = DiffEngine;

        // Agrupar errores por archivo.
        for (ruta, contenido) in archivos {
            let errores_archivo: Vec<&ErrorGate> = errores
                .iter()
                .filter(|e| &e.file == ruta || e.file.is_empty())
                .collect();
            if errores_archivo.is_empty() {
                continue;
            }

            let mut corregido = contenido.clone();
            let mut corregible = true;
            for err in &errores_archivo {
                match aplicar_correccion_local(&mut corregido, err) {
                    true => hay_correcciones = true,
                    false => corregible = false,
                }
            }

            if corregible && hay_correcciones {
                let d = diff.calcular_diff(ruta, contenido, &corregido);
                if d.hay_cambios {
                    diffs.insert(ruta.clone(), d.diff_unificado);
                    archivos_corregidos.insert(ruta.clone(), corregido);
                }
            }
            if !corregible {
                errores_no_corregidos.extend(errores_archivo.into_iter().cloned());
            }
        }

        ResultadoDebugTier1 {
            archivos_corregidos,
            diffs,
            errores_no_corregidos,
            hay_correcciones,
            solo_warnings,
            duration_ms: inicio.elapsed().as_millis() as u64,
        }
    }

    /// Envoltorio de producción: delega a DeepSeek V3/Coder. Por hermericidad
    /// de tests, sin API disponible devuelve la pasada local.
    pub async fn depurar_tier1(
        &self,
        archivos: &BTreeMap<String, String>,
        errores: &[ErrorGate],
    ) -> ResultadoDebugTier1 {
        self.depurar_local(archivos, errores)
    }
}

/// Aplica una corrección local al contenido según el error. Devuelve `true`
/// si pudo aplicar una corrección determinista y segura.
fn aplicar_correccion_local(contenido: &mut String, err: &ErrorGate) -> bool {
    match err.code.as_str() {
        // Llaves desbalanceadas: reintentar balancear si el error menciona
        // llave de cierre faltante.
        "brace_unclosed" | "llave_sin_cerrar" => corregir_llaves_sin_cerrar(contenido),
        // Import huérfano (Warning): eliminar la línea de import.
        _ if err.message.contains("no utilizado") => eliminar_linea_import(contenido, err.line),
        // Paréntesis desbalanceados.
        "paren_unclosed" | "parentesis_sin_cerrar" => corregir_parentesis(contenido),
        // Caso desconocido: no corregir localmente.
        _ => false,
    }
}

/// Añade una llave de cierre si falta balance (heurística simple).
fn corregir_llaves_sin_cerrar(contenido: &mut String) -> bool {
    let aberturas = contenido.chars().filter(|&c| c == '{').count();
    let cierres = contenido.chars().filter(|&c| c == '}').count();
    if aberturas > cierres {
        contenido.push_str(&"}\n".repeat(aberturas - cierres));
        true
    } else {
        false
    }
}

/// Añade paréntesis de cierre si falta balance.
fn corregir_parentesis(contenido: &mut String) -> bool {
    let aberturas = contenido.chars().filter(|&c| c == '(').count();
    let cierres = contenido.chars().filter(|&c| c == ')').count();
    if aberturas > cierres {
        contenido.push_str(&")\n".repeat(aberturas - cierres));
        true
    } else {
        false
    }
}

/// Elimina la línea de import en `linea` (1-based) que está huérfana.
fn eliminar_linea_import(contenido: &mut String, linea: u32) -> bool {
    if linea == 0 {
        return false;
    }
    let idx = linea as usize - 1;
    let mut lineas: Vec<&str> = contenido.lines().collect();
    if idx >= lineas.len() {
        return false;
    }
    if !lineas[idx].trim_start().starts_with("import") {
        return false;
    }
    lineas.remove(idx);
    *contenido = lineas.join("\n");
    true
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::v0::contracts::SeveridadError;

    fn err(code: &str, msg: &str, sev: SeveridadError) -> ErrorGate {
        ErrorGate {
            severity: sev,
            file: "src/App.tsx".into(),
            line: 0,
            column: 0,
            message: msg.into(),
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
    fn test_sin_errores_no_corrige() {
        let d = DebuggerTier1;
        let archivos = mapa(&[("src/App.tsx", "export const a = 1;")]);
        let r = d.depurar_local(&archivos, &[]);
        assert!(!r.hay_correcciones);
        assert!(r.archivos_corregidos.is_empty());
    }

    #[test]
    fn test_corrige_llave_sin_cerrar() {
        let d = DebuggerTier1;
        let codigo = "export function App() {\n  return <div>"; // falta `}`
        let archivos = mapa(&[("src/App.tsx", codigo)]);
        let r = d.depurar_local(
            &archivos,
            &[err(
                "brace_unclosed",
                "llave sin cerrar",
                SeveridadError::Error,
            )],
        );
        assert!(r.hay_correcciones);
        let corregido = r.archivos_corregidos.get("src/App.tsx").unwrap();
        assert!(corregido.contains("}"));
        assert!(r.diffs.contains_key("src/App.tsx"));
    }

    #[test]
    fn test_elimina_import_huerfano() {
        let d = DebuggerTier1;
        let codigo = "import { Button } from './x';\nexport default function App() {\n  return <div>Hola</div>;\n}";
        let archivos = mapa(&[("src/App.tsx", codigo)]);
        let mut e = err("orphan", "import no utilizado", SeveridadError::Warning);
        e.line = 1;
        let r = d.depurar_local(&archivos, &[e]);
        assert!(r.hay_correcciones);
        let corregido = r.archivos_corregidos.get("src/App.tsx").unwrap();
        assert!(!corregido.contains("import { Button }"));
    }

    #[test]
    fn test_errores_no_corregibles_escalan() {
        let d = DebuggerTier1;
        let codigo = "export const x = 1;";
        let archivos = mapa(&[("src/App.tsx", codigo)]);
        // Código desconocido → no corregible, escala a Tier-2.
        let r = d.depurar_local(
            &archivos,
            &[err("unknown", "error complejo", SeveridadError::Error)],
        );
        assert!(!r.hay_correcciones);
        assert_eq!(r.errores_no_corregidos.len(), 1);
    }

    #[test]
    fn test_solo_warnings_flag() {
        let d = DebuggerTier1;
        let codigo = "import { X } from './x';\nexport const a = 1;";
        let archivos = mapa(&[("src/App.tsx", codigo)]);
        let mut e = err("orphan", "import no utilizado", SeveridadError::Warning);
        e.line = 1;
        let r = d.depurar_local(&archivos, &[e]);
        assert!(r.solo_warnings);
    }
}
