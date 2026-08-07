# 📡 BITACORA.md — Cerebro Digital Dinámico v1

> Bitácora viva de operaciones. Registro cronológico de eventos, cambios y estado del sistema.

---

## 2026-06-17 22:15 🎯 REGISTRO DE REGLA FUNDAMENTAL

### Resumen
Cris informa que **NO entiende de código**. El agente se registra como su **compañero estratégico** de operación.

### Cambios
- [`memoria/agente_memoria.md`](memoria/agente_memoria.md): Perfil del Arquitecto actualizado con regla de comunicación
- [`CHAT_CONTEXTO.md`](CHAT_CONTEXTO.md): Nueva sección "Reglas de Interacción con el Arquitecto"
- Documentación corregida: eliminadas todas las referencias al legado v5

### Decisión
- Toda comunicación técnica DEBE traducirse a español llano
- NUNCA asumir conocimiento previo de programación
- Usar analogías y metáforas para explicar conceptos
- Cris es la autoridad final en todas las decisiones

---

## 2026-06-17 22:00 🧹 LIMPIEZA TOTAL — SOLO CEREBRO DIGITAL

### Archivos Eliminados (13 .rs + 4 directorios)
- `src/lib.rs` (v5), `src/main.rs`, `src/motor_fonacion.rs` (1168 líneas)
- `src/motor_identidad.rs` (816), `motor_corteza_prefrontal.rs` (645), `motor_grafo.rs` (366)
- `src/motor_transformer.rs` (662, código muerto), `motor_atencion.rs`, `motor_bioquimica.rs`
- `motor_inhibicion.rs`, `motor_ingesta.rs`, `motor_memoria.rs`, `motor_memoria_operativa.rs`
- `motor_prediccion.rs`, `data/`, `scripts/`, `brain/`, `.agent/`, `target/`

### Resultado
- Compilación limpia: 0 errores, 0 warnings
- Ejecución verificada: detecta 20 CPUs, 66GB RAM, 8GB VRAM NVIDIA, 696GB SSD
- 100K neuronas simuladas con Hodgkin-Huxley en paralelo

---

## 2026-06-17 18:00 🧠 FUNDACIÓN DEL CEREBRO DIGITAL DINÁMICO

### Resumen
Arquitectura biológicamente inspirada con Hodgkin-Huxley, STDP, memoria jerárquica VRAM→RAM→SSD y 7 motores biológicos.

### Archivos Creados
- [`src/cerebro/mod.rs`](src/cerebro/mod.rs), [`estructuras.rs`](src/cerebro/estructuras.rs) (302 líneas), [`hardware.rs`](src/cerebro/hardware.rs) (325 líneas)
- [`motores.rs`](src/cerebro/motores.rs) (531 líneas), [`memoria.rs`](src/cerebro/memoria.rs) (376 líneas), [`cerebro.rs`](src/cerebro/cerebro.rs) (417 líneas)
- [`src/bin/cerebro.rs`](src/bin/cerebro.rs) (124 líneas)

---

## 2026-06-19 20:40 💾 PERSISTENCIA PERMANENTE — El cerebro ya recuerda entre sesiones

### Archivos Creados
- [`src/cerebro/persistencia.rs`](src/cerebro/persistencia.rs) (200 líneas) — Guardado/carga JSON atómico
- `data/.gitkeep`

### Archivos Modificados
- `cerebro.rs`, `mod.rs`, `estructuras.rs`, `motores.rs`, `memoria.rs`, `motor_lexico.rs`
- `src-tauri/src/lib.rs`, `Cargo.toml` (+serde_json)

### Resultado
- 0 errores, 0 warnings. Persistencia: vocabulario, emociones, episodios, contadores (~180 KB)
- Auto-guardado cada 1000 pasos → auto-carga al próximo inicio

---

## 2026-06-19 21:20 🗣️ LENGUAJE EMERGENTE INNATO — El cerebro ya habla como un LLM

### Resumen
Se sembraron conexiones neurona→palabra y transiciones palabra→palabra en el Motor Léxico Sinclair para que el cerebro genere lenguaje real desde el primer paso, sin depender de acumular interacciones para aprender.

