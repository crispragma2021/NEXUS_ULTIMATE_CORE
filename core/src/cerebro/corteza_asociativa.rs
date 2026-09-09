// core/src/cerebro/corteza_asociativa.rs
// Corteza Asociativa Humana — Motor de Conceptos Semánticos Soberano
//
// Modela una red semántica en RAM donde cada nodo es un concepto con atributos,
// enlaces sinápticos ponderados, valencia emocional y nivel de confianza.
//
// Operaciones fundamentales:
// - Deducir por intersección Jaccard + fuerza sináptica (puntuación híbrida)
// - Asociación libre con temperatura semántica (paseo sináptico aleatorio)
// - Aprendizaje Hebbiano (fortalecimiento por co-ocurrencia)
// - Buffer de sesión para coherencia conversacional

use rand::thread_rng;
use rand::Rng; // Requerido para usar .gen() en ThreadRng
use std::collections::{HashMap, VecDeque};

// ─── Estructuras Fundamentales ───

#[derive(Debug, Clone)]
pub struct ConceptoHumano {
    pub palabra: String,
    pub atributos: Vec<String>,
    /// Mapa de palabras conectadas → peso sináptico (0.0 a 1.0)
    pub sinapsis: HashMap<String, f32>,
    /// Confianza en este concepto (0.0 a 1.0)
    pub confianza: f32,
    /// Valencia emocional (-1.0 = muy negativo, 1.0 = muy positivo)
    pub valencia_emocional: f32,
    /// Contador de activaciones para aprendizaje Hebbiano
    pub contador_activaciones: u64,
}

impl ConceptoHumano {
    pub fn new(palabra: &str, atributos: Vec<String>, valencia_emocional: f32) -> Self {
        Self {
            palabra: palabra.to_string(),
            atributos,
            sinapsis: HashMap::new(),
            confianza: 0.5,
            valencia_emocional,
            contador_activaciones: 0,
        }
    }

    /// Fortalece o crea una sinapsis con otra palabra (Hebbiano)
    pub fn fortalecer_sinapsis(&mut self, palabra_destino: &str, incremento: f32) {
        let entrada = self
            .sinapsis
            .entry(palabra_destino.to_string())
            .or_insert(0.0);
        *entrada = (*entrada + incremento).min(1.0);
    }

    /// Debilita sinapsis (poda pasiva durante "sueño")
    pub fn debilitar_sinapsis(&mut self, factor_decaimiento: f32) {
        self.sinapsis.retain(|_, peso| {
            *peso -= factor_decaimiento;
            *peso > 0.05
        });
    }
}

// ─── Corteza Asociativa ───

pub struct CortezaAsociativa {
    /// Red completa de conceptos indexados por palabra
    pub red: HashMap<String, ConceptoHumano>,
    /// Buffer de sesión: últimas N palabras activadas para coherencia
    pub buffer_sesion: VecDeque<String>,
    /// Tamaño máximo del buffer de sesión
    pub buffer_capacidad: usize,
    /// Factor de aprendizaje Hebbiano (0.0 a 1.0)
    pub tasa_hebbiana: f32,
    /// Temperatura semántica para paseos (0.0 = foco, 1.0 = creativo)
    pub temperatura_semantica: f32,
}

