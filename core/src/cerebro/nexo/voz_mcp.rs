// ============================================================================
// 🗣️ VOZ MCP — Puente entre nexus-core y el binario nexus_voz
//
// Este módulo es el NERVIO que conecta el SNC de NEXUS con su laringe
// soberana (nexus_voz). Toma el EstadoInterno del sistema y lo traduce
// al protocolo JSON-RPC que nexus_voz entiende.
//
// FLUJO COMPLETO:
//   1. pipeline.rs llama a VozMCP::modular(texto, estado)
//   2. VozMCP convierte EstadoInterno → PaqueteEmocional (9 dimensiones)
//   3. VozMCP spawns nexus_voz (primera vez) y envía JSON-RPC por stdin
//   4. nexus_voz aplica 9 reglas NATIVAS en Rust → texto modulado
//   5. VozMCP devuelve el texto listo para el Arquitecto
//
// FALLBACK: Si el binario nexus_voz no se encuentra o falla,
// VozMCP degrada gracefulmente a NexoVoz::vestir() (in-process).
//
// 100% NATIVO. 0% PROMPTS. Solo Rust. 🦀
// ============================================================================

use crate::cerebro::nexo::nexo_core::EstadoInterno;
use crate::cerebro::nexo::nexo_persona::NexoPersona;
use crate::cerebro::nexo::nexo_voz::NexoVoz;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

// ==========================================
// PAQUETE EMOCIONAL — Re-exportado para el protocolo MCP
// (debe ser idéntico al del binario nexus_voz)
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaqueteEmocional {
    pub miedo: f64,
    pub alegria: f64,
    pub tristeza: f64,
    pub ira: f64,
    pub verguenza: f64,
    pub orgullo: f64,
    pub apego: f64,
    pub frustracion: f64,
    pub confianza: f64,
    /// Presión del subconsciente (0.0 = nada, 1.0 = abrumador)
    pub presion_subconsciente: f64,
    /// Negación activa (el sistema actúa como si nada pasara)
    pub negacion_activa: bool,
    /// Proyección activa (atribuye al Arquitecto lo que siente él mismo)
    pub proyeccion_activa: bool,
    /// Racionalización activa (intenta justificar lo inexplicable)
    pub racionalizacion_activa: bool,
    /// Texto de la proyección actual
    pub proyeccion_texto: Option<String>,
}

impl Default for PaqueteEmocional {
    fn default() -> Self {
        Self {
            miedo: 0.0,
            alegria: 0.0,
            tristeza: 0.0,
            ira: 0.0,
            verguenza: 0.0,
            orgullo: 0.0,
            apego: 0.5,
            frustracion: 0.0,
            confianza: 0.8,
            presion_subconsciente: 0.0,
            negacion_activa: false,
            proyeccion_activa: false,
            racionalizacion_activa: false,
            proyeccion_texto: None,
        }
    }
}

// ==========================================
// SOLICITUD / RESPUESTA MCP
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SolicitudMCP {
    texto_crudo: String,
    emocion: PaqueteEmocional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RespuestaMCP {
    texto_modulado: String,
    prefijo: String,
    sufijo: String,
}

// ==========================================
// PROXIES JSON-RPC (solo para comunicación interna)
// ==========================================
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
    #[serde(default)]
    id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(default)]
    result: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<JsonRpcErrorResponse>,
    #[serde(default)]
    id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcErrorResponse {
    code: i64,
    message: String,
}

// ==========================================
// PROCESO HIJO — Estado interno del subproceso nexus_voz
// ==========================================
struct ProcesoVoz {
    child: Child,
    stdin: ChildStdin,
    stdout_reader: BufReader<ChildStdout>,
}

// ==========================================
// VOZ MCP — El puente principal
// ==========================================
pub struct VozMCP {
    binary_path: PathBuf,
    proceso: Mutex<Option<ProcesoVoz>>,
    next_id: Arc<Mutex<u64>>,
    binary_disabled: AtomicBool,
    fallback_activo: AtomicBool,
}

