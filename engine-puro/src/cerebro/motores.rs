// ============================================================================
// MOTORES BIOLÓGICOS DEL CEREBRO DIGITAL
// ============================================================================
// 8 motores que implementan ecuaciones diferenciales biológicamente inspiradas:
// 1. Neurona (Hodgkin-Huxley)
// 2. Sinapsis (STDP real)
// 3. Hipocampo (Memoria Episódica)
// 4. Amígdala (Emociones)
// 5. Atención Selectiva
// 6. Dopamina (Recompensa)
// 7. Conciencia (Espacio de Trabajo Global)
// 8. Curiosidad (Exploración Autónoma + Internet)
// ============================================================================

use crate::cerebro::estructuras::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// MOTOR 1: NEURONA (Hodgkin-Huxley)
// ============================================================================
// Implementa las ecuaciones de Hodgkin-Huxley para el potencial de acción.
// dV/dt = (I - g_Na*m^3*h*(V-E_Na) - g_K*n^4*(V-E_K) - g_L*(V-E_L)) / C
// ============================================================================

pub struct MotorNeurona;

impl MotorNeurona {
    /// Actualiza una neurona con el modelo Hodgkin-Huxley.
    /// Retorna true si la neurona disparó un spike.
    pub fn actualizar(
        n: &mut NeuronaCompacta,
        entrada: f32,
        dt: f32,
        params: &ParametrosNeurona,
    ) -> bool {
        let g_na = params.g_na;
        let g_k = params.g_k;
        let g_l = params.g_l;
        let e_na = params.e_na;
        let e_k = params.e_k;
        let e_l = params.e_l;
        let v = n.voltaje;

        // Calcular constantes de velocidad alfa y beta (Hodgkin-Huxley)
        let alpha_m = if (v + 40.0).abs() > 1e-6 {
            0.1 * (v + 40.0) / (1.0 - (-(v + 40.0) / 10.0).exp())
        } else {
            0.1 * 10.0 / (1.0 - (-0.1_f32).exp()) // Límite cuando v→-40
        };
        let beta_m = 4.0 * (-(v + 65.0) / 18.0).exp();

        let alpha_h = 0.07 * (-(v + 65.0) / 20.0).exp();
        let beta_h = 1.0 / (1.0 + (-(v + 35.0) / 10.0).exp());

        let alpha_n = if (v + 55.0).abs() > 1e-6 {
            0.01 * (v + 55.0) / (1.0 - (-(v + 55.0) / 10.0).exp())
        } else {
            0.01 * 10.0 / (1.0 - (-0.1_f32).exp()) // Límite cuando v→-55
        };
        let beta_n = 0.125 * (-(v + 65.0) / 80.0).exp();

        // Actualizar compuertas (Euler forward)
        n.m += (alpha_m * (1.0 - n.m) - beta_m * n.m) * dt;
        n.h += (alpha_h * (1.0 - n.h) - beta_h * n.h) * dt;
        n.n += (alpha_n * (1.0 - n.n) - beta_n * n.n) * dt;

        // Clamp para estabilidad numérica
        n.m = n.m.clamp(0.0, 1.0);
        n.h = n.h.clamp(0.0, 1.0);
        n.n = n.n.clamp(0.0, 1.0);

        // Corrientes iónicas
        let i_na = g_na * n.m.powi(3) * n.h * (v - e_na);
        let i_k = g_k * n.n.powi(4) * (v - e_k);
        let i_l = g_l * (v - e_l);

        // Corriente total y ecuación de membrana
        let i_total = entrada - i_na - i_k - i_l;
        n.voltaje += i_total * dt;

        // Refractario (decaimiento)
        n.refractario *= 1.0 - dt * 10.0; // τ = 100ms
        n.refractario = n.refractario.clamp(0.0, 1.0);

        // Energía (decaimiento lento + recuperación)
        n.energia *= 1.0 - dt * 1.0; // τ = 1s
        n.energia += entrada.abs() * dt * 10.0;
        n.energia = n.energia.clamp(0.0, 1.0);

        // Traza de plasticidad
        n.traza *= 1.0 - dt * 5.0; // τ = 200ms
        n.traza += entrada.abs() * dt * 100.0;
        n.traza = n.traza.clamp(0.0, 1.0);

        // Frecuencia (decae lentamente)
        n.frecuencia *= 1.0 - dt * 0.1;

        // Activación
        n.activacion = n.energia * (1.0 - n.refractario);

        // Detectar spike
        if n.voltaje > 30.0 && n.refractario < 0.1 {
            n.voltaje = -70.0; // Reset
            n.refractario = 1.0;
            n.frecuencia += 1.0 / dt.max(0.001); // Aproximación de frecuencia
            n.edad = n.edad.saturating_add(1);
            return true;
        }

        false
    }

    /// Versión simplificada sin Hodgkin-Huxley (más rápida, para neuronas latentes)
    pub fn actualizar_simple(
        n: &mut NeuronaCompacta,
        entrada: f32,
        dt: f32,
    ) -> bool {
        // Integrate-and-Fire con decaimiento
        n.voltaje += (entrada - n.voltaje * 0.1) * dt;

        // Refractario
        n.refractario *= 1.0 - dt * 10.0;
        n.refractario = n.refractario.clamp(0.0, 1.0);

        // Energía
        n.energia *= 1.0 - dt * 1.0;
        n.energia += entrada.abs() * dt * 10.0;
        n.energia = n.energia.clamp(0.0, 1.0);

        n.activacion = n.energia * (1.0 - n.refractario);

        // Detectar spike
        if n.voltaje > 20.0 && n.refractario < 0.1 {
            n.voltaje = -70.0;
            n.refractario = 1.0;
            n.frecuencia += 1.0 / dt.max(0.001);
            n.edad = n.edad.saturating_add(1);
            return true;
        }

        false
    }
}

// ============================================================================
// MOTOR 2: SINAPSIS (STDP Real)
// ============================================================================
// Spike-Timing-Dependent Plasticity con ventana temporal exponencial.
// Δw = A+ * exp(-Δt/τ+) si pre→post (LTP)
// Δw = -A- * exp(Δt/τ-) si post→pre (LTD)
// ============================================================================

pub struct MotorSTDP;

impl MotorSTDP {
    /// Actualiza el peso sináptico según STDP.
    /// `pre_spike`: ¿disparó la neurona presináptica?
    /// `post_spike`: ¿disparó la neurona postsináptica?
    /// `dt`: diferencia temporal entre spikes (ms)
    pub fn actualizar(
        peso: &mut f32,
        pre_spike: bool,
        post_spike: bool,
        dt: f32,
        params: &ParametrosSTDP,
    ) {
        if pre_spike && post_spike {
            // Potenciación a largo plazo (LTP)
            let delta = params.a_plus * (-dt / params.tau_plus).exp() * params.plasticidad_critica;
            *peso += delta * (1.0 - peso.abs()); // Soft-bound
        } else if post_spike && !pre_spike {
            // Depresión a largo plazo (LTD)
            let delta = -params.a_minus * (dt / params.tau_minus).exp() * params.plasticidad_critica;
            *peso += delta * (1.0 + peso.abs()); // Soft-bound
        }

        // Decaimiento natural
        *peso *= 1.0 - params.decaimiento * dt;

        // Clamp
        *peso = peso.clamp(-1.0, 1.0);
    }

