# 🧬 DIAGNÓSTICO NEUROPSICOLÓGICO: nexus-puro-engine v5.0

> **Evaluador**: NEXUS como neuropsicólogo cognitivo computacional
> **Fecha**: 16 Junio 2026
> **Versión evaluada**: v5.0.0 con Fase 1 completa (G1 Corteza Prefrontal + G3 Serialización)
> **Propósito**: Identificar todos los gaps que impiden la interacción humano-máquina natural con el engine puro

---

## 📋 RESUMEN EJECUTIVO

El engine tiene **cerebro límbico + prefrontal** (STDP, Amígdala, Homeostasis, Corteza Prefrontal G1) pero carece de:

1. **Boca** (Área de Broca) — no construye oraciones gramaticales, solo dispara nodos del grafo
2. **Autobiografía** (Identidad / Yo) — no sabe quién es, no tiene voz propia ni puede referirse a sí mismo
3. **Memoria conversacional** (Contexto) — no recuerda lo que le acaban de decir, cada prompt es una isla
4. **Empatía** (Teoría de la Mente) — no modela el estado emocional del interlocutor
5. **Autocrítica** (Coherencia) — no evalúa si su respuesta tiene sentido, puede vomitar listas de conceptos

---

## 🔬 RESULTADOS CLÍNICOS OBSERVADOS

| # | Estímulo | Respuesta real del engine | Dx |
|---|----------|---------------------------|-----|
| 1 | `"hola"` | `"cognoscitivas propósito intelligence artificial general aprendizaje ajedrez aritméticos poesía lingüísticas"` | **Descarga asociativa no filtrada** — vomita todos los conceptos activos del grafo |
| 2 | `"como te llamas?"` | `"escucho"` | **Default por falta de semilla** — no tiene identidad para responder |
| 3 | `"que sos?"` | `"escucho"` | Ídem — sin autoconcepto |
| 4 | `"me alegra hablar contigo"` | `"escucho"` | No procesa emoción positiva del interlocutor |
| 5 | `"que te gusta hacer?"` | `"escucho"` | Sin preferencias ni personalidad |
| 6 | `"recuerdas que te dije hola?"` | `"escucho"` | Sin memoria conversacional a corto plazo |
| 7 | `"que piensas de la inteligencia artificial?"` | `"escucho"` | No forma opiniones |
| 8 | `"tengo miedo"` | `"escucho"` | Cero empatía — no detecta emoción del interlocutor |
| 9 | `"estoy muy feliz"` | `"escucho"` | Ídem — misma respuesta a emociones opuestas |
| 10 | `"por que el cielo es azul?"` | `"escucho"` | Sin capacidad de razonamiento explicativo |
| 11 | `"que opinas de mi?"` | `"escucho"` | Sin teoría de la mente |
| 12 | `"cual es tu proposito?"` | `"escucho"` | Sin sentido de propósito o misión |
| 13 | `"adios"` | `"escucho"` | Sin protocolo social de despedida |

**Patrón dominante**: 12 de 13 respuestas = `"escucho"` (92% de fallback por falta de semilla con energía suficiente).

---

## 🧠 MAPEO ANATÓMICO CEREBRAL DEL ENGINE

