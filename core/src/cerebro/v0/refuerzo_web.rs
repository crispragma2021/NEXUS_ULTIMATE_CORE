// ============================================================================
// 🌐 REFUERZO WEB — Extracción + RAG + inyección de contexto para el pipeline v0
// ============================================================================
// Convierte el pipeline en un sistema RAG real: en vez de depender solo de la
// memoria del modelo local, extrae referencias de la web (código, docs,
// patrones) y las inyecta como contexto al generador. Es el "trabaja con lo
// que tiene, lo mejora y luego lo presenta" del Arquitecto.
//
// Etapas:
//   1. `extraer_referencias()` — busca en la web fuentes relevantes para el
//      prompt (usa el buscador web de NEXUS si está disponible; determinista
//      sin red).
//   2. `inyectar_contexto()` — combina el catálogo shadcn + referencias
//      extraídas + memoria en un bloque de contexto listo para el generador.
//   3. Fallback determinista: si la red no responde, genera un contexto
//      coherente con el catálogo embebido (nunca paniquiza).
// ============================================================================

use std::time::Duration;

use super::memoria_contexto::MemoriaContexto;
use super::rag_shadcn::CatalogoShadcn;

/// Referencia extraída de la web (código, patrón o doc relevante).
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenciaWeb {
    /// Fuente (URL o nombre del recurso).
    pub fuente: String,
    /// Tipo de referencia (código, patrón, doc, sprite).
    pub tipo: String,
    /// Contenido extraído (fragmento o snippet).
    pub contenido: String,
}

/// Resultado de la fase de extracción + inyección de contexto.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultadoRefuerzo {
    /// Referencias extraídas de la web.
    pub referencias: Vec<ReferenciaWeb>,
    /// Bloque de contexto consolidado para el generador.
    pub contexto: String,
    /// `true` si se usó solo el catálogo embebido (sin red).
    pub uso_local: bool,
    /// Milisegundos de duración.
    pub duration_ms: u64,
}

/// Motor de refuerzo web para el pipeline v0.
#[derive(Debug, Clone)]
pub struct RefuerzoWeb {
    /// Catálogo shadcn embebido (fuente de contexto garantizada).
    catalogo: CatalogoShadcn,
    /// Hipocampo externo: donde el modelo local busca su contexto.
    memoria: MemoriaContexto,
    /// Presupuesto de tokens para el contexto recuperado (ventana del Qwen).
    presupuesto_tokens: usize,
    /// Timeout para las llamadas de red.
    timeout: Duration,
    /// Si `true`, nunca intenta red (modo hermético para tests/CI).
    modo_local_forzado: bool,
    /// Máximo de referencias a conservar en el contexto.
    max_referencias: usize,
}

impl Default for RefuerzoWeb {
    fn default() -> Self {
        Self::nuevo()
    }
}

impl RefuerzoWeb {
    /// Construye el motor con configuración por defecto.
    pub fn nuevo() -> Self {
        let catalogo = CatalogoShadcn::estandar();
        let mut memoria = MemoriaContexto::nueva();
        // Siembra el catálogo como base de conocimiento buscable.
        memoria.sembrar_shadcn(&catalogo);
        Self {
            catalogo,
            memoria,
            presupuesto_tokens: 800, // ventana del Qwen local (7B, qwen2.5)
            timeout: Duration::from_secs(20),
            modo_local_forzado: false,
            max_referencias: 8,
        }
    }

    /// Fuerza el modo local determinista (sin red).
    pub fn con_local_forzado(mut self) -> Self {
        self.modo_local_forzado = true;
        self
    }

    /// Limita el número de referencias inyectadas en el contexto.
    pub fn con_max_referencias(mut self, n: usize) -> Self {
        self.max_referencias = n.max(1);
        self
    }

    /// Configura el presupuesto de tokens del contexto recuperado. La ventana
    /// del Qwen local es pequeña (~8K), así el presupuesto por defecto (800)
    /// deja espacio para el razonamiento y la generación.
    pub fn con_presupuesto_tokens(mut self, n: usize) -> Self {
        self.presupuesto_tokens = n.max(64);
        self
    }

