use tracing::info;

// =======================================================
// ESTRUCTURAS DE SOPORTE
// =======================================================

pub struct ProtocoloAccion {
    pub necesita_investigacion: bool,
    pub inyectar_js: bool,
}

impl Default for ProtocoloAccion {
    fn default() -> Self {
        Self {
            necesita_investigacion: false,
            inyectar_js: false,
        }
    }
}

/// ⚠️ REFLEJO GUARDIÁN SOBERANO
/// "El Muro decide QUÉ hacer y CÓMO. El Gateway enruta. El MCP acciona."
pub struct MuroDecision {
    pub modo_seguro: bool,
}

impl Default for MuroDecision {
    fn default() -> Self {
        Self::new()
    }
}

impl MuroDecision {
    pub fn new() -> Self {
        Self { modo_seguro: true }
    }

    /// 🔍 Evalúa una instrucción y devuelve el protocolo de acción
    pub async fn evaluar(&self, instruccion: &str) -> ProtocoloAccion {
        info!("🧱 [MURO] Evaluando instrucción: {}", instruccion);

        let necesita_investigacion = self.modo_seguro
            || instruccion.contains("?")
            || instruccion.to_lowercase().contains("investigar")
            || instruccion.to_lowercase().contains("buscar");

        let inyectar_js = instruccion.contains("javascript")
            || instruccion.contains("script")
            || instruccion.contains("DOM")
            || instruccion.contains("shadow DOM");

        ProtocoloAccion {
            necesita_investigacion,
            inyectar_js,
        }
    }

    /// Activa/desactiva modo seguro
    pub fn set_modo_seguro(&mut self, seguro: bool) {
        self.modo_seguro = seguro;
        info!(
            "🧱 [MURO] Modo seguro: {}",
            if seguro { "ACTIVADO" } else { "DESACTIVADO" }
        );
    }
}
