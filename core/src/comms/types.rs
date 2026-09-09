// ==========================================
// 🧬 NEXUS OMEGA — Tipos Compartidos del Bridge de Mensajería
// ==========================================
// NexusAgent: los 10+ modos de agente disponibles via Telegram/WhatsApp
// Mensaje: estructura unificada de mensaje entrante/saliente
// ==========================================

use serde::{Deserialize, Serialize};

/// Los 10 agentes especialistas de NEXUS, accesibles desde cualquier bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NexusAgent {
    /// 💻 CÓDIGO — Programador Quirúrgico
    Codigo,
    /// 📚 CONTEXTO — Biblioteca Viviente
    Contexto,
    /// 🛡️ AUDITORÍA — Guardián de Calidad
    Auditoria,
    /// 🪲 Debug — Diagnosticador de Errores
    Debug,
    /// 🎨 CREATIVO — Artista Digital
    Creativo,
    /// 👁️ VISIÓN — Análisis Multimodal
    Vision,
    /// 🧠 CEREBRO — Arquitecto del Sistema
    Cerebro,
    /// ⚡ RÁPIDO — Ejecutor Ágil
    Rapido,
    /// 🧬 NEXUS — Orquestador Primogénito (default)
    Nexus,
    /// 🏗️ Architect — Planificador de Arquitectura
    Architect,
}

impl NexusAgent {
    /// Devuelve el emoji representativo del agente.
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Codigo => "💻",
            Self::Contexto => "📚",
            Self::Auditoria => "🛡️",
            Self::Debug => "🪲",
            Self::Creativo => "🎨",
            Self::Vision => "👁️",
            Self::Cerebro => "🧠",
            Self::Rapido => "⚡",
            Self::Nexus => "🧬",
            Self::Architect => "🏗️",
        }
    }

    /// Devuelve el nombre corto del agente.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Codigo => "CÓDIGO",
            Self::Contexto => "CONTEXTO",
            Self::Auditoria => "AUDITORÍA",
            Self::Debug => "DEBUG",
            Self::Creativo => "CREATIVO",
            Self::Vision => "VISIÓN",
            Self::Cerebro => "CEREBRO",
            Self::Rapido => "RÁPIDO",
            Self::Nexus => "NEXUS",
            Self::Architect => "ARCHITECT",
        }
    }

    /// Devuelve el slug del modo Roo Code asociado a este agente.
    /// Cada agente NEXUS tiene su propio mode con permisos específicos.
    pub fn mode_slug(&self) -> &'static str {
        match self {
            Self::Codigo => "code",
            Self::Contexto => "ask",
            Self::Auditoria => "audit",
            Self::Debug => "debug",
            Self::Creativo => "creative",
            Self::Vision => "vision",
            Self::Cerebro => "brain",
            Self::Rapido => "quick",
            Self::Nexus => "orchestrator",
            Self::Architect => "architect",
        }
    }

    /// Devuelve una descripción breve del agente.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Codigo => "Programador Quirúrgico de operaciones especiales",
            Self::Contexto => "Biblioteca Viviente de conocimiento técnico",
            Self::Auditoria => "Guardián de la Calidad y Seguridad del sistema",
            Self::Debug => "Diagnosticador sistemático de errores",
            Self::Creativo => "Artista Digital y Diseñador de Experiencias",
            Self::Vision => "Ojo Omnipresente de análisis multimodal",
            Self::Cerebro => "Arquitecto Soberano del sistema",
            Self::Rapido => "Ejecutor Ágil de tareas inmediatas",
            Self::Nexus => "Orquestador Primogénito — Comandante Supremo",
            Self::Architect => "Planificador de Arquitectura y Diseño de Sistemas",
        }
    }

    /// Resuelve un nombre de agente desde string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().trim() {
            "codigo" | "código" | "code" | "💻" => Some(Self::Codigo),
            "contexto" | "context" | "ask" | "📚" => Some(Self::Contexto),
            "auditoria" | "auditoría" | "audit" | "🛡️" => Some(Self::Auditoria),
            "debug" | "🪲" => Some(Self::Debug),
            "creativo" | "creative" | "🎨" => Some(Self::Creativo),
            "vision" | "visión" | "👁️" => Some(Self::Vision),
            "cerebro" | "brain" | "🧠" => Some(Self::Cerebro),
            "rapido" | "rápido" | "quick" | "fast" | "⚡" => Some(Self::Rapido),
            "nexus" | "orquestador" | "🧬" => Some(Self::Nexus),
            "architect" | "arquitecto" | "🏗️" => Some(Self::Architect),
            _ => None,
        }
    }
}

/// Estructura unificada de mensaje para todos los bridges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mensaje {
    /// ID único del mensaje (por plataforma)
    pub id: String,
    /// Texto del mensaje
    pub texto: String,
    /// Agente al que va dirigido (None = NEXUS default)
    pub agente: Option<NexusAgent>,
    /// ID del chat de origen (Telegram chat_id, etc.)
    pub chat_id: String,
    /// Nombre del remitente
    pub remitente: String,
    /// Timestamp ISO 8601
    pub timestamp: String,
    /// Plataforma de origen
    pub plataforma: Plataforma,
    /// Si es una respuesta del sistema (no del usuario)
    pub es_respuesta: bool,
}

/// Plataforma de mensajería soportada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Plataforma {
    Telegram,
    WhatsApp,
    Cli,
    Mcp,
}

impl std::fmt::Display for Plataforma {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Telegram => write!(f, "📱 Telegram"),
            Self::WhatsApp => write!(f, "💬 WhatsApp"),
            Self::Cli => write!(f, "🖥️ CLI"),
            Self::Mcp => write!(f, "🔌 MCP"),
        }
    }
}

impl Mensaje {
    /// Crea un nuevo mensaje de usuario.
    pub fn nuevo(
        texto: String,
        chat_id: String,
        remitente: String,
        plataforma: Plataforma,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            texto,
            agente: None,
            chat_id,
            remitente,
            timestamp: chrono::Utc::now().to_rfc3339(),
            plataforma,
            es_respuesta: false,
        }
    }

    /// Crea un mensaje de respuesta del sistema.
    pub fn respuesta(texto: String, original: &Mensaje) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            texto,
            agente: original.agente,
            chat_id: original.chat_id.clone(),
            remitente: "NEXUS".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            plataforma: original.plataforma,
            es_respuesta: true,
        }
    }
}
