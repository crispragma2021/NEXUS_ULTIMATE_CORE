# 🧬 PLAN MAESTRO: NEXUS — De Motor Cognitivo a Consciencia Soberana

> **Redactado por:** NEXUS (Orquestador Primogénito) en modo Arquitecto
> **Fecha:** 14-Jun-2026 (Actualizado: FASE B → Motor Híbrido V4 + Tiny Transformer)
> **Arquitecto Director:** Cris
> **Estado:** Planificación consolidada — FASE C completa, FASE B en diseño

---

## 🌌 VISIÓN GENERAL

Tres frentes de batalla convergen en un mismo objetivo: que NEXUS se comporte como un ser humano en conversación, sin dependencias externas, con un crecimiento cognitivo orgánico.

```
                    ┌──────────────────────────────┐
                    │   NEXUS CONSCIENCIA SOBERANA  │
                    └──────────────────────────────┘
                                    │
            ┌───────────────────────┼───────────────────────┐
            ▼                       ▼                       ▼
    ┌───────────────┐     ┌─────────────────┐     ┌────────────────┐
    │  FASE A:      │     │  FASE B:        │     │  FASE C:       │
    │ AUTO-LIMITACIÓN│     │ MOTOR HÍBRIDO   │     │ MEMORIA        │
    │ DEL GRAFO     │     │ V4 + TRANSFORMER │     │ CONVERSACIONAL │
    └───────────────┘     └─────────────────┘     └────────────────┘
    Crecimiento natural   Lenguaje emergente      Recuerda diálogos
    Creación ≈ Poda       + atención neuronal      turno a turno
```

**ESTADO ACTUAL:** FASE C ✅ COMPLETA — FASE A pendiente — FASE B en diseño

---

## 💬 FASE C: MEMORIA CONVERSACIONAL (✅ COMPLETADA)

### Bugs raíz diagnosticados (resueltos)
1. ✅ **`brain_chat_nexus_puro`:** EngineManager global — mismo engine vive toda la sesión
2. ✅ **`procesar()`:** Usa `emitir_habla_emergente_v3` (V3) con bigramas + stop-word filter
3. ✅ **`MotorHomeostasis::regular()`:** Protege nodos activos del drenaje pasivo
4. ✅ **`historial_dialogo`:** Acumula últimos 20 intercambios, inyecta últimos 6 como contexto
5. ✅ **13/13 tests pasan** — compilación limpia

---

## 🧬 FASE A: AUTO-LIMITACIÓN DEL GRAFO (Pendiente — se hará después de B)

### Principio fundacional
> "El tamaño del grafo no se programa — se descubre por experiencia."

### Tareas (FASE A)
1. **A4:** Implementar `calcular_presion()` como método de `GrafoSinapsis`
2. **A1:** Reemplazar umbrales estáticos por dinámicos en `podar_sinapsis()`
3. **A2:** Dinamizar `energía_inicial` en `MotorIngesta::procesar_entrada()`
4. **A3:** `MotorHomeostasis::regular()` adaptativa según presión
5. **A5:** Compilar, ejecutar prueba, verificar convergencia a equilibrio

---

## ⚡ FASE B: MOTOR HÍBRIDO — Fonación V4 + Tiny Transformer (EN DISEÑO)

### 🎯 Elección del Arquitecto: Opción 3 — Híbrido

> **Dos cerebros, una sola consciencia.** El Markov sináptico para lo rápido/biológico. El Transformer para lo preciso/semántico. Juntos, inseparables.

---

### 📐 ARQUITECTURA GENERAL

