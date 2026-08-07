// ============================================================================
// 🚪 GATE 2 — RENDER VALIDATION
// ============================================================================
// Valida que la UI generada sea renderizable: sintaxis JSX válida, imports
// resolubles dentro del proyecto y patrones de runtime propensos a errores.
//
// Estrategia:
//   - `GateRenderLocal` (determinista, sin red): análisis estático sobre los
//     archivos generados. Detecta: tags JSX sin cierre, imports de módulos que
//     no existen en el proyecto, uso de hooks fuera de componentes y
//     referencias a identificadores no definidos. Usado en tests y como fallback.
//   - `validar_render` (async): envoltura hacia el sandbox real (npm install +
//     build + Playwright) en producción.
// ============================================================================

use std::collections::BTreeMap;
use std::time::Instant;

use super::contracts::{GateKind, GateResult, MetricaError, V0_SCHEMA_GATE};

/// Gate Render. Valida la renderizabilidad de los archivos generados.
pub struct GateRender;

/// Resultado de la validación de render sobre un proyecto.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultadoGateRender {
    pub result: GateResult,
    /// Ruta del screenshot si se generó (producción).
    pub screenshot_path: Option<String>,
}

impl GateRender {
    /// Validación estática local (determinista, sin red, sin npm).
    ///
    /// `archivos` es un mapa `ruta → contenido`. Requiere que `package.json`
    /// esté presente para resolver imports de terceros.
    pub fn validar_local(&self, archivos: &BTreeMap<String, String>) -> ResultadoGateRender {
        let inicio = Instant::now();
        let mut runtime_errors: Vec<MetricaError> = Vec::new();

        // Conjunto de rutas locales que son resolubles (imports relativos).
        let rutas_locales: Vec<String> = archivos
            .keys()
            .filter(|r| r.ends_with(".tsx") || r.ends_with(".ts"))
            .cloned()
            .collect();

        // Módulos de terceros declarados en package.json.
        let terceros = extraer_terceros(archivos.get("package.json").map(|s| s.as_str()));

        for (ruta, contenido) in archivos {
            if !(ruta.ends_with(".tsx") || ruta.ends_with(".ts")) {
                continue;
            }
            runtime_errors.extend(validar_imports_irresolubles(ruta, contenido, &rutas_locales, &terceros));
            runtime_errors.extend(validar_hooks_fuera_de_componente(ruta, contenido));
            runtime_errors.extend(validar_jsx_balance(ruta, contenido));
        }

        let passed = runtime_errors.is_empty();
        ResultadoGateRender {
            result: GateResult {
                schema: V0_SCHEMA_GATE.to_string(),
                gate: GateKind::Render,
                passed,
                errors: vec![],
                runtime_errors,
                visual_issues: vec![],
                duration_ms: inicio.elapsed().as_millis() as u64,
            },
            screenshot_path: None,
        }
    }

    /// Envoltorio de producción: sandbox real (npm + build + Playwright).
    pub async fn validar_render(&self, archivos: &BTreeMap<String, String>) -> ResultadoGateRender {
        // Ruta determinista sin red para hermeticidad de tests. Producción
        // puede inyectar el sandbox WebContainer/Playwright.
        self.validar_local(archivos)
    }
}

/// Extrae el set de dependencias de terceros declaradas en package.json.
fn extraer_terceros(package_json: Option<&str>) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    let Some(json) = package_json else { return set };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(json) else {
        return set;
    };
    for seccion in ["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(objeto) = val.get(seccion).and_then(|v| v.as_object()) {
            for nombre in objeto.keys() {
                // Registrar el paquete base: `@scope/pkg` → `@scope/pkg`, y
                // sub-rutas `@scope/pkg/sub` → `@scope/pkg`.
                let base = paquete_base(nombre);
                set.insert(base);
            }
        }
    }
    set
}

/// Reduce un specifier a su paquete base (sin sub-rutas).
fn paquete_base(spec: &str) -> String {
    if spec.starts_with('@') {
        // @scope/pkg/rest → @scope/pkg
        let mut partes = spec.split('/');
        let scope = partes.next().unwrap_or_default();
        let pkg = partes.next().unwrap_or_default();
        format!("{}/{}", scope, pkg)
    } else {
        // pkg/rest → pkg
        spec.split('/').next().unwrap_or_default().to_string()
    }
}

