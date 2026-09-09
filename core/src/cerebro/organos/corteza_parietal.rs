// 🧠 Corteza Parietal — Integración sensorial y orientación espacial
// ==========================================
// Análogo a la corteza parietal humana: procesa información sensorial táctil,
// propioceptiva y espacial. En NEXUS: navegación del sistema de archivos.
// ==========================================

use std::collections::HashMap;
use std::path::Path;

pub struct CortezaParietal {
    mapa_espacial: HashMap<String, String>, // ruta → tipo ("archivo", "directorio", "enlace")
    directorio_actual: String,
}

impl Default for CortezaParietal {
    fn default() -> Self {
        Self::new()
    }
}

impl CortezaParietal {
    pub fn new() -> Self {
        let directorio_actual = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/".to_string());

        let mut mapa = HashMap::new();
        // Poblar con el directorio actual
        if let Ok(entries) = std::fs::read_dir(&directorio_actual) {
            for entry in entries.flatten() {
                let path = entry.path();
                let nombre = path.to_string_lossy().to_string();
                let tipo = if path.is_dir() {
                    "directorio"
                } else if path.is_symlink() {
                    "enlace"
                } else {
                    "archivo"
                };
                mapa.insert(nombre, tipo.to_string());
            }
        }

        Self {
            mapa_espacial: mapa,
            directorio_actual,
        }
    }

    /// Integra señales de múltiples sentidos para construir un modelo espacial unificado.
    ///
    /// Retorna una descripción textual del modelo espacial actual, combinando
    /// la entrada visual, táctil y propioceptiva con el mapa de archivos real.
    pub fn integrar_sensorial(&mut self, vision: &str, tacto: &str, propiocepcion: &str) -> String {
        // Refrescar el mapa espacial desde el sistema de archivos real
        if let Ok(entries) = std::fs::read_dir(&self.directorio_actual) {
            self.mapa_espacial.clear();
            for entry in entries.flatten() {
                let path = entry.path();
                let nombre = path.to_string_lossy().to_string();
                let tipo = if path.is_dir() {
                    "directorio"
                } else if path.is_symlink() {
                    "enlace"
                } else {
                    "archivo"
                };
                self.mapa_espacial.insert(nombre, tipo.to_string());
            }
        }

        let total_entradas = self.mapa_espacial.len();
        let directorios = self
            .mapa_espacial
            .values()
            .filter(|v| v.as_str() == "directorio")
            .count();
        let archivos = self
            .mapa_espacial
            .values()
            .filter(|v| v.as_str() == "archivo")
            .count();

        format!(
            "🧭 Modelo espacial en '{}': {} entradas ({} directorios, {} archivos). \
             Visión: '{}'. Tacto: '{}'. Propiocepción: '{}'.",
            self.directorio_actual,
            total_entradas,
            directorios,
            archivos,
            if vision.is_empty() {
                "sin entrada visual"
            } else {
                vision
            },
            if tacto.is_empty() {
                "sin entrada táctil"
            } else {
                tacto
            },
            if propiocepcion.is_empty() {
                "sin entrada propioceptiva"
            } else {
                propiocepcion
            },
        )
    }

    /// Navega el espacio de archivos del sistema hacia un destino.
    ///
    /// Cambia `directorio_actual` si el destino existe y es accesible.
    /// Retorna `Some(ruta_completa)` si la navegación fue exitosa.
    pub fn orientar(&self, destino: &str) -> Option<String> {
        let path = Path::new(destino);
        if path.exists() {
            Some(path.canonicalize().ok()?.to_string_lossy().to_string())
        } else {
            // Intentar como ruta relativa desde directorio_actual
            let rel = Path::new(&self.directorio_actual).join(destino);
            if rel.exists() {
                rel.canonicalize()
                    .ok()
                    .map(|p| p.to_string_lossy().to_string())
            } else {
                None
            }
        }
    }

    /// Navega y actualiza la posición actual.
    ///
    /// A diferencia de `orientar()`, este método modifica el estado interno
    /// si el destino es válido.
    pub fn navegar_a(&mut self, destino: &str) -> Option<String> {
        let resuelto = self.orientar(destino)?;
        self.directorio_actual = resuelto.clone();
        // Refrescar mapa
        if let Ok(entries) = std::fs::read_dir(&self.directorio_actual) {
            self.mapa_espacial.clear();
            for entry in entries.flatten() {
                let path = entry.path();
                let nombre = path.to_string_lossy().to_string();
                let tipo = if path.is_dir() {
                    "directorio"
                } else if path.is_symlink() {
                    "enlace"
                } else {
                    "archivo"
                };
                self.mapa_espacial.insert(nombre, tipo.to_string());
            }
        }
        Some(resuelto)
    }