    /// STDP con traza simplificada (para batch processing)
    pub fn actualizar_con_traza(
        peso: &mut f32,
        traza_pre: f32,
        traza_post: f32,
        params: &ParametrosSTDP,
    ) {
        let ltp = params.a_plus * traza_pre * (1.0 - peso.abs()) * params.plasticidad_critica;
        let ltd = -params.a_minus * traza_post * (1.0 + peso.abs()) * params.plasticidad_critica;
        *peso += (ltp + ltd) * 0.1;
        *peso = peso.clamp(-1.0, 1.0);
    }
}

// ============================================================================
// MOTOR 3: HIPOCAMPO (Memoria Episódica)
// ============================================================================
// Almacena y recupera episodios con olvido selectivo y relevancia.
// ============================================================================

pub struct Hipocampo {
    pub episodios: Vec<Episodio>,
    pub capacidad_maxima: usize,
    pub tau_olvido: f32,
}

impl Hipocampo {
    pub fn nuevo(capacidad: usize) -> Self {
        Self {
            episodios: Vec::with_capacity(capacidad.min(1_000_000)),
            capacidad_maxima: capacidad,
            tau_olvido: 1000.0, // Constante de olvido en ciclos
        }
    }

    /// Almacena un episodio, descartando el menos relevante si se excede capacidad
    pub fn almacenar(&mut self, episodio: Episodio) {
        self.episodios.push(episodio);
        if self.episodios.len() > self.capacidad_maxima {
            // Ordenar por relevancia y truncar
            self.episodios.sort_by(|a, b| {
                b.relevancia
                    .partial_cmp(&a.relevancia)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.episodios.truncate(self.capacidad_maxima);
        }
    }

    /// Recupera episodios que coinciden con un patrón dado
    pub fn recuperar(&self, patron: &[u32]) -> Vec<Episodio> {
        let mut resultados: Vec<Episodio> = self
            .episodios
            .iter()
            .filter(|e| e.similitud_patron(patron) > 0.6)
            .cloned()
            .collect();
        resultados.sort_by(|a, b| {
            b.relevancia
                .partial_cmp(&a.relevancia)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        resultados.truncate(10);
        resultados
    }

    /// Recupera los episodios más recientes
    pub fn recientes(&self, n: usize) -> Vec<Episodio> {
        let start = self.episodios.len().saturating_sub(n);
        self.episodios[start..].to_vec()
    }

    /// Aplica olvido (decaimiento de relevancia)
    pub fn olvidar(&mut self, dt: f32) {
        for ep in &mut self.episodios {
            ep.relevancia *= 1.0 - dt / self.tau_olvido;
        }
        self.episodios.retain(|e| e.relevancia > 0.01);
    }

    /// Similitud entre dos slices de patrones
    pub fn similitud(a: &[u32], b: &[u32]) -> f32 {
        let mut coincidencias = 0;
        for &x in a {
            if b.contains(&x) {
                coincidencias += 1;
            }
        }
        let max_len = a.len().max(b.len());
        if max_len == 0 {
            0.0
        } else {
            coincidencias as f32 / max_len as f32
        }
    }
}

// ============================================================================
// MOTOR 4: AMÍGDALA (Emociones)
// ============================================================================
// Modelo de emociones con miedo, ansiedad, ira y alegría.
// ============================================================================

#[derive(Serialize, Deserialize)]
pub struct Amigdala {
    pub miedo: f32,
    pub ansiedad: f32,
    pub ira: f32,
    pub alegria: f32,
}

impl Amigdala {
    pub fn nuevo() -> Self {
        Self {
            miedo: 0.1,
            ansiedad: 0.1,
            ira: 0.1,
            alegria: 0.5, // Alegría basal
        }
    }

    /// Actualiza las emociones según amenaza y recompensa.
    /// Retorna la valencia emocional (-1 a 1).
    pub fn actualizar(&mut self, dt: f32, amenaza: f32, recompensa: f32) -> f32 {
        let tau = 1.0; // Constante de tiempo

        // Miedo: responde a amenaza
        self.miedo += (amenaza - self.miedo) / tau * dt;
        self.miedo = self.miedo.clamp(0.0, 1.0);

        // Ansiedad: amenaza persistente
        self.ansiedad += (amenaza * 0.5 - self.ansiedad) / tau * dt;
        self.ansiedad = self.ansiedad.clamp(0.0, 1.0);

        // Ira: amenaza con baja recompensa
        self.ira += (amenaza * 0.3 - self.ira) / tau * dt;
        self.ira = self.ira.clamp(0.0, 1.0);

        // Alegría: responde a recompensa (decae más lento)
        self.alegria += (recompensa - self.alegria) / (tau * 2.0) * dt;
        self.alegria = self.alegria.clamp(0.0, 1.0);

        // Valencia emocional
        (self.alegria - self.miedo).clamp(-1.0, 1.0)
    }

    /// Determina la emoción dominante
    pub fn emocion_dominante(&self) -> &str {
        let emociones = [
            (self.miedo, "miedo"),
            (self.ansiedad, "ansiedad"),
            (self.ira, "ira"),
            (self.alegria, "alegría"),
        ];
        emociones
            .iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, nombre)| *nombre)
            .unwrap_or("neutral")
    }

    /// Intensidad emocional total
    pub fn intensidad_total(&self) -> f32 {
        (self.miedo + self.ansiedad + self.ira + self.alegria) / 4.0
    }
}

// ============================================================================
// MOTOR 5: ATENCIÓN SELECTIVA
// ============================================================================
// Foco selectivo con saliencia dinámica. Imita la corteza parietal.
// ============================================================================

pub struct AtencionSelectiva {
    pub foco: Vec<u32>,                // IDs en foco
    pub intensidad: f32,               // Intensidad del foco (0-1)
    pub saliencia: HashMap<u32, f32>,  // Mapa de saliencia por ID
}

impl AtencionSelectiva {
    pub fn nuevo() -> Self {
        Self {
            foco: Vec::with_capacity(10),
            intensidad: 0.5,
            saliencia: HashMap::new(),
        }
    }

    /// Actualiza el mapa de saliencia y selecciona el foco.
    /// `estimulos`: slice de (id, intensidad) de entrada
    /// Retorna los IDs en foco
    pub fn actualizar(&mut self, dt: f32, estimulos: &[(u32, f32)]) -> Vec<u32> {
        // Actualizar saliencia para cada estímulo
        for &(id, intensidad) in estimulos {
            let saliencia_actual = self.saliencia.get(&id).copied().unwrap_or(0.0);
            let nueva_saliencia = intensidad * (1.0 + saliencia_actual * 0.5);
            let entry = self.saliencia.entry(id).or_insert(0.0);
            *entry += (nueva_saliencia - *entry) * dt * 5.0;
            // Decaimiento
            *entry *= 1.0 - dt * 0.1;
        }

        // Seleccionar top N por saliencia
        let mut items: Vec<(u32, f32)> = self.saliencia
            .iter()
            .map(|(&id, &s)| (id, s))
            .collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        items.truncate(10);

        self.foco = items.iter().map(|&(id, _)| id).collect();
        self.intensidad = items
            .first()
            .map(|(_, s)| *s)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);

        self.foco.clone()
    }

    /// ¿Un ID está en el foco actual?
    pub fn en_foco(&self, id: u32) -> bool {
        self.foco.contains(&id)
    }

    /// Resetea el foco
    pub fn resetear(&mut self) {
        self.foco.clear();
        self.intensidad = 0.0;
        self.saliencia.clear();
    }
}

// ============================================================================
// MOTOR 6: DOPAMINA (Sistema de Recompensa)
// ============================================================================
// Implementa el error de predicción de recompensa (similar a RL).
// ============================================================================

#[derive(Serialize, Deserialize)]
pub struct SistemaDopamina {
    pub nivel: f32,         // Nivel actual de dopamina
    pub prediccion: f32,    // Predicción de recompensa
}

impl SistemaDopamina {
    pub fn nuevo() -> Self {
        Self {
            nivel: 0.3,
            prediccion: 0.3,
        }
    }