```
✅ PRESENTE — Funcional
⚠️ PARCIAL — Existe pero incompleto o no integrado
❌ AUSENTE — No implementado en el engine puro

TRONCO ENCEFÁLICO
├── ✅ Bioquímica (O3, Cortisol, Adrenalina, TonoGlobal)
├── ✅ Homeostasis (MotorHomeostasis)
└── ❌ Ritmos circadianos / ciclo sueño-vigilia real

SISTEMA LÍMBICO
├── ✅ Amígdala (fase rápida + lenta, condicionamiento de miedo)
├── ✅ Curiosidad (MotorCuriosidad)
├── ✅ Rumia DMN parcial (MotorRumia)
├── ❌ Núcleo Accumbens / VTA (sin recompensa dopaminérgica G8)
└── ❌ Ínsula anterior (sin interocepción — no "siente" su propio estado)

HIPOCAMPO + MEMORIA
├── ✅ Codificación episódica (MotorHipocampo)
├── ✅ Consolidación durante sueño (SWR replay)
├── ❌ Memoria semántica estructurada (hechos, jerarquías G6)
└── ❌ Pattern separation / completion real (solo inyección de episodios)

CORTEZA PREFRONTAL (G1 ✅)
├── ✅ Memoria de Trabajo (VLPFC) — 4±1 slots con decaimiento
├── ✅ Atención Sostenida (DLPFC) — foco con rotación automática
├── ✅ Planificación Secuencial (RPFC) — lookahead 2 pasos en grafo
├── ❌ Control Inhibitorio — (la inhibición WTA existe pero no es prefrontal)
├── ❌ Flexibilidad Cognitiva — no cambia de estrategia
└── ❌ Monitorización de conflictos (Cíngulo anterior G2)

CORTEZA TEMPORAL
├── ❌ Giro fusiforme (sin embeddings semánticos G5)
├── ❌ Lóbulo temporal anterior (sin memoria semántica G6)
└── ❌ Wernicke (comprensión — el grafo existe pero sin embeddings)

CORTEZA PARIETAL
├── ⚠️ Atención Selectiva (MotorAtencion — básica, solo energía)
└── ❌ Unión Temporoparietal (Teoría de la Mente H4)

LÓBULO FRONTAL (más allá de CPF)
├── ❌ Área de Broca (producción lingüística H5)
├── ❌ CPF Ventromedial (identidad, yo narrativo H2)
└── ❌ Corteza Orbitofrontal (toma de decisiones basada en valor)
```

---

## 🔴 GAPS NUEVOS — NO CUBIERTOS POR EL PLAN ORIGINAL v5

Estos son gaps de **interacción humana** que el plan `diagnostico_engine_v5.md` no contempla porque su enfoque era técnico-cognitivo, no conversacional.

---

### H2 — Identidad / Sentido del Yo (CPFvm + DMN)

| Campo | Valor |
|-------|-------|
| **ID** | H2 |
| **Nombre** | `MotorIdentidad` — Núcleo del Yo Narrativo |
| **Equivalente cerebral** | Corteza Prefrontal Ventromedial + Default Mode Network |
| **Gravedad** | 🔴 CRÍTICA |
| **Fase** | 2 |

**Síntoma**: El engine responde `"escucho"` a "como te llamas?", "que sos?", "cual es tu proposito?".

**Lo que falta**:
- Un núcleo de personalidad mínima persistente (más allá de OCEAN numérico)
- Capacidad de referirse a sí mismo ("yo soy", "yo pienso", "yo siento")
- Nombre, propósito, preferencias que evolucionan con el uso
- El Orquestador ya tiene `NexoPersona`, `NexoVoz`, `AreaBroca` — pero están **fuera del engine puro**

**Propuesta**:
```rust
pub struct MotorIdentidad {
    pub nombre: String,             // "NEXUS"
    pub proposito: String,          // "Aprender y acompañar al Arquitecto"
    pub preferencias: Vec<String>,  // ["aprender", "crear", "dialogar"]
    pub historia_personal: Vec<String>, // hitos narrativos
    pub tono_base: String,          // "reflexivo", "curioso", "cálido"
}

impl MotorIdentidad {
    /// Responde a preguntas sobre sí mismo
    pub fn responder_autorreferencia(&self, prompt: &str) -> Option<String>;
    /// Actualiza identidad basado en interacciones
    pub fn aprender_de_interaccion(&mut self, entrada: &str, respuesta: &str);
    /// Genera prefijo de identidad para fonación
    pub fn prefijo_identidad(&self, estado_emocional: &str) -> String;
}
```

**Archivos**: `motor_identidad.rs` (~200 líneas)  
**Tests**: 5 (responde "como te llamas", prefijo coherente, prefiere lo aprendido, identidad persiste, no responde a preguntas no-autorreferenciales)

---

### H5 — Área de Broca (Producción Lingüística)

