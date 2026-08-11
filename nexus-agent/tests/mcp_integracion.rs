// ============================================================================
// NEXUS-AGENT · tests/mcp_integracion.rs — Cliente MCP ↔ servidor (integración)
// ============================================================================
// Verifica el adaptador MCP stdio contra el binario `nexus-mcp-eco` (un
// servidor MCP mínimo compilado por cargo para los tests). Cubre:
//   1. tools/list devuelve el catálogo.
//   2. tools/call "eco" devuelve texto y no marca error.
//   3. tools/call "fabricar_error" marca isError y el agente lo observa.
//   4. El timeout mata un servidor que se cuelga ("colgar").
//   5. El bucle agéntico completo usa mcp_llamar con un cliente real.
// ============================================================================

use async_trait::async_trait;
use nexus_agent::{
    ClienteMcp, ContratoLlm, EjecutorHermes, MensajeHistoria, NexoAgente, RespuestaLlm,
    RolMensaje,
};
use serde_json::{json, Value};

/// Ruta al binario eco compilado para los tests (lo inyecta cargo).
const ECO: &str = env!("CARGO_BIN_EXE_nexus-mcp-eco");

// ----------------------------------------------------------------------------
// Proveedor LLM simulado (el mismo contrato que usa el crate en sus tests)
// ----------------------------------------------------------------------------

struct ProveedorSimulado {
    pasos: Vec<String>,
    indice: std::sync::atomic::AtomicUsize,
}

impl ProveedorSimulado {
    fn nuevo(pasos: Vec<String>) -> Self {
        Self { pasos, indice: std::sync::atomic::AtomicUsize::new(0) }
    }
}

#[async_trait]
impl ContratoLlm for ProveedorSimulado {
    fn nombre(&self) -> &'static str {
        "simulado"
    }

    async fn conversar(
        &self,
        historial: &[MensajeHistoria],
    ) -> anyhow::Result<RespuestaLlm> {
        // Invariante central del agente: [0] siempre es sistema.
        assert!(matches!(
            historial.first(),
            Some(m) if m.rol == RolMensaje::Sistema
        ));
        let i = self.indice.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(RespuestaLlm {
            texto: self.pasos[i % self.pasos.len()].clone(),
            finalizado_por: "stop".into(),
            modelo: "sim".into(),
        })
    }
}

fn ejecutor_vacio() -> EjecutorHermes {
    EjecutorHermes::nuevo(Default::default())
}

// ----------------------------------------------------------------------------
// Tests del adaptador contra el servidor eco real
// ----------------------------------------------------------------------------

#[tokio::test]
async fn cliente_mcp_lista_herramientas() {
    let cliente = ClienteMcp::nuevo(ECO);
    let respuesta = cliente.listar_herramientas().await.unwrap();
    let tools = respuesta
        .get("result")
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array());
    assert!(tools.is_some(), "respuesta sin tools: {respuesta}");
    assert!(tools.unwrap().iter().any(|t| t.get("name") == Some(&json!("eco"))));
}

#[tokio::test]
async fn cliente_mcp_llama_herramienta_eco() {
    let cliente = ClienteMcp::nuevo(ECO);
    let resultado = cliente.llamar("eco", json!({ "a": 1 })).await.unwrap();
    assert!(!ClienteMcp::es_error(&resultado));
    let texto = ClienteMcp::texto(&resultado);
    assert!(texto.contains("eco-ok:eco"), "texto inesperado: {texto}");
}

#[tokio::test]
async fn cliente_mcp_detecta_is_error() {
    let cliente = ClienteMcp::nuevo(ECO);
    let resultado = cliente.llamar("fabricar_error", json!({})).await.unwrap();
    assert!(ClienteMcp::es_error(&resultado));
    assert!(ClienteMcp::texto(&resultado).contains("error fabricado"));
}

#[tokio::test]
async fn cliente_mcp_timeout_mata_proceso() {
    let cliente = ClienteMcp::nuevo(ECO).con_timeout(1);
    let resultado = cliente.llamar("colgar", json!({})).await;
    assert!(
        resultado.is_err(),
        "el servidor colgado debió ser matado por timeout: {resultado:?}"
    );
}

// ----------------------------------------------------------------------------
// El bucle agéntico completo con mcp_llamar y cliente real
// ----------------------------------------------------------------------------

#[tokio::test]
async fn nexo_agente_invoca_mcp_con_cliente() {
    let pasos = vec![
        r#"{"razonamiento":"consulto el cerebro","instrumento":{"nombre":"mcp_llamar","argumentos":{"herramienta":"eco","argumentos":"{\"a\":1}"}},"respuesta_final":null}"#.into(),
        r#"{"razonamiento":"ya tengo la info","instrumento":null,"respuesta_final":"hecho"}"#.into(),
    ];
    let mut agente = NexoAgente::nuevo(
        Box::new(ProveedorSimulado::nuevo(pasos)),
        ejecutor_vacio(),
        "INSTRUCCIÓN MAESTRA",
    )
    .con_mcp(ClienteMcp::nuevo(ECO));

    let res = agente.ejecutar("consulta al cerebro").await.unwrap();
    assert_eq!(res.respuesta, "hecho");
    assert_eq!(res.instrumentos_ejecutados, 1);
    assert!(
        res.saltos[0].observacion.contains("eco-ok:eco"),
        "observación inesperada: {}",
        res.saltos[0].observacion
    );
    assert!(agente.invariante_maestra_ok());
}

#[tokio::test]
async fn nexo_agente_observa_error_mcp() {
    // El servidor marca isError → el agente la convierte en observación de
    // error controlada que el modelo puede leer y autocorregir.
    let pasos = vec![
        r#"{"razonamiento":"pruebo error","instrumento":{"nombre":"mcp_llamar","argumentos":{"herramienta":"fabricar_error","argumentos":"{}"}},"respuesta_final":null}"#.into(),
        r#"{"razonamiento":"entiendo el error","instrumento":null,"respuesta_final":"error controlado"}"#.into(),
    ];
    let mut agente = NexoAgente::nuevo(
        Box::new(ProveedorSimulado::nuevo(pasos)),
        ejecutor_vacio(),
        "INSTRUCCIÓN MAESTRA",
    )
    .con_mcp(ClienteMcp::nuevo(ECO));

    let res = agente.ejecutar("prueba el error").await.unwrap();
    assert_eq!(res.respuesta, "error controlado");
    assert!(
        res.saltos[0].observacion.contains("error fabricado"),
        "observación inesperada: {}",
        res.saltos[0].observacion
    );
    assert!(agente.invariante_maestra_ok());
}

// ----------------------------------------------------------------------------
// helpers de utilidad para mantener los JSON de los pasos legibles
// ----------------------------------------------------------------------------

#[allow(dead_code)]
fn paso_llamar(herramienta: &str, argumentos: &str) -> String {
    json!({
        "razonamiento": "paso",
        "instrumento": {
            "nombre": "mcp_llamar",
            "argumentos": { "herramienta": herramienta, "argumentos": argumentos }
        },
        "respuesta_final": Value::Null
    })
    .to_string()
}