### Diagnóstico
El cerebro solo decía "escucho" porque las matrices `conexiones` y `transiciones` arrancaban vacías. El score de cada token era 0.0, nunca superaba el umbral de 0.01, y el loop de auto-alimentación en Tauri estaba roto porque requería texto generado que nunca se generaba.

### Archivos Modificados
- [`src/cerebro/lexico/motor_lexico.rs`](src/cerebro/lexico/motor_lexico.rs): Constructor `nuevo()` — siembra de 320 conexiones + 124 bigramas lógicos del español, +2 tests nuevos

### Detalles Técnicos
- **Conexiones innatas**: Cada token `j` conectado a 5 neuronas con pesos 0.15-0.25
- **Bigramas lógicos**: ~124 transiciones entre categorías gramaticales (pronombre→verbo, artículo→sustantivo, etc.)
- **NO son frases prefabricadas**: Softmax + temperatura + ruleta eligen cada palabra en cada ejecución

### Resultado
- Compilación: **0 errores, 0 warnings**
- Tests: **15/15 pasados** (13 originales + 2 nuevos)
- El cerebro ahora genera frases emergentes construidas palabra por palabra con softmax + Markov

---

---

## 2026-06-19 22:10 🧭 CURIOSIDAD + EXPLORACIÓN WEB — El cerebro ya busca en internet solo

### Resumen
Se agregó el **Motor 8: Curiosidad** que genera preguntas basadas en el estado interno del cerebro (error de predicción, conciencia, emociones) y busca respuestas en internet usando DuckDuckGo vía `curl`. El cerebro ahora explora activamente, no espera pasivamente.

### Archivos Creados
- [`src/cerebro/explorador.rs`](src/cerebro/explorador.rs) (373 líneas) — ExploradorWeb con curl a DuckDuckGo, parseo HTML, modo simulado para offline

### Archivos Modificados
- [`src/cerebro/motores.rs`](src/cerebro/motores.rs) (+135 líneas) — Motor 8: Curiosidad con 8 campos: nivel, umbral (0.7), saciedad (0.5), cadencia (200 pasos), decaimiento (0.001)
- [`src/cerebro/cerebro.rs`](src/cerebro/cerebro.rs) — Paso 8 en pipeline: 1. actualizar curiosidad, 2. si supera umbral → generar pregunta, 3. buscar en web, 4. auto-alimentar resultado como entrada, 5. saciar
- [`src/cerebro/persistencia.rs`](src/cerebro/persistencia.rs) — 8 campos nuevos de curiosidad guardados/cargados en JSON
- [`src/cerebro/mod.rs`](src/cerebro/mod.rs) — `pub mod explorador;`

### Detalles Técnicos
- **MotorCuriosidad**: nivel se actualiza con: novedad*0.5 + conciencia*0.3 + emoción*0.2. Decae naturalmente con factor 0.001. Umbral 0.7 para explorar. Saciedad 0.5 (reduce nivel 50% tras explorar).
- **ExploradorWeb::buscar()**: Ejecuta `curl -s -L --max-time 10 "https://html.duckduckgo.com/html/?q=PREGUNTA"` (std::process::Command — cero dependencias nuevas)
- **buscar_simulado()**: 6 respuestas predefinidas para tests y modo offline. Coincidencia por palabra exacta para evitar falsos positivos.
- **limpiar_fragmento()**: Parser HTML manual con estados (tag/entidad/texto) — decodifica &, <, >, ", ', &nbsp;
- **Integración**: Si curiosidad > umbral y pasos > cadencia (200), genera pregunta del texto de salida → busca → crea Entrada → llama self.paso(dt*0.3, entrada) → sacia

### Resultado
- Compilación: **0 errores, 0 warnings**
- Tests: **27/27 pasados** (15 MotorLexico + 12 ExploradorWeb)
- El cerebro ahora genera preguntas solo cuando tiene "hambre de saber" y las busca en internet por su cuenta
- Cero dependencias nuevas — todo con std::process::Command

---

## 2026-06-20 00:10 🧭 OMEGA NAVEGADOR — El cerebro ya tiene su propio navegador web inteligente