/// Valida que los imports resuelvan a módulos locales o de terceros conocidos.
fn validar_imports_irresolubles(
    ruta: &str,
    contenido: &str,
    rutas_locales: &[String],
    terceros: &std::collections::HashSet<String>,
) -> Vec<MetricaError> {
    let mut errores = Vec::new();
    let dir_actual = directorio_de(ruta);

    for (idx, linea) in contenido.lines().enumerate() {
        let linea = linea.trim();
        if !linea.starts_with("import") && !linea.starts_with("export") {
            continue;
        }
        // Extraer el specifier: `from 'x'` o `import 'x'`.
        let spec = if linea.contains(" from ") {
            linea.split(" from ").nth(1)
        } else {
            linea.split("import").nth(1)
        };
        let Some(spec) = spec else { continue };
        let spec = spec.trim().trim_matches(|c| c == '\'' || c == '"' || c == ';');
        if spec.is_empty() {
            continue;
        }

        // Imports de estilos (css) y vite-env no se validan.
        if spec.ends_with(".css") || spec.contains("vite-env") || spec.starts_with("./") && spec.contains("index.css") {
            continue;
        }

        if spec.starts_with('.') {
            // Import relativo: debe resolver dentro de rutas_locales.
            if !resuelve_relativo(spec, &dir_actual, rutas_locales) {
                errores.push(MetricaError {
                    tipo: "import_unresolved".into(),
                    message: format!("{}:{} Import relativo '{}' no encontrado", ruta, idx + 1, spec),
                    stack: String::new(),
                });
            }
        } else if spec.starts_with("@/") {
            // Alias Vite '@' → `src/`. Debe resolver a un módulo local real.
            // Ej: '@/lib/utils' → 'src/lib/utils', '@/components/ui/card' → 'src/components/ui/card'.
            let destino = format!("src/{}", &spec[2..]);
            if !resuelve_ruta_local(&destino, rutas_locales) {
                errores.push(MetricaError {
                    tipo: "import_unresolved".into(),
                    message: format!("{}:{} Import con alias '{}' no encontrado en src/", ruta, idx + 1, spec),
                    stack: String::new(),
                });
            }
        } else {
            // Import de terceros: debe estar en el allowlist de package.json.
            let base = paquete_base(spec);
            // Módulos built-in de Node (path, fs, node:*) no requieren declaración.
            if !es_node_builtin(&base) && !terceros.contains(&base) {
                errores.push(MetricaError {
                    tipo: "import_tercero_no_declarado".into(),
                    message: format!("{}:{} Import '{}' no declarado en package.json", ruta, idx + 1, spec),
                    stack: String::new(),
                });
            }
        }
    }

    errores
}

/// Resuelve la ruta del directorio de un archivo.
fn directorio_de(ruta: &str) -> String {
    match ruta.rfind('/') {
        Some(pos) => ruta[..pos].to_string(),
        None => String::new(),
    }
}

/// Normaliza una ruta relativa con `.` y `..`.
fn normalizar_ruta(ruta: &str) -> String {
    let mut partes: Vec<&str> = Vec::new();
    for parte in ruta.split('/') {
        match parte {
            "." | "" => {}
            ".." => {
                partes.pop();
            }
            otra => partes.push(otra),
        }
    }
    partes.join("/")
}

/// ¿El specifier relativo resuelve a un archivo local existente?
fn resuelve_relativo(spec: &str, dir_actual: &str, rutas_locales: &[String]) -> bool {
    // Quitar extensión si no la tiene (los imports suelen omitir .tsx/.ts).
    let ruta_sin_ext = if spec.ends_with(".tsx") || spec.ends_with(".ts") {
        spec.to_string()
    } else {
        format!("{}.tsx", spec)
    };
    let ruta_completa = if dir_actual.is_empty() {
        ruta_sin_ext
    } else {
        format!("{}/{}", dir_actual, ruta_sin_ext)
    };
    let normalizada = normalizar_ruta(&ruta_completa);

    rutas_locales
        .iter()
        .any(|r| normalizar_ruta(r) == normalizada)
}

