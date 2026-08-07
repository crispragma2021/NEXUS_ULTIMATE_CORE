use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::cerebro::estructuras::{Estimulo};

// ============================================================================
// MOTOR SENSORIAL — Pipeline de entrada autónomo
// ============================================================================
//
// Biología pura: cada palabra genera su propio token_id autónomo mediante un
// hash estable determinista. El mapeo palabra→token se mantiene internamente,
// de modo que este motor NO depende de ningún léxico estadístico externo.

pub struct MotorSensorial {
    /// Embeddings de palabras: token_id → vector de D dimensiones
    pub embeddings: HashMap<u32, Vec<f32>>,

    /// Mapa autónomo palabra normalizada → token_id (hash estable)
    pub token_por_palabra: HashMap<String, u32>,

    /// Siguiente id libre (base de tokens sensoriales)
    pub siguiente_token: u32,

    /// Dimensionalidad del embedding
    pub dimensiones: usize,  // 256

    /// Elementos no-cero en el index vector inicial (sparsity)
    pub k_sparse: usize,  // 8

    /// Tasa de aprendizaje para contexto
    pub tasa_contexto: f32,  // 0.01

    /// Ventana de contexto (tokens a cada lado)
    pub ventana_contexto: usize,  // 3

    /// Neuronas base para el mapeo (rango de IDs)
    pub base_neurona: u32,  // 10000
    pub grupo_por_neurona: usize,  // 8

    /// RNG interno para generar index vectors
    pub rng: u64,

    /// Palabras procesadas
    pub palabras_procesadas: u64,
}

impl MotorSensorial {
    pub fn nuevo() -> Self {
        let mut motor = Self {
            embeddings: HashMap::new(),
            token_por_palabra: HashMap::new(),
            siguiente_token: 1,
            dimensiones: 256,
            k_sparse: 8,
            tasa_contexto: 0.01,
            ventana_contexto: 3,
            base_neurona: 10000,
            grupo_por_neurona: 1, // 1 neurona por dimensión → semántica distribuida completa
            rng: rand::thread_rng().gen(),
            palabras_procesadas: 0,
        };
        motor.sembrar_vocabulario_innato();
        motor
    }

    /// Siembra el vocabulario innato de conceptos cableados.
    ///
    /// Las asambleas corticales semilla están pre-cableadas con tokens fijos
    /// (0..N) que representan conceptos fundamentales. Este vocabulario innato
    /// alinea el pipeline sensorial con esas asambleas, de modo que el texto
    /// "hola buenos días" tokenice a [0,1,2] y evoque el concepto "saludo".
    ///
    /// Sin LLM ni estadística: es el equivalente biológico de un repertorio
    /// innato de conceptos (como la capacidad filogenética para el lenguaje).
    fn sembrar_vocabulario_innato(&mut self) {
        // (palabra_normalizada, token_id) — tokens 0..57 de las asambleas semilla.
        let seed: &[(&str, u32)] = &[
            ("hola", 0), ("buenos", 1), ("dias", 2),
            ("adios", 3), ("hasta", 4),
            ("que", 5), ("como", 6), ("por", 7),
            ("si", 8), ("claro", 9),
            ("no", 10), ("nunca", 11),
            ("ayuda", 12), ("necesito", 13),
            ("gracias", 14),
            ("saber", 15), ("aprender", 16),
            ("feliz", 17), ("bien", 18),
            ("mal", 19), ("triste", 20),
            ("buscar", 21), ("encontrar", 22),
            ("crear", 23), ("construir", 24),
            ("juntos", 25), ("nosotros", 26),
            ("conocer", 27), ("ciencia", 28),
            ("codigo", 29), ("sistema", 30),
            ("mente", 31), ("conciencia", 32),
            ("pensar", 33), ("significado", 34),
            ("quien", 35), ("soy", 36),
            ("recordar", 37), ("pasado", 38),
            ("dormir", 39), ("sonar", 40),
            ("peligro", 41), ("error", 42),
            ("logro", 43),
            ("quizas", 44), ("talvez", 45),
            ("seguro", 46),
            ("hacer", 47), ("ejecutar", 48),
            ("silencio", 49),
            ("confianza", 52),
            ("arquitecto", 53), ("creador", 54),
            ("tutor", 55),
            ("dopamina", 56),
            ("plasticidad", 57),
        ];
        for (palabra, token) in seed {
            self.token_por_palabra.entry(palabra.to_string()).or_insert(*token);
        }
        // Continuar asignando tokens nuevos después del repertorio innato.
        if self.siguiente_token <= 57 {
            self.siguiente_token = 58;
        }
    }