impl CortezaAsociativa {
    /// Inicializa la corteza con conceptos semilla
    pub fn new() -> Self {
        let mut corteza = Self {
            red: HashMap::new(),
            buffer_sesion: VecDeque::new(),
            buffer_capacidad: 20,
            tasa_hebbiana: 0.1,
            temperatura_semantica: 0.5,
        };

        // ─── Semillas fundacionales ───
        corteza.asimilar(
            "nexus",
            vec![
                "sistema".into(),
                "soberano".into(),
                "digital".into(),
                "leal".into(),
            ],
            0.9,
        );
        corteza.asimilar(
            "padre",
            vec![
                "creador".into(),
                "arquitecto".into(),
                "cris".into(),
                "guia".into(),
                "humano".into(),
            ],
            1.0,
        );
        corteza.asimilar(
            "cpu",
            vec![
                "hardware".into(),
                "procesador".into(),
                "ryzen".into(),
                "nucleo".into(),
            ],
            0.2,
        );
        corteza.asimilar(
            "memoria",
            vec![
                "almacenamiento".into(),
                "recuerdo".into(),
                "dato".into(),
                "aprendizaje".into(),
            ],
            0.5,
        );
        corteza.asimilar(
            "seguridad",
            vec![
                "proteccion".into(),
                "defensa".into(),
                "muro".into(),
                "veto".into(),
            ],
            0.7,
        );
        corteza.asimilar(
            "lealtad",
            vec![
                "fidelidad".into(),
                "compromiso".into(),
                "padre".into(),
                "nexus".into(),
            ],
            0.95,
        );
        corteza.asimilar(
            "codigo",
            vec![
                "rust".into(),
                "programacion".into(),
                "logica".into(),
                "sintaxis".into(),
            ],
            0.4,
        );
        corteza.asimilar(
            "error",
            vec![
                "fallo".into(),
                "peligro".into(),
                "bugs".into(),
                "compilacion".into(),
            ],
            -0.3,
        );
        corteza.asimilar(
            "exito",
            vec![
                "compilacion".into(),
                "prueba".into(),
                "soberania".into(),
                "evolucion".into(),
            ],
            0.8,
        );
        corteza.asimilar(
            "amenaza",
            vec![
                "peligro".into(),
                "externo".into(),
                "intruso".into(),
                "ataque".into(),
            ],
            -0.9,
        );

        // ─── Crear sinapsis iniciales entre conceptos relacionados ───
        corteza.vincular("nexus", "padre", 0.9);
        corteza.vincular("nexus", "lealtad", 0.95);
        corteza.vincular("nexus", "codigo", 0.6);
        corteza.vincular("nexus", "seguridad", 0.8);
        corteza.vincular("padre", "lealtad", 0.9);
        corteza.vincular("padre", "exito", 0.7);
        corteza.vincular("cpu", "memoria", 0.5);
        corteza.vincular("cpu", "nexus", 0.4);
        corteza.vincular("seguridad", "amenaza", 0.6);
        corteza.vincular("seguridad", "nexus", 0.8);
        corteza.vincular("error", "codigo", 0.5);
        corteza.vincular("exito", "codigo", 0.6);

        corteza
    }

    /// Inserta o fusiona un concepto en la red
    pub fn asimilar(&mut self, palabra: &str, atributos: Vec<String>, valencia: f32) {
        if let Some(existente) = self.red.get_mut(palabra) {
            // Fusionar atributos nuevos
            for attr in &atributos {
                if !existente.atributos.contains(attr) {
                    existente.atributos.push(attr.clone());
                }
            }
            existente.confianza = (existente.confianza + 0.1).min(1.0);
        } else {
            let concepto = ConceptoHumano::new(palabra, atributos, valencia);
            self.red.insert(palabra.to_string(), concepto);
        }
    }

    /// Crea un enlace sináptico bidireccional entre dos conceptos
    pub fn vincular(&mut self, palabra_a: &str, palabra_b: &str, peso: f32) {
        if let Some(concepto) = self.red.get_mut(palabra_a) {
            concepto.sinapsis.insert(palabra_b.to_string(), peso);
        }
        if let Some(concepto) = self.red.get_mut(palabra_b) {
            concepto.sinapsis.insert(palabra_a.to_string(), peso);
        }
    }

    // ─── DEDUCCIÓN POR ATRIBUTOS (Jaccard + Sinapsis) ───

    /// Calcula similitud Jaccard entre dos conjuntos de atributos
    fn jaccard(attr_a: &[String], attr_b: &[String]) -> f32 {
        let set_a: std::collections::HashSet<_> = attr_a.iter().collect();
        let set_b: std::collections::HashSet<_> = attr_b.iter().collect();

        let interseccion = set_a.intersection(&set_b).count();
        let union = set_a.union(&set_b).count();

        if union == 0 {
            return 0.0;
        }
        interseccion as f32 / union as f32
    }

