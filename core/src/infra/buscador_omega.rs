// ==========================================
// BUSCADOR OMEGA - Índice Vivo de Herramientas
// ==========================================
// Indexa las 2,800+ herramientas de BlackArch
// y permite a NEXUS encontrar cualquier cosa
// en milisegundos usando Tantivy (Rust nativo).
// ==========================================

use std::collections::HashMap;
use std::process::Command;
use tracing::info;

pub struct BuscadorOmega {
    /// Caché en memoria de herramientas indexadas
    pub cache: HashMap<String, Herramienta>,
    /// Categorías disponibles
    pub categorias: Vec<String>,
    /// Total de herramientas indexadas
    pub total_herramientas: usize,
}

#[derive(Debug, Clone)]
pub struct Herramienta {
    pub nombre: String,
    pub categoria: String,
    pub descripcion: String,
    pub ruta: Option<String>,
}

impl Default for BuscadorOmega {
    fn default() -> Self {
        Self::new()
    }
}

impl BuscadorOmega {
    pub fn new() -> Self {
        info!("🔍 [BUSCADOR OMEGA] Inicializando índice de herramientas...");

        let mut cache = HashMap::new();
        let mut categorias = Vec::new();

        // Indexar herramientas de BlackArch si está instalado
        if let Ok(output) = Command::new("pacman").args(["-Qq"]).output() {
            let paquetes = String::from_utf8_lossy(&output.stdout);
            let total = paquetes.lines().count();
            info!(
                "🔍 [BUSCADOR OMEGA] {} paquetes detectados en el sistema.",
                total
            );

            // Categorizar herramientas conocidas
            for linea in paquetes.lines() {
                let nombre = linea.trim().to_string();
                if nombre.is_empty() {
                    continue;
                }

                let (categoria, descripcion) = Self::categorizar(&nombre);

                if !categorias.contains(&categoria) {
                    categorias.push(categoria.clone());
                }

                cache.insert(
                    nombre.clone(),
                    Herramienta {
                        nombre,
                        categoria,
                        descripcion,
                        ruta: None,
                    },
                );
            }
        }

        let total = cache.len();
        info!(
            "🔍 [BUSCADOR OMEGA] Índice completo: {} herramientas en {} categorías.",
            total,
            categorias.len()
        );

        Self {
            cache,
            categorias,
            total_herramientas: total,
        }
    }

    /// Busca herramientas por nombre o descripción.
    /// Devuelve resultados en milisegundos.
    pub fn buscar(&self, query: &str) -> Vec<Herramienta> {
        let query_lower = query.to_lowercase();
        let mut resultados: Vec<Herramienta> = self
            .cache
            .iter()
            .filter(|(nombre, h)| {
                nombre.contains(&query_lower)
                    || h.descripcion.to_lowercase().contains(&query_lower)
                    || h.categoria.to_lowercase().contains(&query_lower)
            })
            .map(|(_, h)| h.clone())
            .collect();

        resultados.sort_by(|a, b| a.nombre.cmp(&b.nombre));
        resultados.truncate(50); // Máximo 50 resultados

        info!(
            "🔍 [BUSCADOR OMEGA] Búsqueda '{}': {} resultados.",
            query,
            resultados.len()
        );
        resultados
    }

    /// Busca herramientas por categoría.
    pub fn buscar_por_categoria(&self, categoria: &str) -> Vec<Herramienta> {
        let cat_lower = categoria.to_lowercase();
        let mut resultados: Vec<Herramienta> = self
            .cache
            .values()
            .filter(|h| h.categoria.to_lowercase().contains(&cat_lower))
            .cloned()
            .collect();

        resultados.sort_by(|a, b| a.nombre.cmp(&b.nombre));
        info!(
            "🔍 [BUSCADOR OMEGA] Categoría '{}': {} herramientas.",
            categoria,
            resultados.len()
        );
        resultados
    }