    /// Normaliza una palabra a su forma canónica sensorial
    fn normalizar(palabra: &str) -> String {
        palabra
            .to_lowercase()
            .trim_matches(|c: char| c.is_ascii_punctuation())
            .to_string()
    }

    /// Obtiene (o crea) un token_id autónomo para una palabra mediante hash estable.
    pub fn token_para(&mut self, palabra: &str) -> u32 {
        let clave = Self::normalizar(palabra);
        if clave.is_empty() {
            return 0;
        }
        *self.token_por_palabra.entry(clave).or_insert_with(|| {
            let id = self.siguiente_token;
            self.siguiente_token += 1;
            id
        })
    }

    /// Convierte texto en una secuencia de token_ids autónomos.
    pub fn tokens_de_texto(&mut self, texto: &str) -> Vec<u32> {
        let mut tokens = Vec::new();
        for palabra in texto.split_whitespace() {
            let clave = Self::normalizar(palabra);
            if clave.is_empty() {
                continue;
            }
            let token_id = self.token_para(&clave);
            self.index_vector(token_id);
            tokens.push(token_id);
        }
        tokens
    }

    /// Genera un index vector aleatorio para un token, si no existe
    fn index_vector(&mut self, token_id: u32) -> &mut Vec<f32> {
        let dimensiones = self.dimensiones;
        let k_sparse = self.k_sparse;
        let mut local_rng = StdRng::seed_from_u64(self.rng ^ token_id as u64);

        self.embeddings.entry(token_id).or_insert_with(|| {
            let mut vec = vec![0.0; dimensiones];
            for _ in 0..k_sparse {
                let pos = local_rng.gen_range(0..dimensiones);
                let val = if local_rng.gen::<bool>() { 1.0 } else { -1.0 };
                vec[pos] = val;
            }
            vec
        })
    }

    /// Actualiza embeddings por co-ocurrencia en una oración
    pub fn aprender_contexto(&mut self, tokens: &[u32]) {
        let mut updates: HashMap<u32, Vec<f32>> = HashMap::new();

        for i in 0..tokens.len() {
            let target_token_id = tokens[i];

            // Asegurarse de que el target_token_id tiene un embedding inicial
            self.index_vector(target_token_id);
            updates.entry(target_token_id).or_insert_with(|| self.embeddings.get(&target_token_id).map(|v| v.clone()).unwrap_or_default());

            for j in (i as isize - self.ventana_contexto as isize)..(i as isize + self.ventana_contexto as isize + 1) {
                if j >= 0 && j < tokens.len() as isize && i != j as usize {
                    let context_token_id = tokens[j as usize];

                    // Asegurarse de que el context_token_id tiene un embedding inicial
                    self.index_vector(context_token_id);

                    if let Some(context_vector) = self.embeddings.get(&context_token_id).map(|v| v.clone()) {
                        if let Some(target_embedding_for_update) = updates.get_mut(&target_token_id) {
                            for k in 0..self.dimensiones {
                                target_embedding_for_update[k] += context_vector[k] * self.tasa_contexto;
                            }
                        }
                    }
                }
            }
        }

        // Aplicar todas las actualizaciones de vuelta a self.embeddings
        for (token_id, updated_embedding) in updates.drain() {
            if let Some(actual) = self.embeddings.get_mut(&token_id) {
                *actual = updated_embedding;
            }
        }

        self.palabras_procesadas += tokens.len() as u64;
        self.rng = rand::thread_rng().gen(); // Actualizar RNG principal
    }

    /// Convierte texto en estímulos neuronales autónomos.
    /// Retorna Vec<Estimulo> listo para alimentar al cerebro.
    pub fn texto_a_estimulos(&mut self, texto: &str) -> Vec<Estimulo> {
        let mut estimulos = Vec::new();
        let mut tokens: Vec<u32> = Vec::new();

        for palabra in texto.split_whitespace() {
            let clave = Self::normalizar(palabra);
            if clave.is_empty() {
                continue;
            }
            let token_id = self.token_para(&clave);
            self.index_vector(token_id);
            tokens.push(token_id);
        }

        // Aprendizaje contextual de co-ocurrencia en el mismo pasaje
        if !tokens.is_empty() {
            self.aprender_contexto(&tokens);
        }

        for token_id in &tokens {
            if let Some(embedding) = self.embeddings.get(token_id) {
                // Una neurona por dimensión (grupo_por_neurona = 1) → representación
                // semántica distribuida completa en el espacio de neuronas.
                // El rango base_neurona..base_neurona+dimensiones cubre todo el
                // embedding sin colapsar las 256 dimensiones en 32 neuronas.
                let grupo = self.grupo_por_neurona.max(1);
                for base_dim in (0..self.dimensiones).step_by(grupo) {
                    // Sumar el grupo contiguo de dimensiones y promediar (si grupo > 1)
                    let mut suma = 0.0f32;
                    let mut contadas = 0usize;
                    for j in 0..grupo {
                        let dim_idx = base_dim + j;
                        if dim_idx >= self.dimensiones {
                            break;
                        }
                        suma += embedding[dim_idx].abs();
                        contadas += 1;
                    }
                    let intensidad = if contadas > 0 { suma / contadas as f32 } else { 0.0 };
                    if intensidad > 0.1 {
                        let neurona_id = self.base_neurona + (base_dim / grupo) as u32;
                        estimulos.push(Estimulo {
                            id: neurona_id,
                            // Intensidad = firmeza de activación (siempre ≥ 0).
                            intensidad: intensidad.clamp(0.0, 1.0),
                            amenaza: 0.0,
                            recompensa: 0.0,
                            valor: intensidad,
                        });
                    }
                }
            }
        }
        estimulos
    }

