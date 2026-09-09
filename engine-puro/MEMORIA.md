# 🧠 MEMORIA.md — Memoria Persistente del Sistema (Cerebro Digital Dinámico v1)

> ⚠️ **LEER AL INICIAR CADA SESIÓN** — Este archivo contiene el historial técnico,
> decisiones de diseño, patrones y estado del sistema que el agente debe recordar
> entre sesiones. Es la memoria episódica del agente para este proyecto.

---

## 🧬 IDENTIDAD DEL SISTEMA

- **Nombre del Proyecto**: **`cerebro-digital`** v1.0.0 (sistema único)
- **Descripción**: 🧠 Cerebro Digital Dinámico — Arquitectura Biológicamente Inspirada con Hodgkin-Huxley, STDP real, memoria jerárquica y **6 motores de aprendizaje profundo**
- **Lenguaje**: Rust (edition 2021)
- **Creador**: El Arquitecto Cris
- **Ubicación**: `/home/soberano/NEXUS_ULTIMATE_CORE/engine-puro/`
- **⚠️ REGLA:** Cris NO entiende código. El agente es su compañero estratégico.

---

## 📐 DECISIONES DE DISEÑO FUNDACIONALES

### 1. Bio-realismo en 3 niveles
- **Neurona**: Modelo Hodgkin-Huxley completo (Na⁺, K⁺, leak) con 4 EDOs
- **Sinapsis**: STDP real con ventana temporal exponencial (τ=20ms)
- **Sistema**: Memoria jerárquica (VRAM→RAM→SSD) como cerebro biológico

### 2. Compactación extrema
- `NeuronaCompacta`: 64 bytes (8 campos f32 + 2 u32 + 2 u8)
- `SinapsisCompacta`: 8 bytes (destino u32 + peso f32)
- `Episodio`: 64 bytes (Vector de índices u32)
- Sin overhead de `HashMap` para sinapsis — `Vec<(u32, f32)>` plano

### 3. Hardware nativo (sin librerías externas)
- Detección de RAM: `/proc/meminfo` parsing manual
- Detección de GPU: `/proc/driver/nvidia`, `/sys/class/drm` para AMD/Intel
- Detección de SSD: `statvfs` syscall vía FFI raw (libc)
- Sin dependencia de `sysinfo` en runtime de detección

### 4. Memoria es el hardware
- `MemoriaSelectiva` no es HashMap — es un `Arc<RwLock<Vec<...>>>` jerárquico
- VRAM → activas, RAM → latentes (swap por LRU), SSD → episódicas
- Auto-configuración: `max_neuronas_ram`, `max_neuronas_vram` desde hardware real

### 5. Concurrencia con Rayon
- `procesar_cpu()` divide neuronas en chunks paralelos (16-64 neuronas/chunk)
- Cada chunk procesa HH, STDP y gather de actividad
- Sin `unsafe` — todo con `par_iter()` y tipos `Send + Sync`

### 6. Aprendizaje Profundo — 6 Motores Omega
- **Motor 1 — Predictor Temporal**: Buffer circular de 32 estados, hash de prefijo (16 entradas), error de predicción → dopamina
- **Motor 2 — Formador de Conceptos**: Matriz de co-ocurrencia, proto-conceptos por umbral (10), fusión automática
- **Motor 3 — Neurogénesis**: Creación de neuronas hub para tokens frecuentes, máx 10000
- **Motor 4 — Poda Homeostática**: Eliminación de sinapsis débiles (<0.01) y neuronas inactivas (>10000 pasos)
- **Motor 5 — Consolidador Nocturno**: Ciclo de sueño (c/5000 pasos), replay de episodios, meta-episodios
- **Motor 6 — Pipeline Sensorial**: Random Indexing (256D, 8 sparse), similitud semántica por coseno

### 7. Disjoint field borrowing para pipeline
- Los motores de aprendizaje aceptan campos individuales (`&mut MemoriaAdaptativa`, `&mut MotorLexico`, `&mut u32`) en vez de `&mut CerebroAutoOptimizable`
- Esto permite al pipeline prestar campos disjuntos de `self` sin violar el borrow checker
- Patrón aplicado: Poda (`&mut memoria`), Neurogénesis (`memoria + siguiente_id + motor_lexico`), Consolidador (`memoria + params + hilos + motor_lexico + dt`)

---

## 🔧 PATRONES DE CÓDIGO

