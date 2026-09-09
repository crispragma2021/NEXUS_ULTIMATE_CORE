// Verificación de entorno segura
const isTauri = typeof window !== 'undefined' && window.__TAURI__ !== undefined;
const invoke = isTauri ? window.__TAURI__.core.invoke : async (cmd, args) => {
    console.log(`[MOCK_INVOKE] ${cmd}`, args);
    if (cmd === "ghost_chat_stream") {
        return "Respuesta simulada de NEXUS offline";
    }
    return "Contenido de prueba (Modo Offline)";
};

let editor;
let currentPath = "";

document.addEventListener("DOMContentLoaded", () => {
    console.log("🛠️ Inicializando NEXUS GHOST...");
    
    // Inicializar Ace Editor
    try {
        editor = ace.edit("editor");
        editor.setTheme("ace/theme/monokai");
        editor.session.setMode("ace/mode/rust");
        editor.setOptions({
            fontSize: "13px",
            showPrintMargin: false,
            showGutter: true,
            highlightActiveLine: true,
            enableBasicAutocompletion: true,
            useWorker: false
        });
        console.log("✅ Ace Editor inicializado correctamente.");
    } catch (e) {
        console.error("❌ Fallo al inicializar Ace Editor:", e);
    }

    const orb = document.getElementById("nexus-orb");
    const chat = document.getElementById("ghost-chat");
    const ide = document.getElementById("ghost-ide");
    
    // El Orbe abre y cierra la consola de chat
    orb.addEventListener("click", (e) => {
        chat.classList.toggle("hidden");
        e.stopPropagation();
    });

    // Minimizar chat
    document.getElementById("close-chat").addEventListener("click", () => {
        chat.classList.add("hidden");
    });

    // Alternar editor secundario
    document.getElementById("toggle-editor-btn").addEventListener("click", () => {
        ide.classList.toggle("hidden");
    });

    document.getElementById("close-ide").addEventListener("click", () => {
        ide.classList.add("hidden");
    });

    // Enviar mensajes de chat
    const chatInput = document.getElementById("chat-input");
    const sendBtn = document.getElementById("send-btn");

    async function handleSend() {
        const text = chatInput.value.trim();
        if (!text) return;

        addMessage(text, 'user');
        chatInput.value = "";

        // Mostrar indicador de carga
        const typingEl = addMessage("Procesando...", 'nexus');
        typingEl.classList.add('typing');

        try {
            // Intentar usar websocket o REST/IPC si Tauri está disponible
            let response = "";
            if (isTauri) {
                // Si la API REST local de core está expuesta en 43210
                const res = await fetch("http://localhost:43210/api/chat", {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ prompt: text })
                });
                if (res.ok) {
                    const data = await res.json();
                    response = data.response || data.content || JSON.stringify(data);
                } else {
                    response = "Error al conectar con la API de NEXUS Core.";
                }
            } else {
                response = "Inferencia local deshabilitada. Modo offline activo.";
            }
            typingEl.innerText = response;
            typingEl.classList.remove('typing');
        } catch (err) {
            typingEl.innerText = "Error: " + err.message;
            typingEl.classList.remove('typing');
        }
    }

    sendBtn.addEventListener("click", handleSend);
    chatInput.addEventListener("keypress", (e) => {
        if (e.key === "Enter") handleSend();
    });

    document.getElementById("save-btn").addEventListener("click", saveFile);
    
    document.querySelectorAll(".tree-item").forEach(item => {
        item.addEventListener("click", async () => {
            const path = item.getAttribute("data-path");
            await loadFile(path);
        });
    });
});

function addMessage(text, sender) {
    const chatMessages = document.getElementById("chat-messages");
    const msg = document.createElement("div");
    msg.className = `msg ${sender}`;
    msg.innerText = text;
    chatMessages.appendChild(msg);
    chatMessages.scrollTop = chatMessages.scrollHeight;
    return msg;
}

async function loadFile(path) {
    try {
        console.log(`📂 Cargando archivo: ${path}`);
        const content = await invoke("read_nexus_file", { path });
        editor.setValue(content, -1);
        currentPath = path;
        document.getElementById("current-filename").innerText = path.split('/').pop();
        
        if (path.endsWith(".rs")) editor.session.setMode("ace/mode/rust");
        else if (path.endsWith(".md")) editor.session.setMode("ace/mode/markdown");
    } catch (err) {
        console.error("❌ Error cargando archivo:", err);
    }
}

async function saveFile() {
    if (!currentPath) return;
    try {
        const content = editor.getValue();
        await invoke("save_nexus_file", { path: currentPath, content });
        const btn = document.getElementById("save-btn");
        btn.innerText = "✓ OK";
        setTimeout(() => btn.innerText = "GUARDAR", 2000);
    } catch (err) {
        console.error("❌ Error guardando archivo:", err);
    }
}
