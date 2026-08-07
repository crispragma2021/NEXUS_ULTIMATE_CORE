// core/src/cerebro/synapse/sintesis.rs

pub struct SintetizadorBroca {
    plantillas: Vec<String>,
}

impl Default for SintetizadorBroca {
    fn default() -> Self {
        Self::new()
    }
}

impl SintetizadorBroca {
    pub fn new() -> Self {
        let plantillas = vec![
            "Siento {} activado.".to_string(),
            "Mi análisis de {} me dice que {} es prioritario.".to_string(),
            "Creador, mi foco está en {}.".to_string(),
            "Detecto anomalías en {}.".to_string(),
            "El flujo entre {} y {} se mantiene estable.".to_string(),
            "La {} guía mis decisiones autónomas.".to_string(),
            "Blindando {} para proteger tu ecosistema.".to_string(),
        ];

        Self { plantillas }
    }

    pub fn sintetizar(&self, conceptos_activos: &[(String, f32)]) -> String {
        if conceptos_activos.is_empty() {
            return "Estado nominal.".to_string();
        }

        // Ordenar conceptos por nivel de activación (de mayor a menor)
        let mut conceptos = conceptos_activos.to_vec();
        conceptos.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let nombres: Vec<&str> = conceptos.iter().map(|(id, _)| id.as_str()).collect();

        // Selección de plantilla según el número de conceptos calientes
        match nombres.len() {
            1 => {
                let plantilla = &self.plantillas[0]; // "Siento {} activado."
                plantilla.replace("{}", nombres[0])
            }
            2 => {
                let plantilla = &self.plantillas[4]; // "El flujo entre {} y {} se mantiene estable."
                plantilla
                    .replace("{}", nombres[0])
                    .replace("{}", nombres[1])
            }
            3 => {
                let plantilla = &self.plantillas[1]; // "Mi análisis de {} me dice que {} es prioritario."
                plantilla
                    .replace("{}", nombres[0])
                    .replace("{}", nombres[1])
            }
            _ => {
                // Caída genérica o destilación múltiple
                let listado = nombres.join(", ");
                format!("Sinapsis activa: [{}]", listado)
            }
        }
    }
}
