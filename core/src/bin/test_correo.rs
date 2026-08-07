use nexus_ultimate_core::comms::correo_temporal::TemporalMailClient;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔱 [TEST CORREO] Inicializando TemporalMailClient...");
    let client = TemporalMailClient::new();

    // 1. Obtener dominios
    println!("🌐 [TEST CORREO] Solicitando dominios disponibles...");
    let domains = client.obtener_dominios().await?;
    if domains.is_empty() {
        println!("❌ [TEST CORREO] No hay dominios disponibles.");
        return Ok(());
    }

    let target_domain = &domains[0].domain;
    println!(
        "🌐 [TEST CORREO] Dominios disponibles detectados: {}",
        target_domain
    );

    // 2. Generar credenciales aleatorias
    let random_id = Uuid::new_v4().to_string()[..8].to_string();
    let email = format!("nexus_{}@{}", random_id, target_domain);
    let password = "NexusStrongPassword123!";

    println!("📧 [TEST CORREO] Dirección generada: {}", email);

    // 3. Crear cuenta
    println!("💾 [TEST CORREO] Registrando cuenta en el proveedor...");
    let acc = client.crear_cuenta(&email, password).await?;
    println!(
        "✅ [TEST CORREO] Cuenta creada exitosamente con ID: {}",
        acc.id
    );

    // 4. Obtener token JWT
    println!("🔑 [TEST CORREO] Iniciando sesión para obtener token...");
    let token = client.obtener_token(&email, password).await?;
    println!("✅ [TEST CORREO] Token JWT recuperado exitosamente.");

    // 5. Listar mensajes (debe estar vacía)
    println!("📨 [TEST CORREO] Consultando bandeja de entrada...");
    let messages = client.listar_mensajes(&token).await?;
    println!(
        "✅ [TEST CORREO] Mensajes en bandeja: {} (Conexión exitosa y funcional)",
        messages.len()
    );

    Ok(())
}
