// ============================================================================
// 🦾 Efectores Locales (Claws) — Capacidad Operativa Local para engine-puro
// ============================================================================
// Provee la interfaz física para que el cerebro pueda:
//   1. Leer archivos locales en el workspace
//   2. Escribir/Modificar archivos
//   3. Ejecutar comandos de terminal controlados
// ============================================================================

use std::fs;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EfectorLocal {
    /// Contador de comandos ejecutados
    pub comandos_ejecutados: usize,
    /// Contador de archivos modificados
    pub archivos_modificados: usize,
    /// Último comando ejecutado
    pub ultimo_comando: String,
}

impl EfectorLocal {
    pub fn nuevo() -> Self {
        Self {
            comandos_ejecutados: 0,
            archivos_modificados: 0,
            ultimo_comando: String::new(),
        }
    }

    /// Lee un archivo local de forma segura
    pub fn leer_archivo(&self, ruta: &str) -> Result<String, String> {
        fs::read_to_string(ruta).map_err(|e| format!("Error al leer archivo {}: {}", ruta, e))
    }

    /// Escribe o sobrescribe un archivo local
    pub fn escribir_archivo(&mut self, ruta: &str, contenido: &str) -> Result<(), String> {
        fs::write(ruta, contenido)
            .map(|_| {
                self.archivos_modificados += 1;
            })
            .map_err(|e| format!("Error al escribir archivo {}: {}", ruta, e))
    }

    /// Ejecuta un comando en la shell local (sh) y retorna su salida (stdout + stderr)
    pub fn ejecutar_comando(&mut self, comando: &str) -> Result<String, String> {
        self.comandos_ejecutados += 1;
        self.ultimo_comando = comando.to_string();

        let output = Command::new("sh")
            .arg("-c")
            .arg(comando)
            .output()
            .map_err(|e| format!("Fallo al iniciar el comando '{}': {}", comando, e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        if output.status.success() {
            Ok(stdout)
        } else {
            Err(format!("Comando falló con estado {}.\nStdout: {}\nStderr: {}", output.status, stdout, stderr))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nuevo_contadores_cero() {
        let e = EfectorLocal::nuevo();
        assert_eq!(e.comandos_ejecutados, 0);
        assert_eq!(e.archivos_modificados, 0);
        assert!(e.ultimo_comando.is_empty());
    }

    #[test]
    fn test_leer_archivo_inexistente_error() {
        let e = EfectorLocal::nuevo();
        let err = e.leer_archivo("/ruta/que/no/existe/nexus_xyz.tmp");
        assert!(err.is_err(), "Leer un archivo inexistente debe fallar");
        assert!(err.unwrap_err().contains("Error al leer"));
    }

    #[test]
    fn test_leer_archivo_existente() {
        let e = EfectorLocal::nuevo();
        // Un archivo que siempre existe
        let contenido = e.leer_archivo("Cargo.toml").unwrap_or_default();
        assert!(contenido.contains("cerebro"), "Cargo.toml debe contener cerebro");
    }

    #[test]
    fn test_escribir_y_leer_archivo_tmp() {
        let mut e = EfectorLocal::nuevo();
        let ruta = std::env::temp_dir().join("nexus_efector_test.txt");
        let ruta_str = ruta.to_str().unwrap();
        // Escribir
        assert!(e.escribir_archivo(ruta_str, "contenido_prueba").is_ok());
        assert_eq!(e.archivos_modificados, 1);
        // Leer de vuelta
        assert_eq!(e.leer_archivo(ruta_str).unwrap(), "contenido_prueba");
        // Limpiar
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn test_escribir_sobrescribe() {
        let mut e = EfectorLocal::nuevo();
        let ruta = std::env::temp_dir().join("nexus_efector_overwrite.txt");
        let ruta_str = ruta.to_str().unwrap();
        assert!(e.escribir_archivo(ruta_str, "primera").is_ok());
        assert!(e.escribir_archivo(ruta_str, "segunda").is_ok());
        assert_eq!(e.archivos_modificados, 2);
        assert_eq!(e.leer_archivo(ruta_str).unwrap(), "segunda");
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn test_ejecutar_comando_echo() {
        let mut e = EfectorLocal::nuevo();
        let out = e.ejecutar_comando("echo hola_nexus");
        assert!(out.is_ok(), "echo debe ejecutarse correctamente");
        assert!(out.unwrap().contains("hola_nexus"));
        assert_eq!(e.comandos_ejecutados, 1);
        assert_eq!(e.ultimo_comando, "echo hola_nexus");
    }

    #[test]
    fn test_ejecutar_comando_fallido() {
        let mut e = EfectorLocal::nuevo();
        let out = e.ejecutar_comando("exit 3");
        assert!(out.is_err(), "Comando con estado no cero debe fallar");
        assert_eq!(e.comandos_ejecutados, 1);
    }

    #[test]
    fn test_ejecutar_comando_cadena() {
        let mut e = EfectorLocal::nuevo();
        // Ejecutar dos comandos acumula el contador
        let _ = e.ejecutar_comando("echo uno");
        let _ = e.ejecutar_comando("echo dos");
        assert_eq!(e.comandos_ejecutados, 2);
    }
}
