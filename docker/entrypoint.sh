#!/bin/bash
# 🔱 entrypoint.sh - NEXUS Headless Xvfb Initializer
set -e

echo "🚀 Iniciando Servidor Virtual Xvfb (${RESOLUTION})..."
Xvfb :99 -screen 0 ${RESOLUTION} -ac +extension RANDR &
XVFB_PID=$!

# Esperar a que el servidor X virtual esté listo
sleep 2

echo "🖥️ Iniciando Fluxbox (gestor de ventanas ligero)..."
fluxbox &

# Ejecutar el comando especificado en el contenedor o mantener vivo
if [ "$#" -eq 0 ]; then
    echo "🟢 Contenedor listo en modo Headless. Manteniendo vivo..."
    tail -f /dev/null
else
    echo "⚡ Ejecutando comando: $@"
    exec "$@"
fi
