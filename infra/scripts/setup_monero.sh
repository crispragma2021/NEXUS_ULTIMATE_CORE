#!/bin/bash
# NEXUS Sovereign Monero Node Installer
# Installs a pruned monerod for autonomous validation

MONERO_DIR="/home/soberano/NEXUS_ULTIMATE_CORE/infra/monero"
MONERO_DATA="/home/soberano/NEXUS_ULTIMATE_CORE/infra/monero/data"
mkdir -p $MONERO_DIR
mkdir -p $MONERO_DATA

echo "🛡️ Iniciando Despliegue de Nodo Monero Soberano..."

# 1. Descargar Binarios (Linux 64-bit)
cd $MONERO_DIR
if [ ! -f "monerod" ]; then
    echo "📥 Descargando binarios de Monero..."
    curl -L https://downloads.getmonero.org/cli/linux64 -o monero.tar.bz2
    tar -xjvf monero.tar.bz2 --strip-components=1
    rm monero.tar.bz2
fi

# 2. Configurar Servicio (Nohup por ahora, luego systemd si se requiere)
# Usamos --prune-blockchain para ahorrar espacio (aprox 60GB vs 180GB)
# Usamos --public-node para ayudar a la red si se desea, pero aquí es privado
echo "🚀 Lanzando monerod en modo PRUNED..."
nohup ./monerod --data-dir $MONERO_DATA --prune-blockchain --non-interactive --rpc-bind-ip 127.0.0.1 --confirm-external-bind > $MONERO_DIR/monero.log 2>&1 &

echo "✅ Nodo Monero iniciado en segundo plano."
echo "Logs disponibles en: $MONERO_DIR/monero.log"
echo "Nota: La sincronización inicial puede tardar días dependiendo de la conexión."
