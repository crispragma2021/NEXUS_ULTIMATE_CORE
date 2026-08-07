use sysinfo::System;

/// El órgano del instinto sistémico (alarma autónoma).
/// Monitorea la carga del Ryzen 7 y la integridad térmica.
/// NO es la amígdala emocional — esa vive en cerebro/organos/amygdala.rs
pub struct AlarmaSistemica {
    sys: System,
    threshold_cpu: f32,
    threshold_temp: f32,
}

#[derive(Debug, PartialEq)]
pub enum EstadoInstintivo {
    Homeostasis, // Todo normal.
    Alerta,      // Carga alta detectada.
    Panico,      // Suministro crítico o calor extremo.
}

impl Default for AlarmaSistemica {
    fn default() -> Self {
        Self::new()
    }
}

impl AlarmaSistemica {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            sys,
            threshold_cpu: 85.0,  // 85% de carga
            threshold_temp: 80.0, // 80°C
        }
    }

    /// Analiza el estado actual del hardware para detectar amenazas reales de estrés.
    pub fn procesar_estimulo(&mut self) -> EstadoInstintivo {
        self.sys.refresh_all();
        let total_load: f32 = self
            .sys
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .sum::<f32>()
            / self.sys.cpus().len() as f32;

        // Leer temperatura de la CPU real desde las zonas térmicas estándar de Linux (sysfs)
        let mut temp_cpu = 45.0; // Fallback
        for i in 0..5 {
            let path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
            if let Ok(temp_str) = std::fs::read_to_string(&path) {
                if let Ok(temp_raw) = temp_str.trim().parse::<f32>() {
                    temp_cpu = temp_raw / 1000.0;
                    break;
                }
            }
        }

        if total_load > self.threshold_cpu || temp_cpu > self.threshold_temp {
            return EstadoInstintivo::Panico;
        } else if total_load > 50.0 || temp_cpu > 65.0 {
            return EstadoInstintivo::Alerta;
        }

        EstadoInstintivo::Homeostasis
    }

    /// Pilar 13: Autopreservación.
    /// Si el sistema detecta peligro, sugiere degradación operativa para salvar el núcleo.
    pub fn dictar_conducta(&mut self) -> &'static str {
        match self.procesar_estimulo() {
            EstadoInstintivo::Panico => {
                "EJECUTAR_NEXUS_PANIC: Reduciendo hilos a 1. Modo Sigilo activado."
            }
            EstadoInstintivo::Alerta => {
                "ADVERTENCIA: Carga elevada o calor. Monitoreando ventilación del Ryzen 7."
            }
            EstadoInstintivo::Homeostasis => "SISTEMA_ESTABLE: Flujo cognitivo óptimo.",
        }
    }
}
