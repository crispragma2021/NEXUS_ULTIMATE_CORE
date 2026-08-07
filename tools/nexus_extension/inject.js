(function () {
    console.log("🔱 [NEXUS] Infiltración de Flujo activada...");

    // 1. Intercepción de Fetch (DeepSeek SSE)
    const originalFetch = window.fetch;
    window.fetch = async (...args) => {
        const response = await originalFetch(...args);
        const url = args[0] instanceof Request ? args[0].url : args[0];

        if (url.includes("/api/v0/chat/completion") || url.includes("/chat/send_message")) {
            const clone = response.clone();
            const reader = clone.body.getReader();

            (async () => {
                while (true) {
                    const { done, value } = await reader.read();
                    if (done) break;
                    const chunk = new TextDecoder().decode(value);
                    window.postMessage({ type: 'NEXUS_TOKEN', data: chunk, source: 'FETCH' }, '*');
                }
            })();
        }
        return response;
    };

    // 2. Intercepción de WebSocket (Gemini Protobuf)
    const OriginalWS = window.WebSocket;
    window.WebSocket = function (...args) {
        const socket = new OriginalWS(...args);

        socket.addEventListener('message', (event) => {
            // En el mundo real decodificaríamos Protobuf aquí o en el Rust backend
            window.postMessage({ type: 'NEXUS_TOKEN', data: event.data, source: 'WS' }, '*');
        });

        return socket;
    };

    // Extensión de prototipo para asegurar mimetismo
    window.WebSocket.prototype = OriginalWS.prototype;
})();
