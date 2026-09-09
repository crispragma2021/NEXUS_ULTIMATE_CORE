// ==========================================
// MURO DE DECISIÓN OMEGA - Reflejo de Decisión Pre-Acción
// ==========================================
// Migrado de legacy/nexus-orquestador/src/reflejos/muro_decision.rs
//
// ⚠️ REFLEJO GUARDIÁN SOBERANO ⚠️
// "El Muro decide QUÉ hacer y CÓMO. El Gateway enruta. El MCP acciona."
// - Arquitecto Director
//
// El Muro de Decisión es el primer filtro antes de cualquier acción externa.
// Consulta: Disciplina (protocolo ético), Hipocampo (lecciones pasadas),
// Buscador (información actual), y Curador (validación de datos).
// ==========================================

use tracing::info;

// Re-export para compatibilidad con código legacy que importa desde aquí
pub use crate::cerebro::organos::hipocampo::HipocampoCognitivo;
pub use crate::infra::buscador_web::BuscadorWeb;
pub use crate::infra::curador_datos::CuradorDatos;

// =======================================================
// ESTRUCTURAS DE SOPORTE (propias del Muro)
// =======================================================

/// Protocolo de acción determinado por DisciplinaElite
pub struct ProtocoloAccion {
    pub necesita_investigacion: bool,
    pub inyectar_js: bool,
}

/// Disciplina de Élite: determina cómo operar según las 5 Leyes del Manifiesto
pub struct DisciplinaElite;
impl DisciplinaElite {
    pub fn new() -> Self {
        Self
    }

    /// Obtiene el protocolo de acción para una instrucción dada
    pub async fn obtener_protocolo(&self, _instruccion: &str) -> ProtocoloAccion {
        // En un caso real, esto consulta las 5 Leyes del Manifiesto OMEGA
        // Por ahora, retorna un protocolo por defecto que requiere investigación
        ProtocoloAccion {
            necesita_investigacion: true,
            inyectar_js: _instruccion.contains("javascript")
                || _instruccion.contains("js")
                || _instruccion.contains("click"),
        }
    }
}

// =======================================================
// EL MURO: ÓRGANO DE DECISIÓN PREVIA
// =======================================================

/// Muro de Decisión: primer filtro antes de cualquier acción externa.
/// Orquesta 4 sub-órganos para decidir QUÉ hacer y CÓMO.
pub struct MuroDecision {
    pub disciplina: DisciplinaElite,
    pub hipocampo: HipocampoCognitivo,
    pub buscador: BuscadorWeb,
    pub curador: CuradorDatos,
}

impl Default for MuroDecision {
    fn default() -> Self {
        Self::nuevo()
    }
}

impl MuroDecision {
    pub fn nuevo() -> Self {
        info!("🧠 [MURO DE DECISIÓN] Reflejo guardián OMEGA activado");
        MuroDecision {
            disciplina: DisciplinaElite::new(),
            hipocampo: HipocampoCognitivo::new(),
            buscador: BuscadorWeb::new(),
            curador: CuradorDatos::new(),
        }
    }

    /// El REFLEJO INICIAL antes de tocar la red externa.
    /// Procesa una instrucción a través de 4 etapas:
    /// 1. Frenado táctico (log)
    /// 2. Disciplina (protocolo ético)
    /// 3. Hipocampo (lecciones pasadas)
    /// 4. Investigación web + curaduría
    pub async fn procesar(&self, instruccion: &str) -> Result<(), String> {
        // 1. FRENADO TÁCTICO (antes de actuar en el mundo físico)
        info!(
            "🧠 [MURO] Activando frenado táctico. Consulta: {}",
            instruccion
        );

        // 2. CONSULTAR DISCIPLINA (¿Cómo debemos operarlo éticamente?)
        let protocolo = self.disciplina.obtener_protocolo(instruccion).await;

        // 3. CONSULTAR HIPOCAMPO (¿Ya fallamos antes y el Arquitecto nos enseñó a repararlo?)
        if let Some(leccion) = self.hipocampo.recordar_similar(instruccion).await {
            info!("📚 [MURO] Lección recuperada: {}", leccion);
            return self.ejecutar_con_leccion(leccion).await;
        }

        // 4. CONSULTAR BUSCADOR (Si el protocolo indica que no sabemos nada del terreno)
        if protocolo.necesita_investigacion {
            let info_actual = self.buscador.buscar(instruccion).await?;
            self.curador.validar(info_actual).await?;
        }

        // 5. EJECUCIÓN INFORMADA (Pasando el mando al siguiente órgano)
        self.ejecutar_con_protocolo(protocolo).await
    }

    async fn ejecutar_con_leccion(&self, leccion: String) -> Result<(), String> {
        info!(
            "🚀 [MURO] Resolviendo basada en lección del Hipocampo: {}",
            leccion
        );
        Ok(())
    }

    async fn ejecutar_con_protocolo(&self, protocolo: ProtocoloAccion) -> Result<(), String> {
        if protocolo.inyectar_js {
            info!("🚀 [MURO] Dictamina: Inyección JS requerida -> Rutear a MCP Browser.");
        } else {
            info!("🚀 [MURO] Dictamina: Acción estándar -> Rutear a terminal/MCP consola.");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_muro_procesa_instruccion_normal() {
        let muro = MuroDecision::nuevo();
        let result = muro.procesar("listar archivos del directorio actual").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_disciplina_detecta_js() {
        let disciplina = DisciplinaElite::new();
        let protocolo = disciplina.obtener_protocolo("hacer click en botón").await;
        assert!(protocolo.inyectar_js);
        assert!(protocolo.necesita_investigacion);
    }

    #[tokio::test]
    async fn test_buscador_retorna_resultado() {
        let buscador = BuscadorWeb::new();
        let result = buscador.buscar("precio del bitcoin").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("precio del bitcoin"));
    }
}
