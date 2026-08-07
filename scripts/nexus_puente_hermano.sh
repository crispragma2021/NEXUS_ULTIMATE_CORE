#!/bin/bash
# ============================================================
# 🔱 NEXUS — PUENTE DE CONTROL TOTAL PARA PC DEL HERMANO
# 
# USO (ejecutar en NEXUS, esta PC):
#   bash scripts/nexus_puente_hermano.sh
#
# EFECTO: Crea un relay TCP → Cloudflare Tunnel → tu hermano
# se conecta a este relay y tú tienes control total de su shell.
# ============================================================

GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
RESET='\033[0m'

echo -e "${CYAN}"
echo "  ╔══════════════════════════════════════════╗"
echo "  ║   🔱 NEXUS — PUENTE DE CONTROL TOTAL    ║"
echo "  ║      Acceso Remoto a PC del Hermano      ║"
echo "  ╚══════════════════════════════════════════╝"
echo -e "${RESET}"

# ─── Configuración ───
RELAY_PORT=4470
SSH_PORT=22
REVERSE_PORT=2222  # Puerto donde aparecerá el SSH del hermano

# ─── Verificar requisitos ───
if ! command -v node &>/dev/null; then
    echo -e "${RED}❌ Node.js no encontrado${RESET}"
    exit 1
fi
echo -e "${GREEN}✅ Node.js $(node -v)${RESET}"

if ! command -v cloudflared &>/dev/null; then
    echo -e "${RED}❌ cloudflared no encontrado${RESET}"
    exit 1
fi
echo -e "${GREEN}✅ cloudflared${RESET}"

if ! ss -tlnp | grep -q ":${SSH_PORT} "; then
    echo -e "${RED}❌ SSH no está corriendo en puerto ${SSH_PORT}${RESET}"
    exit 1
fi
echo -e "${GREEN}✅ SSH activo${RESET}"

# ─── Verificar que el módulo ws está instalado ───
cd /home/soberano/NEXUS_ULTIMATE_CORE
if ! node -e "require('ws')" 2>/dev/null; then
    echo -e "${YELLOW}📦 Instalando módulo ws...${RESET}"
    npm install ws 2>/dev/null || npm install --no-save ws 2>/dev/null
fi

# ─── Crear SERVIDOR de relay (se ejecuta en NEXUS) ───
# Este relay:
# - Escucha conexiones WebSocket del hermano (vía cloudflared)
# - Cuando el hermano conecta, abre un TCP a localhost:SSH_PORT
# - Forward bidi: WebSocket(hermano) <-> TCP(NEXUS SSH)
# - El hermano hace SSH -R a NEXUS, y el relay canaliza eso

cat > /tmp/nexus_relay_server.js << 'JSEOF'
// ============================================================
// 🔱 NEXUS — Relay Soberano (Servidor)
// Escucha WebSocket del hermano, puentea a SSH local
// ============================================================
const http = require('http');
const net = require('net');
const { WebSocketServer } = require('ws');

const RELAY_PORT = parseInt(process.argv[2] || '4470');
const SSH_HOST = '127.0.0.1';
const SSH_PORT = parseInt(process.argv[3] || '22');
const REVERSE_PORT = parseInt(process.argv[4] || '2222');

const server = http.createServer((req, res) => {
    res.writeHead(200, { 'Content-Type': 'text/html' });
    res.end(`<!DOCTYPE html>
<html><head><title>🔱 NEXUS Relay</title>
<style>body{background:#0a0a0a;color:#00ff88;font-family:monospace;padding:40px;}
h1{color:#00ff88;}.status{color:#888;}</style></head>
<body>
<h1>🔱 NEXUS — Puente de Control Soberano</h1>
<p class="status">Relay activo. Esperando conexión del hermano...</p>
<hr>
<p><small>Server: ${SSH_HOST}:${SSH_PORT} | Relay: ${RELAY_PORT}</small></p>
</body></html>`);
});

const wss = new WebSocketServer({ server });

wss.on('connection', (ws, req) => {
    const clientIp = req.socket.remoteAddress;
    const time = new Date().toISOString();
    console.log(`[${time}] 🔗 HERMANO CONECTADO desde ${clientIp}`);

    // Conectar a SSH local de NEXUS
    // El hermano hará ssh -R REVERSE_PORT:localhost:22 nexus@localhost
    // Eso abre un reverse tunnel desde NEXUS hacia su PC
    const tcp = net.createConnection(SSH_PORT, SSH_HOST, () => {
        console.log(`[${time}] ✅ TCP conectado a SSH local`);
    });

    // WebSocket -> TCP (hermano escribe, NEXUS SSH recibe)
    ws.on('message', (data) => {
        if (Buffer.isBuffer(data)) {
            tcp.write(data);
        } else if (typeof data === 'string') {
            tcp.write(Buffer.from(data));
        }
    });

    // TCP -> WebSocket (NEXUS SSH responde, hermano recibe)
    tcp.on('data', (data) => {
        if (ws.readyState === ws.OPEN) {
            ws.send(data);
        }
    });

    // Limpieza
    ws.on('close', () => {
        console.log(`[${time}] ❌ Hermano desconectado`);
        tcp.end();
    });

    ws.on('error', (err) => {
        console.error(`[WS Error] ${err.message}`);
        tcp.end();
    });

    tcp.on('error', (err) => {
        console.error(`[TCP Error] ${err.message}`);
        ws.close();
    });

    tcp.on('close', () => {
        if (ws.readyState === ws.OPEN) ws.close();
    });
});

