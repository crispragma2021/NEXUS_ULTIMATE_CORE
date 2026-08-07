// ============================================================================
// ✨ PULIDO V0-REAL — Design tokens, export y dataset de errores
// ============================================================================
// FASE 5 del plan. Cuatro capacidades de "v0-real" sobre el código generado:
//
//   1. `validar_tokens`        — Design token enforcement: detecta valores
//                                hardcodeados (hex/rgb/px) que deberían ser
//                                tokens CSS (var(--primary), etc.).
//   2. `exportar_codesandbox`  — Serializa el árbol de archivos al payload JSON
//                                que CodeSandbox "define" acepta para importar.
//   3. `exportar_stackblitz`   — Payload de proyecto para StackBlitz (files).
//   4. `ErrorDataset`          — Agrega los errores residuales del pipeline a un
//                                dataset de frecuencia, para curar el allowlist
//                                y los prompts del generador.
//
// Todas las funciones son deterministas y sin red, para garantizar tests
// herméticos (misma entrada → misma salida).
// ============================================================================

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use super::contracts::ArchivoGenerado;

/// Tokens CSS que el código generado debe usar en lugar de valores crudos.
const TOKENS_VALIDOS: &[&str] = &[
    "var(--primary)",
    "var(--primary-foreground)",
    "var(--background)",
    "var(--foreground)",
    "var(--border)",
    "var(--radius)",
];

/// Hallazgo de un valor hardcodeado que debería ser un token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HallazgoToken {
    /// Ruta del archivo donde se detectó el valor.
    pub ruta: String,
    /// Valor crudo detectado (ej. `#3B82F6` o `24px`).
    pub valor: String,
    /// Línea aproximada dentro del archivo.
    pub linea: usize,
}

/// Resultado del enforcement de design tokens sobre el código generado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ResultadoTokens {
    /// Hallazgos de valores hardcodeados detectados.
    pub hallazgos: Vec<HallazgoToken>,
    /// `true` si el código usa tokens de forma consistente (sin valores crudos).
    pub limpio: bool,
}

/// Payload de proyecto para CodeSandbox (formato `define` / sandpack).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayloadCodeSandbox {
    /// Mapa de rutas → contenido (API `codesandbox.define`).
    pub files: BTreeMap<String, serde_json::Value>,
    /// Dependencias exigidas por el proyecto (name → version).
    pub dependencies: BTreeMap<String, String>,
    /// Entrada de la aplicación.
    pub entry: String,
}

/// Payload de proyecto para StackBlitz (formato de proyectos).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayloadStackBlitz {
    pub title: String,
    pub files: BTreeMap<String, String>,
    pub template: String,
}

/// Una entrada agregada del dataset de errores frecuentes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorFrecuente {
    /// Código del error (ej. `import_tercero_no_declarado`).
    pub code: String,
    /// Frecuencia acumulada de apariciones.
    pub frecuencia: u32,
    /// Última ruta en la que se observó.
    pub ultima_ruta: String,
}

/// Dataset de errores residuales del pipeline. Su propósito es alimentar la
/// curación del allowlist y de los prompts del generador (FASE 5).
#[derive(Debug, Clone, Default)]
pub struct ErrorDataset {
    /// code → frecuencia agregada.
    por_codigo: HashMap<String, u32>,
    /// code → última ruta observada.
    ultima_ruta_por_codigo: HashMap<String, String>,
    /// total de errores ingeridos.
    total: u64,
}

impl ErrorDataset {
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// Registra un único error (code + ruta) en el dataset.
    pub fn registrar(&mut self, code: &str, ruta: &str) {
        *self.por_codigo.entry(code.to_string()).or_insert(0) += 1;
        self.ultima_ruta_por_codigo
            .insert(code.to_string(), ruta.to_string());
        self.total += 1;
    }

    /// Registra una lista de errores (tipicamente los residuales del pipeline).
    pub fn registrar_muchos<I>(&mut self, errores: I)
    where
        I: IntoIterator<Item = (String, String)>,
    {
        for (code, ruta) in errores {
            self.registrar(&code, &ruta);
        }
    }

    /// Total de errores ingeridos en el dataset.
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Número de códigos distintos agregados.
    pub fn codigos_distintos(&self) -> usize {
        self.por_codigo.len()
    }

    /// Devuelve el dataset ordenado por frecuencia descendente.
    pub fn top(&self) -> Vec<ErrorFrecuente> {
        let mut items: Vec<ErrorFrecuente> = self
            .por_codigo
            .iter()
            .map(|(code, frecuencia)| ErrorFrecuente {
                code: code.clone(),
                frecuencia: *frecuencia,
                ultima_ruta: self
                    .ultima_ruta_por_codigo
                    .get(code)
                    .cloned()
                    .unwrap_or_default(),
            })
            .collect();
        items.sort_by(|a, b| b.frecuencia.cmp(&a.frecuencia));
        items
    }