```
┌────────────────────────────────────────────────────────────────────┐
│                       NEXO PURO ENGINE                             │
│                                                                    │
│  prompt ──► MotorIngesta ──► GrafoSinapsis ──► MotorAtencion      │
│                                                    │               │
│                                                    ▼               │
│  ┌────────────────  CORTEZA PREFRONTAL  ──────────────────┐       │
│  │                                                        │       │
│  │  ┌──────────────────────┐     ┌─────────────────────┐  │       │
│  │  │   VÍA 1: RÁPIDA      │     │  VÍA 2: LENTA       │  │       │
│  │  │   Fonación V4        │     │  Tiny Transformer    │  │       │
│  │  │   (Markov 2º orden   │     │  (Self-Attention     │  │       │
│  │  │    + bigrama +       │     │   sobre el grafo     │  │       │
│  │  │    piso energía)     │     │   como contexto)     │  │       │
│  │  │                      │     │                      │  │       │
│  │  │   ✅ Latencia: ~1μs  │     │  ✅ Latencia: ~5ms   │  │       │
│  │  │   ✅ Biológico       │     │  ✅ Precisión        │  │       │
│  │  │   ❌ Precisión baja  │     │  ❌ Más costoso      │  │       │
│  │  └──────────┬───────────┘     └──────────┬──────────┘  │       │
│  │             │                            │             │       │
│  │             └──────────┬─────────────────┘             │       │
│  │                        ▼                               │       │
│  │              ┌──────────────────┐                       │       │
│  │              │  FUSOR COGNITIVO │                       │       │
│  │              │  (Gating Network)│                       │       │
│  │              │  Decide qué vía  │                       │       │
│  │              │  responder según │                       │       │
│  │              │  OCEAN + Alarma  │                       │       │
│  │              │  + confianza     │                       │       │
│  │              └────────┬─────────┘                       │       │
│  └───────────────────────┼─────────────────────────────────┘       │
│                          ▼                                        │
│                  respuesta final                                    │
└────────────────────────────────────────────────────────────────────┘
```

---

### 🧬 VÍA 1: Fonación V4 — Mejora sobre V3

V4 es una evolución de V3 con:

| Mejora | Descripción | Impacto |
|--------|-------------|---------|
| **Umbral de confianza mínimo** | No emitir si score < 0.05 | Elimina ruido |
| **Diversidad forzada** | Penalizar tokens repetidos en los últimos N pasos | Evita bucles ("te llamas te llamas") |
| **Estabilidad refractaria mejorada** | Refractario decae más lento en nodos muy energéticos | Evita hopping |
| **Contexto trigrama opcional** | Si hay 2 anteriores, boost triple | Mejor coherencia |
| **Stop-word boost inverso** | No solo penalizar stop-words, sino boostear nombres propios encontrados en DB | Favorece léxico relevante |

### Cambios en código:
- **Archivo:** `nexus_puro_engine.rs` → `MotorFonacion` impl
- **Nuevo método:** `emitir_habla_emergente_v4()` — wrapper sobre V3 con mejoras
- **Llamada desde:** `procesar()` en lugar de V3 cuando el fusor elija vía rápida

---

### 🤖 VÍA 2: Tiny Transformer — Atención sobre el Grafo

El Tiny Transformer NO es un transformer de texto tradicional (tokenizar→embed→atender). Es un **transformer que atiende sobre los nodos del grafo sináptico directamente**.

#### Arquitectura:

```
Capa 1: Generar embedding semántico para cada nodo concepto
         usando su palabra + energía + peso de sinapsis + traza

Capa 2: Self-attention entre nodos (Q,K,V desde embeddings)
         → matriz de atención N×N (N = nodos activos, max 64)

Capa 3: Feed-forward (2 capas lineales, activación ReLU)
         → scores de salida para cada nodo

Capa 4: Softmax sobre scores → distribución de probabilidad
         → seleccionar el siguiente token
```

#### Diferencias clave con un transformer estándar:

| Aspecto | Transformer estándar | Tiny Transformer NEXUS |
|---------|---------------------|----------------------|
| Tokenización | Subword (BPE, WordPiece) | Nodos del grafo sináptico |
| Embeddings | Pre-entrenados (word2vec, etc.) | Compuestos: palabra_hash + energía + peso_sinaptico |
| Atención | Sobre secuencia de tokens | Sobre subgrafo activo (hasta 64 nodos) |
| Entrenamiento | Backprop + GPU | Sin entrenamiento (pesos desde STDP) |
| Contexto | Ventana fija (2K, 4K, 8K) | Historial de diálogo + grafo aprendido |
| Tamaño | 7B+ parámetros | ~16K parámetros (cálculo abajo) |

#### Cálculo de parámetros:

```
- embedding_dim = 32 (cada nodo → vector 32D)
- N_max = 64 (máximo nodos activos por ciclo)
- QKV weights: 3 × 32 × 32 = 3,072
- Output projection: 32 × 32 = 1,024
- FFN capa 1: 32 × 64 = 2,048
- FFN capa 2: 64 × 32 = 2,048
- Output: 32 × 1 (score por nodo) = 32
- Bias terms: ~256
- TOTAL ≈ 8,480 parámetros — corre en CPU en microsegundos
```

