// ============================================================================
// 🧠 INGESTA — Pipeline de Digestión Automática de Conocimiento
// ============================================================================
// Propósito: Conecta el FS watcher (MonitorCognitivo) con el Chunker,
//            el generador de embeddings y LanceDB para ingesta continua.
//            Escanea el codebase completo al inicio y luego procesa cambios
//            incrementales. Tabla separada "codebase_knowledge" en LanceDB.
// ============================================================================

use crate::cerebro::organos::chunker::Chunker;
use crate::memoria::memoria_semantica::MemoriaSemantica;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Puente entre el Chunker y la Memoria Semántica
pub struct IngestaPipeline {
    pub chunker: Chunker,
    pub memoria: Arc<MemoriaSemantica>,
    pub table_name: String,
    /// IDs ya indexados para evitar duplicados
    indexed_ids: Mutex<std::collections::HashSet<i64>>,
}

impl IngestaPipeline {
    pub fn new(memoria: Arc<MemoriaSemantica>, table_name: &str) -> Self {
        Self {
            chunker: Chunker::default(),
            memoria,
            table_name: table_name.to_string(),
            indexed_ids: Mutex::new(std::collections::HashSet::new()),
        }
    }

    /// Configura chunker con parámetros específicos
    pub fn with_chunker(mut self, chunker: Chunker) -> Self {
        self.chunker = chunker;
        self
    }

    // ─── Ingesta de archivo individual ─────────────────────────────────────

    /// Ingiere un archivo completo: lee → chunkea → embeddea → indexa en LanceDB
    pub async fn ingerir_archivo(&self, file_path: &str) -> Result<usize> {
        let path = Path::new(file_path);
        if !path.exists() || !path.is_file() {
            warn!("⚠️ [INGESTA] Archivo no encontrado: {}", file_path);
            return Ok(0);
        }

        let content = tokio::fs::read_to_string(path).await?;
        let chunks = self.chunker.chunk(&content, file_path);
        if chunks.is_empty() {
            return Ok(0);
        }

        let mut indexed = 0usize;
        for chunk in &chunks {
            // Generar ID único basado en file_path + start_line
            let chunk_id = IngestaPipeline::generar_id_unico(file_path, chunk.start_line);
            let ids = self.indexed_ids.lock().await;
            if ids.contains(&chunk_id) {
                continue; // Ya indexado
            }
            drop(ids);

            // Generar embedding para este chunk
            let embedding = self.memoria.generar_embedding(&chunk.content).await?;

            // Construir esencia con metadata
            let esencia = format!(
                "📄 {} (líneas {}-{}) · {}",
                file_path, chunk.start_line, chunk.end_line, chunk.content
            );
            let esencia_truncada = if esencia.len() > 5000 {
                // Recorte seguro por límite de carácter UTF-8 (nunca partir un char multibyte)
                let mut boundary = 5000;
                while !esencia.is_char_boundary(boundary) {
                    boundary -= 1;
                }
                &esencia[..boundary]
            } else {
                &esencia
            };

            // Indexar en LanceDB en la tabla de conocimiento
            self.memoria
                .indexar_impresion_con_tabla(
                    chunk_id,
                    esencia_truncada,
                    embedding,
                    &self.table_name,
                )
                .await?;

            let mut ids = self.indexed_ids.lock().await;
            ids.insert(chunk_id);

            indexed += 1;
        }

        info!(
            "📥 [INGESTA] {} → {} chunks indexados en '{}'",
            file_path, indexed, self.table_name
        );
        Ok(indexed)
    }

    // ─── Escaneo masivo del codebase ───────────────────────────────────────

