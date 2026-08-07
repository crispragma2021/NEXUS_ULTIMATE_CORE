// ============================================================================
// 🧠 CHUNKER — Sistema Digestivo Semántico (Document → Chunks Inteligentes)
// ============================================================================
// Propósito: Divide documentos/código en fragmentos semánticos con overlap
//            configurable, detectando límites naturales (funciones, clases,
//            headings, párrafos) para preservar coherencia semántica.
// ============================================================================

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChunkType {
    Code,
    Markdown,
    Text,
    Config,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub content: String,
    pub file_path: String,
    pub chunk_type: ChunkType,
    pub token_count: usize,
    pub start_line: usize,
    pub end_line: usize,
}

/// El digestor semántico: transforma archivos → fragmentos con contexto.
pub struct Chunker {
    pub max_tokens: usize,
    pub overlap_tokens: usize,
}

impl Default for Chunker {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            overlap_tokens: 64,
        }
    }
}

impl Chunker {
    pub fn new(max_tokens: usize, overlap_tokens: usize) -> Self {
        Self {
            max_tokens,
            overlap_tokens,
        }
    }

    /// Configura el máximo de tokens por chunk (64 a 2048)
    pub fn set_max_tokens(&mut self, valor: usize) {
        self.max_tokens = valor.clamp(64, 2048);
    }

    /// Configura el overlap de tokens entre chunks (8 a 512)
    pub fn set_overlap_tokens(&mut self, valor: usize) {
        self.overlap_tokens = valor.clamp(8, 512);
    }

    /// Detecta el tipo de chunk según la extensión del archivo
    pub fn detect_type(path: &str) -> ChunkType {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        match ext {
            "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "h" | "hpp" | "java" | "rb" | "kt"
            | "swift" | "scala" | "r" | "lua" | "zig" | "nim" | "ex" => ChunkType::Code,
            "md" | "mdx" => ChunkType::Markdown,
            "toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "cfg" | "conf" | "dockerfile"
            | "makefile" | "justfile" => ChunkType::Config,
            "txt" | "rst" | "adoc" | "tex" => ChunkType::Text,
            _ => ChunkType::Unknown,
        }
    }

    /// Punto de entrada principal: chunkear cualquier contenido
    pub fn chunk(&self, content: &str, file_path: &str) -> Vec<Chunk> {
        let chunk_type = Self::detect_type(file_path);
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        if total_lines == 0 {
            return vec![];
        }

        let chunks = match chunk_type {
            ChunkType::Code => self.chunk_code(&lines, file_path),
            ChunkType::Markdown => self.chunk_markdown(&lines, file_path),
            ChunkType::Text => self.chunk_text(&lines, file_path),
            ChunkType::Config => self.chunk_by_lines(&lines, file_path, self.max_tokens),
            ChunkType::Unknown => self.chunk_by_lines(&lines, file_path, self.max_tokens),
        };

        info!(
            "🧩 [CHUNKER] {} → {} chunks (tipo={:?})",
            file_path,
            chunks.len(),
            chunk_type
        );
        chunks
    }

    /// Chunking por límites de código (fn, struct, impl, enum, trait, class, def)
    fn chunk_code(&self, lines: &[&str], file_path: &str) -> Vec<Chunk> {
        let boundaries: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| {
                let t = line.trim();
                t.starts_with("fn ")
                    || t.starts_with("pub fn ")
                    || t.starts_with("async fn ")
                    || t.starts_with("pub async fn ")
                    || t.starts_with("struct ")
                    || t.starts_with("pub struct ")
                    || t.starts_with("impl")
                    || t.starts_with("pub impl")
                    || t.starts_with("enum ")
                    || t.starts_with("pub enum ")
                    || t.starts_with("trait ")
                    || t.starts_with("pub trait ")
                    || t.starts_with("mod ")
                    || t.starts_with("pub mod ")
                    || t.starts_with("#[")
                    || t.starts_with("class ")
                    || t.starts_with("def ")
                    || t.starts_with("function ")
                    || t.starts_with("interface ")
                    || t.starts_with("type ")
                    || t.starts_with("pub type ")
            })
            .map(|(i, _)| i)
            .collect();

        if boundaries.is_empty() {
            return self.chunk_by_lines(lines, file_path, self.max_tokens);
        }

