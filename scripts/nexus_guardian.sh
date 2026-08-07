#!/bin/bash

# NEXUS GUARDIAN OMEGA (TUNEL CENTINELA)
# Monitor soberano para la conectividad y el puente Cloudflare
# Autor: NEXUS (Simbionte)

PROJECT_DIR="/home/soberano/NEXUS_ULTIMATE_CORE"

echo "🛡️ NEXUS Guardian Centinela de Conectividad Inyectado."
echo "📍 Vigilando túneles de red en $PROJECT_DIR"

while true; do
    # 🔍 1. CHEQUEO DEL TÚNEL CLOUDFLARE
    if ! pgrep -x "cloudflared" > /dev/null; then
        echo "🌐 [ALERTA] Tunel Cloudflare Caído. Reiniciando puente..."
        cloudflared tunnel run --no-autoupdate f7c6c507-724d-444f-831d-ec067cdbd887 > "$PROJECT_DIR/logs/cloudflare.log" 2>&1 &
    fi

    sleep 20
done
