// ==========================================
// 🧭 GEO HUNTER — Análisis Forense Digital
// ==========================================
// Extrae IPs y coordenadas geográficas de volcados HTML
// para inteligencia de reconocimiento.
//
// Legacy DNA: nexus-orquestador/src/geo_hunter.rs
// Absorbido: 11-Jun-2026

/// Analizador forense de volcados HTML.
/// Escanea contenido en busca de IPs y coordenadas geográficas.
pub struct GeoHunter {
    /// Ruta al archivo de volcado HTML
    pub dump_path: String,
}

impl Default for GeoHunter {
    fn default() -> Self {
        Self::new()
    }
}

impl GeoHunter {
    pub fn new() -> Self {
        Self {
            dump_path: String::new(),
        }
    }

    /// Escanea el volcado HTML y extrae IPs + coordenadas.
    pub fn analizar_volcado(&self, content: &str) -> GeoReporte {
        use regex::Regex;

        let re_ip = Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap();
        let re_coords = Regex::new(r"(-?\d+\.\d+),\s*(-?\d+\.\d+)").unwrap();

        let ips: Vec<String> = re_ip
            .captures_iter(content)
            .map(|cap| cap[1].to_string())
            .collect();

        let coordenadas: Vec<(f64, f64)> = re_coords
            .captures_iter(content)
            .filter_map(|cap| {
                let lat: f64 = cap[1].parse().ok()?;
                let lon: f64 = cap[2].parse().ok()?;
                Some((lat, lon))
            })
            .collect();

        GeoReporte { ips, coordenadas }
    }
}

/// Resultado del análisis forense geo-espacial.
#[derive(Debug, Clone)]
pub struct GeoReporte {
    pub ips: Vec<String>,
    pub coordenadas: Vec<(f64, f64)>,
}
