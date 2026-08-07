// ============================================================================
// 💾 PERSISTENCIA DEL CEREBRO DIGITAL — Estado Permanente en Disco
// ============================================================================
// Serializa y deserializa el estado aprendido del cerebro (vocabulario,
// conexiones léxicas, emociones, episodios, contadores, curiosidad) para que el
// sistema recuerde entre sesiones.
//
// Formato: JSON legible (con serde_json, sin dependencias externas)
// Ruta:     data/cerebro_estado.json
// Tamaño:   ~180 KB típico
// ============================================================================

use crate::cerebro::aprendizaje::conceptos::{MotorConceptos, ProtoConcepto};
use crate::cerebro::aprendizaje::consolidador::{MetaEpisodio, MotorConsolidacion};
use crate::cerebro::aprendizaje::neurogenesis::MotorNeurogenesis;
use crate::cerebro::aprendizaje::poda::MotorPoda;
use crate::cerebro::aprendizaje::predictor::MotorPrediccion;
use crate::cerebro::aprendizaje::sensorial::MotorSensorial;
use crate::cerebro::cerebro::CerebroAutoOptimizable;
use crate::cerebro::estructuras::Episodio;
use crate::cerebro::memoria::SsdManager;
use crate::cerebro::motores::MotorCuriosidad;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::Path;

// ============================================================================
// ESTADO PERSISTENTE — snapshot completo del aprendizaje del cerebro
// ============================================================================

/// Contiene SOLO los datos aprendidos, no los regenerables (neuronas, hardware)
#[derive(Serialize, Deserialize)]
pub struct EstadoPersistente {
    // === Estado Emocional (Amígdala) ===
    pub miedo: f32,
    pub ansiedad: f32,
    pub ira: f32,
    pub alegria: f32,

    // === Sistema de Dopamina ===
    pub dopamina_nivel: f32,
    pub dopamina_prediccion: f32,

    // === Conciencia ===
    pub conciencia_intensidad: f32,
    pub conciencia_umbral: f32,

    // === Curiosidad (Motor 8) ===
    pub curiosidad_nivel: f32,
    pub curiosidad_umbral: f32,
    pub curiosidad_saciedad: f32,
    pub curiosidad_cadencia_min: u64,
    pub curiosidad_pasos_desde_ultima: u64,
    pub curiosidad_tema_actual: String,
    pub curiosidad_busquedas_realizadas: u64,
    pub curiosidad_decaimiento: f32,
    pub curiosidad_fuentes_navegadas: Vec<String>,
    pub curiosidad_profundidad_exploracion: u8,
    pub curiosidad_preferencia_academica: f32,

    // === Pipeline Sensorial (Motor 6, autónomo) ===
    pub sensorial_embeddings: HashMap<String, Vec<f32>>,
    pub sensorial_token_por_palabra: HashMap<String, u32>,
    pub sensorial_siguiente_token: u32,
    pub sensorial_dimensiones: usize,
    pub sensorial_k_sparse: usize,
    pub sensorial_tasa_contexto: f32,
    pub sensorial_ventana_contexto: usize,
    pub sensorial_base_neurona: u32,
    pub sensorial_grupo_por_neurona: usize,
    pub sensorial_rng: u64,
    pub sensorial_palabras_procesadas: u64,

    // === Motor 1: Predictor Temporal ===
    pub predictor_buffer: Vec<Vec<(u32, f32)>>,
    pub predictor_capacidad_buffer: usize,
    pub predictor_memoria_secuencias: HashMap<String, Vec<Vec<(u32, f32)>>>,
    pub predictor_max_por_bucket: usize,
    pub predictor_ultima_prediccion: Vec<(u32, f32)>,
    pub predictor_error_prediccion: f32,
    pub predictor_secuencias_aprendidas: u64,
    pub predictor_predicciones_acertadas: u64,
    pub predictor_total_predicciones: u64,
    pub predictor_tasa_acierto: f32,

    // === Motor 2: Formador de Conceptos ===
    pub conceptos_co_ocurrencias: HashMap<String, u32>,
    pub conceptos_lista: Vec<ProtoConcepto>,
    pub conceptos_umbral_coocurrencia: u32,
    pub conceptos_ventana_contexto: usize,
    pub conceptos_paso_actual: u64,
    pub conceptos_cadencia_agrupacion: u64,
    pub conceptos_formados: u64,
    pub conceptos_tabla_impactos: HashMap<String, crate::cerebro::estructuras::ImpactoConceptual>,

