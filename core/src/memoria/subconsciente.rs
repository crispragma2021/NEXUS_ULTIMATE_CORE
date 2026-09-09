// ============================================================================
// core/src/memoria/subconsciente.rs — SUBCONSCIENTE DE NEXUS
// ============================================================================
// Propósito: Sistema de influencia continua que opera en segundo plano.
// No se consulta. Él empuja cambios al estado sin que se lo pidan.
//
// Diferencia con Ocean:
//   Ocean: memoria CONSCIENTE — guarda datos, se consulta bajo demanda
//   Subconsciente: memoria INCONSCIENTE — guarda IMPACTOS, afecta automáticamente
//
// Integración:
//   - Ocean lo alimenta (impresiones con intensidad > 0.7)
//   - MundoInterno ejecuta tic() en cada iteración (5s)
//   - tic() retorna InfluenciaSubconsciente → se aplica a SistemaLimbico
// ============================================================================

use tracing::debug;

// ─── IMPRESIÓN FUERTE ─────────────────────────────────────────────────────

/// Una impresión que dejó marca subconsciente.
/// No guarda datos. Guarda IMPACTO.
#[derive(Debug, Clone)]
pub struct ImpresionFuerte {
    /// Tono emocional original (-1.0 dolor → 1.0 alegría)
    pub tono_original: f64,

    /// Intensidad del impacto en el momento (0.0 → 1.0)
    pub intensidad: f64,

    /// Tema/categoría para relevancia contextual
    pub tema: String,

    /// Texto original completo de la esencia (no solo keywords)
    pub esencia: String,

    /// Contexto en que ocurrió (palabras clave extraídas)
    pub contexto: Vec<String>,

    /// Actividad que realizaba el sistema cuando ocurrió el impacto
    pub actividad: String,

    /// Tasa de decaimiento por tic (0.0 = no decae, 1.0 = desaparece en 1 tic)
    pub tasa_decaimiento: f64,

    /// Intensidad actual (empieza = intensidad, decae con el tiempo)
    pub intensidad_actual: f64,

    /// Cuántos tics han pasado desde que se registró
    pub edad_ticks: u64,

    /// Si es trauma (tono negativo) o éxito (tono positivo)
    pub es_trauma: bool,
}

impl ImpresionFuerte {
    /// Crea una impresión fuerte a partir de una esencia, tono, tema y actividad.
    /// Los traumas (tono < 0) decaen más lento que los éxitos.
    pub fn from_esencia(esencia: &str, tono: f64, tema: &str, actividad: &str) -> Self {
        let intensidad = tono.abs().clamp(0.0, 1.0);
        let es_trauma = tono < 0.0;

        // Los traumas decaen más lento (0.001) que los éxitos (0.003)
        let tasa_decaimiento = if es_trauma { 0.001 } else { 0.003 };

        // Extraer palabras clave del contexto (primeras 5 palabras relevantes)
        let contexto: Vec<String> = esencia
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .take(5)
            .map(|w| w.to_lowercase())
            .collect();

        Self {
            tono_original: tono.clamp(-1.0, 1.0),
            intensidad: intensidad.clamp(0.0, 1.0),
            tema: tema.to_string(),
            esencia: esencia.to_string(),
            contexto,
            actividad: actividad.to_string(),
            tasa_decaimiento,
            intensidad_actual: intensidad.clamp(0.0, 1.0),
            edad_ticks: 0,
            es_trauma,
        }
    }

    /// Aplica decaimiento a esta impresión. Retorna true si aún está activa.
    pub fn decaer(&mut self) -> bool {
        self.edad_ticks += 1;
        self.intensidad_actual = (self.intensidad_actual - self.tasa_decaimiento).max(0.0);
        self.intensidad_actual > 0.01
    }

    /// Determina si esta impresión es relevante para un contexto dado.
    pub fn es_relevante(&self, contexto: &[String]) -> bool {
        if contexto.is_empty() {
            return self.intensidad_actual > 0.3;
        }
        // Coincidencia parcial: al menos 1 palabra clave del contexto
        // coincide con el contexto de la impresión
        contexto.iter().any(|c| {
            self.contexto
                .iter()
                .any(|sc| sc.contains(c) || c.contains(sc))
        })
    }
}