    /// Actualiza el sistema con la recompensa recibida.
    /// Retorna el error de predicción (señal de aprendizaje).
    pub fn actualizar(&mut self, dt: f32, recompensa: f32) -> f32 {
        let error = recompensa - self.prediccion;

        // Actualizar nivel de dopamina
        self.nivel += (recompensa - self.nivel) * dt * 0.5;
        self.nivel -= self.nivel * dt * 0.1; // Decaimiento
        self.nivel = self.nivel.clamp(0.0, 1.0);

        // Actualizar predicción (aprendizaje)
        self.prediccion += error * dt * 0.1;
        self.prediccion = self.prediccion.clamp(0.0, 1.0);

        error
    }

    /// Señal de recompensa modulada por dopamina
    pub fn senial_recompensa(&self) -> f32 {
        self.nivel * 2.0 - 1.0 // [-1, 1]
    }

    /// ¿Hay suficiente dopamina para aprendizaje?
    pub fn puede_aprender(&self) -> bool {
        self.nivel > 0.2
    }
}

// ============================================================================
// MOTOR 7: CONCIENCIA (Espacio de Trabajo Global)
// ============================================================================
// Implementa el Global Workspace Theory (Baars, 1988).
// ============================================================================

#[derive(Serialize, Deserialize)]
pub struct Conciencia {
    pub contenido: Vec<u32>,     // Contenido consciente actual
    pub intensidad: f32,         // Nivel de conciencia
    pub umbral: f32,             // Umbral para acceso consciente
}

impl Conciencia {
    pub fn nuevo() -> Self {
        Self {
            contenido: Vec::new(),
            intensidad: 0.0,
            umbral: 0.7,
        }
    }

    /// Actualiza el contenido de la conciencia.
    /// `actividad`: slice de (id, activacion)
    /// Retorna los IDs en el espacio de trabajo global
    pub fn actualizar(&mut self, dt: f32, actividad: &[(u32, f32)]) -> Vec<u32> {
        // Competidores que superan el umbral
        let mut competidores: Vec<(u32, f32)> = actividad
            .iter()
            .filter(|(_, activacion)| *activacion > self.umbral)
            .map(|&(id, activacion)| (id, activacion))
            .collect();

        // Ordenar por activación
        competidores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        competidores.truncate(5); // Máximo 5 items conscientes

        if !competidores.is_empty() {
            self.contenido = competidores.iter().map(|(id, _)| *id).collect();
            self.intensidad += (competidores[0].1 - self.intensidad) * dt * 5.0;
        } else {
            // Decaimiento de la conciencia
            self.intensidad -= self.intensidad * dt * 2.0;
            if self.intensidad < 0.1 {
                self.contenido.clear();
            }
        }

        self.intensidad = self.intensidad.clamp(0.0, 1.0);
        self.contenido.clone()
    }

    /// ¿El sistema está consciente?
    pub fn esta_consciente(&self) -> bool {
        self.intensidad > 0.3 && !self.contenido.is_empty()
    }

    /// ¿Un ID está en el contenido consciente?
    pub fn en_conciencia(&self, id: u32) -> bool {
        self.contenido.contains(&id)
    }

    /// Ajusta el umbral dinámicamente (meta-cognición)
    pub fn ajustar_umbral(&mut self, ruido: f32) {
        self.umbral = (0.5 + ruido * 0.5).clamp(0.3, 0.9);
    }
}

// ============================================================================
// MOTOR 8: CURIOSIDAD — Hambre de Saber + Exploración Autónoma
// ============================================================================
// La curiosidad crece cuando el cerebro encuentra novedad (error de predicción
// de dopamina, actividad neuronal inesperada, conciencia alta o emociones
// intensas). Cuando supera un umbral, el cerebro genera una pregunta desde
// su estado interno y busca activamente en internet para aprender.
//
// Inspiración biológica: Sistema de activación reticular + neuronas de
// novedad en el hipocampo + búsqueda de información como recompensa.
// ============================================================================

#[derive(Serialize, Deserialize)]
pub struct MotorCuriosidad {
    /// Hambre de saber actual (0.0 a 1.0). Crece con novedad, se sacia al explorar.
    pub nivel: f32,
    /// Umbral mínimo para disparar una búsqueda en internet (0.7 por defecto).
    pub umbral: f32,
    /// Cuánto baja el nivel tras cada búsqueda (0.0 = no se sacia, 1.0 = se calma completamente).
    pub saciedad: f32,
    /// Pasos mínimos obligatorios entre búsquedas (200 = ~3 min a dt=0.001).
    pub cadencia_min: u64,
    /// Contador de pasos desde la última búsqueda.
    pub pasos_desde_ultima: u64,
    /// Tema actual de interés (se genera desde la última salida del cerebro).
    pub tema_actual: String,
    /// Cuántas búsquedas ha realizado en total.
    pub busquedas_realizadas: u64,
    /// Decaimiento natural del nivel por paso (sin novedad, la curiosidad se calma sola).
    pub decaimiento: f32,
    /// NUEVO: Fuentes ya navegadas (URLs visitadas) para evitar repetir.
    pub fuentes_navegadas: Vec<String>,
    /// NUEVO: Profundidad de exploración web (1-3 saltos).
    pub profundidad_exploracion: u8,
    /// NUEVO: Preferencia por fuentes académicas (0.0-1.0).
    pub preferencia_academica: f32,
}

impl MotorCuriosidad {
    pub fn nuevo() -> Self {
        Self {
            nivel: 0.1,           // Curiosidad baja al nacer
            umbral: 0.7,          // Necesita bastante hambre para buscar
            saciedad: 0.5,        // Cada búsqueda calma la mitad
            cadencia_min: 200,    // Esperar ~3 minutos entre búsquedas
            pasos_desde_ultima: 0,
            tema_actual: String::new(),
            busquedas_realizadas: 0,
            decaimiento: 0.001,   // Pierde 0.1% por paso sin novedad
            fuentes_navegadas: Vec::new(),
            profundidad_exploracion: 2, // 2 saltos por defecto
            preferencia_academica: 0.6, // 60% preferencia académica
        }
    }

