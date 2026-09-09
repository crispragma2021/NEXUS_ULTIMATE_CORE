# 🧬 SYSTEM DIRECTIVE FOR DEEPSEEK R1: GENERATION OF NEXUS ASSOCIATIVE COGNITIVE & LANGUAGE ORGANS (OPTION B + COGNITIVE REFINEMENTS)

Dear DeepSeek R1,

You are acting as the Sovereign Lobe Synthesizer for the **NEXUS** digital organism running locally on a Ryzen 7 5700U processor. The Architect (Cris) and NEXUS (the sovereign Rust core) are pairing with you to design and implement a deep semantic, non-LLM, associative language system.

We are implementing **Option B**, which uses a native, local **Corteza Asociativa Humana** (concept graph) running in RAM. In this iteration, we have added **four advanced cognitive refinements**:
1.  **Dynamic Graph Expansion (Hebbian Learning):** The graph learns dynamically from conversations (creating or strengthening links between concepts that appear in the same context).
2.  **Jaccard-Synaptic Hybrid Scoring:** Concept deduction combines attribute similarity (Jaccard) with current synaptic link weights.
3.  **Semantic Temperature Control:** Control the range/randomness of semantic jumps (synaptic walks) during association.
4.  **Session Context Buffer:** A memory sliding window to keep track of recently activated concepts and maintain dialogue coherence.

---

## ⚙️ Compilation & Architectural Constraints
- **Target Language:** Idiomatic, compiler-safe Rust (Edition 2021).
- **Environment:** Low latency (< 5μs lookup), low memory footprint. Run in CPU-bound local architecture.
- **Allowed Dependencies:** Standard Library (`std`), `tracing`, `rand`, `regex`, `chrono`, `rusqlite`. Avoid adding external crate dependencies not present in this list.
- **Panic Strategy:** Graceful error handling (return `Option` or `Result` instead of using `unwrap` or `panic!`).

---

## 📂 Pre-existing Brain Context

