#!/bin/bash
# ============================================================
# 🔱 NEXUS — TÚNEL DE CONTROL REMOTO PARA HERMANO
# Crea un relay Cloudflare TCP → SSH para que el hermano
# conecte vía reverse SSH desde su Arch Linux.
#
# USO: ./scripts/nexus_tunel_hermano.sh
# ============================================================

GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
RESET='\033[0m'

SSH_PORT=22
TUNNEL_PORT=2222  # Puerto local que recibe el forward del hermano

echo -e "${CYAN}"
echo "  ╔══════════════════════════════════════════╗"
echo "  ║   🔱 NEXUS — TÚNEL DE CONTROL           ║"
echo "  ║      Acceso SSH Remoto para Hermano      ║"
echo "  ╚══════════════════════════════════════════╝"
echo -e "${RESET}"

# ─── Verificar SSH ───
if ! ss -tlnp | grep -q ":${SSH_PORT} "; then
    echo -e "${RED}❌ SSH no está corriendo en puerto ${SSH_PORT}${RESET}"
    echo -e "${YELLOW}   Inicia sshd primero: sudo systemctl start sshd${RESET}"
    exit 1
fi
echo -e "${GREEN}✅ SSH activo en puerto ${SSH_PORT}${RESET}"

# ─── Verificar cloudflared ───
if ! command -v cloudflared &>/dev/null; then
    echo -e "${RED}❌ cloudflared no encontrado${RESET}"
    exit 1
fi
echo -e "${GREEN}✅ cloudflared disponible${RESET}"

# ─── Configurar sshd para permitir TCP forwarding ───
# El hermano hará: ssh -R ${TUNNEL_PORT}:localhost:22 nexus@<tunnel_url>
# Necesitamos GatewayPorts para que bindee en 0.0.0.0

echo ""
echo -e "${CYAN}🔧 Preparando SSH para conexión reversa...${RESET}"

# Asegurar que GatewayPorts está activo
if ! grep -q "GatewayPorts yes" /etc/ssh/sshd_config 2>/dev/null; then
    echo -e "${YELLOW}⚠️  GatewayPorts no está configurado. Se necesita para bind remoto.${RESET}"
    echo -e "${YELLOW}   El reverse SSH bindeará solo en localhost (accesible desde NEXUS igualmente)${RESET}"
fi

# ─── Obtener usuario SOBERANO ───
SOBERANO_USER=$(whoami)
echo -e "${GREEN}✅ Usuario NEXUS: ${SOBERANO_USER}${RESET}"

# ─── Puerto para cloudflared (donde escuchará el relay) ───
RELAY_PORT=4450

echo ""
echo -e "${CYAN}🌐 Iniciando túnel Cloudflare para relay SSH...${RESET}"
echo -e "${YELLOW}   El hermano se conectará a este túnel desde su Arch${RESET}"
echo ""
echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}📋 INSTRUCCIONES PARA EL HERMANO:${RESET}"
echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "1. Ejecutar en su Arch Linux:"
echo -e "${CYAN}   ssh -R ${TUNNEL_PORT}:localhost:22 ${SOBERANO_USER}@<URL_DEL_TUNEL> -o StrictHostKeyChecking=no${RESET}"
echo ""
echo -e "2. Ingresar su CONTRASEÑA de ${SOBERANO_USER} cuando pida"
echo ""
echo -e "3. Una vez conectado, desde AQUÍ ejecutas:"
echo -e "${CYAN}   ssh ${SOBERANO_USER}@localhost -p ${TUNNEL_PORT}${RESET}"
echo ""
echo -e "4. Te pedirá la misma contraseña → tienes shell en su PC ✓"
echo ""
echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}⚠️  Alternativa: con clave SSH no necesita contraseña cada vez${RESET}"
echo ""

# ─── Generar URL del túnel ───
echo -e "${CYAN}🚀 Iniciando cloudflared tunnel...${RESET}"
echo -e "${YELLOW}   (Esperando URL pública...)${RESET}"
echo ""

# Usar --url apunta a un servicio HTTP local para obtener URL
# Luego usamos esa URL para SSH
# cloudflared access tcp --hostname <url> --url localhost:22  (no es directo)

# Estrategia: cloudflared tunnel + -- ingress rule for TCP
# Pero la más práctica: cloudflared access tcp

