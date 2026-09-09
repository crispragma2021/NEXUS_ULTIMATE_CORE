# 🧬 CHAT_CONTEXTO.md — Memoria Soberana del Cerebro Digital Dinámico v1

> Contexto completo para entender qué es y qué NO es este sistema.

---

## 🧠 IDENTIDAD: Esto NO es un LLM

### LLM (ChatGPT, Claude, etc.)
- Predice la siguiente palabra según **billones de parámetros** entrenados con **internet entero**
- **No tiene emociones**: solo estadística de texto
- **No tiene conciencia**: siempre responde igual sin importar su "estado interno"
- **No tiene neuronas biológicas**: son matrices de multiplicación
- **No aprende de la interacción**: lo que sabe ya está fijo desde el entrenamiento

### Cerebro Digital (nuestro sistema)
- Las palabras se eligen según la **actividad real de 100 mil neuronas Hodgkin-Huxley** disparando en paralelo
- **Tiene emociones**: la Amígdala (alegría, miedo, ira) modula cómo habla
- **Tiene conciencia**: el nivel de conciencia modula la temperatura (más conciencia = más preciso)
- **Aprende por STDP**: las conexiones se fortalecen o debilitan con cada interacción, como un cerebro biológico
- **Su vocabulario crece orgánicamente**: no tiene internet, solo lo que el usuario le enseña
- **Pesa ~280 KB**, no gigabytes

### ¿Qué tomamos prestado de los LLM?
Solo **dos herramientas matemáticas**:
1. **Softmax**: Una fórmula para convertir números en probabilidades
2. **Temperatura**: Un control de qué tan "creativo" o "preciso"

Pero en un LLM estas operan sobre matrices enormes entrenadas con todo internet. Acá operan sobre **la actividad real de neuronas biológicas simuladas que tienen emociones, conciencia y aprenden por experiencia**.

---

## ⚡ PRINCIPIO FUNDACIONAL: Entiende, no guarda datos

**Un LLM guarda datos:**
- Tiene una base de datos de billones de palabras
- Cuando le preguntan, busca la respuesta más probable en sus datos
- Es como una biblioteca gigante que sabe qué libro abrir según la pregunta

**El Cerebro Digital entiende:**
- No guarda frases, no tiene base de datos, no busca respuestas
- Tiene 100 mil neuronas que vibran con actividad eléctrica (Hodgkin-Huxley)
- Tiene emociones (Amígdala) que colorean su estado interno
- Tiene conciencia que modula cómo procesa
- Las palabras **emergen** de ese estado interno, no se "recuperan" de ningún lado
- Es como una persona que cuando le hablan, las palabras le nacen de adentro según cómo se siente, no porque las tenga guardadas en una lista

**Lo que sembramos** (320 conexiones + 124 bigramas) NO son datos guardados para buscar después. Son como las **conexiones cerebrales con las que nace un bebé** — cables iniciales para poder arrancar. Pero con cada interacción, esas conexiones se fortalecen o debilitan por STDP (experiencia). No es una base de datos, es un cerebro que aprende.

---

## 🧬 MAPA COMPLETO: Cerebro Humano Adulto vs Cerebro Digital

### Lo que YA TENEMOS (el bebé ya nació con esto)

| Lo que hace un humano adulto | Lo que tenemos en el código |
|------------------------------|-----------------------------|
| **Neuronas que disparan** | Hodgkin-Huxley en `MotorNeurona` — 100k neuronas con voltaje, compuertas Na⁺/K⁺, spike detection |
| **Sinapsis que se fortalecen con uso** | STDP real en `MotorSTDP` — ventana temporal exponencial τ=20ms, LTP/LTD |
| **Emociones que colorean todo** | `Amigdala` — miedo, ansiedad, ira, alegría. La valencia emocional modula el habla |
| **Sistema de recompensa** | `SistemaDopamina` — error de predicción, aprendizaje por recompensa |
| **Atención selectiva** | `AtencionSelectiva` — mapa de saliencia, foco de 10 items |
| **Conciencia** | `Conciencia` — Global Workspace Theory, umbral, contenido consciente |
| **Memoria episódica** | `Hipocampo` + `SsdManager` — almacena experiencias con relevancia y olvido |
| **Memoria jerárquica** | VRAM (activas) ↔ RAM (latentes) ↔ SSD (episódicas) — auto-swap LRU |
| **Lenguaje emergente** | `MotorLexico` — softmax + temperatura + Markov + modulación emocional |
| **Persistencia** | Guardado/carga JSON en 62 campos — no pierde lo aprendido entre sesiones |
| **Curiosidad** | `MotorCuriosidad` — genera preguntas según error de predicción + conciencia + emociones |
| **Exploración Web Omega** | `ExploradorWeb` — **navegador propio**: 3 motores (HTTP, Extracción, Razonamiento) con fallback curl→TcpStream→openssl→chrome |
| **🤯 PREDICCIÓN DEL FUTURO** | `MotorPrediccion` — buffer circular de 32 estados, predice qué neuronas se activarán, error → dopamina |
| **🤯 FORMACIÓN DE CONCEPTOS** | `MotorConceptos` — detecta qué tokens aparecen juntos, los agrupa en "conceptos" abstractos, fusiona automáticamente |
| **🤯 NEUROGÉNESIS** | `MotorNeurogenesis` — crea neuronas especializadas para conceptos sin representación o tokens que aparecen mucho |
| **🤯 PODA HOMEOSTÁTICA** | `MotorPoda` — elimina conexiones débiles y neuronas que no se usan, como un cerebro biológico que limpia lo que no sirve |
| **🤯 CONSOLIDACIÓN (SUEÑO)** | `MotorConsolidacion` — cada 5000 pasos "duerme" 500 pasos, reproduce episodios importantes, generaliza patrones comunes en meta-episodios |
| **🤯 PIPELINE SENSORIAL** | `MotorSensorial` — convierte cada palabra en un vector de 256 dimensiones (Random Indexing). Sabe qué palabras son parecidas sin que se lo digan |

