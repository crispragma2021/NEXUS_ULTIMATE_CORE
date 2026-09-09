// 🧠 Corteza Motora — Planificación y coordinación de acciones motoras
// ==========================================
// Traduce intenciones en planes de acción para los efectores
// (garras, web, terminal). Puente entre la cognición y la acción física.
// ==========================================

use uuid::Uuid;

/// Plan de acción motora: secuencia de comandos atómicos para un efector
#[derive(Debug, Clone)]
pub struct PlanMotor {
    pub id: String,
    pub efector: String,        // "claw", "web", "terminal"
    pub intencion: String,      // descripción de la intención original
    pub secuencia: Vec<String>, // comandos atómicos
    pub prioridad: u8,          // 0 (máxima) a 255 (mínima)
    pub paso_actual: usize,     // índice del paso en ejecución
}

impl PlanMotor {
    /// Crea un plan motor con una secuencia de comandos genérica.
    /// La secuencia se deriva de la intención: una frase por comando.
    fn desde_intencion(intencion: &str, efector: &str, prioridad: u8) -> Self {
        let id = Uuid::new_v4().to_string();
        // Dividir la intención en comandos atómicos (por comas o puntos)
        let secuencia: Vec<String> = intencion
            .split(|c| c == ',' || c == '.')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            id,
            efector: efector.to_string(),
            intencion: intencion.to_string(),
            secuencia: if secuencia.is_empty() {
                vec![intencion.to_string()]
            } else {
                secuencia
            },
            prioridad,
            paso_actual: 0,
        }
    }
}

pub struct CortezaMotora {
    planes_pendientes: Vec<PlanMotor>,
}

impl Default for CortezaMotora {
    fn default() -> Self {
        Self::new()
    }
}

impl CortezaMotora {
    pub fn new() -> Self {
        Self {
            planes_pendientes: Vec::new(),
        }
    }

    /// Planifica una secuencia de acciones motoras a partir de una intención.
    ///
    /// Crea un [`PlanMotor`] con ID único, lo encola y lo retorna.
    /// La secuencia se deriva segmentando la intención por comas/puntos.
    pub fn planificar(&mut self, intencion: &str, efector: &str) -> PlanMotor {
        let prioridad = match efector {
            "claw" => 0,     // máxima: garras son críticas
            "terminal" => 1, // alta: comandos de sistema
            "web" => 2,      // media: acciones en red
            _ => 3,          // baja: otros
        };
        let plan = PlanMotor::desde_intencion(intencion, efector, prioridad);
        tracing::info!(
            "🧠 [CORTEZA MOTORA] Plan '{}' creado: {} pasos para {} (prioridad {})",
            plan.id[..8].to_string(),
            plan.secuencia.len(),
            plan.efector,
            plan.prioridad,
        );
        self.planes_pendientes.push(plan.clone());
        plan
    }

    /// Ejecuta el siguiente paso del plan motor de mayor prioridad.
    ///
    /// Retorna `Some(comando)` si hay un paso pendiente, `None` si todo
    /// está completo o no hay planes. Avanza `paso_actual` internamente.
    pub fn ejecutar_paso(&mut self) -> Option<String> {
        // Ordenar por prioridad (menor = más urgente), luego por antigüedad
        self.planes_pendientes
            .sort_by_key(|p| (p.prioridad, p.paso_actual));

        let plan = self.planes_pendientes.first_mut()?;

        if plan.paso_actual >= plan.secuencia.len() {
            // Plan completado, removerlo
            let completo = self.planes_pendientes.remove(0);
            tracing::info!(
                "🧠 [CORTEZA MOTORA] Plan '{}' completado ({})",
                &completo.id[..8],
                completo.intencion
            );
            return self.ejecutar_paso(); // recursión: intentar siguiente plan
        }

        let comando = plan.secuencia[plan.paso_actual].clone();
        plan.paso_actual += 1;
        tracing::debug!(
            "🧠 [CORTEZA MOTORA] Paso {}/{} del plan '{}': {}",
            plan.paso_actual,
            plan.secuencia.len(),
            &plan.id[..8],
            comando,
        );
        Some(comando)
    }

    /// Cancela un plan motor en curso por su ID.
    ///
    /// Retorna `true` si el plan existía y fue removido.
    pub fn cancelar(&mut self, plan_id: &str) -> bool {
        let pos = self.planes_pendientes.iter().position(|p| p.id == plan_id);
        if let Some(idx) = pos {
            let plan = self.planes_pendientes.remove(idx);
            tracing::info!(
                "🧠 [CORTEZA MOTORA] Plan '{}' cancelado ({})",
                &plan.id[..8],
                plan.intencion
            );
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planificar_crea_plan_con_secuencia() {
        let mut cm = CortezaMotora::new();
        let plan = cm.planificar(
            "explorar directorio, listar archivos, analizar logs",
            "terminal",
        );
        assert_eq!(plan.efector, "terminal");
        assert_eq!(plan.secuencia.len(), 3);
        assert_eq!(plan.secuencia[0], "explorar directorio");
        assert_eq!(plan.secuencia[1], "listar archivos");
    }

    #[test]
    fn test_planificar_asigna_prioridad_por_efector() {
        let mut cm = CortezaMotora::new();
        let claw = cm.planificar("cerrar pinza", "claw");
        let term = cm.planificar("ejecutar comando", "terminal");
        let web = cm.planificar("enviar request", "web");
        assert!(claw.prioridad < term.prioridad);
        assert!(term.prioridad < web.prioridad);
    }

    #[test]
    fn test_ejecutar_paso_devuelve_comandos_en_orden() {
        let mut cm = CortezaMotora::new();
        cm.planificar("paso uno, paso dos, paso tres", "terminal");
        assert_eq!(cm.ejecutar_paso(), Some("paso uno".to_string()));
        assert_eq!(cm.ejecutar_paso(), Some("paso dos".to_string()));
        assert_eq!(cm.ejecutar_paso(), Some("paso tres".to_string()));
        assert_eq!(cm.ejecutar_paso(), None); // ya no hay pasos
    }

    #[test]
    fn test_ejecutar_paso_sin_planes_devuelve_none() {
        let mut cm = CortezaMotora::new();
        assert_eq!(cm.ejecutar_paso(), None);
    }

    #[test]
    fn test_cancelar_remueve_plan() {
        let mut cm = CortezaMotora::new();
        let plan = cm.planificar("acción de prueba", "claw");
        assert!(cm.cancelar(&plan.id));
        assert_eq!(cm.ejecutar_paso(), None); // plan eliminado
    }

    #[test]
    fn test_cancelar_id_inexistente_devuelve_false() {
        let mut cm = CortezaMotora::new();
        assert!(!cm.cancelar("no-existe"));
    }

    #[test]
    fn test_prioridad_ejecuta_primero_plan_mas_urgente() {
        let mut cm = CortezaMotora::new();
        cm.planificar("baja prioridad", "web");
        cm.planificar("alta prioridad", "claw");
        // El claw (prioridad 0) debe ejecutarse antes que web (prioridad 2)
        assert_eq!(cm.ejecutar_paso(), Some("alta prioridad".to_string()));
    }
}