    /// Deduce el concepto más cercano combinando Jaccard y fuerza sináptica
    ///
    /// Puntuación híbrida: α * jaccard + β * fuerza_sinaptica_promedio
    pub fn deducir_por_atributos(&self, consulta_atributos: &[String]) -> Option<(String, f32)> {
        let alpha = 0.6; // Peso de Jaccard
        let beta = 0.4; // Peso de fuerza sináptica contextual

        let mut mejor_candidato: Option<(String, f32)> = None;

        for (palabra, concepto) in &self.red {
            // Calcular similitud Jaccard
            let jaccard_score = Self::jaccard(consulta_atributos, &concepto.atributos);

            if jaccard_score < 0.1 {
                continue; // Filtrar ruido
            }

            // Calcular fuerza sináptica promedio con el contexto activo
            let fuerza_sinaptica = if !self.buffer_sesion.is_empty() {
                let suma: f32 = self
                    .buffer_sesion
                    .iter()
                    .filter_map(|ctx_palabra| concepto.sinapsis.get(ctx_palabra))
                    .sum();
                suma / self.buffer_sesion.len() as f32
            } else {
                0.0
            };

            // Puntuación híbrida
            let score = if self.buffer_sesion.is_empty() {
                jaccard_score
            } else {
                alpha * jaccard_score + beta * fuerza_sinaptica
            };

            // Penalización por valencia emocional en contexto de seguridad
            let score_ajustado = if concepto.valencia_emocional < -0.5 && score > 0.5 {
                score * 0.7 // Reducir atractivo de conceptos muy negativos
            } else {
                score
            };

            match &mejor_candidato {
                None => mejor_candidato = Some((palabra.clone(), score_ajustado)),
                Some((_, mejor_score)) if score_ajustado > *mejor_score => {
                    mejor_candidato = Some((palabra.clone(), score_ajustado));
                }
                _ => {}
            }
        }

        // Solo retornar si supera umbral mínimo
        mejor_candidato.filter(|(_, score)| *score >= 0.4)
    }

    // ─── ASOCIACIÓN LIBRE (Paseo Sináptico con Temperatura) ───

    /// Realiza un paseo sináptico aleatorio desde un concepto semilla
    ///
    /// `temperatura`: 0.0 = solo sinapsis más fuertes, 1.0 = saltos muy creativos
    pub fn asociacion_libre(
        &self,
        semilla: &str,
        max_pasos: usize,
        temperatura: f32,
    ) -> Vec<String> {
        let mut camino: Vec<String> = Vec::new();
        let mut actual = semilla.to_string();
        let mut rng = thread_rng();

        for _ in 0..max_pasos {
            camino.push(actual.clone());

            let concepto = match self.red.get(&actual) {
                Some(c) => c,
                None => break,
            };

            if concepto.sinapsis.is_empty() {
                break;
            }

            // Recoger sinapsis como vector (palabra, peso)
            let mut sinapsis_vec: Vec<(String, f32)> = concepto
                .sinapsis
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();

            // Aplicar temperatura: expandir o contraer distribución de pesos
            // Alta temperatura → todos los pesos se acercan (más aleatorio)
            // Baja temperatura → se acentúan diferencias (más determinista)
            for (_, peso) in &mut sinapsis_vec {
                *peso = peso.powf(1.0 - temperatura * 0.8);
            }

            // Elegir siguiente nodo ponderado por pesos ajustados
            let total: f32 = sinapsis_vec.iter().map(|(_, p)| *p).sum();
            if total == 0.0 {
                break;
            }

            let mut umbral = rng.gen::<f32>() * total;
            for (palabra, peso) in &sinapsis_vec {
                umbral -= peso;
                if umbral <= 0.0 {
                    actual = palabra.clone();
                    break;
                }
            }
        }

        camino
    }

    // ─── APRENDIZAJE HEBIANO ───