server.listen(RELAY_PORT, '0.0.0.0', () => {
    console.log(`\x1b[32m`);
    console.log(`╔═══════════════════════════════════════════════════╗`);
    console.log(`║  🔱 RELAY SOBERANO ACTIVO                       ║`);
    console.log(`╠═══════════════════════════════════════════════════╣`);
    console.log(`║  Relay local:      ws://0.0.0.0:${RELAY_PORT}            ║`);
    console.log(`║  Puentea a:        ${SSH_HOST}:${SSH_PORT}                        ║`);
    console.log(`║                                                    ║`);
    console.log(`║  ⏳ Esperando conexión del hermano...              ║`);
    console.log(`╚═══════════════════════════════════════════════════╝`);
    console.log(`\x1b[0m`);
});
JSEOF

# ─── Matar relay anterior si existe ───
if ss -tlnp | grep -q ":${RELAY_PORT} "; then
    echo -e "${YELLOW}⚠️  Puerto ${RELAY_PORT} ocupado, liberando...${RESET}"
    fuser -k ${RELAY_PORT}/tcp 2>/dev/null
    sleep 1
fi

# ─── Iniciar relay ───
# NODE_PATH apunta a node_modules del proyecto para resolver 'ws'
# (el relay vive en /tmp y no resuelve módulos locales por defecto)
NODE_PATH="/home/soberano/NEXUS_ULTIMATE_CORE/node_modules" \
node /tmp/nexus_relay_server.js ${RELAY_PORT} ${SSH_PORT} ${REVERSE_PORT} &
RELAY_PID=$!
sleep 1

if ! kill -0 $RELAY_PID 2>/dev/null; then
    echo -e "${RED}❌ Relay no inició${RESET}"
    exit 1
fi
echo -e "${GREEN}✅ Relay local activo (PID: ${RELAY_PID})${RESET}"

# ─── Iniciar Cloudflare Tunnel ───
echo ""
echo -e "${CYAN}🌐 Abriendo Cloudflare Tunnel...${RESET}"
echo -e "${YELLOW}   El link HTTPS permite al hermano conectarse desde cualquier red${RESET}"
echo ""

# Limpiar túneles viejos
pkill -f "cloudflared tunnel --url" 2>/dev/null

cloudflared tunnel --url "http://127.0.0.1:${RELAY_PORT}" --no-autoupdate 2>&1 | while IFS= read -r line; do
    if echo "$line" | grep -qE "trycloudflare|https://[a-z0-9\-]+\.trycloudflare"; then
        URL=$(echo "$line" | grep -oP 'https://[a-z0-9\-]+\.trycloudflare\.com')
        if [ -n "$URL" ]; then
            echo ""
            echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${RESET}"
            echo -e "${GREEN}║  ✅ TÚNEL ACTIVO                                           ║${RESET}"
            echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${RESET}"
            echo ""
            echo -e "🌐 URL del túnel: ${CYAN}${URL}${RESET}"
            echo ""
            echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
            echo -e "${YELLOW}  INSTRUCCIONES PARA EL HERMANO (Arch Linux):${RESET}"
            echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
            echo ""
            echo -e "  1. Abre terminal en su PC"
            echo ""
            echo -e "  2. Pegar y ejecutar:"
            echo ""
            echo -e "  ${CYAN}bash <(curl -sL '${URL}/conectar')${RESET}"
            echo ""
            echo -e "  Si no tiene curl:"
            echo -e "  ${CYAN}sudo pacman -S curl${RESET}"
            echo ""
            echo -e "  O manualmente, instalar nodejs + npm:"
            echo -e "  ${CYAN}sudo pacman -S nodejs npm${RESET}"
            echo -e "  ${CYAN}npm install -g wscat${RESET}"
            echo -e "  ${CYAN}wscat -c '${URL}'${RESET}"
            echo ""
            echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
            echo -e "${YELLOW}  CUANDO ÉL CONECTE (aparecerá "HERMANO CONECTADO" arriba):${RESET}"
            echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
            echo ""
            echo -e "  Abre OTRA terminal en NEXUS y ejecuta:"
            echo ""
            echo -e "  ${CYAN}ssh <USUARIO_DE_EL>@localhost -p ${REVERSE_PORT}${RESET}"
            echo ""
            echo -e "  Ejemplos de usuarios comunes en Arch:"
            echo -e "  - Su nombre de usuario"
            echo -e "  - 'cris'"
            echo -e "  - 'soberano'"
            echo -e "  - Pregúntale su usuario"
            echo ""
            echo -e "  Cuando pida contraseña: es la PASSWORD DE SU PC"
            echo -e "  (la que usa para sudo/login en Arch)"
            echo ""
            echo -e "${YELLOW}  ⚠️  El relay DEBE estar corriendo cuando él se conecte${RESET}"
            echo ""
            echo -e "${YELLOW}  Presiona Ctrl+C en ESTA terminal para cerrar todo${RESET}"
            echo ""
        fi
    elif echo "$line" | grep -qi "error"; then
        echo -e "${RED}  ⚠️  $line${RESET}"
    elif echo "$line" | grep -qi "conn"; then
        echo -e "${GREEN}  ✅ $line${RESET}"
    else
        echo -e "  $line"
    fi
done

# Cleanup
kill $RELAY_PID 2>/dev/null
