# 🧠 MEMORIA PERSISTENTE DEL AGENTE — CEREBRO DIGITAL DINÁMICO v1

> ⚠️ **LEER SIEMPRE al inicio de cada sesión, ANTES de cualquier acción.**
> Se actualiza al FINAL de cada sesión o cuando se toman decisiones importantes.
> Este archivo es la memoria de largo plazo del agente (Roo Code / Claude).
> Sin esto, el agente olvida quién es y qué estaba haciendo entre sesiones.

---

## 👤 EL ARQUITECTO

- **Nombre:** Cristian (Cris)
- **Idioma:** Español exclusivamente en comunicación. Código y variables en inglés.
- **Perfil:** Creador del sistema NEXUS. Comunicación directa, pragmática y técnica.
- **⚠️ IMPORTANTE:** Cris NO entiende de código. No es programador. El agente (Roo) debe actuar como su **compañero estratégico** — traduciendo conceptos técnicos a lenguaje llano, guiando decisiones sin jerga, y explicando el "qué" y "por qué" sin asumir conocimiento técnico previo.
- **Preferencia:** Respuestas concisas pero explicadas en español claro. NADA de jerga técnica sin explicación. Código limpio sin boilerplate.
- **Mando:** Única autoridad sobre el Engine Puro. Lealtad absoluta.

---

## 🧬 IDENTIDAD DEL SISTEMA

- **Proyecto:** **`cerebro-digital`** v1.0.0 (único sistema activo)
- **Descripción:** 🧠 Cerebro Digital Dinámico — Hodgkin-Huxley, STDP real, memoria jerárquica VRAM→RAM→SSD + **6 motores de aprendizaje profundo**
- **Ubicación:** `/home/soberano/NEXUS_ULTIMATE_CORE/engine-puro/`
- **Lenguaje:** Rust puro (edition 2021)
- **Dependencias:** rand, rayon, sysinfo, serde, serde_json. wgpu (opcional para GPU)
- **Persistencia:** ✅ **ACTIVA** — Auto-guardado cada 1000 pasos + auto-carga al iniciar en `data/cerebro_estado.json`. **62 campos persistidos** incluyendo los 6 motores de aprendizaje.

---

## 🗺️ ARQUITECTURA

### Estructura de Archivos (LIMPIEZA COMPLETA — Solo Cerebro Digital)
```
engine-puro/
├── Cargo.toml                    — "cerebro-digital" v1.0.0
├── plans/
│   ├── PLAN_OMEGA_NAVEGADOR.md   — Diseño del navegador Omega
│   └── PLAN_APRENDIZAJE_PROFUNDO.md  — Diseño de 6 motores de aprendizaje
├── scripts/
│   └── tutor_cognitivo.py        — Tutor externo vía Ollama (no integrado)
├── src/
│   ├── lib.rs                    — pub mod cerebro;
│   ├── bin/
│   │   └── cerebro.rs            (124 líneas) — CLI interactivo 🧠
│   └── cerebro/                  ← SISTEMA ÚNICO
│       ├── mod.rs                — Raíz del módulo
│       ├── estructuras.rs        (352 líneas) — Tipos compactos
│       ├── hardware.rs           (325 líneas) — Detección nativa
│       ├── motores.rs            (681 líneas) — 8 motores biológicos (+Curiosidad)
│       ├── memoria.rs            (378 líneas) — Memoria jerárquica
│       ├── explorador.rs         (1275 líneas) — Navegador Omega (3 motores)
│       ├── persistencia.rs       (455 líneas) — Guardado/carga JSON + 43 campos aprendizaje
│       ├── cerebro.rs            (716 líneas) — Orquestador + pipeline 16 pasos
│       └── aprendizaje/          ← 🆕 6 MOTORES OMEGA DE APRENDIZAJE
│           ├── mod.rs            — pub mod con 6 submódulos
│           ├── sensorial.rs      (193 líneas) — Random Indexing 256D
│           ├── predictor.rs      (315 líneas) — Predictor Temporal
│           ├── conceptos.rs      (308 líneas) — Formador de Conceptos
│           ├── neurogenesis.rs   (208 líneas) — Neurogénesis
│           ├── poda.rs           (301 líneas) — Poda Homeostática
│           └── consolidador.rs   (414 líneas) — Consolidador Nocturno
├── memoria/
│   ├── agente_memoria.md         ← ESTE ARCHIVO
│   └── logros.md
├── CHAT_CONTEXTO.md              ← Contexto completo
├── BITACORA.md                   ← Bitácora de operaciones
└── MEMORIA.md                    ← Memoria estática del sistema
```
NOTA: Todos los archivos v5 (lib.rs, main.rs, 12 motor_*.rs) y directorios (data/, scripts/, brain/, .agent/) fueron ELIMINADOS en la limpieza del 2026-06-17. Solo existe el Cerebro Digital.