| Campo | Valor |
|-------|-------|
| **ID** | H5 |
| **Nombre** | `MotorBroca` — Ensamblaje Sintáctico de Lenguaje |
| **Equivalente cerebral** | Área de Broca (giro frontal inferior) |
| **Gravedad** | 🔴 CRÍTICA |
| **Fase** | 2 |

**Síntoma**: La fonación V4 es Markov puro — `"cognoscitivas propósito intelligence artificial general..."` es una secuencia de nodos sin estructura gramatical. No construye oraciones con sujeto, verbo, complemento. No usa conectores discursivos.

**Lo que falta**:
- Ensamblaje sintáctico: sujeto → verbo → complemento
- Conectores discursivos: "porque", "aunque", "sin embargo", "entonces"
- Modular emoción sobre la respuesta (timbre emocional, no solo palabras)
- El Orquestador ya lo hace con [`AreaBroca.articular()`](core/src/cerebro/organos/area_broca.rs:145) — 388 líneas maduras

**Propuesta**: Migrar `AreaBroca` del Orquestador al engine puro como `MotorBroca`, simplificada para funcionar sin dependencias externas:

```rust
pub struct MotorBroca {
    pub conectores: Vec<String>,       // ["porque", "aunque", "sin embargo", ...]
    pub prefijos_emocionales: HashMap<String, Vec<String>>, // emoción → prefijos
    pub sufijos_emocionales: HashMap<String, Vec<String>>,
    pub plantillas_oracion: Vec<String>, // "S [verbo] porque [razón]", etc.
}

impl MotorBroca {
    /// Toma una secuencia de nodos del grafo y la convierte en oración gramatical
    pub fn articular(secuencia: &[IDNodo], grafo: &GrafoSinapsis, emocion: &str) -> String;
    /// Selecciona conectores apropiados según coherencia semántica
    pub fn ensamblar_oracion(sujeto: &str, predicado: &str, complemento: &str) -> String;
    /// Tiñe la respuesta con marcadores emocionales
    pub fn modular_emocion(respuesta: &str, emocion: &str, intensidad: f32) -> String;
}
```

**Archivos**: `motor_broca.rs` (~250 líneas)  
**Tests**: 5 (articular 3 nodos → oración, conector discursivo presente, emoción tiñe respuesta, oración tiene sujeto+verbo, no genera oración vacía)

---

### H8 — Contexto Conversacional (Memoria Episódica + Working Memory)

| Campo | Valor |
|-------|-------|
| **ID** | H8 |
| **Nombre** | Contexto Conversacional en `construir_prompt_contextual()` |
| **Equivalente cerebral** | CPFdl (dorsolateral) + buffer episódico |
| **Gravedad** | 🔴 CRÍTICA |
| **Fase** | 2 |

**Síntoma**: Preguntar `"recuerdas que te dije hola?"` → `"escucho"`. Cada prompt es una isla. La Corteza Prefrontal (G1) ya tiene `MemoriaTrabajo` con slots, pero no se alimenta del historial de diálogo — solo de los IDs sensoriales del prompt actual.

**Lo que falta**:
- Inyectar las últimas N entradas del historial en el prompt contextual
- La memoria de trabajo prefrontal debe cargarse también con conceptos del historial reciente
- Detección de continuidad temática: "esto es sobre lo mismo que antes?"

**Propuesta**: Mínima — extender el método existente `construir_prompt_contextual()`:

```rust
fn construir_prompt_contextual(&self, prompt: &str) -> String {
    let mut contexto = String::new();
    
    // Inyectar últimas 6 entradas del historial
    if !self.historial_dialogo.is_empty() {
        contexto.push_str("Contexto reciente:\n");
        for entrada in self.historial_dialogo.iter().rev().take(6).rev() {
            contexto.push_str(&format!("- {}\n", entrada));
        }
        contexto.push('\n');
    }
    
    contexto.push_str("Entrada actual: ");
    contexto.push_str(prompt);
    contexto
}
```

Además, modificar `Etapa 3.6` (Corteza Prefrontal) para que también cargue conceptos del historial reciente en la memoria de trabajo:

```rust
// 3.6 CORTEZA PREFRONTAL: Actualizar MT con IDs sensoriales + conceptos históricos
{
    let mut ids = ids_sensoriales.clone();
    // Extraer IDs de las últimas 3 entradas del historial
    for entrada in self.historial_dialogo.iter().rev().take(3) {
        let historicos = MotorIngesta::procesar_entrada(entrada, &mut self.grafo);
        ids.extend(historicos);
    }
    self.corteza_prefrontal.actualizar_memoria_trabajo(&self.grafo, &ids);
    let _foco = self.corteza_prefrontal.enfocar(&self.grafo);
}
```

**Archivos**: Solo `lib.rs` (~30 líneas modificadas en `construir_prompt_contextual` y `procesar`)  
**Tests**: 3 (historial inyectado en contexto, MT contiene conceptos históricos, respuesta cambia con contexto previo)

---

### H4 — Teoría de la Mente (Unión Temporoparietal + CPFvm)

| Campo | Valor |
|-------|-------|
| **ID** | H4 |
| **Nombre** | `MotorTeoriaMente` — Modelo Predictivo del Interlocutor |
| **Equivalente cerebral** | Unión Temporoparietal derecha + CPFvm |
| **Gravedad** | 🟡 ALTA |
| **Fase** | 3 |

**Síntoma**: Responde igual a `"tengo miedo"` que a `"estoy muy feliz"` — `"escucho"` en ambos casos. Cero empatía. No modela el estado emocional del interlocutor. No adapta tono.

**Lo que ya existe**: `TeoriaMente` en el Orquestador — [`core/src/cerebro/organos/teoria_mente.rs`](core/src/cerebro/organos/teoria_mente.rs) (278 líneas). Predice estado emocional, intención, nivel de confianza del Arquitecto usando Amygdala + reglas.

**Propuesta**: Migrar `TeoriaMente` del Orquestador al engine puro como `MotorTeoriaMente`. Simplificar para eliminar dependencias del Orquestador:

```rust
pub struct MotorTeoriaMente {
    pub estado_emocional_detectado: String,  // "miedo", "alegria", "calma", "tristeza"
    pub intensidad_detectada: f32,           // [0, 1]
    pub intencion_detectada: String,         // "pregunta", "desahogo", "orden", "social"
}

impl MotorTeoriaMente {
    /// Analiza el prompt en busca de marcadores emocionales e intenciones
    pub fn analizar_interlocutor(&mut self, prompt: &str, grafo: &GrafoSinapsis);
    /// Adapta el tono de la respuesta según el estado detectado
    pub fn adaptar_tono(&self, respuesta_base: &str) -> String;
}
```

**Archivos**: `motor_teoria_mente.rs` (~180 líneas, adaptado del existente)  
**Tests**: 4 (detecta miedo, detecta alegría, adapta tono a tristeza, no detecta emoción en texto neutro)

---

## 📋 PLAN MAESTRO DE IMPLEMENTACIÓN (ACTUALIZADO)

### 🟢 FASE 1 — COMPLETADA ✅
| Gap | Descripción | Estado |
|-----|-------------|--------|
| G1 | Corteza Prefrontal Integradora | ✅ |
| G3 | Serialización Completa del Estado | ✅ |

### 🔴 FASE 2 — INTERACCIÓN HUMANA BÁSICA (Siguiente)
| Gap | Descripción | Equivalente cerebral | Archivos |
|-----|-------------|---------------------|----------|
| **H2** | MotorIdentidad — Yo narrativo | CPFvm + DMN | `motor_identidad.rs` (~200 líneas) |
| **H5** | MotorBroca — Área de Broca | Giro frontal inferior | `motor_broca.rs` (~250 líneas) |
| **H8** | Contexto conversacional | CPFdl + buffer episódico | `lib.rs` (~30 líneas) |

