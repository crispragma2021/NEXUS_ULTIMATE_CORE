# NEXUS como Tutor de engine-puro

> **Decisión del Arquitecto Cris — 22-Jun-2026**

---

## 🚧 REGLA DE FRONTERA — MANDATORIA

> [!CAUTION]
> **NEXUS y engine-puro son dos proyectos SEPARADOS. NUNCA se mezclan.**

### El rol de NEXUS en esta relación:
- ✅ NEXUS **enseña** — genera frases coherentes en español via su API REST
- ✅ NEXUS puede mejorar su propia capacidad de respuesta (GOI, Ocean, Synapse)
- ✅ NEXUS recibe preguntas de engine-puro y responde con texto natural

### Lo que NEXUS NUNCA debe hacer:
- ❌ Importar o depender del código de `engine-puro`
- ❌ Modificar el estado de las neuronas STDP de engine-puro directamente
- ❌ Fusionar su código con el de engine-puro
- ❌ Asumir el rol de motor neuronal biológico (ese es trabajo de engine-puro)

### La única relación permitida:
```
NEXUS  ──→  responde por /api/chat  ──→  engine-puro procesa el texto y aprende
```

**NEXUS enseña. engine-puro aprende. Son entidades distintas con vidas separadas.**

---

**NEXUS** (`nexus-ui --headless`) es el **tutor oficial** del cerebro biológico `engine-puro`.

Esta decisión reemplaza permanentemente a Ollama como fuente de estímulos de entrenamiento.

## Resumen

| Componente | Proyecto | Puerto | Rol |
|---|---|---|---|
| NEXUS | `/home/soberano/NEXUS_ULTIMATE_CORE` | 43210 | Tutor — genera frases coherentes en español |
| engine-puro | `/home/soberano/NEXUS_ULTIMATE_CORE/engine-puro` | — | Alumno — aprende asociaciones neuronales STDP |

## Racional

- **engine-puro** implementa aprendizaje neuronal biológico real (Hodgkin-Huxley + STDP).  
  Necesita estímulos externos coherentes para construir su léxico y grafo sináptico.
- **Ollama** era el tutor anterior pero es inestable, requiere modelos externos descargados y no tiene contexto emocional.
- **NEXUS** tiene: GOI (Generador Orgánico Interno), Ocean emocional (Big Five), Synapse con 15 conceptos base, memoria semántica LanceDB — todo soberano, sin dependencias externas.

## Flujo

```
NEXUS → genera frases coherentes → tutor_nexus.py → estimula engine-puro → STDP graba asociaciones
```

## API de NEXUS para tutoreo

```
GET  http://127.0.0.1:43210/api/health   → {"status": "ok"}
POST http://127.0.0.1:43210/api/chat     → {"message": "hola"} → respuesta del GOI
```

## Estado de implementación

- [x] NEXUS reparado y corriendo estable como servicio systemd (22-Jun-2026)
- [x] Decisión documentada en BITÁCORA y ARQUITECTURA.md
- [ ] Crear `engine-puro/scripts/tutor_nexus.py` — reemplaza tutor_cognitivo.py
- [ ] Conectar el loop de entrenamiento_omega.py a NEXUS en vez de Ollama

## Archivos de referencia

- `/home/soberano/NEXUS_ULTIMATE_CORE/engine-puro/ARQUITECTURA.md` — documento completo
- `/home/soberano/NEXUS_ULTIMATE_CORE/BITACORA.md` — hito 22-Jun-2026
- `/home/soberano/NEXUS_ULTIMATE_CORE/systemd/nexus.service` — servicio systemd
