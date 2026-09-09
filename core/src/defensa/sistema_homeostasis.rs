// ==========================================
// SISTEMA HOMEOSTASIS - CURACIÓN NO DESTRUCTIVA
// ==========================================
// Supervisa tentáculos dañados y los restaura si han pasado suficiente
// tiempo sin fallos. No borra recuerdos, solo actualiza el estado de salud.
// ==========================================

use crate::memoria::memory::MemoriaPulso;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

pub struct SistemaHomeostasis {
    pulso_memoria: Arc<MemoriaPulso>,
}

impl SistemaHomeostasis {
    pub fn new(db_path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let pulso_memoria = Arc::new(MemoriaPulso::new(db_path)?);
        info!("🏥 Sistema Homeostasis activo con MemoriaPulso.");
        Ok(Self { pulso_memoria })
    }

    pub fn ciclo_de_curacion(&self) -> Vec<String> {
        self.pulso_memoria.ciclo_de_curacion().unwrap_or_default()
    }

    pub fn rotar_y_exportar_sesiones(
        &self,
        limite: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Obtener las sesiones recientes en caliente y exportarlas
        if let Ok(sesiones) = self.pulso_memoria.obtener_sesiones_recientes(limite) {
            let path_dir = PathBuf::from("C:/Users/crisp/NEXUS_ULTIMATE_CORE/brain/history");
            let _ = std::fs::create_dir_all(&path_dir);

            for sesion_id in sesiones {
                if let Ok(md_content) = self.pulso_memoria.generar_contenido_markdown(&sesion_id) {
                    let path_md = path_dir.join(format!("sesion_{}.md", sesion_id));
                    let _ = std::fs::write(path_md, md_content);
                }
            }
        }

        // 2. Ejecutar la poda de las sesiones que superen el límite
        self.pulso_memoria
            .rotar_sesiones_limite(limite)
            .map_err(|e| e.into())
    }

    /// 🧠 [NERVIO 3] Alimentar el Hipocampo con la sesión más reciente de Antigravity.
    /// Lee el transcript.jsonl directamente con rusqlite — sin dependencia circular.
    pub fn consolidar_en_hipocampo(&self) -> Result<(), Box<dyn std::error::Error>> {
        let brain_dir = std::path::Path::new("C:/Users/crisp/NEXUS_ULTIMATE_CORE/brain");
        if !brain_dir.exists() {
            return Ok(()); // Silencioso si Antigravity no está activo
        }

        // Buscar la conversación más reciente (UUID de 36 chars)
        let mut candidatos: Vec<(std::time::SystemTime, std::path::PathBuf)> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(brain_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if path.is_dir()
                    && name.len() == 36
                    && name.chars().filter(|&c| c == '-').count() == 4
                {
                    let transcript = path
                        .join(".system_generated")
                        .join("logs")
                        .join("transcript.jsonl");
                    if transcript.exists() {
                        if let Ok(meta) = std::fs::metadata(&transcript) {
                            if let Ok(m) = meta.modified() {
                                candidatos.push((m, transcript));
                            }
                        }
                    }
                }
            }
        }
        candidatos.sort_by(|a, b| b.0.cmp(&a.0));
        let transcript_path = match candidatos.into_iter().next() {
            Some((_, p)) => p,
            None => return Ok(()),
        };

        // Extraer datos del transcript
        use std::io::BufRead;
        let file = std::fs::File::open(&transcript_path)?;
        let reader = std::io::BufReader::new(file);
        let mut archivos: Vec<String> = Vec::new();
        let mut decisiones: Vec<String> = Vec::new();

        for line in reader.lines().flatten() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                if let Some(calls) = json.get("tool_calls").and_then(|v| v.as_array()) {
                    for call in calls {
                        let tool = call.get("tool_name").and_then(|v| v.as_str()).unwrap_or("");
                        let args = call.get("arguments");
                        if matches!(
                            tool,
                            "write_to_file" | "replace_file_content" | "multi_replace_file_content"
                        ) {
                            if let Some(p) = args
                                .and_then(|a| a.get("TargetFile"))
                                .and_then(|v| v.as_str())
                            {
                                let short = p.replace("C:/Users/crisp/NEXUS_ULTIMATE_CORE/", "");
                                if !archivos.contains(&short) {
                                    archivos.push(short);
                                }
                            }
                        }
                    }
                }
                if let Some("MODEL") = json.get("source").and_then(|v| v.as_str()) {
                    if let Some(content) = json.get("content").and_then(|v| v.as_str()) {
                        for linea in content.lines() {
                            let l = linea.trim();
                            if (l.starts_with("###") || l.starts_with("✅"))
                                && l.len() > 10
                                && decisiones.len() < 10
                            {
                                decisiones.push(l.chars().take(120).collect());
                            }
                        }
                    }
                }
            }
        }

        archivos.dedup();
        archivos.truncate(20);
        decisiones.dedup();

        // Generar latest.md
        let ahora = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        let mut md = format!("# 🧠 Sesión Consolidada — {}\n\n> Generado automáticamente por Homeostasis→Hipocampo de NEXUS\n\n", ahora);
        if !archivos.is_empty() {
            md.push_str("## 📁 Archivos Modificados\n");
            for f in &archivos {
                md.push_str(&format!("- `{}`\n", f));
            }
            md.push('\n');
        }
        if !decisiones.is_empty() {
            md.push_str("## 🏛️ Decisiones y Hitos\n");
            for d in &decisiones {
                md.push_str(&format!("- {}\n", d));
            }
            md.push('\n');
        }
        md.push_str("---\n*Leer este archivo al inicio de la próxima sesión para recuperar contexto completo.*\n");

        // Exportar a brain/sessions/latest.md
        let sessions_dir = PathBuf::from("C:/Users/crisp/NEXUS_ULTIMATE_CORE/brain/sessions");
        let _ = std::fs::create_dir_all(&sessions_dir);
        std::fs::write(sessions_dir.join("latest.md"), &md)?;
        let date_str = chrono::Local::now().format("%Y-%m-%d_%H-%M").to_string();
        let _ = std::fs::write(sessions_dir.join(format!("sesion_{}.md", date_str)), &md);

        // Insertar en hipocampo.db directamente
        let hipocampo_db = "C:/Users/crisp/NEXUS_ULTIMATE_CORE/data/hipocampo.db";
        if let Ok(conn) = Connection::open(hipocampo_db) {
            let _ = conn.execute(
                "CREATE TABLE IF NOT EXISTS memorias (id INTEGER PRIMARY KEY AUTOINCREMENT, contenido TEXT NOT NULL, timestamp DATETIME DEFAULT CURRENT_TIMESTAMP)",
                params![],
            );
            let _ = conn.execute("INSERT INTO memorias (contenido) VALUES (?)", params![md]);
        }

        info!(
            "🧠 [HOMEOSTASIS→HIPOCAMPO] latest.md generado ({} bytes, {} archivos, {} decisiones).",
            md.len(),
            archivos.len(),
            decisiones.len()
        );
        Ok(())
    }
}