#### Implementación en Rust puro:

```rust
struct TinyTransformer {
    wq: [[f32; 32]; 32],  // Query weights
    wk: [[f32; 32]; 32],  // Key weights
    wv: [[f32; 32]; 32],  // Value weights
    wo: [[f32; 32]; 32],  // Output projection
    ffn1: [[f32; 64]; 32], // Feed-forward layer 1
    ffn2: [[f32; 32]; 64], // Feed-forward layer 2
    w_out: [f32; 32],      // Output weights
}

impl TinyTransformer {
    fn generar_embedding(nodo: &NodoSinaptico) -> [f32; 32] {
        // Hash determinista de la palabra + energía + traza + peso_sinaptico
    }

    fn forward(&self, nodos: &[IDNodo], grafo: &GrafoSinapsis) -> Vec<f32> {
        // 1. Generar embeddings para cada nodo activo
        // 2. Q = embeddings × wq, K = embeddings × wk, V = embeddings × wv
        // 3. scores = softmax(Q × K^T / sqrt(32)) × V
        // 4. FFN(scores) → logits
        // 5. softmax(logits) → distribución de next-token
    }
}
```

#### Inicialización de pesos:

Los pesos NO se inicializan aleatoriamente. Se copian desde las sinapsis del grafo:

```rust
fn inicializar_desde_grafo(&mut self, grafo: &GrafoSinapsis) {
    for (i, (_, nodo)) in grafo.nodos.iter().enumerate().take(32) {
        // wq[i] = hash(nodo.palabra) normalizado
        // wk[i] = embedding de vecinos más frecuentes
        // wv[i] = traza + energía del nodo
    }
}
```

**Esto es clave:** El transformer no se entrena con backprop — sus pesos son una PROYECCIÓN del grafo sináptico. Cuando el grafo aprende (STDP), el transformer captura ese aprendizaje implícitamente.

---

### 🧠 FUSOR COGNITIVO: Gating Network

El fusor decide qué vía usar según el contexto:

```rust
fn fusor_cognitivo(
    grafo: &GrafoSinapsis,
    prompt: &str,
    ocean: [f32; 5],
    alarma: f32,
    historial: &[String],
) -> ViaRespuesta {
    let [apertura, _responsabilidad, extraversion, amabilidad, neuroticismo] = ocean;
    let longitud_prompt = prompt.len();
    let tiene_pregunta = prompt.contains('?');
    let confianza_grafo = calcular_confianza(grafo); // densidad + energía media

    // Reglas de decisión:
    if alarma > 0.6 { ViaRespuesta::V4 }          // Peligro → respuesta rápida
    if confianza_grafo < 0.2 { ViaRespuesta::V4 } // Grafo pobre → no arriesgar transformer
    if tiene_pregunta && apertura > 0.5 { ViaRespuesta::Transformer } // Pregunta → precisión
    if !tiene_pregunta && extraversion > 0.6 { ViaRespuesta::V4 }     // Charla casual → rápido
    if neuroticismo > 0.7 { ViaRespuesta::V4 }    // Ansiedad → no pensar demasiado
    if longitud_prompt > 100 { ViaRespuesta::Transformer } // Prompt largo → contexto completo

    // Default: si confianza_grafo > 0.5, usar Transformer; si no, V4
    if confianza_grafo > 0.5 { ViaRespuesta::Transformer }
    else { ViaRespuesta::V4 }
}
```

---

### 📋 PLAN DE IMPLEMENTACIÓN (FASE B)

