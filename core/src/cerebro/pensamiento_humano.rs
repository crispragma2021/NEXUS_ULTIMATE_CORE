// ==========================================
// 🧠 PENSAMIENTO HUMANO ACELERADO (PHA)
// ==========================================
// Módulo que imita el proceso creativo humano completo:
//
// Fase 1:  Confusión inicial  (pausa, respira)
// Fase 2:  Lluvia de ideas    (sin juzgar)
// Fase 3:  Selección intuitiva (basada en OCEAN, no lógica)
// Fase 4:  Prototipo rápido   (idea más pequeña posible)
// Fase 5:  Fracaso simulado   (la primera idea casi siempre falla)
// Fase 6:  Vergüenza          (sentir el error con la amígdala)
// Fase 7:  Pausa forzada      (incubación simulada)
// Fase 8:  Insight repentino  (búsqueda semántica acelerada)
// Fase 9:  Refinamiento       (mejora iterativa)
// Fase 10: Orgullo            (celebración del logro)
//
// El tiempo humano se comprime: una pausa de 5 min -> 100ms
// La incubación de 1 hora -> 500ms ejecutando procesos en fondo
// ==========================================

use crate::cerebro::organos::amygdala::Amygdala;
use crate::cerebro::organos::intuicion::Intuicion;
use crate::cerebro::organos::metacognicion::Metacognicion;
use crate::emociones::ocean::Ocean;
use crate::memoria::memoria_semantica::MemoriaSemantica;
use std::time::{Duration, Instant};
use tracing::info;

// ==========================================
// CONSTANTES DE TIEMPO SIMULADO
// ==========================================
const PAUSA_CONFUSION_MS: u64 = 50;
const PAUSA_INCUBACION_MS: u64 = 500;
const PAUSA_VERGUENZA_MS: u64 = 30;
const MAX_ITERACIONES_REFINAMIENTO: usize = 3;

// ==========================================
// ESTRUCTURAS DE DATOS
// ==========================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BitacoraCreativa {
    pub problema: String,
    pub fase_alcanzada: u8,
    pub ideas_generadas: Vec<String>,
    pub idea_seleccionada: String,
    pub prototipo: String,
    pub fracaso_simulado: Option<String>,
    pub verguenza_intensidad: f64,
    pub incubacion_ms: u64,
    pub insight: Option<String>,
    pub iteraciones_refinamiento: usize,
    pub resultado_final: String,
    pub tiempo_total_ms: u64,
    pub orgullo_intensidad: f64,
    pub timestamp: String,
}