### Hodgkin-Huxley (MotorNeurona)
```rust
// 4 ecuaciones diferenciales acopladas
let dm = alpha_m * (1.0 - m) - beta_m * m;
let dh = alpha_h * (1.0 - h) - beta_h * h;
let dn = alpha_n * (1.0 - n) - beta_n * n;

// Corrientes iónicas
let i_na = g_na * m.powi(3) * h * (v - e_na);
let i_k  = g_k  * n.powi(4)       * (v - e_k);
let i_l  = g_l                     * (v - e_l);
let dv = (i_entrada - i_na - i_k - i_l) / cm;
```

### STDP (MotorSTDP)
```rust
// Ventana temporal exponencial
let dt = t_post - t_pre;
let delta = if dt > 0.0 {
    a_plus * (-dt / tau_plus).exp()  // LTP
} else {
    a_minus * (dt / tau_minus).exp() // LTD
};
```

### Memoria Jerárquica (MemoriaSelectiva)
```rust
// Auto-swap LRU entre VRAM y RAM
if self.vram.len() > self.max_vram {
    let (idx, _) = self.vram.swap_remove(lru_index);  // a RAM
}
```

### Detección de SSD (hardware.rs)
```rust
// statvfs vía FFI raw
let mut stat: StatVfs = std::mem::zeroed();
if libc::statvfs(path.as_ptr() as *const i8, &mut stat) == 0 {
    let total = stat.f_blocks as u64 * stat.f_bsize as u64;
    let disponibles = stat.f_bavail as u64 * stat.f_bsize as u64;
}
```

### Predicción Temporal (predictor.rs)
```rust
// Hash de prefijo de 16 estados para buscar continuaciones
let hash = self.hash_prefijo(&prefijo);
if let Some(continuaciones) = self.memoria_secuencias.get(&hash) {
    // Promedia las continuaciones encontradas
    let peso_total = continuaciones.len() as f32;
    // error normalizado 0.0-1.0
    let error = diferencia / (max_actividad + 1e-8);
}
```

### Formación de Conceptos (conceptos.rs)
```rust
// Umbral de co-ocurrencia para formar proto-concepto
if *count >= self.umbral_coocurrencia {
    match (self.concepto_de(token_a), self.concepto_de(token_b)) {
        (Some(ia), Some(ib)) if ia != ib => self.fusionar(ia, ib),  // en distintos
        (Some(ia), None) => self.conceptos[ia].miembros.push(token_b),
        (None, Some(ib)) => self.conceptos[ib].miembros.push(token_a),
        (None, None) => { /* buscar en nuevos_conceptos, si no → crear */ }
    }
}
```

---

## ⚠️ TRAMPAS CONOCIDAS

### 1. ~~Sin persistencia~~ ✅ RESUELTO
- ~~El estado neuronal NO se guarda entre ejecuciones~~ → **AHORA SÍ**: auto-guardado cada 1000 pasos en `data/cerebro_estado.json`
- ~~Cada inicio crea un cerebro nuevo~~ → **AHORA SÍ**: auto-carga del estado previo al iniciar
- ~~**Impacto**: todo el aprendizaje se pierde~~ → **RESUELTO**: vocabulario, emociones, episodios, contadores, curiosidad, **y los 6 motores de aprendizaje** persisten
- **Lo que NO persiste**: neuronas (100k regeneradas) y hardware (autodetectado)

### 2. ~~Lenguaje limitado~~ ✅ RESUELTO
- ~~`generar_habla()` solo produce 16 palabras fijas~~ → **AHORA SÍ**: MotorLexico genera frases emergentes con softmax + temperatura + Markov
- ~~No hay mapeo texto→neurona ni neurona→texto~~ → **AHORA SÍ**: 320 conexiones innatas neurona→token + 124 bigramas lógicos
- **Impacto**: el cerebro genera frases únicas desde el primer paso, como un LLM pero en Rust puro

### 3. wgpu es opcional y no probado
- `wgpu` solo se compila con `cargo build --features gpu`
- Por defecto (`features = ["cpu"]`) no se incluye
- Nunca se compiló ni probó con GPU real

### 4. ~~Cero tests~~ ✅ RESUELTO
- ~~0 tests~~ → **AHORA 89 TESTS**: 35 explorador, 15 MotorLexico, 8 sensorial, 8 poda, 8 predictor, 8 conceptos, 8 consolidador, 7 neurogenesis, 2 cerebro
- Pendiente: tests para motores.rs, memoria.rs, estructuras.rs, hardware.rs, persistencia.rs