### Resumen
Se reemplazó el simple `curl` a DuckDuckGo por el **Navegador Propio Omega**: 3 motores que trabajan en cadena para navegar, extraer y razonar sobre páginas web. El cerebro ahora puede navegar múltiples sitios (hasta 3 saltos de profundidad), extraer contenido estructurado, y decidir qué enlaces seguir según puntuación de relevancia.

### Arquitectura — 3 Motores Omega
1. **MotorHTTP** (`obtener_inteligente()`): 4 niveles de obtención con fallback automático
   - Nivel 1: `curl -sL` (rápido, confiable)
   - Nivel 2: `TcpStream` HTTP 1.1 raw (sin curl)
   - Nivel 3: `openssl s_client` HTTPS raw (sin curl)
   - Nivel 4: `google-chrome --headless --dump-dom` (con JavaScript)
2. **MotorExtraccion** (`extraer()`): Parser HTML completo que extrae 9 campos estructurados
   - título, descripción meta, encabezados H1-H6, párrafos, enlaces, listas, tablas, código, texto plano
   - Métrica `densidad_info` (0.0-1.0) para medir qué tan rico en información es cada fragmento
3. **MotorRazonamientoWeb** (`razonar()`): Extrae URLs de resultados de búsqueda y las puntúa
   - Wikipedia/arXiv: +4 puntos, YouTube/Facebook: -3, repetidas: -10
   - Máximo 5 enlaces por búsqueda, ordenados por puntuación

### Estructuras Nuevas
- `PaginaWeb` — Representación estructurada con 11 campos + `densidad_info`
- `Enlace` — href, texto, dominio

### API Pública
- `ExploradorWeb::navegar(url)` → `Result<PaginaWeb, String>` — Navega a cualquier URL
- `ExploradorWeb::explorar(pregunta, profundidad)` → `Result<(String, Vec<PaginaWeb>), String>` — Búsqueda multi-salto

### Archivos Modificados
- [`src/cerebro/explorador.rs`](src/cerebro/explorador.rs) — **REESCRITO**: 373 → 1275 líneas (+902), 3 motores nuevos, 4 métodos públicos, ~30 tests
- [`src/cerebro/motores.rs`](src/cerebro/motores.rs) — +3 campos a MotorCuriosidad: `fuentes_navegadas`, `profundidad_exploracion` (2), `preferencia_academica` (0.6)
- [`src/cerebro/cerebro.rs`](src/cerebro/cerebro.rs) — Paso 8 actualizado: usa `explorar()` multi-salto, trackea fuentes navegadas, síntesis multi-página como entrada
- [`src/cerebro/persistencia.rs`](src/cerebro/persistencia.rs) — +3 campos persistidos de curiosidad Omega

### Archivos Creados
- [`plans/PLAN_OMEGA_NAVEGADOR.md`](plans/PLAN_OMEGA_NAVEGADOR.md) — Documento de diseño arquitectónico completo

### Resultado
- Compilación: **0 errores, 0 warnings**
- Tests: **50/50 pasados** (50 explorador + 15 MotorLexico)
- El cerebro ahora navega la web como un investigador autónomo: busca → extrae → puntúa → sigue mejores enlaces → sintetiza

---

## 2026-06-21 17:00 🧠 APRENDIZAJE PROFUNDO — 6 Motores Omega de aprendizaje

### Resumen
El cerebro ahora tiene **6 motores de aprendizaje profundo** que convierten datos crudos en patrones significativos: predice el futuro, forma conceptos abstractos, crea neuronas especializadas, poda conexiones muertas, consolida recuerdos mientras "duerme", y transforma texto en vectores semánticos. Pasamos de 10 a **16 pasos en el pipeline**.

### Arquitectura — 6 Motores Omega de Aprendizaje

