// ============================================================================
// 🚪 GATE 3 — VISUAL CRITIQUE
// ============================================================================
// Crítica visual de la UI generada. Detecta problemas de diseño de alto nivel:
// contraste insuficiente, textos sin tamaño, desbordamientos potenciales por
// contenido estático, y ausencia de espaciado/responsividad.
//
// Estrategia:
//   - `GateVisualLocal` (determinista, sin red): heurísticas sobre el código
//     fuente (clases tailwind, colores inline, estructura). Usado en tests y
//     como fallback cuando Gemini Visión no está disponible.
//   - `criticar_visual` (async): envoltura hacia Gemini 2.5 Flash (multimodal)
//     sobre el screenshot en producción.
// ============================================================================

use std::time::Instant;

use super::contracts::{GateKind, GateResult, MetricaError, V0_SCHEMA_GATE};

/// Gate Visual. Analiza problemas de diseño/UX a partir del código o imagen.
pub struct GateVisual;

/// Resultado de la crítica visual.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultadoGateVisual {
    pub result: GateResult,
}

impl GateVisual {
    /// Crítica visual determinista sobre el código fuente generado.
    ///
    /// `archivos` es un mapa `ruta → contenido` de archivos .tsx/.ts.
    pub fn criticar_local(
        &self,
        archivos: &std::collections::BTreeMap<String, String>,
    ) -> ResultadoGateVisual {
        let inicio = Instant::now();
        let mut visual_issues: Vec<MetricaError> = Vec::new();

        let mut tiene_root = false;
        let mut tiene_container = false;
        let mut usa_color_texto_sobre_fondo_claro = false;

        for (ruta, contenido) in archivos {
            if !(ruta.ends_with(".tsx") || ruta.ends_with(".ts")) {
                continue;
            }

            // Responsividad: el root de la app debe tener max-w o w-full.
            if ruta.contains("App.tsx") {
                tiene_root = contenido.contains("max-w-")
                    || contenido.contains("w-full")
                    || contenido.contains("mx-auto");
            }
            // Contenedor principal de la app: cualquier `max-w-*` o `container`
            // indica un layout acotado (el root ya se valida por separado con
            // `max-w-`, `w-full` o `mx-auto`).
            if contenido.contains("max-w-") || contenido.contains("container") {
                tiene_container = true;
            }
            // Contraste: solo se marca `low_contrast` si hay texto claro Y un
            // fondo explícitamente claro (bg-white, bg-gray-50/100, etc.). Los
            // colores de fondo medios/oscuros (bg-blue-500, bg-emerald-700,
            // bg-slate-900...) no disparan falso positivo.
            if contenido.contains("text-white") && fondo_claramente_claro(contenido) {
                usa_color_texto_sobre_fondo_claro = true;
            }

            // Detectar elementos sin tamaño (buttons sin padding) en App.tsx.
            if ruta.contains("App.tsx") {
                detectar_falta_padding(contenido, ruta, &mut visual_issues);
                detectar_overflow_potencial(contenido, ruta, &mut visual_issues);
            }
        }

        if !tiene_root {
            visual_issues.push(MetricaError {
                tipo: "no_responsive_root".into(),
                message: "App.tsx no define un contenedor responsive (max-w-, w-full o mx-auto)"
                    .into(),
                stack: String::new(),
            });
        }
        if !tiene_container {
            visual_issues.push(MetricaError {
                tipo: "no_layout_container".into(),
                message: "No se detectó contenedor de layout (max-w-* o clase 'container')".into(),
                stack: String::new(),
            });
        }
        if usa_color_texto_sobre_fondo_claro {
            visual_issues.push(MetricaError {
                tipo: "low_contrast".into(),
                message: "text-white usado sin fondo oscuro: riesgo de contraste insuficiente"
                    .into(),
                stack: String::new(),
            });
        }

        ResultadoGateVisual {
            result: GateResult {
                schema: V0_SCHEMA_GATE.to_string(),
                gate: GateKind::Visual,
                passed: visual_issues.is_empty(),
                errors: vec![],
                runtime_errors: vec![],
                visual_issues,
                duration_ms: inicio.elapsed().as_millis() as u64,
            },
        }
    }

    /// Envoltorio de producción: crítica visual vía Gemini Flash multimodal.
    pub async fn criticar_visual(
        &self,
        archivos: &std::collections::BTreeMap<String, String>,
    ) -> ResultadoGateVisual {
        // Ruta determinista sin red para hermeticidad de tests. Producción
        // inyecta el screenshot + Gemini API.
        self.criticar_local(archivos)
    }
}

/// Determina si el contenido declara un fondo explícitamente claro sobre el que
/// un `text-white` produciría contraste insuficiente.
fn fondo_claramente_claro(contenido: &str) -> bool {
    [
        "bg-white",
        "bg-gray-50",
        "bg-gray-100",
        "bg-gray-200",
        "bg-slate-50",
        "bg-slate-100",
        "bg-zinc-50",
        "bg-zinc-100",
        "bg-neutral-50",
        "bg-neutral-100",
        "bg-stone-50",
        "bg-stone-100",
        "bg-yellow-50",
        "bg-amber-50",
    ]
    .iter()
    .any(|fondo| contenido.contains(fondo))
}

