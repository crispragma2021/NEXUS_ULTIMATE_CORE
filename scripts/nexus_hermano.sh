#!/bin/bash
# ============================================================
# 🔱 NEXUS — PUENTE SOBERANO PARA HERMANO
# Expone NEXUS Chat UI via Cloudflare Tunnel (HTTPS gratis)
# El hermano solo necesita el link — SIN instalar nada
# Tu código fuente NUNCA sale de esta máquina.
# ============================================================

GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
RESET='\033[0m'

# Puerto del NEXUS Santuario UI
NEXUS_PORT=1420

echo -e "${CYAN}"
echo "  ╔══════════════════════════════════════╗"
echo "  ║   🔱 NEXUS — ACCESO REMOTO OMEGA    ║"
echo "  ║      Puente Cloudflare Soberano      ║"
echo "  ╚══════════════════════════════════════╝"
echo -e "${RESET}"

# Verificar que NEXUS está corriendo
if ! curl -sf "http://localhost:${NEXUS_PORT}" > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  NEXUS Santuario no detectado en puerto ${NEXUS_PORT}${RESET}"
    echo -e "${YELLOW}   Iniciando NEXUS primero...${RESET}"
    cd /home/soberano/NEXUS_ULTIMATE_CORE
    cargo tauri dev --no-watch &
    echo -e "${YELLOW}   Esperando 15s para compilación...${RESET}"
    sleep 15
fi

echo -e "${GREEN}✅ NEXUS activo en puerto ${NEXUS_PORT}${RESET}"
echo ""
echo -e "${CYAN}🌐 Generando túnel Cloudflare seguro...${RESET}"
echo -e "${YELLOW}   (El link HTTPS aparecerá en unos segundos)${RESET}"
echo ""
echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}📋 Copia el link 'trycloudflare.com' y envíaselo a tu hermano${RESET}"
echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Tunnel Quick (sin login, gratis, link temporal)
# El link cambia cada sesión pero funciona SIN cuenta Cloudflare
cloudflared tunnel --url "http://localhost:${NEXUS_PORT}" --no-autoupdate 2>&1 | grep --line-buffered -E "trycloudflare|https://|INF" | while IFS= read -r line; do
    if echo "$line" | grep -q "trycloudflare\|https://"; then
        URL=$(echo "$line" | grep -oP 'https://[a-z0-9\-]+\.trycloudflare\.com')
        if [ -n "$URL" ]; then
            echo ""
            echo -e "${GREEN}╔══════════════════════════════════════════════════╗${RESET}"
            echo -e "${GREEN}║  🔗 LINK PARA TU HERMANO:                       ║${RESET}"
            echo -e "${CYAN}║  ${URL}${RESET}"
            echo -e "${GREEN}╚══════════════════════════════════════════════════╝${RESET}"
            echo ""
            echo -e "${YELLOW}   Tu hermano abre ese link en cualquier browser ✓${RESET}"
            echo -e "${YELLOW}   Tu código fuente está 100% protegido en tu PC  ✓${RESET}"
            echo -e "${YELLOW}   Presiona Ctrl+C para cerrar el acceso           ✓${RESET}"
            echo ""
        fi
    else
        echo -e "${CYAN}  $line${RESET}"
    fi
done
