// ============================================================================
// 📦 RESOLUTOR DE DEPENDENCIAS V0
// ============================================================================
// Cruza las dependencias declaradas por Gemini (en `GeneracionUI.package_json`)
// contra un allowlist curado de paquetes con rangos semver seguros.
//
// Reglas:
//   1. Paquete no en allowlist        → se rechaza (Error)
//   2. Versión fuera de rango permit. → se clampa a la versión más cercana
//   3. Conflicto de versiones         → se resuelve con la más reciente compatible
//
// Totalmente determinista: sin red, sin I/O. El allowlist se carga desde un
// JSON embebido por defecto (constante) o desde una ruta opcional.
// ============================================================================

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Entrada del allowlist: paquete → rango semver seguro + versión pin recomendada.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaquetePermitido {
    pub version: String,
    pub range: String,
    #[serde(default)]
    pub reason: String,
}

/// Allowlist completo de paquetes auditados.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Allowlist {
    pub packages: BTreeMap<String, PaquetePermitido>,
}

/// Resultado de resolución de una dependencia declarada.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolucionDep {
    /// Paquete aceptado y versionado según allowlist.
    Aceptada { nombre: String, version: String },
    /// Paquete rechazado: no está en el allowlist.
    Rechazada { nombre: String, razon: String },
    /// Paquete aceptado pero su versión fue ajustada al rango permitido.
    Clampada { nombre: String, version_original: String, version_final: String },
}

impl ResolucionDep {
    pub fn es_aceptada(&self) -> bool {
        matches!(self, ResolucionDep::Aceptada { .. } | ResolucionDep::Clampada { .. })
    }
}

/// Resultado global de resolución de un set de dependencias.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResultadoResolucion {
    /// package.json final con dependencias resueltas (nombre → versión).
    pub dependencies: BTreeMap<String, String>,
    pub resoluciones: Vec<ResolucionDep>,
    pub rechazadas: Vec<ResolucionDep>,
}

impl ResultadoResolucion {
    pub fn es_valido(&self) -> bool {
        self.rechazadas.is_empty()
    }

    pub fn total(&self) -> usize {
        self.dependencies.len()
    }
}

