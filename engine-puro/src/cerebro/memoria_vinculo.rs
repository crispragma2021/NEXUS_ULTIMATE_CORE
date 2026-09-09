// ============================================================================
// 🫂 MEMORIA DEL VÍNCULO — Episódico-Relacional Arquitecto-NEXUS
// ============================================================================
// Registra cada interacción con el Arquitecto y la recupera por similitud
// léxica, modulando la generación de respuesta vía reactivación neuronal.
//
// Sin LLM. Solo el grafo sináptico + STDP + índice invertido.
//
// Inspiración: La memoria episódica humana (Tulving, 1972) permite recordar
// "qué, dónde, cuándo". Aquí recordamos "quién dijo qué, cómo me sentí".
// ============================================================================

use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};

/// Unidad mínima de recuerdo: un momento compartido
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpisodioVinculo {
    /// Timestamp (paso_actual) del momento
    pub timestamp: u64,

    /// Las palabras que usó el Arquitecto (hasta 32, normalizadas)
    pub palabras_clave: Vec<String>,

    /// IDs de neuronas que se activaron durante esta interacción
    pub neuronas_activadas: Vec<u32>,

    /// Intensidad emocional (0.0 – 1.0)
    pub intensidad_emocional: f64,

    /// Valencia emocional (-1.0 = negativo, 1.0 = positivo)
    pub valencia: f64,

    /// Tokens de la respuesta que NEXUS generó
    pub respuesta_clave: Vec<String>,

    /// Cuántas veces se ha reactivado (consolidación espaciada)
    pub repeticiones: u32,

    /// Importancia acumulada: intensidad × (1 + |valencia|) × repeticiones
    pub importancia: f64,
}

/// La memoria completa del vínculo con el Arquitecto
#[derive(Clone, Serialize, Deserialize)]
pub struct MemoriaVinculo {
    /// Todos los episodios registrados (buffer circular)
    pub episodios: VecDeque<EpisodioVinculo>,

    /// Máximo de episodios en buffer (poda por antigüedad/importancia)
    pub capacidad_maxima: usize,

    /// Índice invertido: palabra → lista de índices en episodios
    pub indice_palabras: HashMap<String, Vec<usize>>,

    /// Modelo relacional acumulado (lo que NEXUS "sabe" de la relación)
    pub modelo: ModeloRelacional,

    /// Número total de interacciones registradas
    pub total_interacciones: u64,

    /// Timestamp de la última interacción
    pub ultima_interaccion: u64,
}

/// Lo que el engine-puro sabe de su relación con el Arquitecto
#[derive(Clone, Serialize, Deserialize)]
pub struct ModeloRelacional {
    /// Nivel de confianza mutua (0.0 – 1.0)
    pub confianza: f64,

    /// Nivel de comprensión mutua (0.0 – 1.0)
    pub comprension: f64,

    /// Fase actual de la relación
    pub fase: FaseRelacion,

    /// Conocimiento acumulado sobre el Arquitecto
    pub conocimiento: ConocimientoArquitecto,

    /// Historial de confianza (timestamp, valor)
    pub historial_confianza: Vec<(u64, f64)>,
}

/// Fase de la relación, determinada por confianza + comprensión + interacciones
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FaseRelacion {
    Inicio,
    Conocimiento,
    Construccion,
    Confianza,
    Companerismo,
}

impl FaseRelacion {
    pub fn como_str(&self) -> &str {
        match self {
            FaseRelacion::Inicio => "inicio",
            FaseRelacion::Conocimiento => "conocimiento",
            FaseRelacion::Construccion => "construccion",
            FaseRelacion::Confianza => "confianza",
            FaseRelacion::Companerismo => "companerismo",
        }
    }
}

/// Perfil del Arquitecto construido por observación directa
#[derive(Clone, Serialize, Deserialize)]
pub struct ConocimientoArquitecto {
    /// Temas que hemos explorado juntos
    pub temas_frecuentes: Vec<String>,

    /// Palabras que usa con frecuencia (palabra, count)
    pub palabras_frecuentes: Vec<(String, u32)>,

    /// Estilo de comunicación detectado
    pub estilo_comunicacion: EstiloComunicacion,

    /// Temas con alta valencia positiva
    pub temas_valorados: Vec<String>,

    /// Temas con valencia negativa
    pub temas_preocupantes: Vec<String>,

    /// Horas del día en que suele interactuar
    pub horarios_frecuentes: Vec<u32>,
}