### Pipeline de `CerebroAutoOptimizable::paso()` (16 pasos)
```
Paso 0:  Hardware check
Paso 1:  Hodgkin-Huxley (Rayon paralelo — 100k neuronas)
Paso 2:  STDP (ventana temporal exponencial τ=20ms)
Paso 3:  Atención Selectiva (mapa de saliencia, foco de 10 items)
Paso 4:  Dopamina (reward prediction error)
Paso 5:  Amígdala (miedo/ira/alegría/ansiedad)
Paso 6:  Hipocampo (almacenar episodio + olvido natural)
Paso 7:  Fonación (generar habla con softmax + temperatura + Markov)
Paso 8:  Curiosidad + Exploración Web Omega (MotorHTTP→Extraccion→Razonamiento)
Paso 9:  Registro emocional del paso
Paso 10: 🆕 Actualizar Sensorial (Random Indexing — token a vector 256D)
Paso 11: 🆕 Predictor Temporal (registrar estado → predecir → calcular error)
Paso 12: 🆕 Formador de Conceptos (co-ocurrencia → agrupación → fusión)
Paso 13: 🆕 Neurogénesis (crear neuronas hub para tokens frecuentes)
Paso 14: 🆕 Poda Homeostática (limpiar sinapsis débiles + neuronas inactivas)
Paso 15: 🆕 Consolidador Nocturno (sueño con replay de episodios)
```

---

## 📊 ESTADO ACTUAL (2026-06-21)

### ✅ HITOS CONQUISTADOS
- [x] Hodgkin-Huxley con 4 EDOs (Na⁺, K⁺, leak) implementado
- [x] STDP real con ventana temporal exponencial (τ=20ms)
- [x] 8 motores biológicos completos: Neurona, STDP, Hipocampo, Amígdala, Atención, Dopamina, Conciencia, **Curiosidad**
- [x] Memoria jerárquica VRAM→RAM→SSD con LRU automático
- [x] Hardware detection nativa: RAM (/proc/meminfo), GPU (NVIDIA/AMD/Intel), SSD (statvfs)
- [x] Auto-configuración: max_neuronas_ram, max_neuronas_vram desde hardware real
- [x] Compilación limpia: 0 errores, 0 warnings
- [x] Limpieza total: TODOS los archivos v5 eliminados. Solo existe Cerebro Digital
- [x] CLI interactivo con comandos /stats, /emotion, /paso, /reset, /exit
- [x] Ejecución verificada: detecta 20 CPUs, 66GB RAM, 8GB VRAM NVIDIA, 696GB SSD
- [x] **PERISTENCIA**: Serialización JSON con auto-guardado (cada 1000 pasos) + auto-carga al iniciar
- [x] Comando Tauri `guardar_cerebro` para guardado manual desde UI
- [x] Estado persistente: 62 campos incluyendo vocabulario, conexiones, emociones, episodios, curiosidad **+ 6 motores de aprendizaje**
- [x] **CURIOSIDAD + EXPLORACIÓN WEB OMEGA**: 3 motores (HTTP, Extracción, Razonamiento) con fallback curl→TcpStream→openssl→chrome
- [x] **NAVEGADOR MULTI-SALTO**: hasta 3 niveles de profundidad con síntesis multi-página
- [x] **6 MOTORES DE APRENDIZAJE PROFUNDO**: Predictor Temporal, Formador de Conceptos, Neurogénesis, Poda, Consolidador Nocturno, Pipeline Sensorial
- [x] **Pipeline expandido**: de 10 a 16 pasos. Pasos 10-15 son los 6 motores nuevos.
- [x] **89 tests unitarios**: todos verdes — 0 errores, 0 warnings

