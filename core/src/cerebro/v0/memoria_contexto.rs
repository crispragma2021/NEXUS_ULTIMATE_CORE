// ============================================================================
// 🧠 MEMORIA CONTEXTO — Hipocampo externo para el modelo local (Qwen/Ollama)
// ============================================================================
// El Qwen local (7B) tiene una ventana de contexto pequeña: no le cabe el
// catálogo completo de shadcn/ui ni toda la documentación. Esta memoria es el
// "lugar donde busca su propio contexto": un almacén de fragmentos indexados
// con recuperación selectiva por relevancia y recorte por presupuesto de
// tokens. En vez de inyectar todo, se le entrega solo lo que necesita.
//
// Capacidades (100% deterministas y sin red, para tests herméticos):
//   1. `indexar()`   — almacena un fragmento (componente shadcn, patrón, doc).
//   2. `buscar()`    — recupera fragmentos por relevancia léxica al prompt.
//   3. `recuperar()` — aplica presupuesto de tokens sobre los fragmentos más
//                      relevantes (RAG con ventana pequeña).
//   4. `sembrar_shadcn()` — indexa el catálogo shadcn embebido como base.
// ============================================================================

use super::rag_shadcn::CatalogoShadcn;

/// Fragmento de conocimiento que el modelo puede recuperar.
#[derive(Debug, Clone, PartialEq)]
pub struct FragmentoContexto {
    /// Identificador único (ej. `shadcn:button`, `patron:game-loop`).
    pub id: String,
    /// Categoría del fragmento (componente, patrón, doc, referencia).
    pub categoria: String,
    /// Términos clave para la recuperación (búsqueda léxica).
    pub claves: Vec<String>,
    /// Contenido que se inyecta como contexto al modelo.
    pub contenido: String,
    /// Relevancia calculada en la última búsqueda (0.0–1.0).
    pub score: f32,
}

/// Resultado de una recuperación con presupuesto de tokens.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultadoRecuperacion {
    /// Fragmentos recuperados (ordenados por relevancia descendente).
    pub fragmentos: Vec<FragmentoContexto>,
    /// Contexto concatenado, recortado al presupuesto de tokens.
    pub contexto: String,
    /// Número de tokens estimados del contexto final.
    pub tokens: usize,
    /// Presupuesto máximo aplicado.
    pub presupuesto: usize,
    /// `true` si el presupuesto forzó un recorte (hubo fragmentos descartados).
    pub recortado: bool,
}

/// Hipocampo externo del modelo local.
#[derive(Debug, Clone)]
pub struct MemoriaContexto {
    /// Fragmentos indexados por id (deduplicados).
    fragmentos: Vec<FragmentoContexto>,
}

impl Default for MemoriaContexto {
    fn default() -> Self {
        Self::nueva()
    }
}

impl MemoriaContexto {
    /// Construye una memoria vacía.
    pub fn nueva() -> Self {
        Self {
            fragmentos: Vec::new(),
        }
    }

    /// Índice un fragmento. Si el id ya existe, lo reemplaza (dedup).
    pub fn indexar(&mut self, id: &str, categoria: &str, claves: &[&str], contenido: &str) {
        self.fragmentos.retain(|f| f.id != id);
        self.fragmentos.push(FragmentoContexto {
            id: id.to_string(),
            categoria: categoria.to_string(),
            claves: claves.iter().map(|c| c.to_string()).collect(),
            contenido: contenido.to_string(),
            score: 0.0,
        });
    }

    /// Índice un fragmento desde un nombre + contenido y una lista de claves.
    pub fn indexar_claves(
        &mut self,
        id: &str,
        categoria: &str,
        claves: Vec<String>,
        contenido: &str,
    ) {
        let refs: Vec<&str> = claves.iter().map(|c| c.as_str()).collect();
        self.indexar(id, categoria, &refs, contenido);
    }

