# 🧬 NEXUS MESSENGER BRIDGE — Plan de Arquitectura

## Visión General

Exponer todos los agentes de NEXUS (20+ especialistas) a través de plataformas de mensajería, permitiendo al Arquitecto Director y usuarios autorizados interactuar con cualquier agente desde un grupo de WhatsApp/Telegram.

---

## 📊 Comparativa WhatsApp vs Telegram

| Criterio | 📱 Telegram (teloxide) | 💬 WhatsApp (Baileys) |
|---|---|---|
| Runtime | Rust puro | Node.js (TS) |
| Dependencias | Ya en Cargo.toml | Nueva: npm + bridge IPC |
| Protocolo | Oficial (Bot API v7) | No oficial (reversing) |
| Riesgo ban | Cero | Alto |
| Número teléfono | No necesita | Sí (sms_activate) |
| Setup | 1 min | QR + SMS |
| Grupos | Nativo | Nativo |
| Multi-sesión | Ilimitado | 1 dispositivo |
| Estabilidad | ✅ Años sólida | ⚠️ Cambia cada 6 meses |

---

## 🏛️ Arquitectura General

```
┌──────────────────────────────────────────────────────────────┐
│                      NEXUS CORE (Rust)                        │
│                                                              │
│  ┌─────────────────┐   ┌────────────────────────────────┐    │
│  │ Orquestador      │   │  MESSENGER BRIDGE              │    │
│  │ (pipeline.rs)    │   │                                │    │
│  │ ┌─────────────┐  │   │  ┌─────────────────────────┐  │    │
│  │ │ clasificar  │  │   │  │ TelegramBridge          │  │    │
│  │ │ _tarea()    │◄─┼──┼──│ (teloxide 0.12)          │  │    │
│  │ └─────────────┘  │   │  │ • receive_messages()     │  │    │
│  │                  │   │  │ • send_message()          │  │    │
│  │ ┌─────────────┐  │   │  │ • command_handler()       │  │    │
│  │ │ router de   │  │   │  │ • group_mention_handler() │  │    │
│  │ │ intención   │◄─┼──┼──┤ └─────────────────────────┘  │    │
│  │ │ (agentes)   │  │   │                                │    │
│  │ └─────────────┘  │   │  ┌─────────────────────────┐  │    │
│  │                  │   │  │ WhatsAppBridge           │  │    │
│  │ ┌─────────────┐  │   │  │ (Chromiumoxide CDP)     │  │    │
│  │ │ responder()  │──┼──┼──│ • sesión WA Web         │  │    │
│  │ └─────────────┘  │   │  │ • QR escaneo            │  │    │
│  └─────────────────┘   │  │ • group_listener()       │  │    │
│                        │  │ • message_sender()       │  │    │
│                        │  └─────────────────────────┘  │    │
│                        └────────────────────────────────┘    │
│                                                              │
│  ┌─────────────────┐   ┌────────────────────────────────┐    │
│  │ Sembrador        │   │ VisionFantasma                  │    │
│  │ (sms_activate)   │   │ (stealth para WA Web)          │    │
│  └─────────────────┘   └────────────────────────────────┘    │
│                                                              │
│  ┌─────────────────┐   ┌────────────────────────────────┐    │
│  │ Engine-puro      │   │ Ollama (LLM local)             │    │
│  │ (cerebro nativo) │   │ (API HTTP)                     │    │
│  └─────────────────┘   └────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────┘
```

---

## 📱 FASE 1 — TELEGRAM (Prioritaria)

### Archivos a crear/modificar

| Archivo | Acción | Propósito |
|---|---|---|
| `core/src/comms/telegram_bridge.rs` | **CREAR** | Bot bidireccional completo |
| `core/src/comms/intent_router.rs` | **CREAR** | Router de intención unificado |
| `core/src/comms/mod.rs` | **CREAR** | Módulo comms |
| `core/src/nexus_telegram.rs` | **MODIFICAR** | Migrar alertas unidireccionales |
| `core/src/cerebro/pipeline.rs` | **EXTENDER** | Interfaz para entrada/salida externa |

