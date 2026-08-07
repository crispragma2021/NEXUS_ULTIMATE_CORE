let ws = null;

function connect() {
    console.log("🔱 [NEXUS] Intentando conectar con Dashboard Reactor...");
    ws = new WebSocket('ws://localhost:43211/ws/tunnel');

    ws.onopen = () => {
        console.log("✅ [NEXUS] Túnel OMEGA establecido.");
    };

    ws.onclose = () => {
        console.log("❌ [NEXUS] Túnel cerrado. Reintentando en 5s...");
        setTimeout(connect, 5000);
    };

    ws.onerror = (e) => {
        console.error("⚠️ [NEXUS] Error en el túnel:", e);
    };
}

connect();

chrome.runtime.onMessage.addListener((message) => {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(message));
    }
});
