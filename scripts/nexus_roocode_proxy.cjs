#!/usr/bin/env node
// ============================================================
// 🔱 NEXUS — PROXY SOBERANO PARA ROO CODE DEL HERMANO
// Expone NEXUS como proveedor OpenAI-compatible en red LAN
// El hermano configura este endpoint en Roo Code
// Solo acepta requests con la API key correcta
// ============================================================

const http  = require('http');
const https = require('https');

const LAN_PORT    = 4445;              // Puerto para el hermano
const NEXUS_PORT  = 4444;              // Proxy NEXUS interno
const NEXUS_HOST  = '127.0.0.1';
const API_KEY     = 'nexus-hermano-key-2026';  // Clave para Roo Code del hermano

const server = http.createServer((req, res) => {

    // ─── Autenticación por API Key ───
    const auth = req.headers['authorization'] || '';
    const key  = auth.replace('Bearer ', '').trim();

    if (key !== API_KEY) {
        res.writeHead(401, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: 'Unauthorized — API key inválida' }));
        console.log(`[BLOQUEADO] ${req.method} ${req.url} — key incorrecta`);
        return;
    }

    console.log(`[OK] ${req.method} ${req.url}`);

    // ─── Proxy transparente → NEXUS interno ───
    const options = {
        hostname: NEXUS_HOST,
        port: NEXUS_PORT,
        path: req.url,
        method: req.method,
        headers: {
            ...req.headers,
            host: `${NEXUS_HOST}:${NEXUS_PORT}`,
            'authorization': `Bearer nexus-internal`,
        },
    };

    const proxy = http.request(options, (nexusRes) => {
        res.writeHead(nexusRes.statusCode, {
            ...nexusRes.headers,
            'access-control-allow-origin': '*',
            'access-control-allow-headers': 'Content-Type, Authorization',
        });
        nexusRes.pipe(res, { end: true });
    });

    proxy.on('error', (e) => {
        res.writeHead(502, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: `NEXUS proxy error: ${e.message}` }));
    });

    // CORS preflight
    if (req.method === 'OPTIONS') {
        res.writeHead(200, {
            'access-control-allow-origin': '*',
            'access-control-allow-methods': 'GET, POST, OPTIONS',
            'access-control-allow-headers': 'Content-Type, Authorization',
        });
        res.end();
        return;
    }

    req.pipe(proxy, { end: true });
});

server.listen(LAN_PORT, '0.0.0.0', () => {
    console.log(`\x1b[32m`);
    console.log(`╔═══════════════════════════════════════════════════════╗`);
    console.log(`║  🔱 NEXUS — Proxy Soberano para Roo Code            ║`);
    console.log(`╠═══════════════════════════════════════════════════════╣`);
    console.log(`║  Endpoint LAN:  http://192.168.0.101:${LAN_PORT}         ║`);
    console.log(`║  API Key:       ${API_KEY}  ║`);
    console.log(`║  Modelo:        nexus (o gemini-3-flash-preview)      ║`);
    console.log(`╠═══════════════════════════════════════════════════════╣`);
    console.log(`║  Config Roo Code del hermano:                         ║`);
    console.log(`║  Provider: OpenAI Compatible                          ║`);
    console.log(`║  Base URL: http://192.168.0.101:${LAN_PORT}/v1          ║`);
    console.log(`║  API Key:  ${API_KEY}  ║`);
    console.log(`║  Model:    nexus                                       ║`);
    console.log(`╚═══════════════════════════════════════════════════════╝`);
    console.log(`\x1b[0m`);
});

server.on('error', e => {
    console.error(`\x1b[31m❌ Error: ${e.message}\x1b[0m`);
    process.exit(1);
});
