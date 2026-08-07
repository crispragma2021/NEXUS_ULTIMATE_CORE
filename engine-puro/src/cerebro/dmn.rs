// ============================================================================
// 🧠 DEFAULT MODE NETWORK (DMN) — Rumiación y Pensamiento Interno
// ============================================================================
// Basado en el sistema biológico que se activa cuando el cerebro no está
// enfocado en tareas externas. Permite al motor:
// 1. Reverberar conceptos (rumiar) sin entrada sensorial.
// 2. Fortalecer asociaciones débiles mediante STDP espontáneo.
// 3. Simular escenarios futuros basados en memoria episódica.
// 4. Mantener un "flujo de consciencia" interno constante.
// ============================================================================

use serde::{Deserialize, Serialize};
use crate::cerebro::asamblea_cortical::AsambleaCortical;
use crate::cerebro::memoria_vinculo::MemoriaVinculo;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DMNConfig {
    /// Intensidad de la rumiación espontánea (0.0 - 1.0)
    pub intensidad_rumiacion: f32,
    /// Umbral de silencio externo para activar DMN (pasos sin entrada)
    pub pasos_activacion: u64,
    /// Factor de decaimiento de la atención interna
    pub decaimiento_interno: f32,
}

impl Default for DMNConfig {
    fn default() -> Self {
        Self {
            intensidad_rumiacion: 0.4,
            pasos_activacion: 5, // Se activa muy rápido si no hay ruido externo
            decaimiento_interno: 0.05,
        }
    }
}

pub struct DefaultModeNetwork {
    pub config: DMNConfig,
    /// Contador de pasos en silencio
    pub pasos_sin_estimulo: u64,
    /// ¿Está la DMN activa ahora?
    pub activa: bool,
    /// Concepto actualmente en foco interno
    pub foco_interno: Option<u32>,
    /// Historial de rumiación reciente (asambleas activadas)
    pub rastro_rumiacion: Vec<u32>,
}

impl DefaultModeNetwork {
    pub fn nueva(config: DMNConfig) -> Self {
        Self {
            config,
            pasos_sin_estimulo: 0,
            activa: false,
            foco_interno: None,
            rastro_rumiacion: Vec::with_capacity(10),
        }
    }

    /// Actualiza el estado de la DMN basado en la presencia de entrada externa
    pub fn tick(&mut self, hay_entrada: bool) {
        if hay_entrada {
            self.pasos_sin_estimulo = 0;
            self.activa = false;
        } else {
            self.pasos_sin_estimulo += 1;
            if self.pasos_sin_estimulo >= self.config.pasos_activacion {
                self.activa = true;
            }
        }
    }

    /// Ejecuta un ciclo de rumiación interna
    /// 1. Selecciona una asamblea de la Working Memory o Memoria de Vínculo
    /// 2. Inyecta corriente para mantener la reverberación
    /// 3. Permite que el sistema 'piense' sin generar salida externa necesariamente
    pub fn rumiar(
        &mut self,
        asambleas: &mut AsambleaCortical,
        vinculo: &MemoriaVinculo,
    ) {
        if !self.activa {
            return;
        }

        // FASE 1: Selección de semilla de pensamiento
        // Si no tenemos un foco, buscamos en la Working Memory de asambleas
        if self.foco_interno.is_none() {
            if let Some(&id) = asambleas.trabajando.first() {
                self.foco_interno = Some(id);
            } else {
                // Si no hay nada en WM, recuperamos algo del vínculo con el Arquitecto
                if let Some(recuerdo) = vinculo.episodios.back() {
                    if let Some(_palabra) = recuerdo.palabras_clave.first() {
                        // Intentamos activar una asamblea arbitraria como semilla
                        // En el futuro esto usará un mapeo palabra -> asamblea
                        self.foco_interno = Some(recuerdo.timestamp as u32 % 32); 
                    }
                }
            }
        }

        // FASE 2: Reverberación
        if let Some(id) = self.foco_interno {
            // Inyectar una corriente suave pero persistente en la asamblea foco
            // Esto mantiene el "hilo de pensamiento" vivo
            asambleas.inyectar_corriente_a_asamblea(id, self.config.intensidad_rumiacion);
            
            if !self.rastro_rumiacion.contains(&id) {
                self.rastro_rumiacion.push(id);
                if self.rastro_rumiacion.len() > 10 {
                    self.rastro_rumiacion.remove(0);
                }
            }

            // Probabilidad de saltar a una asamblea relacionada (asociación libre)
            // En un sistema real esto usaría la conectividad estructural
            // Aquí simulamos el "vagabundeo mental" con un decaimiento y salto
            if rand::random::<f32>() < 0.1 {
                self.foco_interno = None; // Reset para buscar nueva semilla en el próximo tick
            }
        }
    }