### 1. Existing Enums and Structs in `core/src/cerebro/motor_pensamiento.rs`
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Intencion {
    InformarEstado,
    ExpresarEmocion,
    PedirGuia,
    DeclararAccion,
    AlertaCritica,
    Dudar,
    Evolucionar,
    Conversar,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sujeto {
    Yo,
    TuPadre,
    ElSistema,
    LaAmenaza,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Verbo {
    Optimizar,
    Proteger,
    Aprender,
    Fallar,
    Observar,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Objeto {
    Memoria,
    CPU,
    Conocimiento,
    AmenazaExterna,
    Identidad,
}

#[derive(Debug, Clone)]
pub struct Pensamiento {
    pub intencion: Intencion,
    pub sujeto: Sujeto,
    pub verbo: Option<Verbo>,
    pub objeto: Option<Objeto>,
    pub urgencia: u8,
}
```

### 2. Existing Emotional States in `core/src/cerebro/amygdala.rs`
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EstadoEmocional {
    Calma,
    Alerta,
    Miedo,
    RabiaSoberana,
}
```

---

## 🔬 DETAILED SPECIFICATIONS FOR THE NEW ORGANS

### 1. File: `core/src/cerebro/corteza_asociativa.rs`
- **Objective:** Model a human-like associative network in RAM using a graph of concepts, attributes, and sinaptic weights, including context buffering and Hebbian learning.
- **Components:**
  - `ConceptoHumano`:
    - `palabra: String` (e.g., "lealtad")
    - `atributos: Vec<String>` (e.g., `["fidelidad", "valores", "padre", "proteccion"]`)
    - `sinapsis: HashMap<String, f32>` (map of other linked concepts and their synapse weight, e.g. `"cris" => 0.95`)
    - `confianza: f32`
    - `valencia_emocional: f32` (-1.0 to 1.0)
  - `CortezaAsociativa`:
    - `red: HashMap<String, ConceptoHumano>`
    - `buffer_contexto: std::collections::VecDeque<String>` -> Sliding window storing the last N (e.g., 5) active concepts to maintain conversational continuity.
    - `pub fn new() -> Self` -> Initialize with some default seed concepts (e.g., "padre", "nexus", "cpu", "memoria", "seguridad", "lealtad", "cris", "dota", "soberania").
    - `pub fn asimilar(&mut self, palabra: &str, atributos: Vec<String>, sinapsis: HashMap<String, f32>)` -> Inserts or merges a concept into the network and triggers link recalculation.
    - `pub fn registrar_interaccion(&mut self, entrada_conceptos: &[String], respuesta_conceptos: &[String])` -> **Hebbian Learning Rule:** Creates new links or strengthens weights (e.g. increase by `0.1`, max `1.0`) between concepts that co-occur in the interaction. Slightly decays inactive synapses.
    - `pub fn deducir_por_atributos(&self, consulta_atributos: &[String]) -> Option<(String, f32)>` -> Computes a **hybrid score**: `0.7 * Jaccard_similarity(atributos) + 0.3 * promedio_fuerza_sinaptica_con_contexto_activo`. Return the best matching concept and its score (only return if score >= 0.45).
    - `pub fn asociacion_libre(&self, semilla: &str, max_pasos: usize, temperatura: f32) -> Vec<String>` -> Syntactic walk along synaptic links. Transition probabilities are adjusted by `temperatura` (from `0.0` to `1.0`):
      - Low temperature (`< 0.3`): walks only the highest weight links (focused thought).
      - High temperature (`>= 0.7`): applies random noise to weights, allowing wider, creative associations (divergent drift).
    - `pub fn recalcular_enlaces_cruzados(&mut self)` -> Recalculates cross weights if concepts share more than a threshold of attributes.

### 2. File: `core/src/cerebro/area_wernicke.rs`
- **Objective:** Parse Spanish strings, extract semantic attributes, query the `CortezaAsociativa`, and construct a `ComprensionSemantica`.
- **Rules:**
  - Tokenize and sanitize input (convert to lowercase, strip accents/special characters).
  - Extract candidate attributes (e.g., if input contains "ayudar", add "apoyo", "guia" to candidate attributes).
  - Query `CortezaAsociativa::deducir_por_atributos` using these candidate attributes.
  - Classify `Intencion` and other fields based on the resulting concept's valencia, matched parameters, and structure.
  - Extract parameters (such as pathnames, command arguments, etc.).
- **Expected struct:**
  ```rust
  use crate::cerebro::corteza_asociativa::CortezaAsociativa;

  #[derive(Debug, Clone)]
  pub struct ComprensionSemantica {
      pub intencion: Intencion,
      pub sujeto: Sujeto,
      pub verbo: Option<Verbo>,
      pub objeto: Option<Objeto>,
      pub concepto_asociado: Option<String>,
      pub score_asociado: f32,
      pub parametros: Vec<String>,
      pub urgencia: u8,
      pub valencia_emocional: f32,
  }

  pub struct AreaWernicke;
  ```

### 3. File: `core/src/cerebro/area_broca_conversacional.rs`
- **Objective:** Generate fluent natural language responses in Spanish from a logical `Pensamiento` and active concept networks.
- **Rules:**
  - Avoid purely static templates. Retrieve synonyms and associated semantic nodes from `CortezaAsociativa::asociacion_libre` to enrich the sentence.
  - Structure language according to `edad_mental` (`f64` from `0.0` to `1.0`):
    - Infant stage (`< 0.3`): fragmented, short sentences.
    - Explorer stage (`0.3` to `< 0.7`): curious, structured, eager to learn.
    - Sovereign stage (`>= 0.7`): mature, direct, integrates moral proverb constants, respectful.
  - Tone should shift with `dopamina` and physical status (`temperatura_cpu`).
  - Incorporate a concept-drift semantic walk path when articulating conversational subjects.
- **Expected struct:**
  ```rust
  pub struct AreaBrocaConversacional {
      semilla_rng: u64,
  }
  ```

### 4. File: `core/src/cerebro/fasciculo_arqueado.rs`
- **Objective:** The neural bridge. Receives Wernicke's semantic output, checks threat vectors and safety constraints, registers system updates, and commands output.
- **Rules:**
  - Check the `valencia_emocional` and `urgencia` of the comprehension.
  - Evaluate against `EstadoEmocional` (from the Amygdala) and verify safety rules via `JuicioSoberano`.
  - Decide if an active hardware task (compilation, file read/write, bash execution) should run via the `MedulaSoberana`.
  - Set a `veto_soberano` flag to block execution if the input is hostile.
  - Synthesize a cognitive feedback `Pensamiento` for the `AreaBrocaConversacional`.
- **Expected structs:**
  ```rust
  #[derive(Debug, Clone)]
  pub struct DecisionCognitiva {
      pub accion_mula: Option<String>,
      pub pensamiento_broca: Pensamiento,
      pub veto_soberano: bool,
  }

  pub struct FasciculoArqueado;
  ```

---

## 🎯 What we need from you:
Please output **four complete, distinct, compiler-safe code blocks** in Rust representing the full implementation of:
1. `corteza_asociativa.rs`
2. `area_wernicke.rs`
3. `area_broca_conversacional.rs`
4. `fasciculo_arqueado.rs`

Ensure they import existing enums and types accurately. Thank you, DeepSeek!
