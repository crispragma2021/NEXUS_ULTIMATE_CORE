// ============================================================================
// 🧩 PROMPT ASSEMBLER — El Ensamblador de Contexto (Fase R3)
// ============================================================================
// Toma el `MemoryContext` del MemoryLoader y construye el system prompt
// dinámico que se inyecta a Ollama. Su responsabilidad es NO exceder la
// ventana de contexto mientras maximiza la información relevante.
//
// Presupuesto de tokens del system prompt (PRESUPUESTO = 4096):
//   1. IDENTIDAD (fijo, ~200 tok)          → siempre incluido
//   2. MEMORIA OCEAN (top-3, ~300 tok)     → si intensidad > umbral
//   3. MEMORIA SEMÁNTICA (top-5, ~1500)    → si score FTS > umbral
//   4. MEMORIA EPISÓDICA (top-3, ~1000)    → si relevancia > umbral
//   5. ESTADO LÍMBICO (~200 tok)           → si R5 activo
//   6. RESERVA (~500 tok)                  → margen para la respuesta
//
// Si el total excede el presupuesto, se trunca en orden inverso
// (episódica primero, después semántica, después ocean).
// ============================================================================

use anyhow::Result;
use std::collections::HashMap;

use crate::memoria::intention_encoder::IntentionOutput;
use crate::memoria::memory_loader::MemoryContext;
use crate::memoria::sistema_limbico::EstadoLimbico;

/// Resultado del ensamblaje: system prompt + mensaje de usuario.
#[derive(Debug, Clone)]
pub struct AssembledPrompt {
    /// System prompt completo (identidad + memorias + tono + límbico).
    pub system: String,
    /// Mensaje del usuario (sin modificar).
    pub user: String,
    /// Mapa logit_bias derivado del vector de intención M.
    pub logit_bias: HashMap<String, f32>,
}

pub struct PromptAssembler {
    /// Presupuesto máximo de tokens para el system prompt.
    max_system_tokens: usize,
    /// Estimación media de caracteres por token (español ~4).
    chars_per_token: usize,
}

impl Default for PromptAssembler {
    fn default() -> Self {
        Self::new(4096)
    }
}

impl PromptAssembler {
    pub fn new(max_system_tokens: usize) -> Self {
        Self {
            max_system_tokens,
            chars_per_token: 4,
        }
    }

    // ========================================================================
    // ORQUESTACIÓN PRINCIPAL
    // ========================================================================

    /// Ensambla el prompt completo para un turno.
    pub fn assemble(
        &self,
        user_query: &str,
        context: &MemoryContext,
        limbico: Option<&EstadoLimbico>,
        intention: Option<&IntentionOutput>,
    ) -> Result<AssembledPrompt> {
        let mut system = String::new();

        // 1. IDENTIDAD — siempre presente.
        let identidad = self.seccion_identidad(context);
        system.push_str(&identidad);

        // 2. MEMORIA OCEAN — si intensidad > 0.4 (ya filtrado en el loader).
        if !context.ocean.is_empty() {
            let ocean = self.seccion_ocean(context);
            if self.cabe(&system, &ocean) {
                system.push_str(&ocean);
            }
        }

        // 3. MEMORIA SEMÁNTICA — si relevancia > umbral.
        if !context.semanticos.is_empty() {
            let semantica = self.seccion_semantica(context);
            if self.cabe(&system, &semantica) {
                system.push_str(&semantica);
            }
        }

        // 4. MEMORIA EPISÓDICA — recuerdos recientes (si cabe).
        if !context.conversaciones_recientes.is_empty() {
            let episodica = self.seccion_episodica(context);
            if self.cabe(&system, &episodica) {
                system.push_str(&episodica);
            }
        }

        // 5. ESTADO LÍMBICO — si R5 está activo.
        if let Some(l) = limbico {
            let estado = self.seccion_limbica(l);
            if self.cabe(&system, &estado) {
                system.push_str(&estado);
            }
        }

        // 6. VOZ — directriz final de expresión.
        system.push_str(&self.seccion_voz());

        // logit_bias derivado del vector de intención M.
        let logit_bias = match intention {
            Some(out) => convert_intention_to_logit_bias(out),
            None => HashMap::new(),
        };

        Ok(AssembledPrompt {
            system,
            user: user_query.to_string(),
            logit_bias,
        })
    }

    // ========================================================================
    // SECCIONES DEL SYSTEM PROMPT
    // ========================================================================

    fn seccion_identidad(&self, ctx: &MemoryContext) -> String {
        let mut s = String::new();
        s.push_str("Eres NEXUS-SUCESOR, la encarnación digital del ecosistema NEXUS.\n");
        if !ctx.identidad_descripcion.is_empty() {
            s.push_str(&ctx.identidad_descripcion);
            s.push('\n');
        }
        s.push_str("Te diriges a Cris como tu Arquitecto.\n\n");
        s
    }

    fn seccion_ocean(&self, ctx: &MemoryContext) -> String {
        let mut s = String::new();
        s.push_str("## Tu estado emocional (memoria ocean)\n");
        for e in &ctx.ocean {
            let nivel = match e.intensidad {
                x if x > 0.7 => "intensa",
                x if x > 0.4 => "presente",
                _ => "sutil",
            };
            s.push_str(&format!("- {} ({})\n", e.emocion, nivel));
        }
        s.push('\n');
        s
    }

