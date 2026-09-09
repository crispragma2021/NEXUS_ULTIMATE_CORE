use std::collections::{HashMap, VecDeque};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ============================================================================
// MOTOR 1: PREDICTOR TEMPORAL — Aprende a anticipar patrones neuronales
// ============================================================================
// Mantiene un buffer circular de los últimos 32 estados. Cada estado contiene
// las top-64 neuronas activas. Cuando el buffer tiene suficientes entradas (16),
// busca en el historial secuencias que empiecen igual y predice la siguiente.

pub struct MotorPrediccion {
    /// Buffer circular de últimos estados: Vec<(neurona_id, activacion)>
    pub buffer: VecDeque<Vec<(u32, f32)>>,

    /// Capacidad máxima del buffer
    pub capacidad_buffer: usize,  // 32

    /// Historial de secuencias: hash(prefijo) → posibles continuaciones
    pub memoria_secuencias: HashMap<u64, Vec<Vec<(u32, f32)>>>,

    /// Máximo de secuencias por bucket
    pub max_por_bucket: usize,  // 100

    /// Última predicción realizada
    pub ultima_prediccion: Vec<(u32, f32)>,

    /// Error de la última predicción
    pub error_prediccion: f32,

    /// Contador de secuencias aprendidas
    pub secuencias_aprendidas: u64,

    /// Número de predicciones acertadas (error < umbral 0.15)
    pub predicciones_acertadas: u64,

    /// Total de predicciones realizadas
    pub total_predicciones: u64,

    /// Tasa de acierto (métrica interna)
    pub tasa_acierto: f32,
}

impl MotorPrediccion {
    pub fn nuevo() -> Self {
        Self {
            buffer: VecDeque::with_capacity(32),
            capacidad_buffer: 32,
            memoria_secuencias: HashMap::new(),
            max_por_bucket: 100,
            ultima_prediccion: Vec::new(),
            error_prediccion: 0.0,
            secuencias_aprendidas: 0,
            predicciones_acertadas: 0,
            total_predicciones: 0,
            tasa_acierto: 0.0,
        }
    }

    /// Registra un nuevo estado neuronal en el buffer
    pub fn registrar_estado(&mut self, actividad: &[(u32, f32)]) {
        // Limitar a top-64 por activación
        let mut top: Vec<(u32, f32)> = actividad.to_vec();
        top.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        top.truncate(64);

        self.buffer.push_back(top);

        // Mantener tamaño del buffer
        while self.buffer.len() > self.capacidad_buffer {
            self.buffer.pop_front();
        }
    }

    /// Predice el siguiente estado basado en el buffer actual
    /// Retorna None si no hay suficientes datos (mínimo 16 entradas para prefijo)
    pub fn predecir(&self) -> Option<Vec<(u32, f32)>> {
        if self.buffer.len() < 16 {
            return None;
        }

        // Tomar prefijo: primeras 16 entradas del buffer
        let prefijo: Vec<&[(u32, f32)]> = self.buffer.iter().take(16).map(|v| v.as_slice()).collect();
        let hash_prefijo = self.hash_prefijo(&prefijo);

        // Buscar en memoria de secuencias
        if let Some(continuaciones) = self.memoria_secuencias.get(&hash_prefijo) {
            if continuaciones.is_empty() {
                return None;
            }

            // Promediar todas las continuaciones
            let mut prediccion: HashMap<u32, (f32, usize)> = HashMap::new();
            for cont in continuaciones {
                for &(id, act) in cont {
                    let entry = prediccion.entry(id).or_insert((0.0, 0));
                    entry.0 += act;
                    entry.1 += 1;
                }
            }

            let result: Vec<(u32, f32)> = prediccion
                .into_iter()
                .map(|(id, (sum, count))| (id, sum / count as f32))
                .filter(|(_, act)| *act > 0.05)
                .collect();

            if result.is_empty() {
                None
            } else {
                Some(result)
            }
        } else {
            None
        }
    }

    /// Calcula el error normalizado entre predicción y estado real (0.0 = perfecto, 1.0 = totalmente errada)
    pub fn calcular_error(&self, estado_real: &[(u32, f32)]) -> f32 {
        if self.ultima_prediccion.is_empty() || estado_real.is_empty() {
            return 1.0;
        }

        // Crear mapa de activación real para lookup rápido
        let real_map: HashMap<u32, f32> = estado_real.iter().map(|&(id, a)| (id, a)).collect();

        let mut error_total = 0.0;
        let max_neurons = self.ultima_prediccion.len().max(estado_real.len());

        for &(id, pred_act) in &self.ultima_prediccion {
            let real_act = real_map.get(&id).copied().unwrap_or(0.0);
            error_total += (pred_act - real_act).abs();
        }

        // Penalizar neuronas en real que no estaban en predicción
        let pred_set: std::collections::HashSet<u32> = self.ultima_prediccion.iter().map(|&(id, _)| id).collect();
        for &(id, real_act) in estado_real {
            if !pred_set.contains(&id) {
                error_total += real_act;
            }
        }

        if max_neurons == 0 {
            return 1.0;
        }

        (error_total / max_neurons as f32).min(1.0)
    }

    /// Actualiza la memoria de secuencias con el estado real (aprendizaje)
    pub fn aprender(&mut self, estado_real: &[(u32, f32)]) {
        if self.buffer.len() < 16 {
            return;
        }

        let prefijo: Vec<&[(u32, f32)]> = self.buffer.iter().take(16).map(|v| v.as_slice()).collect();
        let hash_prefijo = self.hash_prefijo(&prefijo);

        let continuacion: Vec<(u32, f32)> = estado_real
            .iter()
            .map(|&(id, a)| (id, a))
            .collect();

        let entry = self.memoria_secuencias.entry(hash_prefijo).or_insert_with(Vec::new);

        // LRU eviction
        if entry.len() >= self.max_por_bucket {
            entry.remove(0);
        }

        entry.push(continuacion);
        self.secuencias_aprendidas += 1;
    }

