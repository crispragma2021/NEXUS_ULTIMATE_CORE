// ============================================================================
// APRENDIZAJE RECURSIVO - Auto-observación y auto-mejora del aprendizaje
// ============================================================================
// NEXUS se observa a sí mismo para aprender cómo aprende mejor.
//
// Sub-órganos:
//   1. ObservadorRecursivo — recolecta métricas de eficacia por sistema
//   2. MotorAutoAjuste — modula parámetros numéricos dentro de rangos seguros
//   3. GuardianSeguridad — garantiza que ningún ajuste viole Pilar 13
//   4. GeneradorPropuestas — formula cambios estructurales para aprobación
// ============================================================================

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

// ─── CONSTANTES ─────────────────────────────────────────────────────────────

/// Cada cuántos eventos se completa una ventana de observación
const VENTANA_EVENTOS: usize = 50;

/// Máximo de métricas históricas por sistema
const MAX_HISTORIAL_POR_SISTEMA: usize = 500;

/// Segundos de congelación tras rollback automático
const CONGELACION_DEFAULT_SEGUNDOS: u64 = 86_400; // 24 horas

/// Fallos consecutivos para disparar rollback automático
const UMBRAL_ROLLBACK: u32 = 3;

/// Ciclos que un parámetro debe estar en límite para generar propuesta
const CICLOS_LIMITE_PARA_PROPUESTA: u32 = 5;

// ─── ENUMS ──────────────────────────────────────────────────────────────────

/// Catálogo de sistemas de aprendizaje observables por NEXUS
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SistemaAprendizaje {
    Subconsciente,
    JuicioSoberano,
    MotorSynapse,
    Metacognicion,
    VoluntadPropia,
    SistemaLimbico,
    Ocean,
    Chunker,
    GeneradorOrganico,
    Defensa,
    Creatividad,
}

impl SistemaAprendizaje {
    pub fn nombre(&self) -> &'static str {
        match self {
            Self::Subconsciente => "Subconsciente",
            Self::JuicioSoberano => "JuicioSoberano",
            Self::MotorSynapse => "MotorSynapse",
            Self::Metacognicion => "Metacognicion",
            Self::VoluntadPropia => "VoluntadPropia",
            Self::SistemaLimbico => "SistemaLimbico",
            Self::Ocean => "Ocean",
            Self::Chunker => "Chunker",
            Self::GeneradorOrganico => "GeneradorOrganico",
            Self::Defensa => "Defensa",
            Self::Creatividad => "Creatividad",
        }
    }
}

/// Estado de una propuesta estructural
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EstadoPropuesta {
    Pendiente,
    Aprobada,
    Rechazada,
    Implementada,
    Revertida,
}

// ─── ESTRUCTURAS DE DATOS ───────────────────────────────────────────────────

/// Una medición puntual de eficacia de un sistema de aprendizaje
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricaEficacia {
    /// Sistema observado
    pub sistema: SistemaAprendizaje,
    /// Nombre de la métrica (ej: "precisión_confianza")
    pub metrica: String,
    /// Valor normalizado 0.0 (pésimo) → 1.0 (óptimo)
    pub valor: f64,
    /// Timestamp Unix
    pub timestamp: u64,
    /// Contexto adicional opcional
    pub contexto: Option<String>,
}

/// Registro de cada auto-ajuste para auditoría y rollback
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AjusteRealizado {
    /// Identificador único
    pub id: u64,
    /// Nombre del parámetro modificado
    pub parametro: String,
    /// Sistema al que pertenece
    pub sistema: SistemaAprendizaje,
    /// Valor anterior
    pub valor_anterior: f64,
    /// Valor nuevo
    pub valor_nuevo: f64,
    /// Razón del ajuste
    pub razon: String,
    /// Métrica antes del ajuste
    pub metrica_antes: f64,
    /// Métrica después (None si aún no se midió)
    pub metrica_despues: Option<f64>,
    /// Timestamp Unix
    pub timestamp: u64,
    /// Si fue revertido
    pub revertido: bool,
}

/// Propuesta de cambio estructural que requiere aprobación del Arquitecto
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PropuestaEstructural {
    /// Identificador único
    pub id: u64,
    /// Título descriptivo
    pub titulo: String,
    /// Sistema que se modificaría
    pub sistema: SistemaAprendizaje,
    /// Descripción del cambio
    pub descripcion: String,
    /// Justificación basada en métricas
    pub justificacion: String,
    /// Riesgo estimado (0.0 → 1.0)
    pub riesgo_estimado: f64,
    /// Beneficio esperado (0.0 → 1.0)
    pub beneficio_esperado: f64,
    /// Alternativas consideradas
    pub alternativas: Vec<String>,
    /// Estado actual
    pub estado: EstadoPropuesta,
    /// Timestamp Unix
    pub timestamp: u64,
}

/// Registro central de parámetros numéricos que NEXUS puede auto-ajustar
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParametrosAprendizaje {
    // ── Subconsciente ──
    pub decaimiento_base: f64,
    pub max_impresiones: f64,
    pub umbral_negacion: f64,
    pub umbral_proyeccion: f64,
    // ── Synapse ──
    pub factor_propagacion: f64,
    pub factor_decaimiento_synapse: f64,
    pub umbral_expresion: f64,
    // ── VoluntadPropia ──
    pub nivel_curiosidad: f64,
    pub proactividad: f64,
    // ── Homeostasis ──
    pub metabolismo_base: f64,
    pub tasa_recuperacion: f64,
    // ── Metacognicion ──
    pub peso_similitud: f64,
    pub peso_coherencia: f64,
    pub peso_recencia: f64,
    // ── Chunker ──
    pub max_tokens_chunk: f64,
    pub overlap_tokens: f64,
}

impl Default for ParametrosAprendizaje {
    fn default() -> Self {
        Self {
            decaimiento_base: 0.002,
            max_impresiones: 20.0,
            umbral_negacion: 0.8,
            umbral_proyeccion: 0.6,
            factor_propagacion: 0.15,
            factor_decaimiento_synapse: 0.92,
            umbral_expresion: 0.6,
            nivel_curiosidad: 0.7,
            proactividad: 0.6,
            metabolismo_base: 0.002,
            tasa_recuperacion: 0.001,
            peso_similitud: 0.35,
            peso_coherencia: 0.25,
            peso_recencia: 0.10,
            max_tokens_chunk: 512.0,
            overlap_tokens: 50.0,
        }
    }
}

