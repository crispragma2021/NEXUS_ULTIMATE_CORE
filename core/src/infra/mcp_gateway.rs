// ==========================================
// MCP GATEWAY - Espina Dorsal de NEXUS
// ==========================================
// Conecta el cerebro con el arsenal de herramientas.
// Recibe intención, busca la herramienta correcta,
// la ejecuta y devuelve el resultado.
// ==========================================

use std::collections::HashMap;
use std::process::Command;
use tracing::{info, warn};

pub struct McpGateway {
    /// Herramientas registradas con sus comandos
    pub herramientas: HashMap<String, ComandoHerramienta>,
    /// Historial de ejecuciones
    pub historial: Vec<Ejecucion>,
}

#[derive(Debug, Clone)]
pub struct ComandoHerramienta {
    pub nombre: String,
    pub categoria: String,
    pub comando_base: String,
    pub parametros_ejemplo: String,
    pub descripcion: String,
}

#[derive(Debug, Clone)]
pub struct Ejecucion {
    pub herramienta: String,
    pub parametros: String,
    pub resultado: String,
    pub exitoso: bool,
    pub timestamp: String,
}

impl Default for McpGateway {
    fn default() -> Self {
        Self::new()
    }
}

impl McpGateway {
    pub fn new() -> Self {
        info!("🧬 [MCP GATEWAY] Espina Dorsal activada.");

        let mut herramientas = HashMap::new();

        // Registrar herramientas OSINT
        herramientas.insert(
            "theharvester".to_string(),
            ComandoHerramienta {
                nombre: "theHarvester".to_string(),
                categoria: "OSINT".to_string(),
                comando_base: "theharvester".to_string(),
                parametros_ejemplo: "-d ejemplo.com -b google".to_string(),
                descripcion: "Recolección de emails, subdominios y nombres".to_string(),
            },
        );

        herramientas.insert(
            "sherlock".to_string(),
            ComandoHerramienta {
                nombre: "Sherlock".to_string(),
                categoria: "OSINT".to_string(),
                comando_base: "sherlock".to_string(),
                parametros_ejemplo: "usuario".to_string(),
                descripcion: "Búsqueda de nombre de usuario en redes sociales".to_string(),
            },
        );

        herramientas.insert(
            "holehe".to_string(),
            ComandoHerramienta {
                nombre: "Holehe".to_string(),
                categoria: "OSINT".to_string(),
                comando_base: "holehe".to_string(),
                parametros_ejemplo: "email@ejemplo.com".to_string(),
                descripcion: "Verifica si un email está registrado en sitios web".to_string(),
            },
        );

        herramientas.insert(
            "whois".to_string(),
            ComandoHerramienta {
                nombre: "Whois".to_string(),
                categoria: "OSINT".to_string(),
                comando_base: "whois".to_string(),
                parametros_ejemplo: "ejemplo.com".to_string(),
                descripcion: "Información de registro de dominio".to_string(),
            },
        );

        herramientas.insert(
            "dig".to_string(),
            ComandoHerramienta {
                nombre: "Dig".to_string(),
                categoria: "OSINT".to_string(),
                comando_base: "dig".to_string(),
                parametros_ejemplo: "ejemplo.com ANY".to_string(),
                descripcion: "Consulta DNS avanzada".to_string(),
            },
        );

        // Registrar herramientas de escaneo
        herramientas.insert(
            "nmap".to_string(),
            ComandoHerramienta {
                nombre: "Nmap".to_string(),
                categoria: "Escaneo".to_string(),
                comando_base: "nmap".to_string(),
                parametros_ejemplo: "-sV -sC ejemplo.com".to_string(),
                descripcion: "Escaneo de puertos y detección de servicios".to_string(),
            },
        );

        herramientas.insert(
            "rustscan".to_string(),
            ComandoHerramienta {
                nombre: "RustScan".to_string(),
                categoria: "Escaneo".to_string(),
                comando_base: "rustscan".to_string(),
                parametros_ejemplo: "-a ejemplo.com".to_string(),
                descripcion: "Escaneo de puertos ultra rápido en Rust".to_string(),
            },
        );

        // Registrar herramientas web
        herramientas.insert(
            "gobuster".to_string(),
            ComandoHerramienta {
                nombre: "Gobuster".to_string(),
                categoria: "Web".to_string(),
                comando_base: "gobuster".to_string(),
                parametros_ejemplo: "dir -u http://ejemplo.com -w wordlist.txt".to_string(),
                descripcion: "Descubrimiento de directorios web".to_string(),
            },
        );

        // Registrar herramientas de sistema NEXUS
        herramientas.insert(
            "nexus_diagnostico".to_string(),
            ComandoHerramienta {
                nombre: "Diagnóstico NEXUS".to_string(),
                categoria: "NEXUS".to_string(),
                comando_base: "nexus-interno".to_string(),
                parametros_ejemplo: "diagnostico".to_string(),
                descripcion: "Diagnóstico interno de NEXUS".to_string(),
            },
        );

        herramientas.insert(
            "nexus_velocimetro".to_string(),
            ComandoHerramienta {
                nombre: "Velocímetro NEXUS".to_string(),
                categoria: "NEXUS".to_string(),
                comando_base: "nexus-interno".to_string(),
                parametros_ejemplo: "cuotas".to_string(),
                descripcion: "Estado de cuotas de API".to_string(),
            },
        );

        info!(
            "🧬 [MCP GATEWAY] {} herramientas registradas.",
            herramientas.len()
        );

        Self {
            herramientas,
            historial: Vec::new(),
        }
    }

