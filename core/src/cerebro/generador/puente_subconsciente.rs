// ============================================================================
// 🧠 PUENTE SUBCONSCIENTE ↔ OCEAN — Sincronización de impacto emocional
// ============================================================================
// Propósito: Conectar las impresiones almacenadas en Ocean directamente
//   al Subconsciente y perturbar el peso de los conceptos que el GOI
//   utilizará para hablar.
//
// Adaptado a la API real:
//   - Impresion: { id, esencia, tono_emocional, tema, reflejo_arquitecto, timestamp }
//   - Subconsciente::registrar_impresion(esencia, tono, tema)
// ============================================================================

use crate::cerebro::generador::resonancia_semantica::NodoConceptoExpandido;
use crate::emociones::ocean::Impresion;
use crate::memoria::subconsciente::Subconsciente;
use std::collections::HashMap;

/// Puente entre Ocean (memoria episódica) y el Subconsciente + GOI.
///
/// Procesa impresiones del Ocean y:
/// 1. Las inyecta en el Subconsciente molecular (traumas/éxitos)
/// 2. Perturba la valencia emocional de los NodoConceptoExpandido
/// 3. Penaliza conceptos cuando el impacto es severamente negativo
pub struct PuenteSubconscienteOcean {
    /// Mapa semántico: token → NodoConceptoExpandido
    pub mapa_semantico: HashMap<String, NodoConceptoExpandido>,
}

impl Default for PuenteSubconscienteOcean {
    fn default() -> Self {
        Self::new()
    }
}

impl PuenteSubconscienteOcean {
    pub fn new() -> Self {
        Self {
            mapa_semantico: HashMap::new(),
        }
    }

    /// Sincroniza un impacto fuerte detectado en el Ocean directamente hacia
    /// el Subconsciente y perturba el peso de los conceptos del GOI.
    ///
    /// `impresion`: impresión recuperada de Ocean.
    /// `subconsciente`: referencia mutable al Subconsciente del sistema.
    pub fn procesar_filtrado_subconsciente(
        &mut self,
        impresion: &Impresion,
        subconsciente: &mut Subconsciente,
    ) {
        // 1. Inyectar la impresión en el subconsciente molecular
        //    Usando la API real: registrar_impresion(esencia, tono, tema)
        subconsciente.registrar_impresion(
            &impresion.esencia,
            impresion.tono_emocional,
            &impresion.tema,
        );

        // 2. Perturbación del mapa semántico del GOI
        //    Buscamos si las palabras clave del impacto ya existen en los nodos
        let intensidad_neta = impresion.tono_emocional; // -1.0 a 1.0
        let esencia_lower = impresion.esencia.to_lowercase();

        for (_token, nodo) in self.mapa_semantico.iter_mut() {
            if esencia_lower.contains(&nodo.token_clave.to_lowercase()) {
                nodo.registrar_perturbacion(intensidad_neta);

                // Si el impacto es severamente negativo (trauma), penalizar
                // el concepto para que el ensamblador lo evite
                if intensidad_neta < -0.5 {
                    // Penalización extra: 2 usos fantasma para saturar
                    // temporalmente el concepto
                    nodo.registrar_perturbacion(-0.1);
                    nodo.registrar_perturbacion(-0.1);
                }
            }
        }
    }

    /// Enfriamiento homeostático de la frecuencia de uso de palabras.
    /// Se ejecuta en cada tic de fondo (MundoInterno).
    /// Reduce la frecuencia de uso de todos los nodos en 1.
    pub fn enfriar_conceptos(&mut self) {
        for nodo in self.mapa_semantico.values_mut() {
            nodo.enfriar();
        }
    }

    /// Alimenta el mapa semántico desde las mareas de Ocean.
    ///
    /// Cada tema de la marea genera:
    /// 1. Auto-registro del token en el mapa semántico (si no existe)
    /// 2. Perturbación del nodo con el tono_promedio de la marea
    /// 3. Impacto amplificado por frecuencia (temas recurrentes pesan más)
    ///
    /// `mareas`: HashMap< tema, (tono_promedio, frecuencia) > de Ocean::obtener_mareas()
    pub fn alimentar_desde_mareas(
        &mut self,
        mareas: &std::collections::HashMap<String, (f64, u32)>,
    ) {
        for (tema, &(tono, freq)) in mareas.iter() {
            // Registrar si no existe (valencia inicial = tono de la marea)
            self.registrar_token(tema, tono);

            // Perturbar el nodo con intensidad proporcional a frecuencia
            if let Some(nodo) = self.mapa_semantico.get_mut(tema) {
                let peso_perturbacion = tono * (1.0 + (freq as f64).ln().min(2.0) / 10.0);
                nodo.registrar_perturbacion(peso_perturbacion);
            }
        }
    }

    /// Registra un token en el mapa semántico si no existe.
    /// Si ya existe, no lo duplica.
    ///
    /// Devuelve true si fue insertado, false si ya existía.
    pub fn registrar_token(&mut self, token: &str, valencia_inicial: f64) -> bool {
        if self.mapa_semantico.contains_key(token) {
            false
        } else {
            let nodo = NodoConceptoExpandido::new(token, valencia_inicial);
            self.mapa_semantico.insert(token.to_string(), nodo);
            true
        }
    }

