#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# 🔱 NEXUS CUA — ENTRYPOINT DEL ENTORNO HEADLESS GUI
# Arranca Xvfb (:99) + fluxbox (WM) + x11vnc + noVNC (:6080).
# ═══════════════════════════════════════════════════════════════════════════
set -e

# Autorización X11 vacía: Xvfb corre con -ac (sin auth), pero Xlib/PyAutoGUI
# exigen la presencia del archivo. Un .Xauthority vacío evita el error.
export XAUTHORITY=/tmp/.Xauthority
touch /tmp/.Xauthority

echo "🔱 [CUA] Iniciando entorno Headless GUI (Xvfb :99)..."
Xvfb :99 -screen 0 ${RESOLUTION:-1920x1080x24} -nolisten tcp -ac &
XVFB_PID=$!
sleep 2

echo "🖥️ [CUA] Lanzando fluxbox (window manager)..."
fluxbox >/dev/null 2>&1 &
sleep 1

echo "🔒 [CUA] Configurando password VNC..."
mkdir -p ~/.vnc
x11vnc -storepasswd "${VNC_PASSWORD:-nexus}" ~/.vnc/passwd 2>/dev/null || true

echo "🖧 [CUA] Arrancando x11vnc (server VNC en :5900)..."
x11vnc -display :99 -forever -shared -rfbauth ~/.vnc/passwd -rfbport 5900 -noxdamage >/dev/null 2>&1 &
sleep 1

echo "🌐 [CUA] Arrancando noVNC en puerto ${NOVNC_PORT:-6080}..."
websockify --web /usr/share/novnc/ ${NOVNC_PORT:-6080} localhost:5900 >/dev/null 2>&1 &
sleep 2

echo "✅ [CUA] Entorno listo: noVNC en :${NOVNC_PORT:-6080} | VNC :5900 | Display :99"

# Mantener vivo (gestión de señales + trap de limpieza)
cleanup() {
    echo "🛑 [CUA] Deteniendo entorno..."
    kill $XVFB_PID 2>/dev/null || true
    exit 0
}
trap cleanup SIGTERM SIGINT

# Lanzar Chromium por defecto en el display virtual
if [ -n "${AUTOSTART_CHROMIUM:-0}" ]; then
    echo "🌐 [CUA] Autostart Chromium apuntando a ${CHROMIUM_URL:-about:blank}"
    chromium-browser --no-sandbox --disable-dev-shm-usage "${CHROMIUM_URL:-about:blank}" >/dev/null 2>&1 &
fi

wait