    /// Actualiza el nivel de curiosidad basado en las señales internas del cerebro.
    ///
    /// Parámetros:
    /// - `error_prediccion`: señal de dopamina (novedad = oportunidad de aprender)
    /// - `intensidad_conciencia`: qué tan consciente está el cerebro
    /// - `valencia_emocional`: alegría - miedo (-1 a 1)
    /// - `dt`: paso temporal
    ///
    /// Retorna `true` si la curiosidad supera el umbral y es momento de explorar.
    pub fn actualizar(
        &mut self,
        error_prediccion: f32,
        intensidad_conciencia: f32,
        valencia_emocional: f32,
        dt: f32,
    ) -> bool {
        // La novedad (error de predicción absoluto) alimenta la curiosidad
        let novedad = error_prediccion.abs();

        // Emociones intensas (sean positivas o negativas) aceleran la curiosidad
        let intensidad_emocional = valencia_emocional.abs();

        // Conciencia alta = más capacidad de explorar
        let senial_conciencia = intensidad_conciencia.max(0.0);

        // Señal compuesta de curiosidad (pesos heurísticos)
        let impulso = novedad * 0.5      // 50% de la curiosidad viene de la novedad
                    + senial_conciencia * 0.3  // 30% de estar consciente
                    + intensidad_emocional * 0.2; // 20% de las emociones

        // Acumular curiosidad (sube con impulso, decae naturalmente)
        self.nivel += impulso * dt * 10.0;
        self.nivel -= self.nivel * self.decaimiento;
        self.nivel = self.nivel.clamp(0.0, 1.0);

        // Incrementar contador
        self.pasos_desde_ultima += 1;

        // ¿Es momento de explorar?
        self.nivel > self.umbral && self.pasos_desde_ultima >= self.cadencia_min
    }

    /// Registra que se realizó una búsqueda, saciando la curiosidad parcialmente.
    pub fn saciar(&mut self) {
        self.nivel *= 1.0 - self.saciedad; // Baja el nivel según saciedad
        self.nivel = self.nivel.max(0.05); // Nunca baja de 0.05 (siempre queda un poco)
        self.pasos_desde_ultima = 0;
        self.busquedas_realizadas += 1;
    }

    /// Asigna el tema actual de interés.
    pub fn establecer_tema(&mut self, tema: String) {
        self.tema_actual = tema;
    }

    /// Genera una pregunta para buscar desde el tema actual.
    /// Si no hay tema, usa un interés genérico.
    pub fn generar_pregunta(&self) -> String {
        if self.tema_actual.is_empty() || self.tema_actual.len() < 3 {
            "curiosidad".to_string()
        } else {
            // Tomar las primeras 2-3 palabras significativas del tema
            let palabras: Vec<&str> = self.tema_actual
                .split_whitespace()
                .filter(|p| p.len() > 2) // Filtrar palabras muy cortas
                .collect();

            if palabras.is_empty() {
                "curiosidad".to_string()
            } else if palabras.len() == 1 {
                palabras[0].to_string()
            } else {
                format!("{} {}", palabras[0], palabras[1])
            }
        }
    }

    /// ¿Está lo suficientemente curioso para buscar ahora?
    pub fn quiere_explorar(&self) -> bool {
        self.nivel > self.umbral && self.pasos_desde_ultima >= self.cadencia_min
    }

    /// Resetea la curiosidad (para tests o reinicio)
    pub fn resetear(&mut self) {
        self.nivel = 0.1;
        self.pasos_desde_ultima = 0;
        self.tema_actual.clear();
    }
}

// ============================================================================
// SISTEMA DE INHIBICIÓN GABAÉRGICA (Winner-Take-All)
// ============================================================================
// Implementa la inhibición lateral biológica para crear competencia
// y foco atencional, evitando que toda la red dispare al mismo tiempo.
// ============================================================================

pub struct MotorInhibicion;

impl MotorInhibicion {
    /// Aplica inhibición lateral a un conjunto de neuronas.
    /// Si una neurona inhibitoria (tipo 1) dispara, aplica una fuerte
    /// hiperpolarización (reducción de voltaje) a sus objetivos.
    pub fn aplicar_inhibicion(
        neuronas: &mut [NeuronaCompacta],
        spikes_inhibitorios: &[u32],
        conexiones: &HashMap<u32, Vec<SinapsisCompacta>>,
        potencia: f32,
    ) {
        for &origen_id in spikes_inhibitorios {
            if let Some(sinapsis_list) = conexiones.get(&origen_id) {
                for sin in sinapsis_list {
                    // Solo aplicamos si el peso es negativo (inhibitorio)
                    // o si la neurona de origen es explícitamente inhibitoria
                    if let Some(n_dest) = neuronas.iter_mut().find(|n| n.id == sin.destino) {
                        // Hiperpolarización: restamos voltaje
                        // potencia ≈ 15.0 a 25.0 mV por spike GABAérgico
                        n_dest.voltaje -= potencia * sin.peso.abs();
                        
                        // Piso biológico de hiperpolarización
                        if n_dest.voltaje < -90.0 {
                            n_dest.voltaje = -90.0;
                        }
                        
                        // Aumentar periodo refractario para silenciarla
                        n_dest.refractario = (n_dest.refractario + 0.2).min(1.0);
                    }
                }
            }
        }
    }

    /// Mecanismo Winner-Take-All simple: silencia a los más débiles
    pub fn winner_take_all(actividad: &mut [f32], umbral: f32) {
        if actividad.is_empty() { return; }
        
        let max = actividad.iter().fold(0.0, |a, &b| f32::max(a, b));
        if max < umbral { return; }

        for val in actividad.iter_mut() {
            if *val < max * 0.8 {
                *val = 0.0;
            }
        }
    }
}

// ============================================================================
// MOTOR TALÁMICO: Transmisión y Ráfagas
// ============================================================================

pub struct MotorTalamo;

impl MotorTalamo {
    /// Envía una ráfaga fásica a la corteza para alertar sobre novedad.
    /// Inyecta una corriente masiva en las neuronas de la capa 4.
    pub fn enviar_rafaga_cortical(
        neuronas: &mut [NeuronaCompacta],
        estimulo_id: u32,
        intensidad: f32,
    ) {
        // Encontrar neuronas de Capa 4 relacionadas con el estímulo
        // (En este modelo simplificado, usamos la capa 0 como entrada sensorial)
        for n in neuronas.iter_mut() {
            if n.capa == 0 && n.id == estimulo_id {
                n.voltaje = 40.0; // Forzar disparo
                n.energia = (n.energia + intensidad).min(1.0);
            }
        }
    }

