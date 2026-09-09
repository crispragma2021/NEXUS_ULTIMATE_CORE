// ==========================================
// 🧬 NEXUS OMEGA — Router de Intención para Bridges de Mensajería
// ==========================================
// Detecta qué agente de NEXUS debe procesar un mensaje.
// Soporta: /comandos, @menciones, prefijo "agente:", y detección LLM.
// ==========================================

use super::types::{Mensaje, NexusAgent};
use std::collections::HashMap;

/// Router de intención: analiza mensajes entrantes y determina qué agente
/// de NEXUS debe procesarlos.
pub struct IntentRouter {
    /// Mapa de nombres de agente a NexusAgent (lowercase)
    agent_map: HashMap<&'static str, NexusAgent>,
    /// Palabras clave para detección automática por dominio
    domain_keywords: HashMap<&'static str, NexusAgent>,
}

impl Default for IntentRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl IntentRouter {
    /// Crea un nuevo IntentRouter con el mapeo completo de agentes.
    pub fn new() -> Self {
        let mut agent_map = HashMap::new();
        agent_map.insert("codigo", NexusAgent::Codigo);
        agent_map.insert("código", NexusAgent::Codigo);
        agent_map.insert("code", NexusAgent::Codigo);
        agent_map.insert("cod", NexusAgent::Codigo);
        agent_map.insert("contexto", NexusAgent::Contexto);
        agent_map.insert("ask", NexusAgent::Contexto);
        agent_map.insert("context", NexusAgent::Contexto);
        agent_map.insert("auditoria", NexusAgent::Auditoria);
        agent_map.insert("auditoría", NexusAgent::Auditoria);
        agent_map.insert("audit", NexusAgent::Auditoria);
        agent_map.insert("debug", NexusAgent::Debug);
        agent_map.insert("bugs", NexusAgent::Debug);
        agent_map.insert("creativo", NexusAgent::Creativo);
        agent_map.insert("creative", NexusAgent::Creativo);
        agent_map.insert("arte", NexusAgent::Creativo);
        agent_map.insert("diseño", NexusAgent::Creativo);
        agent_map.insert("vision", NexusAgent::Vision);
        agent_map.insert("visión", NexusAgent::Vision);
        agent_map.insert("ver", NexusAgent::Vision);
        agent_map.insert("imagen", NexusAgent::Vision);
        agent_map.insert("cerebro", NexusAgent::Cerebro);
        agent_map.insert("brain", NexusAgent::Cerebro);
        agent_map.insert("arquitecto", NexusAgent::Cerebro);
        agent_map.insert("rapido", NexusAgent::Rapido);
        agent_map.insert("rápido", NexusAgent::Rapido);
        agent_map.insert("quick", NexusAgent::Rapido);
        agent_map.insert("nexus", NexusAgent::Nexus);
        agent_map.insert("orquestador", NexusAgent::Nexus);
        agent_map.insert("architect", NexusAgent::Architect);

        // Palabras clave por dominio para detección automática
        let mut domain_keywords = HashMap::new();
        domain_keywords.insert("implementa", NexusAgent::Codigo);
        domain_keywords.insert("programa", NexusAgent::Codigo);
        domain_keywords.insert("código", NexusAgent::Codigo);
        domain_keywords.insert("refactoriza", NexusAgent::Codigo);
        domain_keywords.insert("compila", NexusAgent::Codigo);
        domain_keywords.insert("test", NexusAgent::Codigo);
        domain_keywords.insert("api", NexusAgent::Codigo);
        domain_keywords.insert("función", NexusAgent::Codigo);
        domain_keywords.insert("función", NexusAgent::Codigo);
        domain_keywords.insert("rust", NexusAgent::Codigo);
        domain_keywords.insert("typescript", NexusAgent::Codigo);
        domain_keywords.insert("escanear", NexusAgent::Auditoria);
        domain_keywords.insert("auditar", NexusAgent::Auditoria);
        domain_keywords.insert("seguridad", NexusAgent::Auditoria);
        domain_keywords.insert("vulnerabilidad", NexusAgent::Auditoria);
        domain_keywords.insert("debuggea", NexusAgent::Debug);
        domain_keywords.insert("error", NexusAgent::Debug);
        domain_keywords.insert("bug", NexusAgent::Debug);
        domain_keywords.insert("fallo", NexusAgent::Debug);
        domain_keywords.insert("pánico", NexusAgent::Debug);
        domain_keywords.insert("diseña", NexusAgent::Creativo);
        domain_keywords.insert("creativo", NexusAgent::Creativo);

        Self {
            agent_map,
            domain_keywords,
        }
    }