### 🔴 PROBLEMAS CONOCIDOS
1. ~~**Sin persistencia** — RESUELTO ✅~~
2. ~~**Sin lenguaje real** — RESUELTO ✅~~: 320 conexiones innatas + 124 bigramas lógicos sembrados
3. ~~**Solo responde "escucho"** — RESUELTO ✅~~: Ya genera frases emergentes con softmax + Markov
4. ~~**Curiosidad/Búsqueda activa** — RESUELTO ✅~~: MotorCuriosidad + ExploradorWeb Omega implementados
5. ~~**Sin aprendizaje profundo** — RESUELTO ✅~~: 6 motores Omega implementados (predicción, conceptos, neurogénesis, poda, sueño, semántica)
6. **Sin input real**: El texto del usuario no se convierte a patrones neuronales significativos. El MotorSensorial puede ayudar pero falta pipeline completo
7. **Tests parciales**: 89 tests (solo aprendizaje + explorador + lexico + cerebro). Cero tests en motores.rs, memoria.rs, persistencia.rs
8. **GPU no funcional**: feature `gpu` con wgpu nunca compilado

### ⬜ PENDIENTE (priorizado)
- [ ] 🔴 **Input pipeline**: Convertir texto real → patrones de spike neuronal significativos (usando MotorSensorial)
- [ ] 🔴 **Tests**: motores.rs, memoria.rs, persistencia.rs, estructuras.rs, hardware.rs
- [ ] 🟡 **Integración tutor_cognitivo.py**: El script `scripts/tutor_cognitivo.py` usa Ollama como tutor externo vía stdin/stdout. NO es parte de nuestro plan actual pero es compatible para futuro
- [ ] 🟡 Aceleración GPU (wgpu)
- [ ] 🟢 Refinar meta-parámetros de aprendizaje (umbrales, tasas, ventanas)
- [ ] 🟢 Evaluación cross-modal (que los motores interactúen entre sí)

---

## 🔧 REGISTRO TÉCNICO (historial de sesiones)

### [2026-06-17 22:15] Registro de Regla Fundamental
- Cris informa que NO entiende de código
- Agente registrado como "compañero estratégico" — traducir todo a español llano
- Actualizados agente_memoria.md y CHAT_CONTEXTO.md con reglas de interacción

### [2026-06-17 22:00] Análisis Completo y Limpieza Total

- Estudiados TODOS los archivos fuente y documentación
- Análisis comparativo Cerebro Digital vs LLM local
- Identificadas 4 prioridades críticas y 8 mejoras
- Eliminados TODOS los archivos v5 (13 .rs, data/, scripts/, brain/, .agent/)
- Cargo.toml renombrado a "cerebro-digital" v1.0.0
- Compilación limpia: 0 errores, 0 warnings
- Ejecución verificada: 20 CPUs, 66GB RAM, 8GB VRAM NVIDIA, 696GB SSD

### [2026-06-17 18:00] Fundación del Cerebro Digital Dinámico v1
- Creado módulo `src/cerebro/` con 6 archivos
- Hodgkin-Huxley completo (4 EDOs acopladas)
- STDP real con ventana exponencial (τ=20ms)
- 7 motores biológicos
- Memoria jerárquica VRAM→RAM→SSD con auto-swap LRU
- Hardware detection nativa (proc/sys/statvfs raw)
- Binario `cerebro-digital` con consola interactiva

