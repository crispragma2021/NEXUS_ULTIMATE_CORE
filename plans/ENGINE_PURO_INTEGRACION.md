# 🔌 INTEGRACIÓN: ENGINE PURO → NEXUS SHELL

> **Propósito:** Aclarar cómo el Engine Puro que ya construiste se integra como backend de inferencia local en el cascarón de NEXUS.

---

## 1. DIAGRAMA DE RELACIONES

```
┌────────────────────────────────────────────────────────────┐
│                   NEXUS SHELL (El cuerpo)                  │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           NEXUS CEREBRO (Orquestador)                │  │
│  │  • 46 órganos • pipeline.rs • memoria • emociones    │  │
│  │                                                      │  │
│  │  ¿Necesito generar texto? ¿Responder al usuario?     │  │
│  │                                                      │  │
│  │  ┌─────────── DECISIÓN ───────────┐                  │  │
│  │  │                                │                  │  │
│  │  │  ¿Hay internet?                │                  │  │
│  │  │  ├── Sí  → Gemini/DeepSeek/API │                  │  │
│  │  │  └── No  → ¿Hay engine local?  │                  │  │
│  │  │           ├── Sí  → ENGINE PURO│                  │  │
│  │  │           └── No  → Fallback   │                  │  │
│  │  └────────────────────────────────┘                  │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │     BACKENDS DE INFERENCIA DISPONIBLES               │  │
│  │                                                      │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐           │  │
│  │  │ Gemini   │  │DeepSeek  │  │ ENGINE   │           │  │
│  │  │ Nativo   │  │ API      │  │ PURO     │           │  │
│  │  │ (online) │  │ (online) │  │ (offline)│           │  │
│  │  └──────────┘  └──────────┘  └──────────┘           │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────┘
```

---

## 2. ¿ES EL ENGINE PURO = MODO OFFLINE?

**No exactamente, pero es una pieza clave.**

| Concepto | Definición |
|----------|-----------|
| **Engine Puro** | Tu proyecto en `engine-puro/` — un motor de inferencia con cerebro, aprendizaje y memoria propios. Es un **sistema de IA completo** que corre localmente. |
| **Modo Offline** | Un **estado operativo** del NEXUX Shell donde TODO funciona sin internet. El Engine Puro puede ser el backend que genera texto cuando no hay APIs. |

### Sin el Engine Puro, modo offline significa:
- NEXUS recuerda todo (memoria local)
- NEXUS razona con su pipeline interno (clasificación, emoción, juicio)
- NEXUS ejecuta herramientas locales
- **Pero no puede generar texto nuevo** sin un modelo que lo haga
- Solo responde con lo que ya sabe o con templates

### Con el Engine Puro, modo offline significa:
- TODO lo anterior
- **Más generación de texto nuevo** con el cerebro del Engine Puro
- NEXUS completo funcionando sin internet
- Independencia total de APIs externas

---

## 3. CÓMO SE INTEGRARÍA

### Hoy: Engine Puro es independiente

```
engine-puro/src/bin/cerebro.rs
  → Su propio cerebro
  → Su propia memoria
  → Su propio tutor
  → CLI independiente
```

### Mañana: Engine Puro como backend de NEXUS

```
NEXUS CEREBRO (Orquestador)
  ↓ necesita generar texto sin internet
  ↓ llama a...
Engine Puro (como librería, no como binario)
  ↓ devuelve inferencia
  ↓ NEXUS procesa, almacena en memoria, responde
```

### La integración técnica

```rust
// Así se vería en el Orquestador:

enum BackendInferencia {
    GeminiNativo,   // Online
    DeepSeekAPI,    // Online  
    EnginePuro,     // Offline — TU cerebro
    Fallback,       // Sin respuesta
}

impl Orquestador {
    async fn generar_texto(&self, prompt: &str) -> String {
        let backend = self.seleccionar_backend(); 
        // 1. ¿Hay internet? → Gemini
        // 2. ¿No hay internet pero está Engine Puro? → EnginePuro
        // 3. ¿Nada? → Fallback
        
        match backend {
            BackendInferencia::EnginePuro => {
                // Llamar al engine-puro como librería
                engine_puro::inferir(prompt).await
            }
            BackendInferencia::GeminiNativo => {
                self.gemini_cli(prompt).await
            }
            // ...
        }
    }
}
```

---

## 4. ESTADO ACTUAL DEL ENGINE PURO

```
engine-puro/
├── src/
│   ├── cerebro/
│   │   ├── cerebro.rs      → Cerebro con predicción y aprendizaje
│   │   ├── efectores.rs    → Ejecuta acciones
│   │   ├── memoria.rs      → Memoria de interacciones
│   │   └── ...
│   └── bin/
│       └── cerebro.rs      → CLI del engine
```

Para integrarlo como backend, necesitaríamos:

1. **Convertir `engine-puro/` en una librería** (lib.rs) además de binario
2. **Exponer una función** tipo `pub async fn inferir(prompt: &str) -> String`
3. **Registrarlo** en el Orquestador como un backend más

Tiempo estimado: **1 sesión de código.**

---

## 5. LA RESPUESTA CORTA A TU PREGUNTA

> **"modo offline es el engine puro?"**

**Casi.** Modo offline es el **estado** donde NEXUS no necesita internet. El Engine Puro es el **motor** que genera inteligencia cuando no hay APIs externas.

Uno es el **estado** (offline).  
El otro es el **backend** (Engine Puro).  
El cascarón es el **cuerpo** que los contiene a todos.