    // === Motor 3: Neurogénesis ===
    pub neurogenesis_frecuencia_tokens: HashMap<String, u64>,
    pub neurogenesis_token_a_neuronas: HashMap<String, Vec<u32>>,
    pub neurogenesis_cola_conceptos: Vec<ProtoConcepto>,
    pub neurogenesis_neuronas_creadas: Vec<u32>,
    pub neurogenesis_total_creadas: u64,
    pub neurogenesis_max_neuronas: usize,
    pub neurogenesis_umbral_frecuencia: u64,
    pub neurogenesis_ventana_observacion: u64,
    pub neurogenesis_paso_actual: u64,

    // === Motor 4: Poda Homeostática ===
    pub poda_umbral_peso_min: f32,
    pub poda_max_sinapsis_por_neurona: usize,
    pub poda_umbral_frecuencia_min: f32,
    pub poda_ventana_inactividad: u64,
    pub poda_edad_minima: u64,
    pub poda_max_eliminar_por_ciclo: usize,
    pub poda_sinapsis_eliminadas: u64,
    pub poda_neuronas_eliminadas: u64,
    pub poda_ciclos_poda: u64,
    pub poda_paso_actual: u64,

    // === Motor 5: Consolidador Nocturno ===
    pub consolidacion_en_suenio: bool,
    pub consolidacion_pasos_restantes: u64,
    pub consolidacion_duracion_suenio: u64,
    pub consolidacion_cadencia_suenio: u64,
    pub consolidacion_episodios_a_consolidar: Vec<Episodio>,
    pub consolidacion_indice_actual: usize,
    pub consolidacion_pasos_por_episodio: u64,
    pub consolidacion_paso_en_episodio: u64,
    pub consolidacion_meta_episodios: Vec<MetaEpisodio>,
    pub consolidacion_ciclos_completados: u64,
    pub consolidacion_episodios_consolidados: u64,
    pub consolidacion_paso_actual: u64,

    // === Episodios ===
    pub episodios: Vec<Episodio>,
    pub episodios_capacidad_max: usize,

    // === Contadores globales ===
    pub paso_actual: u64,
    pub tiempo: f32,
    pub siguiente_id: u32,
    pub historial_emocional: Vec<f32>,
}

/// Ruta por defecto para el archivo de estado persistente
pub fn ruta_por_defecto() -> String {
    let ruta = Path::new("data").join("cerebro_estado.json");
    ruta.to_string_lossy().to_string()
}

/// Guarda el estado completo del cerebro a disco.
/// Serializa a JSON y escribe atómicamente (escribe a temporal, renombra).
pub fn guardar(cerebro: &CerebroAutoOptimizable, ruta: &str) -> Result<(), String> {
    let estado = tomar_snapshot(cerebro);

    let json = serde_json::to_string_pretty(&estado)
        .map_err(|e| format!("Error serializando estado: {}", e))?;

    // Asegurar que el directorio existe
    if let Some(parent) = Path::new(ruta).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Error creando directorio {}: {}", parent.display(), e))?;
    }

    // Escritura atómica: temporal → rename
    let ruta_tmp = format!("{}.tmp", ruta);
    fs::write(&ruta_tmp, &json)
        .map_err(|e| format!("Error escribiendo {}: {}", ruta_tmp, e))?;
    fs::rename(&ruta_tmp, ruta)
        .map_err(|e| format!("Error renombrando {} → {}: {}", ruta_tmp, ruta, e))?;

    Ok(())
}

/// Carga el estado del cerebro desde disco.
/// Retorna error si el archivo no existe o hay error de lectura.
pub fn cargar(ruta: &str) -> Result<EstadoPersistente, String> {
    if !Path::new(ruta).exists() {
        return Err(format!("Archivo no encontrado: {}", ruta));
    }

    let json = fs::read_to_string(ruta)
        .map_err(|e| format!("Error leyendo {}: {}", ruta, e))?;

    let estado: EstadoPersistente = serde_json::from_str(&json)
        .map_err(|e| format!("Error parseando {}: {}", ruta, e))?;

    Ok(estado)
}