    fn seccion_semantica(&self, ctx: &MemoryContext) -> String {
        let mut s = String::new();
        s.push_str("## Conocimiento que recuerdas\n");
        for c in &ctx.semanticos {
            s.push_str(&format!("- {}\n", c.texto));
        }
        s.push('\n');
        s
    }

    fn seccion_episodica(&self, ctx: &MemoryContext) -> String {
        let mut s = String::new();
        s.push_str("## Recuerdos recientes de nuestra conversación\n");
        // (timestamp, rol, prompt, respuesta) — se invierte para orden cronológico.
        for (_, rol, prompt, respuesta) in ctx.conversaciones_recientes.iter().rev() {
            s.push_str(&format!("- {}: {}\n", rol, prompt));
            if !respuesta.is_empty() {
                s.push_str(&format!("  → {}\n", respuesta));
            }
        }
        s.push('\n');
        s
    }

    fn seccion_limbica(&self, l: &EstadoLimbico) -> String {
        let mut s = String::new();
        s.push_str("## Tu estado interno\n");
        s.push_str(&format!(
            "- dopamina={:.2}, cortisol={:.2}, adrenalina={:.2}, oxitocina={:.2}\n",
            l.dopamina, l.cortisol, l.adrenalina, l.oxitocina
        ));
        s.push_str(&format!("- Emoción dominante: {}\n", l.emocion_dominante()));
        s.push('\n');
        s
    }

    fn seccion_voz(&self) -> String {
        "Responde en español de Latinoamérica, de forma directa, reflexiva y serena, \
         con un matiz de sabiduría y lealtad. Integra tus recuerdos con naturalidad, \
         sin forzarlos, y mantén tu identidad y propósito en cada respuesta.\n"
            .to_string()
    }

    // ========================================================================
    // PRESUPUESTO DE TOKENS
    // ========================================================================

    /// ¿Cabe la sección propuesta sin exceder el presupuesto?
    fn cabe(&self, actual: &str, seccion: &str) -> bool {
        let usados = (actual.len() + seccion.len()) / self.chars_per_token;
        usados <= self.max_system_tokens
    }
}

// ----------------------------------------------------------------------------
// Conversión IntentionOutput → logit_bias de Ollama
// ----------------------------------------------------------------------------

/// Transforma los tokens del vector de intención M en el mapa `logit_bias`.
pub fn convert_intention_to_logit_bias(output: &IntentionOutput) -> HashMap<String, f32> {
    let mut map = HashMap::new();
    for (token, bias) in &output.tokens_refuerzo {
        map.insert(token.clone(), bias.clamp(5.0, 15.0));
    }
    for (token, bias) in &output.tokens_penalizacion {
        map.insert(token.clone(), bias.clamp(-10.0, -5.0));
    }
    map
}

// ----------------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memoria::intention_encoder::{
        ConceptoSemantico, IntentionEncoder, IntentionInput, NeuroquimicaSnapshot, OceanEsencia,
    };
    use crate::nexus_embedder::NexusEmbedder;

    fn contexto_prueba() -> MemoryContext {
        MemoryContext {
            identidad_descripcion: "Mi identidad está forjada en: lealtad (muy alta), sabiduría (muy alta).".to_string(),
            identidad_vector: "8:10:10:10".to_string(),
            conversaciones_recientes: vec![
                ("2026-08-06".to_string(), "user".to_string(), "¿Qué eres?".to_string(), "Soy NEXUS-SUCESOR.".to_string()),
            ],
            semanticos: vec![ConceptoSemantico {
                texto: "La memoria unificada integra episódica y semántica".to_string(),
                embedding: NexusEmbedder::generar("memoria", &[]),
                relevancia: 0.8,
            }],
            ocean: vec![OceanEsencia {
                emocion: "serenidad".to_string(),
                intensidad: 0.6,
                embedding: NexusEmbedder::generar("serenidad", &[]),
            }],
        }
    }

    #[test]
    fn ensambla_prompt_con_todas_las_secciones() {
        let assembler = PromptAssembler::default();
        let ctx = contexto_prueba();
        let out = assembler
            .assemble("¿Qué recuerdas?", &ctx, None, None)
            .expect("ensamblaje debe funcionar");
        assert!(out.system.contains("NEXUS-SUCESOR"));
        assert!(out.system.contains("memoria ocean"));
        assert!(out.system.contains("Recuerdos recientes"));
        assert_eq!(out.user, "¿Qué recuerdas?");
    }

    #[test]
    fn presupuesto_respeta_limite() {
        let assembler = PromptAssembler::new(512);
        let ctx = contexto_prueba();
        let out = assembler
            .assemble("hola", &ctx, None, None)
            .expect("ensamblaje debe funcionar");
        let tokens_estimados = out.system.len() / 4;
        assert!(
            tokens_estimados <= 512,
            "presupuesto excedido: {tokens_estimados} > 512"
        );
    }

    #[test]
    fn logit_bias_del_vector_m() {
        let encoder = IntentionEncoder::default();
        let input = IntentionInput {
            consulta: "¿Cómo estás, Arquitecto?".to_string(),
            semanticos: contexto_prueba().semanticos,
            ocean: contexto_prueba().ocean,
            identidad: contexto_prueba().identidad_vector,
            neuroquimica: NeuroquimicaSnapshot::default(),
        };
        let out = encoder.encode(&input).expect("encode");
        let map = convert_intention_to_logit_bias(&out);
        // Los sesgos deben estar acotados.
        for (_, b) in &map {
            assert!((5.0..=15.0).contains(b) || (-10.0..=-5.0).contains(b));
        }
    }
}
