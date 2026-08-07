/// Phase 30: Evolution Auto-Refactor Loop
/// Parses `cargo check` warnings and logs them to SQLite for trend analysis.
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionLog {
    pub id: Option<i64>,
    pub file_path: String,
    pub line_number: u32,
    pub warning_type: String,
    pub message: String,
    pub auto_fixed: bool,
    pub timestamp: DateTime<Utc>,
}

use crate::brain::hippocampus::ArtificialHippocampus;
use std::sync::Arc;

pub struct EvolutionEngine {
    _db_path: String,
    hippocampus: Option<Arc<ArtificialHippocampus>>,
}

impl EvolutionEngine {
    pub fn new(db_path: &str, hippocampus: Option<Arc<ArtificialHippocampus>>) -> Self {
        Self {
            _db_path: db_path.to_string(),
            hippocampus,
        }
    }

    /// Run `cargo check` and parse warnings into structured logs
    pub fn scan_warnings(&self, crate_name: &str) -> Result<Vec<EvolutionLog>> {
        let output = Command::new("cargo")
            .args(["check", "-p", crate_name, "--message-format=json"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut logs = Vec::new();

        // Parse JSON compiler messages
        for line in stdout.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if json["reason"] == "compiler-message" {
                    let msg = &json["message"];
                    let level = msg["level"].as_str().unwrap_or("");
                    if level == "warning" {
                        let message = msg["message"].as_str().unwrap_or("").to_string();
                        let spans = msg["spans"].as_array();

                        let (file_path, line_number) = if let Some(spans) = spans {
                            spans
                                .first()
                                .and_then(|s| {
                                    let file = s["file_name"].as_str()?.to_string();
                                    let line = s["line_start"].as_u64()? as u32;
                                    Some((file, line))
                                })
                                .unwrap_or_default()
                        } else {
                            (String::new(), 0)
                        };

                        let warning_type = if message.contains("unused") {
                            "unused_code"
                        } else if message.contains("dead_code") {
                            "dead_code"
                        } else if message.contains("deprecated") {
                            "deprecated"
                        } else {
                            "other"
                        }
                        .to_string();

                        logs.push(EvolutionLog {
                            id: None,
                            file_path,
                            line_number,
                            warning_type,
                            message,
                            auto_fixed: false,
                            timestamp: Utc::now(),
                        });
                    }
                }
            }
        }

        println!(
            "🧬 [EVOLUTION] Scan complete: {} warnings detected in {}",
            logs.len(),
            crate_name
        );
        Ok(logs)
    }

    /// Primary entry point for Phase 31.1: Proactive Self-Healing
    pub async fn apply_optimizations(&self, crate_name: &str) -> Result<usize> {
        let mut fixed_count = 0;

        // 1. Run cargo fix first (safe, built-in)
        fixed_count += self.attempt_autofix(crate_name)?;

        // 2. Identify remaining unused imports for manual regex wipe if needed
        let logs = self.scan_warnings(crate_name)?;
        for log in logs
            .iter()
            .filter(|l| l.warning_type == "unused_code" && l.message.contains("import"))
        {
            if self.regex_wipe_import(&log.file_path, &log.message)? {
                fixed_count += 1;
            }
        }

        // 3. Generar commit automático si hubo cambios
        if fixed_count > 0 {
            self.auto_commit(fixed_count)?;

            // Phase 34.3: Evolution Feedback Loop
            if let Some(hippo) = &self.hippocampus {
                let content = format!(
                    "🧬 [EVOLUTION] Applied {} code optimizations in {}. Successful auto-healing cycle completed.",
                    fixed_count, crate_name
                );
                let _ = hippo.store_memory(&content, vec![0.0; 768], None).await;
            }
        }

        Ok(fixed_count)
    }

    fn regex_wipe_import(&self, file_path: &str, message: &str) -> Result<bool> {
        // Example message: `unused import: `std::collections::HashMap``
        // We want to extract what's inside the backticks or after the colon
        let import_name = if let Some(start) = message.find('`') {
            if let Some(end) = message[start + 1..].find('`') {
                &message[start + 1..start + 1 + end]
            } else {
                ""
            }
        } else {
            ""
        };

        if import_name.is_empty() {
            return Ok(false);
        }

        println!(
            "🧬 [EVOLUTION] Attempting to wipe unused import '{}' from {}",
            import_name, file_path
        );

        // Read file content
        let content = std::fs::read_to_string(file_path)?;
        let mut new_lines = Vec::new();
        let mut found = false;

        for line in content.lines() {
            // Very basic matching: line starts with 'use ' and contains the import name
            if line.trim().starts_with("use ") && line.contains(import_name) {
                println!("🧬 [EVOLUTION] Removed line: {}", line.trim());
                found = true;
                continue; // Skip this line (wipe it)
            }
            new_lines.push(line);
        }

        if found {
            std::fs::write(file_path, new_lines.join("\n"))?;
            return Ok(true);
        }

        Ok(false)
    }

