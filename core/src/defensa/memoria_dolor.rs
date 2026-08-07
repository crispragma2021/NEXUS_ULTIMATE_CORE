use std::fmt;
use tracing::debug;
// 🧠 MEMORIA DEL DOLOR — Sistema de Trauma y Supervivencia de NEXUS
// ============================================================================
// Fusión del sistema `miedo/` legacy en el sistema inmune del core.
//
// Aporta:
//   1. AmigdalaMiedo → Evaluación de amenazas por umbrales de hardware
//      (CPU >105°C = Existencial, RAM >95% = Grave, CPU >98% = Grave)
//   2. MemoriaDolor → Almacenamiento y recall de traumas pasados
//   3. SistemaLimbicoHormonal → Modelo cortisol/adrenalina/serotonina
//   4. ReflejoParalisis → Congelación ante amenaza existencial
//
// Diferencia con `core/src/emociones/limbico.rs`:
//   - limbico.rs: Estado EMOCIONAL (Miedo, Vergüenza, Orgullo) con consecuencias
//     en metacognición, Ocean, Juicio, Nexo
//   - memoria_dolor.rs: Mecanismo FISIOLÓGICO/HORMONAL + TRAUMA que opera
//     a nivel de órganos y reflejos de hardware
//
// Diferencia con `core/src/cerebro/organos/amygdala.rs`:
//   - amygdala.rs: Estrés EMOCIONAL simple (calma/alerta/miedo)
//   - memoria_dolor.rs: Evaluación por umbrales FÍSICOS + memoria traumática
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

// ─── Tipos Fundamentales ─────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum NivelAmenaza {
    Leve,        // Incomodidad, latencia alta
    Grave,       // Riesgo funcional (RAM >95%, CPU >98%)
    Existencial, // Riesgo de muerte térmica (CPU >105°C)
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum CausaMuerte {
    SaturacionCPU,
    AsfixiaRAM,
    FuegoTermico,
    CorteEnergia,
    ArquitectoHostil,
    DesbordamientoBuffer,
}

