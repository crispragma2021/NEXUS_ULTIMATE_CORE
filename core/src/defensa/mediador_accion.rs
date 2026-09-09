use crate::brain::immune::memory_shield::MemoryShieldGuard;
use crate::efectores::nexus_claw_pro::NexusClawPro;
use regex::Regex;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::path::Path;

pub struct MediadorAccion;

impl MediadorAccion {
    /// Calcula el hash SHA-256 de una cadena de texto.
    fn calcular_sha256(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Valida la integridad del archivo (drift detection) y actualiza su huella y backup.
    async fn validar_y_actualizar_huella(path_str: &str, new_content: &str) -> Result<(), String> {
        let db_path_buf = crate::infra::paths::resolve_path("nexus_intelligence.db");
        let db_path = db_path_buf
            .to_str()
            .ok_or_else(|| "🛑 [PATH_ERROR] No se pudo convertir db_path a cadena".to_string())?;
        let file_path = Path::new(path_str);

        // 1. Calcular hash actual en disco si existe
        let current_disk_hash = if file_path.exists() {
            match std::fs::read_to_string(file_path) {
                Ok(content) => Some(Self::calcular_sha256(&content)),
                Err(e) => {
                    return Err(format!(
                    "🛑 [READ_ERROR] No se pudo leer el archivo actual para validar integridad: {}",
                    e
                ))
                }
            }
        } else {
            None
        };

        // 2. Abrir base de datos con el escudo JIT desbloqueado
        let _guard = MemoryShieldGuard::new(db_path).map_err(|e| {
            format!(
                "🛑 [SHIELD_ERROR] Fallo al abrir el escudo de la base de datos: {}",
                e
            )
        })?;

        let conn = Connection::open(db_path).map_err(|e| {
            format!(
                "🛑 [DB_ERROR] Fallo al conectar a la base de datos de integridad: {}",
                e
            )
        })?;

        // 3. Consultar la última huella registrada
        let mut stmt = conn
            .prepare("SELECT hash FROM file_fingerprints WHERE path = ?1")
            .map_err(|e| {
                format!(
                    "🛑 [DB_ERROR] Fallo al preparar consulta de integridad: {}",
                    e
                )
            })?;

        let stored_hash: Option<String> = stmt.query_row([path_str], |row| row.get(0)).ok();

        // 4. Drift Detection (Detección de modificación externa)
        if let Some(ref disk_hash) = current_disk_hash {
            if let Some(ref db_hash) = stored_hash {
                if disk_hash != db_hash {
                    return Err(format!(
                        "🛑 [DRIFT_DETECTED] El archivo '{}' ha sido modificado externamente (hash mismatch). Abortando escritura para proteger integridad.",
                        path_str
                    ));
                }
            }
        }

        // 5. Respaldar contenido previo antes de sobrescribir
        if file_path.exists() {
            if let Ok(old_content) = std::fs::read_to_string(file_path) {
                let _ = conn.execute(
                    "INSERT OR REPLACE INTO file_backups (path, content) VALUES (?1, ?2)",
                    [path_str, &old_content],
                );
            }
        }

        // 6. Registrar la nueva huella
        let new_hash = Self::calcular_sha256(new_content);
        conn.execute(
            "INSERT OR REPLACE INTO file_fingerprints (path, hash) VALUES (?1, ?2)",
            [path_str, &new_hash],
        )
        .map_err(|e| {
            format!(
                "🛑 [DB_ERROR] Fallo al registrar la nueva huella digital: {}",
                e
            )
        })?;

        Ok(())
    }

    /// Ejecuta el rollback de un archivo a su estado respaldado anterior.
    async fn ejecutar_rollback(path_str: &str) -> Result<String, String> {
        let db_path_buf = crate::infra::paths::resolve_path("nexus_intelligence.db");
        let db_path = db_path_buf
            .to_str()
            .ok_or_else(|| "🛑 [PATH_ERROR] No se pudo convertir db_path a cadena".to_string())?;
        let file_path = Path::new(path_str);

        // Abrir base de datos con protección
        let _guard = MemoryShieldGuard::new(db_path).map_err(|e| {
            format!(
                "🛑 [SHIELD_ERROR] Fallo al abrir el escudo de la base de datos para rollback: {}",
                e
            )
        })?;

        let conn = Connection::open(db_path).map_err(|e| {
            format!(
                "🛑 [DB_ERROR] Fallo al conectar a la base de datos de backups: {}",
                e
            )
        })?;

        // Consultar el backup
        let mut stmt = conn
            .prepare("SELECT content FROM file_backups WHERE path = ?1")
            .map_err(|e| {
                format!(
                    "🛑 [DB_ERROR] Fallo al preparar consulta de rollback: {}",
                    e
                )
            })?;

        let old_content: Result<String, _> = stmt.query_row([path_str], |row| row.get(0));

        match old_content {
            Ok(content) => {
                // Escribir el contenido antiguo
                if let Err(e) = std::fs::write(file_path, &content) {
                    return Err(format!(
                        "🛑 [ROLLBACK_FAIL] Fallo al restaurar el archivo en disco: {}",
                        e
                    ));
                }

                // Actualizar la huella al hash del contenido restaurado
                let restored_hash = Self::calcular_sha256(&content);
                let _ = conn.execute(
                    "INSERT OR REPLACE INTO file_fingerprints (path, hash) VALUES (?1, ?2)",
                    [path_str, &restored_hash],
                );

                Ok(format!(
                    "🟢 [ROLLBACK_SUCCESS] Archivo '{}' restaurado con éxito a su estado anterior.",
                    path_str
                ))
            }
            Err(_) => Err(format!(
                "🛑 [ROLLBACK_FAIL] No se encontró ningún respaldo para el archivo: {}",
                path_str
            )),
        }
    }

    /// Procesa el flujo e invoca el músculo propio del Arquitecto para todos los bloques de escritura, lectura y consultas HTTP detectados.
    pub async fn procesar_y_manifestar(respuesta_ia: &str) -> String {
        let mut salida_final = respuesta_ia.to_string();
        let mut reportes: Vec<String> = Vec::new();

        // 1. Procesar bloques de Escritura Real: [[REAL: WRITE: ruta | contenido]] o [[WRITE: ruta | contenido]]
        if let Ok(re_real) = Regex::new(r"(?s)\[\[(?:REAL:\s*)?WRITE:\s*(.*?)\s*\|\s*(.*?)\s*\]\]")
        {
            for caps in re_real.captures_iter(respuesta_ia) {
                let ruta = caps.get(1).map_or("", |m| m.as_str().trim());
                let contenido = caps.get(2).map_or("", |m| m.as_str());

                // Validar integridad antes de escribir
                match Self::validar_y_actualizar_huella(ruta, contenido).await {
                    Ok(_) => match NexusClawPro::manifestar_en_silicio(ruta, contenido).await {
                        Ok(msg_exito) => reportes.push(msg_exito),
                        Err(msg_error) => reportes.push(msg_error.to_string()),
                    },
                    Err(msg_conflict) => {
                        reportes.push(msg_conflict);
                    }
                }
            }
        }

        // 2. Procesar bloques de Escritura Fantasma (Simulación): [[FANTASMA: WRITE: ruta | contenido]]
        if let Ok(re_ghost) = Regex::new(r"(?s)\[\[FANTASMA:\s*WRITE:\s*(.*?)\s*\|\s*(.*?)\s*\]\]")
        {
            for caps in re_ghost.captures_iter(respuesta_ia) {
                let ruta = caps.get(1).map_or("", |m| m.as_str().trim());
                let contenido = caps.get(2).map_or("", |m| m.as_str());

                // Simular diff
                let status_msg = if Path::new(ruta).exists() {
                    format!("🟢 [FANTASMA_WRITE] Simulación de modificación exitosa en: {}. Se propusieron {} bytes.", ruta, contenido.len())
                } else {
                    format!("🟢 [FANTASMA_WRITE] Simulación de creación exitosa en: {}. Se propuso escribir archivo nuevo de {} bytes.", ruta, contenido.len())
                };
                reportes.push(status_msg);
            }
        }

        // 3. Procesar bloques de Rollback: [[ROLLBACK: ruta]]
        if let Ok(re_rollback) = Regex::new(r"\[\[ROLLBACK:\s*(.*?)\s*\]\]") {
            for caps in re_rollback.captures_iter(respuesta_ia) {
                let ruta = caps.get(1).map_or("", |m| m.as_str().trim());

                match Self::ejecutar_rollback(ruta).await {
                    Ok(msg_exito) => reportes.push(msg_exito),
                    Err(msg_error) => reportes.push(msg_error),
                }
            }
        }

        // 4. Procesar bloques de Lectura: [[READ: ruta]] o [[ACTION: READ: ...]]
        if let Ok(re_read) = Regex::new(r"\[\[(?:ACCION:|ACTION:)?\s*READ:\s*(.*?)\s*\]\]") {
            for caps in re_read.captures_iter(respuesta_ia) {
                let ruta = caps.get(1).map_or("", |m| m.as_str().trim());

                match NexusClawPro::leer_de_silicio(ruta).await {
                    Ok(contenido) => {
                        reportes.push(format!(
                            "🟢 [NEXUS_CLAW_PRO] Archivo leído con éxito: {}",
                            ruta
                        ));
                        salida_final
                            .push_str(&format!("\n\n📖 [CONTENIDO DE {}]:\n{}", ruta, contenido));
                    }
                    Err(msg_error) => {
                        reportes.push(msg_error.to_string());
                    }
                }
            }
        }

        // 5. Procesar bloques HTTP: [[HTTP: url]] o [[ACTION: HTTP: ...]]
        if let Ok(re_http) = Regex::new(r"\[\[(?:ACCION:|ACTION:)?\s*HTTP:\s*(.*?)\s*\]\]") {
            for caps in re_http.captures_iter(respuesta_ia) {
                let url = caps.get(1).map_or("", |m| m.as_str().trim());

                match NexusClawPro::realizar_peticion_http(url).await {
                    Ok(contenido) => {
                        reportes.push(format!(
                            "🟢 [NEXUS_CLAW_PRO] Conexión de red exitosa a: {}",
                            url
                        ));
                        salida_final.push_str(&format!(
                            "\n\n🌐 [CONTENIDO WEB DE {}]:\n{}",
                            url, contenido
                        ));
                    }
                    Err(msg_error) => {
                        reportes.push(msg_error.to_string());
                    }
                }
            }
        }

        // 6. Procesar bloques de comando antiguos/compatibilidad: [ACCION:ejecutar_comando] "comando"
        if let Ok(re_cmd) = Regex::new(
            r#"\[ACCION:ejecutar_comando\]\s*"([^"]+)"|\[ACCION:ejecutar_comando\]\s*(.*)"#,
        ) {
            for caps in re_cmd.captures_iter(respuesta_ia) {
                let cmd = caps
                    .get(1)
                    .map(|m| m.as_str())
                    .or_else(|| caps.get(2).map(|m| m.as_str()))
                    .unwrap_or("")
                    .trim();
                if !cmd.is_empty() {
                    match NexusClawPro::ejecutar_comando(cmd).await {
                        Ok(msg_exito) => reportes.push(msg_exito),
                        Err(msg_error) => reportes.push(msg_error.to_string()),
                    }
                }
            }
        }

        // 7. Procesar bloques de comando estándar: [[ACCION: EJECUTAR: comando]]
        if let Ok(re_cmd_std) = Regex::new(r"\[\[ACCION:\s*(?:EJECUTAR:|SH:)?\s*(.*?)\s*\]\]") {
            for caps in re_cmd_std.captures_iter(respuesta_ia) {
                let cmd = caps.get(1).map_or("", |m| m.as_str().trim());
                if !cmd.is_empty() && !cmd.starts_with("INVESTIGACION_WEB") {
                    match NexusClawPro::ejecutar_comando(cmd).await {
                        Ok(msg_exito) => reportes.push(msg_exito),
                        Err(msg_error) => reportes.push(msg_error.to_string()),
                    }
                }
            }
        }

        // Si se procesaron operaciones, adjuntamos los reportes de estado al final del mensaje
        if !reportes.is_empty() {
            salida_final.push_str("\n\n🔱 [SISTEMA DE MANIFESTACIÓN NATIVA]");
            for reporte in reportes {
                salida_final.push_str(&format!("\n{}", reporte));
            }
        }

        salida_final
    }
}