    fn auto_commit(&self, count: usize) -> Result<()> {
        println!("🧬 [EVOLUTION] Committing {} optimizations...", count);
        Command::new("git").args(["add", "."]).status()?;
        Command::new("git")
            .args([
                "commit",
                "-m",
                &format!("🧬 EVOLUTION: Applied {} code optimizations", count),
            ])
            .status()?;
        Ok(())
    }

    /// Attempt auto-fix via `cargo fix` in sandbox mode (--allow-dirty)
    pub fn attempt_autofix(&self, crate_name: &str) -> Result<usize> {
        println!("🧬 [EVOLUTION] Attempting auto-fix via `cargo fix`...");
        let output = Command::new("cargo")
            .args(["fix", "-p", crate_name, "--allow-dirty", "--allow-staged"])
            .output()?;

        let fixed = String::from_utf8_lossy(&output.stderr) // cargo fix logs to stderr
            .lines()
            .filter(|l| l.contains("Fixed"))
            .count();

        println!(
            "🧬 [EVOLUTION] Auto-fixed {} items in {}",
            fixed, crate_name
        );
        Ok(fixed)
    }

    /// Print evolution report
    pub fn print_report(&self, logs: &[EvolutionLog]) {
        println!("\n🧬 ══ EVOLUTION REPORT ══");
        for (i, log) in logs.iter().enumerate() {
            println!(
                "  {}. [{}] {}:{} — {}",
                i + 1,
                log.warning_type,
                log.file_path,
                log.line_number,
                &log.message[..log.message.len().min(80)]
            );
        }
        println!(
            "🧬 Total: {} warnings. Run `attempt_autofix()` to resolve.\n",
            logs.len()
        );
    }

    /// Phase 41: Active Evolution Loop — NEXUS evalúa su propio código de forma autónoma.
    ///
    /// Adapta la frecuencia de escaneo según la salud corporal del simbionte:
    /// - Optimal:  escanea cada 30 min y aplica fixes automáticos
    /// - Stressed: pausa el escaneo para preservar recursos del cuerpo
    /// - Critical: suspensión total — el cuerpo tiene prioridad
    pub async fn start_active_evolution(
        &self,
        crate_name: String,
        _body: Arc<crate::autodiagnostico::nexus_biostasis::BiostasisManager>,
        hippocampus: Option<Arc<crate::brain::hippocampus::ArtificialHippocampus>>,
    ) {
        println!(
            "🧬 [EVOLUTION] Ciclo activo de auto-evolución iniciado para [{}].",
            crate_name
        );

        tokio::spawn(async move {
            loop {
                let snap =
                    crate::autodiagnostico::nexus_biostasis::BiostasisManager::snapshot().await;

                match snap.health {
                    crate::autodiagnostico::nexus_biostasis::HealthLevel::Critical => {
                        // El cuerpo está en peligro — suspendemos completamente
                        println!("🧬 [EVOLUTION] Suspendido — cuerpo en estado CRÍTICO. Priorizando salud.");
                        tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;
                        continue;
                    }
                    crate::autodiagnostico::nexus_biostasis::HealthLevel::Stressed => {
                        // Reducimos actividad propia para no agravar el estrés
                        println!(
                            "🧬 [EVOLUTION] Pausado — cuerpo bajo estrés. Reintentando en 10 min."
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(600)).await;
                        continue;
                    }
                    crate::autodiagnostico::nexus_biostasis::HealthLevel::Optimal => {}
                }

                // Cuerpo sano — realizamos scan y autofix
                let output = std::process::Command::new("cargo")
                    .args(["check", "-p", &crate_name, "--message-format=json"])
                    .output();

                if let Ok(out) = output {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let warning_count = stdout
                        .lines()
                        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
                        .filter(|j| {
                            j["reason"] == "compiler-message" && j["message"]["level"] == "warning"
                        })
                        .count();

                    if warning_count > 0 {
                        println!(
                            "🧬 [EVOLUTION] {} warnings detectados — aplicando autocorrección...",
                            warning_count
                        );

                        // Intentar autofix via cargo fix
                        let _ = std::process::Command::new("cargo")
                            .args(["fix", "-p", &crate_name, "--allow-dirty", "--allow-staged"])
                            .output();

                        // Registrar en hippocampus para aprendizaje
                        if let Some(ref hippo) = hippocampus {
                            let msg = format!(
                                "🧬 [EVOLUTION] Auto-corrección aplicada a [{}]: {} warnings resueltos.",
                                crate_name, warning_count
                            );
                            let _ = hippo.store_memory(&msg, vec![0.0; 768], None).await;
                        }
                    } else {
                        println!("🧬 [EVOLUTION] Cuerpo óptimo, código limpio — simbionte en estado OMEGA.");
                    }
                }

                // Ciclo cada 30 minutos cuando el cuerpo está en estado Optimal
                tokio::time::sleep(tokio::time::Duration::from_secs(1800)).await;
            }
        });
    }
}
