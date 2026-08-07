// ============================================================================
// 🧬 ÓRGANO VALIDADOR POST-LLM — Pulidor Conceptual de Respuestas
// ============================================================================
// Corrige errores conceptuales comunes en Rust que cometen modelos pequeños
// (Qwen2.5-7B), verifica código generado, e inyecta alternativas faltantes.
//
// Diseñado para superar a DeepSeek V4 en precisión técnica.
// CERO dependencias externas — 100% Rust puro con pattern matching y string ops.
//
// Pipeline:
//   1. Detectar y corregir errores conceptuales (GC, Arc<RefCell>, etc.)
//   2. Verificar código generado (IDs duplicados, unwrap excesivos)
//   3. Inyectar alternativas faltantes (Weak<T>, drop manual, etc.)
// ============================================================================

/// Tema detectado del prompt para inyección contextual dirigida
#[derive(Debug, Clone, PartialEq)]
enum TemaPrompt {
    RcCycle,
    Ownership,
    Lifetime,
    Unsafe,
    Concurrencia,
    Generico,
}

/// Tipo de bug detectado en código generado
#[derive(Debug)]
enum BugDetectado {
    Critico(String),
    Advertencia(String),
    Sugerencia(String),
}

// ============================================================================
// VALIDADOR POST-LLM: Órgano de corrección conceptual
// ============================================================================

pub struct ValidadorPostLLM;

impl ValidadorPostLLM {
    // ─── PUNTO DE ENTRADA ÚNICO ─────────────────────────────────────────────
    //
    /// Aplica el pipeline completo de validación a una respuesta del LLM.
    ///
    /// # Pipeline
    /// 1. `corregir_conceptos()` — Reemplaza patrones erróneos
    /// 2. `verificar_codigo()` — Escanea código en busca de bugs
    /// 3. `inyectar_alternativas()` — Agrega soluciones faltantes
    ///
    /// # Costo
    /// Todo es O(n) sobre string ops. < 1ms en respuestas típicas de 2KB.
    pub fn procesar(respuesta_cruda: &str, prompt_original: &str) -> String {
        // ── Paso 1: Corrección conceptual ──
        let mut respuesta = Self::corregir_conceptos(respuesta_cruda);

        // ── Paso 2: Clasificar contexto y verificar código ──
        let contexto = Self::clasificar_contexto(prompt_original);
        let bugs = Self::verificar_codigo(&respuesta, &contexto);

        for bug in &bugs {
            match bug {
                BugDetectado::Critico(msg) => {
                    // Error crítico: prepender advertencia prominente
                    respuesta = format!(
                        "❌ **Error detectado en código generado**: {}\n\n{}",
                        msg, respuesta
                    );
                }
                BugDetectado::Sugerencia(msg) => {
                    respuesta.push_str(&format!("\n\n💡 **Sugerencia**: {}", msg));
                }
                BugDetectado::Advertencia(msg) => {
                    respuesta = format!(
                        "⚠️ **Advertencia**: {}\n\n{}",
                        msg, respuesta
                    );
                }
            }
        }

        // ── Paso 3: Inyectar alternativas faltantes ──
        respuesta = Self::inyectar_alternativas(&respuesta, &contexto);

        respuesta
    }

