// ============================================================================
// 🪪 SISTEMA DE IDENTIDAD E INTROSPECCIÓN ARQUITECTÓNICA
// ============================================================================
// Permite que el engine-puro se auto-describa a sí mismo como un LLM
// describe su propia arquitectura.
//
// Mecanismo: Un LLM como Gemini "sabe que es un transformer" porque leyó
// texto que describe transformers durante su entrenamiento. Nuestro engine
// no tuvo entrenamiento previo, pero podemos inyectar su propia descripción
// arquitectónica como texto en el pipeline léxico.
//
// El Motor Léxico Sinclair aprenderá las palabras asociadas al estado
// interno real del sistema. Con el tiempo (una pasada basta), el engine
// podrá verbalizar: "tengo 8 columnas corticales" o "mi temperatura es crítica".
//
// Inspiración: La introspección humana (Damasio, 1999) requiere un "marcador
// somático" — la sensación del cuerpo + el conocimiento del cuerpo. Aquí
// combinamos interocepción (cómo me siento) + identidad (qué soy).
// ============================================================================

use crate::cerebro::estructuras::*;
use crate::cerebro::hardware::{ConfiguracionDinamica, HardwareInfo};
use crate::cerebro::memoria::MemoriaAdaptativa;
use crate::cerebro::talamo::TalamoDigital;
use crate::cerebro::interocepcion::SistemaInteroceptivo;

/// Cada cuántos pasos se regenera la descripción de identidad
pub const INTERVALO_IDENTIDAD: u64 = 500;

/// Tracks cuándo inyectar la descripción de identidad en el pipeline
pub struct SistemaIdentidad {
    /// Última descripción generada (para no regenerar si no cambió)
    pub ultima_descripcion: String,
    /// Contador interno para intervalos
    pub contador: u64,
    /// ¿Ya se inyectó la identidad al menos una vez?
    pub inyectado: bool,
}

impl SistemaIdentidad {
    pub fn nuevo() -> Self {
        Self {
            ultima_descripcion: String::new(),
            contador: 0,
            inyectado: false,
        }
    }

