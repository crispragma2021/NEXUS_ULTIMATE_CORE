// ============================================================================
// 🦾 CÓRTEX MOTOR — Ejecución de Acciones por Voluntad Neuronal
// ============================================================================
// Traduce los patrones de disparo de las asambleas motoras en acciones
// reales en el sistema anfitrión.
// ============================================================================

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TipoAccion {
    Shell(String),
    EscrituraArchivo(String, String),
    LecturaArchivo(String),
    NavegacionWeb(String),
    Trading(String), // Placeholder para lógica de trading
}

pub struct AsambleaMotora {
    /// IDs de las neuronas que disparan la acción
    pub neuronas: Vec<u32>,
    /// La acción vinculada a este patrón de disparo
    pub accion: TipoAccion,
    /// Umbral de sincronía para ejecutar (0.0 - 1.0)
    pub umbral_disparo: f32,
}

pub struct CortexMotor {
    /// Mapa de patrones de disparo registrados para acciones
    pub mapa_motor: Vec<AsambleaMotora>,
    /// Historial de acciones ejecutadas por voluntad propia
    pub historial_acciones: Vec<(u64, TipoAccion)>,
}

impl CortexMotor {
    pub fn nuevo() -> Self {
        Self {
            mapa_motor: Vec::new(),
            historial_acciones: Vec::new(),
        }
    }

    /// Vincula un patrón de disparo a una acción específica
    pub fn aprender_accion(&mut self, neuronas: Vec<u32>, accion: TipoAccion) {
        self.mapa_motor.push(AsambleaMotora {
            neuronas,
            accion,
            umbral_disparo: 0.9, // Requiere alta coherencia
        });
    }

    /// Analiza la actividad neuronal actual y decide si ejecutar una acción
    pub fn procesar_voluntad_accion(
        &mut self,
        paso: u64,
        neuronas_activas: &[u32],
    ) -> Vec<TipoAccion> {
        let mut acciones_a_ejecutar = Vec::new();

        for asamblea in &self.mapa_motor {
            let mut coincidencia = 0;
            for &n in neuronas_activas {
                if asamblea.neuronas.contains(&n) {
                    coincidencia += 1;
                }
            }

            let sincronia = coincidencia as f32 / asamblea.neuronas.len() as f32;
            if sincronia >= asamblea.umbral_disparo {
                acciones_a_ejecutar.push(asamblea.accion.clone());
                self.historial_acciones.push((paso, asamblea.accion.clone()));
            }
        }

        acciones_a_ejecutar
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cortex_motor_nuevo_vacio() {
        let c = CortexMotor::nuevo();
        assert!(c.mapa_motor.is_empty());
        assert!(c.historial_acciones.is_empty());
    }

    #[test]
    fn test_aprender_accion_vincula_patron_con_umbral() {
        let mut c = CortexMotor::nuevo();
        c.aprender_accion(vec![1, 2, 3], TipoAccion::Shell("echo hola".to_string()));
        assert_eq!(c.mapa_motor.len(), 1);
        let asamblea = &c.mapa_motor[0];
        assert_eq!(asamblea.neuronas, vec![1, 2, 3]);
        casi(asamblea.umbral_disparo, 0.9);
        assert!(matches!(asamblea.accion, TipoAccion::Shell(_)));
    }

    #[test]
    fn test_ejecuta_accion_con_sincronia_total() {
        let mut c = CortexMotor::nuevo();
        c.aprender_accion(vec![1, 2, 3], TipoAccion::Shell("correr".to_string()));
        let acciones = c.procesar_voluntad_accion(7, &[1, 2, 3]);
        assert_eq!(acciones.len(), 1);
        assert!(matches!(&acciones[0], TipoAccion::Shell(s) if s == "correr"));
    }

    #[test]
    fn test_no_ejecuta_bajo_umbral() {
        let mut c = CortexMotor::nuevo();
        c.aprender_accion(vec![1, 2, 3], TipoAccion::Shell("correr".to_string()));
        // Solo 1 de 3 neuronas => sincronia 0.33 < 0.9
        let acciones = c.procesar_voluntad_accion(1, &[1]);
        assert!(acciones.is_empty());
    }

    #[test]
    fn test_sincronia_justo_en_umbral_ejecuta() {
        let mut c = CortexMotor::nuevo();
        // Patrón de 1 neurona: activarla al 100% da sincronia 1.0
        c.aprender_accion(vec![9], TipoAccion::EscrituraArchivo("a.txt".to_string(), "x".to_string()));
        let acciones = c.procesar_voluntad_accion(2, &[9]);
        assert_eq!(acciones.len(), 1);
    }

    #[test]
    fn test_sincronia_parcial_alta_ejecuta() {
        let mut c = CortexMotor::nuevo();
        // Patrón de 10 neuronas, 9 activas => sincronia 0.9 exacta => ejecuta
        let patron: Vec<u32> = (0..10).collect();
        c.aprender_accion(patron.clone(), TipoAccion::LecturaArchivo("f.txt".to_string()));
        let activas: Vec<u32> = (0..9).collect();
        let acciones = c.procesar_voluntad_accion(3, &activas);
        assert_eq!(acciones.len(), 1);
    }

    #[test]
    fn test_historial_registra_acciones_ejecutadas() {
        let mut c = CortexMotor::nuevo();
        c.aprender_accion(vec![1, 2], TipoAccion::NavegacionWeb("https://x".to_string()));
        c.procesar_voluntad_accion(42, &[1, 2]);
        assert_eq!(c.historial_acciones.len(), 1);
        let (paso, accion) = &c.historial_acciones[0];
        assert_eq!(*paso, 42);
        assert!(matches!(accion, TipoAccion::NavegacionWeb(_)));
    }

    #[test]
    fn test_no_ejecuta_sin_patron_aprendido() {
        let mut c = CortexMotor::nuevo();
        let acciones = c.procesar_voluntad_accion(1, &[1, 2, 3]);
        assert!(acciones.is_empty());
    }

    #[test]
    fn test_multiples_asambleas_solo_disparan_concordantes() {
        let mut c = CortexMotor::nuevo();
        c.aprender_accion(vec![1, 2, 3], TipoAccion::Shell("uno".to_string()));
        c.aprender_accion(vec![10, 20, 30], TipoAccion::Shell("dos".to_string()));
        // Solo el primer patrón tiene sincronia total
        let acciones = c.procesar_voluntad_accion(5, &[1, 2, 3]);
        assert_eq!(acciones.len(), 1);
        assert!(matches!(&acciones[0], TipoAccion::Shell(s) if s == "uno"));
        // Y el historial solo registró la primera
        assert_eq!(c.historial_acciones.len(), 1);
    }

    #[test]
    fn test_clone_tipo_accion_conserva_datos() {
        let mut c = CortexMotor::nuevo();
        c.aprender_accion(vec![7], TipoAccion::Trading("BUY_BTC".to_string()));
        let acciones = c.procesar_voluntad_accion(9, &[7]);
        assert!(matches!(&acciones[0], TipoAccion::Trading(t) if t == "BUY_BTC"));
        // El historial almacena un clone independiente
        assert!(matches!(&c.historial_acciones[0].1, TipoAccion::Trading(_)));
    }

    fn casi(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-4,
            "esperado {:.4}, obtenido {:.4}",
            b,
            a
        );
    }
}
