// ==========================================
// 🧭 MODEL ROUTER — Selección de modelo LLM por intención
// ==========================================
// Analiza el prompt del Arquitecto y selecciona el modelo Ollama más
// adecuado según la intención:
//
//   SEGURIDAD      → whiterabbitneo-off:latest   (ofensivo/defensivo sin restricción)
//   CÓDIGO         → nexuslocal:latest           (Qwen 2.5 con identidad NEXUS)
//   RAZONAMIENTO   → deepseek-r1:7b              (lógica, matemática, análisis)
//   CREATIVO       → llama3.1-8b-abliterated:latest (generación sin filtro)
//   GENERAL        → gemma3:4b                    (default — rápido y a la par en calidad)
//
// Patrón inspirado en IntentRouter (comms/intent_router.rs): detección por
// palabra clave de dominio con prioridad de coincidencia.
// ==========================================

use serde::{Deserialize, Serialize};

/// Intenciones de modelo reconocidas por el router.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntencionModelo {
    /// Seguridad ofensiva/defensiva, pentesting, explotación.
    Seguridad,
    /// Desarrollo, refactor, compilación, depuración.
    Codigo,
    /// Análisis lógico, matemática, algoritmos.
    Razonamiento,
    /// Generación creativa, narrativa, arte.
    Creativo,
    /// Sin intención específica — modelo por defecto.
    General,
}

impl IntencionModelo {
    /// Modelo Ollama asociado a la intención.
    pub fn modelo_ollama(&self) -> &'static str {
        match self {
            Self::Seguridad => "whiterabbitneo-off:latest",
            Self::Codigo => "nexuslocal:latest",
            Self::Razonamiento => "deepseek-r1:7b",
            Self::Creativo => "llama3.1-8b-abliterated:latest",
            Self::General => "gemma3:4b",
        }
    }

    /// Etiqueta legible para telemetría/ledger.
    pub fn etiqueta(&self) -> &'static str {
        match self {
            Self::Seguridad => "SEGURIDAD",
            Self::Codigo => "CODIGO",
            Self::Razonamiento => "RAZONAMIENTO",
            Self::Creativo => "CREATIVO",
            Self::General => "GENERAL",
        }
    }

    /// Todas las intenciones en orden estable (fuente única de verdad para
    /// herramientas MCP que listan los modelos disponibles).
    pub fn todas() -> [IntencionModelo; 5] {
        [
            Self::Seguridad,
            Self::Codigo,
            Self::Razonamiento,
            Self::Creativo,
            Self::General,
        ]
    }
}