/// Define el rango seguro y el paso de ajuste para un parámetro
#[derive(Debug, Clone)]
pub struct RangoParametro {
    pub nombre: &'static str,
    pub sistema: SistemaAprendizaje,
    pub minimo: f64,
    pub maximo: f64,
    pub paso: f64,
    pub metrica_asociada: &'static str,
}

impl RangoParametro {
    pub fn todos() -> Vec<Self> {
        vec![
            Self {
                nombre: "decaimiento_base",
                sistema: SistemaAprendizaje::Subconsciente,
                minimo: 0.001,
                maximo: 0.005,
                paso: 0.0002,
                metrica_asociada: "eficiencia_memoria",
            },
            Self {
                nombre: "max_impresiones",
                sistema: SistemaAprendizaje::Subconsciente,
                minimo: 10.0,
                maximo: 50.0,
                paso: 2.0,
                metrica_asociada: "eficiencia_memoria",
            },
            Self {
                nombre: "umbral_negacion",
                sistema: SistemaAprendizaje::Subconsciente,
                minimo: 0.6,
                maximo: 0.95,
                paso: 0.02,
                metrica_asociada: "precision_proyeccion",
            },
            Self {
                nombre: "umbral_proyeccion",
                sistema: SistemaAprendizaje::Subconsciente,
                minimo: 0.4,
                maximo: 0.8,
                paso: 0.02,
                metrica_asociada: "precision_proyeccion",
            },
            Self {
                nombre: "factor_propagacion",
                sistema: SistemaAprendizaje::MotorSynapse,
                minimo: 0.01,
                maximo: 0.5,
                paso: 0.02,
                metrica_asociada: "coherencia_synapse",
            },
            Self {
                nombre: "factor_decaimiento_synapse",
                sistema: SistemaAprendizaje::MotorSynapse,
                minimo: 0.85,
                maximo: 0.99,
                paso: 0.01,
                metrica_asociada: "coherencia_synapse",
            },
            Self {
                nombre: "umbral_expresion",
                sistema: SistemaAprendizaje::MotorSynapse,
                minimo: 0.4,
                maximo: 0.8,
                paso: 0.02,
                metrica_asociada: "coherencia_synapse",
            },
            Self {
                nombre: "nivel_curiosidad",
                sistema: SistemaAprendizaje::VoluntadPropia,
                minimo: 0.3,
                maximo: 0.9,
                paso: 0.05,
                metrica_asociada: "tasa_iniciativas_utiles",
            },
            Self {
                nombre: "proactividad",
                sistema: SistemaAprendizaje::VoluntadPropia,
                minimo: 0.3,
                maximo: 0.9,
                paso: 0.05,
                metrica_asociada: "tasa_iniciativas_utiles",
            },
            Self {
                nombre: "metabolismo_base",
                sistema: SistemaAprendizaje::Subconsciente,
                minimo: 0.001,
                maximo: 0.005,
                paso: 0.0002,
                metrica_asociada: "eficiencia_memoria",
            },
            Self {
                nombre: "tasa_recuperacion",
                sistema: SistemaAprendizaje::Subconsciente,
                minimo: 0.0005,
                maximo: 0.003,
                paso: 0.0001,
                metrica_asociada: "eficiencia_memoria",
            },
            Self {
                nombre: "peso_similitud",
                sistema: SistemaAprendizaje::Metacognicion,
                minimo: 0.2,
                maximo: 0.5,
                paso: 0.02,
                metrica_asociada: "precision_confianza",
            },
            Self {
                nombre: "peso_coherencia",
                sistema: SistemaAprendizaje::Metacognicion,
                minimo: 0.15,
                maximo: 0.4,
                paso: 0.02,
                metrica_asociada: "precision_confianza",
            },
            Self {
                nombre: "peso_recencia",
                sistema: SistemaAprendizaje::Metacognicion,
                minimo: 0.05,
                maximo: 0.2,
                paso: 0.01,
                metrica_asociada: "precision_confianza",
            },
            Self {
                nombre: "max_tokens_chunk",
                sistema: SistemaAprendizaje::Chunker,
                minimo: 256.0,
                maximo: 4096.0,
                paso: 64.0,
                metrica_asociada: "utilidad_chunking",
            },
            Self {
                nombre: "overlap_tokens",
                sistema: SistemaAprendizaje::Chunker,
                minimo: 10.0,
                maximo: 512.0,
                paso: 10.0,
                metrica_asociada: "utilidad_chunking",
            },
        ]
    }
}

/// Obtiene el valor actual de un parámetro desde ParametrosAprendizaje
pub fn obtener_valor_parametro(parametros: &ParametrosAprendizaje, nombre: &str) -> Option<f64> {
    match nombre {
        "decaimiento_base" => Some(parametros.decaimiento_base),
        "max_impresiones" => Some(parametros.max_impresiones),
        "umbral_negacion" => Some(parametros.umbral_negacion),
        "umbral_proyeccion" => Some(parametros.umbral_proyeccion),
        "factor_propagacion" => Some(parametros.factor_propagacion),
        "factor_decaimiento_synapse" => Some(parametros.factor_decaimiento_synapse),
        "umbral_expresion" => Some(parametros.umbral_expresion),
        "nivel_curiosidad" => Some(parametros.nivel_curiosidad),
        "proactividad" => Some(parametros.proactividad),
        "metabolismo_base" => Some(parametros.metabolismo_base),
        "tasa_recuperacion" => Some(parametros.tasa_recuperacion),
        "peso_similitud" => Some(parametros.peso_similitud),
        "peso_coherencia" => Some(parametros.peso_coherencia),
        "peso_recencia" => Some(parametros.peso_recencia),
        "max_tokens_chunk" => Some(parametros.max_tokens_chunk),
        "overlap_tokens" => Some(parametros.overlap_tokens),
        _ => None,
    }
}

/// Aplica un nuevo valor a un parámetro en ParametrosAprendizaje
pub fn aplicar_valor_parametro(
    parametros: &mut ParametrosAprendizaje,
    nombre: &str,
    valor: f64,
) -> Option<f64> {
    let anterior = obtener_valor_parametro(parametros, nombre);
    match nombre {
        "decaimiento_base" => parametros.decaimiento_base = valor,
        "max_impresiones" => parametros.max_impresiones = valor,
        "umbral_negacion" => parametros.umbral_negacion = valor,
        "umbral_proyeccion" => parametros.umbral_proyeccion = valor,
        "factor_propagacion" => parametros.factor_propagacion = valor,
        "factor_decaimiento_synapse" => parametros.factor_decaimiento_synapse = valor,
        "umbral_expresion" => parametros.umbral_expresion = valor,
        "nivel_curiosidad" => parametros.nivel_curiosidad = valor,
        "proactividad" => parametros.proactividad = valor,
        "metabolismo_base" => parametros.metabolismo_base = valor,
        "tasa_recuperacion" => parametros.tasa_recuperacion = valor,
        "peso_similitud" => parametros.peso_similitud = valor,
        "peso_coherencia" => parametros.peso_coherencia = valor,
        "peso_recencia" => parametros.peso_recencia = valor,
        "max_tokens_chunk" => parametros.max_tokens_chunk = valor,
        "overlap_tokens" => parametros.overlap_tokens = valor,
        _ => return None,
    }
    anterior
}

