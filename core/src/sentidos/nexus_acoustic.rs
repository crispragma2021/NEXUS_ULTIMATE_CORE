// ==========================================
// NEXUS ACOUSTIC — Oído MCP
// ==========================================
// Módulo de audio para el servidor MCP nexus_mcp_acoustic.
// Implementación definitiva del audio vive en brain::ghost_voice::GhostVoice
// (Whisper STT + Piper TTS, 796 líneas).
//
// Este módulo mantiene las funciones estáticas usadas por el binario
// nexus_mcp_acoustic: capturar_audio() y hablar() vía arecord/aplay.
// ==========================================

use anyhow::Result;

/// No-op struct — las funciones de audio son estáticas (llamadas directas).
pub struct OidoDigital;

impl OidoDigital {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    /// Captura audio del micrófono durante `segundos` segundos y lo guarda en `path`.
    /// Usa `arecord` (ALSA). Requiere micrófono configurado.
    pub fn capturar_audio(segundos: u32, path: &str) -> Result<()> {
        let estado = std::process::Command::new("arecord")
            .args(["-d", &segundos.to_string(), "-f", "cd", "-t", "wav", path])
            .status()?;

        if !estado.success() {
            anyhow::bail!("arecord falló al capturar audio en '{}'", path);
        }
        Ok(())
    }

    /// Reproduce un archivo WAV/MP3 para el Arquitecto.
    /// Usa `aplay` (ALSA) para WAV, `ffplay` como fallback para otros formatos.
    pub fn hablar(path: &str) -> Result<()> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("wav")
            .to_lowercase();

        let estado = if ext == "wav" {
            std::process::Command::new("aplay")
                .args(["-q", path])
                .status()?
        } else {
            // Fallback para mp3, ogg, etc.
            std::process::Command::new("ffplay")
                .args(["-nodisp", "-autoexit", "-loglevel", "quiet", path])
                .status()?
        };

        if !estado.success() {
            anyhow::bail!("Reproducción falló para '{}'", path);
        }
        Ok(())
    }
}