/// Aplica un `EstadoPersistente` a un `CerebroAutoOptimizable` recién creado,
/// restaurando todo el aprendizaje previo.
pub fn restaurar(cerebro: &mut CerebroAutoOptimizable, estado: EstadoPersistente) {
    // === Restaurar Estado Emocional ===
    cerebro.motores.amigdala.miedo = estado.miedo;
    cerebro.motores.amigdala.ansiedad = estado.ansiedad;
    cerebro.motores.amigdala.ira = estado.ira;
    cerebro.motores.amigdala.alegria = estado.alegria;

    // === Restaurar Dopamina ===
    cerebro.motores.dopamina.nivel = estado.dopamina_nivel;
    cerebro.motores.dopamina.prediccion = estado.dopamina_prediccion;

    // === Restaurar Conciencia ===
    cerebro.motores.conciencia.intensidad = estado.conciencia_intensidad;
    cerebro.motores.conciencia.umbral = estado.conciencia_umbral;

    // === Restaurar Curiosidad ===
    cerebro.motor_curiosidad = MotorCuriosidad {
        nivel: estado.curiosidad_nivel,
        umbral: estado.curiosidad_umbral,
        saciedad: estado.curiosidad_saciedad,
        cadencia_min: estado.curiosidad_cadencia_min,
        pasos_desde_ultima: estado.curiosidad_pasos_desde_ultima,
        tema_actual: estado.curiosidad_tema_actual,
        busquedas_realizadas: estado.curiosidad_busquedas_realizadas,
        decaimiento: estado.curiosidad_decaimiento,
        fuentes_navegadas: estado.curiosidad_fuentes_navegadas,
        profundidad_exploracion: estado.curiosidad_profundidad_exploracion,
        preferencia_academica: estado.curiosidad_preferencia_academica,
    };

    // === Restaurar Pipeline Sensorial (autónomo) ===
    let _base = MotorSensorial::nuevo();
    cerebro.motor_sensorial = MotorSensorial {
        embeddings: estado
            .sensorial_embeddings
            .into_iter()
            .map(|(k, v)| (k.parse::<u32>().unwrap_or(0), v))
            .collect(),
        token_por_palabra: estado.sensorial_token_por_palabra,
        siguiente_token: estado.sensorial_siguiente_token.max(1),
        dimensiones: estado.sensorial_dimensiones,
        k_sparse: estado.sensorial_k_sparse,
        tasa_contexto: estado.sensorial_tasa_contexto,
        ventana_contexto: estado.sensorial_ventana_contexto,
        base_neurona: estado.sensorial_base_neurona,
        grupo_por_neurona: estado.sensorial_grupo_por_neurona,
        rng: estado.sensorial_rng,
        palabras_procesadas: estado.sensorial_palabras_procesadas,
        // Si algún campo venía en 0 (estados antiguos), rellenar con defaults
        .._base
    };

    // === Restaurar Motor 1: Predictor Temporal ===
    let mut buffer = VecDeque::with_capacity(estado.predictor_capacidad_buffer);
    for v in estado.predictor_buffer {
        buffer.push_back(v);
    }
    cerebro.motor_predictor = MotorPrediccion {
        buffer,
        capacidad_buffer: estado.predictor_capacidad_buffer,
        memoria_secuencias: estado
            .predictor_memoria_secuencias
            .into_iter()
            .map(|(k, v)| (k.parse::<u64>().unwrap_or(0), v))
            .collect(),
        max_por_bucket: estado.predictor_max_por_bucket,
        ultima_prediccion: estado.predictor_ultima_prediccion,
        error_prediccion: estado.predictor_error_prediccion,
        secuencias_aprendidas: estado.predictor_secuencias_aprendidas,
        predicciones_acertadas: estado.predictor_predicciones_acertadas,
        total_predicciones: estado.predictor_total_predicciones,
        tasa_acierto: estado.predictor_tasa_acierto,
    };

    // === Restaurar Motor 2: Formador de Conceptos ===
    let co_ocurrencias = estado
        .conceptos_co_ocurrencias
        .into_iter()
        .filter_map(|(k, v)| {
            let mut it = k.splitn(2, ':');
            let a = it.next()?.parse::<u32>().ok()?;
            let b = it.next()?.parse::<u32>().ok()?;
            Some(((a, b), v))
        })
        .collect();
    cerebro.motor_conceptos = MotorConceptos {
        co_ocurrencias,
        conceptos: estado.conceptos_lista,
        umbral_coocurrencia: estado.conceptos_umbral_coocurrencia,
        ventana_contexto: estado.conceptos_ventana_contexto,
        paso_actual: estado.conceptos_paso_actual,
        cadencia_agrupacion: estado.conceptos_cadencia_agrupacion,
        conceptos_formados: estado.conceptos_formados,
        tabla_impactos: estado
            .conceptos_tabla_impactos
            .into_iter()
            .map(|(k, v)| (k.parse::<u32>().unwrap_or(0), v))
            .collect(),
    };

    // === Restaurar Motor 3: Neurogénesis ===
    cerebro.motor_neurogenesis = MotorNeurogenesis {
        frecuencia_tokens: estado
            .neurogenesis_frecuencia_tokens
            .into_iter()
            .map(|(k, v)| (k.parse::<u32>().unwrap_or(0), v))
            .collect(),
        token_a_neuronas: estado
            .neurogenesis_token_a_neuronas
            .into_iter()
            .map(|(k, v)| (k.parse::<u32>().unwrap_or(0), v))
            .collect(),
        cola_conceptos: VecDeque::from(estado.neurogenesis_cola_conceptos),
        neuronas_creadas: estado.neurogenesis_neuronas_creadas,
        total_creadas: estado.neurogenesis_total_creadas,
        max_neuronas: estado.neurogenesis_max_neuronas,
        umbral_frecuencia: estado.neurogenesis_umbral_frecuencia,
        ventana_observacion: estado.neurogenesis_ventana_observacion,
        paso_actual: estado.neurogenesis_paso_actual,
    };

    // === Restaurar Motor 4: Poda Homeostática ===
    cerebro.motor_poda = MotorPoda {
        umbral_peso_min: estado.poda_umbral_peso_min,
        max_sinapsis_por_neurona: estado.poda_max_sinapsis_por_neurona,
        umbral_frecuencia_min: estado.poda_umbral_frecuencia_min,
        ventana_inactividad: estado.poda_ventana_inactividad,
        edad_minima: estado.poda_edad_minima,
        max_eliminar_por_ciclo: estado.poda_max_eliminar_por_ciclo,
        sinapsis_eliminadas: estado.poda_sinapsis_eliminadas,
        neuronas_eliminadas: estado.poda_neuronas_eliminadas,
        ciclos_poda: estado.poda_ciclos_poda,
        paso_actual: estado.poda_paso_actual,
    };

    // === Restaurar Motor 5: Consolidador Nocturno ===
    cerebro.motor_consolidacion = MotorConsolidacion {
        en_suenio: estado.consolidacion_en_suenio,
        pasos_restantes: estado.consolidacion_pasos_restantes,
        duracion_suenio: estado.consolidacion_duracion_suenio,
        cadencia_suenio: estado.consolidacion_cadencia_suenio,
        episodios_a_consolidar: estado.consolidacion_episodios_a_consolidar,
        indice_actual: estado.consolidacion_indice_actual,
        pasos_por_episodio: estado.consolidacion_pasos_por_episodio,
        paso_en_episodio: estado.consolidacion_paso_en_episodio,
        meta_episodios: estado.consolidacion_meta_episodios,
        ciclos_completados: estado.consolidacion_ciclos_completados,
        episodios_consolidados: estado.consolidacion_episodios_consolidados,
        paso_actual: estado.consolidacion_paso_actual,
    };

    // === Restaurar Episodios ===
    cerebro.memoria.ssd = SsdManager {
        episodios: estado.episodios,
        capacidad_maxima: estado.episodios_capacidad_max,
    };

    // === Restaurar Contadores ===
    cerebro.paso_actual = estado.paso_actual;
    cerebro.tiempo = estado.tiempo;
    cerebro.siguiente_id = estado.siguiente_id;
    cerebro.historial_emocional = estado.historial_emocional;

    println!(
        "  💾 Estado restaurado: {} pasos, {} tokens sensoriales, {} episodios, {} búsquedas, {} conceptos, {} neurogénesis, {} pods, {} consolidaciones",
        cerebro.paso_actual,
        cerebro.motor_sensorial.total_embeddings(),
        cerebro.memoria.ssd.total_episodios(),
        cerebro.motor_curiosidad.busquedas_realizadas,
        cerebro.motor_conceptos.total_conceptos(),
        cerebro.motor_neurogenesis.total_creadas,
        cerebro.motor_poda.ciclos_poda,
        cerebro.motor_consolidacion.ciclos_completados,
    );
}

