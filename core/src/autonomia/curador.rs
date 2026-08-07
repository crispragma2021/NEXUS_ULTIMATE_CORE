use crate::autonomia::detector::DetectorEstancamiento;
use crate::phantom::NexusChameleon;

pub struct CuradorAutonomo {
    pub detector: DetectorEstancamiento,
    pub chameleon: NexusChameleon,
    pub ultima_leccion: Option<String>,
}

impl CuradorAutonomo {
    pub fn new(chameleon: NexusChameleon) -> Self {
        Self {
            detector: DetectorEstancamiento::new(3, 45), // 3 intentos o 45 segundos
            chameleon,
            ultima_leccion: None,
        }
    }

    pub async fn intentar_curacion(&mut self, error: &str) -> anyhow::Result<()> {
        if self.detector.esta_atascado() {
            println!("🚨 [NEXUS] ESTANCAMIENTO DETECTADO. Iniciando DERIVA RECURSIVA...");

            // 2. BUSCAR EN INTERNET (Llamada interna que se conecta al MCP/Search)
            let query = format!(
                "Detección y solución de error {} en linux debian gnome wayland",
                error
            );
            match self.chameleon.diagnosticar_web(&query).await {
                Ok(solucion) => {
                    println!("🧬 [NEXUS] SOLUCIÓN ENCONTRADA: {}", solucion);
                    self.ultima_leccion = Some(solucion);
                    // 3. REGISTRAR EN EL HIPOCAMPO (Ya grabado en victorias.md por el agente)
                    self.detector.resetear();
                }
                Err(e) => println!("⚠️ [NEXUS] Inanición de Búsqueda: {}", e),
            }
        }
        Ok(())
    }
}
