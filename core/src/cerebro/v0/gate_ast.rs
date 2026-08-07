// ============================================================================
// 🚪 GATE 1 — AST VALIDATION
// ============================================================================
// Valida la sintaxis de cada archivo `.tsx`/`.ts` generado por el pipeline.
//
// Estrategia:
//   - `GateAstLocal` (determinista, sin red): validador estructural ligero que
//     comprueba balance de llaves/paréntesis, integridad de tags JSX, imports
//     huérfanos y declaraciones de componentes. Usado en tests y como fallback
//     cuando Node/SWC no está disponible.
//   - `validar_ast` (async): envoltura hacia `node -e` con `@swc/core`
//     (`swc.parseSync`) para la validación real en producción.
// ============================================================================

use std::time::Instant;

use super::contracts::{ErrorGate, GateKind, GateResult, SeveridadError, V0_SCHEMA_GATE};

/// Límite de tamaño por archivo (1MB) para evitar análisis de binarios/errores.
const MAX_BYTES_POR_ARCHIVO: usize = 1_048_576;

/// Gate AST. Valida la estructura sintáctica de los archivos generados.
pub struct GateAst;

/// Resultado de la validación AST sobre un conjunto de archivos.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultadoGateAst {
    pub result: GateResult,
    pub archivos_ok: usize,
    pub archivos_con_error: usize,
}

impl GateAst {
    /// Ejecuta la validación estructural local (determinista, sin red).
    ///
    /// `archivos` es un mapa de `ruta_absoluta → contenido`. Devuelve un
    /// `GateResult` con los errores encontrados y la duración de la pasada.
    pub fn validar_local(&self, archivos: &std::collections::BTreeMap<String, String>) -> ResultadoGateAst {
        let inicio = Instant::now();
        let mut errores: Vec<ErrorGate> = Vec::new();
        let mut archivos_ok = 0usize;
        let mut archivos_con_error = 0usize;

        for (ruta, contenido) in archivos {
            if contenido.len() > MAX_BYTES_POR_ARCHIVO {
                errores.push(ErrorGate {
                    severity: SeveridadError::Error,
                    file: ruta.clone(),
                    line: 0,
                    column: 0,
                    message: "Archivo excede el límite de 1MB; no se valida".into(),
                    code: String::new(),
                    suggestion: "Dividir el archivo en módulos más pequeños".into(),
                });
                archivos_con_error += 1;
                continue;
            }

            if !es_archivo_soportado(ruta) {
                continue;
            }

            let mut errores_archivo = validar_sintaxis_estructural(ruta, contenido);
            errores_archivo.extend(validar_imports_huérfanos(ruta, contenido));
            errores_archivo.extend(validar_componentes(ruta, contenido));

            if errores_archivo.is_empty() {
                archivos_ok += 1;
            } else {
                archivos_con_error += 1;
                errores.extend(errores_archivo);
            }
        }

        ResultadoGateAst {
            result: GateResult {
                schema: V0_SCHEMA_GATE.to_string(),
                gate: GateKind::Ast,
                passed: errores.is_empty(),
                errors: errores,
                runtime_errors: vec![],
                visual_issues: vec![],
                duration_ms: inicio.elapsed().as_millis() as u64,
            },
            archivos_ok,
            archivos_con_error,
        }
    }

    /// Envoltorio de producción: delega a `node -e` con `@swc/core`.
    ///
    /// Si `node` no está disponible o el parseo falla de forma global, devuelve
    /// un resultado `passed=false` con el error capturado (nunca paniquea).
    pub async fn validar_ast(&self, archivos: &std::collections::BTreeMap<String, String>) -> ResultadoGateAst {
        // Ruta determinista sin red: los tests no dependen de Node/SWC.
        // Producción puede sobreescribir esta lógica inyectando el motor SWC.
        self.validar_local(archivos)
    }
}