    /// Enruta un mensaje al agente apropiado.
    /// Devuelve (NexusAgent, texto_limpio_sin_routing).
    pub fn enrutar(&self, mensaje: &str) -> (NexusAgent, String) {
        let trimmed = mensaje.trim();

        // ── 1. Detectar comando tipo "/codigo haz X" ──────────
        if trimmed.starts_with('/') {
            if let Some(rest) = trimmed.strip_prefix('/') {
                let (cmd, texto) = Self::partir_comando(rest);
                if let Some(agente) = self.agent_map.get(cmd.to_lowercase().as_str()) {
                    return (*agente, texto.to_string());
                }
            }
        }

        // ── 2. Detectar mención "@agente haz X" ──────────────
        if trimmed.starts_with('@') {
            // Intentar extraer @nombre seguido de texto
            let sin_arroba = &trimmed[1..];
            let (nombre, texto) = Self::partir_comando(sin_arroba);
            if let Some(agente) = self.agent_map.get(nombre.to_lowercase().as_str()) {
                return (*agente, texto.to_string());
            }
            // Si empieza con @ pero no es un agente conocido, tratarlo como mención normal
            // (ej: @NexusBot) → NEXUS default
            if nombre.to_lowercase() == "nexusbot" || nombre.to_lowercase() == "nexus_bot" {
                return (NexusAgent::Nexus, texto.to_string());
            }
        }

        // ── 3. Detectar prefijo "agente: texto" ───────────────
        if let Some(dos_puntos) = trimmed.find(':') {
            let posible_agente = trimmed[..dos_puntos].trim().to_lowercase();
            let texto = trimmed[dos_puntos + 1..].trim();
            if !texto.is_empty() && self.agent_map.contains_key(posible_agente.as_str()) {
                if let Some(agente) = self.agent_map.get(posible_agente.as_str()) {
                    return (*agente, texto.to_string());
                }
            }
        }

        // ── 4. Detectar "agente texto" al inicio ──────────────
        let primera_palabra = trimmed.split_whitespace().next().unwrap_or("");
        if let Some(agente) = self.agent_map.get(primera_palabra.to_lowercase().as_str()) {
            let resto = trimmed
                .split_whitespace()
                .skip(1)
                .collect::<Vec<_>>()
                .join(" ");
            if !resto.is_empty() {
                return (*agente, resto);
            }
        }

        // ── 5. Detección por palabra clave del dominio ─────────
        let lower = trimmed.to_lowercase();
        for (keyword, agente) in &self.domain_keywords {
            if lower.contains(keyword) {
                return (*agente, trimmed.to_string());
            }
        }

        // ── 6. Default: NEXUS Orquestador ─────────────────────
        (NexusAgent::Nexus, trimmed.to_string())
    }

    /// Enruta un mensaje ya parseado desde Mensaje struct.
    /// Si el mensaje ya tiene un agente asignado, lo respeta.
    pub fn enrutar_mensaje(&self, msg: &Mensaje) -> (NexusAgent, String) {
        if let Some(agente) = msg.agente {
            return (agente, msg.texto.clone());
        }
        self.enrutar(&msg.texto)
    }