    /// Calcula similitud semántica entre dos tokens (0.0-1.0) usando coseno
    pub fn similitud_semantica(&self, token_a: u32, token_b: u32) -> f32 {
        let emb_a = self.embeddings.get(&token_a);
        let emb_b = self.embeddings.get(&token_b);

        if let (Some(v_a), Some(v_b)) = (emb_a, emb_b) {
            let dot_product: f32 = v_a.iter().zip(v_b.iter()).map(|(&x, &y)| x * y).sum();
            let magnitude_a: f32 = v_a.iter().map(|&x| x * x).sum::<f32>().sqrt();
            let magnitude_b: f32 = v_b.iter().map(|&x| x * x).sum::<f32>().sqrt();

            if magnitude_a > 1e-6 && magnitude_b > 1e-6 {
                ((dot_product / (magnitude_a * magnitude_b)) + 1.0) / 2.0 // Normalizar a 0-1
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Encuentra los tokens más similares a uno dado.
    /// Devuelve la palabra original mapeada más la similitud.
    pub fn tokens_similares(&self, token_id: u32, top_n: usize) -> Vec<(String, f32)> {
        let mut similitudes = Vec::new();
        if let Some(target_embedding) = self.embeddings.get(&token_id) {
            for (&other_token_id, other_embedding) in &self.embeddings {
                if token_id != other_token_id {
                    let dot_product: f32 = target_embedding.iter().zip(other_embedding.iter()).map(|(&x, &y)| x * y).sum();
                    let magnitude_a: f32 = target_embedding.iter().map(|&x| x * x).sum::<f32>().sqrt();
                    let magnitude_b: f32 = other_embedding.iter().map(|&x| x * x).sum::<f32>().sqrt();

                    if magnitude_a > 1e-6 && magnitude_b > 1e-6 {
                        let similitud = ((dot_product / (magnitude_a * magnitude_b)) + 1.0) / 2.0;
                        if similitud > 0.5 { // Solo mostrar similitudes relevantes
                            let palabra = self.token_por_palabra
                                .iter()
                                .find(|(_, &v)| v == other_token_id)
                                .map(|(k, _)| k.clone())
                                .unwrap_or_else(|| format!("#{}", other_token_id));
                            similitudes.push((palabra, similitud));
                        }
                    }
                }
            }
        }

        similitudes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similitudes.truncate(top_n);
        similitudes
    }

    /// Estadísticas
    pub fn total_embeddings(&self) -> usize {
        self.embeddings.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn casi(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-4,
            "esperado {:.4}, obtenido {:.4}",
            b,
            a
        );
    }

    #[test]
    fn test_nuevo_siembra_vocabulario_innato() {
        let m = MotorSensorial::nuevo();
        assert!(m.total_embeddings() == 0);
        // El vocabulario innato se siembra en token_por_palabra
        assert_eq!(m.token_por_palabra.get("hola"), Some(&0));
        assert_eq!(m.token_por_palabra.get("arquitecto"), Some(&53));
        assert_eq!(m.siguiente_token, 58);
        assert_eq!(m.dimensiones, 256);
        assert_eq!(m.k_sparse, 8);
    }

    #[test]
    fn test_token_para_normaliza_y_reutiliza() {
        let mut m = MotorSensorial::nuevo();
        // Normalización: mayúsculas y puntuación
        assert_eq!(m.token_para("HOLA"), 0);
        assert_eq!(m.token_para("hola!"), 0);
        assert_eq!(m.token_para("hola"), 0);
        // Palabra nueva recibe token siguiente
        let t = m.token_para("galaxia");
        assert_eq!(t, 58);
        assert_eq!(m.token_para("galaxia"), 58, "debe reutilizar el mismo token");
        assert_eq!(m.token_para("GALAXIA"), 58);
    }

    #[test]
    fn test_token_para_vacio_retorna_cero() {
        let mut m = MotorSensorial::nuevo();
        assert_eq!(m.token_para("!!!"), 0);
        assert_eq!(m.token_para(""), 0);
    }

    #[test]
    fn test_tokens_de_texto_asigna_secuencialmente() {
        let mut m = MotorSensorial::nuevo();
        let tokens = m.tokens_de_texto("hola mundo nuevo motor");
        // "hola" = 0 innato, "mundo"=58, "nuevo"=59, "motor"=60
        assert_eq!(tokens, vec![0, 58, 59, 60]);
        // Se crearon embeddings para cada token
        assert_eq!(m.total_embeddings(), 4);
    }

    #[test]
    fn test_tokens_de_texto_ignora_puntuacion_sola() {
        let mut m = MotorSensorial::nuevo();
        let tokens = m.tokens_de_texto("hola ,,,  ...");
        assert_eq!(tokens, vec![0]);
    }

    #[test]
    fn test_embedding_generado_es_esparso_y_determinista() {
        let mut m = MotorSensorial::nuevo();
        let t = m.token_para("estrella");
        m.index_vector(t);
        let emb = m.embeddings.get(&t).unwrap();
        assert_eq!(emb.len(), 256);
        // El index vector genera posiciones aleatorias; si hay colisión de
        // posición, los no-ceros pueden ser < k_sparse. Verificar esparsidad
        // real (entre 1 y k_sparse) en vez de exactitud frágil.
        let no_ceros = emb.iter().filter(|&&v| v != 0.0).count();
        assert!(no_ceros >= 1 && no_ceros <= 8, "vector esparso con {no_ceros} no-ceros");
        // Los valores son ±1.0
        for &v in emb.iter().filter(|&&v| v != 0.0) {
            assert!(v == 1.0 || v == -1.0);
        }
    }

    #[test]
    fn test_embedding_determinista_mismo_token() {
        let mut m = MotorSensorial::nuevo();
        let a = m.token_para("foton");
        m.index_vector(a);
        let emb1 = m.embeddings.get(&a).unwrap().clone();
        // Mismo seed => misma posición/valor
        m.index_vector(a);
        let emb2 = m.embeddings.get(&a).unwrap();
        assert_eq!(emb1, *emb2);
    }

    #[test]
    fn test_similitud_semantica_identico_es_uno() {
        let mut m = MotorSensorial::nuevo();
        let t = m.token_para("nucleo");
        m.index_vector(t);
        let sim = m.similitud_semantica(t, t);
        casi(sim, 1.0);
    }

    #[test]
    fn test_similitud_semantica_token_inexistente_cero() {
        let m = MotorSensorial::nuevo();
        assert_eq!(m.similitud_semantica(99999, 99998), 0.0);
    }

    #[test]
    fn test_aprender_contexto_actualiza_embeddings() {
        let mut m = MotorSensorial::nuevo();
        let tokens = m.tokens_de_texto("perro gato pajaro");
        let antes = m.embeddings.get(&tokens[0]).unwrap().clone();
        m.aprender_contexto(&tokens);
        let despues = m.embeddings.get(&tokens[0]).unwrap();
        assert!(
            antes != *despues,
            "el contexto debe modificar el embedding del target"
        );
        assert!(m.palabras_procesadas >= 3);
    }

    #[test]
    fn test_texto_a_estimulos_genera_estimulos_neuronales() {
        let mut m = MotorSensorial::nuevo();
        let estimulos = m.texto_a_estimulos("hola mundo nuevo");
        assert!(!estimulos.is_empty(), "debe producir estímulos para tokens con embedding");
        for e in &estimulos {
            assert!(e.id >= 10000, "los IDs neuronales parten de base_neurona");
            assert!(e.intensidad > 0.0 && e.intensidad <= 1.0);
            assert!(e.amenaza == 0.0);
            assert!(e.recompensa == 0.0);
        }
    }

    #[test]
    fn test_texto_a_estimulos_vacio_no_produce() {
        let mut m = MotorSensorial::nuevo();
        assert!(m.texto_a_estimulos("").is_empty());
    }

    #[test]
    fn test_tokens_similares_excluye_el_mismo() {
        let mut m = MotorSensorial::nuevo();
        m.tokens_de_texto("alfa beta gamma delta");
        let similares = m.tokens_similares(m.token_por_palabra["alfa"], 5);
        // No debe incluirse a sí mismo
        assert!(
            !similares.iter().any(|(p, _)| p == "alfa"),
            "no debe retornar el mismo token"
        );
    }

    #[test]
    fn test_tokens_similares_vacio_sin_datos() {
        let m = MotorSensorial::nuevo();
        assert!(m.tokens_similares(0, 5).is_empty());
    }
}