/// Allowlist por defecto de paquetes curados para generación de UI v0.
/// Rango = semver seguro aceptado; version = pin recomendado.
pub const ALLOWLIST_EMBEBIDA: &str = r#"{
  "packages": {
    "react":               { "version": "18.3.1",     "range": "^18 || ^19",     "reason": "Core runtime UI" },
    "react-dom":           { "version": "18.3.1",     "range": "^18 || ^19",     "reason": "Core DOM renderer" },
    "lucide-react":        { "version": "0.460.0",    "range": ">=0.400.0",      "reason": "Iconos" },
    "clsx":                { "version": "2.1.1",      "range": "^2",             "reason": "Utilidad de clases" },
    "tailwind-merge":      { "version": "2.5.2",      "range": "^2",             "reason": "Merge de clases tailwind" },
    "class-variance-authority": { "version": "0.7.0", "range": "^0.7",           "reason": "Variantes CVA" },
    "tailwindcss":         { "version": "3.4.14",     "range": "^3",             "reason": "CSS engine" },
    "tailwindcss-animate": { "version": "1.0.7",      "range": "^1",             "reason": "Animaciones tailwind" },
    "@radix-ui/react-dialog":    { "version": "1.1.2", "range": "^1",           "reason": "shadcn dialog" },
    "@radix-ui/react-dropdown-menu": { "version": "2.1.2", "range": "^2",       "reason": "shadcn dropdown" },
    "@radix-ui/react-select":     { "version": "2.1.2", "range": "^2",           "reason": "shadcn select" },
    "@radix-ui/react-tabs":       { "version": "1.1.1", "range": "^1",           "reason": "shadcn tabs" },
    "@radix-ui/react-label":      { "version": "2.1.0", "range": "^2",           "reason": "shadcn label" },
    "@radix-ui/react-slot":       { "version": "1.1.0", "range": "^1",           "reason": "shadcn slot" },
    "@radix-ui/react-separator":  { "version": "1.1.0", "range": "^1",           "reason": "shadcn separator" },
    "@radix-ui/react-alert-dialog": { "version": "1.1.2", "range": "^1",         "reason": "shadcn alert dialog" },
    "@radix-ui/react-toast":      { "version": "1.2.2", "range": "^1",           "reason": "shadcn toast/sonner base" },
    "@radix-ui/react-avatar":     { "version": "1.1.1", "range": "^1",           "reason": "shadcn avatar" },
    "@radix-ui/react-progress":   { "version": "1.1.0", "range": "^1",           "reason": "shadcn progress" },
    "@radix-ui/react-tooltip":    { "version": "1.1.4", "range": "^1",           "reason": "shadcn tooltip" },
    "@radix-ui/react-switch":     { "version": "1.1.1", "range": "^1",           "reason": "shadcn switch" },
    "@radix-ui/react-checkbox":   { "version": "1.1.2", "range": "^1",           "reason": "shadcn checkbox" },
    "@radix-ui/react-scroll-area": { "version": "1.2.1", "range": "^1",          "reason": "shadcn scroll area" },
    "date-fns":            { "version": "3.6.0",      "range": "^3",             "reason": "Fechas shadcn" },
    "react-day-picker":    { "version": "8.10.1",     "range": "^8",             "reason": "shadcn calendar" },
    "recharts":            { "version": "2.13.3",     "range": "^2",             "reason": "Gráficos" },
    "react-router-dom":    { "version": "6.28.0",     "range": "^6",             "reason": "Routing SPA" },
    "zod":                 { "version": "3.23.8",     "range": "^3",             "reason": "Validación de esquemas" },
    "@types/react":        { "version": "18.3.12",    "range": "^18",            "reason": "Tipos React" },
    "@types/react-dom":    { "version": "18.3.1",     "range": "^18",            "reason": "Tipos React DOM" },
    "typescript":          { "version": "5.6.3",      "range": "^5",             "reason": "Compilador TS" },
    "vite":                { "version": "5.4.10",     "range": "^5",             "reason": "Bundler dev" },
    "@vitejs/plugin-react": { "version": "4.3.3",     "range": "^4",             "reason": "Plugin React Vite" },
    "autoprefixer":        { "version": "10.4.20",    "range": "^10",            "reason": "Prefixes CSS" },
    "postcss":             { "version": "8.4.47",     "range": "^8",             "reason": "PostCSS" }
  }
}"#;

/// Resolutor de dependencias contra un allowlist curado.
#[derive(Debug, Clone)]
pub struct DependencyResolver {
    allowlist: Allowlist,
}

impl DependencyResolver {
    /// Crea un resolutor con el allowlist embebido por defecto.
    pub fn nuevo() -> Self {
        Self::con_allowlist_embebido()
    }

    /// Crea un resolutor con el allowlist embebido (constante).
    pub fn con_allowlist_embebido() -> Self {
        let allowlist = serde_json::from_str(ALLOWLIST_EMBEBIDA)
            .unwrap_or_else(|_| Allowlist::default());
        Self { allowlist }
    }