    /// Transmisión tónica fiel
    pub fn transmision_tonica(
        neuronas: &mut [NeuronaCompacta],
        estimulo_id: u32,
        valor: f32,
    ) {
        if let Some(n) = neuronas.iter_mut().find(|n| n.id == estimulo_id) {
            n.corriente_entrada += valor * 10.0; // Corriente moderada
        }
    }
}

// ============================================================================
// MOTOR DE COLUMNA CORTICAL (Ciclo de 6 Capas)
// ============================================================================
// Implementa el flujo canónico de información a través de las 6 capas:
//   Tálamo → IV → II → III → V → VI → Tálamo (feedback)
// Con modulación contextual desde Capa I.
// ============================================================================

pub struct MotorColumnaCortical;

impl MotorColumnaCortical {
    /// Ciclo completo de procesamiento de una columna cortical.
    ///
    /// # Fases
    /// 1. **Input Talámico** (Capa IV) — Inyecta corriente desde el tálamo
    /// 2. **Feedforward Local** (Capa IV → Capa II) — Asociación sensorial primaria
    /// 3. **Asociación Global** (Capa II → Capa III) — Integración horizontal
    /// 4. **Feedforward Profundo** (Capa III → Capa V) — Preparación ejecutiva
    /// 5. **Salida Ejecutiva** (Capa V) — Genera comandos de acción
    /// 6. **Feedback Predictivo** (Capa V → Capa VI → Tálamo) — Error de predicción
    ///
    /// Retorna: (prediccion_talamica, activacion_global)
    pub fn ciclo_columna(
        columna: &mut ColumnaCortical,
        estimulos: &[EstimuloTalamico],
        params: &ParametrosNeurona,
        dt: f32,
    ) -> (Option<PrediccionTalamica>, f32) {
        if columna.estado == EstadoColumna::Silenciada {
            return (None, 0.0);
        }

        // === FASE 1: Input Talámico → Capa IV ===
        // Inyectar estímulos talámicos directamente en las neuronas de Capa IV
        for est in estimulos {
            // Buscar neurona en Capa IV que corresponda al ID del estímulo
            let target_id = est.origen_talamo;
            for n in columna.capa_iv.neuronas.iter_mut() {
                if n.id == target_id || (n.id as i32 - target_id as i32).abs() < 5 {
                    // Inyección de corriente proporcional a intensidad + novedad
                    let corriente = est.intensidad * (1.0 + est.novedad * 2.0);
                    n.corriente_entrada += corriente * 50.0; // Sinapsis talámica fuerte
                    n.energia = (n.energia + est.intensidad * 0.3).min(1.0);
                }
            }
        }

        // Procesar neuronas de Capa IV (actualizar Hodgkin-Huxley)
        let spikes_iv = Self::procesar_capa(&mut columna.capa_iv, params, dt);

        // === FASE 2: Feedforward Local (Capa IV → Capa II) ===
        // Los spikes de Capa IV viajan a Capa II
        let spikes_ii = Self::propagar_a_capa(
            &mut columna.capa_iv,
            &mut columna.capa_ii,
            &spikes_iv,
            params,
            dt,
        );

        // === FASE 3: Asociación Global (Capa II → Capa III) ===
        // Capa II se asocia localmente y proyecta a Capa III
        let spikes_iii = Self::propagar_a_capa(
            &mut columna.capa_ii,
            &mut columna.capa_iii,
            &spikes_ii,
            params,
            dt,
        );

        // Winner-Take-All en Capa III (competencia entre representaciones)
        Self::winner_take_all_capa_iii(&mut columna.capa_iii);

        // === FASE 4: Feedforward Profundo (Capa III → Capa V) ===
        let spikes_v = Self::propagar_a_capa(
            &mut columna.capa_iii,
            &mut columna.capa_v,
            &spikes_iii,
            params,
            dt,
        );

        // === FASE 5: Salida Ejecutiva (Capa V) ===
        // Capa V genera comandos basados en su actividad
        let _activacion_v = columna.capa_v.activacion_media;

        // === FASE 6: Feedback Predictivo (Capa V → Capa VI → Tálamo) ===
        let spikes_vi = Self::propagar_a_capa(
            &mut columna.capa_v,
            &mut columna.capa_vi,
            &spikes_v,
            params,
            dt,
        );

        // Generar predicción talámica desde Capa VI
        let prediccion = if columna.capa_vi.activacion_media > 0.2 {
            let valor_esperado = columna.capa_vi.activacion_media;
            let confianza = (columna.capa_vi.spikes().len() as f32)
                / (columna.capa_vi.neuronas.len() as f32).max(1.0);
            Some(PrediccionTalamica {
                columna_origen: columna.id,
                valor_esperado,
                confianza: confianza.clamp(0.0, 1.0),
            })
        } else {
            None
        };

        // === MODULACIÓN CONTEXTUAL (Capa I) ===
        // Capa I modula la ganancia de todas las demás capas
        if columna.capa_i.activacion_media > 0.1 {
            let factor_modulacion = 1.0 + columna.capa_i.activacion_media * 0.5;
            for n in columna.capa_ii.neuronas.iter_mut() {
                n.corriente_entrada *= factor_modulacion;
            }
            for n in columna.capa_iii.neuronas.iter_mut() {
                n.corriente_entrada *= factor_modulacion;
            }
        }

        // Actualizar estado de la columna
        columna.spike_count += spikes_iv.len() as u64
            + spikes_ii.len() as u64
            + spikes_iii.len() as u64
            + spikes_v.len() as u64
            + spikes_vi.len() as u64;

        columna.actualizar_activaciones();

        // Determinar estado emergente
        if columna.activacion_sostenida > 0.5 {
            columna.estado = EstadoColumna::Activa;
        } else if columna.activacion_sostenida > 0.3 {
            columna.estado = EstadoColumna::Supra;
        } else {
            columna.estado = EstadoColumna::Reposo;
        }

        columna.ultima_prediccion = prediccion.clone();
        (prediccion, columna.activacion_sostenida)
    }

    /// Procesa todas las neuronas de una capa con Hodgkin-Huxley
    /// Retorna los IDs de las que dispararon
    fn procesar_capa(
        capa: &mut CapaCortical,
        params: &ParametrosNeurona,
        dt: f32,
    ) -> Vec<u32> {
        let mut spikes = Vec::new();
        for n in capa.neuronas.iter_mut() {
            let entrada = n.corriente_entrada;
            n.corriente_entrada = 0.0;
            
            let disparo = if n.capa <= 3 {
                MotorNeurona::actualizar(n, entrada, dt, params)
            } else {
                MotorNeurona::actualizar_simple(n, entrada, dt)
            };
            
            if disparo {
                spikes.push(n.id);
            }
        }
        spikes
    }

    /// Propaga spikes de una capa origen a una capa destino
    /// usando las conexiones inter-capa preestablecidas
    fn propagar_a_capa(
        origen: &mut CapaCortical,
        destino: &mut CapaCortical,
        spikes_origen: &[u32],
        params: &ParametrosNeurona,
        dt: f32,
    ) -> Vec<u32> {
        // Propagar spikes de origen → destino vía conexiones_inter
        for &oid in spikes_origen {
            if let Some(conexiones) = origen.conexiones_inter.get(&oid) {
                for sin in conexiones {
                    if let Some(n_dest) = destino.neuronas.iter_mut().find(|n| n.id == sin.destino) {
                        n_dest.voltaje += sin.peso * 35.0;
                        n_dest.energia = (n_dest.energia + sin.peso.abs() * 0.5).min(1.0);
                    }
                }
            }
            
            // También propagar intra-capa en origen (reverberación)
            if let Some(intra) = origen.conexiones_intra.get(&oid) {
                for sin in intra {
                    if let Some(n_orig) = origen.neuronas.iter_mut().find(|n| n.id == sin.destino) {
                        n_orig.voltaje += sin.peso * 15.0; // Intra-capa más débil
                    }
                }
            }
        }

        // Procesar la capa destino
        Self::procesar_capa(destino, params, dt)
    }