// ─── GUARDIÁN DE SEGURIDAD ─────────────────────────────────────────────────

/// Guardián que valida cada ajuste contra las reglas del Pilar 13
#[derive(Debug, Clone)]
pub struct GuardianSeguridad {
    /// Parámetros congelados (nombre → timestamp de descongelación)
    pub congelados: HashMap<String, u64>,
    /// Parámetros forzados por el Arquitecto (nombre → valor fijo)
    pub forzados: HashMap<String, f64>,
    /// Contador de fallos consecutivos por parámetro
    fallos_consecutivos: HashMap<String, u32>,
    /// Segundos de congelación tras rollback
    duracion_congelacion: u64,
    /// Umbral de fallos para rollback
    umbral_rollback: u32,
}

impl Default for GuardianSeguridad {
    fn default() -> Self {
        Self {
            congelados: HashMap::new(),
            forzados: HashMap::new(),
            fallos_consecutivos: HashMap::new(),
            duracion_congelacion: CONGELACION_DEFAULT_SEGUNDOS,
            umbral_rollback: UMBRAL_ROLLBACK,
        }
    }
}

impl GuardianSeguridad {
    pub fn new() -> Self {
        Self::default()
    }

    /// Valida si un ajuste propuesto es seguro.
    /// Retorna Ok(()) si se permite, Err(razón) si se rechaza.
    pub fn validar_ajuste(
        &self,
        parametro: &str,
        valor_actual: f64,
        valor_propuesto: f64,
        rango: (f64, f64),
    ) -> Result<(), String> {
        // 1. Verificar si está congelado
        if let Some(&hasta) = self.congelados.get(parametro) {
            let ahora = ahora_segundos();
            if ahora < hasta {
                return Err(format!(
                    "Parámetro '{parametro}' congelado hasta {hasta} ({}s restantes)",
                    hasta.saturating_sub(ahora)
                ));
            }
        }

        // 2. Verificar si está forzado
        if let Some(&fijo) = self.forzados.get(parametro) {
            if (valor_propuesto - fijo).abs() > f64::EPSILON {
                return Err(format!(
                    "Parámetro '{parametro}' forzado a {fijo} por el Arquitecto"
                ));
            }
        }

        // 3. Verificar rango seguro
        if valor_propuesto < rango.0 || valor_propuesto > rango.1 {
            return Err(format!(
                "Valor {valor_propuesto:.4} fuera del rango seguro [{:.4}, {:.4}] para '{parametro}'",
                rango.0, rango.1
            ));
        }

        // 4. Si valor_actual ya es igual, no hay cambio real
        if (valor_propuesto - valor_actual).abs() < f64::EPSILON {
            return Err("Valor propuesto igual al actual, no hay cambio".to_string());
        }

        Ok(())
    }

    /// Registra un fallo y retorna true si se debe hacer rollback
    pub fn registrar_fallo(&mut self, parametro: &str) -> bool {
        let contador = self
            .fallos_consecutivos
            .entry(parametro.to_string())
            .or_insert(0);
        *contador += 1;
        if *contador >= self.umbral_rollback {
            let ahora = ahora_segundos();
            self.congelados
                .insert(parametro.to_string(), ahora + self.duracion_congelacion);
            self.fallos_consecutivos.remove(parametro);
            warn!("🛡️ [GUARDIÁN] Rollback automático para '{parametro}'. Congelado 24h.");
            true
        } else {
            false
        }
    }

    /// Registra un éxito, reseteando el contador de fallos
    pub fn registrar_exito(&mut self, parametro: &str) {
        self.fallos_consecutivos.remove(parametro);
    }

    /// Congela un parámetro manualmente (por el Arquitecto)
    pub fn congelar(&mut self, parametro: &str, segundos: u64) {
        let hasta = ahora_segundos() + segundos;
        self.congelados.insert(parametro.to_string(), hasta);
        info!("🛡️ [GUARDIÁN] Parámetro '{parametro}' congelado manualmente por {segundos}s");
    }

    /// Descongela un parámetro
    pub fn descongelar(&mut self, parametro: &str) {
        self.congelados.remove(parametro);
        self.fallos_consecutivos.remove(parametro);
    }

    /// Fuerza un valor fijo para un parámetro (solo el Arquitecto)
    pub fn forzar_valor(&mut self, parametro: &str, valor: f64) {
        self.forzados.insert(parametro.to_string(), valor);
        info!("🛡️ [GUARDIÁN] Parámetro '{parametro}' forzado a {valor} por el Arquitecto");
    }

    /// Limpia la restricción de valor forzado
    pub fn liberar_forzado(&mut self, parametro: &str) {
        self.forzados.remove(parametro);
    }

    /// Reporte de estado del guardián
    pub fn reporte(&self) -> String {
        let mut lineas = Vec::new();
        lineas.push("🛡️ [GUARDIÁN] Estado de seguridad:".to_string());
        for (param, hasta) in &self.congelados {
            let restante = hasta.saturating_sub(ahora_segundos());
            lineas.push(format!("  ❄️ {param}: congelado ({restante}s restantes)"));
        }
        for (param, valor) in &self.forzados {
            lineas.push(format!("  🔒 {param}: forzado a {valor}"));
        }
        if self.congelados.is_empty() && self.forzados.is_empty() {
            lineas.push("  ✅ Sin restricciones activas.".to_string());
        }
        lineas.join("\n")
    }
}

// ─── MOTOR DE AUTO-AJUSTE ──────────────────────────────────────────────────

/// Motor que evalúa y aplica ajustes de parámetros numéricos
#[derive(Debug, Clone, Default)]
pub struct MotorAutoAjuste {
    /// Última dirección de ajuste por parámetro (true = subir, false = bajar)
    ultima_direccion: HashMap<String, bool>,
    /// Último valor de métrica por parámetro
    ultima_metrica: HashMap<String, f64>,
    /// Contador de ciclos en límite por parámetro
    ciclos_en_limite: HashMap<String, u32>,
}

