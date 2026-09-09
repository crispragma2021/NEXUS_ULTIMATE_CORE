// ==========================================
// GANGLIOS BASALES - Hábitos y Automatización
// ==========================================
// Como los ganglios basales humanos: automatiza
// tareas repetitivas para que la corteza no
// tenga que pensar en cada acción.
// ==========================================

use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct GangliosBasales {
    habitos: HashMap<String, (Instant, String)>,
}

impl Default for GangliosBasales {
    fn default() -> Self {
        Self::new()
    }
}

impl GangliosBasales {
    pub fn new() -> Self {
        Self {
            habitos: HashMap::new(),
        }
    }

    /// Registra una acción que podría convertirse en hábito.
    pub fn registrar_accion(&mut self, accion: &str, resultado: &str) {
        self.habitos
            .insert(accion.to_string(), (Instant::now(), resultado.to_string()));
    }

    /// Si una acción se ha repetido 3+ veces, devuelve el resultado automático.
    pub fn ejecutar_habito(&self, accion: &str) -> Option<String> {
        if let Some((timestamp, resultado)) = self.habitos.get(accion) {
            if timestamp.elapsed() < Duration::from_secs(3600) {
                return Some(resultado.clone());
            }
        }
        None
    }

    /// Detecta patrones repetitivos y sugiere automatización.
    pub fn detectar_patron(&self) -> Vec<String> {
        let mut patrones = Vec::new();
        for accion in self.habitos.keys() {
            if accion.contains("curl") || accion.contains("leer_archivo") {
                patrones.push(format!("Hábito detectado: {}", accion));
            }
        }
        patrones
    }
}
