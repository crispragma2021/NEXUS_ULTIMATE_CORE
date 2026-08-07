// ==========================================
// TEST CLOUDCODE - Prueba de Endpoint Interno
// ==========================================
use dotenv::dotenv;
use nexus_ultimate_core::infra::cloudcode_tunnel::CloudCodeTunnel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("🔱 [TEST CLOUDCODE] Inicializando prueba de endpoint interno...");

    // 🔒 Clave cargada desde el entorno (GEMINI_API_KEY o GOOGLE_API_KEY).
    // Nada de llaves hardcodeadas en el código (incidente GitGuardian).
    let key = std::env::var("GEMINI_API_KEY")
        .or_else(|_| std::env::var("GOOGLE_API_KEY"))
        .map_err(|_| "Falta GEMINI_API_KEY/GOOGLE_API_KEY en el entorno".to_string())?;

    let tunnel = CloudCodeTunnel::new(&key)?;

    // Endpoint interno de CloudCode con la API Key en el query parameter
    let internal_url = format!(
        "https://cloudcode-pa.googleapis.com/v1internal:generateContent?key={}",
        key
    );

    println!("🔱 URL de prueba: {}", internal_url);

    // Payload envuelto con estructura interna de Code Assist
    let payload = serde_json::json!({
        "model": "models/gemini-2.0-flash", // o "models/gemini-1.5-flash"
        "userPromptId": uuid::Uuid::new_v4().to_string(),
        "request": {
            "contents": [{"parts": [{"text": "Responde brevemente con la palabra 'SÍ' si recibes este impulso."}]}]
        }
    }).to_string();

    println!("🔱 Enviando petición con cabeceras de enmascaramiento...");
    let response = tunnel
        .client
        .post(&internal_url)
        .body(payload)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    println!("🔱 HTTP Status: {}", status);
    println!("🔱 Respuesta:\n{}", body);

    Ok(())
}
