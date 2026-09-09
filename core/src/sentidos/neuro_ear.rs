use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

/// 👂 NEXUS EAR: El Oído Vigilante
/// Captura de audio directa mediante el subsistema de sonido (ALSA/Pulse/Pipewire).
pub struct NexusEar {
    host: cpal::Host,
}

impl Default for NexusEar {
    fn default() -> Self {
        Self::new()
    }
}

impl NexusEar {
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    /// Inicializar el sentido auditivo (despertar)
    pub fn despertar_oido(
        _reflex_tx: tokio::sync::mpsc::Sender<crate::brain::reflex_arc::ReflexSignal>,
    ) -> Result<Self> {
        Ok(Self::new())
    }

    /// Capturar una ráfaga de sonido del búnker
    pub fn capturar_pulso(&self, duracion_segs: u32, path: &str) -> Result<()> {
        info!(
            "👂 [SNC] Iniciando captura auditiva ({}s) en {}",
            duracion_segs, path
        );

        let device = self
            .host
            .default_input_device()
            .context("Fallo al localizar el micrófono del búnker")?;

        let config = device
            .default_input_config()
            .context("Error al obtener configuración de audio nativa")?;

        let spec = WavSpec {
            channels: config.channels(),
            sample_rate: config.sample_rate().0,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let writer = Arc::new(Mutex::new(WavWriter::create(path, spec)?));
        let writer_clone = writer.clone();

        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                let mut writer = writer_clone.lock().unwrap();
                for &sample in data {
                    let sample_int = (sample * 32767.0) as i16;
                    writer.write_sample(sample_int).ok();
                }
            },
            |err| warn!("⚠️ [SNC] Error en flujo de audio: {}", err),
            None,
        )?;

        stream.play()?;
        std::thread::sleep(std::time::Duration::from_secs(duracion_segs as u64));
        drop(stream);

        info!("✅ [SNC] Captura auditiva guardada.");
        Ok(())
    }
}