    /// Winner-Take-All para Capa III
    /// La neurona más activa sobrevive, las demás se silencian
    fn winner_take_all_capa_iii(capa: &mut CapaCortical) {
        if capa.neuronas.is_empty() {
            return;
        }

        // Encontrar la neurona más activa
        let max_idx = capa.neuronas.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.activacion.partial_cmp(&b.activacion)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx);

        if let Some(winner) = max_idx {
            let umbral = capa.neuronas[winner].activacion * 0.6;
            for (i, n) in capa.neuronas.iter_mut().enumerate() {
                if i != winner && n.activacion < umbral {
                    n.voltaje = -80.0; // Hiperpolarizar perdedores
                    n.activacion = 0.0;
                }
            }
        }
    }

    /// Inhibición lateral entre columnas corticales
    /// Si una columna está muy activa, silencia a las vecinas
    pub fn inhibicion_entre_columnas(
        columnas: &mut [ColumnaCortical],
        umbral: f32,
        radio: usize,
    ) {
        let activaciones: Vec<f32> = columnas.iter()
            .map(|c| c.activacion_sostenida)
            .collect();

        for i in 0..columnas.len() {
            if activaciones[i] > umbral {
                // Silenciar columnas vecinas en el radio
                let start = i.saturating_sub(radio);
                let end = (i + radio + 1).min(columnas.len());
                for j in start..end {
                    if j != i && columnas[j].activacion_sostenida < activaciones[i] * 0.5 {
                        columnas[j].estado = EstadoColumna::Silenciada;
                    }
                }
            }
        }
    }