    /// Devuelve el code más frecuente y su frecuencia, si existe.
    pub fn error_dominante(&self) -> Option<(String, u32)> {
        self.top()
            .first()
            .map(|e| (e.code.clone(), e.frecuencia))
    }
}

/// Escanea el contenido en busca de valores crudos de color y espaciado que
/// deberían expresarse como tokens CSS.
fn hallazgos_en_contenido(ruta: &str, contenido: &str) -> Vec<HallazgoToken> {
    let mut out = Vec::new();
    let tokens_actuales: Vec<&str> = TOKENS_VALIDOS.iter().copied().collect();

    for (idx, linea) in contenido.lines().enumerate() {
        let n_linea = idx + 1;
        // Skip líneas que son solo declaraciones de tokens CSS (`--x: ...`).
        let es_declaracion = linea.trim_start().starts_with("--");

        if es_declaracion {
            continue;
        }

        for valor in &tokens_actuales {
            // No marcar el token en sí mismo.
            if linea.contains(valor) {
                continue;
            }
        }

        detectar_hex(ruta, linea, n_linea, &mut out);
        detectar_px(ruta, linea, n_linea, &mut out);
    }

    out
}

fn detectar_hex(ruta: &str, linea: &str, n_linea: usize, out: &mut Vec<HallazgoToken>) {
    // `#RRGGBB` o `#RGB` en una clase de estilo (dentro de comillas).
    let mut resto = linea;
    while let Some(pos) = resto.find('#') {
        let after = &resto[pos + 1..];
        let mut hex = String::from("#");
        for c in after.chars().take(6) {
            if c.is_ascii_hexdigit() {
                hex.push(c);
            } else {
                break;
            }
        }
        let hex_len = hex.len();
        if hex_len == 4 || hex_len == 7 {
            out.push(HallazgoToken {
                ruta: ruta.to_string(),
                valor: hex,
                linea: n_linea,
            });
            // Avanza tras el valor detectado.
            resto = &after[hex_len - 1..];
        } else {
            resto = after;
        }
    }
}

fn detectar_px(ruta: &str, linea: &str, n_linea: usize, out: &mut Vec<HallazgoToken>) {
    // Detecta `Npx` (ej. `4px`, `24px`) que debería ser spacing token.
    // Ignora la sección CSS de tokens y las clases Tailwind (que usan `p-4`
    // no `padding: 4px`).
    let en_css = linea.contains(':') && (linea.contains("padding") || linea.contains("margin"));
    if !en_css {
        return;
    }
    let mut resto = linea;
    while let Some(pos) = resto.find("px") {
        let before = &resto[..pos];
        let num: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        if !num.is_empty() {
            out.push(HallazgoToken {
                ruta: ruta.to_string(),
                valor: format!("{num}px"),
                linea: n_linea,
            });
        }
        resto = &resto[pos + 2..];
    }
}

/// Enforce de design tokens: escanea todos los archivos generados.
/// Devuelve los valores hardcodeados que deberían ser tokens.
pub fn validar_tokens(archivos: &BTreeMap<String, String>) -> ResultadoTokens {
    let mut hallazgos = Vec::new();
    for (ruta, contenido) in archivos {
        hallazgos.extend(hallazgos_en_contenido(ruta, contenido));
    }
    ResultadoTokens {
        limpio: hallazgos.is_empty(),
        hallazgos,
    }
}

/// Exporta el árbol de archivos al payload JSON de CodeSandbox (`define`).
/// Los archivos JSON (package.json) se serializan como objetos.
pub fn exportar_codesandbox(
    archivos: &[ArchivoGenerado],
    entry: &str,
    package_json: &serde_json::Map<String, serde_json::Value>,
) -> PayloadCodeSandbox {
    let mut files = BTreeMap::new();
    for a in archivos {
        let contenido = a.content.clone();
        let valor = if a.path == "package.json" {
            // Usa el package_json estructurado si coincide la ruta.
            serde_json::Value::Object(package_json.clone())
        } else if a.path.ends_with(".json") {
            serde_json::from_str(&contenido)
                .unwrap_or_else(|_| serde_json::Value::String(contenido))
        } else {
            serde_json::Value::String(contenido)
        };
        files.insert(a.path.clone(), valor);
    }

    // Dependencias para el sandbox: se extraen del package_json.
    let mut dependencies = BTreeMap::new();
    if let Some(deps) = package_json.get("dependencies").and_then(|d| d.as_object()) {
        for (name, ver) in deps {
            let version = ver
                .as_str()
                .map(|s| s.trim_start_matches('^').trim_start_matches('~').to_string())
                .unwrap_or_else(|| "latest".to_string());
            dependencies.insert(name.clone(), version);
        }
    }

    PayloadCodeSandbox {
        files,
        dependencies,
        entry: entry.to_string(),
    }
}

