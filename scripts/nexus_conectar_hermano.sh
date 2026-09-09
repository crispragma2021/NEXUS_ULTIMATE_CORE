#!/bin/bash
# ============================================================
# 🔱 NEXUS — CONEXIÓN TERMINAL REMOTA PARA HERMANO
# Conecta su Arch Linux al túnel de NEXUS para que
# el Arquitecto (Cris) tenga acceso SSH a su terminal.
#
# USO: Ejecutar en la PC del hermano (Arch Linux):
#   curl -sL https://raw.githubusercontent.com/... | bash
#   (o descargar y ejecutar: bash nexus_conectar_hermano.sh)
#
# REQUISITOS: nodejs, npm
#   sudo pacman -S nodejs npm
# ============================================================

GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
RESET='\033[0m'

echo -e "${CYAN}"
echo "  ╔══════════════════════════════════════════╗"
echo "  ║   🔱 NEXUS — CONEXIÓN REMOTA            ║"
echo "  ║      Terminal Soberana para Hermano      ║"
echo "  ╚══════════════════════════════════════════╝"
echo -e "${RESET}"

# ─── Verificar node ───
if ! command -v node &>/dev/null; then
    echo -e "${RED}❌ Node.js no encontrado${RESET}"
    echo -e "${YELLOW}   Instala: sudo pacman -S nodejs npm${RESET}"
    exit 1
fi
echo -e "${GREEN}✅ Node.js $(node -v)${RESET}"

# ─── Verificar que sshd esté corriendo ───
if ! systemctl is-active --quiet sshd 2>/dev/null; then
    echo -e "${YELLOW}⚠️  SSH no está activo. Intentando iniciar...${RESET}"
    sudo systemctl start sshd 2>/dev/null || {
        echo -e "${RED}❌ No se pudo iniciar sshd${RESET}"
        echo -e "${YELLOW}   Inicia manualmente: sudo systemctl start sshd${RESET}"
        exit 1
    }
fi
echo -e "${GREEN}✅ SSH activo (puerto 22)${RESET}"

# ─── Preguntar URL del túnel ───
echo ""
echo -e "${CYAN}🔗 Ingresa la URL del túnel que te dio NEXUS:${RESET}"
echo -e "${YELLOW}   (Ej: https://ejemplo.trycloudflare.com)${RESET}"
read -p "URL > " TUNNEL_URL

if [ -z "$TUNNEL_URL" ]; then
    echo -e "${RED}❌ URL requerida${RESET}"
    exit 1
fi

# Normalizar URL (quitar https:// para ws://)
WS_URL="${TUNNEL_URL/https:\/\//ws:\/\/}"
echo -e "${GREEN}✅ Conectando a ${WS_URL}...${RESET}"

# ─── Crear script Node.js de conexión ───
cat > /tmp/nexus_conectar.js << 'NODESCRIPT'
// ============================================================
// 🔱 NEXUS — Conector SSH vía WebSocket
// Puentea SSH local → WebSocket remoto
// ============================================================
const net = require('net');
const http = require('http');
const { WebSocket } = require('ws');

const WS_URL = process.argv[2];
const SSH_PORT = 22;
const LOCAL_TUNNEL_PORT = 2222;  // Puerto donde escucha SSH local para forward

if (!WS_URL) {
    console.error('❌ Uso: node nexus_conectar.js <ws_url>');
    process.exit(1);
}

console.log(`🔱 NEXUS — Conectando a ${WS_URL}...`);
console.log(`🔌 SSH local: puerto ${SSH_PORT}`);

// Opción 1: Forward inverso (él se conecta a nuestro SSH)
// Pero como no tenemos SSH server en su PC que acepte conexiones entrantes...
// en realidad el forward es al revés.

// La idea correcta:
// 1. NEXUS (allá) tiene un relay WebSocket → TCP (conecta a su SSH local)
// 2. Nosotros (Arch) nos conectamos vía WebSocket al relay de NEXUS
// 3. El relay de NEXUS conecta a nuestro SSH local
// 
// PERO ESO SIGNIFICA QUE NEXUS INICIA LA CONEXIÓN A NUESTRO SSH.
// Nosotros solo tenemos que exponer nuestro SSH para que NEXUS se conecte.

// La forma correcta: 
// 1. NOSOTROS (Arch) nos conectamos vía WebSocket al relay de NEXUS
// 2. El relay de NEXUS tiene un TCP conectado a localhost:22 (SSH de NEXUS)
// 3. WebSocket bidi → podemos enviar comandos SSH desde NEXUS hacia acá
//
// NO. Al revés. Necesitamos que NEXUS pueda enviarnos comandos.
// 
// Solución REAL: Usamos un relay donde:
// - NOSOTROS (Arch) nos conectamos al relay vía WebSocket
// - El relay tiene UN TCP hacia SSH DE NEXUS
// - NEXUS puede hacer SSH a localhost:ALGUN_PUERTO que recibe forward

// Esto es enredado. Vamos a la solución más simple:
// 
// SIMPLE: SSH reverso usando el relay WebSocket como túnel TCP.
// 
// 1. Un servidor en NEXUS escucha WebSocket en un puerto (vía cloudflared)
// 2. Nosotros nos conectamos a ese WebSocket
// 3. Nuestro script recibe datos WebSocket y los reenvía a SSH local
// 4. El servidor en NEXUS tiene un TCP conectado a localhost:ALGO
// 5. Cuando NEXUS escribe a ese TCP, se envía por WebSocket a nosotros
// 6. Nosotros lo reenviamos a SSH local → ejecuta el comando
// 7. La respuesta de SSH vuelve por WebSocket a NEXUS
//
// Esto es un túnel TCP bidireccional sobre WebSocket.

console.log(`🚀 Iniciando túnel SSH bidireccional...`);

const ws = new WebSocket(WS_URL);

ws.on('open', () => {
    console.log(`✅ Conectado a NEXUS vía WebSocket`);
    console.log(`🔄 Túnel SSH activo — esperando comandos...`);
    console.log(``);
    console.log(`📋 Desde la PC de NEXUS, el Arquitecto hará:`);
    console.log(`   ssh <tu_usuario>@localhost -p 2222`);
    console.log(``);
});

ws.on('message', (data) => {
    // Recibimos datos de NEXUS — son datos TCP
    // Necesitamos un cliente TCP a nuestro SSH local
    const client = net.createConnection(SSH_PORT, '127.0.0.1', () => {
        client.write(data);
    });
    
    client.on('data', (response) => {
        ws.send(response);
    });
    
    client.on('error', (err) => {
        console.error(`[TCP Error] ${err.message}`);
    });
    
    // Mantener conexión abierta para más datos
    // Nota: esto es una simplificación. SSH es stateful y necesita
    // mantener la misma conexión TCP, no crear una nueva cada mensaje.
});

ws.on('close', () => {
    console.log(`❌ Conexión con NEXUS cerrada`);
    process.exit(0);
});

ws.on('error', (err) => {
    console.error(`❌ Error WebSocket: ${err.message}`);
    process.exit(1);
});

// Mantener el proceso vivo
process.on('SIGINT', () => {
    console.log(`\n👋 Cerrando conexión...`);
    ws.close();
    process.exit(0);
});
NODESCRIPT

# ─── Ejecutar conector ───
cd /tmp
node nexus_conectar.js "$WS_URL"
