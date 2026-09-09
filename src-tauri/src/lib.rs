// ==========================================
// NEXUS UI - Punto de entrada de Tauri 2.0
// ==========================================



use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "linux")]
            {
                use webkit2gtk::{PermissionRequestExt, SettingsExt, WebViewExt};

                let _ = window.with_webview(|platform_webview| {
                    let inner = platform_webview.inner();

                    // Aceptar automáticamente solicitudes de permisos de medios (cámara, micrófono)
                    inner.connect_permission_request(|_webview, request| {
                        request.allow();
                        true
                    });

                    // Habilitar MediaSource para MediaRecorder
                    if let Some(settings) = inner.settings() {
                        settings.set_enable_mediasource(true);
                        settings.set_enable_media_stream(true);
                    }
                });
            }

            let _ = window.eval("console.log('NEXUS Santuario UI iniciado')");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error al iniciar NEXUS UI");
}
