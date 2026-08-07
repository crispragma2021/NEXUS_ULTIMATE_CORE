// ============================================================================
// 🧠 RAZONADOR QWEN — Motor local de razonamiento y generación (Ollama)
// ============================================================================
// Provee la "inteligencia" local al pipeline v0 sin depender de la nube:
//   - `razonar()`: modo pensamiento — Qwen razona y PLANIFICA antes de
//     escribir código. Es el "piensa antes de entregar" del Arquitecto.
//   - `generar()`: genera un módulo/archivo usando el contexto inyectado
//     (RAG de extracción web + catálogo shadcn).
//   - `razonar_local()`: fallback determinista (sin red) que produce un plan
//     estructurado y coherente para que el pipeline nunca se detenga.
//
// Estrategia:
//   - En producción llama a Ollama (`http://localhost:11434`) con el modelo
//     local (p.ej. `qwen2.5:7b`). El modo razonamiento solicita una respuesta
//     estructurada JSON y luego la regenera como código en `generar`.
//   - Si Ollama no responde (timeout, error, hermeticidad de tests), degrada
//     al razonador determinista sin paniquear.
// ============================================================================

use std::time::Duration;

/// Plan estructurado emitido por el razonador (nube o local).
#[derive(Debug, Clone, PartialEq)]
pub struct PlanRazonado {
    /// Descripción breve del juego/UI a construir.
    pub vision: String,
    /// Lista de módulos/archivos a generar, en orden.
    pub modulos: Vec<String>,
    /// Tecnología elegida (React, Canvas, Phaser, etc.).
    pub tecnologia: String,
    /// `true` si el plan provino del fallback determinista (sin red).
    pub es_local: bool,
}

/// Resultado de una operación de razonamiento/generación.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultadoRazonamiento {
    /// Plan razonado (en modo razonar) o texto generado (en modo generar).
    pub contenido: String,
    /// Plan estructurado derivado (presente en modo razonar).
    pub plan: PlanRazonado,
    /// `true` si se usó el motor local determinista (sin llamada a Ollama).
    pub uso_local: bool,
    /// Milisegundos de duración.
    pub duration_ms: u64,
}

/// Cliente de Ollama para razonamiento y generación local.
#[derive(Debug, Clone)]
pub struct RazonadorQwen {
    base_url: String,
    modelo: String,
    timeout: Duration,
    /// Si `true`, nunca intenta red (modo hermético para tests/CI).
    modo_local_forzado: bool,
}

impl Default for RazonadorQwen {
    fn default() -> Self {
        Self::nuevo()
    }
}