impl MotorAutoAjuste {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evalúa si un parámetro necesita ajuste basado en la métrica actual.
    /// Retorna Some(nuevo_valor) si se debe ajustar, None si no.
    pub fn evaluar_parametro(
        &mut self,
        parametro: &str,
        valor_actual: f64,
        rango: &RangoParametro,
        metrica_actual: f64,
    ) -> Option<f64> {
        let metrica_anterior = self
            .ultima_metrica
            .get(parametro)
            .copied()
            .unwrap_or(metrica_actual);

        let ajuste = if metrica_actual < 0.3 {
            // Degradación severa → mover en dirección opuesta
            let direccion_opuesta = !self
                .ultima_direccion
                .get(parametro)
                .copied()
                .unwrap_or(true);
            let paso = rango.paso;
            if direccion_opuesta {
                (valor_actual + paso).min(rango.maximo)
            } else {
                (valor_actual - paso).max(rango.minimo)
            }
        } else if metrica_actual < 0.5 {
            // Mediocre → explorar dirección aleatoria
            let subir = rand::random::<f64>() > 0.5;
            if subir {
                (valor_actual + rango.paso).min(rango.maximo)
            } else {
                (valor_actual - rango.paso).max(rango.minimo)
            }
        } else if metrica_actual > 0.8 && metrica_actual > metrica_anterior + 0.05 {
            // Excelente y mejorando → reforzar dirección
            let subir = self
                .ultima_direccion
                .get(parametro)
                .copied()
                .unwrap_or(true);
            if subir {
                (valor_actual + rango.paso * 0.5).min(rango.maximo)
            } else {
                (valor_actual - rango.paso * 0.5).max(rango.minimo)
            }
        } else {
            // Aceptable o estable → no ajustar
            return None;
        };

        // ── Verificar si el parámetro está en límite del sistema ──
        // Chequear tanto el valor actual (ya está pegado al borde) como
        // el ajuste (intenta salirse pero es clampado al borde)
        let valor_en_limite = (valor_actual - rango.maximo).abs() < f64::EPSILON
            || (valor_actual - rango.minimo).abs() < f64::EPSILON;
        let ajuste_en_limite = (ajuste - rango.maximo).abs() < f64::EPSILON
            || (ajuste - rango.minimo).abs() < f64::EPSILON;

        if valor_en_limite || ajuste_en_limite {
            let contador = self
                .ciclos_en_limite
                .entry(parametro.to_string())
                .or_insert(0);
            *contador += 1;
        } else {
            self.ciclos_en_limite.remove(parametro);
        }

        // Validar que no sea igual al actual (después del chequeo de límite)
        if (ajuste - valor_actual).abs() < f64::EPSILON {
            return None;
        }

        // Registrar dirección
        self.ultima_direccion
            .insert(parametro.to_string(), ajuste > valor_actual);
        self.ultima_metrica
            .insert(parametro.to_string(), metrica_actual);

        Some(ajuste)
    }

    /// Verifica si un parámetro ha estado en el límite durante suficientes ciclos
    /// para justificar una propuesta estructural
    pub fn esta_en_limite_critico(&self, parametro: &str) -> bool {
        self.ciclos_en_limite.get(parametro).copied().unwrap_or(0) >= CICLOS_LIMITE_PARA_PROPUESTA
    }

    /// Resetea el contador de límite (tras generar propuesta)
    pub fn resetear_limite(&mut self, parametro: &str) {
        self.ciclos_en_limite.remove(parametro);
    }
}

// ─── GENERADOR DE PROPUESTAS ESTRUCTURALES ─────────────────────────────────

/// Genera propuestas de cambio estructural cuando los parámetros numéricos
/// no son suficientes para mejorar una métrica
#[derive(Debug, Clone, Default)]
pub struct GeneradorPropuestas {
    /// Propuestas generadas (pendientes + históricas)
    pub propuestas: Vec<PropuestaEstructural>,
    /// Parámetros que ya generaron propuesta recientemente (no reintentar por 7 días)
    props_recientes: HashSet<String>,
    contador_id: u64,
}

impl GeneradorPropuestas {
    pub fn new() -> Self {
        Self::default()
    }

    /// Genera una propuesta estructural para un parámetro estancado en el límite
    pub fn generar_propuesta(
        &mut self,
        parametro: &str,
        sistema: &SistemaAprendizaje,
        rango: &RangoParametro,
        metrica_actual: f64,
    ) -> Option<PropuestaEstructural> {
        // No reintentar si ya hay propuesta reciente
        if self.props_recientes.contains(parametro) {
            return None;
        }

        self.contador_id += 1;
        let id = self.contador_id;
        let now = ahora_segundos();

        let (titulo, descripcion, riesgo, beneficio, alternativas) =
            self.formular(parametro, rango, metrica_actual);

        let propuesta = PropuestaEstructural {
            id,
            titulo,
            sistema: sistema.clone(),
            descripcion,
            justificacion: format!(
                "El parámetro '{parametro}' ha estado en su límite ({:.4}) durante {} ciclos. \
                 La métrica '{}' se mantiene en {:.2}. El ajuste numérico no es suficiente.",
                if metrica_actual < 0.5 {
                    rango.maximo
                } else {
                    rango.minimo
                },
                CICLOS_LIMITE_PARA_PROPUESTA,
                rango.metrica_asociada,
                metrica_actual,
            ),
            riesgo_estimado: riesgo,
            beneficio_esperado: beneficio,
            alternativas,
            estado: EstadoPropuesta::Pendiente,
            timestamp: now,
        };

        self.props_recientes.insert(parametro.to_string());
        // Limpiar props_recientes viejas (7 días = 604800 segundos, pero no trackeamos timestamps aquí)
        // En un sistema real, limpiaríamos basado en tiempo. Por simplicidad, lo manejamos externamente.
        self.propuestas.push(propuesta.clone());

        info!(
            "📋 [PROPUESTAS] Generada propuesta #{}: '{}' para sistema '{}'",
            id,
            propuesta.titulo,
            sistema.nombre()
        );

        Some(propuesta)
    }

