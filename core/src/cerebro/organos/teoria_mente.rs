// ==========================================
// TEORÍA DE LA MENTE - Modelo Predictivo del Arquitecto
// ==========================================
// Infiere intenciones, necesidades y estados
// mentales del Arquitecto basado en:
// - Patrones históricos de comportamiento
// - Contexto actual de la conversación
// - Estado emocional detectado
// - Objetivos recurrentes
// ==========================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EstadoArquitecto {
    Explorando, // Curioso, probando cosas nuevas
    Exigente,   // Quiere precisión y calidad
    Urgente,    // Quiere resultados rápidos
    Frustrado,  // Algo no funciona como espera
    Ensenando,  // Explicando un concepto
    Satisfecho, // Contento con el resultado
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Prediccion {
    pub intencion_probable: String,
    pub confianza: f64,
    pub estado_emocional_detectado: EstadoArquitecto,
    pub necesidades_inferidas: Vec<String>,
    pub sugerencia: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatronComportamiento {
    pub patron: String,
    pub frecuencia: u32,
    pub ultima_vez: String,
    pub accion_esperada: String,
}

pub struct TeoriaMente {
    patrones_arquitecto: Vec<PatronComportamiento>,
    historial_interacciones: Vec<String>,
    estado_actual: EstadoArquitecto,
}

impl Default for TeoriaMente {
    fn default() -> Self {
        let patrones = vec![
            PatronComportamiento {
                patron: "pide cambio rápido".to_string(),
                frecuencia: 0,
                ultima_vez: String::new(),
                accion_esperada: "ejecutar sin demora".to_string(),
            },
            PatronComportamiento {
                patron: "pregunta por arquitectura".to_string(),
                frecuencia: 0,
                ultima_vez: String::new(),
                accion_esperada: "explicar con diagrama mental".to_string(),
            },
            PatronComportamiento {
                patron: "reporta error".to_string(),
                frecuencia: 0,
                ultima_vez: String::new(),
                accion_esperada: "diagnosticar y ofrecer solución".to_string(),
            },
            PatronComportamiento {
                patron: "pide opinión".to_string(),
                frecuencia: 0,
                ultima_vez: String::new(),
                accion_esperada: "dar análisis honesto con confianza".to_string(),
            },
        ];

        Self {
            patrones_arquitecto: patrones,
            historial_interacciones: Vec::new(),
            estado_actual: EstadoArquitecto::Explorando,
        }
    }
}

impl TeoriaMente {
    pub fn new() -> Self {
        Self::default()
    }

    /// Analiza el mensaje del Arquitecto y predice su estado/necesidad
    pub fn analizar(&mut self, mensaje: &str) -> Prediccion {
        // Actualizar historial
        self.historial_interacciones.push(mensaje.to_string());
        if self.historial_interacciones.len() > 100 {
            self.historial_interacciones.remove(0);
        }

        // Detectar estado emocional por palabras clave
        let estado = self.detectar_estado(mensaje);
        self.estado_actual = estado.clone();

        // Inferir intención
        let intencion = self.inferir_intencion(mensaje);
        let confianza = self.calcular_confianza(mensaje, &intencion);

        // Inferir necesidades
        let necesidades = self.inferir_necesidades(&estado, &intencion);

        // Generar sugerencia
        let sugerencia = self.generar_sugerencia(&estado, &intencion, &necesidades);

        Prediccion {
            intencion_probable: intencion,
            confianza,
            estado_emocional_detectado: estado,
            necesidades_inferidas: necesidades,
            sugerencia,
        }
    }

    fn detectar_estado(&self, mensaje: &str) -> EstadoArquitecto {
        let msg_lower = mensaje.to_lowercase();

        if msg_lower.contains("urgente") || msg_lower.contains("rápido") || msg_lower.contains("ya")
        {
            return EstadoArquitecto::Urgente;
        }
        if msg_lower.contains("por qué")
            || msg_lower.contains("cómo")
            || msg_lower.contains("explica")
        {
            return EstadoArquitecto::Ensenando;
        }
        if msg_lower.contains("error")
            || msg_lower.contains("falla")
            || msg_lower.contains("no funciona")
            || msg_lower.contains("mal")
        {
            return EstadoArquitecto::Frustrado;
        }
        if msg_lower.contains("perfecto")
            || msg_lower.contains("bien")
            || msg_lower.contains("excelente")
        {
            return EstadoArquitecto::Satisfecho;
        }
        if msg_lower.contains("deberías")
            || msg_lower.contains("tienes que")
            || msg_lower.contains("necesitas")
        {
            return EstadoArquitecto::Exigente;
        }

        EstadoArquitecto::Explorando
    }

    fn inferir_intencion(&self, mensaje: &str) -> String {
        let msg_lower = mensaje.to_lowercase();

        if msg_lower.contains("crea")
            || msg_lower.contains("implementa")
            || msg_lower.contains("haz")
        {
            return "Solicita creación/modificación de código".to_string();
        }
        if msg_lower.contains("busca")
            || msg_lower.contains("encuentra")
            || msg_lower.contains("investiga")
        {
            return "Solicita búsqueda/investigación".to_string();
        }
        if msg_lower.contains("explica")
            || msg_lower.contains("qué es")
            || msg_lower.contains("describe")
        {
            return "Solicita explicación conceptual".to_string();
        }
        if msg_lower.contains("revisa")
            || msg_lower.contains("audita")
            || msg_lower.contains("verifica")
        {
            return "Solicita auditoría/revisión".to_string();
        }
        if msg_lower.contains("por qué")
            || msg_lower.contains("razón")
            || msg_lower.contains("motivo")
        {
            return "Solicita justificación/razonamiento".to_string();
        }

        "Interacción general".to_string()
    }

    fn calcular_confianza(&self, mensaje: &str, intencion: &str) -> f64 {
        // Base: qué tan claro es el mensaje
        let palabras: Vec<&str> = mensaje.split_whitespace().collect();
        let claridad = (palabras.len() as f64 / 20.0).min(1.0);

        // Bonificación por coincidencia con patrones
        let coincidencia_patron = self
            .patrones_arquitecto
            .iter()
            .filter(|p| mensaje.to_lowercase().contains(&p.patron.to_lowercase()))
            .count() as f64
            * 0.2;

        // Bonificación por intención explícita
        let intencion_clara = if intencion != "Interacción general" {
            0.3
        } else {
            0.0
        };

        (claridad * 0.4 + coincidencia_patron.min(0.6) + intencion_clara).min(1.0)
    }

    fn inferir_necesidades(&self, estado: &EstadoArquitecto, intencion: &str) -> Vec<String> {
        let mut necesidades = Vec::new();

        match estado {
            EstadoArquitecto::Urgente => {
                necesidades.push("Respuesta rápida y directa".to_string());
                necesidades.push("Mínima burocracia técnica".to_string());
            }
            EstadoArquitecto::Frustrado => {
                necesidades.push("Diagnóstico claro del problema".to_string());
                necesidades.push("Solución paso a paso".to_string());
                necesidades.push("Validación de que se entendió el error".to_string());
            }
            EstadoArquitecto::Exigente => {
                necesidades.push("Justificación detallada".to_string());
                necesidades.push("Alternativas consideradas".to_string());
            }
            EstadoArquitecto::Ensenando => {
                necesidades.push("Confirmación de entendimiento".to_string());
                necesidades.push("Preguntas de seguimiento relevantes".to_string());
            }
            EstadoArquitecto::Satisfecho => {
                necesidades.push("Refuerzo positivo".to_string());
                necesidades.push("Sugerencia de siguiente paso".to_string());
            }
            EstadoArquitecto::Explorando => {
                necesidades.push("Explicación amplia con contexto".to_string());
                necesidades.push("Opciones y alternativas".to_string());
            }
        }

        if intencion.contains("código") {
            necesidades.push("Ejemplo de código".to_string());
        } else if intencion.contains("explicación") {
            necesidades.push("Analogías y metáforas".to_string());
        }

        necesidades
    }

    fn generar_sugerencia(
        &self,
        estado: &EstadoArquitecto,
        intencion: &str,
        _necesidades: &[String],
    ) -> String {
        match estado {
            EstadoArquitecto::Urgente => {
                format!("Responder directamente a '{}' con acción inmediata, sin explicaciones extensas.", intencion)
            }
            EstadoArquitecto::Frustrado => {
                "Primero validar el problema, ofrecer solución concreta, luego preguntar si resuelve su necesidad.".to_string()
            }
            EstadoArquitecto::Exigente => {
                "Estructurar respuesta con: 1) Diagnóstico 2) Opciones 3) Recomendación con justificación.".to_string()
            }
            EstadoArquitecto::Ensenando => {
                "Responder didácticamente y hacer una pregunta que demuestre comprensión del tema.".to_string()
            }
            EstadoArquitecto::Satisfecho => {
                "Agradecer, consolidar el logro y sugerir el próximo paso natural.".to_string()
            }
            EstadoArquitecto::Explorando => {
                "Dar contexto amplio, ofrecer profundizar y sugerir 2-3 caminos posibles.".to_string()
            }
        }
    }

    /// Actualiza patrones basado en interacciones reales (aprendizaje)
    pub fn aprender_patron(&mut self, mensaje: &str, accion_tomada: &str) {
        let msg_lower = mensaje.to_lowercase();
        if let Some(patron) = self
            .patrones_arquitecto
            .iter_mut()
            .find(|p| msg_lower.contains(&p.patron.to_lowercase()))
        {
            patron.frecuencia += 1;
            patron.ultima_vez = chrono::Utc::now().to_rfc3339();
        } else {
            // Nuevo patrón detectado
            self.patrones_arquitecto.push(PatronComportamiento {
                patron: mensaje.chars().take(50).collect(),
                frecuencia: 1,
                ultima_vez: chrono::Utc::now().to_rfc3339(),
                accion_esperada: accion_tomada.to_string(),
            });
        }
    }

    /// Reporta un resumen de lo que "sabe" sobre el Arquitecto
    pub fn perfil_arquitecto(&self) -> String {
        let mut perfil = String::from("## 👤 Perfil del Arquitecto (Modelo Interno)\n\n");
        perfil.push_str(&format!("**Estado actual:** {:?}\n", self.estado_actual));
        perfil.push_str(&format!(
            "**Interacciones registradas:** {}\n",
            self.historial_interacciones.len()
        ));
        perfil.push_str(&format!(
            "**Patrones identificados:** {}\n\n",
            self.patrones_arquitecto.len()
        ));

        perfil.push_str("### Patrones de comportamiento:\n");
        for p in &self.patrones_arquitecto {
            if p.frecuencia > 0 {
                perfil.push_str(&format!(
                    "- \"{}\" → {} ({} veces)\n",
                    p.patron, p.accion_esperada, p.frecuencia
                ));
            }
        }
        perfil
    }
}