    /// Devuelve un resumen del estado interno actual
    pub fn estado_consciente(&self) -> String {
        if !self.activa {
            return "ENFOCADO EN TAREA EXTERNA".to_string();
        }
        format!("RUMIANDO (Foco: {:?}, Rastro: {:?})", self.foco_interno, self.rastro_rumiacion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::asamblea_cortical::AsambleaCortical;
    use crate::cerebro::memoria_vinculo::MemoriaVinculo;

    fn casi(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-4,
            "esperado {:.4}, obtenido {:.4}",
            b,
            a
        );
    }

    #[test]
    fn test_config_default() {
        let cfg = DMNConfig::default();
        casi(cfg.intensidad_rumiacion, 0.4);
        assert_eq!(cfg.pasos_activacion, 5);
        casi(cfg.decaimiento_interno, 0.05);
    }

    #[test]
    fn test_nueva_estado_inicial() {
        let dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        assert_eq!(dmn.pasos_sin_estimulo, 0);
        assert!(!dmn.activa);
        assert!(dmn.foco_interno.is_none());
        assert!(dmn.rastro_rumiacion.is_empty());
    }

    #[test]
    fn test_tick_con_entrada_resetea_contador() {
        let mut dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        // Acumula silencio
        dmn.tick(false);
        dmn.tick(false);
        assert_eq!(dmn.pasos_sin_estimulo, 2);
        // Llega entrada => resetea
        dmn.tick(true);
        assert_eq!(dmn.pasos_sin_estimulo, 0);
        assert!(!dmn.activa);
    }

    #[test]
    fn test_tick_sin_entrada_activa_tras_umbral() {
        let mut dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        // Umbral = 5
        for _ in 0..5 {
            assert!(!dmn.activa, "no debe activarse antes del umbral");
            dmn.tick(false);
        }
        assert!(dmn.activa, "debe activarse al alcanzar el umbral");
        assert_eq!(dmn.pasos_sin_estimulo, 5);
    }

    #[test]
    fn test_tick_no_activa_antes_de_umbral() {
        let mut dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        dmn.tick(false);
        dmn.tick(false);
        dmn.tick(false);
        dmn.tick(false);
        assert!(!dmn.activa);
        assert_eq!(dmn.pasos_sin_estimulo, 4);
    }

    #[test]
    fn test_rumiar_inactivo_no_genera_rastro() {
        let mut dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        let mut asambleas = AsambleaCortical::nueva();
        asambleas.trabajando.push(1);
        let vinculo = MemoriaVinculo::nueva();
        dmn.rumiar(&mut asambleas, &vinculo);
        assert!(dmn.rastro_rumiacion.is_empty());
        assert!(dmn.foco_interno.is_none());
    }

    #[test]
    fn test_rumiar_toma_foco_de_working_memory() {
        let mut dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        // Activar la DMN directamente
        dmn.activa = true;
        let mut asambleas = AsambleaCortical::nueva();
        asambleas.trabajando.push(7);
        let vinculo = MemoriaVinculo::nueva();
        dmn.rumiar(&mut asambleas, &vinculo);
        // El foco se tomó de la WM (7) y reverberó: rastro no vacío
        assert!(!dmn.rastro_rumiacion.is_empty());
    }

    #[test]
    fn test_rumiar_reverbera_semilla_del_vinculo_sin_wm() {
        let mut dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        dmn.activa = true;
        let mut asambleas = AsambleaCortical::nueva();
        // WM vacía => se busca semilla en el vínculo
        let mut vinculo = MemoriaVinculo::nueva();
        vinculo.registrar_interaccion(
            100,
            &["hola".to_string()],
            &[],
            0.5,
            0.5,
            &["respuesta".to_string()],
        );
        dmn.rumiar(&mut asambleas, &vinculo);
        // El vínculo provee semilla: timestamp 100 % 32 = 4
        assert_eq!(dmn.foco_interno, Some(100 % 32));
    }

    #[test]
    fn test_estado_consciente_enfocado() {
        let dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        assert_eq!(dmn.estado_consciente(), "ENFOCADO EN TAREA EXTERNA");
    }

    #[test]
    fn test_estado_consciente_rumiando() {
        let mut dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        dmn.activa = true;
        dmn.foco_interno = Some(3);
        dmn.rastro_rumiacion.push(3);
        let estado = dmn.estado_consciente();
        assert!(estado.contains("RUMIANDO"));
        assert!(estado.contains("Foco"));
        assert!(estado.contains("Rastro"));
    }

    #[test]
    fn test_rastro_rumiacion_limita_a_10() {
        let mut dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        dmn.activa = true;
        // Forzar 15 IDs distintos en el rastro (los empuja rumiar con WM cambiante)
        let mut asambleas = AsambleaCortical::nueva();
        let vinculo = MemoriaVinculo::nueva();
        for i in 0..15 {
            asambleas.trabajando.clear();
            asambleas.trabajando.push(i);
            dmn.rumiar(&mut asambleas, &vinculo);
        }
        assert!(dmn.rastro_rumiacion.len() <= 10);
    }
}