### Flujo de un mensaje en Telegram

```
Usuario escribe en grupo:
  "@NexusBot código implementa una API REST"

1. TelegramBridge recibe Update
2. parse_command() extrae: agente="código", mensaje="implementa una API REST"
3. IntentRouter.enrutar(agente, mensaje) -> selecciona agente 💻
4. Orquestador.responder(mensaje) -> genera respuesta
5. TelegramBridge.send_message() -> envía al grupo
```

### Capacidades del bot Telegram

- Comandos: `/código`, `/auditoría`, `/contexto`, `/debug`, `/creativo`, `/visión`
- Menciones: `@NexusBot haz X`
- Conversación libre en chat privado (NEXUS responde directamente)
- Grupos: detecta menciones y comandos
- Archivos: recibe imágenes (👁️ Visión), código, documentos
- Memoria: mantiene contexto por chat usando IDs de Telegram

### Dependencias nuevas

**Ninguna.** `teloxide = 0.12` ya está en `core/Cargo.toml:40`.

### Configuración necesaria

```env
TELEGRAM_TOKEN=123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11
TELEGRAM_GROUP_ID=-1001234567890
TELEGRAM_ADMIN_ID=123456789
```

---

## 💬 FASE 2 — WHATSAPP (Futura)

### Estrategia: ChromePlanter + WA Web

Reutilizando el módulo existente:

1. `ChromePlanter::lanzar_generico()` abre `web.whatsapp.com`
2. `VisionFantasma::aplicar_camuflaje_omega()` inyecta stealth
3. `ChromePlanter::type_human_like()` escribe mensajes
4. Scrapea el DOM de WA Web para leer mensajes entrantes
5. Responde usando los mismos métodos de typing humano

### Limitaciones de WA Web

- No puede iniciar conversaciones (solo responder)
- La sesión expira si no se mantiene activa
- Escanear QR requiere interacción humana inicial
- Meta puede detectar y banear el número
- Más frágil que Telegram a cambios de UI

---

## 🧠 IntentRouter — Corazón del Bridge

### Arquitectura del Router

```rust
// core/src/comms/intent_router.rs

pub enum Agente {
    Codigo,       // 💻
    Contexto,     // 📚
    Auditoria,    // 🛡️
    Debug,        // 🪲
    Creativo,     // 🎨
    Vision,       // 👁️
    Cerebro,      // 🧠
    Rapido,       // ⚡
    Orquestador,  // 🧬 (default)
}

pub struct IntentRouter {
    // Mapea nombre -> Agente
    agent_map: HashMap<String, Agente>,
}

impl IntentRouter {
    pub fn enrutar(&self, input: &str) -> (Agente, String) {
        // Detecta patrones: "@codigo X", "/codigo X", "código: X"
    }

    pub fn procesar(&self, agente: &Agente, mensaje: &str) -> String {
        // Llama al agente correspondiente vía Orquestador
    }
}
```

### Patrones de detección de agente

| Formato | Ejemplo | Agente |
|---|---|---|
| `@código X` | `@código crea una API` | 💻 Código |
| `/código X` | `/código refactoriza main.rs` | 💻 Código |
| `código: X` | `código: implementa auth JWT` | 💻 Código |
| `@auditoría X` | `@auditoría escanea seguridad` | 🛡️ Auditoría |
| (default) | `¿qué piensas de X?` | 🧬 Orquestador |

---

## 📋 Plan de Implementación

### Paso 1: Módulo `core/src/comms/`

```
core/src/comms/
├── mod.rs              # Re-export
├── telegram_bridge.rs  # Bot bidireccional
├── intent_router.rs    # Router de agentes
└──types.rs             # Tipos compartidos
```

### Paso 2: TelegramBridge