    /// Indexa el catálogo shadcn embebido como base de conocimiento.
    /// Cada componente se convierte en un fragmento buscable por su nombre,
    /// categoría y descripción. (Fuente de contexto garantizada.)
    pub fn sembrar_shadcn(&mut self, catalogo: &CatalogoShadcn) {
        for comp in catalogo.componentes.values() {
            let id = format!("shadcn:{}", comp.nombre);
            let mut claves: Vec<String> = Vec::new();
            claves.push(comp.nombre.clone());
            claves.push(comp.categoria.clone());
            claves.extend(
                comp.descripcion
                    .split_whitespace()
                    .map(|w| {
                        w.trim_matches(|c: char| !c.is_alphanumeric())
                            .to_lowercase()
                    })
                    .filter(|w| w.len() > 3),
            );
            let contenido = format!(
                "[shadcn/{}] ({}) {}\nDependencias: {}\nEjemplo: {}",
                comp.nombre,
                comp.categoria,
                comp.descripcion,
                comp.dependencias.join(", "),
                comp.ejemplo
            );
            self.indexar_claves(&id, "componente", claves, &contenido);
        }
    }

    /// Tokeniza de forma aproximada (1 token ≈ 4 caracteres, OpenAI-rule).
    pub fn contar_tokens(texto: &str) -> usize {
        // Aproximación estándar: ~4 chars por token en texto técnico.
        (texto.chars().count() / 4).max(1)
    }

    /// Puntúa un fragmento por solapamiento léxico con el prompt.
    fn relevancia(&self, frag: &FragmentoContexto, terminos: &[String]) -> f32 {
        if terminos.is_empty() {
            return 0.0;
        }
        let mut hits = 0usize;
        for t in terminos {
            if frag.claves.iter().any(|c| c.to_lowercase().contains(t)) {
                hits += 1;
            }
            if frag.contenido.to_lowercase().contains(t) {
                hits += 1;
            }
        }
        (hits as f32) / (terminos.len() as f32).max(1.0)
    }

    /// Extrae términos clave del prompt (palabras significativas > 3 chars).
    fn terminos_de(&self, prompt: &str) -> Vec<String> {
        prompt
            .split(|c: char| !c.is_alphanumeric())
            .map(|w| w.to_lowercase())
            .filter(|w| w.len() > 3)
            .collect()
    }

