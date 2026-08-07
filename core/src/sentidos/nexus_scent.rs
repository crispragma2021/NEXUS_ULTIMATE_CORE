// ==========================================
// 👃 OLFATO DIGITAL — OlfatoDigital
// ==========================================
// Sentido 5 de NEXUS: Detector de anomalías en streams de datos.
// Olfatea logs, stderr, tráfico y código en busca de patrones de peligro.
// Análogo al olfato humano que detecta humo antes de ver el fuego.
//
// Desarrollado: 2026-06-27
// ==========================================

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

/// Nivel de alerta olfativa
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum NivelAlerta {
    /// Sin anomalías detectadas
    Limpio,
    /// Señal débil — observar
    Sospechoso(f32),
    /// Señal fuerte — intervenir
    Critico(f32),
}

impl NivelAlerta {
    pub fn descripcion(&self) -> String {
        match self {
            NivelAlerta::Limpio => "✅ Sin anomalías".to_string(),
            NivelAlerta::Sospechoso(score) => format!("⚠️ Sospechoso ({:.0}%)", score * 100.0),
            NivelAlerta::Critico(score) => format!("🚨 CRÍTICO ({:.0}%)", score * 100.0),
        }
    }
}

/// Anomalía detectada con contexto
#[derive(Debug, Clone)]
pub struct AnomaliaDetectada {
    pub patron: String,
    pub linea: String,
    pub nivel: NivelAlerta,
    pub fuente: String,
}

/// 👃 El Olfato Digital de NEXUS
/// Detecta anomalías en logs, código, stderr y streams de datos
/// antes de que se conviertan en fallos visibles.
pub struct OlfatoDigital {
    /// Patrones críticos — señalan errores graves
    patrones_criticos: Vec<&'static str>,
    /// Patrones de advertencia — señalan degradación
    patrones_advertencia: Vec<&'static str>,
    /// Historial de anomalías recientes (último escaneo)
    pub historial: Vec<AnomaliaDetectada>,
}

impl Default for OlfatoDigital {
    fn default() -> Self {
        Self::new()
    }
}

impl OlfatoDigital {
    pub fn new() -> Self {
        info!("👃 [OLFATO] Receptores olfativos activados. Detectando anomalías en streams.");
        Self {
            patrones_criticos: vec![
                "PANIC",
                "panic!",
                "thread 'main' panicked",
                "SIGKILL",
                "SIGSEGV",
                "SIGABRT",
                "OOM",
                "out of memory",
                "killed process",
                "FATAL",
                "fatal error",
                "stack overflow",
                "connection refused",
                "connection reset",
                "ENOSPC",
                "no space left",
                "permission denied",
                "Arritmia",
                "Saturación",
                "failed with result 'signal'",
                "exit code: 1",
                "exit status: 1",
                "error[E",  // Errores de compilación Rust
                "SQLSTATE", // Errores SQL
            ],
            patrones_advertencia: vec![
                "WARN",
                "WARNING",
                "warn!",
                "deprecated",
                "Deprecated",
                "retry",
                "Retry",
                "timeout",
                "Timeout",
                "slow",
                "latency",
                "high memory",
                "memory pressure",
                "restart counter",
                "Start request repeated",
                "rate limit",
                "429",
                "fallback",
            ],
            historial: Vec::new(),
        }
    }