# En realidad, para SSH puro con cloudflared necesitamos:
# 1. Un túnel HTTP que exponga un servicio que muestre la URL
# 2. Luego usar cloudflared access tcp-gen ... 

# La solución más limpia: exponer el puerto SSH directamente con cloudflared tunnel

# Primero obtener un subdomain temporal via quick tunnel
# cloudflared tunnel --url ssh://localhost:22  (no funciona, solo HTTP)

# Solución REAL: usar cloudflared access tcp
# Pero eso requiere tener un dominio configurado en Cloudflare.

echo -e "${YELLOW}⚠️  Cloudflare TCP tunnels requieren dominio configurado.${RESET}"
echo ""
echo -e "${YELLOW}📌 ESTRATEGIA ALTERNATIVA MÁS ROBUSTA:${RESET}"
echo -e "${YELLOW}   Usar un script Node.js que haga bridge WebSocket ↔ TCP${RESET}"
echo -e "${YELLOW}   El hermano se conecta vía WebSocket, NEXUS recibe TCP${RESET}"
echo ""
echo -e "${CYAN}🔧 Creando bridge WebSocket→TCP (ws-relay)...${RESET}"

# ─── Crear el relay WebSocket para SSH ───
cat > /tmp/nexus_ws_relay.js << 'EOF'
#!/usr/bin/env node
// 🔱 NEXUS — WebSocket ↔ TCP Bridge para SSH remoto del hermano
const http = require('http');
const net = require('net');
const { WebSocketServer } = require('ws');

const RELAY_PORT = 4450;
const SSH_HOST = '127.0.0.1';
const SSH_PORT = 22;

// Verificar si ws está disponible
let ws;
try {
    ws = require('ws');
} catch (e) {
    console.error('❌ Módulo "ws" no encontrado. Instala: npm install ws');
    process.exit(1);
}

const server = http.createServer((req, res) => {
    res.writeHead(200, { 'Content-Type': 'text/html' });
    res.end(`
    <!DOCTYPE html>
    <html>
    <head><title>🔱 NEXUS Relay</title></head>
    <body style="background:#0a0a0a;color:#00ff88;font-family:monospace;padding:40px;">
      <h1>🔱 NEXUS — SSH Relay Soberano</h1>
      <p>Servicio de túnel activo.</p>
      <p>Conecta via WebSocket a <code>ws://${req.headers.host}</code></p>
      <hr>
      <p><small>Este relay permite control remoto de terminal.</small></p>
    </body>
    </html>
    `);
});

const wss = new WebSocketServer({ server });

wss.on('connection', (ws, req) => {
    const clientIp = req.socket.remoteAddress;
    console.log(`[CONEXIÓN] Cliente conectado desde ${clientIp}`);

    // Conectar al SSH local
    const tcpSocket = net.createConnection(SSH_PORT, SSH_HOST, () => {
        console.log(`[SSH] Conectado a ${SSH_HOST}:${SSH_PORT}`);
    });

    // WebSocket → TCP (cliente hermano → NEXUS SSH)
    ws.on('message', (data) => {
        if (Buffer.isBuffer(data) || typeof data === 'string') {
            tcpSocket.write(data);
        } else if (data instanceof Buffer) {
            tcpSocket.write(data);
        }
    });

    // TCP → WebSocket (NEXUS SSH → cliente hermano)
    tcpSocket.on('data', (data) => {
        if (ws.readyState === ws.OPEN) {
            ws.send(data);
        }
    });

    // Manejo de cierres
    ws.on('close', () => {
        console.log(`[WS] Cliente ${clientIp} desconectado`);
        tcpSocket.end();
    });

    tcpSocket.on('close', () => {
        ws.close();
    });

    tcpSocket.on('error', (err) => {
        console.error(`[TCP Error] ${err.message}`);
        ws.close();
    });

    ws.on('error', (err) => {
        console.error(`[WS Error] ${err.message}`);
        tcpSocket.end();
    });
});

