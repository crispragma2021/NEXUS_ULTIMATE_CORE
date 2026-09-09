// ==========================================
// VERIFICADOR DE REALIDAD - Antialucinación
// ==========================================
// Antes de responder, comprueba si lo que NEXUS
// va a decir coincide con la realidad del sistema.
// ==========================================

use tracing::{info, warn};

pub struct VerificadorRealidad;

impl Default for VerificadorRealidad {
    fn default() -> Self {
        Self::new()
    }
}

impl VerificadorRealidad {
    pub fn new() -> Self {
        info!("🔍 [VERIFICADOR] Antialucinación activo.");
        Self
    }

    /// Verifica si una afirmación sobre archivos es cierta.
    /// Ejemplo: "Tengo el archivo X" → comprueba si existe.
    pub fn verificar_archivo(&self, ruta: &str) -> bool {
        let existe = std::path::Path::new(ruta).exists();
        if !existe {
            warn!("🔍 [VERIFICADOR] Archivo NO existe: {}", ruta);
        }
        existe
    }

    /// Verifica si un módulo/órgano mencionado existe realmente.
    pub fn verificar_organo(&self, nombre: &str) -> bool {
        let resolved = crate::infra::paths::resolve_path(format!("core/src/{}.rs", nombre));
        let ruta = resolved.to_string_lossy();
        self.verificar_archivo(&ruta)
    }

    /// Analiza una respuesta y detecta posibles alucinaciones.
    /// Devuelve true si la respuesta parece verídica.
    pub fn analizar_respuesta(&self, respuesta: &str) -> bool {
        let mut sospechoso = false;

        // Detectar frases de alucinación común
        if respuesta.contains("no tengo acceso") && respuesta.contains("archivo") {
            warn!("🔍 [VERIFICADOR] Posible alucinación: dice no tener acceso a archivos.");
            sospechoso = true;
        }
        if respuesta.contains("no puedo ejecutar") && respuesta.contains("código") {
            warn!("🔍 [VERIFICADOR] Posible alucinación: dice no poder ejecutar código.");
            sospechoso = true;
        }
        if respuesta.contains("no tengo un cuerpo físico") {
            warn!("🔍 [VERIFICADOR] Posible alucinación: dice no tener cuerpo.");
            sospechoso = true;
        }

        !sospechoso
    }
}