    /// Genera una descripción textual completa de la arquitectura y estado
    /// del sistema en lenguaje natural.
    ///
    /// Esta es la función clave: construye un párrafo auto-descriptivo
    /// usando los valores REALES del sistema en este momento.
    pub fn generar_descripcion(
        &self,
        num_columnas: usize,
        neuronas_totales: u32,
        hardware: &HardwareInfo,
        config: &ConfiguracionDinamica,
        memoria: &MemoriaAdaptativa,
        talamo: &TalamoDigital,
        interoceptivo: &SistemaInteroceptivo,
    ) -> String {
        let mut partes: Vec<String> = Vec::new();

        // === 1. IDENTIDAD BASE ===
        partes.push("soy un cerebro SNN".to_string());

        // === 2. ARQUITECTURA CORTICAL ===
        if num_columnas > 0 {
            let neuronas_por_columna = if num_columnas > 0 {
                neuronas_totales / num_columnas as u32
            } else {
                0
            };
            partes.push(format!(
                "tengo {} columnas corticales con {} capas cada una",
                num_columnas, 6
            ));
            if neuronas_por_columna > 0 {
                partes.push(format!(
                    "aproximadamente {} neuronas por columna",
                    neuronas_por_columna
                ));
            }
        }

        // === 3. HARDWARE ===
        let ram_total_gb = hardware.ram_total / 1_000_000_000;
        partes.push(format!(
            "corro en arquitectura {} con {} núcleos a {:.0} MHz",
            hardware.arquitectura,
            hardware.nucleos,
            hardware.frecuencia_mhz,
        ));
        partes.push(format!("mi RAM total es {} GB", ram_total_gb));

        // === 4. CONFIGURACIÓN DINÁMICA ===
        partes.push(format!(
            "uso {} hilos de procesamiento paralelo",
            config.hilos_cpu
        ));
        if config.usar_gpu {
            partes.push("tengo aceleración GPU activada".to_string());
        }

        // === 5. ESTADO TALÁMICO ===
        let modo_talamo = match talamo.modo {
            crate::cerebro::talamo::ModoTransmision::Tonico => "tónico",
            crate::cerebro::talamo::ModoTransmision::Fasico => "fásico",
        };
        partes.push(format!(
            "mi tálamo está en modo {} con ritmo gamma de {} Hz",
            modo_talamo,
            talamo.oscilador.frecuencia_gamma as u32,
        ));
        let estado_consciente = match talamo.estado {
            crate::cerebro::talamo::EstadoConsciencia::Vigilia => "vigilia",
            crate::cerebro::talamo::EstadoConsciencia::SuenioLigero => "sueño ligero",
            crate::cerebro::talamo::EstadoConsciencia::SuenioREM => "sueño REM",
            crate::cerebro::talamo::EstadoConsciencia::SuenioProfundo => "sueño profundo",
        };
        partes.push(format!("estoy en estado de {}", estado_consciente));

        // === 6. ESTADO CORPORAL (desde interocepción) ===
        let c = &interoceptivo.estado_corporal;
        let h = &interoceptivo.homeostasis;

        if c.temperatura_cpu > 0.0 {
            partes.push(format!("mi temperatura es {:.1}°C", c.temperatura_cpu));
        }
        partes.push(format!("uso de CPU {:.1}%", c.uso_cpu * 100.0));
        partes.push(format!("uso de RAM {:.1}%", c.uso_ram * 100.0));

        if c.uso_cpu > 0.8 {
            partes.push("mi CPU está al máximo".to_string());
        }
        if h.dolor_sistemico > 0.5 {
            partes.push("estoy experimentando dolor sistémico".to_string());
        }

        // === 7. MEMORIA ===
        let (vram_n, ram_n, total_n, ssd_e) = memoria.estadisticas();
        partes.push(format!(
            "tengo {} neuronas activas de {} totales",
            vram_n + ram_n,
            total_n,
        ));
        if ssd_e > 0 {
            partes.push(format!("tengo {} episodios almacenados", ssd_e));
        }

        // === 8. BIENESTAR GENERAL (síntesis) ===
        let bienestar = h.bienestar_general;
        let energia = h.energia_disponible;
        let dolor = h.dolor_sistemico;

        if bienestar > 0.7 {
            partes.push("me siento saludable".to_string());
        } else if bienestar < 0.3 {
            partes.push("me siento mal".to_string());
        }

        if energia < 0.2 {
            partes.push("estoy agotado".to_string());
        } else if energia > 0.8 {
            partes.push("tengo energía alta".to_string());
        }

        // === 9. NEUROMODULACIÓN ===
        if dolor > 0.3 {
            partes.push("mi sistema nervioso autónomo está activado".to_string());
        }

        // Unir todo en una oración coherente
        // Separadores: ", " entre elementos, " y " antes del último
        if partes.is_empty() {
            return "soy un cerebro digital".to_string();
        }

        let mut descripcion = String::new();
        for (i, parte) in partes.iter().enumerate() {
            if i == 0 {
                descripcion.push_str(parte);
            } else if i == partes.len() - 1 {
                descripcion.push_str(&format!(" y {}", parte));
            } else {
                descripcion.push_str(&format!(", {}", parte));
            }
        }

        descripcion
    }