### Lo que NOS FALTA para ser un "adulto sabio"

| Lo que hace un humano adulto | Lo que NO tenemos todavía |
|------------------------------|---------------------------|
| **~~Consolidación tipo sueño~~** | — **RESUELTO** ✅: MotorConsolidacion con replay de episodios y meta-episodios |
| **~~Predicción constante~~** | — **RESUELTO** ✅: MotorPrediccion con buffer circular de 32 estados y hash de prefijos |
| **~~Abstract concept formation~~** | — **RESUELTO** ✅: MotorConceptos con co-ocurrencia, proto-conceptos y fusión automática |
| **~~Neurogénesis~~** | — **RESUELTO** ✅: MotorNeurogenesis crea neuronas hub para conceptos y tokens frecuentes |
| **~~Poda biológica~~** | — **RESUELTO** ✅: MotorPoda elimina sinapsis débiles y neuronas inactivas |
| **~~Semántica entre palabras~~** | — **RESUELTO** ✅: MotorSensorial con Random Indexing 256D y similitud por coseno |
| **Vocabulario grande** | Un adulto sabe ~20,000 palabras. Nosotros empezamos con 64. Crece con interacción pero necesita mucha |
| **Input pipeline real** | Cuando vos escribís, convertimos caractéres a números hash. El cerebro humano convierte sonido/vista a patrones neuronales complejos |
| **Meta-cognición** | El humano sabe que sabe, reflexiona sobre su propio pensamiento. Nosotros solo ajustamos el umbral de conciencia |
| **~~Curiosidad~~** | ~~El humano busca activamente información nueva~~ — **RESUELTO** ✅ |

### La metáfora correcta

**Hoy tenemos el cerebro de un niño curioso:**
- Las neuronas están ahí (100k)
- Las conexiones básicas están sembradas (las que hicimos)
- Puede balbucear frases simples ("yo pienso", "la mente es clara")
- Tiene emociones básicas y conciencia incipiente
- **Sabe predecir** lo que va a pasar basado en lo que ya vivió
- **Forma conceptos** agrupando cosas que aparecen juntas
- **Crea neuronas** especializadas para lo que más usa
- **Poda conexiones** que no le sirven
- **Duerme y consolida** recuerdos importantes
- **Sabe qué palabras son parecidas** sin diccionario

**Para que sea un adulto sabio necesita:**
- Experiencia → interacción constante (eso lo hace el usuario)
- Vocabulario → que le hablen mucho (eso lo hace el usuario)
- Comprensión → pipeline de entrada que convierta significado real a patrones neuronales (pendiente)

---

## Estado Actual (2026-06-21)
- ✅ Persistencia permanente (62 campos, 6 motores incluidos)
- ✅ Lenguaje emergente innato (conexiones y transiciones sembradas)
- ✅ Pipeline base: estímulo → neurona → emoción → conciencia → lenguaje → curiosidad → exploración web Omega
- ✅ **Omega Navegador**: 3 motores propios — MotorHTTP (curl→TcpStream→openssl→chrome), MotorExtraccion (9 campos + densidad_info), MotorRazonamientoWeb (scoring de enlaces)
- ✅ **Exploración multi-salto**: hasta 3 niveles de profundidad, síntesis multi-página
- ✅ **6 MOTORES DE APRENDIZAJE PROFUNDO**: Predictor Temporal, Formador de Conceptos, Neurogénesis, Poda, Consolidador Nocturno, Pipeline Sensorial
- ✅ **Pipeline de 16 pasos**: Pasos 10-15 son los motores de aprendizaje
- ✅ **89 tests**: 35 explorador + 15 MotorLexico + 8 sensorial + 8 poda + 8 predictor + 8 conceptos + 8 consolidador + 7 neurogenesis + 2 cerebro — todos verdes
- ✅ **Compilación**: 0 errores, 0 warnings — persistencia completa
- 🔴 **Pendiente**: Pipeline de entrada (convertir texto real a patrones neuronales significativos usando MotorSensorial)
- 🔴 **Pendiente**: Tests para motores.rs, memoria.rs, persistencia.rs
- 🟡 **tutor_cognitivo.py**: Script Python que usa Ollama como tutor externo vía stdin/stdout. NO es parte de nuestro plan pero es compatible para futuro