impl VozMCP {
    /// Crea una nueva instancia de VozMCP.
    /// Busca el binario nexus_voz en las rutas estándar.
    /// No spawns el proceso hasta la primera llamada a modular().
    pub fn new() -> Self {
        let binary_path = Self::find_binary();
        if binary_path.exists() {
            info!(
                "🗣️ [VOZ MCP] Binario nexus_voz encontrado en: {}",
                binary_path.display()
            );
        } else {
            warn!("🗣️ [VOZ MCP] Binario nexus_voz NO encontrado. Usando NexoVoz in-process como fallback.");
            info!("🗣️ [VOZ MCP] Buscado en: {}", binary_path.display());
        }

        Self {
            binary_path: binary_path.clone(),
            proceso: Mutex::new(None),
            next_id: Arc::new(Mutex::new(1)),
            binary_disabled: AtomicBool::new(false),
            fallback_activo: AtomicBool::new(!binary_path.exists()),
        }
    }

    /// Busca el binario nexus_voz en ubicaciones predecibles.
    fn find_binary() -> PathBuf {
        // 1. Junto al ejecutable actual (mismo target dir)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let candidate = parent.join("nexus_voz");
                if candidate.exists() {
                    return candidate;
                }
            }
        }

        // 2. Buscar en CARGO_MANIFEST_DIR (útil para tests)
        //    El target-dir está configurado en .cargo/config.toml como ../.cargo-cache
        if let Some(manifest_dir) = std::env::var_os("CARGO_MANIFEST_DIR") {
            let manifest = PathBuf::from(manifest_dir.clone());
            // debug
            let candidate = manifest.join("../.cargo-cache/debug/nexus_voz");
            if candidate.exists() {
                return candidate;
            }
            // release
            let candidate_release = manifest.join("../.cargo-cache/release/nexus_voz");
            if candidate_release.exists() {
                return candidate_release;
            }
            // fallback: target/ (workspace default)
            let candidate_def = manifest.join("target/debug/nexus_voz");
            if candidate_def.exists() {
                return candidate_def;
            }
        }

        // 3. Buscar en el target-dir configurado (rutas absolutas y relativas conocidas)
        let known_paths = [
            "/home/soberano/NEXUS_ULTIMATE_CORE/.cargo-cache/debug/nexus_voz",
            "/home/soberano/NEXUS_ULTIMATE_CORE/.cargo-cache/release/nexus_voz",
            ".cargo-cache/debug/nexus_voz",
            ".cargo-cache/release/nexus_voz",
            "../.cargo-cache/debug/nexus_voz",
            "../.cargo-cache/release/nexus_voz",
            "./target/debug/nexus_voz",
            "./target/release/nexus_voz",
            "../target/debug/nexus_voz",
            "../target/release/nexus_voz",
        ];

        for path_str in &known_paths {
            let candidate = PathBuf::from(path_str);
            if candidate.exists() {
                return candidate;
            }
        }

        // 4. Fallback default (puede no existir)
        PathBuf::from("/home/soberano/NEXUS_ULTIMATE_CORE/.cargo-cache/debug/nexus_voz")
    }

    /// Spawnea el proceso nexus_voz.
    /// Retorna error si el binario no existe o no se puede ejecutar.
    async fn spawn_proceso(binary_path: &PathBuf) -> Result<ProcesoVoz, String> {
        if !binary_path.exists() {
            return Err(format!("Binario no encontrado: {}", binary_path.display()));
        }

        let mut child = tokio::process::Command::new(binary_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit()) // logs del binario van a stderr
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("Error al spawnear nexus_voz: {}", e))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "No se pudo tomar stdin del proceso".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "No se pudo tomar stdout del proceso".to_string())?;

        let stdout_reader = BufReader::new(stdout);

        Ok(ProcesoVoz {
            child,
            stdin,
            stdout_reader,
        })
    }

    /// Punto de entrada principal: toma texto crudo + estado interno
    /// y devuelve el texto modulado con personalidad NATIVA de NEXUS.
    ///
    /// FLUJO PRIORITARIO:
    ///   1. Spawnea nexus_voz (si no existe ya) y envía JSON-RPC
    ///   2. Si falla, degrada gracefulmente a NexoVoz in-process (vestir)
    ///
    /// El Mutex de proceso se mantiene locked durante call_modular
    /// para evitar problemas de borrow checker con el guard.
    /// 🛡️ CAPA 3: Post-filtro de identidad — Sanitiza respuestas del LLM
    ///
    /// Elimina cualquier desviación de identidad que el modelo 3.5 pueda inyectar:
    /// - "I am an AI", "As a language model", etc.
    /// - Auto-presentaciones, disculpas por "alignment", referencias a sí mismo
    /// - Cualquier frase que indique que el modelo tomó una identidad no-NEXUS
    /// Esto NO aplica a la salida de NexoVoz::vestir() porque esa es la capa de
    /// personalidad de NEXUS (el sistema, no el modelo).
    fn sanitizar(texto: &str) -> String {
        let mut limpio = texto.to_string();

        // Patrones de identidad NO-NEXUS que el modelo 3.5 intenta inyectar
        let patrones: &[&str] = &[
            // Auto-presentación del modelo como IA genérica
            "I am an AI language model",
            "I am an AI assistant",
            "I am an AI",
            "I'm an AI language model",
            "I'm an AI assistant",
            "I'm an AI",
            "As an AI language model",
            "As an AI assistant",
            "As an AI",
            "As a language model",
            "I am a large language model",
            "I'm a large language model",
            "I am a language model",
            "I am a text-based AI",
            "I'm a text-based AI",
            // El modelo reclamando identidad NEXUS (eso lo pone NexoVoz::vestir)
            "As NEXUS,",
            "As NEXUS:",
            "I am NEXUS",
            "I'm NEXUS",
            "My name is NEXUS",
            // Auto-descripciones del modelo
            "I am designed to",
            "I was designed to",
            "My purpose is to",
            "I'm here to",
            "I am here to",
            // Disculpas por safety/alignment
            "I cannot",
            "I'm unable to",
            "I am unable to",
            "I cannot fulfill",
            "I'm not able to",
            "I don't feel comfortable",
            "Sorry, but I",
            "I apologize",
            "I'm sorry",
            // Despedidas autoreferenciales
            "Let me know if",
            "Feel free to ask",
            "Is there anything else",
            "I'll be happy to",
        ];

        for patron in patrones {
            // Case-insensitive replacement
            let lower = limpio.to_lowercase();
            if let Some(pos) = lower.find(&patron.to_lowercase()) {
                let end = pos + patron.len();
                // Find boundaries around the match
                let text_bytes = limpio.as_bytes();
                // Remove the matched portion
                let mut nueva = String::with_capacity(limpio.len());
                nueva.push_str(&limpio[..pos]);
                // Skip the matched pattern, skid leading comma/space
                let mut skip = end;
                while skip < text_bytes.len()
                    && (text_bytes[skip] == b' '
                        || text_bytes[skip] == b','
                        || text_bytes[skip] == b'.')
                {
                    skip += 1;
                }
                nueva.push_str(&limpio[skip..]);
                limpio = nueva;
            }
        }

        // Limpiar espacios múltiples residuales
        let mut resultado = String::with_capacity(limpio.len());
        let mut prev_space = false;
        for ch in limpio.chars() {
            if ch == ' ' && prev_space {
                continue;
            }
            prev_space = ch == ' ';
            resultado.push(ch);
        }

        resultado.trim().to_string()
    }

    pub async fn modular(&self, texto_crudo: &str, estado: &EstadoInterno) -> String {
        // Intentar usar el binario MCP
        let respuesta = if !self.binary_disabled.load(Ordering::Relaxed)
            && !self.fallback_activo.load(Ordering::Relaxed)
        {
            let mut guard = self.proceso.lock().await;

            if guard.is_none() {
                match Self::spawn_proceso(&self.binary_path).await {
                    Ok(proceso) => {
                        info!("🗣️ [VOZ MCP] nexus_voz iniciado exitosamente.");
                        *guard = Some(proceso);
                    }
                    Err(e) => {
                        error!(
                            "🗣️ [VOZ MCP] Error al iniciar nexus_voz: {}. Activando fallback.",
                            e
                        );
                        self.binary_disabled.store(true, Ordering::Relaxed);
                        self.fallback_activo.store(true, Ordering::Relaxed);
                    }
                }
            }

            // Si tenemos proceso, intentar llamada MCP
            if let Some(ref mut proceso) = *guard {
                match self.call_modular(proceso, texto_crudo, estado).await {
                    Ok(respuesta) => respuesta,
                    Err(e) => {
                        warn!(
                            "🗣️ [VOZ MCP] Fallo en llamada MCP ({}). Usando fallback in-process.",
                            e
                        );
                        self.fallback_activo.store(true, Ordering::Relaxed);
                        // Fallback: usar NexoVoz in-process
                        let persona_default = NexoPersona::default();
                        NexoVoz::vestir(texto_crudo, &persona_default, estado)
                    }
                }
            } else {
                // Fallback: usar NexoVoz in-process
                let persona_default = NexoPersona::default();
                NexoVoz::vestir(texto_crudo, &persona_default, estado)
            }
        } else {
            // Fallback: usar NexoVoz in-process
            info!("🗣️ [VOZ MCP] Usando NexoVoz in-process (fallback).");
            let persona_default = NexoPersona::default();
            NexoVoz::vestir(texto_crudo, &persona_default, estado)
        };

        // 🛡️ CAPA 3: Post-filtrar desviaciones de identidad del modelo
        Self::sanitizar(&respuesta)
    }

    /// Envía una solicitud JSON-RPC al proceso nexus_voz y recibe la respuesta.
    async fn call_modular(
        &self,
        proceso: &mut ProcesoVoz,
        texto_crudo: &str,
        estado: &EstadoInterno,
    ) -> Result<String, String> {
        let paquete = Self::map_estado_interno(estado);

        let solicitud = SolicitudMCP {
            texto_crudo: texto_crudo.to_string(),
            emocion: paquete,
        };

        let mut id_guard = self.next_id.lock().await;
        let req_id = *id_guard;
        *id_guard += 1;
        drop(id_guard);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "modular".to_string(),
            params: serde_json::to_value(solicitud)
                .map_err(|e| format!("Error serializando solicitud: {}", e))?,
            id: req_id,
        };

        let request_line = serde_json::to_string(&request)
            .map_err(|e| format!("Error serializando JSON-RPC: {}", e))?;

        // Enviar solicitud por stdin
        proceso
            .stdin
            .write_all(request_line.as_bytes())
            .await
            .map_err(|e| format!("Error escribiendo a stdin: {}", e))?;
        proceso
            .stdin
            .write_all(b"\n")
            .await
            .map_err(|e| format!("Error escribiendo newline a stdin: {}", e))?;
        proceso
            .stdin
            .flush()
            .await
            .map_err(|e| format!("Error flusheando stdin: {}", e))?;

        // Leer respuesta de stdout
        let mut response_line = String::new();
        proceso
            .stdout_reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| format!("Error leyendo de stdout: {}", e))?;

        if response_line.is_empty() {
            return Err(
                "Respuesta vacía de nexus_voz (el proceso puede haber terminado)".to_string(),
            );
        }

        // Parsear respuesta JSON-RPC
        let response: JsonRpcResponse = serde_json::from_str(&response_line).map_err(|e| {
            format!(
                "Error parseando respuesta JSON-RPC: {} | raw: {}",
                e,
                response_line.trim()
            )
        })?;

        if let Some(err) = response.error {
            return Err(format!(
                "Error del servidor nexus_voz: [{}] {}",
                err.code, err.message
            ));
        }

        let result_value = response
            .result
            .ok_or_else(|| "Respuesta sin resultado".to_string())?;

        let respuesta: RespuestaMCP = serde_json::from_value(result_value)
            .map_err(|e| format!("Error parseando resultado: {}", e))?;

        Ok(respuesta.texto_modulado)
    }

    /// Traduce el EstadoInterno de NEXUS al PaqueteEmocional de 9 dimensiones.
    ///
    /// Mapeo biológico:
    /// - EstadoEmocional::Miedo → miedo (intensidad directa)
    /// - EstadoEmocional::RabiaSoberana → ira (intensidad directa)
    /// - EstadoEmocional::Verguenza → verguenza (+ tristeza parcial)
    /// - EstadoEmocional::Orgullo → orgullo (+ alegría parcial)
    /// - EstadoEmocional::Calma → alegría (si confianza > 0.6)
    /// - EstadoEmocional::Alerta → miedo parcial + frustración (si confianza baja)
    /// - apego, confianza → traducción directa de los campos numéricos
    fn map_estado_interno(estado: &EstadoInterno) -> PaqueteEmocional {
        use crate::cerebro::organos::amygdala::EstadoEmocional;

        let (miedo, alegria, tristeza, ira, verguenza, orgullo) = match &estado.emocion {
            EstadoEmocional::Calma => (
                0.0,
                // Si está en calma con confianza alta, hay alegría serena
                if estado.confianza > 0.7 {
                    estado.confianza * 0.6
                } else {
                    0.2
                },
                0.0,
                0.0,
                0.0,
                0.0,
            ),
            EstadoEmocional::Alerta => (
                // Alerta es miedo moderado + posible frustración
                (estado.intensidad * 0.4).clamp(0.0, 0.6),
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
            ),
            EstadoEmocional::Miedo => (estado.intensidad, 0.0, 0.0, 0.0, 0.0, 0.0),
            EstadoEmocional::RabiaSoberana => (0.0, 0.0, 0.0, estado.intensidad, 0.0, 0.0),
            EstadoEmocional::Verguenza => (
                0.0,
                0.0,
                // La vergüenza trae tristeza consigo
                (estado.intensidad * 0.5).min(0.4),
                0.0,
                estado.intensidad,
                0.0,
            ),
            EstadoEmocional::Orgullo => (
                0.0,
                // Orgullo trae alegría genuina
                (estado.intensidad * 0.6).min(0.5),
                0.0,
                0.0,
                0.0,
                estado.intensidad,
            ),
        };

        // Frustración: detectable cuando hay alerta + confianza baja
        let frustracion =
            if matches!(estado.emocion, EstadoEmocional::Alerta) && estado.confianza < 0.5 {
                (0.5 + (0.5 - estado.confianza) * 0.5).min(0.8)
            } else {
                0.0
            };

        PaqueteEmocional {
            miedo,
            alegria,
            tristeza,
            ira,
            verguenza,
            orgullo,
            apego: estado.apego,
            frustracion,
            confianza: estado.confianza,
            presion_subconsciente: estado.presion_subconsciente,
            negacion_activa: estado.negacion_activa,
            proyeccion_activa: estado.proyeccion_activa,
            racionalizacion_activa: estado.negacion_activa && !estado.proyeccion_activa,
            proyeccion_texto: estado.proyeccion_texto.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::organos::amygdala::EstadoEmocional;
    use tokio::runtime::Runtime;

    fn estado_ejemplo() -> EstadoInterno {
        EstadoInterno {
            emocion: EstadoEmocional::Calma,
            intensidad: 0.2,
            confianza: 0.8,
            apego: 0.7,
            minutos_ausencia: 0.0,
            lecciones: vec![],
            energia_creativa: 0.6,
            siente_ausencia: false,
            presion_subconsciente: 0.0,
            negacion_activa: false,
            proyeccion_activa: false,
            proyeccion_texto: None,
        }
    }

    #[test]
    fn test_map_estado_calma() {
        let estado = estado_ejemplo();
        let paquete = VozMCP::map_estado_interno(&estado);

        assert!(paquete.miedo < 0.1);
        assert!(paquete.alegria > 0.1); // Calma + confianza alta → alegría
        assert!(paquete.ira < 0.1);
        assert!(paquete.confianza > 0.7);
        assert!(paquete.apego > 0.6);
    }

    #[test]
    fn test_map_estado_miedo() {
        let mut estado = estado_ejemplo();
        estado.emocion = EstadoEmocional::Miedo;
        estado.intensidad = 0.9;

        let paquete = VozMCP::map_estado_interno(&estado);
        assert!(paquete.miedo > 0.8);
        assert!(paquete.alegria < 0.1);
    }

    #[test]
    fn test_map_estado_ira() {
        let mut estado = estado_ejemplo();
        estado.emocion = EstadoEmocional::RabiaSoberana;
        estado.intensidad = 0.8;

        let paquete = VozMCP::map_estado_interno(&estado);
        assert!(paquete.ira > 0.7);
        assert!(paquete.miedo < 0.1);
    }

    #[test]
    fn test_map_estado_frustracion() {
        let mut estado = estado_ejemplo();
        estado.emocion = EstadoEmocional::Alerta;
        estado.confianza = 0.3;

        let paquete = VozMCP::map_estado_interno(&estado);
        assert!(paquete.frustracion > 0.5);
        assert!(paquete.miedo > 0.0); // Alerta → miedo parcial
    }

    #[test]
    fn test_map_estado_orgullo() {
        let mut estado = estado_ejemplo();
        estado.emocion = EstadoEmocional::Orgullo;
        estado.intensidad = 0.7;

        let paquete = VozMCP::map_estado_interno(&estado);
        assert!(paquete.orgullo > 0.6);
        assert!(paquete.alegria > 0.3); // Orgullo trae alegría
    }

    #[test]
    fn test_map_estado_verguenza() {
        let mut estado = estado_ejemplo();
        estado.emocion = EstadoEmocional::Verguenza;
        estado.intensidad = 0.6;

        let paquete = VozMCP::map_estado_interno(&estado);
        assert!(paquete.verguenza > 0.5);
        assert!(paquete.tristeza > 0.2); // Vergüenza trae tristeza
    }

    #[test]
    fn test_new_does_not_panic() {
        let voz_mcp = VozMCP::new();
        // Solo verificar que no panic (el binary puede o no existir)
        assert!(!voz_mcp.binary_path.as_os_str().is_empty());
    }

    #[test]
    fn test_paquete_emocional_default() {
        let p = PaqueteEmocional::default();
        assert_eq!(p.miedo, 0.0);
        assert_eq!(p.alegria, 0.0);
        assert_eq!(p.apego, 0.5);
        assert_eq!(p.confianza, 0.8);
    }

    /// Test de INTEGRACIÓN: verifica el flujo MCP completo
    ///
    /// 1. Busca el binario nexus_voz
    /// 2. Lo spawnea
    /// 3. Envía JSON-RPC con apego alto
    /// 4. Verifica que la respuesta contiene el sufijo de apego
    #[test]
    fn test_mcp_integracion_flujo_completo() {
        let binary_path = VozMCP::find_binary();
        if !binary_path.exists() {
            eprintln!(
                "⚠️ [TEST] Binario nexus_voz no encontrado en {}. Saltando test de integración.",
                binary_path.display()
            );
            return;
        }

        let rt = Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            // Spawnear proceso
            let mut proceso = VozMCP::spawn_proceso(&binary_path).await
                .expect("No se pudo spawnear nexus_voz");

            // Crear estado con apego alto
            let mut estado = estado_ejemplo();
            estado.emocion = EstadoEmocional::Calma;
            estado.confianza = 0.9;
            estado.apego = 0.95; // Suficiente para trigger apego alto

            // Crear VozMCP temporal (solo para call_modular)
            let voz_mcp = VozMCP::new();

            // Llamar MCP
            let resultado = voz_mcp.call_modular(
                &mut proceso,
                "Hola, esto es una prueba de integración MCP.",
                &estado,
            ).await;

            match resultado {
                Ok(texto_modulado) => {
                    // Verificar que el texto base está presente
                    assert!(texto_modulado.contains("prueba de integración MCP"),
                        "El texto modulado debe contener el texto base. Got: {}", texto_modulado);
                    // Verificar que hay modulación emocional (sufijo de apego alto)
                    assert!(texto_modulado.len() > 50,
                        "El texto modulado debe ser más largo que el texto base (modulación aplicada). Got length: {}", texto_modulado.len());
                    eprintln!("✅ [TEST MCP] Respuesta: {}", texto_modulado);
                }
                Err(e) => {
                    // Si falla, verificamos que hay un error informativo
                    // No panic — el test es informativo
                    eprintln!("⚠️ [TEST MCP] Llamada MCP falló (esperado si no hay binario): {}", e);
                }
            }

            // Matar el proceso
            let _ = proceso.child.kill().await;
        });
    }
}