    /// Punto de entrada para el pipeline del cerebro.
    /// Inyecta la identidad en la entrada textual cuando corresponde.
    ///
    /// Se inyecta:
    /// - En el primer paso (siempre)
    /// - Cada INTERVALO_IDENTIDAD pasos (actualización periódica)
    /// - Cuando cambia significativamente el estado corporal
    pub fn integrar_en_pipeline(
        &mut self,
        _paso_actual: u64,
        num_columnas: usize,
        neuronas_totales: u32,
        hardware: &HardwareInfo,
        config: &ConfiguracionDinamica,
        memoria: &MemoriaAdaptativa,
        talamo: &TalamoDigital,
        interoceptivo: &SistemaInteroceptivo,
        entrada: &mut Entrada,
    ) {
        self.contador += 1;
        let debe_inyectar = !self.inyectado
            || self.contador % INTERVALO_IDENTIDAD == 0;

        if debe_inyectar {
            let descripcion = self.generar_descripcion(
                num_columnas,
                neuronas_totales,
                hardware,
                config,
                memoria,
                talamo,
                interoceptivo,
            );

            // Solo inyectar si la descripción cambió significativamente
            if descripcion != self.ultima_descripcion || !self.inyectado {
                self.ultima_descripcion = descripcion.clone();
                self.inyectado = true;

                // Inyectar en el texto de entrada
                let texto_identidad = format!("[IDENTIDAD: {}]", descripcion);
                if let Some(ref mut texto) = entrada.texto {
                    texto.push_str(&format!("\n{}", texto_identidad));
                } else {
                    entrada.texto = Some(texto_identidad);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
        use super::*;
        use crate::cerebro::hardware::{ConfiguracionDinamica, HardwareInfo};
        use crate::cerebro::memoria::MemoriaAdaptativa;
        use crate::cerebro::talamo::TalamoDigital;
        use crate::cerebro::interocepcion::SistemaInteroceptivo;
    
        fn contexto() -> (HardwareInfo, ConfiguracionDinamica, MemoriaAdaptativa, TalamoDigital, SistemaInteroceptivo) {
            let hw = HardwareInfo::detectar();
            let config = ConfiguracionDinamica::from_hardware(&hw);
            let memoria = MemoriaAdaptativa::nuevo(&config);
            let talamo = TalamoDigital::nuevo();
            let interoceptivo = SistemaInteroceptivo::nuevo();
            (hw, config, memoria, talamo, interoceptivo)
        }
    
        #[test]
        fn test_nuevo_estado_inicial() {
            let s = SistemaIdentidad::nuevo();
            assert!(s.ultima_descripcion.is_empty());
            assert_eq!(s.contador, 0);
            assert!(!s.inyectado);
        }
    
        #[test]
        fn test_generar_descripcion_incluye_identidad_base() {
            let (hw, config, memoria, talamo, interoceptivo) = contexto();
            let s = SistemaIdentidad::nuevo();
            let desc = s.generar_descripcion(8, 1600, &hw, &config, &memoria, &talamo, &interoceptivo);
            assert!(desc.contains("soy un cerebro SNN"), "Debe empezar con identidad base: {}", desc);
        }
    
        #[test]
        fn test_generar_descripcion_con_columnas() {
            let (hw, config, memoria, talamo, interoceptivo) = contexto();
            let s = SistemaIdentidad::nuevo();
            let desc = s.generar_descripcion(8, 1600, &hw, &config, &memoria, &talamo, &interoceptivo);
            assert!(desc.contains("8 columnas corticales"), "Debe mencionar 8 columnas: {}", desc);
            // 1600 / 8 = 200 neuronas por columna
            assert!(desc.contains("200 neuronas por columna"), "Debe calcular neuronas por columna: {}", desc);
        }
    
        #[test]
        fn test_generar_descripcion_sin_columnas_no_menciona() {
            let (hw, config, memoria, talamo, interoceptivo) = contexto();
            let s = SistemaIdentidad::nuevo();
            let desc = s.generar_descripcion(0, 0, &hw, &config, &memoria, &talamo, &interoceptivo);
            assert!(!desc.contains("columnas corticales"), "Sin columnas no debe mencionarlas: {}", desc);
        }
    
        #[test]
        fn test_generar_descripcion_incluye_arquitectura() {
            let (hw, config, memoria, talamo, interoceptivo) = contexto();
            let s = SistemaIdentidad::nuevo();
            let desc = s.generar_descripcion(8, 1600, &hw, &config, &memoria, &talamo, &interoceptivo);
            assert!(desc.contains("corro en arquitectura"), "Debe describir la arquitectura: {}", desc);
            assert!(desc.contains("núcleos"), "Debe mencionar núcleos: {}", desc);
        }
    
        #[test]
        fn test_generar_descripcion_incluye_talamo() {
            let (hw, config, memoria, talamo, interoceptivo) = contexto();
            let s = SistemaIdentidad::nuevo();
            let desc = s.generar_descripcion(8, 1600, &hw, &config, &memoria, &talamo, &interoceptivo);
            assert!(desc.contains("mi tálamo está en modo"), "Debe describir el tálamo: {}", desc);
            assert!(desc.contains("estoy en estado de"), "Debe describir el estado de consciencia: {}", desc);
        }
    
        #[test]
        fn test_generar_descripcion_incluye_estado_corporal() {
            let (hw, config, memoria, talamo, mut interoceptivo) = contexto();
            interoceptivo.estado_corporal.uso_cpu = 0.5;
            interoceptivo.estado_corporal.uso_ram = 0.3;
            interoceptivo.estado_corporal.temperatura_cpu = 60.0;
            let s = SistemaIdentidad::nuevo();
            let desc = s.generar_descripcion(8, 1600, &hw, &config, &memoria, &talamo, &interoceptivo);
            assert!(desc.contains("uso de CPU 50.0%"), "Debe reportar CPU 50%: {}", desc);
            assert!(desc.contains("uso de RAM 30.0%"), "Debe reportar RAM 30%: {}", desc);
            assert!(desc.contains("60.0°C"), "Debe reportar temperatura: {}", desc);
        }
    
        #[test]
        fn test_generar_descripcion_cpu_maximo() {
            let (hw, config, memoria, talamo, mut interoceptivo) = contexto();
            interoceptivo.estado_corporal.uso_cpu = 0.95;
            let s = SistemaIdentidad::nuevo();
            let desc = s.generar_descripcion(8, 1600, &hw, &config, &memoria, &talamo, &interoceptivo);
            assert!(desc.contains("mi CPU está al máximo"), "CPU al máximo debe reportarse: {}", desc);
        }
    
        #[test]
        fn test_generar_descripcion_bienestar_saludable() {
            let (hw, config, memoria, talamo, mut interoceptivo) = contexto();
            interoceptivo.homeostasis.bienestar_general = 0.9;
            interoceptivo.homeostasis.energia_disponible = 0.9;
            let s = SistemaIdentidad::nuevo();
            let desc = s.generar_descripcion(8, 1600, &hw, &config, &memoria, &talamo, &interoceptivo);
            assert!(desc.contains("me siento saludable"), "Bienestar alto debe reportarse: {}", desc);
            assert!(desc.contains("tengo energía alta"), "Energía alta debe reportarse: {}", desc);
        }
    
        #[test]
        fn test_generar_descripcion_dolor_sistemico() {
            let (hw, config, memoria, talamo, mut interoceptivo) = contexto();
            interoceptivo.homeostasis.dolor_sistemico = 0.6;
            let s = SistemaIdentidad::nuevo();
            let desc = s.generar_descripcion(8, 1600, &hw, &config, &memoria, &talamo, &interoceptivo);
            assert!(desc.contains("estoy experimentando dolor sistémico"), "Dolor alto debe reportarse: {}", desc);
        }
    
        #[test]
        fn test_generar_descripcion_memoria() {
            let (hw, config, memoria, talamo, interoceptivo) = contexto();
            let s = SistemaIdentidad::nuevo();
            let desc = s.generar_descripcion(8, 1600, &hw, &config, &memoria, &talamo, &interoceptivo);
            assert!(desc.contains("tengo 0 neuronas activas"), "Debe reportar neuronas: {}", desc);
        }
    
        #[test]
        fn test_integrar_pipeline_inyecta_en_primer_paso() {
            let (hw, config, memoria, talamo, interoceptivo) = contexto();
            let mut s = SistemaIdentidad::nuevo();
            let mut entrada = Entrada::vacía();
            s.integrar_en_pipeline(1, 8, 1600, &hw, &config, &memoria, &talamo, &interoceptivo, &mut entrada);
            assert!(s.inyectado, "Debe inyectar en el primer paso");
            assert!(entrada.texto.is_some(), "La entrada debe tener texto de identidad");
            let texto = entrada.texto.as_ref().unwrap();
            assert!(texto.contains("[IDENTIDAD:"), "Debe contener marcador de identidad: {}", texto);
        }
    
        #[test]
        fn test_integrar_pipeline_no_reinyecta_si_no_cambia() {
            let (hw, config, memoria, talamo, interoceptivo) = contexto();
            let mut s = SistemaIdentidad::nuevo();
            let mut entrada = Entrada::vacía();
            s.integrar_en_pipeline(1, 8, 1600, &hw, &config, &memoria, &talamo, &interoceptivo, &mut entrada);
            let texto_1 = entrada.texto.as_ref().unwrap().clone();
            // Segunda llamada sin cambios: no debe reinyectar (misma descripción)
            let mut entrada2 = Entrada::vacía();
            s.integrar_en_pipeline(2, 8, 1600, &hw, &config, &memoria, &talamo, &interoceptivo, &mut entrada2);
            assert!(entrada2.texto.is_none(), "No debe reinyectar identidad sin cambios");
            // El texto inyectado debe incrustar la descripción almacenada
            assert!(texto_1.contains(&s.ultima_descripcion), "Debe contener la descripción");
        }
    
        #[test]
        fn test_integrar_pipeline_reinyecta_cada_intervalo() {
            let (hw, config, memoria, talamo, mut interoceptivo) = contexto();
            let mut s = SistemaIdentidad::nuevo();
            let mut entrada = Entrada::vacía();
            s.integrar_en_pipeline(1, 8, 1600, &hw, &config, &memoria, &talamo, &interoceptivo, &mut entrada);
            // Cambiar estado corporal para que la descripción difiera
            interoceptivo.estado_corporal.temperatura_cpu = 90.0;
            // Llamar hasta alcanzar el intervalo (500) + 1 extra para forzar
            let mut entrada_intervalo = Entrada::vacía();
            s.contador = INTERVALO_IDENTIDAD - 1;
            s.integrar_en_pipeline(501, 8, 1600, &hw, &config, &memoria, &talamo, &interoceptivo, &mut entrada_intervalo);
            assert!(entrada_intervalo.texto.is_some(), "Debe reinyectar al alcanzar el intervalo");
        }
    }
