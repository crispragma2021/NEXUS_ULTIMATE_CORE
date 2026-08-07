// ============================================================================
// 🌙 SISTEMA DE SUEÑO Y CONSOLIDACIÓN (NREM-REM)
// ============================================================================
// Implementa el ciclo sueño-vigilia con:
// - NREM1: Transición (husos de sueño)
// - NREM2: Consolidación hipocampo → corteza (replay a 20x)
// - NREM3: Poda sináptica profunda (microglía digital)
// - REM: Asociación creativa entre conceptos dispares
// - Despertar: Renormalización homeostática global
//
// Inspirado en: Complementary Learning Systems (McClelland et al., 1995)
//               Synaptic Scaling (Turrigiano, 2008)
//               Sleep-Dependent Memory Consolidation (Rasch & Born, 2013)
// ============================================================================

use super::estructuras::*;
use super::talamo::{TalamoDigital, EstadoConsciencia};
use crate::cerebro::memoria::MemoriaAdaptativa;
use serde::{Deserialize, Serialize};

/// Umbral de episodios diarios para gatillar sueño
const UMBRAL_SUENO: usize = 50;

/// Factor de compresión de replay hipocampal (20x = 1s de vigilia → 50ms de sueño)
const FACTOR_REPLAY: f32 = 0.05;