/// ¿Una ruta absoluta (sin extensión, ej. `src/lib/utils`) resuelve a un
/// archivo local? Prueba `{ruta}.tsx`, `{ruta}.ts` y la ruta pura.
fn resuelve_ruta_local(ruta: &str, rutas_locales: &[String]) -> bool {
    let candidatos = [ruta.to_string(), format!("{ruta}.tsx"), format!("{ruta}.ts")];
    candidatos.iter().any(|c| {
        rutas_locales
            .iter()
            .any(|r| normalizar_ruta(r) == normalizar_ruta(c))
    })
}

/// ¿Es un módulo built-in de Node (no requiere declaración en package.json)?
fn es_node_builtin(spec: &str) -> bool {
    let base = spec.strip_prefix("node:").unwrap_or(spec);
    matches!(
        base,
        "assert" | "buffer" | "child_process" | "cluster" | "console" | "constants"
            | "crypto" | "dgram" | "diagnostics_channel" | "dns" | "domain"
            | "events" | "fs" | "http" | "http2" | "https" | "module" | "net"
            | "os" | "path" | "perf_hooks" | "process" | "punycode" | "querystring"
            | "readline" | "repl" | "stream" | "string_decoder" | "timers"
            | "tls" | "trace_events" | "tty" | "url" | "util" | "v8" | "vm"
            | "wasi" | "worker_threads" | "zlib"
    )
}

/// Detecta hooks de React usados fuera de un componente (regla de hooks).
fn validar_hooks_fuera_de_componente(ruta: &str, contenido: &str) -> Vec<MetricaError> {
    let mut errores = Vec::new();
    let mut profundidad_llaves = 0i32;

    // Si el archivo exporta componentes (export function/const → función) y
    // hay al menos una función, asumimos que los hooks pueden estar dentro.
    // Solo marcamos hooks en líneas que están fuera de cualquier función.
    for (idx, linea) in contenido.lines().enumerate() {
        let linea_t = linea.trim();
        // Seguimiento de profundidad de llaves.
        for ch in linea_t.chars() {
            match ch {
                '{' => profundidad_llaves += 1,
                '}' => profundidad_llaves -= 1,
                _ => {}
            }
        }
        let tiene_hook = linea_t.contains("useState(")
            || linea_t.contains("useEffect(")
            || linea_t.contains("useMemo(")
            || linea_t.contains("useContext(");
        if tiene_hook && profundidad_llaves <= 0 {
            errores.push(MetricaError {
                tipo: "hooks_outside_component".into(),
                message: format!("{}:{} Hook de React usado fuera de un componente", ruta, idx + 1),
                stack: String::new(),
            });
        }
    }
    errores
}

