use std::process::Command;
use std::time::Duration;
use tracing::{error, info};

/// Tacto Digital: Efectores de interacción física con el sistema.
/// Controla ratón y teclado mediante xdotool, con precisión humana.
pub struct TactoDigital;

impl TactoDigital {
    /// Tocar una coordenada específica (Click)
    pub fn click(x: i32, y: i32) -> anyhow::Result<()> {
        info!("🦾 [TACTO] Presionando punto: ({}, {})", x, y);
        let status = Command::new("xdotool")
            .arg("mousemove")
            .arg(x.to_string())
            .arg(y.to_string())
            .arg("click")
            .arg("1")
            .status()?;

        if !status.success() {
            error!("⚠️ [TACTO] Falla en gesto táctil.");
            return Err(anyhow::anyhow!("Falla en xdotool"));
        }
        Ok(())
    }

    /// Escribir texto en la aplicación activa (Digitación directa)
    pub fn escribir(texto: &str) -> anyhow::Result<()> {
        info!("🦾 [TACTO] Digitando ráfaga de datos...");
        let _ = Command::new("xdotool").arg("type").arg(texto).status()?;
        Ok(())
    }

    /// Escribir con simulación de escritura humana (WPM + delays por caracter)
    /// Fusionado desde legacy mano_digital — timing humano anti-detección
    pub async fn escribir_natural(texto: &str, wpm: u32) -> anyhow::Result<()> {
        info!("🦾 [TACTO] Escribiendo a {} WPM con timing humano...", wpm);
        for c in texto.chars() {
            // ~5 chars por palabra → ms por char = 60000 / (wpm * 5)
            let ms = 60000 / (wpm * 5).max(1);
            tokio::time::sleep(Duration::from_millis(ms as u64)).await;

            // Enviar cada carácter individualmente vía xdotool
            let _ = Command::new("xdotool")
                .arg("type")
                .arg(c.to_string())
                .status()?;
        }
        Ok(())
    }

    /// Mover el ratón a una coordenada sin hacer click
    pub fn mover(x: i32, y: i32) -> anyhow::Result<()> {
        info!("🦾 [TACTO] Moviendo a: ({}, {})", x, y);
        let status = Command::new("xdotool")
            .arg("mousemove")
            .arg(x.to_string())
            .arg(y.to_string())
            .status()?;

        if !status.success() {
            error!("⚠️ [TACTO] Falla en movimiento táctil.");
            return Err(anyhow::anyhow!("Falla en xdotool"));
        }
        Ok(())
    }

    /// Obtener la posición actual del cursor
    pub fn posicion() -> anyhow::Result<(i32, i32)> {
        let output = Command::new("xdotool").arg("getmouselocation").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = stdout.split_whitespace().collect();
        if parts.len() >= 2 {
            let x = parts[0]
                .trim_start_matches("x:")
                .parse::<i32>()
                .map_err(|e| anyhow::anyhow!("Error parsing x: {}", e))?;
            let y = parts[1]
                .trim_start_matches("y:")
                .parse::<i32>()
                .map_err(|e| anyhow::anyhow!("Error parsing y: {}", e))?;
            Ok((x, y))
        } else {
            Err(anyhow::anyhow!("No se pudo obtener la posición del cursor"))
        }
    }
}