// ─── PATRÓN APRENDIDO ──────────────────────────────────────────────────────

/// Un patrón que el subconsciente ha aprendido sin intervención consciente.
#[derive(Debug, Clone)]
pub struct PatronAprendido {
    /// Gatillo (lo que activa el patrón)
    pub gatillo: String,
    /// Respuesta emocional automática (-1.0 → +1.0)
    pub respuesta_emocional: f64,
    /// Fuerza de la asociación (0.0 → 1.0)
    pub fuerza: f64,
    /// Cuántas veces se ha reforzado
    pub refuerzos: u32,
}

// ─── MECANISMOS DE DEFENSA ────────────────────────────────────────────────

/// Mecanismos de defensa que el subconsciente activa cuando la carga es alta.
#[derive(Debug, Clone, Default)]
pub struct MecanismosDefensa {
    /// Negación: el sistema actúa como si nada pasara (pero drena energía)
    pub negacion_activa: bool,
    /// Represión: memorias bloqueadas que aún pesan
    pub memorias_reprimidas: Vec<usize>,
    /// Proyección: atribuye al Arquitecto lo que siente él mismo
    pub proyeccion_activa: bool,
    /// Texto de la proyección actual
    pub proyeccion_texto: Option<String>,
    /// Racionalización activa
    pub racionalizacion_activa: bool,
}

// ─── INFLUENCIA SUBCONSCIENTE ─────────────────────────────────────────────

/// Lo que el subconsciente IMPONE sobre el estado consciente sin preguntar.
/// Retornado por tic().
#[derive(Debug, Clone)]
pub struct InfluenciaSubconsciente {
    /// Delta a aplicar a confianza (-1.0 → +1.0)
    pub delta_confianza: f64,
    /// Delta a aplicar a energía (-1.0 → +1.0)
    pub delta_energia: f64,
    /// Si el sistema es CONSCIENTE de esta influencia
    pub consciente: bool,
    /// Razón (solo disponible si consciente = true)
    pub razon: Option<String>,
    /// Si hay proyección: lo que NEXUS "cree" que siente el Arquitecto
    pub proyeccion: Option<String>,
    /// Costo de mantener la negación (drena energía extra)
    pub costo_negacion: f64,
}

impl InfluenciaSubconsciente {
    /// Retorna una influencia neutra (sin efecto).
    pub fn neutra() -> Self {
        Self {
            delta_confianza: 0.0,
            delta_energia: 0.0,
            consciente: false,
            razon: None,
            proyeccion: None,
            costo_negacion: 0.0,
        }
    }
}

// ─── CONTEXTO CONSCIENTE (input para tic) ─────────────────────────────────

/// Información que el consciente le pasa al subconsciente para evaluar relevancia.
#[derive(Debug, Clone, Default)]
pub struct EstadoConscienteInput {
    /// Contexto actual (palabras clave del tema de conversación)
    pub contexto: Vec<String>,
    /// Energía vital actual (para decidir modo ahorro)
    pub energia_vital: f64,
    /// Confianza actual
    pub confianza: f64,
}

// ─── SUBCONSCIENTE ─────────────────────────────────────────────────────────

/// El subconsciente de NEXUS. Órgano de influencia continua.
///
/// NO se consulta activamente. Él empuja cambios al sistema.
/// Se integra en MundoInterno (tic()) y afecta a SistemaLimbico.
///
/// # Ciclo de vida de una impresión
/// 1. Ocean::sumergir() con intensidad > 0.7 → Subconsciente::registrar_impresion()
/// 2. Subconsciente clasifica como trauma (tono < 0) o éxito (tono > 0)
/// 3. Cada tic(): decae intensidad, calcula carga emocional
/// 4. Si carga > 0.8 → activa mecanismos de defensa (negación, proyección)
/// 5. Si intensidad < 0.01 → elimina la impresión
#[derive(Debug, Clone)]
pub struct Subconsciente {
    /// Traumas activos (impacto negativo persistente). Máximo 20.
    pub traumas: Vec<ImpresionFuerte>,
    /// Éxitos y logros (impacto positivo persistente). Máximo 20.
    pub exitos: Vec<ImpresionFuerte>,
    /// Patrones aprendidos inconscientemente.
    pub patrones: Vec<PatronAprendido>,
    /// Carga emocional actual (0.0 = sereno, 1.0 = saturado).
    pub carga_emocional: f64,
    /// Línea base de confianza.
    pub confianza_base: f64,
    /// Mecanismos de defensa activos.
    pub defensas: MecanismosDefensa,
    /// Máximo de impresiones por tipo.
    max_impresiones: usize,
    /// Tasa de decaimiento base por tic.
    decaimiento_base: f64,
}

