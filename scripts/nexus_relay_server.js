// ============================================================
// 🔱 NEXUS — Relay Soberano Autenticado (Servidor)
// Escucha WebSocket del hermano con HANDSHAKE POR TOKEN,
// y puentea a SSH local SOLO tras validar credencial.
// ============================================================
// Uso: node nexus_relay_server.js <relay_port> <ssh_host> <ssh_port> <reverse_port> <token>
//   El cliente debe enviar como PRIMER mensaje: {"auth":"<TOKEN>"}
//   Si el token coincide → se abre el puente TCP a SSH local.
//   Si no → se cierra la conexión inmediatamente.
const http = require('http');
const net = require('net');
const crypto = require('crypto');
const { WebSocketServer } = require('ws');

const RELAY_PORT = parseInt(process.argv[2] || '4470');
const SSH_HOST = process.argv[3] || '127.0.0.1';
const SSH_PORT = parseInt(process.argv[4] || '22');
const REVERSE_PORT = parseInt(process.argv[5] || '2222');
const AUTH_TOKEN = process.argv[6] || 'NEXUS_INSECURE_NO_TOKEN';
const HANDSHAKE_TIMEOUT_MS = 10000; // 10s para autenticar, luego cerrar

const server = http.createServer((req, res) => {
    res.writeHead(200, { 'Content-Type': 'text/html' });
    res.end(`<!DOCTYPE html>
<html><head><title>🔱 NEXUS Relay</title>
<style>body{background:#0a0a0a;color:#00ff88;font-family:monospace;padding:40px;}
h1{color:#00ff88;}.status{color:#888;}</style></head>
<body>
<h1>🔱 NEXUS — Puente de Control Soberano</h1>
<p class="status">Relay autenticado. Esperando conexión autenticada del hermano...</p>
<hr>
<p><small>Server: ${SSH_HOST}:${SSH_PORT} | Relay: ${RELAY_PORT}</small></p>
</body></html>`);
});

const wss = new WebSocketServer({ server });

wss.on('connection', (ws, req) => {
    const clientIp = req.socket.remoteAddress;
    const time = new Date().toISOString();
    console.log(`[${time}] 🔗 CONEXIÓN ENTRANTE desde ${clientIp} — esperando handshake...`);

    let authenticated = false;
    let tcp = null;
    let tcpReady = false;   // ¿el TCP a SSH está listo?
    let pendingQueue = [];  // buffer de datos antes de que TCP esté listo

    // Timer de handshake: si no autentica en X segundos, cerrar
    const handshakeTimer = setTimeout(() => {
        if (!authenticated) {
            console.log(`[${time}] ⛔ Handshake no completado por ${clientIp} — cerrando (timeout)`);
            ws.close(4001, 'autenticación requerida');
        }
    }, HANDSHAKE_TIMEOUT_MS);

    // Función para intentar abrir el TCP a SSH (una sola vez)
    const abrirTcp = () => {
        if (tcp !== null) return; // ya abierto
        tcp = net.createConnection(SSH_PORT, SSH_HOST, () => {
            tcpReady = true;
            console.log(`[${time}] ✅ TCP conectado a SSH local (${SSH_HOST}:${SSH_PORT})`);
            // Vaciar cola pendiente
            for (const chunk of pendingQueue) {
                tcp.write(chunk);
            }
            pendingQueue = [];
        });

        tcp.on('data', (data) => {
            if (ws.readyState === ws.OPEN) {
                ws.send(data);
            }
        });

        tcp.on('error', (err) => {
            console.error(`[${time}] [TCP Error] ${err.message}`);
            if (ws.readyState === ws.OPEN) ws.close(1011, 'tcp error');
        });

        tcp.on('close', () => {
            if (ws.readyState === ws.OPEN) ws.close(1000, 'tcp cerrado');
        });
    };

    ws.on('message', (data) => {
        // El PRIMER mensaje DEBE ser el handshake de autenticación.
        if (!authenticated) {
            let payload;
            try {
                payload = data.toString();
                const obj = JSON.parse(payload);
                const supplied = obj.auth || '';
                // Comparación en tiempo constante para evitar timing attacks
                const a = Buffer.from(String(supplied));
                const b = Buffer.from(AUTH_TOKEN);
                const ok = a.length === b.length && crypto.timingSafeEqual(a, b);
                if (ok) {
                    authenticated = true;
                    clearTimeout(handshakeTimer);
                    console.log(`[${time}] ✅ HANDSHAKE VÁLIDO desde ${clientIp} — autenticado`);
                    ws.send(JSON.stringify({ status: 'auth_ok', relay: 'nexus', ts: time }));
                    abrirTcp();
                    return;
                } else {
                    console.log(`[${time}] ⛔ TOKEN INVÁLIDO desde ${clientIp} — rechazado`);
                    ws.close(4003, 'token inválido');
                    return;
                }
            } catch (e) {
                console.log(`[${time}] ⛔ Handshake malformado desde ${clientIp} — rechazado`);
                ws.close(4002, 'handshake malformado');
                return;
            }
        }

        // Ya autenticado: reenviar bytes a SSH (dato de túnel)
        if (tcpReady) {
            const buf = Buffer.isBuffer(data) ? data : Buffer.from(data);
            tcp.write(buf);
        } else {
            pendingQueue.push(Buffer.isBuffer(data) ? data : Buffer.from(data));
        }
    });

    ws.on('close', (code, reason) => {
        clearTimeout(handshakeTimer);
        if (tcp) tcp.end();
        console.log(`[${time}] ❌ Conexión cerrada (${clientIp}) code=${code} ${reason}`);
    });

    ws.on('error', (err) => {
        clearTimeout(handshakeTimer);
        if (tcp) tcp.end();
        console.error(`[${time}] [WS Error] ${err.message}`);
    });
});

server.listen(RELAY_PORT, '0.0.0.0', () => {
    console.log(`\x1b[32m`);
    console.log(`╔═══════════════════════════════════════════════════╗`);
    console.log(`║  🔱 RELAY SOBERANO AUTENTICADO ACTIVO            ║`);
    console.log(`╠═══════════════════════════════════════════════════╣`);
    console.log(`║  Relay local:      ws://0.0.0.0:${RELAY_PORT}              ║`);
    console.log(`║  Puentea a:        ${SSH_HOST}:${SSH_PORT}                          ║`);
    console.log(`║  Handshake:        TOKEN requerido (primer frame) ║`);
    console.log(`║  Timeout auth:     ${HANDSHAKE_TIMEOUT_MS/1000}s                       ║`);
    console.log(`║                                                    ║`);
    console.log(`║  ⏳ Esperando conexión AUTENTICADA del hermano... ║`);
    console.log(`╚═══════════════════════════════════════════════════╝`);
    console.log(`\x1b[0m`);
});