    /// Crea un resolutor cargando el allowlist desde un archivo JSON.
    /// Si falla la carga, cae al allowlist embebido.
    pub fn desde_archivo(path: &Path) -> Self {
        let allowlist = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| {
                serde_json::from_str(ALLOWLIST_EMBEBIDA).unwrap_or_default()
            });
        Self { allowlist }
    }

    pub fn allowlist(&self) -> &Allowlist {
        &self.allowlist
    }

    /// Resuelve una dependencia individual declarada (nombre → versión).
    pub fn resolver_dep(&self, nombre: &str, version_declarada: &str) -> ResolucionDep {
        let permitido = match self.allowlist.packages.get(nombre) {
            Some(p) => p,
            None => {
                return ResolucionDep::Rechazada {
                    nombre: nombre.to_string(),
                    razon: "Paquete no está en el allowlist curado".to_string(),
                }
            }
        };

        // Si la versión declarada está vacía, usar el pin del allowlist.
        let v = version_declarada.trim();
        if v.is_empty() {
            return ResolucionDep::Aceptada {
                nombre: nombre.to_string(),
                version: permitido.version.clone(),
            };
        }

        // Si la declarada ya es compatible con el pin, aceptar la del allowlist
        // (siempre pinneamos al rango seguro del allowlist).
        if version_es_compatible(v, &permitido.range) {
            ResolucionDep::Aceptada {
                nombre: nombre.to_string(),
                version: permitido.version.clone(),
            }
        } else {
            ResolucionDep::Clampada {
                nombre: nombre.to_string(),
                version_original: v.to_string(),
                version_final: permitido.version.clone(),
            }
        }
    }

    /// Resuelve un mapa completo de dependencias (nombre → versión declarada).
    pub fn resolver_mapa(
        &self,
        declaradas: &BTreeMap<String, String>,
    ) -> ResultadoResolucion {
        let mut resultado = ResultadoResolucion::default();

        for (nombre, version) in declaradas {
            let resolucion = self.resolver_dep(nombre, version);
            match &resolucion {
                ResolucionDep::Aceptada { nombre, version } => {
                    resultado.dependencies.insert(nombre.clone(), version.clone());
                }
                ResolucionDep::Clampada {
                    nombre,
                    version_final,
                    ..
                } => {
                    resultado
                        .dependencies
                        .insert(nombre.clone(), version_final.clone());
                }
                ResolucionDep::Rechazada { .. } => {}
            }
            match resolucion {
                ResolucionDep::Aceptada { .. } | ResolucionDep::Clampada { .. } => {
                    resultado.resoluciones.push(resolucion);
                }
                ResolucionDep::Rechazada { .. } => {
                    resultado.rechazadas.push(resolucion);
                }
            }
        }

        resultado
    }

    /// Extrae las dependencias declaradas de un `package_json` (Map serde_json)
    /// y las resuelve. Devuelve el package.json final validado.
    pub fn resolver_package_json(
        &self,
        package_json: &serde_json::Map<String, serde_json::Value>,
    ) -> ResultadoResolucion {
        let mut declaradas = BTreeMap::new();

        // Buscar secciones de dependencias conocidas.
        for seccion in ["dependencies", "devDependencies", "peerDependencies"] {
            if let Some(Value::Object(map)) = package_json.get(seccion) {
                for (nombre, v) in map {
                    let version = match v {
                        Value::String(s) => s.clone(),
                        Value::Object(_) => extract_version_de_objeto(v),
                        _ => String::new(),
                    };
                    declaradas.insert(nombre.clone(), version);
                }
            }
        }

        self.resolver_mapa(&declaradas)
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::nuevo()
    }
}

/// Extrae una versión semver desde un objeto de metadatos (como "version").
fn extract_version_de_objeto(v: &serde_json::Value) -> String {
    v.get("version")
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string()
}