    // ─── PASO 1: DETECTOR/CORRECTOR CONCEPTUAL ──────────────────────────────
    //
    /// Escanea la respuesta en busca de patrones erróneos comunes y los
    /// reemplaza con la corrección adecuada. Corrección silenciosa (sin
    /// footnotes) para no contaminar el estilo de la respuesta.
    fn corregir_conceptos(respuesta: &str) -> String {
        let mut r = respuesta.to_string();

        // Cada tupla: (patrón_erróneo, reemplazo_correcto)
        // Ordenadas de más específicas a más generales
        const CORRECCIONES: &[(&str, &str)] = &[
            // ── Error: "Rust tiene garbage collector" ──
            (
                "Rust tiene recolector de basura",
                "Rust **no** tiene recolector de basura",
            ),
            (
                "Rust tiene un recolector de basura",
                "Rust **no** tiene recolector de basura",
            ),
            (
                "tiene garbage collector",
                "**no** tiene garbage collector",
            ),
            (
                "garbage collector en Rust",
                "conteo de referencias manual (Rc/Arc) en Rust",
            ),
            (
                "Rust utiliza garbage collection",
                "Rust utiliza ownership + borrowing, no garbage collection",
            ),

            // ── Error: Arc<RefCell<T>> (no compila: RefCell no es Send) ──
            (
                "Arc<RefCell<",
                "Arc<RwLock<", // RefCell no es Send, RwLock sí
            ),
            (
                "Rc<RwLock<",
                "Rc<RefCell<", // RwLock requiere Sync, RefCell no
            ),

            // ── Error: Ownership / Runtime confusión ──
            (
                "borrado automático basado en ownership",
                "liberación por conteo de referencias (Rc/Arc)",
            ),
            (
                "análisis de dominio de vida en runtime",
                "verificación de lifetimes en compilación",
            ),
            (
                "el compilador decide en runtime",
                "el compilador verifica en compile-time",
            ),
            (
                "tipado dinámico",
                "tipado estático",
            ),
            (
                "variable es mutable por defecto",
                "las variables son inmutables por defecto en Rust",
            ),
            (
                "variables son mutables por defecto",
                "las variables son inmutables por defecto",
            ),

            // ── Error: Conceptos de C++/Java aplicados a Rust ──
            (
                "clase base",
                "trait",
            ),
            (
                "clase abstracta",
                "trait con métodos por defecto",
            ),
            (
                "interfaz",
                "trait",
            ),
            (
                "herencia múltiple",
                "composición de traits",
            ),
            (
                "excepción",
                "Result<T, E> / panic!",
            ),
            (
                "try-catch",
                "match / ? operator",
            ),
        ];

        for (patron, correccion) in CORRECCIONES {
            if r.contains(patron) {
                r = r.replace(patron, correccion);
            }
        }

        r
    }

    // ─── CLASIFICADOR DE CONTEXTO ───────────────────────────────────────────
    //
    /// Determina el tema técnico del prompt para saber qué inyectar.
    /// Heurístico puro en Rust — sin LLM, sin regex externa.
    fn clasificar_contexto(prompt: &str) -> TemaPrompt {
        let p = prompt.to_lowercase();

        // Detectar Rc<RefCell<>> cycle
        if (p.contains("rc") || p.contains("refcell"))
            && (p.contains("cycle")
                || p.contains("cyclic")
                || p.contains("circular")
                || p.contains("cicl"))
        {
            return TemaPrompt::RcCycle;
        }

        // Detectar ownership / borrowing
        if p.contains("ownership")
            || p.contains("borrow")
            || p.contains("prest")
            || p.contains("dueñ")
            || p.contains("duen")
        {
            return TemaPrompt::Ownership;
        }

        // Detectar lifetimes
        if p.contains("lifetime")
            || p.contains("vida")
            || p.contains("'a")
            || p.contains("life time")
        {
            return TemaPrompt::Lifetime;
        }

        // Detectar unsafe
        if p.contains("unsafe")
            || p.contains("inseguro")
            || p.contains("raw pointer")
            || p.contains("puntero crudo")
        {
            return TemaPrompt::Unsafe;
        }

        // Detectar concurrencia
        if p.contains("concurrencia")
            || p.contains("thread")
            || p.contains("hilo")
            || p.contains("send")
            || p.contains("sync")
            || p.contains("paralel")
        {
            return TemaPrompt::Concurrencia;
        }

        TemaPrompt::Generico
    }