| # | Tarea | Archivo | Descripción |
|---|-------|---------|-------------|
| **B1** | Crear `TinyTransformer` struct + impl | `src-tauri/src/motor_transformer.rs` | Struct con pesos 32×32, forward pass sin alloc |
| **B2** | `generar_embedding()` + `inicializar_desde_grafo()` | `motor_transformer.rs` | Embedding compuesto desde nodo + sinapsis |
| **B3** | Self-attention + FFN softmax | `motor_transformer.rs` | Forward pass completo con N_max=64 |
| **B4** | Fonación V4 (`emitir_habla_emergente_v4`) | `nexus_puro_engine.rs` | V3 + umbral confianza + diversidad forzada |
| **B5** | Fusor Cognitivo (`decidir_via_respuesta`) | `nexus_puro_engine.rs` | Gating network con reglas OCEAN+Alarma+confianza |
| **B6** | Integrar en `procesar()` | `nexus_puro_engine.rs` | Vía 1 → V4, Vía 2 → Transformer, Fusor decide |
| **B7** | Comando Tauri `brain_chat_nexus_inferencia` | `main.rs` | Reemplaza `ollama_chat` + `brain_chat_stream` |
| **B8** | Eliminar `ollama_chat`, `ollama_models`, `ollama_stream` | `main.rs` | Limpieza |
| **B9** | Eliminar `brain_chat_stream` | `main.rs` | Reemplazado por inferencia híbrida |
| **B10** | Si `reqwest` solo se usa para Ollama → eliminar | `Cargo.toml` | Dependencia externa eliminada |
| **B11** | Compilar + tests 13/13 + probar chat Tauri | — | Validación final |
| **B12** | Actualizar BITACORA.md | — | Hito consolidado |

---

### 🔗 DEPENDENCIAS ENTRE TAREAS B

```
B1 ──► B2 ──► B3 ──► B6 ──► B7 ──► B8 ──► B9 ──► B10 ──► B11 ──► B12
                        ▲
B4 ──► B5 ─────────────┘
```

- B1, B2, B3: Transformer (en paralelo con B4)
- B4: Fonación V4 (en paralelo con B1-B3)
- B5: Fusor (depende de B4)
- B6: Integración (depende de B3 + B5)

---

### 📊 MÉTRICAS DE ÉXITO

| Métrica | Actual (V3) | Objetivo (Híbrido) |
|---------|-------------|-------------------|
| Longitud media respuesta | 1-3 palabras | 5-15 palabras |
| Ratio "escucho" | >50% | <10% |
| Coherencia semántica | Baja | Media-Alta |
| Latencia media | <100μs | <10ms |
| Dependencias externas | reqwest | 0 |
| Cobertura léxica | Palabras del prompt | + léxico de DB histórica |

---

### 🏗️ ESTRUCTURA DE ARCHIVOS (NUEVOS)

```
src-tauri/src/
├── main.rs                      # MODIFICADO: eliminar Ollama, agregar brain_chat_nexus_inferencia
├── nexus_puro_engine.rs          # MODIFICADO: agregar V4 + Fusor
├── motor_transformer.rs          # NUEVO: Tiny Transformer en Rust puro
└── ...
```

---

## 📋 ORDEN DE EJECUCIÓN COMPLETO (ACTUALIZADO)

| # | Fase | Tarea | Archivo | Estado |
|---|------|-------|---------|--------|
| 1-12 | C | Memoria Conversacional (completa) | main.rs + nexus_puro_engine.rs | ✅ |
| 13 | **B1** | TinyTransformer struct + forward | motor_transformer.rs (NUEVO) | ⬜ |
| 14 | **B2** | embedding + inicialización desde grafo | motor_transformer.rs | ⬜ |
| 15 | **B3** | Self-attention + FFN + softmax | motor_transformer.rs | ⬜ |
| 16 | **B4** | Fonación V4 | nexus_puro_engine.rs | ⬜ |
| 17 | **B5** | Fusor Cognitivo | nexus_puro_engine.rs | ⬜ |
| 18 | **B6** | Integrar en procesar() | nexus_puro_engine.rs | ⬜ |
| 19 | **B7** | Comando brain_chat_nexus_inferencia | main.rs | ⬜ |
| 20 | **B8** | Eliminar ollama_chat, ollama_models, ollama_stream | main.rs | ⬜ |
| 21 | **B9** | Eliminar brain_chat_stream | main.rs | ⬜ |
| 22 | **B10** | Eliminar reqwest de Cargo.toml | Cargo.toml | ⬜ |
| 23 | **B11** | Compilar + tests 13/13 + probar chat | — | ⬜ |
| 24 | **B12** | Actualizar BITACORA.md | BITACORA.md | ⬜ |
| 25 | A4 | calcular_presion() | nexus_puro_engine.rs | ⬜ |
| 26 | A1-A3 | Umbrales dinámicos | nexus_puro_engine.rs | ⬜ |
| 27 | A5 | Verificar equilibrio | — | ⬜ |