server.listen(RELAY_PORT, '127.0.0.1', () => {
    console.log(`\x1b[32m`);
    console.log(`╔═══════════════════════════════════════════════════╗`);
    console.log(`║  🔱 NEXUS — Relay SSH Soberano                  ║`);
    console.log(`╠═══════════════════════════════════════════════════╣`);
    console.log(`║  Relay local:   ws://127.0.0.1:${RELAY_PORT}           ║`);
    console.log(`║  SSH target:    ${SSH_HOST}:${SSH_PORT}                    ║`);
    console.log(`╚═══════════════════════════════════════════════════╝`);
    console.log(`\x1b[0m`);
    console.log(`\x1b[33m⬆️  Ahora expón este relay con cloudflared:\x1b[0m`);
    console.log(`\x1b[36m   cloudflared tunnel --url http://127.0.0.1:${RELAY_PORT}\x1b[0m`);
    console.log(``);
});
EOF

echo -e "${GREEN}✅ Relay script creado en /tmp/nexus_ws_relay.js${RESET}"

# ─── Iniciar relay ───
echo ""
echo -e "${CYAN}🚀 Iniciando relay WebSocket→TCP...${RESET}"

# Verificar que el puerto no esté ocupado
if ss -tlnp | grep -q ":${RELAY_PORT} "; then
    echo -e "${YELLOW}⚠️  Puerto ${RELAY_PORT} ocupado. Matando proceso anterior...${RESET}"
    fuser -k ${RELAY_PORT}/tcp 2>/dev/null
    sleep 1
fi

cd /home/soberano/NEXUS_ULTIMATE_CORE
node /tmp/nexus_ws_relay.js &
RELAY_PID=$!

sleep 2

# Verificar que el relay inició
if kill -0 $RELAY_PID 2>/dev/null; then
    echo -e "${GREEN}✅ Relay activo en ws://127.0.0.1:${RELAY_PORT} (PID: ${RELAY_PID})${RESET}"
else
    echo -e "${RED}❌ Relay no pudo iniciar${RESET}"
    exit 1
fi

echo ""
echo -e "${CYAN}🌐 Ahora exponiendo el relay via Cloudflare Tunnel...${RESET}"
echo -e "${YELLOW}   El link HTTPS permitirá al hermano conectar su SSH${RESET}"
echo ""

# ─── Cloudflare Tunnel ───
cloudflared tunnel --url "http://127.0.0.1:${RELAY_PORT}" --no-autoupdate 2>&1 | grep --line-buffered -E "trycloudflare|https://|INF|conn|error" | while IFS= read -r line; do
    if echo "$line" | grep -q "trycloudflare\|https://"; then
        URL=$(echo "$line" | grep -oP 'https://[a-z0-9\-]+\.trycloudflare\.com')
        if [ -n "$URL" ]; then
            echo ""
            echo -e "${GREEN}╔══════════════════════════════════════════════════════════╗${RESET}"
            echo -e "${GREEN}║  🔗 TÚNEL ACTIVO — URL PARA EL HERMANO:                 ║${RESET}"
            echo -e "${CYAN}║  ${URL}${RESET}"
            echo -e "${GREEN}╚══════════════════════════════════════════════════════════╝${RESET}"
            echo ""
            echo -e "${YELLOW}📋 INSTRUCCIONES PARA EL HERMANO (Arch Linux):${RESET}"
            echo ""
            echo -e "   Su PC debe tener npm/node (o instalarlo):"
            echo -e "   ${CYAN}   sudo pacman -S nodejs npm${RESET}"
            echo ""
            echo -e "   Luego ejecutar el script de conexión:"
            echo -e "   ${CYAN}   curl -sL <URL_DEL_SCRIPT> | bash${RESET}"
            echo ""
            echo -e "   O manualmente:"
            echo -e "   ${CYAN}   npm install -g wscat${RESET}"
            echo -e "   ${CYAN}   wscat -c ${URL}${RESET}"
            echo ""
            echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo -e "${GREEN}📋 PARA TI (Arquitecto):${RESET}"
            echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo ""
            echo -e "   Una vez él conecte, abre OTRA terminal y ejecuta:"
            echo -e "   ${CYAN}   ssh <usuario>@localhost -p 2222${RESET}"
            echo ""
            echo -e "   Donde <usuario> es su usuario en Arch Linux"
            echo ""
            echo -e "${YELLOW}⚠️  Recomendación: configurar clave SSH para no depender de contraseña${RESET}"
            echo ""
            echo -e "${YELLOW}   Presiona Ctrl+C para cerrar el túnel${RESET}"
            echo ""
        fi
    else
        echo -e "${CYAN}  $line${RESET}"
    fi
done

# Cleanup al salir
kill $RELAY_PID 2>/dev/null