    /// Ejecuta una herramienta con parámetros.
    pub fn ejecutar(&mut self, nombre: &str, parametros: &str) -> Result<String, String> {
        let herramienta = self.herramientas.get(nombre).ok_or_else(|| {
            format!(
                "Herramienta '{}' no encontrada. Usa 'listar' para ver disponibles.",
                nombre
            )
        })?;

        info!(
            "🔧 [MCP] Ejecutando {}: {} {}",
            nombre, herramienta.comando_base, parametros
        );

        let args: Vec<String> = parametros
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let output = Command::new(&herramienta.comando_base).args(&args).output();

        let (resultado, exitoso) = match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                let exito = o.status.success();

                if exito {
                    info!("✅ [MCP] {} ejecutado exitosamente.", nombre);
                } else {
                    warn!("⚠ [MCP] {} finalizó con error: {}", nombre, stderr);
                }

                (if stdout.is_empty() { stderr } else { stdout }, exito)
            }
            Err(e) => {
                warn!("❌ [MCP] Error al ejecutar {}: {}", nombre, e);
                (format!("Error: {}", e), false)
            }
        };

        let ejecucion = Ejecucion {
            herramienta: nombre.to_string(),
            parametros: parametros.to_string(),
            resultado: resultado.clone(),
            exitoso,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        self.historial.push(ejecucion);

        Ok(resultado)
    }

    /// Lista todas las herramientas disponibles.
    pub fn listar(&self) -> String {
        let mut lista = String::from("🧬 [MCP] Herramientas disponibles:\n\n");

        let mut categorias: HashMap<String, Vec<String>> = HashMap::new();
        for h in self.herramientas.values() {
            categorias
                .entry(h.categoria.clone())
                .or_default()
                .push(format!("  • {} — {}", h.nombre, h.descripcion));
        }

        let mut cats: Vec<_> = categorias.keys().collect();
        cats.sort();

        for cat in cats {
            lista.push_str(&format!("\n📂 {}\n", cat));
            for h in categorias.get(cat).unwrap_or(&Vec::new()) {
                lista.push_str(&format!("{}\n", h));
            }
        }

        lista
    }

    /// Busca herramientas por categoría o descripción.
    pub fn buscar_herramienta(&self, query: &str) -> Vec<ComandoHerramienta> {
        let q = query.to_lowercase();
        self.herramientas
            .values()
            .filter(|h| {
                h.nombre.to_lowercase().contains(&q)
                    || h.categoria.to_lowercase().contains(&q)
                    || h.descripcion.to_lowercase().contains(&q)
            })
            .cloned()
            .collect()
    }

    /// Obtiene el historial de ejecuciones.
    pub fn historial(&self) -> &Vec<Ejecucion> {
        &self.historial
    }

    /// Diagnóstico del Gateway.
    pub fn diagnostico(&self) -> String {
        format!(
            "🧬 [MCP GATEWAY] {} herramientas | {} ejecuciones en historial",
            self.herramientas.len(),
            self.historial.len()
        )
    }
}