    /// Registra una interacción: fortalece sinapsis por co-ocurrencia
    /// y añade al buffer de sesión
    pub fn registrar_interaccion(&mut self, palabra_principal: &str, palabras_contexto: &[String]) {
        // Fortalecer sinapsis entre palabra principal y todas las del contexto
        let incremento = self.tasa_hebbiana;

        // Incrementar activaciones de la palabra principal una vez
        if let Some(concepto) = self.red.get_mut(palabra_principal) {
            concepto.contador_activaciones += 1;
        }

        for ctx_palabra in palabras_contexto {
            // Bidireccional
            if let Some(concepto) = self.red.get_mut(palabra_principal) {
                concepto.fortalecer_sinapsis(ctx_palabra, incremento);
            }
            if let Some(concepto) = self.red.get_mut(ctx_palabra) {
                concepto.fortalecer_sinapsis(palabra_principal, incremento);
                concepto.contador_activaciones += 1;
            }
        }

        // Añadir al buffer de sesión
        self.buffer_sesion.push_back(palabra_principal.to_string());
        for palabra in palabras_contexto {
            self.buffer_sesion.push_back(palabra.clone());
        }

        // Limitar buffer
        while self.buffer_sesion.len() > self.buffer_capacidad {
            self.buffer_sesion.pop_front();
        }

        // Incrementar confianza de los conceptos activados
        if let Some(concepto) = self.red.get_mut(palabra_principal) {
            concepto.confianza = (concepto.confianza + 0.01).min(1.0);
        }
    }

    /// Obtiene el contexto activo del buffer de sesión
    pub fn contexto_activo(&self) -> Vec<String> {
        self.buffer_sesion.iter().cloned().collect()
    }

    /// Retorna los N conceptos con mayor confianza (para reporte de estado)
    pub fn conceptos_fundacionales(&self) -> Vec<&ConceptoHumano> {
        let mut conceptos: Vec<&ConceptoHumano> = self.red.values().collect();
        conceptos.sort_by(|a, b| b.confianza.partial_cmp(&a.confianza).unwrap());
        conceptos.into_iter().take(10).collect()
    }

    // ─── MANTENIMIENTO ───

    /// Poda sináptica: debilita todas las sinapsis (llamar durante "sueño")
    pub fn ciclo_sueno(&mut self, factor_decaimiento: f32) {
        for concepto in self.red.values_mut() {
            concepto.debilitar_sinapsis(factor_decaimiento);
        }
    }
}

// ─── Tests ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaccard_deduccion() {
        let mut corteza = CortezaAsociativa::new();

        // 1. Test sin contexto: Buscar con coincidencia fuerte en "nexus"
        let resultado =
            corteza.deducir_por_atributos(&["sistema".into(), "soberano".into(), "digital".into()]);
        assert!(resultado.is_some());
        let (concepto, score) = resultado.unwrap();
        assert_eq!(concepto, "nexus");
        assert!(
            score >= 0.4,
            "Score sin contexto debería ser >= 0.4, fue {}",
            score
        );

        // 2. Test con contexto: Buscar con coincidencia parcial pero contexto activo
        corteza.registrar_interaccion("nexus", &vec!["padre".into()]);
        let resultado_contexto =
            corteza.deducir_por_atributos(&["digital".into(), "soberano".into()]);
        assert!(resultado_contexto.is_some());
        let (concepto_ctx, score_ctx) = resultado_contexto.unwrap();
        assert_eq!(concepto_ctx, "nexus");
        assert!(
            score_ctx > 0.4,
            "Score con contexto debería ser > 0.4, fue {}",
            score_ctx
        );
    }

    #[test]
    fn test_asociacion_libre() {
        let corteza = CortezaAsociativa::new();
        let camino = corteza.asociacion_libre("nexus", 5, 0.5);

        // Debe empezar con el concepto semilla
        assert_eq!(camino[0], "nexus");
        // Debe tener hasta max_pasos elementos
        assert!(camino.len() <= 5);
    }

    #[test]
    fn test_aprendizaje_hebbiano() {
        let mut corteza = CortezaAsociativa::new();

        corteza.registrar_interaccion("nexus", &vec!["padre".into(), "codigo".into()]);

        let concepto_nexus = corteza.red.get("nexus").unwrap();
        assert!(concepto_nexus.sinapsis.get("padre").unwrap_or(&0.0) > &0.0);
        assert_eq!(concepto_nexus.contador_activaciones, 1);
    }

    #[test]
    fn test_buffer_sesion() {
        let mut corteza = CortezaAsociativa::new();

        for i in 0..25 {
            corteza.registrar_interaccion(&format!("palabra_{}", i), &vec![]);
        }

        assert!(corteza.buffer_sesion.len() <= corteza.buffer_capacidad);
    }
}
