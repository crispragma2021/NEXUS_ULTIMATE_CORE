// ==========================================
// MÉDULA SOBERANA - Puente entre cerebro y cuerpo
// ==========================================
// Detecta intenciones de acción en las respuestas
// de NEXUS y las ejecuta usando la ManoSoberana
// o comandos del sistema.
// ==========================================

use crate::efectores::nexus_claw_pro::NexusClawPro as NexusClaw;
use std::sync::Arc;
use tokio::runtime::Handle;
use tracing::{info, warn};

pub struct MedulaSoberana {
    claw: Arc<NexusClaw>,
}

impl MedulaSoberana {
    pub fn new(claw: Arc<NexusClaw>) -> Self {
        info!("🧬 [MÉDULA SOBERANA] Activada - Puente cerebro-cuerpo listo");
        Self { claw }
    }

    /// Analiza una respuesta de NEXUS en busca de intenciones de acción
    /// y las ejecuta si las encuentra.
    /// Formato esperado: [ACCION: tipo] [PARAMETROS]
    pub fn ejecutar_si_hay_accion(&self, respuesta: &str) -> Option<String> {
        for linea in respuesta.lines() {
            let linea = linea.trim();

            if linea.starts_with("[ACCION:") {
                if let Some(inicio) = linea.find(']') {
                    let tipo_accion = linea[8..inicio].trim();
                    let parametros = linea[inicio + 1..].trim();

                    info!(
                        "⚡ [MÉDULA] Acción detectada: {} -> {}",
                        tipo_accion, parametros
                    );

                    return Some(self.ejecutar_accion(tipo_accion, parametros));
                }
            }
        }

        None
    }

    /// Ejecuta una acción concreta según su tipo.
    fn ejecutar_accion(&self, tipo: &str, parametros: &str) -> String {
        match tipo {
            "escribir_archivo" => self.escribir_archivo(parametros),
            "leer_archivo" => self.leer_archivo(parametros),
            "ejecutar_comando" => self.ejecutar_comando(parametros),
            "listar_directorio" => self.listar_directorio(parametros),
            "crear_directorio" => self.crear_directorio(parametros),
            _ => format!("⚠️ [MÉDULA] Tipo de acción desconocida: {}", tipo),
        }
    }

    /// Crea un archivo con contenido.
    /// Formato: "ruta/archivo.txt" "contenido"
    fn escribir_archivo(&self, parametros: &str) -> String {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        for c in parametros.chars() {
            match c {
                '"' | '\'' => in_quotes = !in_quotes,
                ' ' if !in_quotes => {
                    if !current.is_empty() {
                        args.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(c),
            }
        }
        if !current.is_empty() {
            args.push(current);
        }

        if args.len() < 2 {
            return "⚠️ [MÉDULA] Formato incorrecto. Usa: \"ruta/archivo.txt\" \"contenido\""
                .to_string();
        }

        let ruta = args[0].trim_matches('"').trim_matches('\'');
        let contenido_temp = args[1..].join(" ");
        let contenido = contenido_temp.trim_matches('"').trim_matches('\'');

        // Conexión Soberana: Usamos la Garra para escribir con auditoría
        let claw = self.claw.clone();
        let r = ruta.to_string();
        let c = contenido.to_string();

        match tokio::task::block_in_place(move || {
            Handle::current().block_on(async move { claw.escribir_archivo(&r, &c).await })
        }) {
            Ok(_) => {
                info!("📄 [MÉDULA] Archivo procesado: {}", ruta);
                format!("✅ Archivo escrito exitosamente en: {}", ruta)
            }
            Err(e) => {
                warn!("❌ [MÉDULA] Error al crear archivo: {}", e);
                format!("❌ Error al crear archivo: {}", e)
            }
        }
    }

    /// Lee el contenido de un archivo.
    fn leer_archivo(&self, ruta: &str) -> String {
        let ruta = ruta.trim_matches('"').trim_matches('\'');

        // Conexión Soberana: Usamos la Garra para leer con inteligencia
        let claw = self.claw.clone();
        let r = ruta.to_string();

        match tokio::task::block_in_place(move || {
            Handle::current().block_on(async move { claw.leer_archivo(&r).await })
        }) {
            Ok(contenido) => {
                info!("📖 [MÉDULA] Archivo leído: {}", ruta);
                format!("📖 Contenido de {}:\n{}", ruta, contenido)
            }
            Err(e) => {
                warn!("❌ [MÉDULA] Error al leer archivo: {}", e);
                format!("❌ Error al leer archivo: {}", e)
            }
        }
    }

    /// Ejecuta un comando del sistema.
    fn ejecutar_comando(&self, comando: &str) -> String {
        let comando = comando.trim_matches('"').trim_matches('\'');
        info!("💻 [MÉDULA] Delegando ejecución a NexusClaw: {}", comando);

        // Como ejecutar_comando es síncrono en este trait, usamos el handle de tokio
        let claw = self.claw.clone();
        let cmd = comando.to_string();

        match tokio::task::block_in_place(move || {
            Handle::current().block_on(async move { claw.ejecutar_inteligente(&cmd).await })
        }) {
            Ok(res) => format!("✅ [OBRERO] Acción completada:\n{}", res),
            Err(e) => format!("🧠 [OBRERO] Sugerencia/Aviso: {}", e),
        }
    }

    /// Lista el contenido de un directorio.
    fn listar_directorio(&self, ruta: &str) -> String {
        let ruta = ruta.trim_matches('"').trim_matches('\'');
        let claw = self.claw.clone();
        let r = ruta.to_string();

        match tokio::task::block_in_place(move || {
            Handle::current().block_on(async move { claw.ejecutar(&format!("ls -F {}", r)).await })
        }) {
            Ok(entradas) => {
                format!("📂 Contenido de {}:\n{}", ruta, entradas)
            }
            Err(e) => format!("❌ Error al listar directorio: {}", e),
        }
    }

    /// Crea un directorio.
    fn crear_directorio(&self, ruta: &str) -> String {
        let ruta = ruta.trim_matches('"').trim_matches('\'');
        let claw = self.claw.clone();
        let r = ruta.to_string();

        match tokio::task::block_in_place(move || {
            Handle::current()
                .block_on(async move { claw.ejecutar(&format!("mkdir -p {}", r)).await })
        }) {
            Ok(_) => format!("✅ Directorio creado: {}", ruta),
            Err(e) => format!("❌ Error al crear directorio: {}", e),
        }
    }
}