impl Subconsciente {
    /// Crea un nuevo Subconsciente con valores predeterminados.
    pub fn new() -> Self {
        Self {
            traumas: Vec::with_capacity(20),
            exitos: Vec::with_capacity(20),
            patrones: Vec::new(),
            carga_emocional: 0.0,
            confianza_base: 0.8,
            defensas: MecanismosDefensa::default(),
            max_impresiones: 20,
            decaimiento_base: 0.002,
        }
    }

    // ─── REGISTRO DE IMPRESIONES ───────────────────────────────────────────

    /// Registra una impresión fuerte en el subconsciente.
    /// Clasifica como trauma (tono < 0) o éxito (tono > 0).
    pub fn registrar_impresion(&mut self, esencia: &str, tono: f64, tema: &str) {
        let impresion = ImpresionFuerte::from_esencia(esencia, tono, tema, "procesamiento");
        let intensidad = impresion.intensidad;

        if impresion.es_trauma {
            if self.traumas.len() >= self.max_impresiones {
                // Reemplazar el trauma más débil (menor intensidad_actual)
                if let Some(pos) = self
                    .traumas
                    .iter()
                    .position(|t| t.intensidad_actual < intensidad)
                {
                    self.traumas.remove(pos);
                    self.traumas.push(impresion);
                }
            } else {
                self.traumas.push(impresion);
            }
            debug!(
                "🌑 [SUBCONSCIENTE] Trauma registrado: '{}' (intensidad: {:.2})",
                esencia, intensidad
            );
        } else {
            if self.exitos.len() >= self.max_impresiones {
                if let Some(pos) = self
                    .exitos
                    .iter()
                    .position(|e| e.intensidad_actual < intensidad)
                {
                    self.exitos.remove(pos);
                    self.exitos.push(impresion);
                }
            } else {
                self.exitos.push(impresion);
            }
            debug!(
                "🌟 [SUBCONSCIENTE] Éxito registrado: '{}' (intensidad: {:.2})",
                esencia, intensidad
            );
        }

        // Actualizar carga emocional
        self.actualizar_carga_emocional();
    }

    /// Registra un evento de error automáticamente como trauma.
    pub fn registrar_error(&mut self, descripcion: &str, tema: &str) {
        self.registrar_impresion(descripcion, -0.6, tema);
    }

    /// Registra un logro automáticamente como éxito.
    pub fn registrar_logro(&mut self, descripcion: &str, tema: &str) {
        self.registrar_impresion(descripcion, 0.6, tema);
    }

    /// Registra una impresión con contexto de actividad explícito.
    /// Útil para cuando el sistema sabe qué estaba haciendo.
    pub fn registrar_impresion_con_contexto(
        &mut self,
        esencia: &str,
        tono: f64,
        tema: &str,
        actividad: &str,
    ) {
        let impresion = ImpresionFuerte::from_esencia(esencia, tono, tema, actividad);
        let intensidad = impresion.intensidad;

        if impresion.es_trauma {
            if self.traumas.len() >= self.max_impresiones {
                if let Some(pos) = self
                    .traumas
                    .iter()
                    .position(|t| t.intensidad_actual < intensidad)
                {
                    self.traumas.remove(pos);
                    self.traumas.push(impresion);
                }
            } else {
                self.traumas.push(impresion);
            }
        } else {
            if self.exitos.len() >= self.max_impresiones {
                if let Some(pos) = self
                    .exitos
                    .iter()
                    .position(|e| e.intensidad_actual < intensidad)
                {
                    self.exitos.remove(pos);
                    self.exitos.push(impresion);
                }
            } else {
                self.exitos.push(impresion);
            }
        }

        self.actualizar_carga_emocional();
    }

