// ============================================================================
// NEXUS-AGENT — Motor agéntico soberano (100% código propio)
// ============================================================================
// Orquestación agéntica multi-proveedor:
//   - ContratoLlm:     capa de abstracción de proveedores (DeepSeek / OpenAI / Ollama)
//   - InstruccionMaestra: directiva inmutable que vive siempre en posición [0]
//   - NexoAgente:      orquestador con bucle razonamiento→acción→observación
//   - EjecutorHermes:  sandbox de herramientas (bash, leer, escribir)
//   - ReglasJSON:      guardarraíl estricto para respuestas estructuradas
// ============================================================================

pub mod contrato;
pub mod delegacion;
pub mod ejecutor;
pub mod mcp_cliente;
pub mod memoria_estado;
pub mod memoria_proyecto;
pub mod nexo;
pub mod programador;
pub mod reglas_json;
pub mod sesion;
pub mod skills;
pub mod tareas;
pub mod web;

pub use contrato::{
    ContratoLlm, DeepSeekCliente, ModeloCliente, ModeloClienteGenerico, OllamaCliente,
    RespuestaLlm, RolMensaje, SaltoAgente, MensajeHistoria, VariableEntorno,
};
pub use delegacion::{Delegador, TareaDelegada};
pub use ejecutor::{EjecutorHermes, ResultadoHerramienta, SandboxConfig};
pub use mcp_cliente::{ClienteMcp, ConfigClienteMcp};
pub use memoria_estado::MemoriaEstado;
pub use memoria_proyecto::MemoriaProyecto;
pub use nexo::{ConfigAgente, NexoAgente, ResultadoCiclo};
pub use programador::{Programador, TareaProgramada};
pub use reglas_json::{PasoEstructurado, ReglasJSON, InstrumentoLlamado};
pub use sesion::{EntradaSesion, Transcripcion};
pub use skills::{BibliotecaSkills, Skill};
pub use tareas::{ListaTareas, Tarea};
pub use web::{ClienteWeb, ResultadoBusqueda};
