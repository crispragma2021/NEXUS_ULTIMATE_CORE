#!/bin/bash
# ============================================================
# 🔱 NEXUS — PANTALLA VIRTUAL SOBERANA
# Tu hermano ve NEXUS como escritorio virtual en su browser
# No puede copiar archivos — solo recibe píxeles
# ============================================================

GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
RESET='\033[0m'

# ─── Configuración ───
DISPLAY_NUM=":99"          # Display virtual oculto
VNC_PORT=5999              # Puerto VNC interno
NOVNC_PORT=6080            # Puerto que ve el hermano en browser
NEXUS_DIR="/home/soberano/NEXUS_ULTIMATE_CORE"
YOUR_IP="192.168.0.101"
VNC_PASSWORD="nexus_virt"  # Contraseña VNC (cambiar si quieres)

echo -e "${CYAN}"
echo "  ╔══════════════════════════════════════════╗"
echo "  ║   🔱 NEXUS — PANTALLA VIRTUAL SOBERANA  ║"
echo "  ║   Tu hermano ve pixels — no archivos     ║"
echo "  ╚══════════════════════════════════════════╝"
echo -e "${RESET}"

# ─── 1. Limpiar procesos anteriores ───
echo -e "${YELLOW}[1/5] Limpiando procesos anteriores...${RESET}"
pkill -f "Xvfb ${DISPLAY_NUM}" 2>/dev/null
pkill -f "x11vnc.*${VNC_PORT}" 2>/dev/null
pkill -f "websockify.*${NOVNC_PORT}" 2>/dev/null
sleep 1

# ─── 2. Pantalla virtual Xvfb ───
echo -e "${YELLOW}[2/5] Iniciando pantalla virtual (display ${DISPLAY_NUM})...${RESET}"
Xvfb ${DISPLAY_NUM} -screen 0 1280x800x24 -ac &
XVFB_PID=$!
sleep 2

if ! kill -0 $XVFB_PID 2>/dev/null; then
    echo -e "${RED}❌ Error iniciando Xvfb${RESET}"
    exit 1
fi
echo -e "${GREEN}   ✓ Pantalla virtual activa (1280x800)${RESET}"

# ─── 3. Abrir NEXUS en la pantalla virtual ───
echo -e "${YELLOW}[3/5] Lanzando NEXUS en pantalla virtual...${RESET}"
DISPLAY=${DISPLAY_NUM} \
WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    chromium-browser --no-sandbox \
    --app="http://localhost:1420" \
    --window-size=1280,800 \
    --window-position=0,0 \
    --disable-infobars \
    --kiosk \
    2>/dev/null &
# Si no tiene chromium, intenta con firefox
if [ $? -ne 0 ]; then
    DISPLAY=${DISPLAY_NUM} firefox --kiosk "http://localhost:1420" 2>/dev/null &
fi
sleep 3
echo -e "${GREEN}   ✓ NEXUS visible en pantalla virtual${RESET}"

# ─── 4. Servidor VNC que captura la pantalla ───
echo -e "${YELLOW}[4/5] Iniciando captura VNC (sin clipboard = no puede copiar)...${RESET}"

# Crear contraseña VNC
mkdir -p ~/.vnc
x11vnc -storepasswd "${VNC_PASSWORD}" ~/.vnc/passwd 2>/dev/null

x11vnc \
    -display ${DISPLAY_NUM} \
    -rfbport ${VNC_PORT} \
    -rfbauth ~/.vnc/passwd \
    -noclipboard \          # ← Desactiva clipboard compartido
    -nocursorshape \
    -forever \
    -quiet \
    2>/dev/null &
VNC_PID=$!
sleep 2
echo -e "${GREEN}   ✓ VNC activo en puerto ${VNC_PORT} (clipboard desactivado)${RESET}"

# ─── 5. noVNC — convierte VNC a browser ───
echo -e "${YELLOW}[5/5] Iniciando noVNC (interfaz web)...${RESET}"
websockify \
    --web /usr/share/novnc \
    ${NOVNC_PORT} \
    localhost:${VNC_PORT} \
    2>/dev/null &
NOVNC_PID=$!
sleep 2
echo -e "${GREEN}   ✓ noVNC activo${RESET}"

# ─── Resultado ───
echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════════╗${RESET}"
echo -e "${GREEN}║  🖥️  PANTALLA VIRTUAL SOBERANA — ACTIVA               ║${RESET}"
echo -e "${GREEN}╠═══════════════════════════════════════════════════════╣${RESET}"
echo -e "${GREEN}║  🔗 Link para tu hermano:                             ║${RESET}"
echo -e "${CYAN}║  http://${YOUR_IP}:${NOVNC_PORT}/vnc.html?autoconnect=1&resize=scale ║${RESET}"
echo -e "${GREEN}║  🔑 Contraseña VNC: ${VNC_PASSWORD}                         ║${RESET}"
echo -e "${GREEN}╠═══════════════════════════════════════════════════════╣${RESET}"
echo -e "${GREEN}║  🛡️  Protecciones activas:                             ║${RESET}"
echo -e "${GREEN}║     ✓ Clipboard desactivado (no puede copiar/pegar)  ║${RESET}"
echo -e "${GREEN}║     ✓ Solo ve píxeles — no archivos                  ║${RESET}"
echo -e "${GREEN}║     ✓ NEXUS corre 100% en tu PC                      ║${RESET}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════╝${RESET}"
echo ""
echo -e "${YELLOW}  Ctrl+C para apagar la pantalla virtual y revocar acceso${RESET}"
echo ""

# Mantener vivo
wait $XVFB_PID
