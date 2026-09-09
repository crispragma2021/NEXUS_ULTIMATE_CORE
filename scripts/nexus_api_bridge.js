#!/usr/bin/env node
// ============================================================================
// 🔱 NEXUS API BRIDGE — Proxy LLM OpenAI-compatible
// Sirve /v1/chat/completions sin necesidad de Tauri/Display.
// Conecta Roo Code directamente al Orquestador NEXUS en :43210
// ============================================================================

const http = require('http');
const PORT = 43211; // Puerto espejo para este proxy

// ─── Reenviar al Orquestador real ───
function consultarNexus(prompt) {
    return new Promise((resolve, reject) => {
        const body = JSON.stringify({ prompt, modelo: 'nexus' });
        const req = http.request('http://localhost:43210/api/consultar', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Content-Length': Buffer.byteLength(body)
            }
        }, (res) => {
            let data = '';
            res.on('data', c => data += c);
            res.on('end', () => {
                try {
                    const j = JSON.parse(data);
                    resolve(j.respuesta || j.response || data);
                } catch { resolve(data); }
            });
        });
        req.on('error', e => reject(e.message));
        req.write(body);
        req.end();
    });
}

// ─── Servidor HTTP ───
const server = http.createServer(async (req, res) => {
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');

    if (req.method === 'OPTIONS') {
        res.writeHead(200); res.end();
        return;
    }

    if (req.method === 'GET' && req.url === '/api/health') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ status: 'ok', server: 'nexus-api-bridge' }));
        return;
    }

    if (req.method === 'POST' && (req.url === '/v1/chat/completions' || req.url === '/api/consultar')) {
        let body = '';
        req.on('data', c => body += c);
        req.on('end', async () => {
            try {
                const data = JSON.parse(body);

                // Soporte dual: /v1/chat/completions (OpenAI) y /api/consultar (nativo)
                let prompt;
                if (req.url === '/v1/chat/completions') {
                    const messages = data.messages || [];
                    const lastUser = messages.filter(m => m.role === 'user').pop();
                    prompt = lastUser?.content || '';
                } else {
                    prompt = data.prompt || data.query || '';
                }

                if (!prompt) {
                    res.writeHead(400, { 'Content-Type': 'application/json' });
                    res.end(JSON.stringify({ error: 'No prompt provided' }));
                    return;
                }

                const respuesta = await consultarNexus(prompt);

                if (req.url === '/v1/chat/completions') {
                    // Formato OpenAI
                    res.writeHead(200, { 'Content-Type': 'application/json' });
                    res.end(JSON.stringify({
                        id: `chatcmpl-${Date.now()}`,
                        object: 'chat.completion',
                        created: Math.floor(Date.now() / 1000),
                        model: 'nexus-orquestador',
                        choices: [{
                            index: 0,
                            message: { role: 'assistant', content: respuesta },
                            finish_reason: 'stop'
                        }],
                        usage: { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 }
                    }));
                } else {
                    // Formato nativo
                    res.writeHead(200, { 'Content-Type': 'application/json' });
                    res.end(JSON.stringify({ respuesta, modelo_usado: 'nexus', proveedor: 'Nexus Omega' }));
                }
            } catch (e) {
                res.writeHead(500, { 'Content-Type': 'application/json' });
                res.end(JSON.stringify({ error: e.message }));
            }
        });
        return;
    }

    res.writeHead(404); res.end('Not found');
});

server.listen(PORT, '0.0.0.0', () => {
    console.log(`\x1b[36m╔══════════════════════════════════════════════════╗\x1b[0m`);
    console.log(`\x1b[36m║  🔱 NEXUS API BRIDGE — Proxy LLM Activo       ║\x1b[0m`);
    console.log(`\x1b[36m║  Endpoint: http://localhost:${PORT}/v1/chat/completions  ║\x1b[0m`);
    console.log(`\x1b[36m║  Conectado al Orquestador en :43210            ║\x1b[0m`);
    console.log(`\x1b[36m╚══════════════════════════════════════════════════╝\x1b[0m`);
});

server.on('error', e => console.error('❌ Error:', e.message));