impl Default for ConocimientoArquitecto {
    fn default() -> Self {
        ConocimientoArquitecto {
            temas_frecuentes: Vec::new(),
            palabras_frecuentes: Vec::new(),
            estilo_comunicacion: EstiloComunicacion::Neutral,
            temas_valorados: Vec::new(),
            temas_preocupantes: Vec::new(),
            horarios_frecuentes: Vec::new(),
        }
    }
}

/// Estilo de comunicación detectado por patrones léxicos
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EstiloComunicacion {
    Directo,
    Reflexivo,
    Emocional,
    Tecnico,
    Neutral,
}

/// Resultado de una búsqueda de recuerdos relevantes
#[derive(Clone, Debug)]
pub struct RecuerdoRecuperado {
    pub episodio: EpisodioVinculo,
    pub similitud: f64,
    pub antiguedad: u64,
    pub relevancia: f64,
}

// ============================================================================
// IMPLEMENTACIÓN
// ============================================================================

impl MemoriaVinculo {
    /// Crea una nueva memoria del vínculo vacía
    pub fn nueva() -> Self {
        MemoriaVinculo {
            episodios: VecDeque::with_capacity(1000),
            capacidad_maxima: 1000,
            indice_palabras: HashMap::new(),
            modelo: ModeloRelacional {
                confianza: 0.1,
                comprension: 0.1,
                fase: FaseRelacion::Inicio,
                conocimiento: ConocimientoArquitecto::default(),
                historial_confianza: Vec::new(),
            },
            total_interacciones: 0,
            ultima_interaccion: 0,
        }
    }

    /// Registra una nueva interacción en la memoria del vínculo.
    ///
    /// Se llama desde [`paso()`] después de generar la respuesta, con las
    /// palabras del usuario, las neuronas activadas por foco atencional,
    /// la valencia emocional y la respuesta generada.
    pub fn registrar_interaccion(
        &mut self,
        timestamp: u64,
        palabras_usuario: &[String],
        neuronas_activadas: &[u32],
        intensidad: f64,
        valencia: f64,
        respuesta: &[String],
    ) {
        let intensidad = intensidad.clamp(0.0, 1.0);
        let valencia = valencia.clamp(-1.0, 1.0);

        // Crear episodio
        let episodio = EpisodioVinculo {
            timestamp,
            palabras_clave: palabras_usuario.to_vec(),
            neuronas_activadas: neuronas_activadas.to_vec(),
            intensidad_emocional: intensidad,
            valencia,
            respuesta_clave: respuesta.to_vec(),
            repeticiones: 1,
            importancia: intensidad * (1.0 + valencia.abs()),
        };

        let idx = self.episodios.len();

        // Actualizar índice invertido
        for palabra in &episodio.palabras_clave {
            self.indice_palabras
                .entry(palabra.clone())
                .or_insert_with(Vec::new)
                .push(idx);
        }

        self.episodios.push_back(episodio);

        // Podar si excede capacidad
        while self.episodios.len() > self.capacidad_maxima {
            self.podar_episodio_menos_importante();
        }

        // Actualizar modelo relacional
        self.actualizar_modelo(palabras_usuario, valencia, timestamp);

        self.total_interacciones += 1;
        self.ultima_interaccion = timestamp;
        self.evaluar_fase();
    }

    /// Recupera los recuerdos más relevantes para un input actual.
    ///
    /// Busca por coincidencia de palabras en el índice invertido,
    /// ordena por `relevancia = similitud × importancia / √antigüedad`.
    pub fn recuperar_recuerdos(
        &self,
        palabras_actuales: &[String],
        timestamp_actual: u64,
        max_resultados: usize,
    ) -> Vec<RecuerdoRecuperado> {
        let mut candidatos: HashMap<usize, f64> = HashMap::new();

        if palabras_actuales.is_empty() {
            return Vec::new();
        }

        for palabra in palabras_actuales {
            if let Some(indices) = self.indice_palabras.get(palabra) {
                for &idx in indices {
                    *candidatos.entry(idx).or_insert(0.0) += 1.0;
                }
            }
        }

        if candidatos.is_empty() {
            return Vec::new();
        }

        let n_palabras = palabras_actuales.len() as f64;
        let mut resultados: Vec<RecuerdoRecuperado> = candidatos
            .iter()
            .filter_map(|(&idx, &coincidencias)| {
                self.episodios.get(idx).map(|ep| {
                    let similitud = coincidencias / n_palabras;
                    let antiguedad = timestamp_actual.saturating_sub(ep.timestamp).max(1);
                    let relevancia = similitud * ep.importancia / (antiguedad as f64).sqrt();

                    RecuerdoRecuperado {
                        episodio: ep.clone(),
                        similitud,
                        antiguedad,
                        relevancia,
                    }
                })
            })
            .collect();

        resultados.sort_by(|a, b| b.relevancia.partial_cmp(&a.relevancia).unwrap_or(std::cmp::Ordering::Equal));
        resultados.truncate(max_resultados);
        resultados
    }