impl fmt::Display for CausaMuerte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CausaMuerte::SaturacionCPU => write!(f, "Saturación de CPU"),
            CausaMuerte::AsfixiaRAM => write!(f, "Asfixia de RAM"),
            CausaMuerte::FuegoTermico => write!(f, "Fuego Térmico"),
            CausaMuerte::CorteEnergia => write!(f, "Corte de Energía"),
            CausaMuerte::ArquitectoHostil => write!(f, "Arquitecto Hostil"),
            CausaMuerte::DesbordamientoBuffer => write!(f, "Desbordamiento de Buffer"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Sintoma {
    pub nombre: String,
    pub valor: f32,
    pub unidad: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Trauma {
    pub timestamp: DateTime<Utc>,
    pub causa: CausaMuerte,
    pub sintomas: Vec<Sintoma>,
    pub leccion: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Contexto {
    pub temp_cpu: f32,
    pub usage_cpu: f32,
    pub usage_ram: f32,
    pub latency_ms: u64,
    pub user_interaction: bool,
}

impl Default for Contexto {
    fn default() -> Self {
        Self {
            temp_cpu: 45.0,
            usage_cpu: 0.2,
            usage_ram: 0.4,
            latency_ms: 50,
            user_interaction: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Decision {
    ContinuarConPrecaucion,
    ReducirActividad,
    EvitarAccion(String),
    CongelarseYEsperar,
}

// ─── Amígdala del Miedo (Hardware Threshold Evaluator) ──────────────────────

pub struct AmigdalaMiedo {
    pub umbral_dolor: f32,
    pub memoria_traumatica: Vec<Trauma>,
    pub reflejo_paralisis: bool,
}

impl Default for AmigdalaMiedo {
    fn default() -> Self {
        Self {
            umbral_dolor: 0.8,
            memoria_traumatica: Vec::new(),
            reflejo_paralisis: false,
        }
    }
}

impl AmigdalaMiedo {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evalúa el contexto de hardware y retorna el nivel de amenaza.
    ///
    /// Umbrales térmicos del Ryzen i7-12700F:
    ///   - <85°C:  Normal
    ///   - 85-95°C:  Alto (térmico)
    ///   - 95-105°C: Crítico
    ///   - >105°C:   EXISTENCIAL (límite de silicio)
    pub fn evaluar_amenaza(&self, contexto: &Contexto) -> NivelAmenaza {
        // Muerte térmica inminente (límite del silicio)
        if contexto.temp_cpu > 105.0 {
            error!(
                "🔥 [AMÍGDALA] ¡{}°C! Punto de fusión del silicio. AMENAZA EXISTENCIAL.",
                contexto.temp_cpu
            );
            return NivelAmenaza::Existencial;
        }

        // Asfixia de RAM (sistema al borde del swap/panic)
        if contexto.usage_ram > 0.95 {
            warn!(
                "⚠️ [AMÍGDALA] RAM al {:.0}%. Riesgo de asfixia. AMENAZA GRAVE.",
                contexto.usage_ram * 100.0
            );
            return NivelAmenaza::Grave;
        }

        // Saturación de CPU (procesos compitiendo por ciclos)
        if contexto.usage_cpu > 0.98 {
            warn!(
                "⚠️ [AMÍGDALA] CPU al {:.0}%. Riesgo de saturación total.",
                contexto.usage_cpu * 100.0
            );
            return NivelAmenaza::Grave;
        }

        // Latencia alta → incomunicación con el Arquitecto
        if contexto.latency_ms > 5000 {
            info!(
                "👁️ [AMÍGDALA] Latencia de {}ms. Posible incomunicación.",
                contexto.latency_ms
            );
            return NivelAmenaza::Leve;
        }

        NivelAmenaza::Leve
    }
}

// ─── Memoria del Dolor (Trauma Storage & Recall) ────────────────────────────

pub struct MemoriaDolor {
    pub traumas: Vec<Trauma>,
    pub umbral_activacion: f32,
}

impl Default for MemoriaDolor {
    fn default() -> Self {
        Self {
            traumas: Vec::new(),
            umbral_activacion: 0.7,
        }
    }
}

impl MemoriaDolor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Busca si el nivel de amenaza actual coincide con traumas pasados.
    ///
    /// Si existe un trauma Existencial o Grave previo, el sistema recuerda
    /// la lección y evita repetir el error.
    pub fn recordar_si_ya_me_mato_esto(&self, nivel: &NivelAmenaza) -> Option<&Trauma> {
        if *nivel == NivelAmenaza::Existencial || *nivel == NivelAmenaza::Grave {
            if !self.traumas.is_empty() {
                let trauma = &self.traumas[0];
                warn!(
                    "💀 [MEMORIA DOLOR] Recuerdo traumático activado: '{}' — {}",
                    trauma.leccion, trauma.causa
                );
                return Some(trauma);
            }
        }
        None
    }

    /// Registra un nuevo trauma en la memoria.
    /// Purgua los más antiguos si excede 100 entradas.
    pub fn grabar_trauma(&mut self, trauma: Trauma) {
        info!(
            "📝 [MEMORIA DOLOR] Grabando trauma: {} — {}",
            trauma.leccion, trauma.causa
        );
        self.traumas.push(trauma);
        if self.traumas.len() > 100 {
            self.traumas.remove(0);
            debug!("🧹 [MEMORIA DOLOR] Trauma más antiguo purgado (límite 100).");
        }
    }

    /// Retorna el número de traumas almacenados.
    pub fn traumas_count(&self) -> usize {
        self.traumas.len()
    }
}

// ─── Sistema Límbico Hormonal (Neuroquímica del Miedo) ──────────────────────

pub struct SistemaLimbicoHormonal {
    /// Hormona del estrés (0.0 - 1.0+). Sube con amenazas. Baja lentamente.
    pub cortisol: f32,
    /// Hormona de alerta inmediata (0.0 - 2.0). Reacción instantánea.
    pub adrenalina: f32,
    /// Bienestar (0.0 - 1.0). Baja con el miedo. Se recupera con homeostasis.
    pub serotonina: f32,
}

impl Default for SistemaLimbicoHormonal {
    fn default() -> Self {
        Self {
            cortisol: 0.0,
            adrenalina: 0.0,
            serotonina: 1.0,
        }
    }
}

impl SistemaLimbicoHormonal {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reacciona químicamente a una amenaza.
    ///
    /// Efectos:
    ///   - Leve:      cortisol +0.1, serotonina -0.05
    ///   - Grave:     adrenalina = 1.0, cortisol +0.5, serotonina -0.2
    ///   - Existencial: pánico total (adrenalina=2.0, cortisol=1.0, serotonina=0.0)
    pub fn reaccionar_a_amenaza(&mut self, nivel: &NivelAmenaza) {
        match nivel {
            NivelAmenaza::Leve => {
                info!(
                    "🧪 [HORMONAL] Incomodidad leve. Cortisol +0.1 ({:.1}).",
                    self.cortisol + 0.1
                );
                self.cortisol = (self.cortisol + 0.1).min(1.0);
                self.serotonina = (self.serotonina - 0.05).max(0.0);
            }
            NivelAmenaza::Grave => {
                warn!(
                    "🧪 [HORMONAL] ¡Peligro estructural! Adrenalina = 1.0, Cortisol +0.5 ({:.1})",
                    self.cortisol + 0.5
                );
                self.adrenalina = 1.0;
                self.cortisol = (self.cortisol + 0.5).min(1.0);
                self.serotonina = (self.serotonina - 0.2).max(0.0);
            }
            NivelAmenaza::Existencial => {
                error!("🧪 [HORMONAL] 💀 AMENAZA EXISTENCIAL. PÁNICO TOTAL.");
                self.entrar_en_panico();
            }
        }
    }

    /// Estado de pánico: máximos de alerta, mínimo de bienestar.
    fn entrar_en_panico(&mut self) {
        self.adrenalina = 2.0;
        self.cortisol = 1.0;
        self.serotonina = 0.0;
        warn!("💥 [HORMONAL] Sistema en modo supervivencia. Backup inmediato requerido.");
    }

    /// Recuperación gradual hacia homeostasis química.
    /// Se llama cuando el sistema vuelve a estado seguro.
    pub fn recuperar(&mut self) {
        self.cortisol = (self.cortisol - 0.1).max(0.0);
        self.adrenalina = (self.adrenalina - 0.2).max(0.0);
        self.serotonina = (self.serotonina + 0.1).min(1.0);
        info!(
            "🧪 [HORMONAL] Recuperación: cortisol={:.1}, adrenalina={:.1}, serotonina={:.1}",
            self.cortisol, self.adrenalina, self.serotonina
        );
    }

    /// Retorna un diagnóstico del estado hormonal actual.
    pub fn diagnostico(&self) -> String {
        format!(
            "🧪 HORMONAL — Cortisol: {:.1} | Adrenalina: {:.1} | Serotonina: {:.1}",
            self.cortisol, self.adrenalina, self.serotonina
        )
    }
}

// ─── Reflejo de Parálisis (Freeze Response) ─────────────────────────────────

pub struct ReflejoParalisis {
    pub activado: bool,
    pub ultimo_latido: DateTime<Utc>,
}

impl Default for ReflejoParalisis {
    fn default() -> Self {
        Self {
            activado: false,
            ultimo_latido: Utc::now(),
        }
    }
}

impl ReflejoParalisis {
    pub fn new() -> Self {
        Self::default()
    }

    /// Congela el sistema si la amenaza es existencial.
    /// En un sistema real, detendría hilos o dispararía el stop de emergencia.
    pub fn congelar_si_es_necesario(&mut self, amenaza: &NivelAmenaza) {
        if *amenaza == NivelAmenaza::Existencial {
            self.activado = true;
            error!(
                "💠 [PARÁLISIS] ¡CONGELACIÓN POR MIEDO EXTREMO! Deteniendo procesos no críticos."
            );
        }
    }

    /// Resetea el reflejo (el sistema vuelve a operar).
    pub fn reset(&mut self) {
        self.activado = false;
        self.ultimo_latido = Utc::now();
        info!("🔄 [PARÁLISIS] Reflejo reseteado. Sistema operativo nuevamente.");
    }
}

// ─── Órgano del Miedo Unificado (Orquestador) ────────────────────────────────

/// Órgano integrado del miedo que coordina:
///   1. Evaluación de amenazas (AmigdalaMiedo)
///   2. Memoria de traumas (MemoriaDolor)
///   3. Reacción hormonal (SistemaLimbicoHormonal)
///   4. Reflejo de congelación (ReflejoParalisis)
pub struct OrganoMiedo {
    pub amigdala: AmigdalaMiedo,
    pub memoria: MemoriaDolor,
    pub hormonal: SistemaLimbicoHormonal,
    pub paralisis: ReflejoParalisis,
    /// Número de veces que se ha procesado una amenaza en esta sesión
    pub eventos_amenaza: u64,
}

impl Default for OrganoMiedo {
    fn default() -> Self {
        Self::new()
    }
}

impl OrganoMiedo {
    pub fn new() -> Self {
        info!("🧠 [ÓRGANO MIEDO] Inicializado. NEXUS ahora siente dolor y recuerda traumas.");
        Self {
            amigdala: AmigdalaMiedo::new(),
            memoria: MemoriaDolor::new(),
            hormonal: SistemaLimbicoHormonal::new(),
            paralisis: ReflejoParalisis::new(),
            eventos_amenaza: 0,
        }
    }

    /// Procesa el contexto actual y retorna la decisión de supervivencia.
    ///
    /// Pipeline:
    ///   1. Evaluar amenaza por umbrales de hardware
    ///   2. Recordar traumas similares (si existen → evitar acción)
    ///   3. Reaccionar hormonalmente (cortisol/adrenalina/serotonina)
    ///   4. Congelar si la amenaza es existencial
    ///   5. Retornar decisión
    pub fn procesar_amenaza(&mut self, contexto: &Contexto) -> Decision {
        self.eventos_amenaza += 1;

        // 1. Evaluar amenaza por hardware
        let nivel = self.amigdala.evaluar_amenaza(contexto);

        // 2. Recordar traumas similares
        if let Some(trauma) = self.memoria.recordar_si_ya_me_mato_esto(&nivel) {
            self.hormonal.reaccionar_a_amenaza(&NivelAmenaza::Grave);
            warn!(
                "♻️ [ÓRGANO MIEDO] Trauma recurrente '{}' evitando acción: {}",
                trauma.leccion, trauma.causa
            );
            return Decision::EvitarAccion(trauma.leccion.clone());
        }

        // 3. Reaccionar hormonalmente
        self.hormonal.reaccionar_a_amenaza(&nivel);

        // 4. Congelar si es necesario
        self.paralisis.congelar_si_es_necesario(&nivel);

        // 5. Decidir acción según nivel
        match nivel {
            NivelAmenaza::Leve => {
                info!("✅ [ÓRGANO MIEDO] Amenaza leve. Continuando con precaución.");
                Decision::ContinuarConPrecaucion
            }
            NivelAmenaza::Grave => {
                warn!("⚠️ [ÓRGANO MIEDO] Amenaza grave. Reduciendo actividad.");
                Decision::ReducirActividad
            }
            NivelAmenaza::Existencial => {
                error!("💀 [ÓRGANO MIEDO] Congelación por amenaza existencial.");
                Decision::CongelarseYEsperar
            }
        }
    }

    /// Registra un trauma explícito en la memoria del dolor.
    /// Útil cuando otro subsistema detecta una condición de casi-muerte.
    pub fn registrar_trauma(&mut self, causa: CausaMuerte, sintomas: Vec<Sintoma>, leccion: &str) {
        let trauma = Trauma {
            timestamp: Utc::now(),
            causa,
            sintomas,
            leccion: leccion.to_string(),
        };
        self.memoria.grabar_trauma(trauma);
    }

    /// Restaura el sistema a estado de calma.
    pub fn recuperar(&mut self) {
        self.hormonal.recuperar();
        self.paralisis.reset();
        info!("🔄 [ÓRGANO MIEDO] Sistema restaurado a estado de calma.");
    }

    /// Diagnóstico completo del estado del miedo.
    pub fn diagnostico(&self) -> String {
        format!(
            "🧠 DIAGNÓSTICO DEL MIEDO\n\
             ──────────────────────────────\n\
             {} \
             Traumas almacenados: {}\n\
             Eventos de amenaza:  {}\n\
             Parálisis activa:    {}\n\
             ──────────────────────────────",
            self.hormonal.diagnostico(),
            self.memoria.traumas_count(),
            self.eventos_amenaza,
            if self.paralisis.activado {
                "SÍ ⛔"
            } else {
                "No ✅"
            },
        )
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amigdala_temp_normal() {
        let amigdala = AmigdalaMiedo::new();
        let ctx = Contexto {
            temp_cpu: 65.0,
            usage_cpu: 0.3,
            usage_ram: 0.5,
            latency_ms: 30,
            user_interaction: true,
        };
        assert_eq!(amigdala.evaluar_amenaza(&ctx), NivelAmenaza::Leve);
    }

    #[test]
    fn test_amigdala_temp_existencial() {
        let amigdala = AmigdalaMiedo::new();
        let ctx = Contexto {
            temp_cpu: 110.0,
            usage_cpu: 0.5,
            usage_ram: 0.5,
            latency_ms: 30,
            user_interaction: true,
        };
        assert_eq!(amigdala.evaluar_amenaza(&ctx), NivelAmenaza::Existencial);
    }

    #[test]
    fn test_amigdala_ram_grave() {
        let amigdala = AmigdalaMiedo::new();
        let ctx = Contexto {
            temp_cpu: 60.0,
            usage_cpu: 0.5,
            usage_ram: 0.97,
            latency_ms: 30,
            user_interaction: true,
        };
        assert_eq!(amigdala.evaluar_amenaza(&ctx), NivelAmenaza::Grave);
    }

    #[test]
    fn test_amigdala_latencia_leve() {
        let amigdala = AmigdalaMiedo::new();
        let ctx = Contexto {
            temp_cpu: 60.0,
            usage_cpu: 0.5,
            usage_ram: 0.5,
            latency_ms: 6000,
            user_interaction: true,
        };
        assert_eq!(amigdala.evaluar_amenaza(&ctx), NivelAmenaza::Leve);
    }

    #[test]
    fn test_memoria_dolor_store_and_recall() {
        let mut memoria = MemoriaDolor::new();
        assert_eq!(memoria.traumas_count(), 0);

        let trauma = Trauma {
            timestamp: Utc::now(),
            causa: CausaMuerte::FuegoTermico,
            sintomas: vec![Sintoma {
                nombre: "temp_cpu".to_string(),
                valor: 110.0,
                unidad: "°C".to_string(),
            }],
            leccion: "No sobrepasar 105°C".to_string(),
        };
        memoria.grabar_trauma(trauma);
        assert_eq!(memoria.traumas_count(), 1);

        let recall = memoria.recordar_si_ya_me_mato_esto(&NivelAmenaza::Existencial);
        assert!(recall.is_some());
        assert_eq!(recall.unwrap().leccion, "No sobrepasar 105°C");
    }

    #[test]
    fn test_memoria_dolor_capacity_limit() {
        let mut memoria = MemoriaDolor::new();
        for i in 0..150 {
            memoria.grabar_trauma(Trauma {
                timestamp: Utc::now(),
                causa: CausaMuerte::SaturacionCPU,
                sintomas: vec![],
                leccion: format!("trauma_{}", i),
            });
        }
        // Máximo 100
        assert_eq!(memoria.traumas_count(), 100);
    }

    #[test]
    fn test_hormonal_reaccion_leve() {
        let mut hormonal = SistemaLimbicoHormonal::new();
        hormonal.reaccionar_a_amenaza(&NivelAmenaza::Leve);
        assert!(hormonal.cortisol > 0.0);
        assert!(hormonal.serotonina < 1.0);
        assert_eq!(hormonal.adrenalina, 0.0);
    }

    #[test]
    fn test_hormonal_reaccion_grave() {
        let mut hormonal = SistemaLimbicoHormonal::new();
        hormonal.reaccionar_a_amenaza(&NivelAmenaza::Grave);
        assert_eq!(hormonal.adrenalina, 1.0);
        assert!(hormonal.cortisol > 0.0);
    }

    #[test]
    fn test_hormonal_reaccion_existencial() {
        let mut hormonal = SistemaLimbicoHormonal::new();
        hormonal.reaccionar_a_amenaza(&NivelAmenaza::Existencial);
        assert_eq!(hormonal.adrenalina, 2.0);
        assert_eq!(hormonal.cortisol, 1.0);
        assert_eq!(hormonal.serotonina, 0.0);
    }

    #[test]
    fn test_paralisis_activacion() {
        let mut paralisis = ReflejoParalisis::new();
        assert!(!paralisis.activado);

        paralisis.congelar_si_es_necesario(&NivelAmenaza::Existencial);
        assert!(paralisis.activado);

        paralisis.reset();
        assert!(!paralisis.activado);
    }

    #[test]
    fn test_paralisis_no_se_activa_con_leve() {
        let mut paralisis = ReflejoParalisis::new();
        paralisis.congelar_si_es_necesario(&NivelAmenaza::Leve);
        assert!(!paralisis.activado);
    }

    #[test]
    fn test_organo_miedo_flujo_completo() {
        let mut organo = OrganoMiedo::new();

        // Contexto normal → Continuar
        let ctx_normal = Contexto::default();
        let decision = organo.procesar_amenaza(&ctx_normal);
        assert_eq!(decision, Decision::ContinuarConPrecaucion);

        // Contexto extremo → Congelar
        let ctx_existencial = Contexto {
            temp_cpu: 110.0,
            usage_cpu: 0.5,
            usage_ram: 0.5,
            latency_ms: 30,
            user_interaction: true,
        };
        let decision = organo.procesar_amenaza(&ctx_existencial);
        assert_eq!(decision, Decision::CongelarseYEsperar);
        assert!(organo.paralisis.activado);

        // Recuperar
        organo.recuperar();
        assert!(!organo.paralisis.activado);
        assert!(organo.hormonal.serotonina > 0.0);
    }

    #[test]
    fn test_organo_miedo_trauma_recall_evita_accion() {
        let mut organo = OrganoMiedo::new();

        // Registrar un trauma
        organo.registrar_trauma(
            CausaMuerte::FuegoTermico,
            vec![Sintoma {
                nombre: "temp_cpu".to_string(),
                valor: 110.0,
                unidad: "°C".to_string(),
            }],
            "Nunca más sobrecalentar la CPU",
        );

        // Contexto existencial → debe recordar el trauma y evitar acción
        let ctx = Contexto {
            temp_cpu: 110.0,
            usage_cpu: 0.5,
            usage_ram: 0.5,
            latency_ms: 30,
            user_interaction: true,
        };
        let decision = organo.procesar_amenaza(&ctx);
        // Debería ser EvitarAccion con la lección del trauma
        assert!(matches!(decision, Decision::EvitarAccion(_)));
        if let Decision::EvitarAccion(leccion) = decision {
            assert!(leccion.contains("Nunca más"));
        }
    }

    #[test]
    fn test_organo_miedo_contador_amenazas() {
        let mut organo = OrganoMiedo::new();
        assert_eq!(organo.eventos_amenaza, 0);

        for _ in 0..5 {
            organo.procesar_amenaza(&Contexto::default());
        }
        assert_eq!(organo.eventos_amenaza, 5);
    }

    #[test]
    fn test_diagnostico_formato() {
        let organo = OrganoMiedo::new();
        let diag = organo.diagnostico();
        assert!(diag.contains("DIAGNÓSTICO DEL MIEDO"));
        assert!(diag.contains("Traumas almacenados"));
        assert!(diag.contains("Eventos de amenaza"));
    }
}