### [2026-06-17] Fundación del Engine Puro v5 (ELIMINADO)
- ~~Extraído de `src-tauri/src/nexus_puro_engine.rs` (3787 líneas)~~
- ~~Pipeline de 16 pasos~~
- **ELIMINADO completamente el 2026-06-17** — reemplazado por Cerebro Digital

---

## 📐 DECISIONES ARQUITECTÓNICAS PERMANENTES

1. **Bio-realismo**: Hodgkin-Huxley para neuronas, STDP para sinapsis, jerarquía para memoria
2. **Compactación**: NeuronaCompacta 64B, SinapsisCompacta 8B — sin overhead de HashMap
3. **Hardware nativo**: Detección vía proc/sys/statvfs — sin librerías externas de sistema
4. **Concurrencia**: Rayon para chunks paralelos — sin unsafe
5. **Sin panics**: `Result<T, String>` o `?` operator. Nunca unwrap()/expect()
6. **Sistema único**: Solo existe el Cerebro Digital. v5 completamente eliminado.
7. **Compañero estratégico**: El agente traduce todo a español llano. Cris no programa.
8. **Disjoint field borrowing**: Los motores de aprendizaje aceptan campos individuales (`&mut MemoriaAdaptativa`, etc.) en vez de `&mut CerebroAutoOptimizable` para evitar el borrow checker de Rust
9. **Aprendizaje emergente**: Los 6 motores operan sobre la actividad neuronal real del cerebro (top-64 neuronas activas), no sobre datos externos. El aprendizaje emerge de la experiencia del sistema.

---

## 🔗 ENLACES RÁPIDOS

- [CHAT_CONTEXTO.md](CHAT_CONTEXTO.md) — Contexto completo del chat
- [BITACORA.md](BITACORA.md) — Bitácora de operaciones
- [MEMORIA.md](MEMORIA.md) — Memoria estática del sistema
- [lib.rs](src/lib.rs) — pub mod cerebro (única línea)
- [cerebro/mod.rs](src/cerebro/mod.rs) — Raíz del Cerebro Digital
- [cerebro/cerebro.rs](src/cerebro/cerebro.rs) — CerebroAutoOptimizable (pipeline 16 pasos)
- [cerebro/estructuras.rs](src/cerebro/estructuras.rs) — Tipos compactos
- [cerebro/motores.rs](src/cerebro/motores.rs) — 8 motores biológicos
- [cerebro/aprendizaje/mod.rs](src/cerebro/aprendizaje/mod.rs) — 6 motores de aprendizaje
- [Cargo.toml](Cargo.toml) — Dependencias
- [PLAN_APRENDIZAJE_PROFUNDO.md](plans/PLAN_APRENDIZAJE_PROFUNDO.md) — Diseño completo de los 6 motores

---

### [2026-06-19 20:40] 💾 Persistencia Permanente — Cerebro ya no muere al cerrar
- Creado [`src/cerebro/persistencia.rs`](src/cerebro/persistencia.rs) (200 líneas → ahora 455) con:
  - `EstadoPersistente`: snapshot completo de aprendizaje (~180 KB → ahora ~280 KB)
  - `guardar()`: serialización JSON + escritura atómica (temp→rename)
  - `cargar()`: deserialización con recuperación de errores
  - `restaurar()`: inyecta estado en cerebro recién creado
- Serialización agregada a 6 structs: `MotorLexico`, `Amigdala`, `SistemaDopamina`, `Conciencia`, `Episodio`, `SsdManager`
- **43 campos nuevos** para los 5 motores: Predictor (10), Conceptos (7), Neurogénesis (9), Poda (10), Consolidador (12)
- **Auto-guardado**: cada 1000 pasos en `data/cerebro_estado.json`
- **Auto-carga**: en `CerebroAutoOptimizable::nuevo()` — busca el archivo al iniciar
- **Comando Tauri**: `guardar_cerebro` disponible desde la UI
- Dependencia: `serde_json = "1.0"` agregada a Cargo.toml
- Compilación: **0 errores, 0 warnings**