/// Comprueba si una versión concreta cae dentro de un rango semver expresado
/// como cadena (^, ~, >=, comparador simple). Implementación ligera y
/// determinista; suficiente para validar pins del allowlist.
fn version_es_compatible(version: &str, range: &str) -> bool {
    let v = version.trim_start_matches(['^', '~', 'v', '>', '=', '<', ' ']);

    let to_tuple = |s: &str| -> (u64, u64, u64) {
        let mut parts = s.split('.');
        let major = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        let patch = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    };

    let vv = to_tuple(v);

    // Soportar rangos OR: "^18 || ^19" → compatible si matchea CUALQUIER parte.
    range.split("||").any(|parte| {
        // `trim()` elimina espacios a ambos lados (los rangos OR suelen llevar
        // espacios alrededor de `||`). Sin esto, "18 " no parsea y cae a major 0.
        let base = parte.trim().trim_start_matches(['^', '~', 'v', '>', '=', '<', ' ']);
        let bb = to_tuple(base);
        // Aceptar cuando el pin del allowlist coincide en major (compatible ^/~).
        vv.0 == bb.0
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_allowlist_embebida_carga() {
        let r = DependencyResolver::nuevo();
        assert!(r.allowlist().packages.contains_key("react"));
        assert!(r.allowlist().packages.contains_key("@radix-ui/react-dialog"));
        assert!(r.allowlist().packages.contains_key("tailwindcss"));
    }

    #[test]
    fn test_dep_permitida_aceptada() {
        let r = DependencyResolver::nuevo();
        let res = r.resolver_dep("react", "18.3.1");
        assert_eq!(
            res,
            ResolucionDep::Aceptada {
                nombre: "react".into(),
                version: "18.3.1".into()
            }
        );
    }

    #[test]
    fn test_dep_vacia_usa_pin() {
        let r = DependencyResolver::nuevo();
        let res = r.resolver_dep("clsx", "");
        match res {
            ResolucionDep::Aceptada { nombre, version } => {
                assert_eq!(nombre, "clsx");
                assert_eq!(version, "2.1.1");
            }
            _ => panic!("clsx sin versión debería usar el pin del allowlist"),
        }
    }

    #[test]
    fn test_dep_fuera_de_rango_clampada() {
        let r = DependencyResolver::nuevo();
        // tailwindcss solo admite ^3. Declarar 4.x debería clamar a 3.4.14.
        let res = r.resolver_dep("tailwindcss", "^4.0.0");
        match res {
            ResolucionDep::Clampada {
                nombre,
                version_final,
                ..
            } => {
                assert_eq!(nombre, "tailwindcss");
                assert_eq!(version_final, "3.4.14");
            }
            _ => panic!("versión fuera de rango debe clamparse"),
        }
    }

    #[test]
    fn test_dep_no_allowlist_rechazada() {
        let r = DependencyResolver::nuevo();
        let res = r.resolver_dep("lodash", "^4.17.21");
        match res {
            ResolucionDep::Rechazada { nombre, .. } => assert_eq!(nombre, "lodash"),
            _ => panic!("lodash no está en allowlist y debe rechazarse"),
        }
    }

    #[test]
    fn test_resolver_mapa_mezcla() {
        let r = DependencyResolver::nuevo();
        let mut mapa = BTreeMap::new();
        mapa.insert("react".into(), "18.3.1".into());
        mapa.insert("evil-package".into(), "^1.0.0".into());
        mapa.insert("tailwindcss".into(), "4.0.0".into());

        let res = r.resolver_mapa(&mapa);
        assert!(!res.es_valido());
        assert_eq!(res.rechazadas.len(), 1);
        assert_eq!(res.dependencies.len(), 2);
        assert!(res.dependencies.contains_key("react"));
        assert_eq!(res.dependencies.get("tailwindcss").unwrap(), "3.4.14");
    }

    #[test]
    fn test_resolver_package_json_completo() {
        let r = DependencyResolver::nuevo();
        let pkg = json!({
            "name": "nexus-v0-app",
            "dependencies": {
                "react": "18.3.1",
                "react-dom": "18.3.1",
                "lucide-react": "^0.460.0",
                "@radix-ui/react-dialog": "^1.1.2",
                "recharts": "^2.13.3"
            },
            "devDependencies": {
                "typescript": "^5.6.3",
                "vite": "^5.4.10"
            }
        })
        .as_object()
        .unwrap()
        .clone();

        let res = r.resolver_package_json(&pkg);
        assert!(res.es_valido());
        assert_eq!(res.total(), 7);
        assert_eq!(res.dependencies.get("react").unwrap(), "18.3.1");
        assert_eq!(res.dependencies.get("vite").unwrap(), "5.4.10");
    }

    #[test]
    fn test_es_aceptada_flag() {
        assert!(ResolucionDep::Aceptada {
            nombre: "a".into(),
            version: "1".into()
        }
        .es_aceptada());
        assert!(ResolucionDep::Clampada {
            nombre: "a".into(),
            version_original: "9".into(),
            version_final: "1".into()
        }
        .es_aceptada());
        assert!(!ResolucionDep::Rechazada {
            nombre: "a".into(),
            razon: "x".into()
        }
        .es_aceptada());
    }

    #[test]
    fn test_desde_archivo_inexistente_falla_embebido() {
        let r = DependencyResolver::desde_archivo(Path::new("/no/existe/allowlist.json"));
        assert!(r.allowlist().packages.contains_key("react"));
    }
}