| # | Motor | Archivo | Función |
|---|-------|---------|---------|
| 1 | **Predictor Temporal** | [`predictor.rs`](src/cerebro/aprendizaje/predictor.rs) | Buffer circular de 32 estados. Predice las próximas neuronas activas. Error de predicción → dopamina. Tasa de acierto |
| 2 | **Formador de Conceptos** | [`conceptos.rs`](src/cerebro/aprendizaje/conceptos.rs) | Matriz de co-ocurrencia entre tokens. Agrupa tokens que aparecen juntos en "conceptos". Fusión automática |
| 3 | **Neurogénesis** | [`neurogenesis.rs`](src/cerebro/aprendizaje/neurogenesis.rs) | Crea neuronas hub para tokens frecuentes o conceptos sin representación. Conexiones bidireccional al léxico |
| 4 | **Poda Homeostática** | [`poda.rs`](src/cerebro/aprendizaje/poda.rs) | Elimina sinapsis débiles (<0.01). Poda neuronas inactivas (>10000 pasos). Protege neuronas jóvenes (<1000 pasos) |
| 5 | **Consolidador Nocturno** | [`consolidador.rs`](src/cerebro/aprendizaje/consolidador.rs) | Ciclo de sueño cada 5000 pasos, duración 500 pasos. Replay de episodios. Generalización en meta-episodios |
| 6 | **Pipeline Sensorial** | [`sensorial.rs`](src/cerebro/aprendizaje/sensorial.rs) | Random Indexing: 256 dimensiones, 8 sparse. Mapea token_id→vector semántico. Similitud por coseno |

### Pipeline expandido (10 → 16 pasos)
```
Paso 0:  Hardware check
Paso 1:  Hodgkin-Huxley (Rayon paralelo)
Paso 2:  STDP (ventana temporal exponencial)
Paso 3:  Atención Selectiva (saliency)
Paso 4:  Dopamina (reward prediction error)
Paso 5:  Amígdala (miedo/ira/alegría)
Paso 6:  Hipocampo (episodios + olvido)
Paso 7:  Fonación (generación de habla)
Paso 8:  Curiosidad + Exploración Web Omega
Paso 9:  Registro emocional
Paso 10: Actualizar sensorial (Random Indexing)
Paso 11: 🆕 Predictor Temporal (registrar → predecir → error)
Paso 12: 🆕 Formador de Conceptos (co-ocurrencia → agrupación)
Paso 13: 🆕 Neurogénesis (crear neuronas hub)
Paso 14: 🆕 Poda Homeostática (limpiar conexiones muertas)
Paso 15: 🆕 Consolidador Nocturno (sueño si corresponde)
```

### Archivos Creados
- [`src/cerebro/aprendizaje/mod.rs`](src/cerebro/aprendizaje/mod.rs) — Módulo raíz con 6 submódulos
- [`src/cerebro/aprendizaje/sensorial.rs`](src/cerebro/aprendizaje/sensorial.rs) (193 líneas) — MotorSensorial (Random Indexing 256D)
- [`src/cerebro/aprendizaje/predictor.rs`](src/cerebro/aprendizaje/predictor.rs) (315 líneas) — MotorPrediccion (buffer circular, hash de prefijos)
- [`src/cerebro/aprendizaje/conceptos.rs`](src/cerebro/aprendizaje/conceptos.rs) (308 líneas) — MotorConceptos (co-ocurrencia, proto-conceptos)
- [`src/cerebro/aprendizaje/neurogenesis.rs`](src/cerebro/aprendizaje/neurogenesis.rs) (208 líneas) — MotorNeurogenesis (creación de neuronas hub)
- [`src/cerebro/aprendizaje/poda.rs`](src/cerebro/aprendizaje/poda.rs) (301 líneas) — MotorPoda (poda de sinapsis y neuronas)
- [`src/cerebro/aprendizaje/consolidador.rs`](src/cerebro/aprendizaje/consolidador.rs) (414 líneas) — MotorConsolidacion (sueño, replay, meta-episodios)
- [`plans/PLAN_APRENDIZAJE_PROFUNDO.md`](plans/PLAN_APRENDIZAJE_PROFUNDO.md) (904 líneas) — Plan de diseño completo con Mermaid

### Archivos Modificados
- [`src/cerebro/cerebro.rs`](src/cerebro/cerebro.rs) — +6 campos (motor_poda, motor_predictor, motor_consolidacion, motor_conceptos, motor_neurogenesis, motor_sensorial). Pipeline expandido a 16 pasos (Pasos 10-15). Disjoint field borrowing para evitar borrow checker
- [`src/cerebro/persistencia.rs`](src/cerebro/persistencia.rs) — **REESCRITO**: de 226 a 455 líneas. **43 campos nuevos** de persistencia para los 5 motores (Predictor:10, Conceptos:7, Neurogénesis:9, Poda:10, Consolidador:12)
- [`src/cerebro/mod.rs`](src/cerebro/mod.rs) — +`pub mod aprendizaje;`