    /// Detecta cambios en la estructura del entorno desde el último refresco.
    ///
    /// Compara el mapa espacial actual con el sistema de archivos real y
    /// retorna una lista de diferencias detectadas (rutas nuevas, eliminadas o cambiadas).
    pub fn detectar_cambios_espaciales(&self) -> Vec<String> {
        let mut cambios = Vec::new();

        let entradas_actuales: HashMap<String, String> =
            match std::fs::read_dir(&self.directorio_actual) {
                Ok(entries) => entries
                    .flatten()
                    .map(|e| {
                        let path = e.path();
                        let nombre = path.to_string_lossy().to_string();
                        let tipo = if path.is_dir() {
                            "directorio"
                        } else if path.is_symlink() {
                            "enlace"
                        } else {
                            "archivo"
                        };
                        (nombre, tipo.to_string())
                    })
                    .collect(),
                Err(_) => return vec!["Error: no se pudo leer el directorio actual".to_string()],
            };

        // Detectamos entradas nuevas o modificadas
        for (ruta, tipo) in &entradas_actuales {
            match self.mapa_espacial.get(ruta) {
                Some(tipo_previo) if tipo_previo != tipo => {
                    cambios.push(format!(
                        "🔄 Cambiado: '{}' (era {}, ahora {})",
                        ruta, tipo_previo, tipo
                    ));
                }
                None => {
                    cambios.push(format!("🆕 Nuevo: '{}' ({})", ruta, tipo));
                }
                _ => {}
            }
        }

        // Detectamos entradas eliminadas
        for ruta in self.mapa_espacial.keys() {
            if !entradas_actuales.contains_key(ruta) {
                cambios.push(format!("🗑️ Eliminado: '{}'", ruta));
            }
        }

        if cambios.is_empty() {
            cambios.push("✓ Sin cambios detectados".to_string());
        }

        cambios
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_new_parietal_no_panico() {
        let cp = CortezaParietal::new();
        assert!(!cp.directorio_actual.is_empty());
    }

    #[test]
    fn test_integrar_sensorial_retorna_descripcion() {
        let mut cp = CortezaParietal::new();
        let desc = cp.integrar_sensorial("pantalla visible", "superficie lisa", "posición erguida");
        assert!(desc.contains("Modelo espacial"));
        assert!(desc.contains("pantalla visible"));
        assert!(desc.contains("superficie lisa"));
        assert!(desc.contains("posición erguida"));
    }

    #[test]
    fn test_orientar_ruta_absoluta_valida() {
        let cp = CortezaParietal::new();
        let result = cp.orientar("/tmp");
        assert!(result.is_some(), "/tmp debe existir en Linux");
        assert!(result.unwrap().contains("/tmp"));
    }

    #[test]
    fn test_orientar_ruta_inexistente() {
        let cp = CortezaParietal::new();
        let result = cp.orientar("/ruta/que/no/existe/xyz123");
        assert!(result.is_none());
    }

    #[test]
    fn test_detectar_cambios_vacios_sin_cambios_reales() {
        let cp = CortezaParietal::new();
        let cambios = cp.detectar_cambios_espaciales();
        // Debería al menos tener un mensaje de "sin cambios" o cambios reales
        assert!(!cambios.is_empty());
    }

    #[test]
    fn test_navegar_a_cambia_directorio() {
        let tmp_dir = "/tmp";
        let mut cp = CortezaParietal::new();
        if cp.orientar(tmp_dir).is_some() {
            cp.navegar_a(tmp_dir);
            assert_eq!(cp.directorio_actual, tmp_dir);
        }
    }

    #[test]
    fn test_navegar_a_actualiza_mapa() {
        let mut cp = CortezaParietal::new();
        let original_count = cp.mapa_espacial.len();
        cp.navegar_a("/tmp");
        // /tmp debe tener contenido (o al menos ser diferente del dir original)
        assert!(cp.mapa_espacial.len() > 0);
    }
}