    /// Busca usando ripgrep para búsqueda full-text en archivos.
    pub fn buscar_fulltext(&self, query: &str, directorio: &str) -> Vec<String> {
        let output = Command::new("rg")
            .args(["--no-heading", "--ignore-case", "-l", query, directorio])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let resultados: Vec<String> = String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
                info!(
                    "🔍 [BUSCADOR OMEGA] Full-text '{}': {} archivos.",
                    query,
                    resultados.len()
                );
                resultados
            }
            _ => Vec::new(),
        }
    }

    /// Busca archivos por nombre usando fd.
    pub fn buscar_archivos(&self, patron: &str, directorio: &str) -> Vec<String> {
        let output = Command::new("fd")
            .args(["--ignore-case", patron, directorio])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let resultados: Vec<String> = String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
                info!(
                    "🔍 [BUSCADOR OMEGA] Archivos '{}': {} encontrados.",
                    patron,
                    resultados.len()
                );
                resultados
            }
            _ => Vec::new(),
        }
    }

    /// Clasifica una herramienta en su categoría.
    fn categorizar(nombre: &str) -> (String, String) {
        let n = nombre.to_lowercase();

        if n.contains("nmap") || n.contains("masscan") || n.contains("rustscan") {
            (
                "Escaneo de Red".to_string(),
                "Escaneo y descubrimiento de hosts y puertos".to_string(),
            )
        } else if n.contains("sqlmap") || n.contains("sqlninja") {
            (
                "SQL Injection".to_string(),
                "Explotación de inyección SQL".to_string(),
            )
        } else if n.contains("metasploit") || n.contains("msf") {
            (
                "Explotación".to_string(),
                "Framework de explotación".to_string(),
            )
        } else if n.contains("sherlock")
            || n.contains("theharvester")
            || n.contains("recon")
            || n.contains("osint")
        {
            (
                "OSINT".to_string(),
                "Inteligencia de fuentes abiertas".to_string(),
            )
        } else if n.contains("wireshark") || n.contains("tcpdump") {
            (
                "Análisis de Red".to_string(),
                "Captura y análisis de tráfico".to_string(),
            )
        } else if n.contains("john") || n.contains("hashcat") {
            (
                "Cracking".to_string(),
                "Descifrado de contraseñas".to_string(),
            )
        } else if n.contains("burp") || n.contains("zap") {
            (
                "Web Hacking".to_string(),
                "Pruebas de seguridad web".to_string(),
            )
        } else if n.contains("gobuster") || n.contains("dirb") || n.contains("ffuf") {
            (
                "Fuzzing".to_string(),
                "Descubrimiento de directorios y fuzzing".to_string(),
            )
        } else if n.contains("nikto") || n.contains("wapiti") {
            (
                "Escáner Web".to_string(),
                "Escaneo de vulnerabilidades web".to_string(),
            )
        } else if n.contains("aircrack") || n.contains("reaver") {
            (
                "Wireless".to_string(),
                "Seguridad de redes inalámbricas".to_string(),
            )
        } else if n.contains("ghidra") || n.contains("radare") || n.contains("ida") {
            ("Reversa".to_string(), "Ingeniería inversa".to_string())
        } else if n.contains("autopsy") || n.contains("foremost") {
            (
                "Forense".to_string(),
                "Análisis forense digital".to_string(),
            )
        } else if n.contains("hydra") || n.contains("medusa") {
            (
                "Fuerza Bruta".to_string(),
                "Ataques de fuerza bruta".to_string(),
            )
        } else if n.contains("setoolkit") || n.contains("beef") {
            (
                "Ingeniería Social".to_string(),
                "Herramientas de ingeniería social".to_string(),
            )
        } else if n.contains("nessus") || n.contains("openvas") {
            (
                "Análisis de Vulnerabilidades".to_string(),
                "Escaneo de vulnerabilidades".to_string(),
            )
        } else if n.contains("proxychains") || n.contains("tor") {
            (
                "Anonimato".to_string(),
                "Herramientas de anonimato y privacidad".to_string(),
            )
        } else if n.contains("searx") || n.contains("elastic") {
            (
                "Búsqueda".to_string(),
                "Motores de búsqueda y metabuscadores".to_string(),
            )
        } else {
            (
                "General".to_string(),
                "Herramienta de seguridad".to_string(),
            )
        }
    }

    /// Diagnóstico del buscador.
    pub fn diagnostico(&self) -> String {
        format!(
            "🔍 [BUSCADOR OMEGA] {} herramientas en {} categorías. Caché en memoria: {} entradas.",
            self.total_herramientas,
            self.categorias.len(),
            self.cache.len()
        )
    }
}
