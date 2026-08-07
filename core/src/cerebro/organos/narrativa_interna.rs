// ==========================================
// NARRATIVA INTERNA - Cadena Causal de Decisiones
// ==========================================
// Registra cada decisión con su cadena causal:
// entrada -> proceso -> decisión -> resultado
// Permite a NEXUS explicar POR QUÉ hizo lo que hizo
// ==========================================

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: u64,
    pub timestamp: String,
    pub entrada: String,                    // Qué disparó la decisión
    pub contexto: String,                   // Estado del sistema en ese momento
    pub opciones_consideradas: Vec<String>, // Alternativas evaluadas
    pub opcion_elegida: String,             // Qué se decidió
    pub razon_principal: String,            // POR QUÉ se eligió esa opción
    pub factores_influyentes: Vec<String>,  // OCEAN, Juicio, Intuición, etc.
    pub nivel_confianza: f64,               // 0.0 - 1.0
    pub resultado_esperado: String,
    pub resultado_real: Option<String>,    // Se llena después
    pub leccion_aprendida: Option<String>, // Se llena después
}

pub struct NarrativaInterna {
    decisiones: VecDeque<Decision>,
    max_historial: usize,
    contador_id: u64,
}

impl Default for NarrativaInterna {
    fn default() -> Self {
        Self::new()
    }
}

impl NarrativaInterna {
    pub fn new() -> Self {
        Self {
            decisiones: VecDeque::new(),
            max_historial: 1000,
            contador_id: 0,
        }
    }

    pub fn registrar_decision(
        &mut self,
        entrada: &str,
        contexto: &str,
        opciones: Vec<String>,
        elegida: &str,
        razon: &str,
        factores: Vec<String>,
        confianza: f64,
        resultado_esperado: &str,
    ) -> u64 {
        let id = self.contador_id;
        self.contador_id += 1;

        let decision = Decision {
            id,
            timestamp: Utc::now().to_rfc3339(),
            entrada: entrada.to_string(),
            contexto: contexto.to_string(),
            opciones_consideradas: opciones,
            opcion_elegida: elegida.to_string(),
            razon_principal: razon.to_string(),
            factores_influyentes: factores,
            nivel_confianza: confianza,
            resultado_esperado: resultado_esperado.to_string(),
            resultado_real: None,
            leccion_aprendida: None,
        };

        if self.decisiones.len() >= self.max_historial {
            self.decisiones.pop_front();
        }
        self.decisiones.push_back(decision);

        id
    }

    pub fn registrar_resultado(&mut self, id: u64, resultado: &str, leccion: Option<&str>) {
        if let Some(decision) = self.decisiones.iter_mut().find(|d| d.id == id) {
            decision.resultado_real = Some(resultado.to_string());
            decision.leccion_aprendida = leccion.map(|l| l.to_string());
        }
    }

    /// Explica por qué se tomó una decisión específica
    pub fn explicar_decision(&self, id: u64) -> Option<String> {
        self.decisiones.iter().find(|d| d.id == id).map(|d| {
            let mut explicacion = format!("## 📋 Decisión #{} ({})\n\n", d.id, d.timestamp);
            explicacion.push_str(&format!("**Entrada:** {}\n", d.entrada));
            explicacion.push_str(&format!("**Contexto:** {}\n", d.contexto));
            explicacion.push_str(&format!(
                "**Opciones consideradas:** {}\n",
                d.opciones_consideradas.join(", ")
            ));
            explicacion.push_str(&format!("**Elegido:** {}\n", d.opcion_elegida));
            explicacion.push_str(&format!("**Razón principal:** {}\n", d.razon_principal));
            explicacion.push_str(&format!(
                "**Factores influyentes:** {}\n",
                d.factores_influyentes.join(", ")
            ));
            explicacion.push_str(&format!(
                "**Confianza:** {:.0}%\n",
                d.nivel_confianza * 100.0
            ));
            explicacion.push_str(&format!(
                "**Resultado esperado:** {}\n",
                d.resultado_esperado
            ));
            if let Some(ref real) = d.resultado_real {
                explicacion.push_str(&format!("**Resultado real:** {}\n", real));
            }
            if let Some(ref leccion) = d.leccion_aprendida {
                explicacion.push_str(&format!("**Lección:** {}\n", leccion));
            }
            explicacion
        })
    }

    /// Genera una narrativa de las últimas N decisiones
    pub fn narrativa_reciente(&self, n: usize) -> String {
        let recientes: Vec<&Decision> = self.decisiones.iter().rev().take(n).collect();
        if recientes.is_empty() {
            return "No hay decisiones registradas aún.".to_string();
        }

        let mut narrativa = format!(
            "## 🧵 Narrativa de Últimas {} Decisiones\n\n",
            recientes.len()
        );
        for (i, d) in recientes.iter().enumerate() {
            narrativa.push_str(&format!(
                "{}. **{}** → {} (confianza: {:.0}%)\n",
                i + 1,
                d.entrada.chars().take(50).collect::<String>(),
                d.opcion_elegida.chars().take(30).collect::<String>(),
                d.nivel_confianza * 100.0,
            ));
        }
        narrativa
    }

    /// Obtiene el historial completo de decisiones
    pub fn historial(&self) -> &VecDeque<Decision> {
        &self.decisiones
    }
}