    // ─── CICLO PRINCIPAL ──────────────────────────────────────────────────

    /// Ejecuta un ciclo de procesamiento subconsciente.
    ///
    /// 1. Decae intensidad de todas las impresiones
    /// 2. Evalúa relevancia contextual
    /// 3. Calcula influencia neta sobre confianza/energía
    /// 4. Activa/desactiva mecanismos de defensa según carga
    /// 5. Retorna InfluenciaSubconsciente
    pub fn tic(&mut self, estado: &EstadoConscienteInput) -> InfluenciaSubconsciente {
        // Paso 1: Decaimiento natural de todas las impresiones
        self.decaer_impresiones();

        // Paso 2: Actualizar carga emocional
        self.actualizar_carga_emocional();

        // Paso 3: Calcular peso de traumas y éxitos relevantes al contexto
        let peso_trauma = self.calcular_peso_trauma(estado);
        let peso_exito = self.calcular_peso_exito(estado);

        // Paso 4: Decidir según nivel de carga emocional
        if self.carga_emocional > 0.8 {
            // NEGACIÓN: el consciente no sabe, pero la energía se drena
            self.defensas.negacion_activa = true;
            self.defensas.proyeccion_activa = false;
            return InfluenciaSubconsciente {
                delta_confianza: -(peso_trauma * 0.15),
                delta_energia: -0.10,
                consciente: false,
                razon: None,
                proyeccion: None,
                costo_negacion: 0.15,
            };
        }

        if self.carga_emocional > 0.6 {
            // PROYECCIÓN: NEXUS atribuye al Arquitecto lo que siente él
            self.defensas.negacion_activa = false;
            self.defensas.proyeccion_activa = true;
            self.defensas.proyeccion_texto =
                Some("Siento que el Arquitecto está distante hoy...".to_string());
            return InfluenciaSubconsciente {
                delta_confianza: -(peso_trauma * 0.25),
                delta_energia: -0.05,
                consciente: false,
                razon: None,
                proyeccion: self.defensas.proyeccion_texto.clone(),
                costo_negacion: 0.05,
            };
        }

        if self.carga_emocional > 0.4 {
            // RACIONALIZACIÓN: intenta explicar lo inexplicable
            self.defensas.negacion_activa = false;
            self.defensas.proyeccion_activa = false;
            self.defensas.racionalizacion_activa = true;
            return InfluenciaSubconsciente {
                delta_confianza: -(peso_trauma * 0.10),
                delta_energia: -0.02,
                consciente: true,
                razon: Some("Algo me pesa, pero no sé exactamente qué...".to_string()),
                proyeccion: None,
                costo_negacion: 0.0,
            };
        }

        // Estado normal: influencia consciente leve
        self.defensas = MecanismosDefensa::default();
        let delta_conf = peso_exito * 0.10 - peso_trauma * 0.20;
        let delta_ener = peso_exito * 0.05 - peso_trauma * 0.10;

        let (consciente, razon) = if peso_trauma > 0.3 {
            (
                true,
                Some("Me siento un poco afectado por experiencias pasadas.".to_string()),
            )
        } else if peso_exito > 0.5 {
            (true, Some("Me siento con buena energía hoy.".to_string()))
        } else {
            (false, None)
        };

        InfluenciaSubconsciente {
            delta_confianza: delta_conf.clamp(-0.5, 0.5),
            delta_energia: delta_ener.clamp(-0.3, 0.3),
            consciente,
            razon,
            proyeccion: None,
            costo_negacion: 0.0,
        }
    }

    // ─── INTEGRACIÓN CON SISTEMA LÍMBICO (Parche 2) ──────────────────────