        let mut chunks = Vec::new();
        for i in 0..boundaries.len() {
            let start = boundaries[i];
            let end = if i + 1 < boundaries.len() {
                boundaries[i + 1]
            } else {
                lines.len()
            };
            let chunk_lines: Vec<&str> = lines[start..end].to_vec();
            let content = chunk_lines.join("\n");

            let token_count = content.len() / 4;
            chunks.push(Chunk {
                content,
                file_path: file_path.to_string(),
                chunk_type: ChunkType::Code,
                token_count,
                start_line: start + 1,
                end_line: end,
            });
        }
        chunks
    }

    /// Chunking por headings (##, #) para markdown
    fn chunk_markdown(&self, lines: &[&str], file_path: &str) -> Vec<Chunk> {
        let headings: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| {
                let t = line.trim();
                t.starts_with("##") || t.starts_with("# ") || t.starts_with("###")
            })
            .map(|(i, _)| i)
            .collect();

        if headings.is_empty() {
            return self.chunk_by_lines(lines, file_path, self.max_tokens);
        }

        let mut chunks = Vec::new();
        for i in 0..headings.len() {
            let start = headings[i];
            let end = if i + 1 < headings.len() {
                headings[i + 1]
            } else {
                lines.len()
            };
            let chunk_lines: Vec<&str> = lines[start..end].to_vec();
            let content = chunk_lines.join("\n");
            let token_count = content.len() / 4;
            chunks.push(Chunk {
                content,
                file_path: file_path.to_string(),
                chunk_type: ChunkType::Markdown,
                token_count,
                start_line: start + 1,
                end_line: end,
            });
        }
        chunks
    }

    /// Chunking por párrafos (separados por líneas en blanco)
    fn chunk_text(&self, lines: &[&str], file_path: &str) -> Vec<Chunk> {
        let mut paragraphs: Vec<Vec<&str>> = Vec::new();
        let mut current: Vec<&str> = Vec::new();
        for line in lines {
            if line.trim().is_empty() {
                if !current.is_empty() {
                    paragraphs.push(current);
                    current = Vec::new();
                }
            } else {
                current.push(line);
            }
        }
        if !current.is_empty() {
            paragraphs.push(current);
        }

        let mut chunks = Vec::new();
        let mut line_acc: usize = 0;
        for para_lines in paragraphs {
            let content = para_lines.join("\n");
            let start = line_acc + 1;
            let end = line_acc + para_lines.len();
            let token_count = content.len() / 4;
            chunks.push(Chunk {
                content,
                file_path: file_path.to_string(),
                chunk_type: ChunkType::Text,
                token_count,
                start_line: start,
                end_line: end,
            });
            line_acc += para_lines.len() + 1;
        }
        chunks
    }

    /// Chunking por líneas con overlap (fallback universal)
    fn chunk_by_lines(&self, lines: &[&str], file_path: &str, chunk_size: usize) -> Vec<Chunk> {
        let est_lines_per_chunk = chunk_size * 4; // ~4 chars por token
        let step = est_lines_per_chunk.saturating_sub(self.overlap_tokens * 4);
        let step = if step < 1 { 1 } else { step };
        let mut chunks = Vec::new();

        let mut start = 0;
        while start < lines.len() {
            let end = (start + est_lines_per_chunk).min(lines.len());
            let content = lines[start..end].join("\n");
            let token_count = content.len() / 4;
            chunks.push(Chunk {
                content,
                file_path: file_path.to_string(),
                chunk_type: ChunkType::Unknown,
                token_count,
                start_line: start + 1,
                end_line: end,
            });
            if end >= lines.len() {
                break;
            }
            start += step;
        }
        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunker_code_detects_functions() {
        let chunker = Chunker::default();
        let code = r#"
fn hola() {
    println!("hola");
}

pub fn mundo() -> i32 {
    42
}

struct Foo {
    bar: i32
}
"#;
        let chunks = chunker.chunk(code, "test.rs");
        // Should detect fn hola, pub fn mundo, struct Foo
        assert!(
            chunks.len() >= 3,
            "Expected >=3 chunks, got {}",
            chunks.len()
        );
        assert!(chunks[0].content.contains("fn hola"));
    }

    #[test]
    fn test_chunker_markdown_by_headings() {
        let chunker = Chunker::default();
        let md = r#"# Title
content

## Section 1
section body

## Section 2
more body
"#;
        let chunks = chunker.chunk(md, "test.md");
        assert!(
            chunks.len() >= 3,
            "Expected >=3 sections, got {}",
            chunks.len()
        );
    }

    #[test]
    fn test_chunker_text_by_paragraphs() {
        let chunker = Chunker::default();
        let text = "Para1 line1\nPara1 line2\n\nPara2 line1\nPara2 line2\n\nPara3 line1";
        let chunks = chunker.chunk(text, "test.txt");
        assert!(
            chunks.len() >= 3,
            "Expected >=3 paragraphs, got {}",
            chunks.len()
        );
    }

    #[test]
    fn test_detect_type() {
        assert_eq!(Chunker::detect_type("foo.rs"), ChunkType::Code);
        assert_eq!(Chunker::detect_type("foo.py"), ChunkType::Code);
        assert_eq!(Chunker::detect_type("doc.md"), ChunkType::Markdown);
        assert_eq!(Chunker::detect_type("config.toml"), ChunkType::Config);
        assert_eq!(Chunker::detect_type("readme.txt"), ChunkType::Text);
        assert_eq!(Chunker::detect_type("binary.bin"), ChunkType::Unknown);
    }
}
