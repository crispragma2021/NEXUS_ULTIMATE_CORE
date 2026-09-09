#!/usr/bin/env node
// ============================================================
// 🔱 NEXUS — SERVIDOR SOBERANO DE UI PARA HERMANO
// Solo sirve dist/index.html — CERO exposición de código fuente
// No hay directory listing, no hay acceso a otros archivos
// ============================================================

const http = require('http');
const fs   = require('fs');
const path = require('path');

const PORT    = 1421;                  // Puerto separado del dev Vite
const HOST    = '0.0.0.0';            // Toda la red LAN
const UI_FILE = path.join(__dirname, '../dist/index.html');

// Un solo archivo servido — nada más
const HTML = fs.readFileSync(UI_FILE, 'utf8');

const server = http.createServer((req, res) => {
    // Toda petición → solo el index.html (SPA)
    // Sin importar la ruta, sin directory listing, sin archivos del sistema
    res.writeHead(200, {
        'Content-Type': 'text/html; charset=utf-8',
        'X-Content-Type-Options': 'nosniff',
        'X-Frame-Options': 'DENY',
        'Cache-Control': 'no-store',
        // Bloquear descarga forzada
        'Content-Disposition': 'inline',
    });
    res.end(HTML);
});

server.listen(PORT, HOST, () => {
    console.log(`\x1b[32m✅ NEXUS Servidor Soberano activo\x1b[0m`);
    console.log(`\x1b[36m🔗 Tu hermano accede en: http://192.168.0.101:${PORT}\x1b[0m`);
    console.log(`\x1b[33m🛡️  Solo sirve index.html — código fuente 100% protegido\x1b[0m`);
    console.log(`\x1b[33m   Ctrl+C para revocar acceso\x1b[0m`);
});

server.on('error', (e) => {
    console.error(`\x1b[31m❌ Error: ${e.message}\x1b[0m`);
    process.exit(1);
});