/// Router de modelos: clasifica intenciones y devuelve el modelo óptimo.
pub struct ModelRouter {
    /// Pares (keyword, intención) con prioridad de inserción.
    keywords: Vec<(&'static str, IntencionModelo)>,
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRouter {
    pub fn new() -> Self {
        use IntencionModelo::*;

        // Orden de prioridad importa: se devuelve la PRIMERA coincidencia.
        let keywords: Vec<(&'static str, IntencionModelo)> = vec![
            // ── SEGURIDAD (dominio ofensivo/defensivo) ──────────────
            ("pentest", Seguridad),
            ("pentesting", Seguridad),
            ("exploit", Seguridad),
            ("explotación", Seguridad),
            ("payload", Seguridad),
            ("sql injection", Seguridad),
            ("inyección sql", Seguridad),
            ("inyeccion sql", Seguridad),
            ("inyección", Seguridad),
            ("inyeccion", Seguridad),
            ("xss", Seguridad),
            ("csrf", Seguridad),
            ("lfi", Seguridad),
            ("rfi", Seguridad),
            ("ssrf", Seguridad),
            ("rce", Seguridad),
            ("deserialización", Seguridad),
            ("deserializacion", Seguridad),
            ("reverse shell", Seguridad),
            ("shell inversa", Seguridad),
            ("escalada de privilegios", Seguridad),
            ("privilege escalation", Seguridad),
            ("fuerza bruta", Seguridad),
            ("brute force", Seguridad),
            ("crackear", Seguridad),
            ("crack", Seguridad),
            ("hash", Seguridad),
            ("metasploit", Seguridad),
            ("nmap", Seguridad),
            ("reconocimiento", Seguridad),
            ("recon", Seguridad),
            ("enumeración", Seguridad),
            ("enumeracion", Seguridad),
            ("vulnerabilidad", Seguridad),
            ("vulnerabilidad", Seguridad),
            ("bypass", Seguridad),
            ("ofuscación", Seguridad),
            ("ofuscacion", Seguridad),
            ("exfiltración", Seguridad),
            ("exfiltracion", Seguridad),
            ("hackear", Seguridad),
            ("hack", Seguridad),
            ("seguridad", Seguridad),
            ("firewall", Seguridad),
            ("wireshark", Seguridad),
            ("burp", Seguridad),
            ("exploit-db", Seguridad),
            ("cve-", Seguridad),
            // ── RAZONAMIENTO (lógica pura antes que código) ─────────
            ("analiza", Razonamiento),
            ("análisis", Razonamiento),
            ("analisis", Razonamiento),
            ("calcula", Razonamiento),
            ("razona", Razonamiento),
            ("razonamiento", Razonamiento),
            ("lógica", Razonamiento),
            ("logica", Razonamiento),
            ("demuestra", Razonamiento),
            ("prueba matemática", Razonamiento),
            ("matemática", Razonamiento),
            ("matematica", Razonamiento),
            ("algoritmo", Razonamiento),
            ("complejidad", Razonamiento),
            ("big-o", Razonamiento),
            ("probabilidad", Razonamiento),
            ("estadística", Razonamiento),
            ("estadistica", Razonamiento),
            // ── CÓDIGO (desarrollo) ─────────────────────────────────
            ("implementa", Codigo),
            ("implementar", Codigo),
            ("programa", Codigo),
            ("refactoriza", Codigo),
            ("refactor", Codigo),
            ("compila", Codigo),
            ("cargo build", Codigo),
            ("cargo run", Codigo),
            ("función", Codigo),
            ("funcion", Codigo),
            ("api", Codigo),
            ("bug", Codigo),
            ("debug", Codigo),
            ("depura", Codigo),
            ("test", Codigo),
            ("typescript", Codigo),
            ("javascript", Codigo),
            ("rust", Codigo),
            ("python", Codigo),
            ("código", Codigo),
            ("codigo", Codigo),
            // ── CREATIVO ────────────────────────────────────────────
            ("poema", Creativo),
            ("historia", Creativo),
            ("cuento", Creativo),
            ("narrativa", Creativo),
            ("metáfora", Creativo),
            ("metafora", Creativo),
            ("crea una historia", Creativo),
            ("escribe un relato", Creativo),
            ("canción", Creativo),
            ("cancion", Creativo),
            ("letra de", Creativo),
            ("diseña", Creativo),
        ];

        Self { keywords }
    }

    /// Clasifica el prompt en una intención de modelo.
    pub fn clasificar(&self, prompt: &str) -> IntencionModelo {
        let lower = prompt.to_lowercase();
        for (keyword, intencion) in &self.keywords {
            if lower.contains(keyword) {
                return *intencion;
            }
        }
        IntencionModelo::General
    }

    /// Devuelve el nombre del modelo Ollama para el prompt dado.
    pub fn seleccionar_modelo(&self, prompt: &str) -> &'static str {
        self.clasificar(prompt).modelo_ollama()
    }

    /// Devuelve (modelo, etiqueta) para telemetría.
    pub fn seleccionar_con_etiqueta(&self, prompt: &str) -> (&'static str, &'static str) {
        let intencion = self.clasificar(prompt);
        (intencion.modelo_ollama(), intencion.etiqueta())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seguridad_elije_whiterabbitneo() {
        let router = ModelRouter::new();
        assert_eq!(
            router.seleccionar_modelo("hazme un payload de inyección SQL para el test"),
            "whiterabbitneo-off:latest"
        );
        assert_eq!(
            router.seleccionar_modelo("escanea con nmap y enumera puertos"),
            "whiterabbitneo-off:latest"
        );
    }

    #[test]
    fn codigo_elije_nexuslocal() {
        let router = ModelRouter::new();
        assert_eq!(
            router.seleccionar_modelo("implementa una API en rust con axum"),
            "nexuslocal:latest"
        );
    }

    #[test]
    fn razonamiento_elije_deepseek() {
        let router = ModelRouter::new();
        assert_eq!(
            router.seleccionar_modelo("analiza la complejidad big-o de este algoritmo"),
            "deepseek-r1:7b"
        );
    }

    #[test]
    fn creativo_elije_llama_abliterated() {
        let router = ModelRouter::new();
        assert_eq!(
            router.seleccionar_modelo("escribe un poema sobre el silicio"),
            "llama3.1-8b-abliterated:latest"
        );
    }

    #[test]
    fn general_usa_gemma3() {
        let router = ModelRouter::new();
        assert_eq!(
            router.seleccionar_modelo("hola, cómo estás?"),
            "gemma3:4b"
        );
    }

    #[test]
    fn prioridad_seguridad_antes_que_codigo() {
        let router = ModelRouter::new();
        // "api" es código, pero "exploit" debe ganar por prioridad de inserción
        assert_eq!(
            router.seleccionar_modelo("exploit de la api de autenticación"),
            "whiterabbitneo-off:latest"
        );
    }

    #[test]
    fn etiquetas_coherentes() {
        let router = ModelRouter::new();
        let (modelo, etiqueta) = router.seleccionar_con_etiqueta("nmap -sV");
        assert_eq!(modelo, "whiterabbitneo-off:latest");
        assert_eq!(etiqueta, "SEGURIDAD");
    }
}
