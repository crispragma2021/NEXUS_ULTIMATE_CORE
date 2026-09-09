#!/bin/bash
# 🔱 Lanzador Seguro de NEXUS Santuario en Tauri
# Previene colisiones de puertos y arranques duplicados.

BASE_DIR="/home/soberano/NEXUS_ULTIMATE_CORE"
PORT_FRONT=1420
PORT_BACK=43210

echo "🧹 [SANTUARIO] Limpiando procesos previos en puertos $PORT_FRONT y $PORT_BACK..."

# Encontrar y matar cualquier proceso escuchando en el puerto 1420 (Vite)
PID_FRONT=$(lsof -t -i:$PORT_FRONT)
if [ -n "$PID_FRONT" ]; then
    echo "⚠️  Detectado proceso residual en puerto $PORT_FRONT (PID: $PID_FRONT). Matando..."
    kill -9 $PID_FRONT 2>/dev/null
fi

# Encontrar y matar cualquier proceso escuchando en el puerto 43210 (Backend REST API)
PID_BACK=$(lsof -t -i:$PORT_BACK)
if [ -n "$PID_BACK" ]; then
    echo "⚠️  Detectado proceso residual en puerto $PORT_BACK (PID: $PID_BACK). Matando..."
    kill -9 $PID_BACK 2>/dev/null
fi

# Purgar procesos de tauri duplicados
pkill -9 -f "cargo-tauri" || true
pkill -9 -f "nexus-ui" || true

sleep 1

echo "🚀 [SANTUARIO] Puertos liberados. Iniciando Tauri..."

cd "$BASE_DIR"
export DISPLAY=${DISPLAY:-:0}
export WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-wayland-0}
export XDG_RUNTIME_DIR=${XDG_RUNTIME_DIR:-/run/user/1000}
export NEXUS_ROOT="$BASE_DIR"
export VITE_DEV_SERVER_URL="http://localhost:${PORT_FRONT}"

# Iniciar tauri dev
cargo tauri dev

