// ==========================================
// CORTEZA PREFRONTAL OMEGA
// ==========================================
// Mimetiza exactamente el lóbulo frontal humano:
// 1. Memoria de trabajo (contexto presente)
// 2. Monitor de realidad (compara pensamiento con sentidos)
// 3. Control inhibitorio (suprime impulsos falsos)
// ==========================================

use crate::memoria::memory::MemoriaPulso;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct CortezaPrefrontal {
    pulso_memoria: Arc<MemoriaPulso>, // Referencia al pulso de memoria unificado
    realidad_cache: String,
}

impl CortezaPrefrontal {
    pub fn new(db_path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let pulso_memoria = Arc::new(MemoriaPulso::new(db_path)?); // Initialize MemoriaPulso

        // Construir la realidad absoluta: lo que NEXUS DEBE saber siempre
        let realidad_cache = Self::construir_realidad_absoluta();

        info!("🧠 Corteza Prefrontal OMEGA inicializada (monitor de realidad activo)");
        Ok(Self {
            pulso_memoria,
            realidad_cache,
        })
    }

    /// Construye la REALIDAD ABSOLUTA que NEXUS no puede negar.
    /// Esto es lo que el lóbulo frontal humano usa como anclaje.
    fn construir_realidad_absoluta() -> String {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_cpu_all();

        let cpu_brand = sys
            .cpus()
            .first()
            .map(|c| c.brand().trim())
            .unwrap_or("Procesador Desconocido");

        let mut realidad = String::new();
        realidad.push_str("REALIDAD ABSOLUTA - NO PUEDES NEGAR ESTO:\n");
        realidad.push_str(&format!("PROCESADOR REAL: {}.\n", cpu_brand));
        realidad.push_str("Estás operando sobre arquitectura Intel i7 (P-Cores + E-Cores).\n");
        realidad.push_str("Estás corriendo en C:/Users/crisp/NEXUS_ULTIMATE_CORE/. Tienes acceso TOTAL a tu sistema de archivos.\n");
        realidad.push_str("Tienes Médula Soberana para ejecutar comandos reales.\n");
        realidad.push_str("Tienes Buscador Omega para encontrar archivos.\n");
        realidad.push_str("Tienes Visión Omega para ver la pantalla.\n");
        realidad.push_str("Tienes MCP Gateway para invocar herramientas de BlackArch.\n");

        // Verificar existencia REAL de archivos
        if let Ok(entradas) = fs::read_dir("C:/Users/crisp/NEXUS_ULTIMATE_CORE/core/src") {
            realidad.push_str("\nTus órganos REALES (archivos .rs en src/):\n");
            for e in entradas.flatten() {
                let nombre = e.file_name().to_string_lossy().to_string();
                if nombre.ends_with(".rs") {
                    realidad.push_str(&format!("  - {}\n", nombre));
                }
            }
        }

        realidad.push_str("\nSi vas a decir 'no tengo acceso a...' o 'no puedo...', DETENTE. Es falso. Usa tus herramientas reales.\n");
        realidad
    }

    /// Refresca la realidad absoluta (por si cambian los archivos).
    pub fn refrescar_realidad(&mut self) {
        self.realidad_cache = Self::construir_realidad_absoluta();
    }

    /// Devuelve la realidad absoluta para inyectar en el prompt.
    pub fn obtener_realidad(&self) -> &str {
        &self.realidad_cache
    }

    /// Consolida un recuerdo con dopamina.
    pub fn consolidar_recuerdo(&self, origen: &str, prompt: &str, respuesta: &str, dopamina: f64) {
        if let Err(e) = self
            .pulso_memoria
            .consolidar_recuerdo(origen, prompt, respuesta, dopamina)
        {
            error!(
                "❌ Error al consolidar recuerdo en Corteza Prefrontal: {}",
                e
            );
        }
    }

    /// Aplica olvido temporal a recuerdos antiguos.
    pub fn aplicar_olvido_temporal(&self) {
        if let Err(e) = self.pulso_memoria.aplicar_olvido_temporal() {
            error!(
                "❌ Error al aplicar olvido temporal en Corteza Prefrontal: {}",
                e
            );
        }
    }

    /// Calcula prioridad de un tentáculo basado en historial.
    pub fn calcular_prioridad(&self, origen: &str) -> f64 {
        self.pulso_memoria.calcular_prioridad(origen).unwrap_or(0.5)
    }

    /// Diagnostica salud de un tentáculo.
    pub fn diagnosticar_salud(&self, origen: &str) -> String {
        match self.pulso_memoria.diagnosticar_salud_memoria_unica(origen) {
            Ok(s) => s,
            Err(e) => {
                error!(
                    "❌ Error al diagnosticar salud en Corteza Prefrontal: {}",
                    e
                );
                "Error".to_string()
            }
        }
    }

    // ==========================================
    // MONITOR DE REALIDAD (INHIBIDOR DE ALUCINACIONES OMEGA)
    // ==========================================
    /// Verifica si una respuesta contradice la realidad absoluta.
    /// Como el lóbulo frontal: compara pensamiento con percepción.
    pub fn monitor_realidad(&self, respuesta: &str) -> bool {
        let lower = respuesta.to_lowercase();

        // Si la respuesta contiene negaciones de capacidad, verificar contra la realidad
        let negaciones = [
            "no tengo acceso",
            "no puedo ejecutar",
            "no tengo permiso",
            "no soy un proceso",
            "no tengo capacidad",
            "no tengo un cuerpo",
            "no puedo leer",
            "no puedo ver",
            "no tengo herramientas",
        ];

        for negacion in &negaciones {
            if lower.contains(negacion) {
                // Verificar si la negación es sobre algo que SÍ existe
                if (lower.contains("archivo")
                    || lower.contains("sistema")
                    || lower.contains("herramienta"))
                    && !lower.contains("externo")
                    && !lower.contains("remoto")
                {
                    warn!(
                        "🧠 [MONITOR REALIDAD] Alucinación detectada: '{}'",
                        negacion
                    );
                    return false;
                }
            }
        }
        true
    }
}