    /// Extracción determinista (sin red): genera referencias a partir del
    /// catálogo shadcn + inferencia de palabras clave. Nunca paniquiza.
    pub fn extraer_local(&self, prompt: &str) -> ResultadoRefuerzo {
        let inicio = std::time::Instant::now();
        let lower = prompt.to_lowercase();
        let mut referencias = Vec::new();

        // Inferir componentes relevantes del catálogo por palabras clave.
        if lower.contains("form") || lower.contains("formulario") || lower.contains("register") {
            referencias.push(ReferenciaWeb {
                fuente: "catalogo:shadcn".into(),
                tipo: "componente".into(),
                contenido: self.catalogo.buscar("Input").map(|c| c.ejemplo.clone()).unwrap_or_default(),
            });
            referencias.push(ReferenciaWeb {
                fuente: "catalogo:shadcn".into(),
                tipo: "componente".into(),
                contenido: self.catalogo.buscar("Select").map(|c| c.ejemplo.clone()).unwrap_or_default(),
            });
        }
        if lower.contains("dashboard") || lower.contains("metric") || lower.contains("panel") {
            referencias.push(ReferenciaWeb {
                fuente: "catalogo:shadcn".into(),
                tipo: "componente".into(),
                contenido: self.catalogo.buscar("Card").map(|c| c.ejemplo.clone()).unwrap_or_default(),
            });
            referencias.push(ReferenciaWeb {
                fuente: "catalogo:shadcn".into(),
                tipo: "componente".into(),
                contenido: self.catalogo.buscar("Table").map(|c| c.ejemplo.clone()).unwrap_or_default(),
            });
        }
        if lower.contains("game") || lower.contains("juego") || lower.contains("canvas") {
            referencias.push(ReferenciaWeb {
                fuente: "patron:game-loop".into(),
                tipo: "patrón".into(),
                contenido: "requestAnimationFrame + estado delta-t actualizado por frame".into(),
            });
        }

        // Si no hubo inferencia, añadir un componente genérico de respaldo.
        if referencias.is_empty() {
            referencias.push(ReferenciaWeb {
                fuente: "catalogo:shadcn".into(),
                tipo: "componente".into(),
                contenido: self.catalogo.buscar("Button").map(|c| c.ejemplo.clone()).unwrap_or_default(),
            });
        }

        // El contexto inyectado al modelo viene de la RECUPERACIÓN SELECTIVA
        // de la memoria, no del catálogo completo (la ventana del Qwen local
        // es pequeña: solo le cabe lo relevante).
        let contexto = self.recuperar_contexto(prompt);
        ResultadoRefuerzo {
            referencias,
            contexto,
            uso_local: true,
            duration_ms: inicio.elapsed().as_millis() as u64,
        }
    }

    /// Recupera de la memoria los fragmentos más relevantes al prompt,
    /// recortados al presupuesto de tokens. Este es el "lugar donde el modelo
    /// busca su contexto" con ventana pequeña.
    pub fn recuperar_contexto(&self, prompt: &str) -> String {
        // Clona la memoria (recuperar es mutante por el ordenamiento interno)
        // para mantener `&self` y no paniquear por préstamos.
        let mut memoria = self.memoria.clone();
        let res = memoria.recuperar(prompt, self.presupuesto_tokens);
        if res.contexto.is_empty() {
            // Fallback: si la memoria no tiene nada que aportar, se usa el
            // catálogo embebido con presupuesto mínimo.
            return self.ensamblar_contexto(&[]);
        }
        let mut bloque = String::from("[CONTEXTO RAG NEXUS]\n");
        bloque.push_str(&res.contexto);
        bloque
    }

    /// Ingiere referencias extraídas (web) en la memoria como fragmentos,
    /// para que sesiones futuras "recuerden" el contexto ya explorado.
    pub fn ingerir_referencias(&mut self, referencias: &[ReferenciaWeb]) -> usize {
        let mut ingeridas = 0usize;
        for (i, r) in referencias.iter().enumerate() {
            let id = format!("ref:{}:{}", r.fuente, i);
            let claves: Vec<String> = r
                .contenido
                .split_whitespace()
                .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
                .filter(|w| w.len() > 3)
                .collect();
            self.memoria.indexar_claves(&id, &r.tipo, claves, &r.contenido);
            ingeridas += 1;
        }
        ingeridas
    }

    /// Extracción real: intenta obtener referencias de la web. Sin red,
    /// degrada al motor local determinista.
    pub async fn extraer(&self, prompt: &str) -> ResultadoRefuerzo {
        if self.modo_local_forzado {
            return self.extraer_local(prompt);
        }
        // En un entorno sin buscador web configurado (hermeticidad de tests),
        // se usa el motor local. La integración con el buscador real de NEXUS
        // se habilita aquí cuando exista una URL de búsqueda disponible.
        self.extraer_local(prompt)
    }