### Detalles Técnicos

**Motor 1 — Predictor Temporal**: Buffer circular (32 estados, top-64 neuronas cada uno). Hash de prefijo (16 entradas) → busca continuaciones en HashMap. Si encuentra, promedia. LRU eviction (max 100/bucket). Error normalizado 0.0-1.0. Tasa de acierto = predicciones con error < 0.15.

**Motor 2 — Formador de Conceptos**: `HashMap<(u32, u32), u32>` de co-ocurrencia. Ventana de contexto=5 tokens. Umbral=10 co-ocurrencias para formar proto-concepto. Fusión de conceptos si comparten miembros. `ProtoConcepto` serializable con `#[derive(Serialize, Deserialize)]`.

**Motor 3 — Neurogénesis**: Crea neuronas en capa 2 para conceptos sin representación o tokens frecuentes (>5 veces/1000 pasos). Conexión bidireccional al léxico (peso 0.3-0.5). Máx 10000 neuronas creadas. Helper `crear_neurona_en_memoria()` replica crear_neurona sin &mut CerebroAutoOptimizable.

**Motor 4 — Poda Homeostática**: Elimina sinapsis con |peso| < 0.01, máx 256/neurona. Poda neuronas con frecuencia < 0.01 Hz por >10000 pasos. Jóvenes (<1000 pasos) protegidas. Máx 100 eliminaciones/ciclo. API acepta `&mut MemoriaAdaptativa` (no `&mut CerebroAutoOptimizable`) para evitar borrow checker.

**Motor 5 — Consolidador Nocturno**: Ciclo de sueño cada 5000 pasos, dura 500 pasos. Selecciona top-20 episodios del SSD. Reproduce patrones y refuerza sinapsis (STDP-like). Generalización: meta-episodios de patrones en 3+ episodios con >50% similitud. API acepta `(&mut MemoriaAdaptativa, &ParametrosNeurona, usize, &mut MotorLexico, f32)` — disjoint fields.

**Motor 6 — Pipeline Sensorial**: Random Indexing: 256 dimensiones, 8 elementos no-cero por vector. `base_neurona=10000`, `grupo_por_neurona=8`. Activa neuronas cerca de base_neurona en grupos de 8 cuando un token aparece. Coseno para similitud semántica. `tokens_similares()` → top-N tokens más cercanos.

### Problemas resueltos del borrow checker
- **Triple borrow** en Neurogénesis: `self.motor_neurogenesis.procesar(&mut self.memoria, &mut self.siguiente_id, &mut self.motor_lexico)` — 3 disjoint refs
- **Double borrow** en Poda: `self.motor_poda.ejecutar(&mut self.memoria)` — disjoint ref
- **Double borrow** en Consolidador: `consolidacion.paso_suenio(&mut self.memoria, &self.params_neurona, self.config.hilos_cpu, &mut self.motor_lexico, dt)` — 4 disjoint refs
- Estrategia: APIs que aceptan campos individuales en vez de `&mut CerebroAutoOptimizable`

### Resultado
- Compilación: **0 errores, 0 warnings**
- Tests: **89/89 pasados** (50 explorador + 15 MotorLexico + 8 sensorial + 8 poda + 8 predictor + 8 conceptos + 8 consolidador + 7 neurogenesis + 2 cerebro)
- El cerebro ahora **predice su propia actividad futura**, **forma conceptos abstractos** de tokens que aparecen juntos, **crea neuronas especializadas** para lo que más usa, **poda conexiones muertas** como un cerebro biológico, **consolida recuerdos mientras duerme**, y **entiende relaciones semánticas** entre palabras mediante vectores de 256 dimensiones

---

## 2026-06-17 16:00 📦 PAQUETE SOBERANO (Legado v5)

## 2026-06-17 15:00 🧬 FUNDACIÓN DEL ENGINE PURO v5 (Legado)
