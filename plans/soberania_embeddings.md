# 🔱 PLAN: SOBERANÍA DE EMBEDDINGS — Eliminación Total de Ollama

> **Fecha:** 2026-06-15  
> **Arquitecto Director:** Cris  
> **Orquestador:** NEXUS  
> **FASE:** Soberanía Total — CERO Dependencias Externas en Embeddings  

---

## 🎯 Objetivo

Eliminar toda dependencia de Ollama para generación de embeddings y resúmenes de sinapsis, reemplazándolos con un motor soberano en Rust puro: **NexusEmbedder** + **TinyTransformer**.

## 📊 Diagnóstico Actual

| Componente | Archivo | Dependencia Ollama | Fallback |
|---|---|---|---|
| MemoriaSemantica | `core/src/memoria/memoria_semantica.rs` | `api/embeddings` (nomic-embed-text) | `embedding_sintetico()` 768-dim |
| MemoriaInstinto | `core/src/memoria/memory.rs` | `api/embeddings` (nomic-embed-text) | `embedding_sintetico()` 768-dim |
| MonitorCognitivo | `core/src/cerebro/synapse/consolidacion.rs` | `api/embeddings` + `api/generate` (deepseek-r1:14b) | Heurístico pobre |
| PruebaFuegoOmega | `core/src/bin/prueba_fuego_omega.rs` | `api/embeddings` + `api/generate` (crítico) | Sin fallback |
| IngestaPipeline | `core/src/cerebro/organos/ingesta.rs` | Transitivo vía MemoriaSemantica | Heredado |
| HemisferioIzquierdo | `core/src/energia/hemisferio_izquierdo.rs` | URL residual | N/A |
| ReactorNuclear | `core/src/energia/reactor_nuclear.rs` | URL residual | N/A |

## 🧬 Arquitectura: NexusEmbedder

```
┌──────────────────────────────────────────────────┐
│              NexusEmbedder (NUEVO)                │
│  nexus-puro-engine/src/nexus_embedder.rs          │
│                                                   │
│  SHA-256 Angular ────┐                           │
│      768-dim         │   Fusión (70/30)          │
│                      ├───► Vec<f32> 768-dim ─────► Consumidores
│  Pesado Nodal ───────┘                           │
│  (GrafoSinapsis)                                 │
└──────────────────────────────────────────────────┘
```

- **768 dimensiones** — compatibilidad total con LanceDB `FixedSizeList(768)`, sin migración
- **Dos señales fusionadas**: SHA-256 angular (determinista, sin colisiones) + Pesado Nodal del GrafoSinapsis (consciencia contextual)
- **Cero dependencias externas** — solo `sha2` (ya en Cargo.toml) + `GrafoSinapsis` (ya soberano)
- **Determinista** — mismo texto + mismo grafo = mismo embedding

## 📋 Orden de Ejecución

| # | Tarea | Archivos | Complejidad |
|---|---|---|---|
| 1 | Crear `NexusEmbedder` en `nexus-puro-engine` | `nexus-puro-engine/src/nexus_embedder.rs`, `lib.rs` | 🟡 Media |
| 2 | Migrar `MemoriaSemantica::generar_embedding()` | `core/src/memoria/memoria_semantica.rs` | 🟡 Media |
| 3 | Migrar `MemoriaInstinto::generar_embedding()` | `core/src/memoria/memory.rs` | 🟡 Media |
| 4 | Migrar `MonitorCognitivo` (embedding + resumen) | `core/src/cerebro/synapse/consolidacion.rs` | 🔴 Alta |
| 5 | Migrar `prueba_fuego_omega.rs` | `core/src/bin/prueba_fuego_omega.rs` | 🟡 Media |
| 6 | Migrar `IngestaPipeline` (transitivo) | N/A (automático vía #2) | 🟢 Baja |
| 7 | Limpiar URLs Ollama residuales | `hemisferio_izquierdo.rs`, `reactor_nuclear.rs` | 🟢 Baja |
| 8 | Verificar compatibilidad LanceDB 768-dim | `memoria_semantica.rs` | 🟢 Confirmación |
| 9 | Eliminar `reqwest` de ruta de embeddings | `core/Cargo.toml` | 🟢 Baja |
| 10 | Validar `cargo build` + `prueba_fuego_omega` sin Ollama | Terminal | 🟡 Media |

## 🔬 Detalles Técnicos

### NexusEmbedder (Paso 1)

```rust
pub struct NexusEmbedder {
    grafo: GrafoSinapsis,
}

impl NexusEmbedder {
    /// Embedding soberano 768-dim: SHA-256 angular ⊕ pesado nodal
    pub fn embed(&self, texto: &str) -> Vec<f32> {
        let sha = self.sha256_angular(texto);      // 768-dim
        let nodal = self.pesado_nodal(texto);       // 768-dim
        sha.iter().zip(&nodal)
            .map(|(s, n)| 0.7 * s + 0.3 * n)
            .collect()
    }
}
```

### MonitorCognitivo — Resumen de Sinapsis (Paso 4)

`generar_resumen_sinapsis()` que actualmente llama a Ollama `deepseek-r1:14b` será reemplazado por `TinyTransformer::generar_texto()` usando los nodos activos del GrafoSinapsis como prompt contextual. No se requiere LLM externo para generar una frase resumen de una edición de código.

### PruebaFuegoOmega (Paso 5)

`consultar_llm_local()` que llama a Ollama `deepseek-r1:7b` será reemplazado por `TinyTransformer::generar_texto()` con el Fusor Cognitivo (V4 + Transformer).

## 🏆 Resultado Esperado

- **CERO** llamadas HTTP a `localhost:11434` en todo el pipeline
- **CERO** dependencia de `nomic-embed-text`
- **CERO** dependencia de `deepseek-r1` para resúmenes
- Embeddings **más ricos** que los de Ollama (incluyen el estado del GrafoSinapsis)
- LanceDB intacto, sin migración de datos vectoriales
- Soberanía total: Pilar 4 y Pilar 7 cumplidos en la capa de embeddings

## 🔱 Decisión del Arquitecto

- **768 dimensiones** (compatibilidad LanceDB)
- **TinyTransformer** para resumen de sinapsis