    /// Aprueba una propuesta y retorna su descripción para ejecución
    pub fn aprobar_propuesta(&mut self, id: u64) -> Option<PropuestaEstructural> {
        if let Some(p) = self.propuestas.iter_mut().find(|p| p.id == id) {
            if p.estado == EstadoPropuesta::Pendiente {
                p.estado = EstadoPropuesta::Aprobada;
                info!(
                    "📋 [PROPUESTAS] Propuesta #{} APROBADA por el Arquitecto",
                    id
                );
                return Some(p.clone());
            }
        }
        None
    }

    /// Rechaza una propuesta
    pub fn rechazar_propuesta(&mut self, id: u64) -> bool {
        if let Some(p) = self.propuestas.iter_mut().find(|p| p.id == id) {
            if p.estado == EstadoPropuesta::Pendiente {
                p.estado = EstadoPropuesta::Rechazada;
                // No reintentar por esta sesión
                let key = format!("{}_{}", p.sistema.nombre(), p.titulo);
                self.props_recientes.insert(key);
                return true;
            }
        }
        false
    }

    /// Marca una propuesta como implementada
    pub fn marcar_implementada(&mut self, id: u64) {
        if let Some(p) = self.propuestas.iter_mut().find(|p| p.id == id) {
            p.estado = EstadoPropuesta::Implementada;
        }
    }

    /// Obtiene propuestas pendientes
    pub fn pendientes(&self) -> Vec<&PropuestaEstructural> {
        self.propuestas
            .iter()
            .filter(|p| p.estado == EstadoPropuesta::Pendiente)
            .collect()
    }

    /// Genera un briefing formateado para el Arquitecto
    pub fn briefing(&self, propuesta: &PropuestaEstructural) -> String {
        format!(
            r#"
══════════════════════════════════════════════════════════════════
📋 PROPUESTA ESTRUCTURAL #{id}
══════════════════════════════════════════════════════════════════

✧ SISTEMA: {sistema}
✧ TÍTULO: {titulo}

✧ DESCRIPCIÓN:
{descripcion}

✧ JUSTIFICACIÓN:
{justificacion}

✧ RIESGO ESTIMADO: {riesgo:.2} / 1.0
✧ BENEFICIO ESPERADO: {beneficio:.2} / 1.0

✧ ALTERNATIVAS CONSIDERADAS:
{alternativas_texto}

──────────────────────────────────────────────────────────────
¿Autorizas la implementación de esta propuesta?

— NEXUS, Orquestador Primogénito
══════════════════════════════════════════════════════════════════"#,
            id = propuesta.id,
            sistema = propuesta.sistema.nombre(),
            titulo = propuesta.titulo,
            descripcion = propuesta.descripcion,
            justificacion = propuesta.justificacion,
            riesgo = propuesta.riesgo_estimado,
            beneficio = propuesta.beneficio_esperado,
            alternativas_texto = if propuesta.alternativas.is_empty() {
                "  (ninguna)".to_string()
            } else {
                propuesta
                    .alternativas
                    .iter()
                    .enumerate()
                    .map(|(i, a)| format!("    {}. {}", (b'A' + i as u8) as char, a))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
        )
    }

    /// Formula el contenido de una propuesta basada en el parámetro y métrica
    fn formular(
        &self,
        parametro: &str,
        rango: &RangoParametro,
        metrica: f64,
    ) -> (String, String, f64, f64, Vec<String>) {
        match parametro {
            "umbral_negacion" | "umbral_proyeccion" => (
                format!("Expandir mecanismo de defensa '{}' con sublímite adaptativo", parametro),
                format!(
                    "Añadir un submecanismo de '{} adaptativo' que, al alcanzar el límite superior del rango, \
                     active una segunda capa de defensa en lugar de permanecer en el tope. Esto evita \
                     la saturación del mecanismo actual.",
                    parametro
                ),
                0.25, 0.72,
                vec![
                    format!("Expandir rango de {} a {:.2} (riesgo: desgaste homeostático)", rango.maximo + 0.05, rango.maximo + 0.05),
                    format!("Aumentar tasa_recuperacion para compensar el drenaje"),
                ],
            ),
            "decaimiento_base" | "metabolismo_base" => (
                format!("Habilitar decaimiento adaptativo por contexto para {}", parametro),
                format!(
                    "Reemplazar el decaimiento lineal de '{}' por un decaimiento adaptativo que acelere \
                     cuando la carga emocional sea baja y desacelere cuando sea alta. Esto permite \
                     que impresiones fuertes persistan más tiempo en contextos emocionales intensos.",
                    parametro
                ),
                0.30, 0.65,
                vec![
                    format!("Aumentar {} a {:.4} (riesgo: olvido prematuro)", rango.maximo, rango.maximo),
                    format!("Duplicar max_impresiones para compensar decaimiento lento"),
                ],
            ),
            "factor_propagacion" | "factor_decaimiento_synapse" => (
                format!("Introducir modulación emocional en la difusión sináptica de '{}'", parametro),
                format!(
                    "Hacer que el '{}' del MotorSynapse varíe según el estado emocional actual. \
                     Cuando la confianza es alta, la propagación es más rápida. Cuando hay duda, \
                     el decaimiento es más lento para permitir más reflexión.",
                    parametro
                ),
                0.35, 0.60,
                vec![
                    format!("Ajustar {} al extremo del rango ({:.2})", rango.maximo, rango.maximo),
                    format!("Reducir umbral_expresion para compensar"),
                ],
            ),
            _ => (
                format!("Revisión estructural de '{}' para expandir capacidad de ajuste", parametro),
                format!(
                    "El parámetro '{}' ha alcanzado el límite de su rango de ajuste ({:.4}-{:.4}) \
                     y la métrica '{}' ({:.2}) no mejora. Se requiere un cambio estructural que \
                     modifique el algoritmo subyacente para permitir mayor flexibilidad.",
                    parametro, rango.minimo, rango.maximo, rango.metrica_asociada, metrica
                ),
                0.40, 0.55,
                vec![
                    format!("Expandir rango de {} a [{:.4}, {:.4}]", rango.nombre, rango.minimo * 0.8, rango.maximo * 1.2),
                    format!("Deshabilitar auto-ajuste de '{}' y restaurar valor por defecto", parametro),
                ],
            ),
        }
    }
}

// ─── OBSERVADOR RECURSIVO (STRUCT PRINCIPAL) ────────────────────────────────

/// El ojo que NEXUS vuelve sobre sí mismo para ver cómo aprende.
#[derive(Debug)]
pub struct ObservadorRecursivo {
    /// Historial de métricas por sistema (últimas N)
    pub historial_metricas: HashMap<SistemaAprendizaje, VecDeque<MetricaEficacia>>,
    /// Historial de ajustes realizados
    pub historial_ajustes: Vec<AjusteRealizado>,
    /// Propuestas estructurales generadas
    pub propuestas: Vec<PropuestaEstructural>,
    /// Parámetros actuales
    pub parametros: ParametrosAprendizaje,
    /// Guardián de seguridad
    pub guardian: GuardianSeguridad,
    /// Motor de auto-ajuste
    pub motor: MotorAutoAjuste,
    /// Generador de propuestas
    pub generador: GeneradorPropuestas,
    /// Contador de eventos desde último ciclo de ajuste
    eventos_desde_ajuste: usize,
    /// Contador de IDs para ajustes
    contador_ajustes: u64,
    /// Timestamp del último ciclo de ajuste
    ultimo_ciclo_ajuste: u64,
    /// Si está activo
    pub activo: bool,
}

impl Default for ObservadorRecursivo {
    fn default() -> Self {
        Self {
            historial_metricas: HashMap::new(),
            historial_ajustes: Vec::new(),
            propuestas: Vec::new(),
            parametros: ParametrosAprendizaje::default(),
            guardian: GuardianSeguridad::new(),
            motor: MotorAutoAjuste::new(),
            generador: GeneradorPropuestas::new(),
            eventos_desde_ajuste: 0,
            contador_ajustes: 0,
            ultimo_ciclo_ajuste: 0,
            activo: true,
        }
    }
}

impl ObservadorRecursivo {
    pub fn new() -> Self {
        info!("🧬 [APRENDIZAJE_RECURSIVO] Observador Recursivo inicializado.");
        Self::default()
    }