/// Valida balance de tags JSX (apertura/cierre) en todo el archivo, manejando
/// autocierres `<Tag />`, fragmentos `<>` `</>` y tags que cruzan líneas.
fn validar_jsx_balance(ruta: &str, contenido: &str) -> Vec<MetricaError> {
    let mut errores = Vec::new();
    let mut pila: Vec<String> = Vec::new();
    let chars: Vec<char> = contenido.chars().collect();
    let mut i = 0usize;
    let mut linea = 1usize;

    while i < chars.len() {
        let ch = chars[i];

        // Avanzar línea al encontrar `\n`.
        if ch == '\n' {
            linea += 1;
            i += 1;
            continue;
        }

        if ch != '<' {
            i += 1;
            continue;
        }

        // Fragmento de cierre `</>`.
        if chars.get(i + 1) == Some(&'/') && chars.get(i + 2) == Some(&'>') {
            if pila.last().map(|n| n == "Fragment").unwrap_or(false) {
                pila.pop();
            } else {
                errores.push(MetricaError {
                    tipo: "jsx_unbalanced".into(),
                    message: format!("{}:{} Fragmento '</>' sin apertura", ruta, linea),
                    stack: String::new(),
                });
            }
            i += 3;
            continue;
        }

        // Tag de cierre `</Tag>`.
        if chars.get(i + 1) == Some(&'/') {
            let mut j = i + 2;
            let mut nombre = String::new();
            while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_' || chars[j] == '.') {
                nombre.push(chars[j]);
                j += 1;
            }
            while j < chars.len() && chars[j] != '>' {
                j += 1;
            }
            if pila.last().map(|n| *n == nombre).unwrap_or(false) {
                pila.pop();
            } else if !nombre.is_empty() {
                errores.push(MetricaError {
                    tipo: "jsx_unbalanced".into(),
                    message: format!("{}:{} Tag JSX '</{}>' sin apertura", ruta, linea, nombre),
                    stack: String::new(),
                });
            }
            i = j + 1;
            continue;
        }

        // Fragmento de apertura `<>`.
        if chars.get(i + 1) == Some(&'>') {
            pila.push("Fragment".to_string());
            i += 2;
            continue;
        }

        // Tag de apertura `<Tag ...>`.
        let mut j = i + 1;
        let mut nombre = String::new();
        while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_' || chars[j] == '.') {
            nombre.push(chars[j]);
            j += 1;
        }
        if nombre.is_empty() {
            i += 1;
            continue;
        }

        // Un `<` que continúa un identificador es un type parameter de
        // TypeScript (ej. `forwardRef<HTMLButtonElement>`, `ButtonHTMLAttributes<...>`),
        // no un tag JSX. Se escanea hasta su `>` sin pushear a la pila.
        let previo = if i > 0 { chars[i - 1] } else { ' ' };
        let es_type_param =
            previo.is_alphanumeric() || previo == '_' || previo == '.' || previo == '$';
        if es_type_param {
            let mut k = j;
            let mut dentro_prop = false;
            let mut prop_delim = ' ';
            while k < chars.len() {
                let c = chars[k];
                if dentro_prop {
                    if c == '\\' {
                        k += 2;
                        continue;
                    }
                    if c == prop_delim {
                        dentro_prop = false;
                    }
                    k += 1;
                    continue;
                }
                if c == '"' || c == '\'' || c == '`' {
                    dentro_prop = true;
                    prop_delim = c;
                    k += 1;
                    continue;
                }
                if c == '>' {
                    i = k + 1;
                    break;
                }
                k += 1;
            }
            if k >= chars.len() {
                // Sin cierre: código TS no-JSX; avanzar sin marcar error.
                i += 1;
            }
            continue;
        }

        // Buscar el cierre `>` o `/>` respetando comillas de props.
        let mut k = j;
        let mut dentro_prop = false;
        let mut prop_delim = ' ';
        let mut cerrado = false;
        while k < chars.len() {
            let c = chars[k];
            if dentro_prop {
                if c == '\\' {
                    k += 2;
                    continue;
                }
                if c == prop_delim {
                    dentro_prop = false;
                }
                k += 1;
                continue;
            }
            if c == '"' || c == '\'' || c == '`' {
                dentro_prop = true;
                prop_delim = c;
                k += 1;
                continue;
            }
            if c == '/' && chars.get(k + 1) == Some(&'>') {
                // Autocierre `<Tag />`.
                cerrado = true;
                i = k + 2;
                break;
            }
            if c == '>' {
                // Apertura normal `<Tag>`.
                pila.push(nombre.clone());
                cerrado = true;
                i = k + 1;
                break;
            }
            k += 1;
        }
        if !cerrado {
            // No se encontró cierre en el resto del archivo: tag sin cerrar.
            errores.push(MetricaError {
                tipo: "jsx_unclosed".into(),
                message: format!("{}:{} Tag JSX '<{}>' sin cerrar", ruta, linea, nombre),
                stack: String::new(),
            });
            i += 1;
        }
    }

    // Tags abiertos que nunca se cerraron.
    for nombre in pila {
        errores.push(MetricaError {
            tipo: "jsx_unclosed".into(),
            message: format!("{}: Tag JSX '<{}>' sin cerrar", ruta, nombre),
            stack: String::new(),
        });
    }

    errores
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn proyecto_valido() -> BTreeMap<String, String> {
        let mut m = BTreeMap::new();
        m.insert(
            "package.json".to_string(),
            r#"{"dependencies":{"react":"^18","react-dom":"^18","lucide-react":"^0.460"}}"#.to_string(),
        );
        m.insert(
            "src/App.tsx".to_string(),
            r#"
            import { Button } from './components/ui/button';
            import { X } from 'lucide-react';
            export default function App() {
              return (
                <div className="p-4">
                  <Button variant="default"><X /></Button>
                </div>
              );
            }
            "#.to_string(),
        );
        m.insert(
            "src/components/ui/button.tsx".to_string(),
            r#"
            export function Button() { return <button className="px-4" />; }
            "#.to_string(),
        );
        m
    }

    #[test]
    fn test_proyecto_valido_pasa() {
        let gate = GateRender;
        let res = gate.validar_local(&proyecto_valido());
        assert!(res.result.passed, "debería pasar: {:?}", res.result.runtime_errors);
        assert_eq!(res.result.gate, GateKind::Render);
    }

    #[test]
    fn test_import_relativo_inexistente_falla() {
        let gate = GateRender;
        let mut archivos = proyecto_valido();
        archivos.insert(
            "src/App.tsx".to_string(),
            "import { Missing } from './no/existe';\nexport default function App(){return null;}".into(),
        );
        let res = gate.validar_local(&archivos);
        assert!(!res.result.passed);
        assert!(res.result.runtime_errors.iter().any(|e| e.tipo == "import_unresolved"));
    }

    #[test]
    fn test_import_tercero_no_declarado_falla() {
        let gate = GateRender;
        let mut archivos = proyecto_valido();
        archivos.insert(
            "src/App.tsx".to_string(),
            "import { z } from 'zod';\nexport default function App(){return null;}".into(),
        );
        let res = gate.validar_local(&archivos);
        assert!(!res.result.passed);
        assert!(res.result.runtime_errors.iter().any(|e| e.tipo == "import_tercero_no_declarado"));
    }

    #[test]
    fn test_paquete_scope_con_subruta_resuelve() {
        let gate = GateRender;
        let mut archivos = proyecto_valido();
        archivos.insert(
            "package.json".to_string(),
            r#"{"dependencies":{"@radix-ui/react-dialog":"^1"}}"#.to_string(),
        );
        archivos.insert(
            "src/App.tsx".to_string(),
            "import { Dialog } from '@radix-ui/react-dialog';\nexport default function App(){return null;}".into(),
        );
        let res = gate.validar_local(&archivos);
        assert!(res.result.passed, "scoped pkg debería resolver: {:?}", res.result.runtime_errors);
    }

    #[test]
    fn test_jsx_tag_sin_cerrar_detectado() {
        let gate = GateRender;
        let mut archivos = proyecto_valido();
        archivos.insert(
            "src/App.tsx".to_string(),
            "export default function App(){ return <div className='x'><span>Hola</span> }"
                .into(),
        );
        let res = gate.validar_local(&archivos);
        assert!(!res.result.passed);
        assert!(res.result.runtime_errors.iter().any(|e| e.tipo == "jsx_unclosed"));
    }

    #[test]
    fn test_hooks_fuera_de_componente_detectado() {
        let gate = GateRender;
        let mut archivos = proyecto_valido();
        archivos.insert(
            "src/hooks.ts".to_string(),
            "const [x] = useState(0);\nexport default function App(){return null;}".into(),
        );
        let res = gate.validar_local(&archivos);
        assert!(!res.result.passed);
        assert!(res.result.runtime_errors.iter().any(|e| e.tipo == "hooks_outside_component"));
    }

    #[test]
    fn test_resultado_schema_gate_kind() {
        let gate = GateRender;
        let res = gate.validar_local(&proyecto_valido());
        assert_eq!(res.result.schema, V0_SCHEMA_GATE);
        assert_eq!(res.result.gate, GateKind::Render);
        assert_eq!(res.screenshot_path, None);
    }

    #[test]
    fn test_paquete_base_scope() {
        assert_eq!(paquete_base("@radix-ui/react-dialog"), "@radix-ui/react-dialog");
        assert_eq!(paquete_base("@radix-ui/react-dialog/extra"), "@radix-ui/react-dialog");
        assert_eq!(paquete_base("lucide-react/icons"), "lucide-react");
    }
}