/// Exporta el árbol de archivos al payload de proyecto de StackBlitz.
pub fn exportar_stackblitz(
    archivos: &[ArchivoGenerado],
    title: &str,
) -> PayloadStackBlitz {
    let mut files = BTreeMap::new();
    for a in archivos {
        files.insert(a.path.clone(), a.content.clone());
    }
    PayloadStackBlitz {
        title: title.to_string(),
        files,
        template: "node".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::v0::contracts::GeneracionUI;

    fn generacion_ejemplo() -> GeneracionUI {
        let mut mapa = BTreeMap::new();
        mapa.insert(
            "src/App.tsx".to_string(),
            "className='bg-white text-black p-4'".to_string(),
        );
        mapa.insert(
            "src/index.css".to_string(),
            ":root { --primary: 221 83% 53%; } body { color: #111827; }\n".to_string(),
        );
        // GeneracionUI: solo necesitamos files para exportar.
        let files: Vec<ArchivoGenerado> = mapa
            .iter()
            .map(|(path, content)| ArchivoGenerado {
                path: path.clone(),
                content: content.clone(),
                language: if path.ends_with("css") { "css" } else { "tsx" }.to_string(),
            })
            .collect();
        GeneracionUI {
            schema: "v0/generate".to_string(),
            plan_id: "plan-1".to_string(),
            files,
            package_json: serde_json::json!({
                "name": "app",
                "dependencies": { "react": "^18.3.1", "lucide-react": "^0.400.0" }
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
            entry_point: "src/App.tsx".to_string(),
        }
    }

    fn mapa_de(gen: &GeneracionUI) -> BTreeMap<String, String> {
        gen.files
            .iter()
            .map(|a| (a.path.clone(), a.content.clone()))
            .collect()
    }

    #[test]
    fn test_validar_tokens_detecta_hex_hardcodeado() {
        let gen = generacion_ejemplo();
        let res = validar_tokens(&mapa_de(&gen));
        assert!(!res.limpio);
        assert!(res.hallazgos.iter().any(|h| h.valor == "#111827"));
        // El token declarado en :root no se marca.
        assert!(!res.hallazgos.iter().any(|h| h.valor == "221 83% 53%"));
    }

    #[test]
    fn test_validar_tokens_px_en_css_detectado() {
        let mut mapa = BTreeMap::new();
        mapa.insert(
            "src/App.tsx".to_string(),
            "<div style={{ padding: '24px' }} />".to_string(),
        );
        let res = validar_tokens(&mapa);
        assert!(!res.limpio);
        assert!(res.hallazgos.iter().any(|h| h.valor == "24px"));
    }

    #[test]
    fn test_css_con_tokens_limpio() {
        let mut mapa = BTreeMap::new();
        mapa.insert(
            "src/index.css".to_string(),
            ":root { --primary: 221 83% 53%; --background: 0 0% 100%; }\n".to_string(),
        );
        let res = validar_tokens(&mapa);
        assert!(res.limpio, "hallazgos={:?}", res.hallazgos);
    }

    #[test]
    fn test_exportar_codesandbox_mapa_y_deps() {
        let gen = generacion_ejemplo();
        let payload = exportar_codesandbox(&gen.files, &gen.entry_point, &gen.package_json);
        assert!(payload.files.contains_key("src/App.tsx"));
        assert_eq!(payload.entry, "src/App.tsx");
        assert_eq!(payload.dependencies.get("react").map(|s| s.as_str()), Some("18.3.1"));
        assert!(payload.files.contains_key("src/index.css"));
    }

    #[test]
    fn test_exportar_stackblitz_payload() {
        let gen = generacion_ejemplo();
        let payload = exportar_stackblitz(&gen.files, "Mi App");
        assert_eq!(payload.title, "Mi App");
        assert_eq!(payload.template, "node");
        assert_eq!(payload.files.len(), gen.files.len());
    }

    #[test]
    fn test_error_dataset_acumula_frecuencia() {
        let mut ds = ErrorDataset::nuevo();
        ds.registrar("import_tercero_no_declarado", "src/App.tsx");
        ds.registrar("import_tercero_no_declarado", "src/components/ui/x.tsx");
        ds.registrar("llaves_desbalanceadas", "src/App.tsx");
        assert_eq!(ds.total(), 3);
        assert_eq!(ds.codigos_distintos(), 2);
        let (dominante, freq) = ds.error_dominante().unwrap();
        assert_eq!(dominante, "import_tercero_no_declarado");
        assert_eq!(freq, 2);
        let top = ds.top();
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].code, "import_tercero_no_declarado");
    }

    #[test]
    fn test_error_dataset_vacio() {
        let ds = ErrorDataset::nuevo();
        assert_eq!(ds.total(), 0);
        assert_eq!(ds.codigos_distintos(), 0);
        assert!(ds.error_dominante().is_none());
        assert!(ds.top().is_empty());
    }

    #[test]
    fn test_error_dataset_registrar_muchos() {
        let mut ds = ErrorDataset::nuevo();
        let errores = vec![
            ("low_contrast".to_string(), "src/App.tsx".to_string()),
            ("overflow_potencial".to_string(), "src/App.tsx".to_string()),
        ];
        ds.registrar_muchos(errores);
        assert_eq!(ds.total(), 2);
    }
}