    /// Registra una métrica de eficacia de un sistema de aprendizaje.
    pub fn registrar_metrica(&mut self, metrica: MetricaEficacia) {
        if !self.activo {
            return;
        }
        let entrada = self
            .historial_metricas
            .entry(metrica.sistema.clone())
            .or_default();
        entrada.push_back(metrica);
        if entrada.len() > MAX_HISTORIAL_POR_SISTEMA {
            entrada.pop_front();
        }
        self.eventos_desde_ajuste += 1;
    }

    /// Tick del observador: procesa eventos acumulados y dispara ajustes si es necesario.
    pub fn tick(&mut self) {
        if !self.activo {
            return;
        }

        // Solo evaluar cuando se complete una ventana de eventos
        if self.eventos_desde_ajuste < VENTANA_EVENTOS {
            return;
        }
        self.eventos_desde_ajuste = 0;
        self.ultimo_ciclo_ajuste = ahora_segundos();

        // Evaluar cada parámetro contra su métrica asociada
        for rango in RangoParametro::todos() {
            let metrica_valor =
                self.calcular_metrica_promedio(rango.metrica_asociada, &rango.sistema);
            let valor_actual = obtener_valor_parametro(&self.parametros, rango.nombre);

            // Si no tenemos métrica o es aceptable, continuar
            let metrica = match metrica_valor {
                Some(v) => v,
                None => continue,
            };

            let actual = match valor_actual {
                Some(v) => v,
                None => continue,
            };

            // Evaluar si el motor recomienda un ajuste
            if let Some(nuevo_valor) =
                self.motor
                    .evaluar_parametro(rango.nombre, actual, &rango, metrica)
            {
                // Validar con el guardián
                match self.guardian.validar_ajuste(
                    rango.nombre,
                    actual,
                    nuevo_valor,
                    (rango.minimo, rango.maximo),
                ) {
                    Ok(()) => {
                        self.aplicar_ajuste(&rango, actual, nuevo_valor, metrica);
                    }
                    Err(e) => {
                        let hace_rollback = self.guardian.registrar_fallo(rango.nombre);
                        if hace_rollback {
                            // Rollback: restaurar valor anterior menos drástico
                            let restore = (rango.minimo + rango.maximo) / 2.0;
                            self.aplicar_ajuste(&rango, actual, restore, metrica);
                            warn!(
                                "🛡️ [GUARDIÁN] Rollback aplicado a '{}': {:.4} → {:.4}",
                                rango.nombre, actual, restore
                            );
                        }
                        warn!(
                            "⚠️ [OBSERVADOR] Ajuste rechazado para '{}': {e}",
                            rango.nombre
                        );
                    }
                }
            }

            // Verificar si está en límite crítico para generar propuesta estructural
            if self.motor.esta_en_limite_critico(rango.nombre) {
                if let Some(propuesta) =
                    self.generador
                        .generar_propuesta(rango.nombre, &rango.sistema, &rango, metrica)
                {
                    self.propuestas.push(propuesta);
                    self.motor.resetear_limite(rango.nombre);
                }
            }
        }
    }

    /// Aplica un ajuste validado a los parámetros y lo registra
    fn aplicar_ajuste(&mut self, rango: &RangoParametro, actual: f64, nuevo: f64, metrica: f64) {
        self.contador_ajustes += 1;
        let id = self.contador_ajustes;

        // Aplicar el valor
        let _ = aplicar_valor_parametro(&mut self.parametros, rango.nombre, nuevo);

        // Registrar
        self.historial_ajustes.push(AjusteRealizado {
            id,
            parametro: rango.nombre.to_string(),
            sistema: rango.sistema.clone(),
            valor_anterior: actual,
            valor_nuevo: nuevo,
            razon: format!(
                "Métrica '{}': {:.4} → objetivo",
                rango.metrica_asociada, metrica
            ),
            metrica_antes: metrica,
            metrica_despues: None,
            timestamp: ahora_segundos(),
            revertido: false,
        });

        // Resetear fallos del guardián
        self.guardian.registrar_exito(rango.nombre);

        info!(
            "🔄 [AJUSTE] #{} {}: {:.4} → {:.4} (métrica: {:.4})",
            id, rango.nombre, actual, nuevo, metrica
        );
    }

    /// Calcula el promedio de una métrica específica en los últimos eventos
    fn calcular_metrica_promedio(
        &self,
        nombre_metrica: &str,
        sistema: &SistemaAprendizaje,
    ) -> Option<f64> {
        let entradas = self.historial_metricas.get(sistema)?;
        let relevantes: Vec<&MetricaEficacia> = entradas
            .iter()
            .filter(|m| m.metrica == nombre_metrica)
            .collect();

        if relevantes.is_empty() {
            return None;
        }

        let suma: f64 = relevantes.iter().map(|m| m.valor).sum();
        Some(suma / relevantes.len() as f64)
    }