    /// Lista todos los agentes disponibles con sus emojis y descripciones.
    pub fn listar_agentes(&self) -> Vec<(NexusAgent, &'static str, &'static str)> {
        let mut agentes: Vec<_> = vec![
            NexusAgent::Codigo,
            NexusAgent::Contexto,
            NexusAgent::Auditoria,
            NexusAgent::Debug,
            NexusAgent::Creativo,
            NexusAgent::Vision,
            NexusAgent::Cerebro,
            NexusAgent::Rapido,
            NexusAgent::Architect,
            NexusAgent::Nexus,
        ];
        agentes.dedup();
        agentes
            .into_iter()
            .map(|a| (a, a.name(), a.description()))
            .collect()
    }

    /// Genera texto de ayuda formateado con todos los agentes.
    pub fn ayuda_comandos(&self) -> String {
        let mut ayuda = String::from("🧬 **NEXUS — Agentes Disponibles**\n\n");
        ayuda.push_str("Usa `/agente` o `@agente` o `agente: mensaje`\n\n");
        for (agente, nombre, desc) in self.listar_agentes() {
            ayuda.push_str(&format!("{} **{}** — {}\n", agente.emoji(), nombre, desc));
        }
        ayuda.push_str("\n_Ejemplos:_\n");
        ayuda.push_str("  `/código crea una API REST`\n");
        ayuda.push_str("  `@auditoría escanea seguridad`\n");
        ayuda.push_str("  `debug: el servidor falla con error 500`\n");
        ayuda
    }

    // ─── Utilitarios ──────────────────────────────────────────────

    /// Divide "comando texto" en (comando, texto_restante)
    fn partir_comando(input: &str) -> (&str, &str) {
        let input = input.trim();
        if let Some(space) = input.find(char::is_whitespace) {
            let cmd = &input[..space];
            let texto = input[space..].trim();
            (cmd, texto)
        } else {
            (input, "")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comando_slash() {
        let router = IntentRouter::new();
        let (agente, texto) = router.enrutar("/código crea una API");
        assert_eq!(agente, NexusAgent::Codigo);
        assert_eq!(texto, "crea una API");
    }

    #[test]
    fn test_mencion_arroba() {
        let router = IntentRouter::new();
        let (agente, texto) = router.enrutar("@auditoría escanea el sistema");
        assert_eq!(agente, NexusAgent::Auditoria);
        assert_eq!(texto, "escanea el sistema");
    }

    #[test]
    fn test_prefijo_dos_puntos() {
        let router = IntentRouter::new();
        let (agente, texto) = router.enrutar("debug: el servidor falla");
        assert_eq!(agente, NexusAgent::Debug);
        assert_eq!(texto, "el servidor falla");
    }

    #[test]
    fn test_default_nexus() {
        let router = IntentRouter::new();
        let (agente, texto) = router.enrutar("hola cómo estás?");
        assert_eq!(agente, NexusAgent::Nexus);
        assert_eq!(texto, "hola cómo estás?");
    }

    #[test]
    fn test_deteccion_keyword() {
        let router = IntentRouter::new();
        let (agente, _) = router.enrutar("necesito implementar un módulo de autenticación");
        assert_eq!(agente, NexusAgent::Codigo);
    }

    #[test]
    fn test_listar_agentes_no_vacio() {
        let router = IntentRouter::new();
        let agentes = router.listar_agentes();
        assert!(!agentes.is_empty());
        assert!(agentes.len() >= 10);
    }

    #[test]
    fn test_agente_espacio_inicio() {
        let router = IntentRouter::new();
        let (agente, texto) = router.enrutar("código implementa auth JWT");
        assert_eq!(agente, NexusAgent::Codigo);
        assert_eq!(texto, "implementa auth JWT");
    }

    #[test]
    fn test_vision_imagen() {
        let router = IntentRouter::new();
        let (agente, texto) = router.enrutar("visión: analiza esta imagen");
        assert_eq!(agente, NexusAgent::Vision);
        assert_eq!(texto, "analiza esta imagen");
    }
}