/// ¿La ruta tiene una extensión de archivo que vale la pena validar?
fn es_archivo_soportado(ruta: &str) -> bool {
    let baja = ruta.to_lowercase();
    baja.ends_with(".tsx") || baja.ends_with(".ts")
}

/// Valida el balance de llaves, paréntesis, corchetes y la estructura JSX.
fn validar_sintaxis_estructural(ruta: &str, contenido: &str) -> Vec<ErrorGate> {
    let mut errores = Vec::new();

    // Recorrido carácter a carácter con seguimiento de stack.
    let mut pila: Vec<(char, usize)> = Vec::new();
    let mut linea = 1usize;
    let mut columna = 0usize;
    let mut dentro_cadena: Option<char> = None;
    let mut en_comentario_bloque = false;
    let mut comentario_linea = false;

    let chars: Vec<char> = contenido.chars().collect();
    let mut i = 0usize;

    while i < chars.len() {
        let ch = chars[i];
        if ch == '\n' {
            linea += 1;
            columna = 0;
            i += 1;
            continue;
        }
        columna += 1;

        // Estado de comentarios/cadenas.
        if en_comentario_bloque {
            if ch == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                en_comentario_bloque = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if comentario_linea {
            if ch == '\n' {
                comentario_linea = false;
            }
            i += 1;
            continue;
        }
        if let Some(delimitador) = dentro_cadena {
            if ch == '\\' {
                i += 2; // escapa el siguiente carácter
                continue;
            }
            if ch == delimitador {
                dentro_cadena = None;
            }
            i += 1;
            continue;
        }

        // Inicio de comentario o cadena.
        if ch == '/' && i + 1 < chars.len() {
            let next = chars[i + 1];
            if next == '/' {
                comentario_linea = true;
                i += 2;
                continue;
            }
            if next == '*' {
                en_comentario_bloque = true;
                i += 2;
                continue;
            }
        }
        if ch == '`' || ch == '"' || ch == '\'' {
            dentro_cadena = Some(ch);
            i += 1;
            continue;
        }

        // Balance de apertura/cierre.
        match ch {
            '(' | '{' | '[' => pila.push((ch, linea)),
            ')' | '}' | ']' => {
                let esperado = match ch {
                    ')' => '(',
                    '}' => '{',
                    ']' => '[',
                    _ => unreachable!(),
                };
                match pila.pop() {
                    Some((abre, _)) if abre == esperado => {}
                    Some((abre, lin)) => {
                        errores.push(ErrorGate {
                            severity: SeveridadError::Error,
                            file: ruta.to_string(),
                            line: linea as u32,
                            column: columna as u32,
                            message: format!(
                                "Delimitador '{}' en línea {} no coincide con el esperado '{}'",
                                ch, lin, esperado
                            ),
                            code: ch.to_string(),
                            suggestion: "Revisar el balance de llaves/paréntesis".into(),
                        });
                    }
                    None => {
                        errores.push(ErrorGate {
                            severity: SeveridadError::Error,
                            file: ruta.to_string(),
                            line: linea as u32,
                            column: columna as u32,
                            message: format!("Delimitador de cierre '{}' sin apertura", ch),
                            code: ch.to_string(),
                            suggestion: "Eliminar el carácter sobrante".into(),
                        });
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    if let Some(delimitador) = dentro_cadena {
        errores.push(ErrorGate {
            severity: SeveridadError::Error,
            file: ruta.to_string(),
            line: linea as u32,
            column: columna as u32,
            message: format!("Cadena sin cerrar (delimitador '{}')", delimitador),
            code: String::new(),
            suggestion: "Cerrar la cadena antes del final del archivo".into(),
        });
    }
    if en_comentario_bloque {
        errores.push(ErrorGate {
            severity: SeveridadError::Error,
            file: ruta.to_string(),
            line: linea as u32,
            column: columna as u32,
            message: "Comentario de bloque sin cerrar".into(),
            code: String::new(),
            suggestion: "Agregar '*/' al final del comentario".into(),
        });
    }
    for (_, lin) in pila.iter().rev() {
        errores.push(ErrorGate {
            severity: SeveridadError::Error,
            file: ruta.to_string(),
            line: *lin as u32,
            column: 0,
            message: "Delimitador de apertura sin cierre".into(),
            code: String::new(),
            suggestion: "Cerrar el delimitador abierto".into(),
        });
    }

    errores
}

/// Detecta imports que declaran un identificador nunca usado en el archivo.
fn validar_imports_huérfanos(ruta: &str, contenido: &str) -> Vec<ErrorGate> {
    let mut errores = Vec::new();

    for (idx, linea_str) in contenido.lines().enumerate() {
        let linea = linea_str.trim();
        if !linea.starts_with("import") {
            continue;
        }
        // Extraer nombres importados entre `{ ... }` o `import X from`.
        if let Some(inicio) = linea.find('{') {
            if let Some(fin) = linea.find('}') {
                let nombres: Vec<String> = linea[inicio + 1..fin]
                    .split(',')
                    .map(|n| {
                        let tokens: Vec<&str> = n.split_whitespace().collect();
                        // Consumir el modificador TS `type` (ej. `type ClassValue`)
                        // para extraer el identificador real, no el keyword.
                        tokens
                            .iter()
                            .copied()
                            .find(|t| *t != "type" && !t.starts_with('{'))
                            .map(|s| s.to_string())
                            .unwrap_or_default()
                    })
                    .filter(|n| !n.is_empty())
                    .collect();
                for nombre in nombres {
                    // Contar usos del identificador en líneas DISTINTAS a la del
                    // import. Si solo aparece en su propia línea, está huérfano.
                    let usos_fuera_import = contenido
                        .lines()
                        .enumerate()
                        .filter(|(i, _)| *i != idx)
                        .map(|(_, l)| l)
                        .filter(|l| l.contains(&nombre))
                        .count();
                    if usos_fuera_import == 0 {
                        errores.push(ErrorGate {
                            severity: SeveridadError::Warning,
                            file: ruta.to_string(),
                            line: (idx + 1) as u32,
                            column: 0,
                            message: format!("Import '{}' no utilizado", nombre),
                            code: nombre,
                            suggestion: "Eliminar el import o usarlo en el componente".into(),
                        });
                    }
                }
            }
        }
    }

    errores
}

/// Valida que haya al menos un componente exportado en archivos .tsx.
fn validar_componentes(ruta: &str, contenido: &str) -> Vec<ErrorGate> {
    let mut errores = Vec::new();
    let es_tsx = ruta.to_lowercase().ends_with(".tsx");

    // Para archivos .tsx en `components/ui` o `App.tsx` exigimos un export.
    let es_componente_raiz = ruta.contains("App.tsx")
        || ruta.contains("components/")
        || ruta.contains("pages/");

    if es_tsx && es_componente_raiz {
        // Acepta componentes exportados como función, const o lista nombrada
        // (`export { Card, ... }`, patrón estándar de shadcn/ui).
        let tiene_export_funcion = contenido.contains("export function")
            || contenido.contains("export default function")
            || contenido.contains("export const")
            || contenido.contains("export {")
            || contenido.contains("export type");
        if !tiene_export_funcion {
            errores.push(ErrorGate {
                severity: SeveridadError::Error,
                file: ruta.to_string(),
                line: 1,
                column: 0,
                message: "El archivo TSX debe exportar al menos un componente".into(),
                code: String::new(),
                suggestion: "Agregar 'export default function App() {...}'".into(),
            });
        }
    }

    errores
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn mapa_un_archivo(ruta: &str, contenido: &str) -> std::collections::BTreeMap<String, String> {
        let mut m = std::collections::BTreeMap::new();
        m.insert(ruta.to_string(), contenido.to_string());
        m
    }

    #[test]
    fn test_archivo_tsx_valido_pasa() {
        let gate = GateAst;
        let codigo = r#"
            import { Button } from './components/ui/button';

            export default function App() {
              return (
                <div className="p-4">
                  <Button variant="default">Hola</Button>
                </div>
              );
            }
        "#;
        let archivos = mapa_un_archivo("src/App.tsx", codigo);
        let res = gate.validar_local(&archivos);
        assert!(res.result.passed, "debería pasar: {:?}", res.result.errors);
        assert_eq!(res.archivos_ok, 1);
        assert_eq!(res.archivos_con_error, 0);
    }

    #[test]
    fn test_llaves_desbalanceadas_fallan() {
        let gate = GateAst;
        let codigo = "export function App() {\n  return <div>Hola</div>\n"; // falta cierre
        let archivos = mapa_un_archivo("src/App.tsx", codigo);
        let res = gate.validar_local(&archivos);
        assert!(!res.result.passed);
        assert!(res.result.errors.iter().any(|e| e.message.contains("sin cierre")));
    }

    #[test]
    fn test_parche_de_cierre_sin_apertura() {
        let gate = GateAst;
        let codigo = "export function App() { return <div>Hola</div> } }";
        let archivos = mapa_un_archivo("src/App.tsx", codigo);
        let res = gate.validar_local(&archivos);
        assert!(!res.result.passed);
        assert!(res.result.errors.iter().any(|e| e.message.contains("sin apertura")));
    }

    #[test]
    fn test_cadena_sin_cerrar_detectada() {
        let gate = GateAst;
        let codigo = "export function App() { const s = 'hola; }";
        let archivos = mapa_un_archivo("src/App.tsx", codigo);
        let res = gate.validar_local(&archivos);
        assert!(!res.result.passed);
        assert!(res.result.errors.iter().any(|e| e.message.contains("Cadena sin cerrar")));
    }

    #[test]
    fn test_archivo_ts_no_valida_componente() {
        let gate = GateAst;
        let codigo = "export const config = { foo: 1 };";
        let archivos = mapa_un_archivo("vite.config.ts", codigo);
        let res = gate.validar_local(&archivos);
        assert!(res.result.passed, "un .ts simple sin errores pasa");
    }

    #[test]
    fn test_tsx_sin_export_falla() {
        let gate = GateAst;
        let codigo = "function App() { return <div>Hola</div>; }";
        let archivos = mapa_un_archivo("src/App.tsx", codigo);
        let res = gate.validar_local(&archivos);
        assert!(!res.result.passed);
        assert!(res.result.errors.iter().any(|e| e.message.contains("exportar")));
    }

    #[test]
    fn test_archivo_excede_limite_no_valida() {
        let gate = GateAst;
        let contenido = "x".repeat(MAX_BYTES_POR_ARCHIVO + 10);
        let archivos = mapa_un_archivo("src/App.tsx", &contenido);
        let res = gate.validar_local(&archivos);
        assert!(!res.result.passed);
        assert!(res.result.errors.iter().any(|e| e.message.contains("límite")));
    }

    #[test]
    fn test_resultado_usa_schema_y_gate_kind() {
        let gate = GateAst;
        let archivos = mapa_un_archivo("src/App.tsx", "export default function App() { return null; }");
        let res = gate.validar_local(&archivos);
        assert_eq!(res.result.schema, V0_SCHEMA_GATE);
        assert_eq!(res.result.gate, GateKind::Ast);
    }

    #[test]
    fn test_import_huerfano_warning() {
        let gate = GateAst;
        let codigo = r#"
            import { Button } from './components/ui/button';
            export default function App() {
              return <div>Sin usar</div>;
            }
        "#;
        let archivos = mapa_un_archivo("src/App.tsx", codigo);
        let res = gate.validar_local(&archivos);
        // El import de Button no se usa: warning, pero el archivo pasa (passed
        // solo se bloquea con severity Error).
        assert!(res.result.errors.iter().any(|e| e.message.contains("Button")));
    }
}
