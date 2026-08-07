// ==========================================
// DIAGNÓSTICO DEL ORQUESTADOR
// ==========================================
// Métodos de autodiagnóstico: escaneo de warnings, salud corporal,
// nivel de estrés de la Ínsula, autocorrección sugerida.
// ==========================================
use super::constructor::Orquestador;
use std::process::Command;
use tracing::{info, warn};

impl Orquestador {
    /// Escanea el output de `cargo build --lib` en busca de warnings
    /// y los inyecta en la Ínsula como "dolor corporal".
    /// Retorna un resumen legible de los warnings detectados.
    #[allow(clippy::wrong_self_convention)]
    fn escanear_warnings(&self) -> String {
        let output = Command::new("cargo")
            .args([
                "build",
                "--lib",
                "-p",
                "nexus_ultimate_core",
                "--message-format=json",
            ])
            .current_dir(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")))
            .output();

        let stderr = match output {
            Ok(ref o) => String::from_utf8_lossy(&o.stderr).to_string(),
            Err(e) => {
                warn!("⚠️ [ÍNSULA] Error ejecutando cargo build: {}", e);
                return String::new();
            }
        };

        let mut total_warnings = 0u32;
        let mut archivos_afectados: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        // Patrones de warning comunes en Rust
        let patrones = [
            "warning:",
            "unused import:",
            "unused variable:",
            "field is never read",
            "associated function is never used",
            "function is never used",
            "value assigned to",
            "unexpected `cfg`",
            "method is never used",
        ];

        for linea in stderr.lines() {
            let lower = linea.to_lowercase();
            if !patrones.iter().any(|p| lower.contains(p)) {
                continue;
            }

            // Extraer archivo y línea del formato: "  --> archivo.rs:line:col"
            let (archivo, num_linea) = {
                let mut arch = String::from("unknown");
                let mut num: u32 = 0;
                for (i, part) in linea.split_whitespace().enumerate() {
                    if part.contains("-->") && i + 1 < linea.split_whitespace().count() {
                        let path_part = linea.split_whitespace().nth(i + 1).unwrap_or("");
                        if let Some(pos) = path_part.find(':') {
                            arch = path_part[..pos].to_string();
                            let rest = &path_part[pos + 1..];
                            if let Some(pos2) = rest.find(':') {
                                num = rest[..pos2].parse().unwrap_or(0);
                            }
                        }
                    }
                }
                (arch, num)
            };

            // Extraer el mensaje de warning (después del primer `warning:`)
            let mensaje = if let Some(pos) = linea.find("warning:") {
                let after = &linea[pos + 8..];
                if let Some(arrow_pos) = after.find("-->") {
                    &after[..arrow_pos]
                } else {
                    after
                }
            } else {
                linea
            };

            let mensaje_clean = mensaje.trim().to_string();
            if mensaje_clean.is_empty() {
                continue;
            }

            // Inyectar en la Ínsula sin unwrap — si falla el lock, se degrada con gracia
            if let Ok(mut insula_guard) = self.insula.lock() {
                insula_guard.sentir_warning(&archivo, num_linea, &mensaje_clean);
            } else {
                warn!("⚠️ [ÍNSULA] No se pudo adquirir lock para inyectar warning");
            }
            if !archivo.is_empty() && archivo != "unknown" {
                archivos_afectados.insert(archivo.clone());
            }
            total_warnings += 1;
        }

        if total_warnings == 0 {
            info!("✅ [ÍNSULA] Escaneo limpio — 0 warnings detectados.");
            return String::new();
        }

        // Leer nivel de estrés sin unwrap
        let nivel_estres = if let Ok(insula_guard) = self.insula.lock() {
            insula_guard.nivel_estres() * 100.0
        } else {
            0.0
        };

        let resumen = format!(
            "⚠️ [ÍNSULA] {} warnings detectados en {} archivo(s). Estrés corporal: {:.1}%",
            total_warnings,
            archivos_afectados.len(),
            nivel_estres,
        );
        warn!("{}", resumen);

        // Si el umbral de autocorrección se excede, registrar los archivos más doloridos
        if let Ok(guard) = self.insula.lock() {
            if guard.necesita_autocorreccion() {
                let doloridos = guard.archivos_doloridos();
                for (arch, count) in &doloridos {
                    warn!("   🩹 {}: {} warnings", arch, count);
                }
                if let Some((archivo_critico, acciones)) = guard.sugerir_correccion() {
                    warn!("   🔧 Prioridad: {} — sugerencias:", archivo_critico);
                    for accion in &acciones {
                        warn!("      • {}", accion);
                    }
                }
            }
        }

        resumen
    }

    /// Ejecuta autodiagnóstico completo: escanea warnings y devuelve diagnóstico de salud corporal.
    pub fn autodiagnosticar(&self) -> String {
        let resumen_warnings = self.escanear_warnings();
        let estado_insula = if let Ok(guard) = self.insula.lock() {
            guard.estado_interno()
        } else {
            "⚠️ [ÍNSULA] Lock no disponible".to_string()
        };
        let salud_corteza = self.corteza.diagnosticar_salud("WEBCLAW");

        let mut diagnostico = format!(
            "🧬 [AUTODIAGNÓSTICO NEXUS]\n{}\n🧠 Corteza: {}\n{}",
            estado_insula,
            salud_corteza,
            if resumen_warnings.is_empty() {
                "✅ Sin advertencias de compilación.".to_string()
            } else {
                resumen_warnings
            }
        );

        // Verificar si necesita descanso o autocorrección sin unwrap
        if let Ok(guard) = self.insula.lock() {
            if guard.necesita_descanso() {
                diagnostico
                    .push_str("\n⚠️ [ÍNSULA] NEXUS necesita descanso. Estrés corporal elevado.");
            }
            if guard.necesita_autocorreccion() {
                if let Some((archivo, acciones)) = guard.sugerir_correccion() {
                    diagnostico.push_str(&format!(
                        "\n🔧 [AUTOCORRECCIÓN SUGERIDA] Archivo: {}",
                        archivo
                    ));
                    for accion in &acciones {
                        diagnostico.push_str(&format!("\n   • {}", accion));
                    }
                }
            }
        }

        diagnostico
    }

    /// Diagnóstico rápido de salud del sistema.
    pub fn diagnostico(&self) -> String {
        let salud = self.corteza.diagnosticar_salud("WEBCLAW");
        let estado = if let Ok(guard) = self.insula.lock() {
            guard.estado_interno()
        } else {
            "⚠️ Lock no disponible".to_string()
        };
        let herramientas = if let Ok(mcp_guard) = self.mcp.lock() {
            mcp_guard.herramientas.len()
        } else {
            0
        };
        format!(
            "🧠 Salud: {} | {} | 🔧 {} herramientas MCP",
            salud, estado, herramientas
        )
    }
}