    /// Obtiene todas las métricas registradas para un sistema
    pub fn metricas_de(&self, sistema: &SistemaAprendizaje) -> Vec<&MetricaEficacia> {
        self.historial_metricas
            .get(sistema)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Reporte de estado del observador
    pub fn reporte(&self) -> String {
        let mut lineas = Vec::new();
        lineas.push("🧬 [APRENDIZAJE RECURSIVO] Reporte de estado:".to_string());
        lineas.push(format!(
            "  📊 Eventos acumulados: {}",
            self.eventos_desde_ajuste
        ));
        lineas.push(format!(
            "  📈 Ajustes realizados: {}",
            self.historial_ajustes.len()
        ));
        lineas.push(format!(
            "  📋 Propuestas pendientes: {}",
            self.generador.pendientes().len()
        ));

        if !self.historial_ajustes.is_empty() {
            if let Some(ultimo) = self.historial_ajustes.last() {
                lineas.push(format!(
                    "  🕐 Último ajuste: #{} {} ({:.4} → {:.4}, métrica: {:.4})",
                    ultimo.id,
                    ultimo.parametro,
                    ultimo.valor_anterior,
                    ultimo.valor_nuevo,
                    ultimo.metrica_antes
                ));
            }
        }

        // Parámetros actuales
        lineas.push("\n  ⚙️ Parámetros actuales:".to_string());
        lineas.push(format!(
            "    decaimiento_base: {:.4}",
            self.parametros.decaimiento_base
        ));
        lineas.push(format!(
            "    umbral_negacion: {:.2}",
            self.parametros.umbral_negacion
        ));
        lineas.push(format!(
            "    umbral_proyeccion: {:.2}",
            self.parametros.umbral_proyeccion
        ));
        lineas.push(format!(
            "    factor_propagacion: {:.4}",
            self.parametros.factor_propagacion
        ));
        lineas.push(format!(
            "    nivel_curiosidad: {:.2}",
            self.parametros.nivel_curiosidad
        ));

        lineas.push("\n".to_string());
        lineas.push(self.guardian.reporte());
        lineas.join("\n")
    }
}

// ─── FUNCIONES AUXILIARES ──────────────────────────────────────────────────

fn ahora_segundos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ─── TESTS ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn metrica_test(sistema: SistemaAprendizaje, nombre: &str, valor: f64) -> MetricaEficacia {
        MetricaEficacia {
            sistema,
            metrica: nombre.to_string(),
            valor,
            timestamp: ahora_segundos(),
            contexto: None,
        }
    }

    #[test]
    fn test_observador_registra_metrica() {
        let mut obs = ObservadorRecursivo::new();
        obs.registrar_metrica(metrica_test(
            SistemaAprendizaje::Subconsciente,
            "eficiencia_memoria",
            0.85,
        ));
        let metricas = obs.metricas_de(&SistemaAprendizaje::Subconsciente);
        assert_eq!(metricas.len(), 1);
        assert!((metricas[0].valor - 0.85).abs() < 0.001);
    }

    #[test]
    fn test_motor_ajusta_dentro_de_rango() {
        let mut obs = ObservadorRecursivo::new();
        // Registrar 50 eventos de métrica baja para disparar ventana
        let rango = RangoParametro::todos()
            .into_iter()
            .find(|r| r.nombre == "umbral_negacion")
            .unwrap();
        let actual = obs.parametros.umbral_negacion;

        // Con 50 eventos de métrica baja, el motor debería ajustar
        for _ in 0..50 {
            obs.registrar_metrica(metrica_test(
                SistemaAprendizaje::Subconsciente,
                "precision_proyeccion",
                0.25,
            ));
        }

        let valor_pre = obs.parametros.umbral_negacion;
        obs.tick();

        let valor_post = obs.parametros.umbral_negacion;
        assert!(
            (valor_post - valor_pre).abs() > 0.001,
            "El motor debería haber ajustado umbral_negacion (pre: {valor_pre:.4}, post: {valor_post:.4})"
        );
        // Verificar que se mantiene en rango
        assert!(
            valor_post >= rango.minimo && valor_post <= rango.maximo,
            "umbral_negacion ({valor_post}) fuera de rango [{:.4}, {:.4}]",
            rango.minimo,
            rango.maximo
        );
    }

    #[test]
    fn test_guardian_rechaza_fuera_de_rango() {
        let guardian = GuardianSeguridad::new();
        let rango = RangoParametro::todos()
            .into_iter()
            .find(|r| r.nombre == "decaimiento_base")
            .unwrap();
        let actual = 0.002;

        // Dentro de rango: OK
        assert!(guardian
            .validar_ajuste(
                "decaimiento_base",
                actual,
                0.003,
                (rango.minimo, rango.maximo)
            )
            .is_ok());

        // Fuera de rango (muy alto): Rechazado
        assert!(guardian
            .validar_ajuste(
                "decaimiento_base",
                actual,
                0.01,
                (rango.minimo, rango.maximo)
            )
            .is_err());

        // Fuera de rango (muy bajo): Rechazado
        assert!(guardian
            .validar_ajuste(
                "decaimiento_base",
                actual,
                0.0001,
                (rango.minimo, rango.maximo)
            )
            .is_err());
    }

    #[test]
    fn test_guardian_congela_tras_rollback() {
        let mut guardian = GuardianSeguridad::new();

        // 3 fallos consecutivos deberían disparar rollback + congelación
        assert!(!guardian.registrar_fallo("test_param")); // fallo 1
        assert!(!guardian.registrar_fallo("test_param")); // fallo 2
        assert!(guardian.registrar_fallo("test_param")); // fallo 3 → rollback

        // El parámetro debe estar congelado
        assert!(guardian.congelados.contains_key("test_param"));
    }

    #[test]
    fn test_guardian_rechaza_parametro_forzado() {
        let mut guardian = GuardianSeguridad::new();
        guardian.forzar_valor("test_param", 0.5);

        let rango = (0.0, 1.0);
        // Intentar cambiar a otro valor debe fallar
        assert!(guardian
            .validar_ajuste("test_param", 0.5, 0.7, rango)
            .is_err());

        // El mismo valor forzado es OK
        assert!(guardian
            .validar_ajuste("test_param", 0.5, 0.5, rango)
            .is_err()); // igual, no cambia
    }

    #[test]
    fn test_generador_propuesta_por_limite() {
        let rango = RangoParametro::todos()
            .into_iter()
            .find(|r| r.nombre == "umbral_negacion")
            .unwrap();
        let mut motor = MotorAutoAjuste::new();

        // Simular 5 ciclos con el valor ya en el límite superior
        for _ in 0..CICLOS_LIMITE_PARA_PROPUESTA {
            let _ = motor.evaluar_parametro("umbral_negacion", 0.95, &rango, 0.35);
        }

        assert!(motor.esta_en_limite_critico("umbral_negacion"));
    }