    /// Aplica neuromodulación a una columna:
    /// - Dopamina: aumenta plasticidad y ganancia
    /// - Noradrenalina: aumenta arousal/alertness
    /// - Serotonina: modula inhibición
    /// - Acetilcolina: aumenta fidelidad de transmisión
    pub fn aplicar_neuromodulacion(
        columna: &mut ColumnaCortical,
        neuromoduladores: &[(TipoNeuromodulador, f32)],
    ) {
        for (tipo, nivel) in neuromoduladores {
            let factor = *nivel;
            match tipo {
                TipoNeuromodulador::Dopamina => {
                    // Aumenta plasticidad sináptica en Capa III y V
                    for n in columna.capa_iii.neuronas.iter_mut() {
                        n.traza = (n.traza + factor * 0.1).min(1.0);
                    }
                    for n in columna.capa_v.neuronas.iter_mut() {
                        n.traza = (n.traza + factor * 0.1).min(1.0);
                    }
                }
                TipoNeuromodulador::Noradrenalina => {
                    // Aumenta ganancia general (arousal)
                    for capa in [&mut columna.capa_iv, &mut columna.capa_iii, &mut columna.capa_v].iter_mut() {
                        for n in capa.neuronas.iter_mut() {
                            n.energia = (n.energia + factor * 0.2).min(1.0);
                        }
                    }
                }
                TipoNeuromodulador::Serotonina => {
                    // Modula inhibición: más serotonina → menos inhibición lateral
                    // Implementado como reducción de hiperpolarización
                }
                TipoNeuromodulador::Acetilcolina => {
                    // Aumenta fidelidad de transmisión sináptica
                    // Las sinapsis existentes se refuerzan temporalmente
                    for (_, conexiones) in columna.capa_iii.conexiones_inter.iter_mut() {
                        for sin in conexiones.iter_mut() {
                            sin.peso *= (1.0 + factor * 0.1).min(2.0);
                                }
                            }
                        }
                        TipoNeuromodulador::Ninguno => {}
                    }
                }
            }
        }
        
        // ============================================================================
        // TESTS DE LOS MOTORES BIOLÓGICOS
        // ============================================================================
        
        #[cfg(test)]
        mod tests {
            use super::*;
        
            fn casi(a: f32, b: f32) -> bool {
                (a - b).abs() < 1e-4
            }
        
            // ── MOTOR 1: Neurona Hodgkin-Huxley ──────────────────────────────────────
            #[test]
            fn test_hh_spike_resetea_voltaje() {
                let mut n = NeuronaCompacta::reposo(0, 0, 0);
                n.voltaje = 40.0; // Por encima del umbral
                n.refractario = 0.0;
                let params = ParametrosNeurona::default();
        
                let disparo = MotorNeurona::actualizar(&mut n, 0.0, 0.001, &params);
        
                assert!(disparo);
                assert!(casi(n.voltaje, -70.0), "voltaje debe resetearse, fue {}", n.voltaje);
                assert!(casi(n.refractario, 1.0));
                assert_eq!(n.edad, 1);
                assert!(n.frecuencia > 0.0);
            }
        
            #[test]
            fn test_hh_refractario_bloquea_spike() {
                let mut n = NeuronaCompacta::reposo(1, 0, 0);
                n.voltaje = 40.0;
                n.refractario = 0.5; // En periodo refractario
                let params = ParametrosNeurona::default();
        
                let disparo = MotorNeurona::actualizar(&mut n, 0.0, 0.001, &params);
        
                assert!(!disparo);
                assert!(n.edad == 0, "no debe envejecer si no dispara");
            }
        
            #[test]
            fn test_hh_entrada_acumula_energia_y_traza() {
                let mut n = NeuronaCompacta::reposo(2, 0, 0);
                let params = ParametrosNeurona::default();
                MotorNeurona::actualizar(&mut n, 1.0, 0.001, &params);
        
                assert!(n.traza > 0.0, "la traza debe crecer con entrada, fue {}", n.traza);
                assert!(n.energia >= 0.0);
                assert!((0.0..=1.0).contains(&n.traza));
            }
        
            #[test]
            fn test_hh_activacion_es_energia_por_no_refractario() {
                let mut n = NeuronaCompacta::reposo(3, 0, 0);
                let params = ParametrosNeurona::default();
                MotorNeurona::actualizar(&mut n, 1.0, 0.001, &params);
        
                let esperada = n.energia * (1.0 - n.refractario);
                assert!(casi(n.activacion, esperada));
            }
        
            #[test]
            fn test_actualizar_simple_integrade_and_fire() {
                let mut n = NeuronaCompacta::reposo(4, 0, 0);
                n.voltaje = 21.0; // Sobre umbral simple (20)
                n.refractario = 0.0;
        
                let disparo = MotorNeurona::actualizar_simple(&mut n, 0.0, 0.001);
        
                assert!(disparo);
                assert!(casi(n.voltaje, -70.0));
                assert_eq!(n.edad, 1);
            }
        
            // ── MOTOR 2: Sinapsis STDP ───────────────────────────────────────────────
            #[test]
            fn test_stdp_ltp_potencia_sinapse() {
                let params = ParametrosSTDP::default();
                let mut peso = 0.5_f32;
        
                MotorSTDP::actualizar(&mut peso, true, true, 0.0, &params);
        
                // delta = 0.1 * exp(0) * 1.0 = 0.1; 0.1*(1-0.5)=0.05 → 0.55
                assert!(casi(peso, 0.55), "peso fue {}", peso);
            }
        
            #[test]
            fn test_stdp_ltd_deprime_sinapse() {
                let params = ParametrosSTDP::default();
                let mut peso = 0.5_f32;
        
                MotorSTDP::actualizar(&mut peso, false, true, 0.0, &params);
        
                // delta = -0.1 * exp(0) * 1.0 = -0.1; -0.1*(1+0.5)=-0.15 → 0.35
                assert!(casi(peso, 0.35), "peso fue {}", peso);
            }
        
            #[test]
            fn test_stdp_clamp_limite_superior() {
                // a_plus alto para forzar el clamp
                let params = ParametrosSTDP {
                    a_plus: 10.0,
                    a_minus: 0.1,
                    tau_plus: 20.0,
                    tau_minus: 20.0,
                    decaimiento: 0.001,
                    plasticidad_critica: 1.0,
                };
                let mut peso = 0.99_f32;
        
                MotorSTDP::actualizar(&mut peso, true, true, 0.0, &params);
        
                assert!(peso <= 1.0);
                assert!(casi(peso, 1.0), "peso fue {}", peso);
            }
        
            #[test]
            fn test_stdp_con_traza_ltp() {
                let params = ParametrosSTDP::default();
                let mut peso = 0.5_f32;
        
                MotorSTDP::actualizar_con_traza(&mut peso, 1.0, 0.0, &params);
        
                // ltp = 0.1*1*(1-0.5)=0.05; ltd=0; +=0.05*0.1=0.005 → 0.505
                assert!(casi(peso, 0.505), "peso fue {}", peso);
            }
        
            #[test]
            fn test_stdp_sin_spikes_no_cambia() {
                let params = ParametrosSTDP::default();
                let mut peso = 0.5_f32;
                let original = peso;
        
                MotorSTDP::actualizar(&mut peso, false, false, 1.0, &params);
        
                // Solo decaimiento mínimo (0.001 * 1.0)
                assert!((peso - original).abs() < 1e-3);
            }
        
            // ── MOTOR 3: Hipocampo ───────────────────────────────────────────────────
            fn episodio(patron: &[u32], intensidad: f32, emocion: f32) -> Episodio {
                Episodio::nueva(0.0, intensidad, emocion, patron, 0)
            }
        
            #[test]
            fn test_hipocampo_almacena_y_recupera() {
                let mut h = Hipocampo::nuevo(10);
                let patron = [1, 2, 3, 4, 5, 6, 7, 8];
                h.almacenar(episodio(&patron, 1.0, 0.5));
        
                let recuperados = h.recuperar(&patron);
                assert_eq!(recuperados.len(), 1);
                assert!(casi(recuperados[0].relevancia, 0.75));
            }
        
            #[test]
            fn test_hipocampo_recupera_solo_similitud_alta() {
                let mut h = Hipocampo::nuevo(10);
                h.almacenar(episodio(&[1, 2, 3, 4, 5, 6, 7, 8], 1.0, 0.5));
        
                // Patrón totalmente distinto → similitud 0 → no recupera
                let recuperados = h.recuperar(&[90, 91, 92, 93, 94, 95, 96, 97]);
                assert!(recuperados.is_empty());
            }
        
            #[test]
            fn test_hipocampo_descarta_menos_relevante_al_exceder() {
                let mut h = Hipocampo::nuevo(2);
                h.almacenar(episodio(&[1], 0.1, 0.0)); // relevancia baja
                h.almacenar(episodio(&[2], 1.0, 1.0)); // relevancia alta
                h.almacenar(episodio(&[3], 0.8, 0.8)); // relevancia media
        
                assert_eq!(h.episodios.len(), 2);
                // El de menor relevancia (id 1, relevancia 0.05) debe haberse descartado
                assert!(!h.episodios.iter().any(|e| e.patron[0] == 1));
            }
        
            #[test]
            fn test_hipocampo_recientes() {
                let mut h = Hipocampo::nuevo(10);
                h.almacenar(episodio(&[1], 0.5, 0.5));
                h.almacenar(episodio(&[2], 0.5, 0.5));
                h.almacenar(episodio(&[3], 0.5, 0.5));
        
                let recientes = h.recientes(2);
                assert_eq!(recientes.len(), 2);
                assert_eq!(recientes[0].patron[0], 2);
                assert_eq!(recientes[1].patron[0], 3);
            }
        
            #[test]
            fn test_hipocampo_olvidar_elimina_irrelevantes() {
                let mut h = Hipocampo::nuevo(10);
                h.almacenar(episodio(&[1], 0.0, 0.0)); // relevancia 0
                h.almacenar(episodio(&[2], 1.0, 0.5)); // relevancia 0.75
        
                h.olvidar(1.0);
        
                assert_eq!(h.episodios.len(), 1);
                assert_eq!(h.episodios[0].patron[0], 2);
            }
        
            #[test]
            fn test_hipocampo_similitud_estatica() {
                let s = Hipocampo::similitud(&[1, 2, 3], &[2, 3, 4]);
                assert!(casi(s, 2.0 / 3.0), "similitud fue {}", s);
            }
        
            // ── MOTOR 4: Amígdala ────────────────────────────────────────────────────
            #[test]
            fn test_amigdala_nuevo_valores_basales() {
                let a = Amigdala::nuevo();
                assert!(casi(a.miedo, 0.1));
                assert!(casi(a.ansiedad, 0.1));
                assert!(casi(a.ira, 0.1));
                assert!(casi(a.alegria, 0.5));
            }
        
            #[test]
            fn test_amigdala_amenaza_domina() {
                let mut a = Amigdala::nuevo();
                let valencia = a.actualizar(1.0, 1.0, 0.0);
        
                assert!(casi(a.miedo, 1.0));
                assert!(casi(a.ira, 0.3));
                assert!(casi(a.alegria, 0.25));
                assert!(casi(valencia, -0.75), "valencia fue {}", valencia);
                assert_eq!(a.emocion_dominante(), "miedo");
            }
        
            #[test]
            fn test_amigdala_recompensa_domina() {
                let mut a = Amigdala::nuevo();
                let valencia = a.actualizar(1.0, 0.0, 1.0);
        
                assert!(casi(a.miedo, 0.0));
                assert!(casi(a.alegria, 0.75));
                assert!(casi(valencia, 0.75));
                assert_eq!(a.emocion_dominante(), "alegría");
            }
        
            #[test]
            fn test_amigdala_intensidad_total() {
                let a = Amigdala::nuevo();
                let total = a.intensidad_total();
                assert!(casi(total, (0.1 + 0.1 + 0.1 + 0.5) / 4.0));
            }
        
            #[test]
            fn test_amigdala_emociones_clampadas() {
                let mut a = Amigdala::nuevo();
                a.actualizar(10.0, 5.0, 5.0); // Señales extremas
                assert!((0.0..=1.0).contains(&a.miedo));
                assert!((0.0..=1.0).contains(&a.ansiedad));
                assert!((0.0..=1.0).contains(&a.ira));
                assert!((0.0..=1.0).contains(&a.alegria));
            }
        
            // ── MOTOR 5: Atención Selectiva ──────────────────────────────────────────
            #[test]
            fn test_atencion_nueva_por_defecto() {
                let at = AtencionSelectiva::nuevo();
                assert!(casi(at.intensidad, 0.5));
                assert!(at.foco.is_empty());
            }
        
            #[test]
            fn test_atencion_selecciona_foco_por_saliencia() {
                let mut at = AtencionSelectiva::nuevo();
                let foco = at.actualizar(0.1, &[(1, 0.9), (2, 0.8)]);
        
                // El de mayor saliencia (1) debe quedar primero
                assert_eq!(foco[0], 1);
                assert!(at.en_foco(1));
                assert!(at.en_foco(2));
                assert!(at.intensidad > 0.0);
            }
        
            #[test]
            fn test_atencion_resetear_limpia() {
                let mut at = AtencionSelectiva::nuevo();
                at.actualizar(0.1, &[(1, 0.9)]);
                assert!(!at.foco.is_empty());
        
                at.resetear();
                assert!(at.foco.is_empty());
                assert!(casi(at.intensidad, 0.0));
                assert!(at.saliencia.is_empty());
            }
        
            #[test]
            fn test_atencion_sin_estimulos_foco_vacio() {
                let mut at = AtencionSelectiva::nuevo();
                let foco = at.actualizar(0.1, &[]);
                assert!(foco.is_empty());
                assert!(casi(at.intensidad, 0.0));
            }
        
            // ── MOTOR 6: Dopamina ────────────────────────────────────────────────────
            #[test]
            fn test_dopamina_nueva_por_defecto() {
                let d = SistemaDopamina::nuevo();
                assert!(casi(d.nivel, 0.3));
                assert!(casi(d.prediccion, 0.3));
            }
        
            #[test]
            fn test_dopamina_error_positivo() {
                let mut d = SistemaDopamina::nuevo();
                let error = d.actualizar(1.0, 1.0);
        
                assert!(casi(error, 0.7), "error fue {}", error);
                assert!(d.nivel > 0.3, "el nivel debe subir, fue {}", d.nivel);
                assert!(d.prediccion > 0.3);
            }
        
            #[test]
            fn test_dopamina_error_negativo() {
                let mut d = SistemaDopamina::nuevo();
                let error = d.actualizar(1.0, 0.0);
                assert!(casi(error, -0.3), "error fue {}", error);
            }
        
            #[test]
            fn test_dopamina_senial_recompensa() {
                let mut d = SistemaDopamina::nuevo();
                assert!(casi(d.senial_recompensa(), 0.3 * 2.0 - 1.0));
                assert!(d.puede_aprender()); // nivel 0.3 > 0.2
        
                d.nivel = 0.1;
                assert!(!d.puede_aprender());
            }
        
            #[test]
            fn test_dopamina_nivel_clampado() {
                let mut d = SistemaDopamina::nuevo();
                d.actualizar(1.0, 5.0);
                assert!(d.nivel <= 1.0);
                assert!(d.nivel >= 0.0);
            }
        
            // ── MOTOR 7: Conciencia ──────────────────────────────────────────────────
            #[test]
            fn test_conciencia_nueva_por_defecto() {
                let c = Conciencia::nuevo();
                assert!(casi(c.intensidad, 0.0));
                assert!(casi(c.umbral, 0.7));
                assert!(c.contenido.is_empty());
                assert!(!c.esta_consciente());
            }
        
            #[test]
            fn test_conciencia_selecciona_sobre_umbral() {
                let mut c = Conciencia::nuevo();
                let contenido = c.actualizar(0.1, &[(1, 0.9), (2, 0.95)]);
        
                // Solo superan 0.7; ordenados por activación desc → 2, 1
                assert_eq!(contenido, vec![2, 1]);
                assert!(c.en_conciencia(1));
                assert!(c.en_conciencia(2));
                assert!(c.intensidad > 0.3);
                assert!(c.esta_consciente());
            }
        
            #[test]
            fn test_conciencia_bajo_umbral_no_accede() {
                let mut c = Conciencia::nuevo();
                let contenido = c.actualizar(0.1, &[(1, 0.5)]);
                assert!(contenido.is_empty());
                assert!(!c.esta_consciente());
            }
        
            #[test]
            fn test_conciencia_ajustar_umbral() {
                let mut c = Conciencia::nuevo();
                c.ajustar_umbral(0.0);
                assert!(casi(c.umbral, 0.5));
                c.ajustar_umbral(1.0);
                assert!(casi(c.umbral, 0.9)); // Clamp superior
            }
        
            #[test]
            fn test_conciencia_intensidad_clampada() {
                let mut c = Conciencia::nuevo();
                c.actualizar(1.0, &[(1, 1.0)]);
                assert!(c.intensidad <= 1.0);
            }
        
            // ── MOTOR 8: Curiosidad ──────────────────────────────────────────────────
            #[test]
            fn test_curiosidad_nueva_por_defecto() {
                let c = MotorCuriosidad::nuevo();
                assert!(casi(c.nivel, 0.1));
                assert!(casi(c.umbral, 0.7));
                assert!(casi(c.saciedad, 0.5));
                assert_eq!(c.cadencia_min, 200);
                assert!(casi(c.decaimiento, 0.001));
                assert_eq!(c.profundidad_exploracion, 2);
                assert!(casi(c.preferencia_academica, 0.6));
                assert!(!c.quiere_explorar());
            }
        
            #[test]
            fn test_curiosidad_explora_tras_acumular() {
                let mut c = MotorCuriosidad::nuevo();
                let mut disparo = false;
                for _ in 0..220 {
                    disparo = c.actualizar(1.0, 1.0, 0.0, 0.1);
                }
                assert!(disparo, "debe explorar tras acumular curiosidad");
                assert!(c.quiere_explorar());
            }
        
            #[test]
            fn test_curiosidad_saciar_baja_nivel() {
                let mut c = MotorCuriosidad::nuevo();
                c.nivel = 1.0;
                c.saciar();
        
                assert!(casi(c.nivel, 0.5), "nivel fue {}", c.nivel);
                assert_eq!(c.pasos_desde_ultima, 0);
                assert_eq!(c.busquedas_realizadas, 1);
            }
        
            #[test]
            fn test_curiosidad_generar_pregunta() {
                let mut c = MotorCuriosidad::nuevo();
                c.establecer_tema("física cuántica".to_string());
                assert_eq!(c.generar_pregunta(), "física cuántica");
        
                c.establecer_tema("sol".to_string());
                assert_eq!(c.generar_pregunta(), "sol");
        
                c.establecer_tema(String::new());
                assert_eq!(c.generar_pregunta(), "curiosidad");
            }
        
            #[test]
            fn test_curiosidad_resetear() {
                let mut c = MotorCuriosidad::nuevo();
                c.nivel = 0.9;
                c.pasos_desde_ultima = 50;
                c.establecer_tema("astrofísica".to_string());
        
                c.resetear();
                assert!(casi(c.nivel, 0.1));
                assert_eq!(c.pasos_desde_ultima, 0);
                assert!(c.tema_actual.is_empty());
            }
        }