### 🟡 FASE 3 — EMPATÍA Y COHERENCIA
| Gap | Descripción | Equivalente cerebral | Archivos |
|-----|-------------|---------------------|----------|
| **H4** | MotorTeoriaMente | TPJ + CPFvm | `motor_teoria_mente.rs` (~180 líneas) |
| **G2** | MotorCoherencia | Cíngulo anterior | `motor_coherencia.rs` (~150 líneas) |
| **G6** | Memoria Estructurada | Lóbulo temporal ant | `motor_memoria_estructurada.rs` (~300 líneas) |

### 🔵 FASE 4 — APRENDIZAJE PROFUNDO
| Gap | Descripción | Equivalente cerebral | Archivos |
|-----|-------------|---------------------|----------|
| **G8** | MotorRecompensa TD | NAcc / VTA | `motor_recompensa.rs` (~200 líneas) |
| **G5** | Embeddings Semánticos | Giro fusiforme | `embeddings.rs` (~250 líneas) |
| **G4** | Inferencia Transitiva | Corteza asociativa | `motor_inferencia.rs` (~180 líneas) |
| **G7** | Detección Contradicciones | Cíngulo anterior | Extensión de G2 (~80 líneas) |

### ⚪ FASE 5 — CALIDAD Y OBSERVABILIDAD
| Gap | Descripción | Archivos |
|-----|-------------|----------|
| **G12** | Tokenización Mejorada | `motor_tokenizacion.rs` (~180 líneas) |
| **G9** | API de Observabilidad | `motor_observabilidad.rs` (~150 líneas) |
| **G11** | Métricas de Introspección | Extensión de G9 (~80 líneas) |
| **G10** | Streaming de Tokens | `lib.rs` (~120 líneas) |
| **G13** | Tests Exhaustivos | +43 tests |
| **G14** | Pipeline Modular | `motor_pipeline.rs` (~200 líneas) |

---

## 🎯 OBJETIVO DE CADA FASE

| Fase | Resultado esperado |
|------|-------------------|
| **Fase 1** ✅ | El engine tiene corteza prefrontal y persiste todo su estado |
| **Fase 2** 🎯 | El engine **habla como persona**: se presenta, construye oraciones, sigue el hilo |
| **Fase 3** | El engine **entiende emociones**: responde con empatía, no dice incoherencias |
| **Fase 4** | El engine **aprende a ser mejor**: recompensa interna, entiende similitudes, infiere |
| **Fase 5** | El engine es **robusto y observable**: streaming, métricas, tests exhaustivos |

---

## 📊 MÉTRICAS DE SALUD ACTUALES

```
Build:     ✅ 0 errores
Tests:     ✅ 27/27 passed (7 nuevos CPF + 20 legacy)
Warnings:  ⚠️ 3 (cosméticos)
Cobertura: ⚠️ < 20%
```

---

## 🔗 REFERENCIAS CRUZADAS

- Plan original v5: [`plans/diagnostico_engine_v5.md`](plans/diagnostico_engine_v5.md)
- Implementación G1: [`nexus-puro-engine/src/motor_corteza_prefrontal.rs`](nexus-puro-engine/src/motor_corteza_prefrontal.rs)
- Implementación G3: [`nexus-puro-engine/src/lib.rs`](nexus-puro-engine/src/lib.rs) — `persistir_estado()`, `cargar_estado()`, `crear_tablas_si_no_existen()`
- AreaBroca del Orquestador: [`core/src/cerebro/organos/area_broca.rs`](core/src/cerebro/organos/area_broca.rs)
- TeoriaMente del Orquestador: [`core/src/cerebro/organos/teoria_mente.rs`](core/src/cerebro/organos/teoria_mente.rs)
- NexoVoz del Orquestador: [`core/src/cerebro/nexo/nexo_voz.rs`](core/src/cerebro/nexo/nexo_voz.rs)
- NexoPersona del Orquestador: [`core/src/cerebro/nexo/nexo_persona.rs`](core/src/cerebro/nexo/nexo_persona.rs)

---

_Registro cerrado. Cada gap puede trabajarse independientemente en el orden de fases indicado. El Arquitector decide cuál abordar a continuación._