```rust
pub struct TelegramBridge {
    bot: Bot,
    router: Arc<IntentRouter>,
    orchestrator: Arc<Orquestador>,
    chat_contexts: HashMap<i64, Vec<Mensaje>>, // Memoria por chat
}

impl TelegramBridge {
    pub async fn new(token: &str, router: Arc<IntentRouter>, orch: Arc<Orquestador>) -> Self;
    pub async fn start(&self); // Inicia long-polling
    pub async fn send_message(&self, chat_id: i64, text: &str);
    pub async fn broadcast(&self, text: &str); // Envía a grupo de admin
}
```

### Paso 3: Migrar alertas existentes

El actual [`core/src/nexus_telegram.rs`](core/src/nexus_telegram.rs) solo hace `send_alert()` unidireccional. Se migra a usar el nuevo bridge, manteniendo compatibilidad:

```rust
// nexus_telegram.rs actualizado
pub async fn send_alert(message: &str) {
    // Delegar a TelegramBridge::broadcast()
}
```

### Paso 4: Integración con Orquestador

```rust
// En pipeline.rs o un adapter
impl Orquestador {
    pub async fn responder_messenger(&self, mensaje: &str, agente: Option<Agente>) -> String {
        match agente {
            Some(Agente::Codigo) => self.responder_con_contexto("💻", mensaje),
            Some(Agente::Auditoria) => self.responder_con_contexto("🛡️", mensaje),
            None => self.responder(mensaje).await,
        }
    }
}
```

---

## 🔄 Diagrama de Flujo Completo

```mermaid
flowchart TD
    User["👤 Usuario en Grupo"] -->|@NexusBot comando| TG["Telegram API"]
    TG -->|Update| TB["TelegramBridge\nreceive_messages"]
    TB -->|parse| IR["IntentRouter\nenrutar"]
    IR -->|agente + msg| ORQ["Orquestador\nresponder"]
    
    ORQ -->|Si es código| AG_CODE["💻 Agente Código"]
    ORQ -->|Si es auditoría| AG_AUDIT["🛡️ Agente Auditoría"]
    ORQ -->|Si es debug| AG_DEBUG["🪲 Agente Debug"]
    ORQ -->|Si es creativo| AG_CREATIVE["🎨 Agente Creativo"]
    ORQ -->|Si es visión| AG_VISION["👁️ Agente Visión"]
    ORQ -->|Default| AG_NEXUS["🧬 NEXUS Orquestador"]
    
    AG_CODE --> RESP["Respuesta generada"]
    AG_AUDIT --> RESP
    AG_DEBUG --> RESP
    AG_CREATIVE --> RESP
    AG_VISION --> RESP
    AG_NEXUS --> RESP
    
    RESP --> TB
    TB -->|send_message| TG
    TG --> User
```

---

## 🎯 Criterios de Aceptación FASE 1

- [ ] Bot Telegram responde a comandos `/código`, `/auditoría`, `/contexto`, `/debug`
- [ ] Bot Telegram detecta menciones `@NexusBot` en grupos
- [ ] Bot mantiene contexto por chat (mínimo últimas 20 interacciones)
- [ ] Alertas críticas existentes siguen funcionando (send_alert)
- [ ] Se puede invocar `engine-puro` como LLM local desde el bot
- [ ] Se puede invocar Ollama desde el bot
- [ ] Compila con `cargo build` (0 errores, 0 warnings nuevos)
- [ ] Documentación de configuración en .env.example

---

## 🚀 Notas Técnicas Adicionales

### Telegram
- `teloxide` usa long-polling por defecto (sin webhooks, más simple)
- Soporta comandos vía `#[command]` macro
- Grupos: el bot necesita ser admin para leer todos los mensajes
- Chat privado: el bot ve todo automáticamente

### WhatsApp (para FASE 2)
- Usar `ChromePlanter::lanzar_generico()` con perfil persistente
- `VisionFantasma::aplicar_camuflaje_omega()` antes de navegar a WA Web
- Polling del DOM cada 2-5 segundos para nuevos mensajes
- Mantener sesión con keepalive: recargar WA Web cada 30 min
- Almacenar `session.data` (cookies/localStorage) en perfil de identidad