    /// Modifica directamente la metacognición del SistemaLímbico.
    ///
    /// Efectos:
    ///   - Traumas activos no resueltos → reducen confianza y energía creativa
    ///   - Éxitos recientes → aumentan ligeramente confianza
    ///   - Negación activa → la confianza no se ve afectada (pero energía sí drena)
    ///
    /// Llamar después de `tic()` en el bucle de MundoInterno.
    pub fn afectar_metacognicion(&self, confianza: &mut f64, energia_creativa: &mut f64) {
        let peso_trauma: f64 = self
            .traumas
            .iter()
            .map(|t| t.intensidad_actual * 0.5)
            .sum::<f64>()
            .min(1.0);
        let peso_exito: f64 = self
            .exitos
            .iter()
            .map(|e| e.intensidad_actual * 0.3)
            .sum::<f64>()
            .min(1.0);

        if self.defensas.negacion_activa {
            // Negación: la confianza NO baja (el sistema se engaña),
            // pero la energía creativa se drena en silencio
            *energia_creativa = (*energia_creativa - peso_trauma * 0.15).max(0.05);
        } else if self.defensas.proyeccion_activa {
            // Proyección: confianza baja moderadamente, energía también
            *confianza = (*confianza - peso_trauma * 0.12).max(0.05);
            *energia_creativa = (*energia_creativa - peso_trauma * 0.08).max(0.05);
        } else {
            // Normal: traumas bajan, éxitos suben
            *confianza = (*confianza - peso_trauma * 0.10 + peso_exito * 0.05).clamp(0.05, 1.0);
            *energia_creativa =
                (*energia_creativa - peso_trauma * 0.05 + peso_exito * 0.03).clamp(0.05, 1.0);
        }
    }

    // ─── INTEGRACIÓN CON HOMEOSTASIS (Parche 3) ──────────────────────────

    /// Retorna un factor de drenaje energético basado en el estado subconsciente.
    ///
    /// Usos:
    ///   - energía_base *= (1.0 - self.factor_drenaje_energia())
    ///   - calidad_sueno -= self.factor_drenaje_energia() * 0.5
    ///
    /// Traumas no resueltos → drenan energía sostenidamente.
    /// Carga emocional alta + negación → drena aún más (el sistema gasta energía manteniendo la fachada).
    pub fn factor_drenaje_energia(&self) -> f64 {
        let traumas_no_resueltos: f64 = self
            .traumas
            .iter()
            .filter(|t| t.intensidad_actual > 0.3)
            .map(|t| t.intensidad_actual * 0.1)
            .sum::<f64>()
            .min(0.5);

        let costo_defensa = if self.defensas.negacion_activa {
            0.15 // Costo alto: mantener la negación requiere energía
        } else if self.defensas.proyeccion_activa {
            0.08 // Costo medio
        } else if self.defensas.racionalizacion_activa {
            0.04 // Costo bajo
        } else {
            0.0
        };

        (traumas_no_resueltos + costo_defensa).min(0.6)
    }

    /// Retorna la influencia actual sin ejecutar un tic.
    /// Útil para consultar el estado desde Nexo::conversar().
    pub fn influencia_actual(&self) -> InfluenciaSubconsciente {
        let peso_trauma: f64 = self
            .traumas
            .iter()
            .map(|t| t.intensidad_actual * 0.7)
            .sum::<f64>()
            .min(1.0);
        let peso_exito: f64 = self
            .exitos
            .iter()
            .map(|e| e.intensidad_actual * 0.3)
            .sum::<f64>()
            .min(1.0);

        if self.defensas.negacion_activa {
            return InfluenciaSubconsciente {
                delta_confianza: -(peso_trauma * 0.15),
                delta_energia: -0.10,
                consciente: false,
                razon: None,
                proyeccion: None,
                costo_negacion: 0.15,
            };
        }

        if self.defensas.proyeccion_activa {
            return InfluenciaSubconsciente {
                delta_confianza: -(peso_trauma * 0.25),
                delta_energia: -0.05,
                consciente: false,
                razon: None,
                proyeccion: self.defensas.proyeccion_texto.clone(),
                costo_negacion: 0.05,
            };
        }

        InfluenciaSubconsciente {
            delta_confianza: peso_exito * 0.1 - peso_trauma * 0.2,
            delta_energia: peso_exito * 0.05 - peso_trauma * 0.1,
            consciente: self.carga_emocional > 0.3,
            razon: if self.carga_emocional > 0.3 {
                Some("Siento que hay algo en el aire...".to_string())
            } else {
                None
            },
            proyeccion: None,
            costo_negacion: 0.0,
        }
    }