    /// Escanea recursivamente un directorio e ingiere todos los archivos
    /// que coinciden con extensiones de código/documentación.
    pub async fn escanear_codebase(&self, root_dir: &str) -> Result<IngestaReport> {
        let mut report = IngestaReport::new(root_dir);
        let start = std::time::Instant::now();

        let mut dir_entries = Vec::new();
        let mut stack = vec![std::path::PathBuf::from(root_dir)];

        while let Some(dir) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        // Excluir directorios de build/artefactos
                        if !dir_name.starts_with('.')
                            && dir_name != "target"
                            && dir_name != "node_modules"
                            && dir_name != ".cargo"
                            && dir_name != ".git"
                        {
                            stack.push(path);
                        }
                    } else if Self::es_archivo_indexable(&path) {
                        dir_entries.push(path);
                    }
                }
            }
        }

        info!(
            "📂 [INGESTA] Escaneando {} archivos en '{}'...",
            dir_entries.len(),
            root_dir
        );

        for path in &dir_entries {
            let path_str = path.to_string_lossy();
            match self.ingerir_archivo(&path_str).await {
                Ok(n) => {
                    report.total_files += 1;
                    report.total_chunks += n;
                    if n > 0 {
                        report.files_indexed.push(path_str.to_string());
                    }
                }
                Err(e) => {
                    warn!("⚠️ [INGESTA] Error ingiriendo {}: {}", path_str, e);
                    report.errors.push((path_str.to_string(), e.to_string()));
                }
            }
        }

        report.duration_ms = start.elapsed().as_millis() as u64;
        info!(
            "📊 [INGESTA] COMPLETO: {} archivos, {} chunks, {} errores en {}ms",
            report.total_files,
            report.total_chunks,
            report.errors.len(),
            report.duration_ms
        );

        Ok(report)
    }

    // ─── Búsqueda sobre conocimiento indexado ──────────────────────────────

    /// Busca chunks similares en la tabla de conocimiento (no en Ocean)
    pub async fn buscar_conocimiento(
        &self,
        query: &str,
        limite: usize,
    ) -> Result<Vec<(String, f32)>> {
        let embedding = self.memoria.generar_embedding(query).await?;
        let resultados = self
            .memoria
            .buscar_similares_en_tabla(&embedding, limite, &self.table_name)
            .await?;

        // Convertir IDs a contenido (recuperamos de los vectores de LanceDB)
        let mut output = Vec::new();
        for (id, score) in &resultados {
            output.push((format!("[chunk_id:{}]", id), *score));
        }
        Ok(output)
    }

    // ─── Helpers ───────────────────────────────────────────────────────────

    fn generar_id_unico(file_path: &str, start_line: usize) -> i64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        file_path.hash(&mut hasher);
        start_line.hash(&mut hasher);
        hasher.finish() as i64
    }

    fn es_archivo_indexable(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext,
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "go"
                    | "c"
                    | "cpp"
                    | "toml"
                    | "yaml"
                    | "yml"
                    | "json"
                    | "md"
                    | "mdx"
                    | "txt"
                    | "sh"
                    | "rb"
                    | "java"
                    | "kt"
                    | "swift"
                    | "css"
                    | "scss"
                    | "html"
                    | "lua"
                    | "zig"
                    | "sql"
                    | "r"
                    | "ex"
                    | "lock" // keep for documentation value
            )
        } else {
            false
        }
    }
}

/// Reporte de una ingesta masiva
#[derive(Debug, Clone)]
pub struct IngestaReport {
    pub root_dir: String,
    pub total_files: usize,
    pub total_chunks: usize,
    pub files_indexed: Vec<String>,
    pub errors: Vec<(String, String)>,
    pub duration_ms: u64,
}

impl IngestaReport {
    pub fn new(root_dir: &str) -> Self {
        Self {
            root_dir: root_dir.to_string(),
            total_files: 0,
            total_chunks: 0,
            files_indexed: Vec::new(),
            errors: Vec::new(),
            duration_ms: 0,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "📊 INGESTA '{}': {} archivos, {} chunks, {} errores en {}ms",
            self.root_dir,
            self.total_files,
            self.total_chunks,
            self.errors.len(),
            self.duration_ms
        )
    }
}