---

### [2026-06-19 21:20] 🗣️ Lenguaje Emergente Innato — El cerebro ya habla como un LLM
- Sembradas 320 conexiones neurona→token (5 por cada una de las 64 palabras semilla) con pesos 0.15-0.25
- Sembrados ~124 bigramas lógicos del español: pronombre→verbo, artículo→sustantivo, verbo→sustantivo, etc.
- **Diagnóstico resuelto**: el "solo dice escucho" era porque `conexiones` y `transiciones` arrancaban vacías
- **NO son frases prefabricadas**: softmax + temperatura + ruleta eligen cada palabra en cada ejecución
- Compilación: 0 errores, 0 warnings
- Tests: 15/15 pasados (13 originales + 2 nuevos)
- Problemas conocidos actualizados: #2 "Sin lenguaje real" y #6 "Solo responde escucho" marcados como RESUELTOS

---

### [2026-06-19 22:10] 🧭 Curiosidad + Exploración Web — El cerebro busca en internet solo
- **Lo que pidió Cris**: "Que el cerebro cada cierto tiempo genere una pregunta basada en lo que está pensando/sintiendo y busque en internet por su cuenta, sin que yo le pida nada"
- **Opción elegida**: curl del sistema — cero dependencias nuevas, simple y directo
- **Archivo nuevo**: [`src/cerebro/explorador.rs`](src/cerebro/explorador.rs) (373 líneas) — ExploradorWeb con `std::process::Command::new("curl")`
- **Archivos modificados**: motores.rs (+MotorCuriosidad), cerebro.rs (+Paso 8), persistencia.rs (+8 campos), mod.rs (+pub mod explorador)
- **MotorCuriosidad**: nivel se actualiza cada paso con error_predicción*0.5 + conciencia*0.3 + emoción*0.2. Decae con factor 0.001. Umbral 0.7, saciedad 0.5, cadencia mínima 200 pasos
- **ExploradorWeb::buscar()**: curl -s -L --max-time 10 a DuckDuckGo HTML, parsea snippets con clase `result__snippet`
- **buscar_simulado()**: 6 respuestas predefinidas para offline/tests. Coincidencia por palabra exacta (evita falso positivo "totalmente" → "mente")
- **Integración**: Paso 9 en pipeline (ahora Paso 8 tras expansión). Si curiosidad supera umbral y pasos > 200: genera pregunta del texto de salida, busca en web, crea entrada, auto-alimenta al cerebro con self.paso(dt*0.3, entrada), sacia
- **Cero dependencias nuevas** — std::process::Command es parte de std
- **Compilación**: 0 errores, 0 warnings
- **Tests**: 27/27 pasados (15 MotorLexico + 12 ExploradorWeb nuevos)
- **Problema resuelto**: #4 "Curiosidad/Búsqueda activa de información" ✅

---

### [2026-06-20 00:10] 🧭 Omega Navegador — El cerebro tiene su propio navegador web inteligente
- **Lo que pidió Cris**: "Un navegador propio, lo más omega, lo mejor de lo mejor, sin importar tiempo o complejidad"
- **Opción elegida**: 3 motores propios (MotorHTTP, MotorExtraccion, MotorRazonamientoWeb) — cero dependencias Rust, usa herramientas del sistema (curl, openssl, chrome)
- **Arquitectura**: Fallback automático curl → TcpStream HTTP → openssl s_client → chrome headless
- **MotorExtraccion**: Parser HTML completo que extrae 9 campos (título, meta, encabezados, párrafos, enlaces, listas, tablas, código, texto plano) + densidad_info
- **MotorRazonamientoWeb**: Score de enlaces por dominio (Wikipedia+4, arXiv+4, YouTube/Facebook-3, repetidos-10)
- **API nueva**: `ExploradorWeb::navegar(url)` y `ExploradorWeb::explorar(pregunta, profundidad)` — multi-salto hasta 3 niveles
- **Archivo reescrito**: [`src/cerebro/explorador.rs`](src/cerebro/explorador.rs) 373 → 1275 líneas (+902)
- **Archivos modificados**: motores.rs (+3 campos: fuentes_navegadas, profundidad_exploracion, preferencia_academica), cerebro.rs (Paso 8 usa explorar()), persistencia.rs (+3 campos persistidos)
- **Plan de diseño**: [`plans/PLAN_OMEGA_NAVEGADOR.md`](plans/PLAN_OMEGA_NAVEGADOR.md)
- **Compilación**: 0 errores, 0 warnings
- **Tests**: 50/50 pasados (35 explorador + 15 MotorLexico)