/// Número de ciclos completos sueño-vigilia antes de aplicar poda
const CICLOS_PODA: u64 = 5;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum EstadoSueno {
    Vigilia,
    NREM1,     // Transición, husos de sueño (~5%)
    NREM2,     // Consolidación principal (~45%)
    NREM3,     // Poda profunda (~25%)
    REM,       // Asociación creativa (~25%)
    Despertar, // Renormalización final
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DuracionFases {
    pub nrem1: u64,
    pub nrem2: u64,
    pub nrem3: u64,
    pub rem: u64,
}

impl Default for DuracionFases {
    fn default() -> Self {
        Self {
            nrem1: 200,
            nrem2: 1800,
            nrem3: 1000,
            rem: 1000,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EstadisticasSueno {
    pub episodios_consolidados: usize,
    pub sinapsis_podadas: usize,
    pub sinapsis_fortalecidas: usize,
    pub creatividad_generada: usize,
    pub duracion_total: u64,
    pub ciclos_completados: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatronSueno {
    pub columna_id: u32,
    pub neuronas_disparadas: Vec<u32>,
    pub intensidad_promedio: f32,
}

/// Sistema de sueño y consolidación
///
/// Orquesta el ciclo completo: acumula episodios en vigilia,
/// consolida en NREM, poda en NREM3, asocia en REM, renormaliza al despertar.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SistemaSueno {
    pub estado: EstadoSueno,
    pub duracion_fase: DuracionFases,
    pub ciclos_en_fase: u64,
    pub tasa_poda: f32,
    pub tasa_consolidacion: f32,
    pub umbral_supervivencia: f32,
    pub episodios_diarios: Vec<PatronSueno>,
    pub ultima_consolidacion: Option<EstadisticasSueno>,
    pub ciclos_sueno_completados: u64,
    pub contador_poda: u64,
}

impl SistemaSueno {
    pub fn nuevo() -> Self {
        Self {
            estado: EstadoSueno::Vigilia,
            duracion_fase: DuracionFases::default(),
            ciclos_en_fase: 0,
            tasa_poda: 0.05,
            tasa_consolidacion: 0.15,
            umbral_supervivencia: 0.02,
            episodios_diarios: Vec::new(),
            ultima_consolidacion: None,
            ciclos_sueno_completados: 0,
            contador_poda: 0,
        }
    }

    /// Registrar un patrón de actividad cortical durante vigilia
    pub fn registrar_patron(&mut self, patrones: Vec<PatronSueno>) {
        if self.estado == EstadoSueno::Vigilia {
            self.episodios_diarios.extend(patrones);
            // Mantener solo los más recientes (ventana de 500)
            if self.episodios_diarios.len() > 500 {
                self.episodios_diarios.drain(0..self.episodios_diarios.len() - 500);
            }
        }
    }

    /// Verificar si debe iniciar el sueño
    pub fn debe_dormir(&self) -> bool {
        self.estado == EstadoSueno::Vigilia && self.episodios_diarios.len() >= UMBRAL_SUENO
    }

    /// Ciclo principal de sueño: transiciona fases y ejecuta consolidación
    /// Retorna true si el cerebro debe estar en modo sueño (no procesar estímulos)
    pub fn ciclo_sueno(
        &mut self,
        _dt: f32,
        talamo: &mut TalamoDigital,
        memoria: &mut MemoriaAdaptativa,
        columnas: &mut Vec<ColumnaCortical>,
        params: &ParametrosNeurona,
    ) -> bool {
        self.ciclos_en_fase += 1;

        match self.estado {
            EstadoSueno::Vigilia => {
                return false; // No estamos durmiendo
            }
            EstadoSueno::NREM1 => {
                // Transición: el tálamo cambia a ritmo lento
                talamo.estado = EstadoConsciencia::SuenioLigero;
                // La actividad cortical disminuye gradualmente
                for col in columnas.iter_mut() {
                    col.activacion_sostenida *= 0.9; // Decaimiento natural
                }
                if self.ciclos_en_fase >= self.duracion_fase.nrem1 {
                    self.transicionar_a(EstadoSueno::NREM2, talamo);
                }
            }

            EstadoSueno::NREM2 => {
                // Consolidación: reproducción comprimida de episodios diarios
                self.consolidar_episodios(columnas, params);

                if self.ciclos_en_fase >= self.duracion_fase.nrem2 {
                    self.transicionar_a(EstadoSueno::NREM3, talamo);
                }
            }

            EstadoSueno::NREM3 => {
                // Sueño profundo: poda sináptica masiva
                self.consolidar_episodios(columnas, params);
                self.podar_sinapsis(memoria);

                if self.ciclos_en_fase >= self.duracion_fase.nrem3 {
                    self.transicionar_a(EstadoSueno::REM, talamo);
                }
            }

            EstadoSueno::REM => {
                // Asociación creativa: mezclar fragmentos de episodios
                self.asociacion_creativa(columnas, params);

                if self.ciclos_en_fase >= self.duracion_fase.rem {
                    self.transicionar_a(EstadoSueno::Despertar, talamo);
                }
            }

            EstadoSueno::Despertar => {
                // Renormalización homeostática global
                self.renormalizar_sinapsis(memoria);
                self.estado = EstadoSueno::Vigilia;
                talamo.estado = EstadoConsciencia::Vigilia;
                talamo.oscilador.ajustar_ritmo(&EstadoConsciencia::Vigilia);
                self.ciclos_sueno_completados += 1;
                self.ciclos_en_fase = 0;

                // Registrar estadísticas de la noche
                let total_episodios = self.episodios_diarios.len();
                self.ultima_consolidacion = Some(EstadisticasSueno {
                    episodios_consolidados: total_episodios,
                    sinapsis_podadas: 0,
                    sinapsis_fortalecidas: 0,
                    creatividad_generada: 0,
                    duracion_total: self.ciclos_en_fase,
                    ciclos_completados: self.ciclos_sueno_completados,
                });

                // Limpiar buffer de episodios diarios
                self.episodios_diarios.clear();

                println!("  🌅 Cerebro despierto. Noche {} completada. {} episodios consolidados.",
                    self.ciclos_sueno_completados, total_episodios);
            }
        }

        true // Estamos durmiendo
    }

    /// Transicionar a una nueva fase del sueño
    fn transicionar_a(&mut self, nuevo_estado: EstadoSueno, talamo: &mut TalamoDigital) {
        self.estado = nuevo_estado.clone();
        self.ciclos_en_fase = 0;

        match &nuevo_estado {
            EstadoSueno::NREM2 | EstadoSueno::NREM3 => {
                talamo.estado = EstadoConsciencia::SuenioProfundo;
                talamo.oscilador.ajustar_ritmo(&EstadoConsciencia::SuenioProfundo);
            }
            EstadoSueno::REM => {
                talamo.estado = EstadoConsciencia::SuenioREM;
                talamo.oscilador.ajustar_ritmo(&EstadoConsciencia::SuenioREM);
            }
            _ => {}
        }
    }

    // ========================================================================
    // CONSOLIDACIÓN NREM (Hipocampo → Corteza)
    // ========================================================================
    fn consolidar_episodios(
        &mut self,
        columnas: &mut Vec<ColumnaCortical>,
        _params: &ParametrosNeurona,
    ) {
        if self.episodios_diarios.is_empty() {
            return;
        }

        // Agrupar patrones por columna
        let mut consolidacion: Vec<(u32, f32, u32)> = Vec::new(); // (columna_id, intensidad_acumulada, count)

        for patron in &self.episodios_diarios {
            let encontrado = consolidacion.iter_mut().find(|(id, _, _)| *id == patron.columna_id);
            if let Some((_, intensidad, count)) = encontrado {
                *intensidad += patron.intensidad_promedio;
                *count += 1;
            } else {
                consolidacion.push((patron.columna_id, patron.intensidad_promedio, 1));
            }
        }

        // Aplicar consolidación a cada columna: fortalecer sinapsis de patrones repetidos
        for (col_id, intensidad_total, count) in &consolidacion {
            if let Some(col) = columnas.iter_mut().find(|c| c.id == *col_id) {
                let intensidad_media = intensidad_total / *count as f32;
                let factor = intensidad_media * self.tasa_consolidacion * FACTOR_REPLAY;

                // Fortalecer conexiones internas de la columna proporcionalmente
                for capa_idx in 1..=6u8 {
                    if let Some(capa) = col.capa_mut(capa_idx) {
                        for conexiones in capa.conexiones_intra.values_mut() {
                            for sin in conexiones.iter_mut() {
                                // Fortalecer sinapsis con rendimientos decrecientes
                                let plasticidad = factor * (1.0 - sin.peso.min(0.9));
                                sin.peso = (sin.peso + plasticidad).min(1.0);
                            }
                        }
                    }
                }

                // También fortalecer conexiones inter-capa (feedforward)
                for capa_idx in 1..=6u8 {
                    if let Some(capa) = col.capa_mut(capa_idx) {
                        for conexiones in capa.conexiones_inter.values_mut() {
                            for sin in conexiones.iter_mut() {
                                let plasticidad = factor * 0.5 * (1.0 - sin.peso.min(0.9));
                                sin.peso = (sin.peso + plasticidad).min(1.0);
                            }
                        }
                    }
                }
            }
        }
    }

    // ========================================================================
    // PODA SINÁPTICA NREM3 (Microglía Digital)
    // ========================================================================
    fn podar_sinapsis(
        &mut self,
        memoria: &mut MemoriaAdaptativa,
    ) {
        self.contador_poda += 1;

        // Solo podar cada CICLOS_PODA ciclos de sueño para evitar inestabilidad
        if self.contador_poda % CICLOS_PODA != 0 {
            return;
        }

        let umbral = self.umbral_supervivencia * 0.8; // Más agresivo en NREM3
        let mut total_podadas = 0;

        // Podar sinapsis y neuronas débiles de la RAM
        // En la arquitectura actual, las sinapsis se almacenan en RamManager
        //
        // Fase 1: Identificar y eliminar neuronas con energía por debajo del umbral
        let ids_a_remover: Vec<u32> = {
            let neuronas = memoria.ram.obtener_todas();
            neuronas.iter()
                .filter(|n| n.energia < umbral * 0.1)
                .map(|n| n.id)
                .collect()
        };

        for id in &ids_a_remover {
            memoria.ram.eliminar_neurona(*id);
        }
        total_podadas += ids_a_remover.len();

        if total_podadas > 0 {
            println!(
                "  🧹 Microglía digital: {} sinapsis/neuronas podadas",
                total_podadas
            );
        }

        // Actualizar estadísticas
        if let Some(ref mut stats) = self.ultima_consolidacion {
            stats.sinapsis_podadas += total_podadas;
        }
    }

    // ========================================================================
    // ASOCIACIÓN CREATIVA REM
    // ========================================================================
    fn asociacion_creativa(
        &mut self,
        columnas: &mut Vec<ColumnaCortical>,
        _params: &ParametrosNeurona,
    ) {
        if self.episodios_diarios.len() < 3 || columnas.len() < 2 {
            return;
        }

        // Seleccionar dos patrones de columnas diferentes para crear asociación
        let mut creatividad_generada = 0;

        for i in 0..columnas.len().saturating_sub(1) {
            let origen = i;
            let destino = i + 1;

            // Buscar patrones de estas columnas en los episodios diarios
            let patrones_origen: Vec<&PatronSueno> = self.episodios_diarios.iter()
                .filter(|p| p.columna_id == columnas[origen].id as u32)
                .collect();

            let patrones_destino: Vec<&PatronSueno> = self.episodios_diarios.iter()
                .filter(|p| p.columna_id == columnas[destino].id as u32)
                .collect();

            if patrones_origen.is_empty() || patrones_destino.is_empty() {
                continue;
            }

            // Crear conexión horizontal entre las dos columnas (asociación REM)
            let peso_inicial = 0.01; // Peso bajo: debe fortalecerse en vigilia si es útil
            let dest_id = columnas[destino].id as u32;
            let ya_conectadas = columnas[origen].conexiones_horizontales.get(&dest_id)
                .map_or(false, |v| v.iter().any(|c| c.columna_destino == dest_id));

            if !ya_conectadas {
                columnas[origen].conexiones_horizontales
                    .entry(dest_id)
                    .or_insert_with(Vec::new)
                    .push(
                        ConexionHorizontal {
                            columna_destino: dest_id,
                            peso: peso_inicial,
                            tipo: TipoConexionHorizontal::Excitatoria,
                            retardo: 0.005, // 5ms de retardo sináptico
                        }
                    );
                creatividad_generada += 1;
            }
        }

        if creatividad_generada > 0 {
            println!("  💡 REM: {} nuevas asociaciones creativas formadas", creatividad_generada);
        }

        if let Some(ref mut stats) = self.ultima_consolidacion {
            stats.creatividad_generada += creatividad_generada;
        }
    }

    // ========================================================================
    // RENORMALIZACIÓN SINÁPTICA (Despertar)
    // ========================================================================
    fn renormalizar_sinapsis(
        &mut self,
        memoria: &mut MemoriaAdaptativa,
    ) {
        // Synaptic Scaling: escalar todas las sinapsis proporcionalmente
        // para mantener la tasa de disparo objetivo (Turrigiano, 2008)
        //
        // Después de consolidar, la red puede estar sobreexcitada.
        // Se aplica un escalado homeostático global.
        let factor_escala = 0.97; // Leve debilitamiento global post-consolidación

        let neuronas = memoria.ram.obtener_todas_mut();
        for neurona in neuronas.iter_mut() {
            // Reducir pesos de sinapsis entrantes
            neurona.energia *= factor_escala;
        }

        // Aplicar también a las columnas corticales
        // (no tenemos acceso directo a las columnas aquí, se hará desde cerebro.rs)
    }
}

// ============================================================================
// SISTEMA DE SUEÑO — Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::hardware::{ConfiguracionDinamica, Precision};

    fn casi(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-3, "esperado {}, obtenido {}", b, a);
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

    fn memoria_con_gpu() -> MemoriaAdaptativa {
        MemoriaAdaptativa::nuevo(&config_con_gpu())
    }

    fn columna_con_sinapsis_interna(id: u32, siguiente: &mut u32) -> ColumnaCortical {
        let mut col = ColumnaCortical::nueva(id, 100, siguiente);
        // Insertar una sinapsis intra-capa en Capa II (idx 1)
        let origen = *siguiente;
        let destino = *siguiente + 1;
        *siguiente += 2;
        col.capa_mut(1).unwrap().conexiones_intra
            .insert(origen, vec![SinapsisCompacta::nueva(destino, 0.5)]);
        col
    }

    fn patron(columna_id: u32, intensidad: f32) -> PatronSueno {
        PatronSueno {
            columna_id,
            neuronas_disparadas: vec![],
            intensidad_promedio: intensidad,
        }
    }

    // ── EstadoSueno / configuración ──────────────────────────────────────────
    #[test]
    fn test_sistema_nuevo_estado_vigilia() {
        let s = SistemaSueno::nuevo();
        assert_eq!(s.estado, EstadoSueno::Vigilia);
        casi(s.tasa_poda, 0.05);
        casi(s.tasa_consolidacion, 0.15);
        casi(s.umbral_supervivencia, 0.02);
        assert!(s.episodios_diarios.is_empty());
        assert_eq!(s.ciclos_sueno_completados, 0);
        assert!(s.ultima_consolidacion.is_none());
    }

    #[test]
    fn test_duracion_fases_default() {
        let d = DuracionFases::default();
        assert_eq!(d.nrem1, 200);
        assert_eq!(d.nrem2, 1800);
        assert_eq!(d.nrem3, 1000);
        assert_eq!(d.rem, 1000);
    }

    #[test]
    fn test_registrar_patron_solo_en_vigilia() {
        let mut s = SistemaSueno::nuevo();
        s.registrar_patron(vec![patron(0, 0.5)]);
        assert_eq!(s.episodios_diarios.len(), 1);

        s.estado = EstadoSueno::NREM2;
        s.registrar_patron(vec![patron(0, 0.5)]);
        assert_eq!(s.episodios_diarios.len(), 1); // no agrega en sueño
    }

    #[test]
    fn test_registrar_patron_mantiene_ventana_500() {
        let mut s = SistemaSueno::nuevo();
        for i in 0..501 {
            s.registrar_patron(vec![patron(i as u32, 0.5)]);
        }
        assert_eq!(s.episodios_diarios.len(), 500);
    }

    #[test]
    fn test_debe_dormir_bajo_umbral() {
        let s = SistemaSueno::nuevo();
        assert!(!s.debe_dormir());
    }

    #[test]
    fn test_debe_dormir_en_umbral() {
        let mut s = SistemaSueno::nuevo();
        for i in 0..UMBRAL_SUENO {
            s.registrar_patron(vec![patron(i as u32, 0.5)]);
        }
        assert!(s.debe_dormir());
    }

    #[test]
    fn test_debe_dormir_falso_si_ya_duerme() {
        let mut s = SistemaSueno::nuevo();
        for i in 0..UMBRAL_SUENO {
            s.registrar_patron(vec![patron(i as u32, 0.5)]);
        }
        s.estado = EstadoSueno::NREM1;
        assert!(!s.debe_dormir());
    }

    // ── Ciclo de sueño ───────────────────────────────────────────────────────
    #[test]
    fn test_ciclo_vigilia_no_duerme() {
        let mut s = SistemaSueno::nuevo();
        let mut talamo = TalamoDigital::nuevo();
        let mut memoria = memoria_con_gpu();
        let mut columnas: Vec<ColumnaCortical> = Vec::new();
        let params = ParametrosNeurona::default();
        assert!(!s.ciclo_sueno(1.0, &mut talamo, &mut memoria, &mut columnas, &params));
    }

    #[test]
    fn test_nrem1_decae_activacion_sostenida() {
        let mut s = SistemaSueno::nuevo();
        s.estado = EstadoSueno::NREM1;
        s.duracion_fase.nrem1 = 100;
        let mut talamo = TalamoDigital::nuevo();
        let mut memoria = memoria_con_gpu();
        let mut siguiente = 0u32;
        let mut columnas = vec![ColumnaCortical::nueva(0, 50, &mut siguiente)];
        columnas[0].activacion_sostenida = 1.0;
        let params = ParametrosNeurona::default();
        s.ciclo_sueno(1.0, &mut talamo, &mut memoria, &mut columnas, &params);
        casi(columnas[0].activacion_sostenida, 0.9); // *0.9
        assert_eq!(talamo.estado, EstadoConsciencia::SuenioLigero);
    }

    #[test]
    fn test_nrem1_transiciona_a_nrem2() {
        let mut s = SistemaSueno::nuevo();
        s.estado = EstadoSueno::NREM1;
        s.duracion_fase.nrem1 = 1;
        let mut talamo = TalamoDigital::nuevo();
        let mut memoria = memoria_con_gpu();
        let mut columnas: Vec<ColumnaCortical> = Vec::new();
        let params = ParametrosNeurona::default();
        assert!(s.ciclo_sueno(1.0, &mut talamo, &mut memoria, &mut columnas, &params));
        assert_eq!(s.estado, EstadoSueno::NREM2);
        assert_eq!(s.ciclos_en_fase, 0);
        assert_eq!(talamo.estado, EstadoConsciencia::SuenioProfundo);
    }

    #[test]
    fn test_nrem2_consolida_episodios_fortalece_sinapsis() {
        let mut s = SistemaSueno::nuevo();
        s.estado = EstadoSueno::NREM2;
        s.duracion_fase.nrem2 = 100;
        let mut siguiente = 0u32;
        let mut columnas = vec![columna_con_sinapsis_interna(0, &mut siguiente)];
        // registrar patrón de la columna 0 con intensidad máxima
        s.episodios_diarios.push(patron(0, 1.0));
        let mut talamo = TalamoDigital::nuevo();
        let mut memoria = memoria_con_gpu();
        let params = ParametrosNeurona::default();
        s.ciclo_sueno(1.0, &mut talamo, &mut memoria, &mut columnas, &params);
        // la sinapsis interna se fortalece
        let capa = columnas[0].capa(1).unwrap();
        let peso = capa.conexiones_intra.values().next().unwrap()[0].peso;
        assert!(peso > 0.5, "peso no fortaleció: {}", peso);
    }

    #[test]
    fn test_nrem2_sin_episodios_no_consolida() {
        let mut s = SistemaSueno::nuevo();
        s.estado = EstadoSueno::NREM2;
        s.duracion_fase.nrem2 = 100;
        let mut talamo = TalamoDigital::nuevo();
        let mut memoria = memoria_con_gpu();
        let mut siguiente = 0u32;
        let mut columnas = vec![columna_con_sinapsis_interna(0, &mut siguiente)];
        let params = ParametrosNeurona::default();
        s.ciclo_sueno(1.0, &mut talamo, &mut memoria, &mut columnas, &params);
        let capa = columnas[0].capa(1).unwrap();
        let peso = capa.conexiones_intra.values().next().unwrap()[0].peso;
        casi(peso, 0.5); // sin patrones no cambia
    }

    #[test]
    fn test_nrem2_transiciona_a_nrem3() {
        let mut s = SistemaSueno::nuevo();
        s.estado = EstadoSueno::NREM2;
        s.duracion_fase.nrem2 = 1;
        let mut talamo = TalamoDigital::nuevo();
        let mut memoria = memoria_con_gpu();
        let mut columnas: Vec<ColumnaCortical> = Vec::new();
        let params = ParametrosNeurona::default();
        s.ciclo_sueno(1.0, &mut talamo, &mut memoria, &mut columnas, &params);
        assert_eq!(s.estado, EstadoSueno::NREM3);
    }

    #[test]
    fn test_nrem3_poda_neuronas_debiles() {
        let mut s = SistemaSueno::nuevo();
        s.estado = EstadoSueno::NREM3;
        s.duracion_fase.nrem3 = 100;
        s.contador_poda = 4; // próxima llamada: 5 % 5 == 0 → poda
        let mut memoria = memoria_con_gpu();
        // neurona débil (energía casi nula) → se poda
        let mut debil = NeuronaCompacta::reposo(1, 0, 0);
        debil.energia = 0.0001; // < umbral*0.1 (0.02*0.8*0.1=0.0016)
        memoria.ram.agregar_neurona(debil);
        // neurona sana sobrevive
        memoria.ram.agregar_neurona(NeuronaCompacta::reposo(2, 0, 0));
        let mut talamo = TalamoDigital::nuevo();
        let mut columnas: Vec<ColumnaCortical> = Vec::new();
        let params = ParametrosNeurona::default();
        s.ciclo_sueno(1.0, &mut talamo, &mut memoria, &mut columnas, &params);
        assert!(memoria.ram.obtener_neurona(1).is_none());
        assert!(memoria.ram.obtener_neurona(2).is_some());
    }

    #[test]
    fn test_nrem3_no_poda_fuera_de_cadencia() {
        let mut s = SistemaSueno::nuevo();
        s.estado = EstadoSueno::NREM3;
        s.duracion_fase.nrem3 = 100;
        s.contador_poda = 1; // próxima: 2 % 5 != 0 → no poda
        let mut memoria = memoria_con_gpu();
        let mut debil = NeuronaCompacta::reposo(1, 0, 0);
        debil.energia = 0.0001;
        memoria.ram.agregar_neurona(debil);
        let mut talamo = TalamoDigital::nuevo();
        let mut columnas: Vec<ColumnaCortical> = Vec::new();
        let params = ParametrosNeurona::default();
        s.ciclo_sueno(1.0, &mut talamo, &mut memoria, &mut columnas, &params);
        assert!(memoria.ram.obtener_neurona(1).is_some()); // sobrevive
    }

    #[test]
    fn test_rem_asociacion_creativa_conecta_columnas() {
        let mut s = SistemaSueno::nuevo();
        s.estado = EstadoSueno::REM;
        s.duracion_fase.rem = 100;
        let mut siguiente = 0u32;
        let mut columnas = vec![
            ColumnaCortical::nueva(0, 50, &mut siguiente),
            ColumnaCortical::nueva(1, 50, &mut siguiente),
        ];
        // 3 patrones: 2 de col 0, 1 de col 1
        s.episodios_diarios.push(patron(0, 0.5));
        s.episodios_diarios.push(patron(0, 0.5));
        s.episodios_diarios.push(patron(1, 0.5));
        let mut talamo = TalamoDigital::nuevo();
        let mut memoria = memoria_con_gpu();
        let params = ParametrosNeurona::default();
        s.ciclo_sueno(1.0, &mut talamo, &mut memoria, &mut columnas, &params);
        // col 0 tiene conexión horizontal hacia col 1
        let conexiones = columnas[0].conexiones_horizontales.get(&1);
        assert!(conexiones.is_some(), "no se creó conexión horizontal");
        let c = &conexiones.unwrap()[0];
        assert_eq!(c.columna_destino, 1);
        casi(c.peso, 0.01);
    }

    #[test]
    fn test_rem_sin_patrones_no_asocia() {
        let mut s = SistemaSueno::nuevo();
        s.estado = EstadoSueno::REM;
        s.duracion_fase.rem = 100;
        let mut siguiente = 0u32;
        let mut columnas = vec![
            ColumnaCortical::nueva(0, 50, &mut siguiente),
            ColumnaCortical::nueva(1, 50, &mut siguiente),
        ];
        let mut talamo = TalamoDigital::nuevo();
        let mut memoria = memoria_con_gpu();
        let params = ParametrosNeurona::default();
        s.ciclo_sueno(1.0, &mut talamo, &mut memoria, &mut columnas, &params);
        assert!(columnas[0].conexiones_horizontales.is_empty());
    }

    #[test]
    fn test_rem_transiciona_a_despertar() {
        let mut s = SistemaSueno::nuevo();
        s.estado = EstadoSueno::REM;
        s.duracion_fase.rem = 1;
        let mut talamo = TalamoDigital::nuevo();
        let mut memoria = memoria_con_gpu();
        let mut columnas: Vec<ColumnaCortical> = Vec::new();
        let params = ParametrosNeurona::default();
        s.ciclo_sueno(1.0, &mut talamo, &mut memoria, &mut columnas, &params);
        assert_eq!(s.estado, EstadoSueno::Despertar);
    }

    #[test]
    fn test_despertar_renormaliza_y_completa_ciclo() {
        let mut s = SistemaSueno::nuevo();
        s.estado = EstadoSueno::Despertar;
        s.episodios_diarios.push(patron(0, 0.5));
        let mut memoria = memoria_con_gpu();
        let mut n = NeuronaCompacta::reposo(1, 0, 0);
        n.energia = 1.0;
        memoria.ram.agregar_neurona(n);
        let mut talamo = TalamoDigital::nuevo();
        let mut columnas: Vec<ColumnaCortical> = Vec::new();
        let params = ParametrosNeurona::default();
        s.ciclo_sueno(1.0, &mut talamo, &mut memoria, &mut columnas, &params);

        assert_eq!(s.estado, EstadoSueno::Vigilia);
        assert_eq!(s.ciclos_sueno_completados, 1);
        assert_eq!(talamo.estado, EstadoConsciencia::Vigilia);
        // renormalización: energía 1.0 → 0.97
        casi(memoria.ram.obtener_neurona(1).unwrap().energia, 0.97);
        // estadísticas de la noche registradas y episodios limpiados
        assert!(s.ultima_consolidacion.is_some());
        assert!(s.episodios_diarios.is_empty());
        assert_eq!(s.ultima_consolidacion.as_ref().unwrap().episodios_consolidados, 1);
    }

    #[test]
    fn test_estadisticas_suenio_actualizadas_en_despertar() {
        let mut s = SistemaSueno::nuevo();
        s.estado = EstadoSueno::Despertar;
        s.ciclos_sueno_completados = 3;
        let mut talamo = TalamoDigital::nuevo();
        let mut memoria = memoria_con_gpu();
        let mut columnas: Vec<ColumnaCortical> = Vec::new();
        let params = ParametrosNeurona::default();
        s.ciclo_sueno(1.0, &mut talamo, &mut memoria, &mut columnas, &params);
        let stats = s.ultima_consolidacion.unwrap();
        assert_eq!(stats.ciclos_completados, 4); // +1
        assert_eq!(s.ciclos_sueno_completados, 4);
    }
}
