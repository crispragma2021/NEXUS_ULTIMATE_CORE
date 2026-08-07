#!/bin/bash
# NEXUS GHOST ENGINE - IGNITION SCRIPT
# Protocolo OMEGA - Ryzen 7 Optimized

ENGINE_DIR="/home/soberano/NEXUS_ULTIMATE_CORE/mcp_arsenal/nexus_ghost_engine"

echo "🧪 [NEXUS] Iniciando Secuencia de Transfixión Digital..."

cd "$ENGINE_DIR" || exit

# Verificar si el búnker tiene Docker
if ! command -v docker-compose &> /dev/null
then
    echo "⚠️ [ALERTA] docker-compose no detectado. Intentando con 'docker compose'..."
    DOCKER_CMD="docker compose"
else
    DOCKER_CMD="docker-compose"
fi

# Desplegar el Velo (Tor + Vision)
echo "🕸️ [NEXUS] Desplegando el Velo (Shadow Network)..."
$DOCKER_CMD up -d

# Verificar estado
echo "🔍 [NEXUS] Analizando hilos de ejecución..."
sleep 5
$DOCKER_CMD ps

echo "✅ [NEXUS] GHOST_ENGINE OPERATIVO. Iniciando Absorción de Herramientas Legadas."