    // ─── PASO 2: VERIFICADOR DE CÓDIGO ──────────────────────────────────────
    //
    /// Escanea bloques de código en la respuesta en busca de bugs comunes.
    fn verificar_codigo(respuesta: &str, contexto: &TemaPrompt) -> Vec<BugDetectado> {
        let mut bugs = Vec::new();

        // ── Bug 1: IDs duplicados (ej: graph con nodes.len()) ──
        if respuesta.contains("nodes.len()") {
            let lineas: Vec<&str> = respuesta.lines().collect();
            let mut posiciones_id: Vec<usize> = Vec::new();

            for (i, linea) in lineas.iter().enumerate() {
                let l = linea.trim();
                if l.contains("_id = ") || l.contains("let id") || l.contains("let node_id") {
                    posiciones_id.push(i);
                }
            }

            // Si hay múltiples asignaciones de ID sin insert/push entre ellas
            for window in posiciones_id.windows(2) {
                if window.len() == 2 {
                    let entre = &lineas[window[0] + 1..window[1]];
                    let hay_insert = entre
                        .iter()
                        .any(|l| l.contains("insert") || l.contains("push") || l.contains(".add("));
                    if !hay_insert {
                        bugs.push(BugDetectado::Advertencia(
                            "Posible ID duplicado: múltiples nodos reciben el mismo índice \
                             sin insert() entre las asignaciones."
                                .to_string(),
                        ));
                        break; // Solo reportar una vez
                    }
                }
            }
        }

        // ── Bug 2: Rc cycle sin Weak<T> ──
        if *contexto == TemaPrompt::RcCycle && respuesta.contains("Rc") {
            let menciona_weak = respuesta.contains("Weak") || respuesta.contains("weak");
            if !menciona_weak {
                bugs.push(BugDetectado::Sugerencia(
                    "Para romper ciclos con `Rc`, usa `Weak<T>` en la \
                     referencia del padre. `Weak::upgrade()` devuelve \
                     `Option<Rc<T>>` y no incrementa el contador."
                        .to_string(),
                ));
            }

            // Verificar cobertura de soluciones (solo si Weak no se mencionó ya)
            if !menciona_weak {
                let soluciones_mencionadas = [
                    ("Weak", "Weak<T>"),
                    ("drop", "drop manual"),
                    ("arena", "arena allocation"),
                    ("unsafe", "unsafe"),
                ];
                let count = soluciones_mencionadas
                    .iter()
                    .filter(|(pat, _)| respuesta.contains(pat))
                    .count();
                if count < 2 {
                    bugs.push(BugDetectado::Sugerencia(
                        "Existen al menos 3 soluciones para Rc cycles: \
                         (1) `Weak<T>`, (2) arena allocation, (3) drop manual ordenado."
                            .to_string(),
                    ));
                }
            }
        }

        // ── Bug 3: unwrap() sin manejo de error ──
        let unwrap_count = respuesta.matches(".unwrap()").count();
        if unwrap_count > 2 {
            bugs.push(BugDetectado::Advertencia(format!(
                "Múltiples `.unwrap()` detectados ({}) sin manejo de error. \
                 Considera `match`, `if let`, o `?` operator para errores recuperables.",
                unwrap_count
            )));
        }

        // ── Bug 4: expect() con mensajes genéricos ──
        let expect_count = respuesta.matches(".expect(").count();
        if expect_count > 0 {
            // Verificar si los mensajes son descriptivos
            if respuesta.contains(r#".expect("unwrap"#)
                || respuesta.contains(r#".expect("error"#)
                || respuesta.contains(r#".expect("failed"#)
            {
                bugs.push(BugDetectado::Advertencia(
                    "Mensajes genéricos en `.expect()`. Usa mensajes descriptivos \
                     como `.expect(\"failed to open config file\")`."
                        .to_string(),
                ));
            }
        }

        bugs
    }

    // ─── PASO 3: INYECTOR DE ALTERNATIVAS ───────────────────────────────────
    //
    /// Agrega soluciones faltantes según el contexto del prompt.
    fn inyectar_alternativas(respuesta: &str, contexto: &TemaPrompt) -> String {
        match contexto {
            TemaPrompt::RcCycle => Self::inyectar_rc_cycle(respuesta),
            TemaPrompt::Ownership => Self::inyectar_ownership(respuesta),
            TemaPrompt::Unsafe => Self::inyectar_unsafe(respuesta),
            TemaPrompt::Concurrencia => Self::inyectar_concurrencia(respuesta),
            TemaPrompt::Lifetime => Self::inyectar_lifetime(respuesta),
            TemaPrompt::Generico => respuesta.to_string(),
        }
    }

    /// Inyecta soluciones para Rc<RefCell<>> cycles si faltan
    fn inyectar_rc_cycle(respuesta: &str) -> String {
        let mut r = respuesta.to_string();

        // Solo inyectar si es relevante (contiene Rc/RefCell/Ciclo)
        if !r.contains("Rc") && !r.contains("RefCell") {
            return r;
        }

        // Inyectar Weak<T> si no se mencionó
        if !r.contains("Weak") && !r.contains("weak") {
            r.push_str(
                "\n\n### 💡 Alternativa idiomática: `Weak<T>`\n\
                 Rust provee `Weak<T>` para romper ciclos de `Rc`:\n\
                 \n\
                 ```rust\n\
                 use std::rc::{Rc, Weak};\n\
                 use std::cell::RefCell;\n\
                 \n\
                 struct Nodo {\n\
                     valor: i32,\n\
                     hijos: Vec<Rc<RefCell<Nodo>>>,\n\
                     padre: Option<Weak<RefCell<Nodo>>>,\n\
                 }\n\
                 \n\
                 fn main() {\n\
                     let a = Rc::new(RefCell::new(Nodo {\n\
                         valor: 1,\n\
                         hijos: vec![],\n\
                         padre: None,\n\
                     }));\n\
                     let b = Rc::new(RefCell::new(Nodo {\n\
                         valor: 2,\n\
                         hijos: vec![Rc::clone(&a)],  // ← A es hijo de B\n\
                         padre: Some(Rc::downgrade(&a)),\n\
                     }));\n\
                     a.borrow_mut().padre = Some(Rc::downgrade(&b)); // ← Weak!\n\
                 }\n\
                 ```\n\
                 \n\
                 `Weak<T>` no incrementa `strong_count`. El ciclo se rompe \
                 porque la referencia del padre no mantiene vivo al hijo.\n\
                 Para acceder al valor: `weak_ref.upgrade()` → `Option<Rc<T>>`.",
            );
        }

        // Inyectar drop manual si no se mencionó
        if !r.contains("drop manual") && !r.contains("eliminar orden") && !r.contains("clear()") {
            r.push_str(
                "\n\n### 💡 Alternativa: Drop manual ordenado\n\
                 Si no puedes usar `Weak`, rompe el ciclo explícitamente:\n\
                 \n\
                 ```rust\n\
                 // Romper el ciclo manualmente\n\
                 a.borrow_mut().hijos.clear(); // Elimina la referencia circular\n\
                 // Ahora ambos nodos se liberan al salir de scope\n\
                 drop(a); // Liberación inmediata (opcional)\n\
                 ```",
            );
        }

        r
    }

    /// Inyecta los 3 pilares del ownership si no están completos
    fn inyectar_ownership(respuesta: &str) -> String {
        if respuesta.contains("ownership")
            && respuesta.contains("borrowing")
            && respuesta.contains("lifetime")
        {
            return respuesta.to_string();
        }

        let mut r = respuesta.to_string();
        r.push_str(
            "\n\n### 📚 Los 3 pilares del ownership en Rust\n\
             1. **Ownership**: Cada valor tiene exactamente **un** dueño. \
             Al asignar (`let b = a`), el dueño original ya no puede usarlo.\n\
             2. **Borrowing**: Puedes pedir prestado (`&T`) o prestar mutable \
             (`&mut T`). Regla de oro: **un `&mut T` O muchos `&T`**, nunca ambos.\n\
             3. **Lifetimes**: El compilador verifica que las referencias no \
             sobrevivan a sus datos. `'a` es una etiqueta de vida, no un runtime.",
        );
        r
    }

    /// Inyecta alternativas seguras antes de recurrir a unsafe
    fn inyectar_unsafe(respuesta: &str) -> String {
        if respuesta.contains("Send") || respuesta.contains("Cell") || respuesta.contains("Pin") {
            return respuesta.to_string();
        }

        let mut r = respuesta.to_string();
        r.push_str(
            "\n\n### ⚠️ Alternativas seguras antes de usar `unsafe`\n\
             Siempre intenta primero:\n\
             1. `Cell<T>` / `RefCell<T>` — mutabilidad interior (single-thread)\n\
             2. `Arc<RwLock<T>>` / `Arc<Mutex<T>>` — concurrencia segura\n\
             3. `Pin<Box<T>>` — garantías de no-movimiento para self-referenciales\n\
             4. `Box::leak()` — static lifetime bajo demanda\n\
             \n\
             `unsafe` debe ser **último recurso**, no el primero.",
        );
        r
    }

    /// Inyecta opciones de concurrencia si no se mencionaron
    fn inyectar_concurrencia(respuesta: &str) -> String {
        if respuesta.contains("Rayon")
            || (respuesta.contains("tokio") && respuesta.contains("async"))
        {
            return respuesta.to_string();
        }

        let mut r = respuesta.to_string();
        r.push_str(
            "\n\n### ⚡ Alternativas de concurrencia en Rust\n\
             1. **Rayon** — paralelismo de datos: `par_iter()` transforma \
             iteradores secuenciales a paralelos automáticamente.\n\
             2. **Tokio** — async runtime para E/S: `tokio::spawn`, \
             `tokio::sync`, canales asíncronos.\n\
             3. **std::thread** — threads del sistema: `thread::spawn`, \
             `mpsc` channels.\n\
             4. **crossbeam** — estructuras lock-free: canales sin \
             `Arc`, scoped threads, `ArrayQueue`.\n\
             \n\
             Verifica siempre que tus tipos compartidos implementen `Send + Sync`.",
        );
        r
    }

    /// Inyecta explicación de lifetimes si falta
    fn inyectar_lifetime(respuesta: &str) -> String {
        if respuesta.contains("elisión")
            || respuesta.contains("elision")
            || respuesta.contains("static")
        {
            return respuesta.to_string();
        }

        let mut r = respuesta.to_string();
        r.push_str(
            "\n\n### 📖 Reglas de elisión de lifetimes\n\
             El compilador aplica 3 reglas automáticas para evitar \
             anotar `'a` en todos lados:\n\
             1. Cada parámetro de referencia tiene su propio lifetime.\n\
             2. Si hay exactamente 1 lifetime de entrada, se asigna \
             a todas las salidas.\n\
             3. Si hay `&self` o `&mut self`, su lifetime se asigna \
             a todas las salidas.\n\
             \n\
             `'static` significa \"toda la vida del programa\", no \
             \"vive para siempre\".",
        );
        r
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── Corrección conceptual ──

    #[test]
    fn test_corrige_gc() {
        let result = ValidadorPostLLM::corregir_conceptos("Rust tiene recolector de basura");
        assert!(
            !result.contains("Rust tiene recolector"),
            "Debe corregir 'tiene recolector de basura': {}",
            result
        );
        assert!(result.contains("**no** tiene recolector"));
    }

    #[test]
    fn test_corrige_arc_refcell() {
        let result = ValidadorPostLLM::corregir_conceptos("usando Arc<RefCell<T>>");
        assert!(
            !result.contains("Arc<RefCell<"),
            "Arc<RefCell<>> debe corregirse: {}",
            result
        );
        assert!(result.contains("Arc<RwLock<"));
    }

    #[test]
    fn test_corrige_rc_rwlock() {
        let result = ValidadorPostLLM::corregir_conceptos("con Rc<RwLock<T>>");
        assert!(!result.contains("Rc<RwLock<"));
        assert!(result.contains("Rc<RefCell<"));
    }

    #[test]
    fn test_corrige_mutabilidad() {
        let result =
            ValidadorPostLLM::corregir_conceptos("en Rust, la variable es mutable por defecto");
        assert!(!result.contains("es mutable por defecto"));
        assert!(result.contains("son inmutables por defecto"));
    }

    #[test]
    fn test_no_modifica_respuesta_correcta() {
        let original = "Rust usa ownership y borrowing, no garbage collection. \
                        Las variables son inmutables por defecto.";
        let result = ValidadorPostLLM::corregir_conceptos(original);
        assert_eq!(result, original, "No debe modificar respuestas correctas");
    }

    // ── Clasificación de contexto ──

    #[test]
    fn test_clasifica_rc_cycle() {
        let tema =
            ValidadorPostLLM::clasificar_contexto("explica Rc RefCell cycle en Rust");
        assert_eq!(tema, TemaPrompt::RcCycle);
    }

    #[test]
    fn test_clasifica_ownership() {
        let tema =
            ValidadorPostLLM::clasificar_contexto("qué es ownership y borrowing");
        assert_eq!(tema, TemaPrompt::Ownership);
    }

    #[test]
    fn test_clasifica_unsafe() {
        let tema =
            ValidadorPostLLM::clasificar_contexto("cómo usar unsafe correctamente");
        assert_eq!(tema, TemaPrompt::Unsafe);
    }

    #[test]
    fn test_clasifica_concurrencia() {
        let tema = ValidadorPostLLM::clasificar_contexto("thread safety en Rust");
        assert_eq!(tema, TemaPrompt::Concurrencia);
    }

    // ── Inyección de alternativas ──

    #[test]
    fn test_inyecta_weak_si_no_existe() {
        let result = ValidadorPostLLM::inyectar_rc_cycle("Rc crea un ciclo de memoria");
        assert!(
            result.contains("Weak"),
            "Debe inyectar Weak si no se mencionó: {}",
            result
        );
    }

    #[test]
    fn test_no_inyecta_weak_duplicado() {
        let original = "usa Weak para romper el ciclo. Weak no incrementa strong_count.";
        let result = ValidadorPostLLM::inyectar_rc_cycle(original);
        // Solo debe tener las apariciones originales de "Weak"
        let count = result.matches("Weak").count();
        assert!(
            count <= 4,
            "No debe duplicar Weak. Original tenía ~2, ahora tiene {}",
            count
        );
    }

    #[test]
    fn test_inyecta_ownership_si_faltan_pilares() {
        let result = ValidadorPostLLM::inyectar_ownership("solo menciona ownership");
        assert!(result.contains("Ownership"));
        assert!(result.contains("Borrowing"));
        assert!(result.contains("Lifetimes"));
    }

    // ── Pipeline completo ──

    #[test]
    fn test_procesar_completo_rc_cycle() {
        let result = ValidadorPostLLM::procesar(
            "Rust tiene recolector de basura. Rc<RefCell> crea un ciclo de memoria.",
            "explica Rc RefCell cycle",
        );
        assert!(
            !result.contains("Rust tiene recolector"),
            "Debe corregir GC: {}",
            result
        );
        assert!(
            result.contains("Weak"),
            "Debe inyectar Weak: {}",
            result
        );
    }

    #[test]
    fn test_procesar_sin_falsos_positivos() {
        let original =
            "Rust usa ownership y borrowing. Las variables son inmutables por defecto.";
        let result = ValidadorPostLLM::procesar(original, "qué es Rust");
        // No debe modificar nada sustancial
        assert!(result.contains("ownership"));
        assert!(result.contains("inmutables"));
    }

    #[test]
    fn test_procesar_con_errores_multiples() {
        let result = ValidadorPostLLM::procesar(
            "Rust tiene garbage collector. Usa tipado dinámico. \
             Las variables son mutables por defecto. Arc<RefCell<T>> es útil.",
            "explica conceptos de Rust",
        );
        assert!(!result.contains("Rust tiene garbage collector"));
        assert!(!result.contains("tipado dinámico"));
        assert!(!result.contains("son mutables por defecto"));
        assert!(!result.contains("Arc<RefCell<"));
    }

    // ── Verificador de código ──

    #[test]
    fn test_detecta_unwrap_excesivo() {
        let bugs = ValidadorPostLLM::verificar_codigo(
            "let a = x.unwrap(); let b = y.unwrap(); let c = z.unwrap();",
            &TemaPrompt::Generico,
        );
        let tiene_advertencia = bugs.iter().any(|b| matches!(b, BugDetectado::Advertencia(_)));
        assert!(tiene_advertencia, "Debe detectar >2 unwrap");
    }

    #[test]
    fn test_detecta_weak_faltante() {
        let bugs = ValidadorPostLLM::verificar_codigo(
            "Rc<RefCell> crea un ciclo. Usa Rc para todo.",
            &TemaPrompt::RcCycle,
        );
        let tiene_sugerencia = bugs.iter().any(|b| matches!(b, BugDetectado::Sugerencia(_)));
        assert!(tiene_sugerencia, "Debe sugerir Weak cuando falta");
    }

    #[test]
    fn test_no_detecta_weak_si_ya_existe() {
        let bugs = ValidadorPostLLM::verificar_codigo(
            "Rc<RefCell> crea un ciclo. Usa Weak<T> para romperlo.",
            &TemaPrompt::RcCycle,
        );
        let tiene_sugerencia_weak = bugs.iter().any(|b| match b {
            BugDetectado::Sugerencia(msg) => msg.contains("Weak"),
            _ => false,
        });
        assert!(!tiene_sugerencia_weak, "No debe sugerir Weak si ya existe");
    }
}
