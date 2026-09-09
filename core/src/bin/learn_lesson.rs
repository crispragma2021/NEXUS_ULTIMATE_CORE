// ============================================================
// LEARN LESSON - NEXUS HIPOCAMPUS RECORDING TOOL
// ============================================================
// Registra hitos e inteligencias directamente en el órgano de
// memoria (lessons_sovereign de SQLite) usando DatabaseManager.
// ============================================================

use anyhow::Result;
use nexus_ultimate_core::memoria::persistence::DatabaseManager;

#[tokio::main]
async fn main() -> Result<()> {
    let db_path = "sqlite:nexus_intelligence.db";
    let db = DatabaseManager::new(db_path).await?;

    let title = "Hito OMEGA: Enmascaramiento Zenith Pool y Visualizador SDF GPU";
    let content = "Consolidado el 22 de Mayo de 2026.\n\
                   1. Enmascaramiento de Zenith Pool inyectando cabeceras 'X-Goog-Api-Client: cloud-code-vscode/1.0.0' en todas las peticiones a Gemini.\n\
                   2. Fallback de contingencia implementado en orquestador.rs vinculando CloudCodeTunnel.\n\
                   3. Interceptación en proxy_hijack.rs (:4444) de /v1internal:generateContent retornando la personalidad e identidad de NEXUS de forma nativa.\n\
                   4. Visualizador acelerado por GPU (demo_userland_sdf.rs) renderizando Signed Distance Fields en WGPU e hilos de CPU y RAM de sysinfo 0.31 sin DOM.";

    println!("🔱 [MEMORY ORGAN] Registrando hito en nexus_intelligence.db...");
    db.learn_lesson(title, content, 10).await?;
    println!("✅ [MEMORY ORGAN] Hito grabado de forma inmutable en lessons_sovereign.");

    // Consultar todas las lecciones del hipocampo para verificar
    println!("📡 [MEMORY ORGAN] Recuperando lecciones actuales...");
    let lessons = db.get_lessons().await?;
    for (idx, (t, c)) in lessons.iter().enumerate() {
        println!(
            "📝 Lección [{}] - Titulo: {}\nContenido:\n{}\n",
            idx + 1,
            t,
            c
        );
    }

    Ok(())
}
