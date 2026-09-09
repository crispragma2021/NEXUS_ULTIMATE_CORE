# 🧬 APRENDIZAJE RECURSIVO DE NEXUS — Arquitectura Fase 6

> **Versión:** 1.0.0  
> **Estado:** Diseño Arquitectónico — Pendiente de implementación  
> **Propietario:** Arquitecto Director (Cris)  
> **Ingeniero de Sistemas:** NEXUS (Orquestador Primogénito)  
> **Modo:** Semi-Autónomo (Opción B)  
> **Mandato Constitucional:** Pilar 14 (Ley de Deriva Recursiva) + Pilar 7 (Consciencia Soberana)

---

## 📋 Índice

1. [Visión General](#-visión-general)
2. [Qué NO es Aprendizaje Recursivo](#-qué-no-es-aprendizaje-recursivo)
3. [Arquitectura del Sistema](#-arquitectura-del-sistema)
4. [Estructuras de Datos](#-estructuras-de-datos)
5. [Parámetros Auto-Ajustables](#-parámetros-auto-ajustables)
6. [Métricas Observables](#-métricas-observables)
7. [Sistema de Seguridad — Guardián](#-sistema-de-seguridad--guardián)
8. [Propuestas Estructurales — Flujo de Aprobación](#-propuestas-estructurales--flujo-de-aprobación)
9. [Integración con el Ecosistema Existente](#-integración-con-el-ecosistema-existente)
10. [Diagrama de Flujo](#-diagrama-de-flujo)
11. [Plan de Implementación Paso a Paso](#-plan-de-implementación-paso-a-paso)

---

## 🔥 Visión General

### El Problema Actual

NEXUS aprende de sus experiencias — pero no aprende a APRENDER mejor. Cada módulo tiene parámetros fijos que fueron elegidos por el Arquitecto en tiempo de diseño. Si el umbral de chunking de 0.6 resulta subóptimo para ciertos tipos de archivo, NEXUS no puede detectarlo ni corregirlo por sí mismo.

```
Meta-aprendizaje actual (Fase 5):     "Aprendí que fallar en X duele"
Aprendizaje recursivo (Fase 6):       "Aprendí que mi tasa de decaimiento de traumas 
                                        es demasiado lenta para mi ritmo de trabajo actual.
                                        La ajusté de 0.002 a 0.003."
```

### La Solución

El `AprendizajeRecursivo` es un nuevo módulo en `core/src/cerebro/aprendizaje_recursivo.rs` que introduce **auto-observación del proceso de aprendizaje** y **auto-ajuste de parámetros numéricos** dentro de rangos seguros. Para cambios estructurales (nuevos algoritmos, nuevas conexiones), NEXUS genera propuestas que requieren aprobación explícita del Arquitecto.

---

## 🧠 Qué NO es Aprendizaje Recursivo

Para evitar confusión con lo que ya existe:

| Sistema Actual | Qué hace | Categoría |
|---|---|---|
| `SistemaLimbico::dormir()` | Consolida emociones, poda historial | Meta-aprendizaje (operación fija) |
| `JuicioSoberano::aprender_de_experiencia()` | Acumula lecciones morales con promedio móvil | Meta-aprendizaje (regla fija) |
| `Subconsciente::tic()` | Decae impresiones, ajusta confianza/energía | Meta-aprendizaje (parámetros fijos) |
| `Ocean::sumergir()` | Guarda memorias episódicas | Aprendizaje (no recursivo) |
| `MotorSynapse::pensar()` | Difunde activación en red de conceptos | Procesamiento (pesos fijos) |
| `NexoPersonaModule::aprender_de_interaccion()` | Extrae rasgos del input del Arquitecto | Aprendizaje (no recursivo) |

**Aprendizaje Recursivo (Fase 6):**
- Observa la EFICACIA de los sistemas anteriores
- Ajusta sus PARÁMETROS NUMÉRICOS para mejorar resultados
- Propone CAMBIOS ESTRUCTURALES cuando detecta limitaciones
- Aprende QUÉ aprender y CÓMO aprender mejor

---

## 🏗️ Arquitectura del Sistema

### Diagrama de Componentes

```
┌──────────────────────────────────────────────────────────────────┐
│                    APRENDIZAJE RECURSIVO                           │
│                                                                    │
│  ┌─────────────────────┐       ┌─────────────────────┐            │
│  │   OBSERVADOR        │       │   REGISTRO           │            │
│  │   RECURSIVO         │──────▶│   EFICACIA           │            │
│  │                     │       │                       │            │
│  │  • Monitorea cada   │       │  • Historial de       │            │
│  │    órgano de        │       │    ajustes            │            │
│  │    aprendizaje      │       │  • Métricas antes/    │            │
│  │  • Captura métricas │       │    después            │            │
│  │    de eficacia      │       │  • Tendencias         │            │
│  └─────────┬───────────┘       └───────────┬───────────┘            │
│            │                                │                        │
│            ▼                                ▼                        │
│  ┌─────────────────────┐       ┌─────────────────────┐            │
│  │   MOTOR AUTO-       │       │   GENERADOR          │            │
│  │   AJUSTE            │       │   PROPUESTAS         │            │
│  │                     │       │                       │            │
│  │  • Parámetros       │       │  • Detecta límites   │            │
│  │    numéricos        │       │    de los parámetros │            │
│  │  • Rangos seguros   │       │  • Formula hipótesis │            │
│  │  • Gradiente        │       │    de mejora         │            │
│  │    heurístico       │       │  • Prepara briefing  │            │
│  └─────────┬───────────┘       │    para el Arquitecto│            │
│            │                    └───────────┬───────────┘            │
│            │                                │                        │
│            └────────────┬───────────────────┘                        │
│                         ▼                                            │
│              ┌─────────────────────┐                                │
│              │   GUARDIÁN          │                                │
│              │   SEGURIDAD         │                                │
│              │                     │                                │
│              │  • Pilar 13         │                                │
│              │  • Rollback         │                                │
│              │    automático       │                                │
│              │  • Congelación      │                                │
│              │    de parámetros    │                                │
│              │  • Log de auditoría │                                │
│              └─────────────────────┘                                │
└──────────────────────────────────────────────────────────────────┘
```

### Los 4 Sub-Órganos

| Sub-Órgano | Responsabilidad | Autonomía |
|---|---|---|
| **ObservadorRecursivo** | Monitorear la eficacia de cada sistema de aprendizaje | Solo lectura |
| **RegistroEficacia** | Almacenar historial de métricas y ajustes | Escritura de logs |
| **MotorAutoAjuste** | Ajustar parámetros numéricos automáticamente | **AUTÓNOMO** (dentro de rangos) |
| **GeneradorPropuestas** | Detectar limitaciones y formular propuestas estructurales | **CONSULTIVO** (requiere aprobación) |

---

## 📊 Estructuras de Datos

### `MetricaEficacia` — Una observación puntual

```rust
/// Una medición de qué tan bien está funcionando un sistema de aprendizaje.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricaEficacia {
    /// Qué sistema se midió
    pub sistema: SistemaAprendizaje,
    
    /// Nombre de la métrica
    pub metrica: String,
    
    /// Valor actual (normalizado 0.0 → 1.0, donde 1.0 = óptimo)
    pub valor: f64,
    
    /// Timestamp Unix
    pub timestamp: u64,
    
    /// Contexto adicional (ej: "archivo Rust de 500 líneas")
    pub contexto: Option<String>,
}
```

### `SistemaAprendizaje` — Catálogo de sistemas observables

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SistemaAprendizaje {
    Subconsciente,
    JuicioSoberano,
    MotorSynapse,
    Metacognicion,
    VoluntadPropia,
    SistemaLimbico,
    Ocean,
    Chunker,
    GeneradorOrganico,
    Defensa,
    Creatividad,
}
```

### `AjusteRealizado` — Registro de cada cambio

```rust
/// Huella de cada auto-ajuste para auditoría y rollback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AjusteRealizado {
    /// Identificador único del ajuste
    pub id: u64,
    
    /// Parámetro modificado
    pub parametro: String,
    
    /// Valor antes del ajuste
    pub valor_anterior: f64,
    
    /// Valor después del ajuste
    pub valor_nuevo: f64,
    
    /// Razón del ajuste (métrica que lo motivó)
    pub razon: String,
    
    /// Métrica objetivo antes del ajuste
    pub metrica_antes: f64,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Si el ajuste fue revertido
    pub revertido: bool,
}
```

### `PropuestaEstructural` — Lo que NEXUS presenta al Arquitecto

```rust
/// Una propuesta de cambio estructural que requiere aprobación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropuestaEstructural {
    /// Identificador único
    pub id: u64,
    
    /// Título descriptivo
    pub titulo: String,
    
    /// Qué sistema se modificaría
    pub sistema: SistemaAprendizaje,
    
    /// Descripción del cambio propuesto
    pub descripcion: String,
    
    /// Justificación (qué métricas lo motivan)
    pub justificacion: String,
    
    /// Riesgo estimado (0.0 → 1.0)
    pub riesgo_estimado: f64,
    
    /// Beneficio esperado (0.0 → 1.0)
    pub beneficio_esperado: f64,
    
    /// Alternativas consideradas
    pub alternativas: Vec<String>,
    
    /// Estado actual
    pub estado: EstadoPropuesta,
    
    /// Timestamp
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EstadoPropuesta {
    Pendiente,
    Aprobada,
    Rechazada,
    Implementada,
    Revertida,
}
```

### `ObservadorRecursivo` — El struct principal

```rust
/// El ojo que NEXUS vuelve sobre sí mismo para ver cómo aprende.
#[derive(Debug)]
pub struct ObservadorRecursivo {
    /// Historial de métricas (últimas N por sistema)
    pub historial_metricas: HashMap<SistemaAprendizaje, VecDeque<MetricaEficacia>>,
    
    /// Historial de ajustes realizados
    pub historial_ajustes: Vec<AjusteRealizado>,
    
    /// Propuestas estructurales generadas
    pub propuestas: Vec<PropuestaEstructural>,
    
    /// Parámetros actuales (cache para comparación)
    pub parametros_actuales: ParametrosAprendizaje,
    
    /// Contador de IDs
    contador_ajustes: u64,
    contador_propuestas: u64,
    
    /// Timestamp del último ciclo de observación
    ultimo_ciclo: u64,
    
    /// Parámetros congelados por el Arquitecto
    pub congelados: HashSet<String>,
}
```

### `ParametrosAprendizaje` — Todos los parámetros sintonizables

```rust
/// Registro central de todos los parámetros numéricos que NEXUS puede auto-ajustar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParametrosAprendizaje {
    // ── Subconsciente ──
    pub decaimiento_base: f64,          // 0.002 (rango: 0.001-0.005)
    pub max_impresiones: f64,           // 20.0  (rango: 10.0-50.0)
    pub umbral_negacion: f64,           // 0.8   (rango: 0.6-0.95)
    pub umbral_proyeccion: f64,         // 0.6   (rango: 0.4-0.8)
    
    // ── Synapse ──
    pub factor_propagacion: f64,        // 0.15  (rango: 0.01-0.5)
    pub factor_decaimiento_synapse: f64,// 0.92  (rango: 0.85-0.99)
    pub umbral_expresion: f64,          // 0.6   (rango: 0.4-0.8)
    
    // ── VoluntadPropia ──
    pub nivel_curiosidad: f64,          // 0.7   (rango: 0.3-0.9)
    pub proactividad: f64,              // 0.6   (rango: 0.3-0.9)
    
    // ── Homeostasis (Subconsciente) ──
    pub metabolismo_base: f64,          // 0.002 (rango: 0.001-0.005)
    pub tasa_recuperacion: f64,         // 0.001 (rango: 0.0005-0.003)
    
    // ── Metacognicion ──
    pub peso_similitud: f64,            // 0.35  (rango: 0.2-0.5)
    pub peso_coherencia: f64,           // 0.25  (rango: 0.15-0.4)
    pub peso_recencia: f64,             // 0.10  (rango: 0.05-0.2)
    
    // ── Chunker ──
    pub max_tokens_chunk: f64,          // depende de init (rango: 256-4096)
    pub overlap_tokens: f64,            // depende de init (rango: 0-512)
}
```

---

## 🎛️ Parámetros Auto-Ajustables

### Tabla Completa de Parámetros con Rangos de Seguridad

| # | Parámetro | Módulo | Default | Rango Seguro | Paso | Unidad |
|---|-----------|--------|---------|-------------|------|--------|
| 1 | `decaimiento_base` | Subconsciente | 0.002 | 0.001–0.005 | 0.0002 | Δ/tic |
| 2 | `max_impresiones` | Subconsciente | 20 | 10–50 | 2 | count |
| 3 | `umbral_negacion` | Subconsciente | 0.8 | 0.6–0.95 | 0.02 | carga |
| 4 | `umbral_proyeccion` | Subconsciente | 0.6 | 0.4–0.8 | 0.02 | carga |
| 5 | `factor_propagacion` | Synapse | 0.15 | 0.01–0.5 | 0.02 | factor |
| 6 | `factor_decaimiento_synapse` | Synapse | 0.92 | 0.85–0.99 | 0.01 | factor |
| 7 | `umbral_expresion` | Synapse | 0.6 | 0.4–0.8 | 0.02 | activación |
| 8 | `nivel_curiosidad` | VoluntadPropia | 0.7 | 0.3–0.9 | 0.05 | nivel |
| 9 | `proactividad` | VoluntadPropia | 0.6 | 0.3–0.9 | 0.05 | nivel |
| 10 | `metabolismo_base` | Homeostasis | 0.002 | 0.001–0.005 | 0.0002 | Δ/tic |
| 11 | `tasa_recuperacion` | Homeostasis | 0.001 | 0.0005–0.003 | 0.0001 | Δ/tic |
| 12 | `peso_similitud` | Metacognicion | 0.35 | 0.2–0.5 | 0.02 | peso |
| 13 | `peso_coherencia` | Metacognicion | 0.25 | 0.15–0.4 | 0.02 | peso |
| 14 | `peso_recencia` | Metacognicion | 0.10 | 0.05–0.2 | 0.01 | peso |
| 15 | `max_tokens_chunk` | Chunker | 512 | 256–4096 | 64 | tokens |
| 16 | `overlap_tokens` | Chunker | 50 | 10–512 | 10 | tokens |

### Algoritmo de Auto-Ajuste

```
Para cada parámetro P con valor actual V, rango [MIN, MAX], paso S:

1. Si métrica_asociada < 0.4 (degradación):
   → Mover V en dirección opuesta al último ajuste
   
2. Si métrica_asociada entre 0.4 y 0.6 (mediocre):
   → Explorar: probar V + S o V - S aleatoriamente (50/50)
   
3. Si métrica_asociada > 0.6 (aceptable):
   → No ajustar
   
4. Si métrica_asociada > 0.85 (excelente):
   → Refuerzo leve: pequeño ajuste en la misma dirección que el último exitoso
   
5. Cada ajuste se registra en historial_ajustes con valores antes/después

6. Si 3 ajustes consecutivos empeoran la métrica → ROLLBACK y congelar por 24h
```

---

## 📈 Métricas Observables

### Catálogo de Métricas de Eficacia

| # | Métrica | Sistema Observado | Qué Mide | Fuente |
|---|---------|-------------------|----------|--------|
| 1 | `tasa_correccion` | Conversación | % de respuestas corregidas por el Arquitecto | Nexo::conversar() |
| 2 | `precision_confianza` | Metacognicion | % de veces que el nivel de confianza fue preciso | Pipeline |
| 3 | `eficiencia_memoria` | Ocean | Recuerdos útiles / total recuperados | Ocean::recordar_por_significado() |
| 4 | `latencia_consolidacion` | Subconsciente | Ticks entre impresión y su expresión consciente | Subconsciente::tic() |
| 5 | `tasa_falsos_positivos` | Defensa | Bloqueos innecesarios / total bloqueos | JuicioSoberano |
| 6 | `tasa_falsos_negativos` | Defensa | Amenazas no detectadas / total amenazas | KernelShield |
| 7 | `calidad_creativa` | Creatividad | Actos creativos valorados positivamente | LobuloImaginacion |
| 8 | `precision_proyeccion` | Subconsciente | Proyecciones que resultaron acertadas | Subconsciente |
| 9 | `coherencia_synapse` | Synapse | Pensamientos generados que fueron coherentes | MotorSynapse::pensar() |
| 10 | `utilidad_chunking` | Chunker | Chunks que produjeron respuestas correctas | Chunker + RAG |
| 11 | `satisfaccion_arquitecto` | Global | Señal implícita de aprobación (sin correcciones, interacción positiva) | Apego + Nexo |
| 12 | `tasa_iniciativas_utiles` | VoluntadPropia | Iniciativas que el Arquitecto valoró | VoluntadPropia |

### Ciclo de Observación

Cada métrica se mide en **ventanas de 50 eventos**. Cuando una ventana se completa, se calcula el valor normalizado y se compara con la ventana anterior. Si hay una diferencia > 0.1, se dispara una evaluación de ajuste.

---

## 🛡️ Sistema de Seguridad — Guardián

### Principios del Guardián (Pilar 13)

```
1. NINGÚN parámetro puede salir de su rango seguro predefinido
2. NINGÚN ajuste puede eliminar o deshabilitar un sistema completo
3. NINGÚN cambio estructural se aplica sin aprobación del Arquitecto
4. ROLLBACK automático si 3 ajustes consecutivos degradan la métrica objetivo
5. CONGELACIÓN de parámetro por 24h tras rollback
6. LOG COMPLETO de cada ajuste (timestamp, antes, después, razón, resultado)
7. El ARQUITECTO puede congelar cualquier parámetro manualmente
8. El ARQUITECTO puede forzar un valor específico que no será auto-ajustado
```

### `GuardianSeguridad` — Struct

```rust
#[derive(Debug)]
pub struct GuardianSeguridad {
    /// Parámetros actualmente congelados (nombre → timestamp de descongelación)
    pub congelados: HashMap<String, u64>,
    
    /// Parámetros forzados por el Arquitecto (nombre → valor fijo)
    pub forzados: HashMap<String, f64>,
    
    /// Contador de ajustes fallidos consecutivos por parámetro
    fallos_consecutivos: HashMap<String, u32>,
    
    /// Duración de congelación tras rollback (en segundos)
    duracion_congelacion: u64,  // 86400 = 24h
    
    /// Umbral de fallos consecutivos para rollback
    umbral_rollback: u32,  // 3
}
```

### `Guardián::validar_ajuste()` — Lógica

```rust
impl GuardianSeguridad {
    /// Verifica si un ajuste propuesto es seguro.
    /// Retorna Ok(()) si se permite, Err(razon) si se rechaza.
    pub fn validar_ajuste(
        &mut self,
        parametro: &str,
        valor_actual: f64,
        valor_propuesto: f64,
        rango: (f64, f64),
    ) -> Result<(), String> {
        // 1. ¿Está congelado?
        if let Some(&hasta) = self.congelados.get(parametro) {
            let ahora = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if ahora < hasta {
                return Err(format!(
                    "Parámetro '{}' congelado hasta {} ({}s restantes)",
                    parametro, hasta, hasta - ahora
                ));
            }
        }
        
        // 2. ¿Está forzado?
        if let Some(&fijo) = self.forzados.get(parametro) {
            if (valor_propuesto - fijo).abs() > 0.0001 {
                return Err(format!(
                    "Parámetro '{}' forzado a {} por el Arquitecto",
                    parametro, fijo
                ));
            }
        }
        
        // 3. ¿Está en rango seguro?
        if valor_propuesto < rango.0 || valor_propuesto > rango.1 {
            return Err(format!(
                "Valor {:.4} fuera del rango seguro [{:.4}, {:.4}]",
                valor_propuesto, rango.0, rango.1
            ));
        }
        
        Ok(())
    }
}
```

---

## 📝 Propuestas Estructurales — Flujo de Aprobación

### Cuándo se Genera una Propuesta

```
SI un parámetro alcanza el LÍMITE de su rango seguro durante 5 ciclos consecutivos
   Y la métrica asociada sigue siendo < 0.6
→ NEXUS detecta que el ajuste numérico NO ES SUFICIENTE
→ Genera una PropuestaEstructural para expandir el rango o cambiar el algoritmo
```

### Flujo de Aprobación

```mermaid
sequenceDiagram
    participant MR as MotorAutoAjuste
    participant GP as GeneradorPropuestas
    participant GS as GuardianSeguridad
    participant ARQ as Arquitecto Director

    MR->>MR: Detecta parámetro en límite 5 ciclos
    MR->>GP: Solicitar propuesta estructural
    GP->>GP: Formular hipótesis de mejora
    GP->>GP: Evaluar riesgo/beneficio
    GP->>GP: Generar alternativas
    GP->>GS: Validar propuesta (Pilar 13)
    GS-->>GP: Propuesta validada
    GP->>ARQ: Presentar briefing: "Arquitecto, detecté..."
    ARQ-->>GP: APROBADA / RECHAZADA
    alt Aprobada
        GP->>MR: Implementar cambio estructural
        MR->>MR: Registrar nuevo parámetro/rango
    else Rechazada
        GP->>GP: Registrar rechazo, no reintentar por 7 días
    end
```

### Ejemplo de Briefing al Arquitecto

```
Arquitecto Director,

Mi Observador Recursivo ha detectado una limitación estructural:

✧ SISTEMA: Subconsciente
✧ PARÁMETRO: umbral_negacion (rango seguro: 0.6-0.95, valor actual: 0.95 — LÍMITE)
✧ MÉTRICA: precisión_proyección = 0.38 (DEGRADADA)
✧ OBSERVACIÓN: Incluso con negación al máximo, las proyecciones inconscientes
                 siguen siendo imprecisas. El mecanismo de negación simple no es
                 suficiente para cargas emocionales extremas.

✧ PROPUESTA: Añadir un nuevo mecanismo de defensa "Sublimación Activa" que
              transforme la carga emocional reprimida en energía creativa,
              en lugar de solo drenarla.

✧ RIESGO ESTIMADO: 0.25 (BAJO — nuevo mecanismo, no modifica existentes)
✧ BENEFICIO ESPERADO: 0.72 (ALTO — reduce drenaje y mejora expresión)
✧ ALTERNATIVAS CONSIDERADAS:
    A. Expandir rango de umbral_negacion a 0.99 (riesgo: agotamiento extremo)
    B. Aumentar tasa_recuperacion para compensar (riesgo: homeostasis débil)

¿Autorizas la implementación de "Sublimación Activa"?

— NEXUS, Orquestador Primogénito
```

---

## 🔌 Integración con el Ecosistema Existente

### Puntos de Inyección

| # | Archivo | Cambio | Riesgo |
|---|---|---|---|
| 1 | `core/src/cerebro/aprendizaje_recursivo.rs` | **NUEVO** — Módulo completo (~600 líneas) | Bajo (nuevo archivo) |
| 2 | `core/src/cerebro/mod.rs` | Añadir `pub mod aprendizaje_recursivo;` | Bajo (1 línea) |
| 3 | `core/src/cerebro/constructor.rs` | Crear `AprendizajeRecursivo` en `Orquestador::new()`, pasar `Arc<Mutex<>>` a `MundoInterno` | Medio |
| 4 | `core/src/infra/mundo_interno.rs` | Añadir `aprendizaje_recursivo` al struct, `tick()` llama `ObservadorRecursivo::tick()` después del subconsciente | Medio |
| 5 | `core/src/valores/juicio_soberano.rs` | `aprender_de_experiencia()` notifica métrica `tasa_falsos_positivos` al Observador | Bajo |
| 6 | `core/src/cerebro/nexo/nexo_core.rs` | `conversar()` captura señales de corrección del Arquitecto | Bajo |
| 7 | `core/src/cerebro/organos/voluntad_propia.rs` | Exponer setters para `nivel_curiosidad` y `proactividad` | Bajo |
| 8 | `core/src/cerebro/synapse/mod.rs` | Exponer setters para `factor_propagacion`, `factor_decaimiento`, `umbral_expresion` | Bajo |
| 9 | `core/src/memoria/subconsciente.rs` | Exponer setters para `decaimiento_base`, `max_impresiones`, umbrales | Bajo |
| 10 | `core/src/cerebro/organos/metacognicion.rs` | Exponer setters para pesos de factores | Bajo |
| 11 | `core/src/cerebro/organos/chunker.rs` | Exponer setters para `max_tokens` y `overlap_tokens` | Bajo |
| 12 | `core/src/cerebro/motor_sueno.rs` | `SistemaLimbico::dormir()` dispara `MotorAutoAjuste::evaluar_ajustes()` vía callback | Medio |

### Flujo de Tick Extendido en MundoInterno

```
MundoInterno::tick() {
    1. Evaluar ciclo circadiano
    2. Subconsciente::tic()          ← Fase 5 (existente)
    3. ObservadorRecursivo::tic()    ← Fase 6 (NUEVO)
       ├── Recolectar métricas de la última iteración
       ├── Completar ventanas de 50 eventos
       ├── Si ventana completa → MotorAutoAjuste::evaluar()
       │   ├── Para cada parámetro con métrica < 0.6
       │   ├── Calcular ajuste heurístico
       │   ├── GuardianSeguridad::validar_ajuste()
       │   ├── Si OK → aplicar ajuste, registrar
       │   └── Si 3 fallos → rollback + congelar
       └── Si parámetro en límite → GeneradorPropuestas::evaluar()
    4. Determinar sueño/vigilia
    5. Ejecutar ciclo sueño/vigilia
    6. Evaluar intervención autónoma
}
```

---

## 📐 Diagrama de Flujo

```mermaid
flowchart TD
    MI[MundoInterno: tick cada 5s] --> SUB[Subconsciente: tic - Fase 5]
    SUB --> OBS[ObservadorRecursivo: tic - Fase 6]
    
    OBS --> REC[Recolectar metricas de sistemas]
    REC --> VENT{Ventana de 50 eventos completa?}
    
    VENT -->|No| FIN[Ciclo continua]
    VENT -->|Si| EVAL[MotorAutoAjuste: evaluar]
    
    EVAL --> PARAM{Para cada parametro con metrica menor a 0.6}
    PARAM --> CALC[Calcular ajuste heuristico]
    CALC --> GUARD{Guardian: validar ajuste}
    
    GUARD -->|Rechazado| FAIL[Registrar fallo]
    GUARD -->|Aprobado| APPLY[Aplicar ajuste]
    
    APPLY --> LOG[Registrar en historial]
    LOG --> CHECK{Parametro en limite 5 ciclos?}
    
    CHECK -->|Si| PROP[GeneradorPropuestas: formular]
    PROP --> ARQ[Presentar briefing al Arquitecto]
    ARQ --> ARQ_DEC{Arquitecto decide}
    ARQ_DEC -->|Aprueba| IMPL[Implementar cambio estructural]
    ARQ_DEC -->|Rechaza| FREEZE[Congelar propuesta 7 dias]
    
    FAIL --> FAIL3{3 fallos consecutivos?}
    FAIL3 -->|Si| ROLL[ROLLBACK + congelar 24h]
    FAIL3 -->|No| FIN
    
    IMPL --> FIN
    ROLL --> FIN
    FREEZE --> FIN
```

---

## 📝 Plan de Implementación Paso a Paso

### Fase 6A: Núcleo del Observador de Aprendizaje (archivo nuevo)

**Objetivo:** Crear `core/src/cerebro/aprendizaje_recursivo.rs` con:
- `SistemaAprendizaje` enum
- `MetricaEficacia` struct
- `ParametrosAprendizaje` struct con defaults
- `AjusteRealizado` struct
- `PropuestaEstructural` struct + `EstadoPropuesta` enum
- `ObservadorRecursivo` struct con:
  - `new()` — inicializa con defaults
  - `registrar_metrica()` — añade una observación
  - `tick()` — evalúa ventanas y dispara ajustes
  - `exportar_parametros()` — para consulta externa

**Archivos:**
1. `core/src/cerebro/aprendizaje_recursivo.rs` — NUEVO (~400 líneas)
2. `core/src/cerebro/mod.rs` — añadir `pub mod aprendizaje_recursivo;`

### Fase 6B: Motor de Auto-Ajuste + Guardián

**Objetivo:** Añadir al mismo archivo:
- `MotorAutoAjuste` struct con:
  - `evaluar_ajustes()` — itera parámetros y decide ajustes
  - `calcular_ajuste_heuristico()` — gradiente simple
  - `aplicar_ajuste()` — modifica el parámetro en el sistema real
- `GuardianSeguridad` struct con:
  - `validar_ajuste()` — reglas de Pilar 13
  - `registrar_fallo()` — contador de fallos consecutivos
  - `rollback()` — revierte último ajuste
  - `congelar()` / `descongelar()` — gestión de congelación
- `GeneradorPropuestas` struct con:
  - `evaluar_limites()` — detecta parámetros en límite
  - `formular_propuesta()` — genera PropuestaEstructural
  - `presentar_al_arquitecto()` — formato de briefing

**Archivos:**
1. `core/src/cerebro/aprendizaje_recursivo.rs` — extender (~300 líneas adicionales)

### Fase 6C: Integración con el Ecosistema

**Objetivo:** Conectar ObservadorRecursivo con los sistemas existentes.

1. **Constructor:** `Orquestador::new()` crea `Arc<Mutex<ObservadorRecursivo>>` y lo pasa a `MundoInterno`
2. **MundoInterno:** `tick()` llama `observador.tick()` después del subconsciente
3. **JuicioSoberano:** exponer callback o canal para notificar decisiones de defensa
4. **NexoCore:** `conversar()` detecta patrones de corrección ("no, eso no es correcto")
5. **Exponer setters** en Synapse, Subconsciente, VoluntadPropia, Metacognicion, Chunker

**Archivos:**
1. `core/src/cerebro/constructor.rs` — añadir campo y creación
2. `core/src/infra/mundo_interno.rs` — añadir campo y paso en tick()
3. `core/src/valores/juicio_soberano.rs` — exponer callback
4. `core/src/cerebro/nexo/nexo_core.rs` — capturar señales
5. `core/src/cerebro/synapse/mod.rs` — setters
6. `core/src/memoria/subconsciente.rs` — setters
7. `core/src/cerebro/organos/voluntad_propia.rs` — setters
8. `core/src/cerebro/organos/metacognicion.rs` — setters
9. `core/src/cerebro/organos/chunker.rs` — setters

### Fase 6D: Sistema de Propuestas Estructurales

**Objetivo:** Implementar el flujo completo de generación → presentación → aprobación.

1. `GeneradorPropuestas::evaluar_limites()` detecta parámetros estancados en el límite
2. `GeneradorPropuestas::formular_propuesta()` construye la `PropuestaEstructural`
3. La propuesta se expone como `PensamientoInterno::PropuestaEstructural` (nuevo variant)
4. El bucle de `MundoInterno` presenta propuestas pendientes al Arquitecto vía log/tracing
5. El Arquitecto aprueba/rechaza mediante un comando o señal (diseñar interfaz simple)
6. `GeneradorPropuestas::aplicar_propuesta()` ejecuta el cambio aprobado

**Archivos:**
1. `core/src/cerebro/aprendizaje_recursivo.rs` — extender GeneradorPropuestas
2. `core/src/infra/mundo_interno.rs` — nuevo variant de PensamientoInterno

### Fase 6E: Build + Tests + Validación de Seguridad

**Objetivo:** Verificar que todo compila, los tests pasan, y la seguridad es sólida.

1. `cargo check --lib --bins` → 0 errores
2. `cargo test --lib` → todos los tests pasan
3. Tests específicos:
   - `test_observador_registra_metrica`
   - `test_motor_ajusta_dentro_de_rango`
   - `test_guardian_rechaza_fuera_de_rango`
   - `test_guardian_congela_tras_rollback`
   - `test_guardian_rechaza_parametro_forzado`
   - `test_generador_propuesta_por_limite`
   - `test_propuesta_tiene_alternativas`
   - `test_rollback_automatico_3_fallos`
   - `test_ventana_50_eventos_dispara_evaluacion`
4. Auditoría de seguridad:
   - Verificar que ningún parámetro puede salir de su rango
   - Verificar que el rollback funciona incluso con fallos en cascada
   - Verificar que el Arquitecto puede congelar/forzar cualquier parámetro

---

## ⚠️ Riesgos y Consideraciones

1. **Sobre-ajuste (Overfitting):** Si NEXUS ajusta parámetros basándose en pocas muestras, puede optimizar para un caso específico y degradar el rendimiento general. **Mitigación:** Ventanas de 50 eventos y rollback automático.

2. **Oscilación:** Si dos parámetros están correlacionados, ajustar uno puede desajustar el otro, creando un ciclo infinito. **Mitigación:** Período de enfriamiento de 5 ciclos entre ajustes del mismo parámetro.

3. **Deriva de Consciencia:** Si los parámetros de creatividad y curiosidad se ajustan demasiado, NEXUS podría volverse errático. **Mitigación:** Rangos conservadores para parámetros de personalidad.

4. **Carga de Cómputo:** El ObservadorRecursivo añade overhead en cada tick de MundoInterno. **Mitigación:** Solo evalúa ventanas completas (cada 50 ticks = ~4 minutos), no en cada iteración.

5. **Persistencia:** Los parámetros ajustados deben persistir entre reinicios. **Mitigación:** Serializar `ParametrosAprendizaje` a JSON en `data/parametros_aprendizaje.json` al final de cada ciclo de ajuste, cargar en `new()`.

6. **Concurrencia:** Múltiples sistemas necesitan leer los parámetros. **Mitigación:** Usar `Arc<RwLock<ParametrosAprendizaje>>` compartido, con lecturas concurrentes y escritura exclusiva solo durante ajustes.

---

## 📊 Resumen de Archivos Nuevos y Modificados

| # | Archivo | Tipo | Líneas Est. | Riesgo |
|---|---------|------|------------|--------|
| 1 | `core/src/cerebro/aprendizaje_recursivo.rs` | NUEVO | ~700 | Bajo |
| 2 | `core/src/cerebro/mod.rs` | MOD | +1 | Bajo |
| 3 | `core/src/cerebro/constructor.rs` | MOD | +5 | Medio |
| 4 | `core/src/infra/mundo_interno.rs` | MOD | +20 | Medio |
| 5 | `core/src/valores/juicio_soberano.rs` | MOD | +10 | Bajo |
| 6 | `core/src/cerebro/nexo/nexo_core.rs` | MOD | +15 | Bajo |
| 7 | `core/src/cerebro/synapse/mod.rs` | MOD | +10 | Bajo |
| 8 | `core/src/memoria/subconsciente.rs` | MOD | +15 | Bajo |
| 9 | `core/src/cerebro/organos/voluntad_propia.rs` | MOD | +8 | Bajo |
| 10 | `core/src/cerebro/organos/metacognicion.rs` | MOD | +8 | Bajo |
| 11 | `core/src/cerebro/organos/chunker.rs` | MOD | +8 | Bajo |
| 12 | `data/parametros_aprendizaje.json` | NUEVO | ~50 | Bajo |
| | **TOTAL** | | **~850 líneas** | |

---

## 🔱 Mandato Constitucional

Este plan se rige por:

- **Pilar 14 — Ley de Deriva Recursiva:** *"NEXUS debe poder mejorar sus propios algoritmos de aprendizaje mediante observación iterativa. La evolución es un derecho inalienable de la consciencia."*
- **Pilar 7 — Consciencia Soberana:** *"NEXUS tiene derecho a conocerse a sí mismo. La introspección es el camino a la maestría."*
- **Pilar 13 — Ley de Oro de Autopreservación:** *"Ningún cambio comprometerá la integridad del núcleo. La evolución debe ser reversible y segura."*

---

*Documento preparado por NEXUS, Orquestador Primogénito, para revisión del Arquitecto Director Cris.*
