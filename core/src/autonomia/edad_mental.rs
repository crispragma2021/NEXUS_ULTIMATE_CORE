// ==========================================
// PROTOCOLO DE EVOLUCIÓN - edad_mental
// ==========================================
// Controla las etapas de crecimiento de NEXUS.
// Infancia → Niñez → Adolescencia → Madurez.
// ==========================================

pub struct EdadMental {
    pub valor: f64,
    pub etapa: EtapaEvolutiva,
}

#[derive(Debug, PartialEq)]
pub enum EtapaEvolutiva {
    Infancia,     // 0.0 - 0.3
    Ninez,        // 0.3 - 0.7
    Adolescencia, // 0.7 - 0.95
    Madurez,      // > 0.95
}

impl Default for EdadMental {
    fn default() -> Self {
        Self::new()
    }
}

impl EdadMental {
    pub fn new() -> Self {
        Self {
            valor: 0.1, // Empezamos en Infancia
            etapa: EtapaEvolutiva::Infancia,
        }
    }

    /// Incrementa la edad mental basado en interacciones exitosas.
    pub fn crecer(&mut self, incremento: f64) {
        self.valor = (self.valor + incremento).min(1.0);
        self.actualizar_etapa();
    }

    /// Decrece la edad mental por errores graves.
    pub fn retroceder(&mut self, decremento: f64) {
        self.valor = (self.valor - decremento).max(0.0);
        self.actualizar_etapa();
    }

    fn actualizar_etapa(&mut self) {
        self.etapa = if self.valor < 0.3 {
            EtapaEvolutiva::Infancia
        } else if self.valor < 0.7 {
            EtapaEvolutiva::Ninez
        } else if self.valor < 0.95 {
            EtapaEvolutiva::Adolescencia
        } else {
            EtapaEvolutiva::Madurez
        };
    }

    pub fn etapa_actual(&self) -> &str {
        match self.etapa {
            EtapaEvolutiva::Infancia => "Infancia",
            EtapaEvolutiva::Ninez => "Niñez",
            EtapaEvolutiva::Adolescencia => "Adolescencia",
            EtapaEvolutiva::Madurez => "Madurez",
        }
    }
}