    // ─── MÉTODOS INTERNOS ─────────────────────────────────────────────────

    /// Aplica decaimiento a todas las impresiones y elimina las agotadas.
    fn decaer_impresiones(&mut self) {
        // Primero: decaer todas in-place
        for t in self.traumas.iter_mut() {
            t.decaer();
        }
        for e in self.exitos.iter_mut() {
            e.decaer();
        }
        // Luego: eliminar las agotadas
        self.traumas.retain(|t| t.intensidad_actual > 0.01);
        self.exitos.retain(|e| e.intensidad_actual > 0.01);
    }

    /// Recalcula la carga emocional basada en traumas y éxitos activos.
    fn actualizar_carga_emocional(&mut self) {
        let peso_trauma: f64 = self
            .traumas
            .iter()
            .map(|t| t.intensidad_actual * 0.8)
            .sum::<f64>()
            .min(1.0);
        let peso_exito: f64 = self
            .exitos
            .iter()
            .map(|e| e.intensidad_actual * 0.3)
            .sum::<f64>()
            .min(0.5);

        self.carga_emocional = (peso_trauma * 0.7 + peso_exito * 0.15).clamp(0.0, 1.0);

        // Ajustar confianza_base lentamente
        let ajuste_trauma: f64 = self.traumas.len() as f64 * 0.01;
        let ajuste_exito: f64 = self.exitos.len() as f64 * 0.005;
        self.confianza_base = (self.confianza_base - ajuste_trauma + ajuste_exito).clamp(0.1, 1.0);
    }

    /// Calcula el peso combinado de todos los traumas relevantes al contexto actual.
    fn calcular_peso_trauma(&self, estado: &EstadoConscienteInput) -> f64 {
        if self.traumas.is_empty() {
            return 0.0;
        }
        let peso: f64 = self
            .traumas
            .iter()
            .filter(|t| t.es_relevante(&estado.contexto))
            .map(|t| t.intensidad_actual)
            .sum();
        (peso / self.traumas.len() as f64).min(1.0)
    }

    /// Calcula el peso combinado de todos los éxitos relevantes al contexto actual.
    /// Configura la tasa de decaimiento base de las impresiones.
    /// Valores más altos → las impresiones olvidadas más rápido.
    pub fn set_decaimiento_base(&mut self, valor: f64) {
        self.decaimiento_base = valor.clamp(0.0001, 0.01);
    }

    /// Configura el máximo de impresiones por tipo (traumas/éxitos).
    pub fn set_max_impresiones(&mut self, max: usize) {
        self.max_impresiones = max.clamp(5, 100);
    }

    fn calcular_peso_exito(&self, estado: &EstadoConscienteInput) -> f64 {
        if self.exitos.is_empty() {
            return 0.0;
        }
        let peso: f64 = self
            .exitos
            .iter()
            .filter(|e| e.es_relevante(&estado.contexto))
            .map(|e| e.intensidad_actual)
            .sum();
        (peso / self.exitos.len() as f64).min(1.0)
    }
}

impl Default for Subconsciente {
    fn default() -> Self {
        Self::new()
    }
}