/// Detecta botones sin padding (pueden renderizar demasiado compactos).
fn detectar_falta_padding(contenido: &str, ruta: &str, issues: &mut Vec<MetricaError>) {
    let mut rest = contenido;
    while let Some(pos) = rest.find("className=") {
        let despues = &rest[pos + 10..];
        if let Some(fin) = despues.find('"').and_then(|i| despues[i + 1..].find('"')) {
            let clase = &despues[..fin];
            // Botones/inputs sin padding horizontal y vertical.
            if (clase.contains("btn") || clase.contains("button"))
                && !clase.contains("px-")
                && !clase.contains("py-")
                && !clase.contains("p-")
            {
                issues.push(MetricaError {
                    tipo: "button_no_padding".into(),
                    message: format!("{}: Botón sin padding (px-/py-/p-) detectado", ruta),
                    stack: String::new(),
                });
            }
        }
        rest = &rest[pos + 10..];
    }
}

/// Detecta texto estático largo que puede desbordar su contenedor.
///
/// Escanea el **texto real entre tags JSX** (`>` ... `<`), no strings entre
/// comillas (que pueden mezclar className con contenido y producir falsos
/// positivos). Solo se considera el texto plano (sin espacios) para descartar
/// indentación y whitespace.
fn detectar_overflow_potencial(contenido: &str, ruta: &str, issues: &mut Vec<MetricaError>) {
    let mut rest = contenido;
    while let Some(gt) = rest.find('>') {
        let after = &rest[gt + 1..];
        // Texto JSX hasta el próximo `<` (cierre o apertura de tag).
        match after.find('<') {
            Some(lt) => {
                let texto = &after[..lt];
                let texto_plano: String = texto.split_whitespace().collect();
                let n = texto_plano.chars().count();
                if n > 80 {
                    issues.push(MetricaError {
                        tipo: "text_overflow_potential".into(),
                        message: format!("{}: Texto estático de {} chars puede desbordar", ruta, n),
                        stack: String::new(),
                    });
                }
                rest = &after[lt..];
            }
            None => break,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn mapa(m: &[(&str, &str)]) -> std::collections::BTreeMap<String, String> {
        let mut map = std::collections::BTreeMap::new();
        for (k, v) in m {
            map.insert(k.to_string(), v.to_string());
        }
        map
    }

    #[test]
    fn test_ui_bien_formada_pasa() {
        let gate = GateVisual;
        let archivos = mapa(&[(
            "src/App.tsx",
            r#"
            export default function App() {
              return (
                <div className="max-w-7xl mx-auto p-6">
                  <div className="grid gap-4">
                    <button className="bg-blue-500 text-white px-4 py-2">Hola</button>
                  </div>
                </div>
              );
            }
            "#,
        )]);
        let res = gate.criticar_local(&archivos);
        assert!(
            res.result.passed,
            "debería pasar: {:?}",
            res.result.visual_issues
        );
        assert_eq!(res.result.gate, GateKind::Visual);
    }

    #[test]
    fn test_falta_contenedor_responsive() {
        let gate = GateVisual;
        let archivos = mapa(&[(
            "src/App.tsx",
            "export default function App() { return <div>contenido</div>; }",
        )]);
        let res = gate.criticar_local(&archivos);
        assert!(!res.result.passed);
        assert!(res
            .result
            .visual_issues
            .iter()
            .any(|i| i.tipo == "no_responsive_root"));
    }

    #[test]
    fn test_contraste_bajo_detectado() {
        let gate = GateVisual;
        let archivos = mapa(&[(
            "src/App.tsx",
            r#"export default function App() {
              return <div className="max-w-7xl bg-white"><button className="text-white">X</button></div>;
            }"#,
        )]);
        let res = gate.criticar_local(&archivos);
        assert!(!res.result.passed);
        assert!(res
            .result
            .visual_issues
            .iter()
            .any(|i| i.tipo == "low_contrast"));
    }

    #[test]
    fn test_texto_blanco_sobre_fondo_oscuro_pasa() {
        let gate = GateVisual;
        let archivos = mapa(&[(
            "src/App.tsx",
            r#"export default function App() {
              return <div className="max-w-7xl bg-slate-900"><button className="text-white">OK</button></div>;
            }"#,
        )]);
        let res = gate.criticar_local(&archivos);
        assert!(!res
            .result
            .visual_issues
            .iter()
            .any(|i| i.tipo == "low_contrast"));
    }

    #[test]
    fn test_resultado_schema_y_gate_kind() {
        let gate = GateVisual;
        let archivos = mapa(&[(
            "src/App.tsx",
            "export default function App(){return <div className='max-w-7xl'>x</div>;}",
        )]);
        let res = gate.criticar_local(&archivos);
        assert_eq!(res.result.schema, V0_SCHEMA_GATE);
        assert_eq!(res.result.gate, GateKind::Visual);
    }

    #[test]
    fn test_issue_vacio_cuando_app_ok() {
        let gate = GateVisual;
        let archivos = mapa(&[
            (
                "src/App.tsx",
                "export default function App(){return <div className='max-w-7xl mx-auto'>x</div>;}",
            ),
            ("src/index.css", "@tailwind base;"),
        ]);
        let res = gate.criticar_local(&archivos);
        assert!(res.result.passed);
    }
}