impl RazonadorQwen {
    /// Construye el razonador con la configuración por defecto:
    /// Ollama local (`localhost:11434`), modelo `qwen2.5:7b`, timeout 60s.
    pub fn nuevo() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            modelo: "qwen2.5:7b".to_string(),
            timeout: Duration::from_secs(60),
            modo_local_forzado: false,
        }
    }

    /// Fuerza el modo local determinista (sin red). Útil para tests y CI.
    pub fn con_local_forzado(mut self) -> Self {
        self.modo_local_forzado = true;
        self
    }

    /// Configura el modelo local a usar.
    pub fn con_modelo(mut self, modelo: &str) -> Self {
        self.modelo = modelo.to_string();
        self
    }

    /// Razonamiento determinista (sin red): produce un plan estructurado y
    /// coherente a partir del prompt. Nunca paniquiza.
    pub fn razonar_local(&self, prompt: &str) -> ResultadoRazonamiento {
        let inicio = std::time::Instant::now();
        let vision = if prompt.trim().is_empty() {
            "UI genérica generada por NEXUS".to_string()
        } else {
            prompt.trim().to_string()
        };

        // Inferir tecnología y módulos a partir de palabras clave del prompt.
        let lower = prompt.to_lowercase();
        let tecnologia = if lower.contains("juego") || lower.contains("game") || lower.contains("canvas") {
            "HTML5 Canvas + TypeScript"
        } else if lower.contains("dashboard") || lower.contains("panel") {
            "React + Tailwind + shadcn/ui"
        } else {
            "React + Tailwind + shadcn/ui"
        };

        let mut modulos = vec![
            "src/App.tsx".to_string(),
            "src/main.tsx".to_string(),
            "src/index.css".to_string(),
            "package.json".to_string(),
            "tailwind.config.ts".to_string(),
            "vite.config.ts".to_string(),
        ];
        if tecnologia.starts_with("HTML5 Canvas") {
            modulos.insert(1, "src/engine/game_loop.ts".to_string());
            modulos.insert(2, "src/engine/input.ts".to_string());
            modulos.insert(3, "src/entities/player.ts".to_string());
        }

        let plan = PlanRazonado {
            vision,
            modulos,
            tecnologia: tecnologia.to_string(),
            es_local: true,
        };

        // El "contenido" del modo razonar es el plan serializado legible.
        let contenido = format!(
            "PLAN RAZONADO (local)\nVisión: {}\nTecnología: {}\nMódulos:\n  - {}",
            plan.vision,
            plan.tecnologia,
            plan.modulos.join("\n  - ")
        );

        ResultadoRazonamiento {
            contenido,
            plan,
            uso_local: true,
            duration_ms: inicio.elapsed().as_millis() as u64,
        }
    }

    /// Modo razonamiento real: llama a Ollama para que Qwen razone y
    /// planifique. Si no hay red, degrada al razonador determinista.
    pub async fn razonar(&self, prompt: &str) -> ResultadoRazonamiento {
        if self.modo_local_forzado {
            return self.razonar_local(prompt);
        }

        let inicio = std::time::Instant::now();
        let sistema = "\
Eres el planificador de un motor de generación de UI/juegos. Analiza el prompt y \
responde EXCLUSIVAMENTE con un JSON plano de este esquema:\n\
{\"vision\":\"<frase breve>\",\"tecnologia\":\"<React+Canvas|React+Tailwind>\",\
\"modulos\":[\"src/App.tsx\",\"src/main.tsx\",\"src/index.css\",\"package.json\"]}\n\
No añadas texto fuera del JSON.";

        match self.llamar_ollama(sistema, prompt, 0).await {
            Ok(texto) => {
                let plan = self.extraer_plan_del_json(&texto);
                ResultadoRazonamiento {
                    contenido: texto,
                    plan,
                    uso_local: false,
                    duration_ms: inicio.elapsed().as_millis() as u64,
                }
            }
            Err(_) => self.razonar_local(prompt),
        }
    }

    /// Modo generación real: pide a Qwen que genere el código de un módulo
    /// usando el contexto (RAG) proporcionado. Degrada a local sin red.
    pub async fn generar(&self, prompt: &str, contexto: &str) -> ResultadoRazonamiento {
        if self.modo_local_forzado {
            return self.razonar_local(prompt);
        }

        let inicio = std::time::Instant::now();
        let sistema = format!(
            "\
Eres el generador de código de un motor de UI/juegos (React + TypeScript + Tailwind + shadcn). \
Genera SOLO código válido, sin texto explicativo, usando este contexto de referencia:\n\
---CONTEXTO---\n{}\n---FIN CONTEXTO---",
            contexto
        );

        match self.llamar_ollama(&sistema, prompt, 0).await {
            Ok(texto) => ResultadoRazonamiento {
                plan: PlanRazonado {
                    vision: prompt.to_string(),
                    modulos: vec![],
                    tecnologia: String::new(),
                    es_local: false,
                },
                contenido: texto,
                uso_local: false,
                duration_ms: inicio.elapsed().as_millis() as u64,
            },
            Err(_) => self.razonar_local(prompt),
        }
    }

    /// Ejecuta una llamada a la API de Ollama `/api/chat`.
    async fn llamar_ollama(&self, sistema: &str, prompt: &str, _max_tokens: u32) -> Result<String, String> {
        let cliente = reqwest::Client::new();
        let url = format!("{}/api/chat", self.base_url);

        let body = serde_json::json!({
            "model": self.modelo,
            "stream": false,
            "messages": [
                { "role": "system", "content": sistema },
                { "role": "user", "content": prompt }
            ],
            "options": { "temperature": 0.7 }
        });

        let respuesta = cliente
            .post(&url)
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| format!("error red ollama: {e}"))?
            .error_for_status()
            .map_err(|e| format!("error http ollama: {e}"))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| format!("error json ollama: {e}"))?;

        respuesta
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "respuesta ollama sin message.content".to_string())
    }

    /// Extrae un `PlanRazonado` desde la respuesta JSON de Qwen.
    /// Si el parseo falla, degrada a un plan local derivado del texto crudo.
    fn extraer_plan_del_json(&self, texto: &str) -> PlanRazonado {
        // Intentar extraer el objeto JSON embebido en la respuesta.
        let inicio_obj = texto.find('{');
        let fin_obj = texto.rfind('}');
        let json_str = match (inicio_obj, fin_obj) {
            (Some(i), Some(f)) if f > i => &texto[i..=f],
            _ => texto,
        };

        if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
            let vision = v
                .get("vision")
                .and_then(|x| x.as_str())
                .unwrap_or(texto)
                .to_string();
            let tecnologia = v
                .get("tecnologia")
                .and_then(|x| x.as_str())
                .unwrap_or("React + Tailwind")
                .to_string();
            let modulos = v
                .get("modulos")
                .and_then(|x| x.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| m.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| {
                    vec![
                        "src/App.tsx".to_string(),
                        "src/main.tsx".to_string(),
                        "src/index.css".to_string(),
                        "package.json".to_string(),
                    ]
                });
            return PlanRazonado {
                vision,
                modulos,
                tecnologia,
                es_local: false,
            };
        }

        // Degradación: plan local con la tecnología inferida del texto.
        let local = self.razonar_local(texto);
        PlanRazonado {
            es_local: false,
            ..local.plan
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_razonar_local_produce_plan() {
        let r = RazonadorQwen::nuevo();
        let res = r.razonar_local("crea un juego de plataformas");
        assert!(res.plan.es_local);
        assert!(res.plan.tecnologia.contains("Canvas"));
        assert!(res.plan.modulos.contains(&"src/entities/player.ts".to_string()));
    }

    #[test]
    fn test_razonar_local_dashboard() {
        let r = RazonadorQwen::nuevo();
        let res = r.razonar_local("dashboard de métricas");
        assert!(res.plan.tecnologia.contains("React"));
    }

    #[test]
    fn test_razonar_local_prompt_vacio() {
        let r = RazonadorQwen::nuevo();
        let res = r.razonar_local("   ");
        assert!(res.plan.vision.contains("genérica"));
    }

    #[test]
    fn test_razonar_forzado_no_usa_red() {
        let r = RazonadorQwen::nuevo().con_local_forzado();
        let res = r.razonar_local("juego rpg");
        assert!(res.uso_local);
    }

    #[tokio::test]
    async fn test_razonar_async_local_forzado() {
        let r = RazonadorQwen::nuevo().con_local_forzado();
        let res = r.razonar("juego de naves").await;
        assert!(res.uso_local);
        assert!(res.plan.es_local);
    }

    #[test]
    fn test_extraer_plan_de_json_valido() {
        let r = RazonadorQwen::nuevo();
        let plan = r.extraer_plan_del_json(
            r#"{"vision":"plataformero","tecnologia":"React+Canvas","modulos":["a.ts","b.ts"]}"#,
        );
        assert_eq!(plan.vision, "plataformero");
        assert_eq!(plan.modulos.len(), 2);
        assert!(!plan.es_local);
    }

    #[test]
    fn test_extraer_plan_de_json_corrupto_degrada() {
        let r = RazonadorQwen::nuevo();
        let plan = r.extraer_plan_del_json("texto sin json válido {incompleto");
        // Degrada a local pero mantiene es_local=false (vino de una respuesta del modelo).
        assert!(!plan.es_local);
        assert!(!plan.modulos.is_empty());
    }
}