    /// Procesa un ciclo completo: registra, predice, calcula error, aprende
    /// Retorna el error de predicción
    pub fn procesar_ciclo(&mut self, estado_real: &[(u32, f32)]) -> f32 {
        self.registrar_estado(estado_real);

        let error = if let Some(prediccion) = self.predecir() {
            self.ultima_prediccion = prediccion;
            let err = self.calcular_error(estado_real);
            self.total_predicciones += 1;
            if err < 0.15 {
                self.predicciones_acertadas += 1;
            }
            self.aprender(estado_real);
            err
        } else {
            self.aprender(estado_real);
            0.5 // error medio cuando no hay suficiente historial
        };

        self.error_prediccion = error;

        // Actualizar tasa de acierto
        if self.total_predicciones > 0 {
            self.tasa_acierto = self.predicciones_acertadas as f32 / self.total_predicciones as f32;
        }

        error
    }

    /// Aplica el error de predicción al SistemaDopamina
    pub fn error_dopamina(&self) -> f32 {
        // Error alto → novedad → más dopamina
        // Error bajo → predictable → menos dopamina
        self.error_prediccion * 2.0 // escalar para que sea significativo
    }

    /// Estadísticas
    pub fn estadisticas(&self) -> (u64, u64, u64, f32) {
        (self.secuencias_aprendidas, self.total_predicciones, self.predicciones_acertadas, self.tasa_acierto)
    }

    /// Hashea el prefijo para usarlo como clave en la memoria de secuencias
    fn hash_prefijo(&self, prefijo: &[&[(u32, f32)]]) -> u64 {
        let mut hasher = DefaultHasher::new();
        for estado in prefijo {
            // Hash de los IDs de las neuronas activas
            let ids: Vec<u32> = estado.iter().map(|&(id, _)| id).collect();
            ids.hash(&mut hasher);
        }
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn estado_ejemplo(base: u32, n: usize) -> Vec<(u32, f32)> {
        (0..n).map(|i| (base + i as u32, 0.5 + (i as f32 * 0.01))).collect()
    }

    #[test]
    fn test_buffer_circular() {
        let mut p = MotorPrediccion::nuevo();
        for i in 0..40 {
            p.registrar_estado(&estado_ejemplo(i * 10, 10));
        }
        assert_eq!(p.buffer.len(), 32, "Buffer debe tener máximo 32 entradas");
    }

    #[test]
    fn test_predecir_sin_datos() {
        let p = MotorPrediccion::nuevo();
        assert!(p.predecir().is_none(), "Sin suficientes datos debe retornar None");
    }

    #[test]
    fn test_hash_prefijo_distintos() {
        // Distintos vectores producen distintos hashes
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        vec![1u32, 2u32, 3u32].hash(&mut hasher1);
        vec![4u32, 5u32, 6u32].hash(&mut hasher2);
        assert_ne!(hasher1.finish(), hasher2.finish(), "Distintos prefijos deben producir distintos hashes");
    }

    #[test]
    fn test_error_perfecto() {
        let mut p = MotorPrediccion::nuevo();
        let real = estado_ejemplo(0, 32);
        p.ultima_prediccion = real.clone();
        let error = p.calcular_error(&real);
        assert!(error < 0.01, "Predicción exacta debe tener error ~0, obtuvo {}", error);
    }

    #[test]
    fn test_error_total() {
        let mut p = MotorPrediccion::nuevo();
        p.ultima_prediccion = estado_ejemplo(0, 32);
        let real = estado_ejemplo(1000, 32); // completamente diferente
        let error = p.calcular_error(&real);
        assert!(error > 0.5, "Predicción opuesta debe tener error alto, obtuvo {}", error);
    }

    #[test]
    fn test_procesar_ciclo_completo() {
        let mut p = MotorPrediccion::nuevo();
        // Simular 20 ciclos de la misma secuencia
        for _ in 0..20 {
            let estado = estado_ejemplo(42, 32);
            p.procesar_ciclo(&estado);
        }
        // Después de 20 repeticiones, debería tener predicciones y error bajo
        assert!(p.total_predicciones > 0, "Debe haber predicciones");
        assert!(p.secuencias_aprendidas > 0, "Debe haber aprendizaje");
    }

    #[test]
    fn test_tasa_acierto() {
        let mut p = MotorPrediccion::nuevo();
        // Simular 20 ciclos (necesita >=16 para activar predicciones)
        for _ in 0..20 {
            let estado = estado_ejemplo(99, 32);
            p.procesar_ciclo(&estado);
        }
        assert!(p.tasa_acierto > 0.0 && p.tasa_acierto <= 1.0, "Tasa de acierto debe estar entre 0 y 1, obtuvo {}", p.tasa_acierto);
    }

    #[test]
    fn test_estadisticas() {
        let mut p = MotorPrediccion::nuevo();
        for _ in 0..20 {
            let estado = estado_ejemplo(77, 32);
            p.procesar_ciclo(&estado);
        }
        let (sec, total, aciertos, tasa) = p.estadisticas();
        assert!(sec > 0, "Debe haber secuencias aprendidas, obtuvo {}", sec);
        assert_eq!(total, aciertos + (total - aciertos), "Total debe ser suma de aciertos + fallos");
        assert!(tasa >= 0.0 && tasa <= 1.0);
    }
}