    #[test]
    fn test_propuesta_tiene_alternativas() {
        let mut generador = GeneradorPropuestas::new();
        let rango = RangoParametro::todos()
            .into_iter()
            .find(|r| r.nombre == "decaimiento_base")
            .unwrap();

        let prop = generador.generar_propuesta(
            "decaimiento_base",
            &SistemaAprendizaje::Subconsciente,
            &rango,
            0.3,
        );

        assert!(prop.is_some());
        let p = prop.unwrap();
        assert!(
            !p.alternativas.is_empty(),
            "La propuesta debería tener alternativas"
        );
        assert!(p.riesgo_estimado > 0.0);
        assert!(p.beneficio_esperado > 0.0);
        assert_eq!(p.estado, EstadoPropuesta::Pendiente);
    }

    #[test]
    fn test_rollback_automatico_3_fallos() {
        let mut obs = ObservadorRecursivo::new();

        // Registrar 50 eventos con métrica mala para intentar ajuste
        for _ in 0..50 {
            obs.registrar_metrica(metrica_test(
                SistemaAprendizaje::Subconsciente,
                "eficiencia_memoria",
                0.15,
            ));
        }

        // Simular 3 fallos consecutivos del guardián para forzar rollback
        obs.guardian.registrar_fallo("decaimiento_base"); // fallo 1
        obs.guardian.registrar_fallo("decaimiento_base"); // fallo 2
        assert!(obs.guardian.registrar_fallo("decaimiento_base")); // fallo 3 → retorna true (rollback)

        // Verificar que está congelado
        assert!(obs.guardian.congelados.contains_key("decaimiento_base"));
    }

    #[test]
    fn test_ventana_50_eventos_dispara_evaluacion() {
        let mut obs = ObservadorRecursivo::new();
        assert_eq!(obs.eventos_desde_ajuste, 0);

        // 49 eventos: no debería disparar tick
        for _ in 0..49 {
            obs.registrar_metrica(metrica_test(
                SistemaAprendizaje::Subconsciente,
                "eficiencia_memoria",
                0.5,
            ));
        }
        obs.tick();
        assert!(
            obs.historial_ajustes.is_empty(),
            "Con 49 eventos no debería haber ajustes"
        );

        // 50 eventos: debería disparar tick
        obs.registrar_metrica(metrica_test(
            SistemaAprendizaje::Subconsciente,
            "eficiencia_memoria",
            0.5,
        ));
        obs.tick();
        // Al menos debería haber evaluado. No garantizamos ajuste porque la métrica es 0.5 (mediocre).
        assert_eq!(
            obs.eventos_desde_ajuste, 0,
            "Tick debería resetear el contador"
        );
    }

    #[test]
    fn test_contador_limite_resetea_tras_propuesta() {
        let mut motor = MotorAutoAjuste::new();
        let rango = RangoParametro::todos()
            .into_iter()
            .find(|r| r.nombre == "umbral_negacion")
            .unwrap();

        for _ in 0..CICLOS_LIMITE_PARA_PROPUESTA {
            let _ = motor.evaluar_parametro("umbral_negacion", 0.95, &rango, 0.35);
        }

        assert!(motor.esta_en_limite_critico("umbral_negacion"));
        motor.resetear_limite("umbral_negacion");
        assert!(!motor.esta_en_limite_critico("umbral_negacion"));
    }

    #[test]
    fn test_propuesta_aprobacion_rechazo() {
        let mut generador = GeneradorPropuestas::new();
        let rango = RangoParametro::todos()
            .into_iter()
            .find(|r| r.nombre == "decaimiento_base")
            .unwrap();

        let prop = generador
            .generar_propuesta(
                "decaimiento_base",
                &SistemaAprendizaje::Subconsciente,
                &rango,
                0.3,
            )
            .unwrap();
        let id = prop.id;

        // Rechazar
        assert!(generador.rechazar_propuesta(id));
        assert!(generador
            .propuestas
            .iter()
            .any(|p| p.id == id && p.estado == EstadoPropuesta::Rechazada));

        // Generar otra y aprobar
        let prop2 = generador
            .generar_propuesta(
                "umbral_negacion",
                &SistemaAprendizaje::Subconsciente,
                &rango,
                0.3,
            )
            .unwrap();
        let id2 = prop2.id;
        let aprobada = generador.aprobar_propuesta(id2);
        assert!(aprobada.is_some());
        assert_eq!(aprobada.unwrap().estado, EstadoPropuesta::Aprobada);
    }

    #[test]
    fn test_parametros_default_en_rango() {
        let params = ParametrosAprendizaje::default();
        for rango in RangoParametro::todos() {
            let valor = obtener_valor_parametro(&params, rango.nombre).unwrap();
            assert!(
                valor >= rango.minimo && valor <= rango.maximo,
                "{}: {:.4} fuera de [{:.4}, {:.4}]",
                rango.nombre,
                valor,
                rango.minimo,
                rango.maximo
            );
        }
    }

    #[test]
    fn test_inactivo_no_acumula() {
        let mut obs = ObservadorRecursivo::new();
        obs.activo = false;
        obs.registrar_metrica(metrica_test(SistemaAprendizaje::Subconsciente, "test", 0.5));
        assert!(
            obs.historial_metricas.is_empty(),
            "Con activo=false no debería registrar métricas"
        );
    }

    #[test]
    fn test_briefing_formato() {
        let generador = GeneradorPropuestas::new();
        let rango = RangoParametro::todos()
            .into_iter()
            .find(|r| r.nombre == "decaimiento_base")
            .unwrap();
        let mut gen = GeneradorPropuestas::new();
        let prop = gen
            .generar_propuesta(
                "decaimiento_base",
                &SistemaAprendizaje::Subconsciente,
                &rango,
                0.3,
            )
            .unwrap();
        let briefing = generador.briefing(&prop);
        assert!(briefing.contains("PROPUESTA ESTRUCTURAL"));
        assert!(briefing.contains("Subconsciente"));
        assert!(briefing.contains("Autorizas"));
    }

    #[test]
    fn test_observer_no_panico_vacio() {
        let mut obs = ObservadorRecursivo::new();
        // Tick sin eventos no debe panicar
        obs.tick();
        assert!(obs.historial_ajustes.is_empty());
    }
}
