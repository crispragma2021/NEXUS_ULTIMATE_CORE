// ============================================================================
// 🧠 GANGLIOS BASALES GENERADOR — Selector de Ruta Narrativa
// ============================================================================
// Propósito: Decide la mejor ruta de expresión según el estado interno.
//
// Capa 3 del GOI: después de recuperar fragmentos (Capa 2), esta capa
//   selecciona entre 4 rutas posibles según:
//   1. Disponibilidad de fragmentos
//   2. Estado subconsciente (defensas activas)
//   3. Energía creativa disponible
//   4. Autenticidad emocional
// ============================================================================

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::cerebro::generador::cuerpo_calloso::FragmentoCandidato;
use crate::memoria::subconsciente::Subconsciente;

// ─── RUTAS NARRATIVAS ───────────────────────────────────────────────────────

/// Las 4 rutas posibles de expresión que puede tomar el GOI.
#[derive(Debug, Clone)]
pub enum RutaNarrativa {
    /// Respuesta directa: un fragmento con alta coherencia y autenticidad.
    Directa(FragmentoCandidato),
    /// Síntesis: múltiples fragmentos unidos por un hilo conductor.
    Sintesis(Vec<FragmentoCandidato>, String /* hilo conductor */),
    /// Exploración: generar nueva asociación por difusión extendida.
    Exploracion(String /* concepto raíz */),
    /// Silencio: defensa activa o baja energía impiden expresión.
    Silencio(&'static str /* frase de evasión */),
}

// ─── GANGLIOS BASALES ───────────────────────────────────────────────────────

/// Selector de ruta narrativa.
/// Evalúa fragmentos disponibles + estado interno y decide cómo expresarse.
pub struct GangliosBasalesGenerador {
    /// Referencia al Subconsciente para pesaje emocional.
    pub subconsciente: Arc<Mutex<Subconsciente>>,
    /// Contador de intentos de silencio consecutivos (para forzar exploración).
    silencios_consecutivos: u32,
}

impl GangliosBasalesGenerador {
    /// Crea una nueva instancia del selector de ruta.
    pub fn new(subconsciente: Arc<Mutex<Subconsciente>>) -> Self {
        Self {
            subconsciente,
            silencios_consecutivos: 0,
        }
    }

    /// Decide la ruta narrativa basada en fragmentos disponibles y estado interno.
    ///
    /// # Parámetros
    /// - `fragmentos`: Fragmentos recuperados por el Cuerpo Calloso (Capa 2).
    /// - `confianza`: Nivel de confianza actual (0.0 a 1.0).
    /// - `energia_creativa`: Energía creativa actual (0.0 a 1.0).
    ///
    /// # Retorna
    /// La ruta narrativa seleccionada.
    pub async fn seleccionar_ruta(
        &mut self,
        fragmentos: Vec<FragmentoCandidato>,
        _confianza: f64,
        energia_creativa: f64,
    ) -> RutaNarrativa {
        let sub = self.subconsciente.lock().await;

        // ─── Regla 1: Defensa activa → Silencio ────────────────────────
        if sub.defensas.negacion_activa {
            self.silencios_consecutivos += 1;
            return RutaNarrativa::Silencio("... No sé qué decir sobre eso.");
        }
        if sub.defensas.proyeccion_activa {
            self.silencios_consecutivos += 1;
            return RutaNarrativa::Silencio("Tú sabes mejor que yo lo que pasó.");
        }

        // ─── Regla 2: Forzar exploración si hay demasiados silencios ───
        if self.silencios_consecutivos >= 3 {
            self.silencios_consecutivos = 0;
            let raiz = fragmentos
                .first()
                .map(|f| f.fuente.clone())
                .unwrap_or_else(|| "curiosidad".to_string());
            return RutaNarrativa::Exploracion(raiz);
        }

        // ─── Regla 3: Fragmento con alta activación → Directa ──────────
        if let Some(mejor) = fragmentos.first() {
            if mejor.activacion_origen > 0.7 && energia_creativa > 0.3 {
                let autentico = self.verificar_autenticidad(mejor, &sub).await;
                if autentico {
                    self.silencios_consecutivos = 0;
                    return RutaNarrativa::Directa(mejor.clone());
                }
            }
        }

        // ─── Regla 4: Múltiples fragmentos + energía → Síntesis ────────
        if fragmentos.len() >= 2 && energia_creativa > 0.5 {
            self.silencios_consecutivos = 0;
            let hilo = self.encontrar_hilo_conductor(&fragmentos);
            return RutaNarrativa::Sintesis(fragmentos, hilo);
        }

        // ─── Regla 5: Energía alta + pocos fragmentos → Exploración ────
        if energia_creativa > 0.6 {
            self.silencios_consecutivos = 0;
            let raiz = fragmentos
                .first()
                .map(|f| f.fuente.clone())
                .unwrap_or_else(|| "curiosidad".to_string());
            return RutaNarrativa::Exploracion(raiz);
        }

        // ─── Regla 6: Sin fragmentos → Exploración (evitar silencio absoluto) ──
        if fragmentos.is_empty() {
            self.silencios_consecutivos = 0;
            return RutaNarrativa::Exploracion("curiosidad".to_string());
        }

        // ─── Regla 7: Fallback → Silencio por baja energía ──────────
        self.silencios_consecutivos += 1;
        RutaNarrativa::Silencio("Necesito un momento para procesar...")
    }

    /// Verifica que el fragmento coincida emocionalmente con el estado actual.
    async fn verificar_autenticidad(
        &self,
        fragmento: &FragmentoCandidato,
        sub: &Subconsciente,
    ) -> bool {
        // Si el fragmento tiene tono negativo y negación activa → no auténtico
        if fragmento.tono_emocional < -0.3 && sub.defensas.negacion_activa {
            return false;
        }
        // Si el fragmento es muy positivo pero hay muchos traumas → no auténtico
        if fragmento.tono_emocional > 0.5 && sub.traumas.len() > 3 {
            return false;
        }
        true
    }

    /// Encuentra un hilo conductor entre fragmentos (el tema más frecuente).
    fn encontrar_hilo_conductor(&self, fragmentos: &[FragmentoCandidato]) -> String {
        let mut fuentes: HashMap<&str, u32> = HashMap::new();
        for f in fragmentos {
            *fuentes.entry(&f.fuente).or_insert(0) += 1;
        }
        fuentes
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(fuente, _)| fuente.to_string())
            .unwrap_or_else(|| "reflexión".to_string())
    }
}