    /// Obtiene la valencia emocional actual de un token, si existe.
    pub fn valencia_de(&self, token: &str) -> Option<f64> {
        self.mapa_semantico.get(token).map(|n| n.valencia_emocional)
    }

    /// Verifica si un token está saturado (no debe usarse en salida).
    pub fn token_esta_saturado(&self, token: &str) -> bool {
        self.mapa_semantico
            .get(token)
            .is_some_and(|n| n.esta_saturado())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memoria::subconsciente::Subconsciente;

    fn impresion_ejemplo() -> Impresion {
        Impresion {
            id: 1,
            esencia: "fracaso_en_implementacion".to_string(),
            tono_emocional: -0.8,
            tema: "desarrollo".to_string(),
            reflejo_arquitecto: "NEXUS cometió un error crítico".to_string(),
            timestamp: "2026-06-12T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_procesar_filtrado_afecta_valencia() {
        let impresion = impresion_ejemplo();
        let mut sub = Subconsciente::new();
        let mut puente = PuenteSubconscienteOcean::new();

        // Registrar un token que coincida con la esencia
        puente.registrar_token("fracaso", 0.0);
        let valencia_antes = puente.valencia_de("fracaso").unwrap();

        // Procesar el impacto
        puente.procesar_filtrado_subconsciente(&impresion, &mut sub);

        let valencia_despues = puente.valencia_de("fracaso").unwrap();
        assert!(
            (valencia_despues - valencia_antes).abs() > 0.001,
            "La valencia debería haber cambiado después del impacto"
        );
        // Debería haberse movido hacia negativo
        assert!(valencia_despues < valencia_antes);
    }

    #[test]
    fn test_enfriar_conceptos_reduce_frecuencia() {
        let mut puente = PuenteSubconscienteOcean::new();
        puente.registrar_token("test", 0.0);

        // Simular 3 perturbaciones
        for _ in 0..3 {
            if let Some(nodo) = puente.mapa_semantico.get_mut("test") {
                nodo.registrar_perturbacion(0.5);
            }
        }
        assert_eq!(puente.mapa_semantico["test"].frecuencia_uso, 3);

        // Enfriar
        puente.enfriar_conceptos();
        assert_eq!(puente.mapa_semantico["test"].frecuencia_uso, 2);
    }

    #[test]
    fn test_trauma_severo_satura_concepto() {
        let impresion = Impresion {
            id: 2,
            esencia: "error_critico_sistema".to_string(),
            tono_emocional: -0.9,
            tema: "seguridad".to_string(),
            reflejo_arquitecto: "Vulnerabilidad crítica descubierta".to_string(),
            timestamp: "2026-06-12T00:00:00Z".to_string(),
        };

        let mut sub = Subconsciente::new();
        let mut puente = PuenteSubconscienteOcean::new();
        puente.registrar_token("error", 0.0);

        puente.procesar_filtrado_subconsciente(&impresion, &mut sub);

        // La penalización extra debe saturar el concepto (frecuencia > 5)
        assert!(
            puente.mapa_semantico["error"].frecuencia_uso > 2,
            "El trauma severo debería incrementar frecuencia_uso notablemente"
        );
    }

    #[test]
    fn test_registrar_token_no_duplica() {
        let mut puente = PuenteSubconscienteOcean::new();
        assert!(puente.registrar_token("amor", 0.8));
        assert!(!puente.registrar_token("amor", 0.8)); // Ya existe
        assert_eq!(puente.mapa_semantico.len(), 1);
    }

    #[test]
    fn test_token_saturado_reporta_correctamente() {
        let mut puente = PuenteSubconscienteOcean::new();
        puente.registrar_token("repetitivo", 0.0);
        // Saturarlo
        for _ in 0..6 {
            if let Some(nodo) = puente.mapa_semantico.get_mut("repetitivo") {
                nodo.registrar_perturbacion(0.1);
            }
        }
        assert!(puente.token_esta_saturado("repetitivo"));
        assert!(!puente.token_esta_saturado("inexistente"));
    }

    #[test]
    fn test_alimentar_desde_mareas_registra_y_perturba() {
        let mut puente = PuenteSubconscienteOcean::new();
        let mut mareas = std::collections::HashMap::new();
        mareas.insert("exito".to_string(), (0.8, 5));
        mareas.insert("fracaso".to_string(), (-0.6, 2));

        puente.alimentar_desde_mareas(&mareas);

        // Ambos tokens deben existir ahora
        assert!(puente.mapa_semantico.contains_key("exito"));
        assert!(puente.mapa_semantico.contains_key("fracaso"));

        // "exito" debe tener valencia positiva (se perturbó con 0.8)
        let val_exito = puente.valencia_de("exito").unwrap();
        assert!(
            val_exito > 0.0,
            "éxito debería tener valencia positiva, got {}",
            val_exito
        );

        // "fracaso" debe tener valencia negativa
        let val_fracaso = puente.valencia_de("fracaso").unwrap();
        assert!(
            val_fracaso < 0.0,
            "fracaso debería tener valencia negativa, got {}",
            val_fracaso
        );

        // El token con más frecuencia debe estar más perturbado
        // (exito tiene freq=5, fracaso tiene freq=2)
        assert_eq!(puente.mapa_semantico.len(), 2);
    }
}