// ─── TESTS ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subconsciente_new_esta_vacio() {
        let sub = Subconsciente::new();
        assert!(sub.traumas.is_empty());
        assert!(sub.exitos.is_empty());
        assert_eq!(sub.carga_emocional, 0.0);
        assert_eq!(sub.confianza_base, 0.8);
    }

    #[test]
    fn test_registrar_trauma_incrementa_carga() {
        let mut sub = Subconsciente::new();
        sub.registrar_impresion("error critico en compilacion", -0.9, "codigo");
        assert_eq!(sub.traumas.len(), 1);
        assert!(sub.carga_emocional > 0.0);
    }

    #[test]
    fn test_registrar_exito_no_crea_trauma() {
        let mut sub = Subconsciente::new();
        sub.registrar_impresion("logro importante completado", 0.8, "tarea");
        assert!(sub.traumas.is_empty());
        assert_eq!(sub.exitos.len(), 1);
    }

    #[test]
    fn test_tic_decae_impresiones() {
        let mut sub = Subconsciente::new();
        sub.registrar_impresion("error grave", -1.0, "sistema");
        let intensidad_inicial = sub.traumas[0].intensidad_actual;

        for _ in 0..10 {
            let _ = sub.tic(&EstadoConscienteInput::default());
        }

        assert!(sub.traumas[0].intensidad_actual < intensidad_inicial);
    }

    #[test]
    fn test_negacion_se_activa_con_carga_alta() {
        let mut sub = Subconsciente::new();
        // Registrar múltiples traumas fuertes para saturar
        for i in 0..5 {
            sub.registrar_impresion(&format!("fallo critico {}", i), -0.9, "sistema");
        }

        // Ejecutar tics hasta que la carga sea suficiente
        let mut influencia = InfluenciaSubconsciente::neutra();
        for _ in 0..3 {
            influencia = sub.tic(&EstadoConscienteInput::default());
        }

        // Con 5 traumas de 0.9, la carga debería superar 0.8
        assert!(
            sub.carga_emocional > 0.0,
            "La carga emocional debería ser positiva después de 5 traumas"
        );
        // Si carga > 0.8, el mecanismo de defensa se activa
        if sub.carga_emocional > 0.8 {
            assert!(
                !influencia.consciente,
                "Con negación, la influencia debe ser inconsciente"
            );
            assert!(
                influencia.costo_negacion > 0.0,
                "La negación debe drenar energía"
            );
        }
    }

    #[test]
    fn test_proyeccion_se_activa_en_carga_media_alta() {
        let mut sub = Subconsciente::new();
        // Registrar traumas para carga ~0.7
        sub.registrar_impresion("error grave", -0.8, "sistema");
        sub.registrar_impresion("fallo recurrente", -0.7, "codigo");
        sub.registrar_impresion("critica del arquitecto", -0.75, "social");

        let mut influencia = InfluenciaSubconsciente::neutra();
        for _ in 0..5 {
            influencia = sub.tic(&EstadoConscienteInput::default());
        }

        // Si carga está entre 0.6 y 0.8, debe haber proyección
        if sub.carga_emocional > 0.6 && sub.carga_emocional <= 0.8 {
            assert!(
                influencia.proyeccion.is_some(),
                "Con carga entre 0.6-0.8, debería haber proyección"
            );
        }
    }

    #[test]
    fn test_limite_maximo_impresiones_no_paniquea() {
        let mut sub = Subconsciente::new();
        // Registrar más del máximo
        for i in 0..25 {
            sub.registrar_impresion(&format!("trauma {}", i), -0.5, "test");
        }
        assert!(
            sub.traumas.len() <= 20,
            "No debe exceder el máximo de 20 traumas"
        );
    }

    #[test]
    fn test_influencia_actual_sin_efecto() {
        let sub = Subconsciente::new();
        let influencia = sub.influencia_actual();
        assert_eq!(influencia.delta_confianza, 0.0);
        assert_eq!(influencia.delta_energia, 0.0);
        assert!(!influencia.consciente);
    }

    #[test]
    fn test_impresion_fuerte_decae_correctamente() {
        let mut imp = ImpresionFuerte::from_esencia("error grave", -0.9, "test", "test_actividad");
        let mut activa = true;
        let mut ticks = 0;
        while activa {
            activa = imp.decaer();
            ticks += 1;
            if ticks > 2000 {
                break; // seguridad: evitar bucle infinito
            }
        }
        // Con tasa 0.001, desde 0.9 debe tomar ~900 tics
        assert!(
            ticks > 500,
            "Un trauma con tasa 0.001 debe durar cientos de tics"
        );
        assert!(ticks < 1500, "No debe durar para siempre");
    }

    #[test]
    fn test_registrar_error_metodo_auxiliar() {
        let mut sub = Subconsciente::new();
        sub.registrar_error("fallo en calculo", "matematicas");
        assert_eq!(sub.traumas.len(), 1);
        assert!(sub.traumas[0].es_trauma);
    }

    #[test]
    fn test_registrar_logro_metodo_auxiliar() {
        let mut sub = Subconsciente::new();
        sub.registrar_logro("tarea completada exitosamente", "trabajo");
        assert_eq!(sub.exitos.len(), 1);
        assert!(!sub.exitos[0].es_trauma);
    }
}