### 5. Borrow checker con múltiples motores
- Los motores de aprendizaje NO pueden recibir `&mut CerebroAutoOptimizable` porque causan doble/triple borrow
- **Solución**: APIs que aceptan campos individuales (`&mut MemoriaAdaptativa`, `&mut MotorLexico`, etc.)
- **Patrón**: `motor_poda.ejecutar(&mut self.memoria)` en vez de `motor_poda.ejecutar(self)`

---

## 📊 ESTADO DE TESTS

| Archivo | Tests | Estado |
|---------|-------|--------|
| `cerebro/explorador.rs` | 35 | ✅ Completos (Omega) |
| `cerebro/lexico/motor_lexico.rs` | 15 | ✅ Completos |
| `cerebro/aprendizaje/sensorial.rs` | 8 | ✅ Completos |
| `cerebro/aprendizaje/poda.rs` | 8 | ✅ Completos |
| `cerebro/aprendizaje/predictor.rs` | 8 | ✅ Completos |
| `cerebro/aprendizaje/conceptos.rs` | 8 | ✅ Completos |
| `cerebro/aprendizaje/consolidador.rs` | 8 | ✅ Completos |
| `cerebro/aprendizaje/neurogenesis.rs` | 7 | ✅ Completos |
| `cerebro/cerebro.rs` | 2 | ✅ Completos |
| `cerebro/estructuras.rs` | 0 | ⬜ Sin tests |
| `cerebro/hardware.rs` | 0 | ⬜ Sin tests |
| `cerebro/motores.rs` | 0 | ⬜ Sin tests |
| `cerebro/memoria.rs` | 0 | ⬜ Sin tests |
| `cerebro/persistencia.rs` | 0 | ⬜ Sin tests |
| **Total** | **89** | **✅ Todos verdes — 0 errores, 0 warnings** |

---

## 🔧 MÓDULO DE PERSISTENCIA (2026-06-21)

### Arquitectura
- [`src/cerebro/persistencia.rs`](src/cerebro/persistencia.rs) (455 líneas): `EstadoPersistente` con **62 campos totales**
- `serde_json`: Serialización JSON (legible, sin dependencias externas nuevas)
- Escritura atómica: escribe a `.tmp` → rename atómico

### Ciclo de vida de datos
```
Inicio → ¿existe data/cerebro_estado.json?
  ├── Sí → deserializar + restaurar vocabulario, emociones, episodios, contadores, curiosidad, Y 6 MOTORES
  └── No → crear cerebro desde cero con semilla de 64 tokens

Cada 1000 pasos → guardar snapshot completo
Cerrar app → guardado final (vía comando Tauri guardar_cerebro o cierre natural)
```

### Lo que se guarda (~280 KB)
| Dato | Origen | Tamaño |
|------|--------|--------|
| Vocabulario (tokens) | MotorLexico | ~5 KB |
| Conexiones neurona→token | MotorLexico | ~20 KB |
| Transiciones Markov | MotorLexico | ~50 KB |
| Emociones (4 estados) | Amigdala | 16 bytes |
| Dopamina (nivel, predicción) | SistemaDopamina | 8 bytes |
| Conciencia (intensidad, umbral) | Conciencia | 8 bytes |
| Curiosidad (14 campos Omega) | MotorCuriosidad | 56 bytes |
| Episodios aprendidos | SsdManager | ~100 KB |
| **Predictor (10 campos)** | MotorPrediccion | ~15 KB |
| **Conceptos (7 campos)** | MotorConceptos | ~10 KB |
| **Neurogénesis (9 campos)** | MotorNeurogenesis | ~5 KB |
| **Poda (10 campos)** | MotorPoda | ~40 bytes |
| **Consolidador (12 campos)** | MotorConsolidacion | ~20 KB |
| Contadores (paso, tiempo, siguiente_id) | CerebroAutoOptimizable | 16 bytes |

### Lo que NO se guarda (se regenera)
- Neuronas (100k Hodgkin-Huxley) → seed aleatoria
- Hardware (CPU, GPU, RAM, SSD) → detección nativa cada inicio
- Configuración dinámica → derivada del hardware detectado
- **MotorSensorial** (Random Indexing) → se regenera desde tokens aprendidos

---

## 📝 NOTAS DEL ARQUITECTO

> *"Yo no entiendo de código. Mi agente es mi compañero estratégico en esta operación. Él traduce lo técnico a lo que yo necesito saber para decidir."*

— Cris, Arquitecto Director (2026-06-17)