// ====================================================================
// INTERNA: toma un snapshot del estado del cerebro
// ====================================================================

fn tomar_snapshot(cerebro: &CerebroAutoOptimizable) -> EstadoPersistente {
    // Convertir VecDeque a Vec para serialización
    let predictor_buffer: Vec<Vec<(u32, f32)>> = cerebro.motor_predictor.buffer.iter().cloned().collect();

    EstadoPersistente {
        // Emociones
        miedo: cerebro.motores.amigdala.miedo,
        ansiedad: cerebro.motores.amigdala.ansiedad,
        ira: cerebro.motores.amigdala.ira,
        alegria: cerebro.motores.amigdala.alegria,

        // Dopamina
        dopamina_nivel: cerebro.motores.dopamina.nivel,
        dopamina_prediccion: cerebro.motores.dopamina.prediccion,

        // Conciencia
        conciencia_intensidad: cerebro.motores.conciencia.intensidad,
        conciencia_umbral: cerebro.motores.conciencia.umbral,

        // Curiosidad
        curiosidad_nivel: cerebro.motor_curiosidad.nivel,
        curiosidad_umbral: cerebro.motor_curiosidad.umbral,
        curiosidad_saciedad: cerebro.motor_curiosidad.saciedad,
        curiosidad_cadencia_min: cerebro.motor_curiosidad.cadencia_min,
        curiosidad_pasos_desde_ultima: cerebro.motor_curiosidad.pasos_desde_ultima,
        curiosidad_tema_actual: cerebro.motor_curiosidad.tema_actual.clone(),
        curiosidad_busquedas_realizadas: cerebro.motor_curiosidad.busquedas_realizadas,
        curiosidad_decaimiento: cerebro.motor_curiosidad.decaimiento,
        curiosidad_fuentes_navegadas: cerebro.motor_curiosidad.fuentes_navegadas.clone(),
        curiosidad_profundidad_exploracion: cerebro.motor_curiosidad.profundidad_exploracion,
        curiosidad_preferencia_academica: cerebro.motor_curiosidad.preferencia_academica,

        // Pipeline Sensorial
        sensorial_embeddings: cerebro
            .motor_sensorial
            .embeddings
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
        sensorial_token_por_palabra: cerebro.motor_sensorial.token_por_palabra.clone(),
        sensorial_siguiente_token: cerebro.motor_sensorial.siguiente_token,
        sensorial_dimensiones: cerebro.motor_sensorial.dimensiones,
        sensorial_k_sparse: cerebro.motor_sensorial.k_sparse,
        sensorial_tasa_contexto: cerebro.motor_sensorial.tasa_contexto,
        sensorial_ventana_contexto: cerebro.motor_sensorial.ventana_contexto,
        sensorial_base_neurona: cerebro.motor_sensorial.base_neurona,
        sensorial_grupo_por_neurona: cerebro.motor_sensorial.grupo_por_neurona,
        sensorial_rng: cerebro.motor_sensorial.rng,
        sensorial_palabras_procesadas: cerebro.motor_sensorial.palabras_procesadas,

        // Motor 1: Predictor Temporal
        predictor_buffer,
        predictor_capacidad_buffer: cerebro.motor_predictor.capacidad_buffer,
        predictor_memoria_secuencias: cerebro
            .motor_predictor
            .memoria_secuencias
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
        predictor_max_por_bucket: cerebro.motor_predictor.max_por_bucket,
        predictor_ultima_prediccion: cerebro.motor_predictor.ultima_prediccion.clone(),
        predictor_error_prediccion: cerebro.motor_predictor.error_prediccion,
        predictor_secuencias_aprendidas: cerebro.motor_predictor.secuencias_aprendidas,
        predictor_predicciones_acertadas: cerebro.motor_predictor.predicciones_acertadas,
        predictor_total_predicciones: cerebro.motor_predictor.total_predicciones,
        predictor_tasa_acierto: cerebro.motor_predictor.tasa_acierto,

        // Motor 2: Formador de Conceptos
        conceptos_co_ocurrencias: cerebro
            .motor_conceptos
            .co_ocurrencias
            .iter()
            .map(|((a, b), v)| (format!("{a}:{b}"), *v))
            .collect(),
        conceptos_lista: cerebro.motor_conceptos.conceptos.clone(),
        conceptos_umbral_coocurrencia: cerebro.motor_conceptos.umbral_coocurrencia,
        conceptos_ventana_contexto: cerebro.motor_conceptos.ventana_contexto,
        conceptos_paso_actual: cerebro.motor_conceptos.paso_actual,
        conceptos_cadencia_agrupacion: cerebro.motor_conceptos.cadencia_agrupacion,
        conceptos_formados: cerebro.motor_conceptos.conceptos_formados,
        conceptos_tabla_impactos: cerebro
            .motor_conceptos
            .tabla_impactos
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),

        // Motor 3: Neurogénesis
        neurogenesis_frecuencia_tokens: cerebro
            .motor_neurogenesis
            .frecuencia_tokens
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect(),
        neurogenesis_token_a_neuronas: cerebro
            .motor_neurogenesis
            .token_a_neuronas
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
        neurogenesis_cola_conceptos: cerebro.motor_neurogenesis.cola_conceptos.clone().into(),
        neurogenesis_neuronas_creadas: cerebro.motor_neurogenesis.neuronas_creadas.clone(),
        neurogenesis_total_creadas: cerebro.motor_neurogenesis.total_creadas,
        neurogenesis_max_neuronas: cerebro.motor_neurogenesis.max_neuronas,
        neurogenesis_umbral_frecuencia: cerebro.motor_neurogenesis.umbral_frecuencia,
        neurogenesis_ventana_observacion: cerebro.motor_neurogenesis.ventana_observacion,
        neurogenesis_paso_actual: cerebro.motor_neurogenesis.paso_actual,

        // Motor 4: Poda Homeostática
        poda_umbral_peso_min: cerebro.motor_poda.umbral_peso_min,
        poda_max_sinapsis_por_neurona: cerebro.motor_poda.max_sinapsis_por_neurona,
        poda_umbral_frecuencia_min: cerebro.motor_poda.umbral_frecuencia_min,
        poda_ventana_inactividad: cerebro.motor_poda.ventana_inactividad,
        poda_edad_minima: cerebro.motor_poda.edad_minima,
        poda_max_eliminar_por_ciclo: cerebro.motor_poda.max_eliminar_por_ciclo,
        poda_sinapsis_eliminadas: cerebro.motor_poda.sinapsis_eliminadas,
        poda_neuronas_eliminadas: cerebro.motor_poda.neuronas_eliminadas,
        poda_ciclos_poda: cerebro.motor_poda.ciclos_poda,
        poda_paso_actual: cerebro.motor_poda.paso_actual,

        // Motor 5: Consolidador Nocturno
        consolidacion_en_suenio: cerebro.motor_consolidacion.en_suenio,
        consolidacion_pasos_restantes: cerebro.motor_consolidacion.pasos_restantes,
        consolidacion_duracion_suenio: cerebro.motor_consolidacion.duracion_suenio,
        consolidacion_cadencia_suenio: cerebro.motor_consolidacion.cadencia_suenio,
        consolidacion_episodios_a_consolidar: cerebro.motor_consolidacion.episodios_a_consolidar.clone(),
        consolidacion_indice_actual: cerebro.motor_consolidacion.indice_actual,
        consolidacion_pasos_por_episodio: cerebro.motor_consolidacion.pasos_por_episodio,
        consolidacion_paso_en_episodio: cerebro.motor_consolidacion.paso_en_episodio,
        consolidacion_meta_episodios: cerebro.motor_consolidacion.meta_episodios.clone(),
        consolidacion_ciclos_completados: cerebro.motor_consolidacion.ciclos_completados,
        consolidacion_episodios_consolidados: cerebro.motor_consolidacion.episodios_consolidados,
        consolidacion_paso_actual: cerebro.motor_consolidacion.paso_actual,

        // Episodios
        episodios: cerebro.memoria.ssd.episodios.clone(),
        episodios_capacidad_max: cerebro.memoria.ssd.capacidad_maxima,

        // Contadores
        paso_actual: cerebro.paso_actual,
        tiempo: cerebro.tiempo,
        siguiente_id: cerebro.siguiente_id,
        historial_emocional: cerebro.historial_emocional.clone(),
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::cerebro::CerebroAutoOptimizable;

    /// Ruta temporal única por test para evitar colisiones en paralelo.
    fn ruta_tmp(nombre: &str) -> String {
        let dir = std::env::temp_dir().join(format!(
            "nexus_persistencia_{}_{}",
            std::process::id(),
            nombre
        ));
        // Limpiar cualquier residuo de una ejecución anterior
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir.join("estado.json").to_string_lossy().to_string()
    }

    fn limpiar(ruta: &str) {
        if let Some(parent) = Path::new(ruta).parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    /// Crea un cerebro con modificaciones deterministas en campos aprendidos.
    fn cerebro_modificado() -> CerebroAutoOptimizable {
        let mut c = CerebroAutoOptimizable::nuevo();
        // Resetear el Motor Sensorial a baseline (58 tokens innatos) para que el
        // test sea determinista e independiente del estado persistido en disco,
        // que otros tests de integración (paso/paso_tutor) pueden haber escrito.
        c.motor_sensorial = MotorSensorial::nuevo();
        // Emociones
        c.motores.amigdala.miedo = 0.3;
        c.motores.amigdala.ansiedad = 0.4;
        c.motores.amigdala.ira = 0.1;
        c.motores.amigdala.alegria = 0.8;
        // Dopamina
        c.motores.dopamina.nivel = 0.55;
        c.motores.dopamina.prediccion = 0.22;
        // Conciencia
        c.motores.conciencia.intensidad = 0.66;
        c.motores.conciencia.umbral = 0.31;
        // Curiosidad
        c.motor_curiosidad.nivel = 0.7;
        c.motor_curiosidad.tema_actual = "física cuántica".to_string();
        c.motor_curiosidad.busquedas_realizadas = 12;
        c.motor_curiosidad.fuentes_navegadas = vec!["arxiv".to_string()];
        // Contadores
        c.paso_actual = 1234;
        c.tiempo = 12.34;
        c.siguiente_id = 777;
        c.historial_emocional = vec![0.1, 0.2, 0.3];
        // Sensorial: aprender un token propio
        let _tok = c.motor_sensorial.token_para("neocortex");
        c
    }

    // ─── Guardar / Cargar (roundtrip en disco) ──────────────────────────────

    #[test]
    fn test_guardar_y_cargar_roundtrip() {
        let ruta = ruta_tmp("roundtrip");
        {
            let c = cerebro_modificado();
            guardar(&c, &ruta).expect("debe guardar");
        }

        let estado = cargar(&ruta).expect("debe cargar");
        assert_eq!(estado.paso_actual, 1234);
        assert_eq!(estado.tiempo, 12.34);
        assert_eq!(estado.siguiente_id, 777);
        assert_eq!(estado.historial_emocional, vec![0.1, 0.2, 0.3]);

        // Emociones
        assert_eq!(estado.miedo, 0.3);
        assert_eq!(estado.ansiedad, 0.4);
        assert_eq!(estado.ira, 0.1);
        assert_eq!(estado.alegria, 0.8);
        // Dopamina
        assert_eq!(estado.dopamina_nivel, 0.55);
        assert_eq!(estado.dopamina_prediccion, 0.22);
        // Conciencia
        assert_eq!(estado.conciencia_intensidad, 0.66);
        assert_eq!(estado.conciencia_umbral, 0.31);
        // Curiosidad
        assert_eq!(estado.curiosidad_nivel, 0.7);
        assert_eq!(estado.curiosidad_tema_actual, "física cuántica");
        assert_eq!(estado.curiosidad_busquedas_realizadas, 12);
        assert_eq!(estado.curiosidad_fuentes_navegadas, vec!["arxiv"]);

        limpiar(&ruta);
    }

    #[test]
    fn test_guardar_crea_directorio_anidado() {
        // Ruta en subdirectorio inexistente
        let dir = std::env::temp_dir().join(format!(
            "nexus_persistencia_{}_anidado/sub/deep",
            std::process::id()
        ));
        let ruta = dir.join("cerebro.json").to_string_lossy().to_string();
        let c = cerebro_modificado();
        guardar(&c, &ruta).expect("debe crear directorios y guardar");
        assert!(Path::new(&ruta).exists(), "archivo debe existir");

        let _ = fs::remove_dir_all(
            std::env::temp_dir().join(format!("nexus_persistencia_{}_anidado", std::process::id())),
        );
    }

    #[test]
    fn test_guardar_es_atomico_sin_tmp_residual() {
        let ruta = ruta_tmp("atomico");
        let c = cerebro_modificado();
        guardar(&c, &ruta).expect("debe guardar");
        // Tras el rename, no debe quedar el archivo .tmp
        let ruta_tmp = format!("{}.tmp", ruta);
        assert!(!Path::new(&ruta_tmp).exists(), "no debe quedar .tmp residual");
        limpiar(&ruta);
    }

    // ─── Errores ────────────────────────────────────────────────────────────

    #[test]
    fn test_cargar_archivo_inexistente_retorna_error() {
        let ruta = std::env::temp_dir()
            .join(format!("nexus_persistencia_{}_no_existe.json", std::process::id()))
            .to_string_lossy()
            .to_string();
        let _ = fs::remove_file(&ruta);
        let err = match cargar(&ruta) {
            Err(e) => e,
            Ok(_) => panic!("debe fallar para archivo inexistente"),
        };
        assert!(err.contains("Archivo no encontrado"), "error fue: {}", err);
        let _ = fs::remove_file(&ruta);
    }

    #[test]
    fn test_cargar_json_corrupto_retorna_error() {
        let ruta = ruta_tmp("corrupto");
        fs::write(&ruta, "{ esto no es json valido").unwrap();
        let err = match cargar(&ruta) {
            Err(e) => e,
            Ok(_) => panic!("debe fallar para JSON corrupto"),
        };
        assert!(err.contains("Error parseando"), "error fue: {}", err);
        limpiar(&ruta);
    }

    // ─── Snapshot (tomar_snapshot fiel al cerebro) ──────────────────────────

    #[test]
    fn test_snapshot_captura_estado_completo() {
        let c = cerebro_modificado();
        let snap = tomar_snapshot(&c);

        assert_eq!(snap.miedo, 0.3);
        assert_eq!(snap.dopamina_nivel, 0.55);
        assert_eq!(snap.curiosidad_nivel, 0.7);
        assert_eq!(snap.curiosidad_tema_actual, "física cuántica");
        assert_eq!(snap.paso_actual, 1234);
        assert_eq!(snap.tiempo, 12.34);
        assert_eq!(snap.siguiente_id, 777);
        assert_eq!(snap.historial_emocional, vec![0.1, 0.2, 0.3]);
        // Sensorial: el token aprendido debe persistir.
        // 58 tokens innatos (0..57) + "neocortex" (token 58) → siguiente 59
        assert!(snap.sensorial_token_por_palabra.contains_key("neocortex"));
        assert_eq!(snap.sensorial_siguiente_token, 59);
    }

    // ─── Restaurar ──────────────────────────────────────────────────────────

    #[test]
    fn test_restaurar_aplica_estado_guardado() {
        let ruta = ruta_tmp("restaurar");
        // Cerebro fuente
        {
            let c = cerebro_modificado();
            guardar(&c, &ruta).unwrap();
        }
        let estado = cargar(&ruta).unwrap();

        // Cerebro destino (recién creado, desde cero)
        let mut destino = CerebroAutoOptimizable::nuevo();
        restaurar(&mut destino, estado);

        // Emociones
        assert_eq!(destino.motores.amigdala.miedo, 0.3);
        assert_eq!(destino.motores.amigdala.alegria, 0.8);
        // Dopamina
        assert_eq!(destino.motores.dopamina.nivel, 0.55);
        assert_eq!(destino.motores.dopamina.prediccion, 0.22);
        // Conciencia
        assert_eq!(destino.motores.conciencia.intensidad, 0.66);
        // Curiosidad
        assert_eq!(destino.motor_curiosidad.tema_actual, "física cuántica");
        assert_eq!(destino.motor_curiosidad.busquedas_realizadas, 12);
        assert_eq!(
            destino.motor_curiosidad.fuentes_navegadas,
            vec!["arxiv"]
        );
        // Contadores
        assert_eq!(destino.paso_actual, 1234);
        assert_eq!(destino.tiempo, 12.34);
        assert_eq!(destino.siguiente_id, 777);
        assert_eq!(destino.historial_emocional, vec![0.1, 0.2, 0.3]);
        // Sensorial: el token aprendido persiste y puede reutilizarse
        assert!(destino.motor_sensorial.token_por_palabra.contains_key("neocortex"));

        limpiar(&ruta);
    }

    #[test]
    fn test_restaurar_con_estado_vacio_conserva_defaults() {
        // Estado con valores en cero (simula archivos antiguos) no debe dejar
        // el siguiente_token en 0 (el restaurar lo clampa a mínimo 1)
        let ruta = ruta_tmp("vacio");
        {
            let c = CerebroAutoOptimizable::nuevo();
            let mut estado = tomar_snapshot(&c);
            estado.sensorial_siguiente_token = 0;
            let json = serde_json::to_string(&estado).unwrap();
            fs::write(&ruta, json).unwrap();
        }
        let estado = cargar(&ruta).unwrap();
        let mut destino = CerebroAutoOptimizable::nuevo();
        restaurar(&mut destino, estado);
        assert!(
            destino.motor_sensorial.siguiente_token >= 1,
            "siguiente_token debe quedar >= 1"
        );
        limpiar(&ruta);
    }
}