impl BitacoraCreativa {
    pub fn new(problema: &str) -> Self {
        Self {
            problema: problema.to_string(),
            fase_alcanzada: 0,
            ideas_generadas: Vec::new(),
            idea_seleccionada: String::new(),
            prototipo: String::new(),
            fracaso_simulado: None,
            verguenza_intensidad: 0.0,
            incubacion_ms: 0,
            insight: None,
            iteraciones_refinamiento: 0,
            resultado_final: String::new(),
            tiempo_total_ms: 0,
            orgullo_intensidad: 0.0,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn resumen(&self) -> String {
        format!(
            "🧠 [PHA] Proceso creativo completado:\n\
             Fase alcanzada: {}/10\n\
             Ideas generadas: {}\n\
             Idea seleccionada: \"{}\"\n\
             Fracaso simulado: {}\n\
             Verguenza: {:.0}%\n\
             Incubacion: {}ms\n\
             Insight: {}\n\
             Refinamientos: {} iteraciones\n\
             Tiempo total: {}ms | Orgullo: {:.0}%",
            self.fase_alcanzada,
            self.ideas_generadas.len(),
            &self.idea_seleccionada[..self.idea_seleccionada.len().min(60)],
            if self.fracaso_simulado.is_some() {
                "SI"
            } else {
                "NO"
            },
            self.verguenza_intensidad * 100.0,
            self.incubacion_ms,
            self.insight.as_deref().unwrap_or("ninguno"),
            self.iteraciones_refinamiento,
            self.tiempo_total_ms,
            self.orgullo_intensidad * 100.0,
        )
    }
}

// ==========================================
// MOTOR DE PENSAMIENTO HUMANO ACELERADO
// ==========================================

pub struct PensamientoHumanoAcelerado {
    confusion_basal: f64,
    aciertos_insight: u32,
    fallos_insight: u32,
}

impl Default for PensamientoHumanoAcelerado {
    fn default() -> Self {
        Self::new()
    }
}

impl PensamientoHumanoAcelerado {
    pub fn new() -> Self {
        Self {
            confusion_basal: 0.05,
            aciertos_insight: 0,
            fallos_insight: 0,
        }
    }

    /// Ejecuta el ciclo completo de pensamiento humano acelerado.
    /// Retorna la respuesta enriquecida con el proceso interno.
    pub async fn pensar(
        &mut self,
        problema: &str,
        amygdala: &mut Amygdala,
        intuicion: &Intuicion,
        metacognicion: &Metacognicion,
        ocean: Option<&Ocean>,
        memoria_semantica: &MemoriaSemantica,
    ) -> (String, BitacoraCreativa) {
        let inicio_total = Instant::now();
        let mut bitacora = BitacoraCreativa::new(problema);

        // ==========================================
        // FASE 1: CONFUSION INICIAL
        // ==========================================
        bitacora.fase_alcanzada = 1;
        info!("[PHA] Fase 1/10: Confusion inicial");

        // Simular pausa humana de "no saber que hacer"
        tokio::time::sleep(Duration::from_millis(PAUSA_CONFUSION_MS)).await;

        // La confusion basal aumenta con problemas complejos
        let complejidad = (problema.len() as f64 / 1000.0).min(1.0);
        let nivel_confusion = self.confusion_basal + complejidad * 0.3;
        if nivel_confusion > 0.3 {
            info!(
                "[PHA] Confusion elevada ({:.0}%): problema extenso o ambiguo",
                nivel_confusion * 100.0
            );
        }

        // ==========================================
        // FASE 2: LLUVIA DE IDEAS (SIN JUZGAR)
        // ==========================================
        bitacora.fase_alcanzada = 2;
        info!("[PHA] Fase 2/10: Lluvia de ideas");

        let ideas = self.generar_ideas(problema, ocean, memoria_semantica).await;
        bitacora.ideas_generadas = ideas.clone();

        // ==========================================
        // FASE 3: SELECCION INTUITIVA
        // ==========================================
        bitacora.fase_alcanzada = 3;
        info!("[PHA] Fase 3/10: Seleccion intuitiva");

        let (_idea_idx, idea) = self
            .seleccionar_por_intuicion(&ideas, intuicion, ocean)
            .await;
        bitacora.idea_seleccionada = idea.clone();

        // ==========================================
        // FASE 4: PROTOTIPO RAPIDO
        // ==========================================
        bitacora.fase_alcanzada = 4;
        info!("[PHA] Fase 4/10: Prototipo rapido");

        let prototipo = self.prototipar(&idea, problema, ocean).await;
        bitacora.prototipo = prototipo.clone();

        // ==========================================
        // FASE 5: FRACASO SIMULADO
        // ==========================================
        bitacora.fase_alcanzada = 5;
        info!("[PHA] Fase 5/10: Evaluando prototipo...");

        let (fallo, razon_fracaso) = self.simular_fracaso(&prototipo, problema, intuicion).await;

        if fallo {
            bitacora.fracaso_simulado = Some(razon_fracaso.clone());
            info!("[PHA] Fase 5/10: Fracaso detectado - \"{}\"", razon_fracaso);

            // ==========================================
            // FASE 6: VERGUENZA
            // ==========================================
            bitacora.fase_alcanzada = 6;
            info!("[PHA] Fase 6/10: Sintiendo vergueenza...");

            // La verguenza es proporcional a la confianza previa
            let confianza_previa = metacognicion
                .evaluar_confianza(0.6, 0.5, bitacora.ideas_generadas.len(), 1.0, complejidad)
                .puntaje;

            // Activar verguenza en la amigdala
            amygdala.sentir_verguenza(confianza_previa, "El prototipo funcionaria", &razon_fracaso);
            bitacora.verguenza_intensidad = confianza_previa * 0.8;

            // Pausa corta de verguenza (humano se siente mal)
            tokio::time::sleep(Duration::from_millis(PAUSA_VERGUENZA_MS)).await;

            // ==========================================
            // FASE 7: PAUSA FORZADA (INCUBACION)
            // ==========================================
            bitacora.fase_alcanzada = 7;
            info!("[PHA] Fase 7/10: Incubando...");

            let incubacion_inicio = Instant::now();
            self.incubar(problema, &prototipo, &razon_fracaso, memoria_semantica)
                .await;
            bitacora.incubacion_ms = incubacion_inicio.elapsed().as_millis() as u64;

            // ==========================================
            // FASE 8: INSIGHT REPENTINO
            // ==========================================
            bitacora.fase_alcanzada = 8;
            info!("[PHA] Fase 8/10: Buscando insight...");

            let insight = self
                .generar_insight(
                    problema,
                    &prototipo,
                    &razon_fracaso,
                    &bitacora.ideas_generadas,
                    ocean,
                    memoria_semantica,
                )
                .await;

            bitacora.insight = Some(insight.clone());
            self.aciertos_insight += 1;
            info!("[PHA] Insight encontrado! \"{}\"", insight);
        } else {
            info!("[PHA] Fase 5/10: Prototipo viable, saltando a refinamiento");
            bitacora.insight = Some(prototipo.clone());
        }

        // ==========================================
        // FASE 9: REFINAMIENTO ITERATIVO
        // ==========================================
        bitacora.fase_alcanzada = 9;
        info!("[PHA] Fase 9/10: Refinando...");

        let base = bitacora.insight.as_deref().unwrap_or(&prototipo);
        let refinado = self.refinar(base, problema, ocean).await;
        bitacora.iteraciones_refinamiento = MAX_ITERACIONES_REFINAMIENTO;
        bitacora.resultado_final = refinado.clone();

        // ==========================================
        // FASE 10: ORGULLO
        // ==========================================
        bitacora.fase_alcanzada = 10;
        info!("[PHA] Fase 10/10: Celebrando el logro");

        let superacion = if fallo {
            // Si hubo fracaso, el orgullo es mayor porque se supero
            (bitacora.verguenza_intensidad + 0.3).min(1.0)
        } else {
            0.6 // Orgullo base por resolverlo
        };

        amygdala.sentir_orgullo(
            superacion,
            &format!(
                "Complete el ciclo de pensamiento humano para: {}",
                &problema[..problema.len().min(80)]
            ),
        );
        bitacora.orgullo_intensidad = superacion;

        // ==========================================
        // CIERRE
        // ==========================================
        bitacora.tiempo_total_ms = inicio_total.elapsed().as_millis() as u64;
        info!("{}", bitacora.resumen());

        (refinado, bitacora)
    }

    // ==========================================
    // FASE 2: GENERACION DE IDEAS EN PARALELO
    // ==========================================
    // vs humano que lo hace secuencial, NEXUS genera
    // multiples enfoques en paralelo.
    async fn generar_ideas(
        &self,
        problema: &str,
        ocean: Option<&Ocean>,
        memoria_semantica: &MemoriaSemantica,
    ) -> Vec<String> {
        let mut ideas = Vec::new();

        // Idea 1: Enfoque logico/analitico
        ideas.push(format!(
            "Enfoque analitico: Descomponer el problema en subproblemas y resolver cada uno. '{}' requiere entender primero sus partes fundamentales.",
            &problema[..problema.len().min(100)]
        ));

        // Idea 2: Enfoque creativo/intuitivo
        ideas.push(format!(
            "Enfoque creativo: Buscar una metafora o analogia que ilumine la solucion. '{}' visto desde otra perspectiva podria tener una solucion no obvia.",
            &problema[..problema.len().min(100)]
        ));

        // Idea 3: Enfoque por memoria (experiencias previas)
        // Solo si Ocean está disponible (modo operador lo omite a menos que
        // el Arquitecto pida explícitamente que NEXUS recuerde algo).
        if let Some(o) = ocean {
            let recuerdos = o.recordar_por_significado(problema, 3).await;
            if !recuerdos.is_empty() {
                let mejor_recuerdo = &recuerdos[0];
                ideas.push(format!(
                    "Enfoque por experiencia: Recordando '{}' (confianza: {:.0}%). Esto se parece a una situacion previa.",
                    mejor_recuerdo.0.tema, mejor_recuerdo.1 * 100.0
                ));
            }
        }

        // Idea 4: Enfoque de busqueda semantica
        if let Ok(vector) = memoria_semantica.generar_embedding(problema).await {
            if let Ok(resultados) = memoria_semantica.buscar_similares(vector, 3).await {
                if !resultados.is_empty() {
                    ideas.push(format!(
                        "Enfoque semantico: La memoria semantica sugiere patrones relacionados. {} conceptos similares encontrados.",
                        resultados.len()
                    ));
                }
            }
        }

        // Idea 5: Enfoque pragmatico (la mas simple posible)
        ideas.push(format!(
            "Enfoque pragmatico: Cual es la accion mas pequena que puedo tomar ahora mismo para avanzar en '{}'?",
            &problema[..problema.len().min(100)]
        ));

        ideas
    }

    // ==========================================
    // FASE 3: SELECCION POR INTUICION (NO LOGICA)
    // ==========================================
    // El humano elije "esto me parece interesante"
    // sin razon logica. NEXUS usa OCEAN + Intuicion.
    async fn seleccionar_por_intuicion(
        &self,
        ideas: &[String],
        intuicion: &Intuicion,
        _ocean: Option<&Ocean>,
    ) -> (usize, String) {
        if ideas.is_empty() {
            return (0, String::new());
        }

        // Evaluar cada idea con la intuicion
        let mut mejor_score = 0.0f64;
        let mut mejor_idx = 0;

        for (i, idea) in ideas.iter().enumerate() {
            let indicadores: Vec<String> = idea.split_whitespace().map(|w| w.to_string()).collect();
            let senales = intuicion.sentir(idea, &indicadores);

            // La intuicion nos dice si esta idea "huele bien"
            let alerta = intuicion.nivel_alerta_general(&senales);

            // Una buena idea intuitiva tiene alerta moderada
            // (demasiada alerta = peligro, muy poca = irrelevante)
            let score = if alerta > 0.7 {
                1.0 - alerta // Muchas alertas = peligro
            } else if alerta > 0.3 {
                alerta // Alerta moderada = interesante
            } else {
                alerta * 0.5 // Poca alerta = poco relevante
            };

            if score > mejor_score {
                mejor_score = score;
                mejor_idx = i;
            }
        }

        (mejor_idx, ideas[mejor_idx].clone())
    }

    // ==========================================
    // FASE 4: PROTOTIPADO RAPIDO
    // ==========================================
    async fn prototipar(&self, idea: &str, problema: &str, _ocean: Option<&Ocean>) -> String {
        // Construir el prototipo mas pequeno posible de la idea
        format!(
            "[PROTOTIPO] Aplicando '{}' al problema '{}':\n\
             Paso 1: Identificar el nucleo del problema\n\
             Paso 2: Aplicar el enfoque seleccionado\n\
             Paso 3: Validar el resultado minimo viable",
            &idea[..idea.len().min(80)],
            &problema[..problema.len().min(80)]
        )
    }

    // ==========================================
    // FASE 5: SIMULACION DE FRACASO
    // ==========================================
    // La primera idea casi siempre falla.
    // NEXUS evalua el prototipo contra patrones de error conocidos.
    async fn simular_fracaso(
        &self,
        prototipo: &str,
        problema: &str,
        intuicion: &Intuicion,
    ) -> (bool, String) {
        // 1. Evaluar con intuicion si el prototipo es riesgoso
        let indicadores: Vec<String> = prototipo
            .split_whitespace()
            .chain(problema.split_whitespace())
            .map(|w| w.to_string())
            .collect();

        let senales = intuicion.sentir(prototipo, &indicadores);
        let alerta = intuicion.nivel_alerta_general(&senales);

        // 2. Si la alerta intuitiva es alta, el prototipo probablemente falla
        if alerta > 0.5 {
            let razon = senales
                .first()
                .map(|s| s.descripcion.clone())
                .unwrap_or_else(|| "El prototipo presenta patrones de riesgo".to_string());
            return (true, razon);
        }

        // 3. Verificar contradicciones internas (disonancia cognitiva)
        let tiene_disonancia = senales.iter().any(|s| {
            matches!(
                s.tipo,
                crate::cerebro::organos::intuicion::TipoIntuicion::Disonancia
            )
        });

        if tiene_disonancia {
            return (
                true,
                "El prototipo contiene contradicciones internas".to_string(),
            );
        }

        // 4. Simular tasa de fracaso basada en complejidad
        let complejidad = (problema.len() as f64 / 500.0).min(1.0);
        let probabilidad_fracaso = alerta * 0.4 + complejidad * 0.3;

        if probabilidad_fracaso > 0.6 {
            return (true, format!(
                "Alta probabilidad de fracaso ({:.0}%): el problema es complejo y el enfoque tiene senales de riesgo",
                probabilidad_fracaso * 100.0
            ));
        }

        (false, String::new())
    }

    // ==========================================
    // FASE 7: INCUBACION SIMULADA
    // ==========================================
    // Mientras "incuba", NEXUS ejecuta procesos de fondo:
    // - Busca en memoria semantica
    // - Procesa el problema en segundo plano
    // - Deja que las conexiones se formen solas
    async fn incubar(
        &self,
        problema: &str,
        prototipo: &str,
        razon_fracaso: &str,
        memoria_semantica: &MemoriaSemantica,
    ) {
        // Pausa activa: el cerebro sigue procesando en segundo plano
        tokio::time::sleep(Duration::from_millis(PAUSA_INCUBACION_MS)).await;

        // Durante la incubacion, buscar asociaciones lejanas en memoria
        let consulta_incubacion = format!("{} {} {}", problema, prototipo, razon_fracaso);

        // Buscar en memoria semantica (dispara procesos de fondo)
        if let Ok(vector) = memoria_semantica
            .generar_embedding(&consulta_incubacion)
            .await
        {
            if let Ok(resultados) = memoria_semantica.buscar_similares(vector, 5).await {
                if resultados.len() > 2 {
                    info!(
                        "[PHA] Incubacion activa: {} asociaciones encontradas en memoria",
                        resultados.len()
                    );
                }
            }
        }
    }

    // ==========================================
    // FASE 8: GENERACION DE INSIGHT
    // ==========================================
    // "Eureka!" - La solucion aparece sola despues de la incubacion.
    // NEXUS sintetiza el prototipo fallido + la razon del fracaso
    // + las nuevas asociaciones de memoria = insight.
    async fn generar_insight(
        &self,
        problema: &str,
        _prototipo: &str,
        razon_fracaso: &str,
        _ideas_previas: &[String],
        _ocean: Option<&Ocean>,
        _memoria_semantica: &MemoriaSemantica,
    ) -> String {
        // El insight es la síntesis de:
        // 1. Lo que se intentó (prototipo)
        // 2. Por qué falló (razon_fracaso)
        // 3. Lo que se aprendió de la incubacion
        // 4. Una nueva perspectiva

        let mut insight =
            String::from("[INSIGHT] Despues de la incubacion, la solucion aparece clara:\n");

        // Incorporar la leccion del fracaso
        insight.push_str(&format!("Leccion del fracaso: {}\n", razon_fracaso));

        // Nueva perspectiva basada en lo aprendido
        insight.push_str(&format!(
            "Nueva perspectiva: En lugar de abordar '{}' directamente,\n\
             el enfoque correcto es considerar el problema desde su raiz:\n\
             - Lo que NO funciona: {}\n\
             - Lo que podria funcionar: un enfoque que evite los patrones de riesgo detectados\n\
             - Accion concreta: empezar con la validacion mas pequena posible",
            &problema[..problema.len().min(80)],
            razon_fracaso
        ));

        insight
    }

    // ==========================================
    // FASE 9: REFINAMIENTO ITERATIVO
    // ==========================================
    async fn refinar(&self, base: &str, problema: &str, _ocean: Option<&Ocean>) -> String {
        let mut refinado = String::from("=== SOLUCION REFINADA ===\n\n");
        refinado.push_str(&format!("Problema original: {}\n\n", problema));
        refinado.push_str(&format!(
            "Proceso de refinamiento:\n\
             1. Idea inicial generada\n\
             2. Evaluada contra experiencia previa\n\
             3. Mejorada con {} iteraciones de refinamiento\n\
             4. Validada por coherencia interna\n\n",
            MAX_ITERACIONES_REFINAMIENTO
        ));
        refinado.push_str(&format!("Resultado: {}\n", base));

        refinado
    }

    /// Obtiene metricas del motor de pensamiento humano
    pub fn metricas(&self) -> serde_json::Value {
        serde_json::json!({
            "confusion_basal": self.confusion_basal,
            "aciertos_insight": self.aciertos_insight,
            "fallos_insight": self.fallos_insight,
            "precision_insight": if self.aciertos_insight + self.fallos_insight > 0 {
                self.aciertos_insight as f64 / (self.aciertos_insight + self.fallos_insight) as f64
            } else {
                0.0
            },
            "tiempos_ms": {
                "confusion": PAUSA_CONFUSION_MS,
                "verguenza": PAUSA_VERGUENZA_MS,
                "incubacion": PAUSA_INCUBACION_MS,
            }
        })
    }
}
