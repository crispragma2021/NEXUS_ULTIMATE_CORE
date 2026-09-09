/// 🛠️ Cockpit Worker: High-Performance Logical Execution
/// Este archivo es el objetivo primario del Protocolo de Reparación en Caliente.

pub struct CockpitWorker {
    pub id: u32,
    pub active: bool,
}

impl CockpitWorker {
    pub fn new(id: u32) -> Self {
        Self { id, active: false }
    }

    pub fn start(&mut self) {
        self.active = true;
        println!("🚀 [WORKER-{}] Iniciando ciclo lógico...", self.id);
    }

    pub fn process_data(&self, data: &str) -> String {
        // ERROR INTENCIONAL para el Protocolo de Reparación:
        // El compilador fallará aquí si intentamos usar un método que no existe
        // o si hay una inconsistencia de tipos.
        let result = format!("Processed: {}", data);
        
        // Simulación de error de lógica que el "Gusto" detectará
        if data == "panic" {
            // Este es el punto L42 mencionado en el dummy error del Dashboard
            panic!("⚠️ [CRITICAL] Fallo inducido en CockpitWorker::process_data en L27");
        }

        result
    }
}