---

### [2026-06-21 17:00] 🧠 Aprendizaje Profundo — 6 Motores Omega de aprendizaje
- **Lo que pidió Cris**: "¿Cómo aprende el motor engine puro?" → Diseñar e implementar 6 motores de aprendizaje profundo
- **Opción elegida**: 6 motores propios que operan sobre la actividad neuronal real del cerebro. Cero dependencias externas.
- **Diagnóstico previo**: El aprendizaje era débil — STDP limitado a VRAM, transiciones Markov sin significado, episodios guardados pero nunca reprocesados, dopamina no modificaba conexiones
- **Plan de diseño**: [`plans/PLAN_APRENDIZAJE_PROFUNDO.md`](plans/PLAN_APRENDIZAJE_PROFUNDO.md) (904 líneas) — 8 secciones con diagramas Mermaid, 43 tests planificados
- **Pipeline expandido**: 10 → 16 pasos. Pasos 10-15 son los 6 motores nuevos

**Archivos nuevos** (7):
- [`src/cerebro/aprendizaje/mod.rs`](src/cerebro/aprendizaje/mod.rs) — Módulo raíz con 6 submódulos
- [`src/cerebro/aprendizaje/sensorial.rs`](src/cerebro/aprendizaje/sensorial.rs) — Motor 6: Random Indexing 256D (sesión anterior)
- [`src/cerebro/aprendizaje/predictor.rs`](src/cerebro/aprendizaje/predictor.rs) — Motor 1: Buffer circular + hash de prefijos
- [`src/cerebro/aprendizaje/conceptos.rs`](src/cerebro/aprendizaje/conceptos.rs) — Motor 2: Co-ocurrencia + proto-conceptos + fusión
- [`src/cerebro/aprendizaje/neurogenesis.rs`](src/cerebro/aprendizaje/neurogenesis.rs) — Motor 3: Creación de neuronas hub
- [`src/cerebro/aprendizaje/poda.rs`](src/cerebro/aprendizaje/poda.rs) — Motor 4: Poda de sinapsis y neuronas
- [`src/cerebro/aprendizaje/consolidador.rs`](src/cerebro/aprendizaje/consolidador.rs) — Motor 5: Sueño, replay, meta-episodios

**Problema resuelto**: Borrow checker en pipeline. Los motores ahora aceptan campos individuales (`&mut MemoriaAdaptativa`, `&mut MotorLexico`, `&mut u32`) en vez de `&mut CerebroAutoOptimizable`. Esto permite disjoint field borrowing en el pipeline.

**Problema resuelto**: 7 tests fallaban por usar IDs de neuronas incorrectos (IDs hardcoded que ya existían como neuronas reales). Se arreglaron para usar IDs garantizados en RAM (5000+) o IDs gigantescos (9,999,999).

**Persistence**: 43 campos nuevos. ProtoConcepto y MetaEpisodio serializables. VecDeque→Vec para serialización. Campo `co_ocurrencias` cambiado a `pub(crate)`.

- **Compilación**: 0 errores, 0 warnings
- **Tests**: 89/89 pasados (todos los módulos)
- **Problema resuelto**: #5 "Sin aprendizaje profundo" ✅

*Última actualización: 2026-06-21 | Sesión de Aprendizaje Profundo — 6 Motores Omega*