    /// Olfatear un archivo de log y detectar anomalías
    pub fn olfatear_archivo(&mut self, path: &str) -> Vec<AnomaliaDetectada> {
        let mut anomalias = Vec::new();

        let contenido = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                warn!("👃 [OLFATO] No pude leer '{}': {}", path, e);
                return anomalias;
            }
        };

        // Recoger las últimas 200 líneas (Lines no implementa ExactSizeIterator para rev().take())
        let lineas: Vec<&str> = contenido.lines().collect();
        let inicio = lineas.len().saturating_sub(200);
        let ultimas = &lineas[inicio..];

        for (num_linea, linea) in ultimas.iter().enumerate().rev() {
            // Escanear últimas 200 líneas
            let linea_upper = linea.to_uppercase();

            for &patron in &self.patrones_criticos {
                if linea_upper.contains(&patron.to_uppercase()) {
                    anomalias.push(AnomaliaDetectada {
                        patron: patron.to_string(),
                        linea: format!(
                            "L{}: {}",
                            num_linea + 1,
                            linea.trim().chars().take(120).collect::<String>()
                        ),
                        nivel: NivelAlerta::Critico(0.9),
                        fuente: path.to_string(),
                    });
                    break;
                }
            }

            for &patron in &self.patrones_advertencia {
                if linea.contains(patron) {
                    anomalias.push(AnomaliaDetectada {
                        patron: patron.to_string(),
                        linea: format!(
                            "L{}: {}",
                            num_linea + 1,
                            linea.trim().chars().take(120).collect::<String>()
                        ),
                        nivel: NivelAlerta::Sospechoso(0.5),
                        fuente: path.to_string(),
                    });
                    break;
                }
            }
        }

        self.historial = anomalias.clone();
        anomalias
    }

    /// Olfatear texto de stderr/stdout directo (compilación, comandos)
    pub fn olfatear_stream(&self, stream: &str, fuente: &str) -> NivelAlerta {
        let upper = stream.to_uppercase();

        let criticos_encontrados: usize = self
            .patrones_criticos
            .iter()
            .filter(|&&p| upper.contains(&p.to_uppercase()))
            .count();

        let warnings_encontrados: usize = self
            .patrones_advertencia
            .iter()
            .filter(|&&p| stream.contains(p))
            .count();

        if criticos_encontrados > 0 {
            let score = (criticos_encontrados as f32 * 0.3).min(1.0);
            warn!(
                "👃 [OLFATO] 🚨 CRÍTICO en '{}': {} patrones peligrosos detectados",
                fuente, criticos_encontrados
            );
            NivelAlerta::Critico(score)
        } else if warnings_encontrados > 2 {
            info!(
                "👃 [OLFATO] ⚠️ Sospechoso en '{}': {} advertencias acumuladas",
                fuente, warnings_encontrados
            );
            NivelAlerta::Sospechoso((warnings_encontrados as f32 * 0.1).min(0.8))
        } else {
            NivelAlerta::Limpio
        }
    }

    /// Olfatear TODOS los logs del directorio de NEXUS
    pub fn olfatear_sistema(&mut self) -> Vec<AnomaliaDetectada> {
        let log_dir = Path::new("/home/soberano/NEXUS_ULTIMATE_CORE/logs");
        let mut todas_anomalias = Vec::new();

        if !log_dir.exists() {
            return todas_anomalias;
        }

        if let Ok(entradas) = fs::read_dir(log_dir) {
            for entrada in entradas.flatten() {
                let path = entrada.path();
                if path.extension().map(|e| e == "log").unwrap_or(false) {
                    let path_str = path.to_string_lossy().to_string();
                    let mut anomalias = self.olfatear_archivo(&path_str);
                    todas_anomalias.append(&mut anomalias);
                }
            }
        }

        // Ordenar por criticidad
        todas_anomalias.sort_by(|a, b| {
            let score_a = match &a.nivel {
                NivelAlerta::Critico(s) => *s + 1.0,
                NivelAlerta::Sospechoso(s) => *s,
                NivelAlerta::Limpio => 0.0,
            };
            let score_b = match &b.nivel {
                NivelAlerta::Critico(s) => *s + 1.0,
                NivelAlerta::Sospechoso(s) => *s,
                NivelAlerta::Limpio => 0.0,
            };
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        info!(
            "👃 [OLFATO] Escaneo del sistema completo: {} anomalías detectadas",
            todas_anomalias.len()
        );

        self.historial = todas_anomalias.clone();
        todas_anomalias
    }

    /// Resumen compacto para inyectar en el contexto del LLM
    pub fn resumen_para_llm(&self) -> String {
        if self.historial.is_empty() {
            return "👃 OLFATO: Sin anomalías detectadas en logs del sistema.".to_string();
        }

        let criticos: Vec<&AnomaliaDetectada> = self
            .historial
            .iter()
            .filter(|a| matches!(a.nivel, NivelAlerta::Critico(_)))
            .take(5)
            .collect();

        let sospechosos: Vec<&AnomaliaDetectada> = self
            .historial
            .iter()
            .filter(|a| matches!(a.nivel, NivelAlerta::Sospechoso(_)))
            .take(3)
            .collect();

        let mut resumen = format!(
            "👃 OLFATO DIGITAL — {} anomalías detectadas:\n",
            self.historial.len()
        );

        if !criticos.is_empty() {
            resumen.push_str("🚨 CRÍTICOS:\n");
            for a in criticos {
                resumen.push_str(&format!(
                    "  - [{}] {}\n",
                    a.fuente.split('/').last().unwrap_or("?"),
                    a.linea
                ));
            }
        }

        if !sospechosos.is_empty() {
            resumen.push_str("⚠️ ADVERTENCIAS:\n");
            for a in sospechosos {
                resumen.push_str(&format!(
                    "  - [{}] {}\n",
                    a.fuente.split('/').last().unwrap_or("?"),
                    a.linea
                ));
            }
        }

        resumen
    }

    /// Contar anomalías por tipo
    pub fn estadisticas(&self) -> HashMap<&str, usize> {
        let mut stats = HashMap::new();
        stats.insert(
            "criticos",
            self.historial
                .iter()
                .filter(|a| matches!(a.nivel, NivelAlerta::Critico(_)))
                .count(),
        );
        stats.insert(
            "sospechosos",
            self.historial
                .iter()
                .filter(|a| matches!(a.nivel, NivelAlerta::Sospechoso(_)))
                .count(),
        );
        stats.insert("total", self.historial.len());
        stats
    }
}