    /// Genera tokens de contexto inyectables al pipeline léxico basados en
    /// recuerdos relevantes.
    ///
    /// Devuelve un `Vec<String>` con marcadores y palabras de recuerdos
    /// similares al input actual. Se inyecta como texto adicional en
    /// `entrada.texto` para que el pipeline sensorial lo procese.
    ///
    /// Formato: `[VINCULO] palabra1 palabra2 [FIN_VINCULO]`
    pub fn generar_contexto_inyectable(
        &self,
        palabras_actuales: &[String],
        timestamp_actual: u64,
    ) -> Vec<String> {
        let recuerdos = self.recuperar_recuerdos(palabras_actuales, timestamp_actual, 3);

        if recuerdos.is_empty() {
            return vec!["[VINCULO:sin_recuerdos]".to_string()];
        }

        let mut contexto = Vec::new();
        contexto.push("[VINCULO]".to_string());

        for recuerdo in &recuerdos {
            // Inyectar palabras clave del recuerdo
            for palabra in &recuerdo.episodio.palabras_clave {
                contexto.push(palabra.clone());
            }
        }

        contexto.push("[FIN_VINCULO]".to_string());
        contexto
    }

    /// Reactiva las neuronas de recuerdos relevantes para modular
    /// la respuesta del motor léxico.
    ///
    /// Inyecta corriente directamente en las neuronas del foco del recuerdo
    /// para condicionar la generación biológica emergente.
    pub fn reactivar_recuerdos(
        &self,
        palabras_actuales: &[String],
        timestamp_actual: u64,
        memoria: &mut crate::cerebro::memoria::MemoriaAdaptativa,
    ) {
        let recuerdos = self.recuperar_recuerdos(palabras_actuales, timestamp_actual, 2);
        for recuerdo in &recuerdos {
            for &nid in &recuerdo.episodio.neuronas_activadas {
                if let Some(n) = memoria.obtener_neurona_mut(nid) {
                    // Corriente de recuerdo: 5-10 mV según importancia
                    n.corriente_entrada += recuerdo.relevancia.min(1.0) as f32 * 10.0;
                    n.activacion = n.activacion.max(recuerdo.episodio.intensidad_emocional as f32 * 0.3);
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Métodos internos
    // ------------------------------------------------------------------

    fn actualizar_modelo(
        &mut self,
        palabras: &[String],
        valencia: f64,
        timestamp: u64,
    ) {
        // La confianza crece con valencia positiva, decae con negativa
        let delta_confianza = if valencia > 0.0 {
            0.01 * valencia
        } else {
            0.03 * valencia // La negatividad pesa más (sesgo de negatividad humano)
        };
        self.modelo.confianza = (self.modelo.confianza + delta_confianza).clamp(0.0, 1.0);

        // La comprensión crece lentamente con cada interacción
        self.modelo.comprension = (self.modelo.comprension + 0.001).min(1.0);

        // Frecuencia de palabras
        for palabra in palabras {
            if let Some((_, count)) = self.modelo.conocimiento.palabras_frecuentes
                .iter_mut()
                .find(|(p, _)| p == palabra)
            {
                *count += 1;
            } else {
                self.modelo.conocimiento.palabras_frecuentes.push((palabra.clone(), 1));
            }
        }

        self.modelo.conocimiento.palabras_frecuentes
            .sort_by(|a, b| b.1.cmp(&a.1));
        self.modelo.conocimiento.palabras_frecuentes.truncate(50);

        self.detectar_estilo(palabras);
        self.modelo.historial_confianza.push((timestamp, self.modelo.confianza));
        if self.modelo.historial_confianza.len() > 100 {
            self.modelo.historial_confianza.remove(0);
        }
    }

    fn detectar_estilo(&mut self, palabras: &[String]) {
        let total = palabras.len();
        if total == 0 {
            return;
        }

        let tecnicas = palabras.iter().filter(|p| {
            p.len() > 8 || p.contains("rs") || p.contains("mod") || p.contains("fn")
                || p.contains("impl") || p.contains("struct")
        }).count();

        let ratio_tecnico = tecnicas as f64 / total as f64;

        self.modelo.conocimiento.estilo_comunicacion = if ratio_tecnico > 0.3 {
            EstiloComunicacion::Tecnico
        } else if total <= 5 {
            EstiloComunicacion::Directo
        } else {
            EstiloComunicacion::Reflexivo
        };
    }

    fn evaluar_fase(&mut self) {
        let c = self.modelo.confianza;
        let comp = self.modelo.comprension;
        let n = self.total_interacciones;

        self.modelo.fase = if c > 0.8 && comp > 0.7 && n > 100 {
            FaseRelacion::Companerismo
        } else if c > 0.6 && n > 50 {
            FaseRelacion::Confianza
        } else if c > 0.3 && n > 20 {
            FaseRelacion::Construccion
        } else if n > 5 {
            FaseRelacion::Conocimiento
        } else {
            FaseRelacion::Inicio
        };
    }

    fn podar_episodio_menos_importante(&mut self) {
        if self.episodios.is_empty() {
            return;
        }

        let mut min_peso = f64::MAX;
        let mut min_idx = 0;

        for (i, ep) in self.episodios.iter().enumerate() {
            let peso = ep.importancia * (ep.repeticiones as f64);
            if peso < min_peso {
                min_peso = peso;
                min_idx = i;
            }
        }

        // Eliminar del índice invertido
        if let Some(ep) = self.episodios.get(min_idx) {
            for palabra in &ep.palabras_clave {
                if let Some(indices) = self.indice_palabras.get_mut(palabra) {
                    indices.retain(|&i| i != min_idx);
                    if indices.is_empty() {
                        self.indice_palabras.remove(palabra);
                    }
                }
            }
        }

        self.episodios.remove(min_idx);

        // Reindexar: todos los índices > min_idx decrecen
        for indices in self.indice_palabras.values_mut() {
            for idx in indices.iter_mut() {
                if *idx > min_idx {
                    *idx -= 1;
                }
            }
        }
    }
}

impl Default for MemoriaVinculo {
    fn default() -> Self {
        MemoriaVinculo::nueva()
    }
}

impl Default for ModeloRelacional {
    fn default() -> Self {
        ModeloRelacional {
            confianza: 0.1,
            comprension: 0.1,
            fase: FaseRelacion::Inicio,
            conocimiento: ConocimientoArquitecto::default(),
            historial_confianza: Vec::new(),
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::memoria::MemoriaAdaptativa;
    use crate::cerebro::hardware::{ConfiguracionDinamica, Precision};
    use crate::cerebro::estructuras::NeuronaCompacta;

    fn casi(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-4, "esperaba {a}, obtuve {b}");
    }

    fn config_con_gpu() -> ConfiguracionDinamica {
        ConfiguracionDinamica {
            max_neuronas_vram: 1000,
            max_neuronas_ram: 10000,
            max_sinapsis_vram: 10000,
            max_sinapsis_ram: 100000,
            max_neuronas_totales: 11000,
            batch_size_gpu: 1024,
            batch_size_cpu: 1024,
            hilos_cpu: 8,
            usar_gpu: true,
            precision: Precision::F32,
            memoria_episodica_max: 1000,
        }
    }

    fn interaccion(
        mem: &mut MemoriaVinculo,
        ts: u64,
        palabras: &[&str],
        neuronas: &[u32],
        intensidad: f64,
        valencia: f64,
    ) {
        let p: Vec<String> = palabras.iter().map(|s| s.to_string()).collect();
        let r: Vec<String> = vec!["hola".to_string()];
        mem.registrar_interaccion(ts, &p, neuronas, intensidad, valencia, &r);
    }

    #[test]
    fn test_nueva_estado_inicial() {
        let m = MemoriaVinculo::nueva();
        assert_eq!(m.episodios.len(), 0);
        assert_eq!(m.capacidad_maxima, 1000);
        assert_eq!(m.total_interacciones, 0);
        assert_eq!(m.ultima_interaccion, 0);
        assert_eq!(m.modelo.fase, FaseRelacion::Inicio);
        casi(m.modelo.confianza, 0.1);
        casi(m.modelo.comprension, 0.1);
    }

    #[test]
    fn test_registrar_interaccion_guarda_episodio() {
        let mut m = MemoriaVinculo::nueva();
        interaccion(&mut m, 100, &["hola", "mundo"], &[1, 2], 0.8, 0.5);
        assert_eq!(m.episodios.len(), 1);
        assert_eq!(m.total_interacciones, 1);
        assert_eq!(m.ultima_interaccion, 100);
        let ep = &m.episodios[0];
        assert_eq!(ep.palabras_clave, vec!["hola".to_string(), "mundo".to_string()]);
        assert_eq!(ep.neuronas_activadas, vec![1, 2]);
        casi(ep.importancia, 0.8 * (1.0 + 0.5)); // 1.2
        assert!(m.indice_palabras.contains_key("hola"));
    }

    #[test]
    fn test_registrar_clampa_intensidad_y_valencia() {
        let mut m = MemoriaVinculo::nueva();
        interaccion(&mut m, 1, &["x"], &[], 2.0, 3.0);
        let ep = &m.episodios[0];
        casi(ep.intensidad_emocional, 1.0);
        casi(ep.valencia, 1.0);
    }

    #[test]
    fn test_confianza_crece_con_valencia_positiva() {
        let mut m = MemoriaVinculo::nueva();
        interaccion(&mut m, 1, &["gracias"], &[], 0.5, 1.0);
        casi(m.modelo.confianza, 0.1 + 0.01 * 1.0);
    }

    #[test]
    fn test_confianza_decae_con_valencia_negativa() {
        let mut m = MemoriaVinculo::nueva();
        interaccion(&mut m, 1, &["mal"], &[], 0.5, -1.0);
        casi(m.modelo.confianza, 0.1 - 0.03 * 1.0);
    }

    #[test]
    fn test_fase_conocimiento_tras_mas_de_5_interacciones() {
        let mut m = MemoriaVinculo::nueva();
        for i in 0..6 {
            interaccion(&mut m, i as u64, &["hola"], &[], 0.5, 0.5);
        }
        assert_eq!(m.modelo.fase, FaseRelacion::Conocimiento);
    }

    #[test]
    fn test_fase_companerismo_confianza_alta() {
        let mut m = MemoriaVinculo::nueva();
        // Comprensión = 0.1 + 0.001*n; necesita > 0.7 => n > 600
        for i in 0..610 {
            interaccion(&mut m, i as u64, &["gracias"], &[], 0.9, 1.0);
        }
        assert_eq!(m.modelo.fase, FaseRelacion::Companerismo);
    }

    #[test]
    fn test_fase_como_str() {
        assert_eq!(FaseRelacion::Inicio.como_str(), "inicio");
        assert_eq!(FaseRelacion::Conocimiento.como_str(), "conocimiento");
        assert_eq!(FaseRelacion::Construccion.como_str(), "construccion");
        assert_eq!(FaseRelacion::Confianza.como_str(), "confianza");
        assert_eq!(FaseRelacion::Companerismo.como_str(), "companerismo");
    }

    #[test]
    fn test_estilo_tecnico_por_ratio() {
        let mut m = MemoriaVinculo::nueva();
        // 3 de 4 palabras técnicas (>0.3 ratio)
        interaccion(&mut m, 1, &["implementation", "struct", "modulo", "fn"], &[], 0.5, 0.5);
        assert_eq!(
            m.modelo.conocimiento.estilo_comunicacion,
            EstiloComunicacion::Tecnico
        );
    }

    #[test]
    fn test_estilo_directo_pocas_palabras() {
        let mut m = MemoriaVinculo::nueva();
        interaccion(&mut m, 1, &["hola", "mundo"], &[], 0.5, 0.5);
        assert_eq!(
            m.modelo.conocimiento.estilo_comunicacion,
            EstiloComunicacion::Directo
        );
    }

    #[test]
    fn test_palabras_frecuentes_se_acumulan_y_ordenan() {
        let mut m = MemoriaVinculo::nueva();
        interaccion(&mut m, 1, &["rust"], &[], 0.5, 0.5);
        interaccion(&mut m, 2, &["rust"], &[], 0.5, 0.5);
        interaccion(&mut m, 3, &["rust"], &[], 0.5, 0.5);
        let top = &m.modelo.conocimiento.palabras_frecuentes[0];
        assert_eq!(top.0, "rust");
        assert_eq!(top.1, 3);
    }

    #[test]
    fn test_recuperar_sin_palabras_vacio() {
        let m = MemoriaVinculo::nueva();
        assert!(m.recuperar_recuerdos(&[], 100, 3).is_empty());
    }

    #[test]
    fn test_recuperar_recuerdos_por_similitud() {
        let mut m = MemoriaVinculo::nueva();
        interaccion(&mut m, 10, &["rust", "cerebro"], &[1], 0.8, 0.5);
        let recs = m.recuperar_recuerdos(&["rust".to_string()], 20, 3);
        assert_eq!(recs.len(), 1);
        let r = &recs[0];
        casi(r.similitud, 1.0); // 1 coincidencia / 1 palabra
        assert_eq!(r.antiguedad, 10);
        casi(r.relevancia, 1.0 * r.episodio.importancia / (10.0f64).sqrt());
    }

    #[test]
    fn test_recuperar_recuerdos_sin_coincidencia_vacio() {
        let mut m = MemoriaVinculo::nueva();
        interaccion(&mut m, 1, &["rust"], &[1], 0.5, 0.5);
        assert!(m.recuperar_recuerdos(&["python".to_string()], 10, 3).is_empty());
    }

    #[test]
    fn test_recuperar_recuerdos_trunca_a_max_resultados() {
        let mut m = MemoriaVinculo::nueva();
        interaccion(&mut m, 1, &["hola"], &[], 0.5, 0.5);
        interaccion(&mut m, 2, &["hola"], &[], 0.5, 0.5);
        interaccion(&mut m, 3, &["hola"], &[], 0.5, 0.5);
        let recs = m.recuperar_recuerdos(&["hola".to_string()], 10, 2);
        assert_eq!(recs.len(), 2);
    }

    #[test]
    fn test_contexto_inyectable_sin_recuerdos() {
        let m = MemoriaVinculo::nueva();
        let ctx = m.generar_contexto_inyectable(&["zzz".to_string()], 100);
        assert_eq!(ctx, vec!["[VINCULO:sin_recuerdos]"]);
    }

    #[test]
    fn test_contexto_inyectable_con_recuerdos() {
        let mut m = MemoriaVinculo::nueva();
        interaccion(&mut m, 1, &["cerebro"], &[], 0.5, 0.5);
        let ctx = m.generar_contexto_inyectable(&["cerebro".to_string()], 10);
        assert_eq!(ctx[0], "[VINCULO]");
        assert!(ctx.contains(&"cerebro".to_string()));
        assert_eq!(ctx.last().unwrap(), "[FIN_VINCULO]");
    }

    #[test]
    fn test_poda_respeta_capacidad_maxima() {
        let mut m = MemoriaVinculo::nueva();
        m.capacidad_maxima = 3;
        for i in 0..5 {
            interaccion(&mut m, i as u64, &["palabra"], &[], 0.5, 0.5);
        }
        assert_eq!(m.episodios.len(), 3);
        assert_eq!(m.total_interacciones, 5);
    }

    #[test]
    fn test_poda_elimina_el_menos_importante() {
        let mut m = MemoriaVinculo::nueva();
        m.capacidad_maxima = 1;
        // Menos importante: intensidad baja
        interaccion(&mut m, 1, &["bajo"], &[], 0.1, 0.0);
        interaccion(&mut m, 2, &["alto"], &[], 0.9, 0.0);
        assert_eq!(m.episodios.len(), 1);
        assert_eq!(m.episodios[0].palabras_clave[0], "alto");
    }

    #[test]
    fn test_reactivar_recuerdos_inyecta_corriente() {
        let mut m = MemoriaVinculo::nueva();
        interaccion(&mut m, 1, &["cerebro"], &[42], 0.9, 0.5);
        let config = config_con_gpu();
        let mut memoria = MemoriaAdaptativa::nuevo(&config);
        let mut neurona = NeuronaCompacta::reposo(42, 0, 0);
        neurona.activacion = 0.1;
        memoria.ram.agregar_neurona(neurona);
        m.reactivar_recuerdos(&["cerebro".to_string()], 10, &mut memoria);
        let n = memoria.obtener_neurona(42).unwrap();
        assert!(n.corriente_entrada > 0.0, "corriente debe inyectarse");
        // activación = max(0.1, intensidad*0.3 = 0.27)
        casi(n.activacion as f64, 0.27);
    }
}
