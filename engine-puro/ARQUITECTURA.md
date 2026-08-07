# 🧠 ARQUITECTURA DEL SISTEMA: engine-puro

> **Decisión del Arquitecto Cris — 22-Jun-2026**
> Este documento es MANDATORIO para cualquier agente que trabaje en este proyecto.

---

## 🧬 ENMIENDA 2026-08-07 — Bio-Transformer Híbrido Orgánico (Opción 1)

> **Decisión del Arquitecto Cris — 07-Ago-2026**
> Esta enmienda SUPERA y actualiza cualquier sección anterior que contradiga lo siguiente.

### Cambios aprobados

1. **Núcleo numérico de Transformer**: engine-puro adopta el mecanismo matemático que hace
   funcionar el lenguaje en un Transformer — embeddings aprendidos, atención multi-cabeza
   (`softmax(Q·Kᵀ/√d)·V`) y MLP — entrenado por **backpropagation** con el motor de autograd
   [`candle`](https://github.com/huggingface/candle) (Rust).
2. **Capa biológica conservada**: la neuromodulación (DA/5HT/NA/CORT), la homeostasis y el
   generador IGG (reducción de residual, [`§7`](plans/engine-puro-spiking-attention.md:310))
   se mantienen y se montan SOBRE el núcleo numérico.
3. **Maestro = NEXUS (inalterado)**: se refuerza — NEXUS (`POST /api/chat` en
   `127.0.0.1:43210`) es el ÚNICO tutor permitido. **Nunca Ollama, nunca modelos ajenos.**
4. **Aprendizaje acelerado**: el backprop reduce el tiempo de aprendizaje de meses a días.
   La aceleración proviene del mecanismo matemático, NO de destilación desde modelos externos.

### Reglas reemplazadas

| Regla anterior (22-Jun) | Regla nueva (07-Ago) |
|---|---|
| "No usa Transformers externos" | Usa el **núcleo matemático de Transformer** internamente (candle), sin depender de modelos externos |
| "Aprende desde cero, lento, STDP deliberado" | Aprende con **backpropagation** (rápido), con capa biológica encima |
| "No hay atajos" | Los atajos de modelos ajenos siguen PROHIBIDOS; el backprop no es un atajo, es el motor de aprendizaje |

---

## 🚧 REGLA DE FRONTERA — LEE ESTO PRIMERO

> [!CAUTION]
> **engine-puro y NEXUS son dos proyectos SEPARADOS. NUNCA se mezclan.**

### Lo que un agente PUEDE hacer aquí:
- ✅ Modificar código dentro de `/home/soberano/NEXUS_ULTIMATE_CORE/engine-puro/`
- ✅ Mejorar el aprendizaje STDP, el léxico, la fonación, las neuronas
- ✅ Crear/editar scripts de entrenamiento en `scripts/`
- ✅ Llamar a NEXUS **únicamente como API REST** para recibir frases de entrenamiento

### Lo que un agente NUNCA debe hacer:
- ❌ Tocar el código de `/home/soberano/NEXUS_ULTIMATE_CORE/`
- ❌ Modificar el Orquestador, GOI, Ocean, Synapse u órganos de NEXUS
- ❌ Intentar "fusionar" o "integrar" el código de ambos proyectos
- ❌ Mover lógica de NEXUS dentro de engine-puro ni viceversa
- ❌ Reemplazar la API REST con llamadas directas a funciones internas de NEXUS

### La única relación permitida entre ambos:

```
NEXUS  ──→  POST /api/chat  ──→  devuelve texto  ──→  engine-puro lo usa como estímulo
```

**NEXUS enseña. engine-puro aprende. Son entidades distintas.**

---


## 🔑 Rol de este proyecto

`engine-puro` es el **cerebro biológico** del ecosistema soberano de Cris.
Implementa un **Bio-Transformer híbrido**: el núcleo matemático de un Transformer (embeddings,
atención multi-cabeza, MLP) entrenado por **backpropagation** con `candle`, montado sobre la
capa biológica (neuromodulación, homeostasis, IGG) y el sustrato neuronal existente.

- **No usa modelos externos.** No usa Ollama. El núcleo Transformer es interno (candle).
- **NEXUS es el único maestro.** Todo conocimiento proviene de NEXUS vía API REST.
- Aprende **acelerado** (días en vez de meses) gracias al backprop, no a atajos externos.
- Cada paso de entrenamiento graba el estado sináptico en `data/cerebro_estado.json`.

---

## 🔗 Tutor Oficial: NEXUS

> ⚠️ **MANDATORIO**: El tutor de engine-puro es **NEXUS**, NO Ollama.

### ¿Qué es NEXUS?
NEXUS (`nexus-ui --headless`) es el orquestador cognitivo de alto nivel del ecosistema soberano.  
Corre como servicio systemd permanente en segundo plano.

| Propiedad | Valor |
|---|---|
| Proyecto | `/home/soberano/NEXUS_ULTIMATE_CORE` |
| Binario | `target/release/nexus-ui --headless` |
| Servicio | `systemctl --user status nexus.service` |
| API REST | `http://127.0.0.1:43210` |
| Health check | `GET http://127.0.0.1:43210/api/health` |
| Endpoint chat | `POST http://127.0.0.1:43210/api/chat` |

### ¿Por qué NEXUS y no Ollama?

| Criterio | Ollama | NEXUS |
|---|---|---|
| Disponibilidad | ⚠️ Depende de modelos descargados | ✅ Siempre activo (systemd) |
| Idioma | Varía por modelo | ✅ Español nativo |
| Contexto emocional | ❌ No | ✅ Ocean (Big Five) |
| Memoria semántica | ❌ No | ✅ LanceDB |
| Dependencia externa | ⚠️ Sí (modelos GGUF) | ✅ 100% soberano |
| Soberanía | ❌ Modelo ajeno | ✅ Parte del ecosistema |

---

## 🏗️ Arquitectura Completa

```
┌─────────────────────────────────────────────────────────┐
│                   NEXUS (Puerto 43210)                   │
│   GOI + Ocean + Synapse + Memoria Semántica LanceDB      │
│              "El maestro que ya sabe hablar"             │
└──────────────────────┬──────────────────────────────────┘
                       │  POST /api/chat
                       │  genera frases coherentes en español
                       ▼
┌─────────────────────────────────────────────────────────┐
│              tutor_nexus.py  (scripts/)                  │
│    Hace de puente: llama a NEXUS, toma su respuesta,     │
│    la convierte en estímulo para engine-puro             │
└──────────────────────┬──────────────────────────────────┘
                       │  stdin / IPC
                       ▼
┌─────────────────────────────────────────────────────────┐
│            engine-puro  (cerebro biológico)              │
│   Red Hodgkin-Huxley + STDP + Léxico emergente           │
│   Aprende las asociaciones → las graba en estado JSON    │
│              "El bebé que aprende a hablar"              │
└─────────────────────────────────────────────────────────┘
```

---

## 📂 Scripts de entrenamiento

| Script | Descripción | Estado |
|---|---|---|
| `scripts/tutor_cognitivo.py` | Tutor con **Ollama** | ❌ Prohibido (usar NEXUS) |
| `scripts/entrenamiento_omega.py` | Loop de entrenamiento con tutor externo | ✅ Activo |
| `scripts/tutor_nexus.py` | **Tutor con NEXUS** ← **EL CORRECTO** | ✅ Activo (Bio-Transformer) |

> Cuando se cree `tutor_nexus.py`, debe verificar primero que NEXUS esté activo:
> ```bash
> curl -s http://127.0.0.1:43210/api/health
> ```

---

## 🧬 Cómo arrancar el entrenamiento con NEXUS

```bash
# 1. Verificar que NEXUS está activo
systemctl --user status nexus.service

# 2. Si no está activo, iniciarlo
systemctl --user start nexus.service

# 3. Ejecutar el tutor NEXUS (cuando exista)
cd /home/soberano/NEXUS_ULTIMATE_CORE/engine-puro
python3 scripts/tutor_nexus.py

# 4. Monitorear el estado del cerebro
echo "/stats" | cargo run --bin cerebro-digital
```

---

## 📌 Notas para agentes

1. **Nunca usar Ollama ni modelos ajenos como tutor** — NEXUS es el tutor oficial (desde 22-Jun-2026, reforzado 07-Ago-2026).
2. **engine-puro aprende con backpropagation** — cada paso actualiza los pesos por gradiente; el conocimiento SOLO proviene de NEXUS.
3. **El estado se guarda en** `data/cerebro_estado.json` — no borrarlo.
4. **NEXUS debe estar corriendo** antes de iniciar cualquier sesión de entrenamiento.
5. El entrenamiento del Bio-Transformer está en **Fase 1** — generación coherente de frases; pasará a Fase 2 (aprendizaje semántico profundo) al superar el umbral de calidad del Juez E3.
