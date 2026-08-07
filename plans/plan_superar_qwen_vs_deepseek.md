# 🔱 PLAN: Superar DeepSeek V4 con Qwen2.5-7B + Algoritmo Puro

## Diagnóstico de los 5 errores de Qwen2.5

| # | Error | Gravedad | Causa Raíz |
|---|-------|----------|------------|
| 1 | "Rust tiene recolector de basura" | 🔴 CRÍTICO | Alucinación conceptual — modelo pequeño mezcla conceptos |
| 2 | IDs duplicados (`a_id = b_id = 0`) | 🟡 MEDIO | El modelo no "ejecuta" código, solo lo genera por patrón |
| 3 | No mencionó `Weak<T>` | 🟡 MEDIO | La solución más idiomática no está en sus pesos entrenados |
| 4 | "borrado automático basado en ownership" | 🔴 CRÍTICO | Confunde `Rc` con el borrow checker |
| 5 | Solo 1 solución vs 3 de DeepSeek | 🟢 LEVE | Limitación de cobertura |

## Arquitectura: Órgano Validador Post-LLM

```
┌─────────────┐    ┌──────────────────┐    ┌──────────────┐
│   Qwen2.5    │    │  ÓRGANO VALIDADOR │    │  Respuesta   │
│   (LLM)      │───→│  (Rust Puro)      │───→│  Pulida      │
│              │    │                    │    │              │
│  Respuesta   │    │  1. Detectar       │    │  Sin errores │
│  con errores │    │  2. Corregir       │    │  + mejorada  │
└─────────────┘    │  3. Enriquecer     │    └──────────────┘
                   └──────────────────┘
```

## Componentes del Validador

### 1. Detector de Errores Conceptuales (`detector_conceptual.rs`)

Reglas hardcodeadas en Rust que escanean la respuesta del LLM:

```rust
// Reglas de detección — puro pattern matching en Rust
const ERRORES_CONCEPTUALES: &[(&str, &str, &str)] = &[
    // (patrón_erróneo, concepto_correcto, explicación_corta)
    (
        "recolector de basura",
        "Rust no tiene garbage collector",
        "Rust usa ownership + borrow checker, no GC. Rc<RefCell> es conteo de referencias manual, no recolección de basura."
    ),
    (
        "borrado automático basado en",
        "Rc usa conteo de referencias",
        "El drop de Rc ocurre cuando strong_count llega a 0, no por analysis de ownership."
    ),
    (
        "análisis de dominio de vida",
        "lifetime analysis del compilador",
        "Los lifetimes son verificados en compilación, no en runtime. Rc/RefCell son runtime."
    ),
    (
        "tipado dinámico",
        "tipado estático",
        "Rust es tipado estáticamente. No hay type erasure como en Java/C#."
    ),
];
```

### 2. Verificador de Código (`verificador_codigo.rs`)

Parseo básico para detectar bugs comunes en código generado:

```rust
pub fn verificar_codigo_generado(respuesta: &str) -> Vec<BugDetectado> {
    let mut bugs = Vec::new();
    
    // Bug 1: IDs duplicados
    if let Some(id_assignment) = extraer_asignaciones_id(respuesta) {
        // Si dos variables reciben el mismo valor de nodes.len() sin insert
        bugs.extend(detectar_ids_duplicados(&id_assignment));
    }
    
    // Bug 2: Rc cycle sin Weak
    if respuesta.contains("Rc") && respuesta.contains("RefCell") {
        if !respuesta.contains("Weak") {
            bugs.push(BugDetectado::Sugerencia(
                "Ciclo Rc detectado. Sugerencia: usar Weak<T> para referencia del padre."
            ));
        }
    }
    
    // Bug 3: unwrap/expect sin manejo de error
    if respuesta.contains(".unwrap()") && !respuesta.contains("match") {
        bugs.push(BugDetectado::Advertencia(
            "Múltiples .unwrap() sin manejo de error. Considerar match o ? operator."
        ));
    }
    
    bugs
}
```

### 3. Inyector de Alternativas (`inyector_alternativas.rs`)

Para preguntas técnicas, inyecta soluciones faltantes:

```rust
// Template-based enrichment
pub fn enriquecer_respuesta(respuesta: &str, contexto: ContextoPregunta) -> String {
    match contexto.tema {
        Tema::RcCycle => {
            // Si no mencionó Weak, agregarlo
            if !respuesta.contains("Weak") {
                let extra = format!(
                    "\n\n### 💡 Alternativa más idiomática: Weak<T>\n\
                    En lugar de arena allocation, Rust ofrece `Weak<T>`:\n\
                    ```rust\n\
                    use std::rc::Weak;\n\
                    struct Nodo {{\n\
                        valor: i32,\n\
                        hijos: Vec<Rc<RefCell<Nodo>>>,\n\
                        padre: Option<Weak<RefCell<Nodo>>>,\n\
                    }}\n\
                    ```\n\
                    `Weak` no incrementa el contador de referencias, rompiendo el ciclo.";
                respuesta.push_str(extra);
            }
        },
        Tema::Unsafe => {
            // Inyectar SAFE alternatives
        },
        _ => {}
    }
    respuesta
}
```

### 4. Post-Process Pipeline

```rust
pub struct ValidadorPostLLM;

impl ValidadorPostLLM {
    pub fn procesar(respuesta_cruda: &str, prompt_original: &str) -> String {
        let mut respuesta = respuesta_cruda.to_string();
        
        // Paso 1: Detectar y corregir errores conceptuales
        for (patron, correcto, explicacion) in ERRORES_CONCEPTUALES {
            if respuesta.contains(patron) {
                respuesta = respuesta.replace(patron, correcto);
                // Agregar footnote con explicación
                respuesta.push_str(&format!(
                    "\n\n> ⚠️ **Nota de corrección**: Reemplacé '{}' por '{}'. {}",
                    patron, correcto, explicacion
                ));
            }
        }
        
        // Paso 2: Verificar código generado
        let bugs = verificar_codigo_generado(&respuesta);
        for bug in bugs {
            match bug {
                BugDetectado::Critico(msg) => {
                    // Agregar advertencia prominente
                    respuesta = format!("❌ **Error detectado en código**: {}\n\n{}", msg, respuesta);
                },
                BugDetectado::Sugerencia(msg) => {
                    respuesta.push_str(&format!("\n\n💡 **Sugerencia**: {}", msg));
                },
                _ => {}
            }
        }
        
        // Paso 3: Enriquecer con alternativas faltantes
        let contexto = clasificar_contexto(prompt_original);
        respuesta = enriquecer_respuesta(&respuesta, contexto);
        
        respuesta
    }
}
```

## Mapa de Implementación

| Archivo | Función | Líneas |
|---------|---------|--------|
| `src-tauri/src/validador_post_llm.rs` | Módulo principal | ~200 |
| `src-tauri/src/validador_post_llm.rs` → `detector_conceptual` | Reglas de corrección | ~80 |
| `src-tauri/src/validador_post_llm.rs` → `verificador_codigo` | Parseo de código | ~100 |
| `src-tauri/src/validador_post_llm.rs` → `inyector_alternativas` | Templates de enriquecimiento | ~80 |
| `src-tauri/src/razonador_local.rs` | Integrar validador en pipeline | +5 líneas |

## Integración en Pipeline

Solo 3 cambios en [`razonador_local.rs`](src-tauri/src/razonador_local.rs):

**En `procesar_con_cot` (línea 369)**:
```rust
// ANTES
let respuesta = llamar_ollama_directo(prompt, ...).await;

// DESPUÉS
let respuesta = llamar_ollama_directo(prompt, ...).await;
let respuesta = ValidadorPostLLM::procesar(&respuesta, prompt);  // ← NUEVO
```

**En `llamar_ollama_directo` (línea 334)**:
```rust
// ANTES
Ok(contenido) => contenido,
// DESPUÉS
Ok(contenido) => ValidadorPostLLM::procesar(&contenido, prompt),  // ← NUEVO
```

**En `sintetizar_respuesta` (línea 276)**:
```rust
// ANTES
Ok(respuesta_final) => Ok(respuesta_final),
// DESPUÉS
Ok(respuesta_final) => Ok(ValidadorPostLLM::procesar(&respuesta_final, prompt_original)),
```

## Zero Dependencies — Rust Puro

Todo el validador se implementa con:
- `std` (regex Lite con `contains`, `split`, `lines`)
- Pattern matching
- Sin serde, sin regex externa, sin LLM

## Métricas de Éxito

| Métrica | Antes | Después (target) |
|---------|-------|-------------------|
| Errores conceptuales (GC, ownership) | 2/5 respuestas | 0/5 |
| Bugs en código generado | 1/3 respuestas | 0/3 |
| Cobertura de alternativas | 1 solución | 3+ soluciones |
| Tiempo adicional | 0ms | <1ms (puro string ops) |
| Dependencias nuevas | — | 0 (cero) |
