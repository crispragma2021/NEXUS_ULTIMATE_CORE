use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};

/// 💎 SINAPSIS GEMINI LIVE (WebSockets OMEGA)
/// Implementación de la API Multimodal Live para latencia ultra-baja.
pub struct GeminiLiveAPI {
    api_key: String,
    model: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LiveSetup {
    pub setup: SetupConfig,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetupConfig {
    pub model: String,
    pub generation_config: GenerationConfig,
    pub speech_config: SpeechConfig,
    pub system_instruction: Option<SystemInstruction>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    pub response_modalities: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpeechConfig {
    pub voice_config: VoiceConfig,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VoiceConfig {
    pub prebuilt_voice_config: PrebuiltVoiceConfig,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrebuiltVoiceConfig {
    pub voice_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemInstruction {
    pub parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Part {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeInput {
    pub realtime_input: MediaChunks,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaChunks {
    pub chunks: Vec<Blob>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Blob {
    pub mime_type: String,
    pub data: String, // Base64 PCM
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeOutput {
    pub server_content: Option<ServerContent>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServerContent {
    pub model_turn: Option<ModelTurn>,
}

#[derive(Deserialize, Debug)]
pub struct ModelTurn {
    pub parts: Vec<OutputPart>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutputPart {
    pub text: Option<String>,
    pub inline_data: Option<InlineData>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InlineData {
    pub mime_type: String,
    pub data: String, // Base64 PCM 24kHz
}

impl GeminiLiveAPI {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "models/gemini-2.0-flash".to_string(),
        }
    }

    pub async fn start_session(
        &self,
        system_prompt: String,
        mut pcm_rx: mpsc::UnboundedReceiver<Vec<i16>>,
        audio_engine: Arc<crate::phantom::AudioEngine>,
    ) -> Result<()> {
        let url = format!(
            "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1alpha.GenerativeService/BiDiGenerateContent?key={}",
            self.api_key
        );

        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| anyhow!("Fallo al conectar WebSocket de Gemini Live: {}", e))?;

        let (mut write, mut read) = ws_stream.split();

        // 1. Enviar Mensaje de SETUP
        let setup_msg = LiveSetup {
            setup: SetupConfig {
                model: self.model.clone(),
                generation_config: GenerationConfig {
                    response_modalities: vec!["AUDIO".to_string(), "TEXT".to_string()],
                },
                speech_config: SpeechConfig {
                    voice_config: VoiceConfig {
                        prebuilt_voice_config: PrebuiltVoiceConfig {
                            voice_name: "Ursa".to_string(),
                        },
                    },
                },
                system_instruction: Some(SystemInstruction {
                    parts: vec![Part {
                        text: system_prompt,
                    }],
                }),
            },
        };

        let setup_json = serde_json::to_string(&setup_msg)?;
        write.send(Message::Text(setup_json)).await?;
        info!("💎 [GEMINI LIVE] Sesión iniciada y configurada con Ursa Voice.");

        // 2. Bucle de Recepción (Gemini -> NEXUS)
        let engine_clone = audio_engine.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(output) = serde_json::from_str::<RealtimeOutput>(&text) {
                            if let Some(content) = output.server_content {
                                if let Some(turn) = content.model_turn {
                                    for part in turn.parts {
                                        if let Some(text) = part.text {
                                            info!("🟢 [GEMINI LIVE] Texto: {}", text);
                                        }
                                        if let Some(audio_blob) = part.inline_data {
                                            if let Ok(pcm_bytes) =
                                                general_purpose::STANDARD.decode(audio_blob.data)
                                            {
                                                // Convertir bytes (L16) a i16
                                                let pcm_vec: Vec<i16> = pcm_bytes
                                                    .chunks_exact(2)
                                                    .map(|a| i16::from_le_bytes([a[0], a[1]]))
                                                    .collect();
                                                engine_clone.play_pcm_24khz(pcm_vec);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("🔥 [GEMINI LIVE] Error en WebSocket: {}", e);
                        break;
                    }
                }
            }
            warn!("🛑 [GEMINI LIVE] WebSocket cerrado.");
        });

        // 3. Bucle de Envío (NEXUS -> Gemini)
        while let Some(pcm_chunk) = pcm_rx.recv().await {
            let mut bytes = Vec::with_capacity(pcm_chunk.len() * 2);
            for sample in pcm_chunk {
                bytes.extend_from_slice(&sample.to_le_bytes());
            }

            let input_msg = RealtimeInput {
                realtime_input: MediaChunks {
                    chunks: vec![Blob {
                        mime_type: "audio/pcm;rate=16000".to_string(),
                        data: general_purpose::STANDARD.encode(bytes),
                    }],
                },
            };

            let input_json = serde_json::to_string(&input_msg)?;
            if let Err(e) = write.send(Message::Text(input_json)).await {
                error!("🔥 [GEMINI LIVE] Error al enviar audio: {}", e);
                break;
            }
        }

        Ok(())
    }
}