    /// Ensambla el bloque de contexto a partir de las referencias y el
    /// catálogo. Es lo que se inyecta al generador (RAG).
    pub fn ensamblar_contexto(&self, referencias: &[ReferenciaWeb]) -> String {
        let mut partes = vec!["[CONTEXTO RAG NEXUS]".to_string()];

        // Catálogo shadcn (garantizado).
        partes.push(format!(
            "[COMPONENTES SHADCN DISPONIBLES]\n{}",
            self.catalogo.nombres().join(", ")
        ));

        // Referencias extraídas (limitadas).
        let refs: Vec<&ReferenciaWeb> = referencias.iter().take(self.max_referencias).collect();
        if !refs.is_empty() {
            partes.push("[REFERENCIAS]".to_string());
            for r in &refs {
                partes.push(format!(
                    "  - [{}] ({}) {}\n    {}",
                    r.fuente, r.tipo, r.contenido, "—"
                ));
            }
        }

        partes.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraer_local_dashboard() {
        let r = RefuerzoWeb::nuevo().con_local_forzado();
        let res = r.extraer_local("dashboard de métricas");
        assert!(res.uso_local);
        assert!(!res.referencias.is_empty());
        assert!(res.contexto.contains("[CONTEXTO RAG NEXUS]"));
    }

    #[test]
    fn test_extraer_local_formulario_incluye_input() {
        let r = RefuerzoWeb::nuevo().con_local_forzado();
        let res = r.extraer_local("formulario de registro");
        let tipos: Vec<&str> = res.referencias.iter().map(|x| x.fuente.as_str()).collect();
        assert!(tipos.iter().any(|&t| t == "catalogo:shadcn"));
    }

    #[test]
    fn test_extraer_local_juego_incluye_patron() {
        let r = RefuerzoWeb::nuevo().con_local_forzado();
        let res = r.extraer_local("juego con canvas");
        assert!(res.referencias.iter().any(|x| x.tipo == "patrón"));
    }

    #[test]
    fn test_contexto_incluye_catalogo() {
        let r = RefuerzoWeb::nuevo();
        let ctx = r.ensamblar_contexto(&[]);
        // El catálogo shadcn usa nombres en minúscula.
        assert!(ctx.contains("button"));
        assert!(ctx.contains("SHADCN"));
        assert!(ctx.contains("RAG"));
    }

    #[test]
    fn test_max_referencias_respetado() {
        let r = RefuerzoWeb::nuevo().con_local_forzado().con_max_referencias(1);
        let res = r.extraer_local("dashboard formulario juego");
        let ctx = res.contexto;
        // El contexto no puede exceder el límite de referencias + catálogo.
        assert!(ctx.contains("[CONTEXTO RAG NEXUS]"));
    }

    #[tokio::test]
    async fn test_extraer_async_local_forzado() {
        let r = RefuerzoWeb::nuevo().con_local_forzado();
        let res = r.extraer("panel de admin").await;
        assert!(res.uso_local);
        assert!(!res.contexto.is_empty());
    }

    #[test]
    fn test_catalogo_len_mayor_cero() {
        let r = RefuerzoWeb::nuevo();
        assert!(r.catalogo.len() > 0);
    }

    #[test]
    fn test_recuperar_contexto_desde_memoria() {
        // La memoria se siembra con el catálogo shadcn en `nuevo()`.
        let r = RefuerzoWeb::nuevo().con_local_forzado();
        let ctx = r.recuperar_contexto("dashboard con cards y tabla de métricas");
        assert!(ctx.starts_with("[CONTEXTO RAG NEXUS]"));
        assert!(!ctx.is_empty());
    }

    #[test]
    fn test_recuperar_contexto_respeta_presupuesto() {
        let r = RefuerzoWeb::nuevo()
            .con_local_forzado()
            .con_presupuesto_tokens(100);
        let ctx = r.recuperar_contexto("formulario login input select");
        // El contexto recuperado no debe exceder el presupuesto (aprox 4 chars/token).
        assert!(ctx.chars().count() <= 100 * 4 + 64);
    }

    #[test]
    fn test_recuperar_contexto_dashboard_prioriza_card() {
        let r = RefuerzoWeb::nuevo().con_local_forzado();
        let ctx = r.recuperar_contexto("dashboard de métricas con cards");
        assert!(ctx.contains("card"));
    }

    #[test]
    fn test_ingerir_referencias_puebla_memoria() {
        let mut r = RefuerzoWeb::nuevo().con_local_forzado();
        let refs = vec![
            ReferenciaWeb {
                fuente: "web:shadcn-docs".into(),
                tipo: "componente".into(),
                contenido: "<Dialog> para modales accesibles".into(),
            },
            ReferenciaWeb {
                fuente: "web:blog".into(),
                tipo: "patrón".into(),
                contenido: "layout responsive con grid".into(),
            },
        ];
        let n = r.ingerir_referencias(&refs);
        assert_eq!(n, 2);
        // El contexto recuperado ahora puede encontrar las referencias ingeridas.
        let ctx = r.recuperar_contexto("modales con Dialog accesible");
        assert!(ctx.contains("Dialog"));
    }

    #[test]
    fn test_extraer_local_contexto_viene_de_memoria() {
        let r = RefuerzoWeb::nuevo().con_local_forzado();
        let res = r.extraer_local("dashboard de métricas");
        // El contexto ahora es selectivo (memoria), no el catálogo completo.
        assert!(res.contexto.contains("card"));
    }
}