    /// Recupera los fragmentos más relevantes, recortados al presupuesto de
    /// tokens. Devuelve el contexto listo para inyectar al modelo local.
    pub fn recuperar(&mut self, prompt: &str, presupuesto: usize) -> ResultadoRecuperacion {
        let terminos = self.terminos_de(prompt);
        let presupuesto = presupuesto.max(32);

        // Calcular relevancia de cada fragmento. Se puntúa sobre una copia
        // de las claves para evitar préstamo doble (mutable en `fragmentos`
        // + inmutable en `self` dentro de `relevancia`).
        let scores: Vec<(usize, f32)> = self
            .fragmentos
            .iter()
            .enumerate()
            .map(|(i, f)| (i, self.relevancia(f, &terminos)))
            .collect();
        for (i, score) in scores {
            self.fragmentos[i].score = score;
        }

        // Ordenar por relevancia descendente, luego por id para estabilidad.
        self.fragmentos.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });

        // Seleccionar fragmentos hasta llenar el presupuesto de tokens.
        let mut contexto = String::new();
        let mut tokens = 0usize;
        let mut seleccionados: Vec<FragmentoContexto> = Vec::new();
        let mut recortado = false;

        for frag in self.fragmentos.iter() {
            let costo = Self::contar_tokens(&frag.contenido) + 2; // + separador
            if tokens + costo > presupuesto {
                if !seleccionados.is_empty() {
                    recortado = true;
                }
                break;
            }
            contexto.push_str(&frag.contenido);
            contexto.push('\n');
            tokens += costo;
            seleccionados.push(frag.clone());
        }

        // Si nada cabe (fragmento mayor que presupuesto), forzar el primero.
        if seleccionados.is_empty() {
            if let Some(mejor) = self.fragmentos.first().cloned() {
                recortado = true;
                let recortado_texto: String =
                    mejor.contenido.chars().take(presupuesto * 4).collect();
                seleccionados.push(FragmentoContexto {
                    contenido: recortado_texto.clone(),
                    ..mejor.clone()
                });
                contexto = recortado_texto;
                tokens = Self::contar_tokens(&contexto);
            }
        }

        ResultadoRecuperacion {
            fragmentos: seleccionados,
            contexto,
            tokens,
            presupuesto,
            recortado,
        }
    }

    /// Número de fragmentos indexados.
    pub fn len(&self) -> usize {
        self.fragmentos.len()
    }

    /// `true` si la memoria está vacía.
    pub fn is_empty(&self) -> bool {
        self.fragmentos.is_empty()
    }

    /// Devuelve el contexto completo sin recorte (para depuración / datasets).
    pub fn contexto_completo(&self) -> String {
        self.fragmentos
            .iter()
            .map(|f| f.contenido.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memoria_sembrada() -> MemoriaContexto {
        let mut m = MemoriaContexto::nueva();
        m.sembrar_shadcn(&CatalogoShadcn::estandar());
        m
    }

    #[test]
    fn test_sembrar_shadcn_indexa_catalogo() {
        let m = memoria_sembrada();
        // El catálogo estándar tiene al menos los componentes base.
        assert!(m.len() >= 5);
    }

    #[test]
    fn test_indexar_dedup_reemplaza() {
        let mut m = MemoriaContexto::nueva();
        m.indexar(
            "patron:loop",
            "patrón",
            &["loop", "frame"],
            "requestAnimationFrame",
        );
        m.indexar(
            "patron:loop",
            "patrón",
            &["loop", "frame"],
            "delta-t actualizado por frame",
        );
        assert_eq!(m.len(), 1);
        assert!(m.contexto_completo().contains("delta-t"));
    }

    #[test]
    fn test_recuperar_selecciona_relevantes() {
        let mut m = memoria_sembrada();
        let res = m.recuperar("dashboard con cards y tablas de métricas", 2000);
        assert!(!res.fragmentos.is_empty());
        // Debe priorizar los componentes del dashboard (card, table).
        let ids: Vec<&str> = res.fragmentos.iter().map(|f| f.id.as_str()).collect();
        assert!(ids.iter().any(|id| *id == "shadcn:card"));
        assert!(res.tokens <= 2000);
    }

    #[test]
    fn test_presupuesto_recorta_fragmentos() {
        let mut m = memoria_sembrada();
        let amplio = m.recuperar("dashboard cards", 4000);
        let estrecho = m.recuperar("dashboard cards", 64);
        // Presupuesto estrecho debe caber en tokens.
        assert!(estrecho.tokens <= 64);
        assert!(amplio.fragmentos.len() >= estrecho.fragmentos.len());
    }

    #[test]
    fn test_contexto_no_vacia_siempre() {
        let mut m = memoria_sembrada();
        let res = m.recuperar("algo irrelevante no buscable", 128);
        // Aun sin coincidencias, entrega el fragmento mejor puntuado.
        assert!(!res.fragmentos.is_empty());
        assert!(!res.contexto.is_empty());
    }

    #[test]
    fn test_memoria_vacia_devuelve_vacio() {
        let mut m = MemoriaContexto::nueva();
        let res = m.recuperar("cualquier cosa", 1000);
        assert!(res.fragmentos.is_empty());
        assert!(res.contexto.is_empty());
        assert!(m.is_empty());
    }

    #[test]
    fn test_contar_tokens_aproximado() {
        let n = MemoriaContexto::contar_tokens("hola mundo de prueba con tokens");
        assert!(n >= 1);
    }

    #[test]
    fn test_recuperar_ordena_por_relevancia() {
        let mut m = memoria_sembrada();
        let res = m.recuperar("formulario de login con input y botón", 3000);
        let scores: Vec<f32> = res.fragmentos.iter().map(|f| f.score).collect();
        // Los scores deben ir no-crecientes.
        for w in scores.windows(2) {
            assert!(w[0] >= w[1]);
        }
    }
}
