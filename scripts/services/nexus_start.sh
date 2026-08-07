#!/bin/bash
# 🔱 [NEXUS OMEGA STARTUP SCRIPT]
# Inyectando variables de entorno gráfico para Visión Multimodal

export DISPLAY=:1
export WAYLAND_DISPLAY=wayland-0
export XDG_RUNTIME_DIR=/run/user/1000

echo "👁️ [VISION] Inyectando DISPLAY=$DISPLAY y WAYLAND_DISPLAY=$WAYLAND_DISPLAY"

# Resolve binary path dynamically relative to the script location
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
BINARY="$SCRIPT_DIR/../target/release/nexus-ui"
if [ ! -f "$BINARY" ]; then
    BINARY="/home/soberano/.cargo-target/release/nexus-ui"
fi
if [ ! -f "$BINARY" ]; then
    BINARY="/home/soberano/.cargo-target/debug/nexus-ui"
fi
if [ ! -f "$BINARY" ]; then
    BINARY="$SCRIPT_DIR/../.cargo-cache/release/nexus-ui"
fi
if [ ! -f "$BINARY" ]; then
    BINARY="$SCRIPT_DIR/../.cargo-cache/debug/nexus-ui"
fi

if [ ! -f "$BINARY" ]; then
    echo "🚨 [ERROR] Orquestador no compilado. Ejecuta: cargo build"
    exit 1
fi

chmod +x "$BINARY"
echo "🧹 [NEXUS] Liberando instancias residuales en puerto 43210..."
# Encontrar y matar procesos escuchando en el puerto 43210 (REST API)
fuser -k 43210/tcp >/dev/null 2>&1 || true
pkill -9 -f "nexus-ui" || true
sleep 1

echo "🚀 [NEXUS] Iniciando núcleo Omega..."
exec "$BINARY" --headless
