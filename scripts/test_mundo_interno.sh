#!/bin/bash
# =====================================================================
# NEXUS OMEGA - TEST DE AISLAMIENTO (MUNDO INTERNO)
# =====================================================================

SOCKET="/tmp/nexus_internal_os.sock"
WORKSHOP="/home/soberano/NEXUS_ULTIMATE_CORE/workshop"

echo "🧪 [TEST] Iniciando Verificación de Aislamiento OMEGA..."

# 1. Validar Infraestructura
if [ ! -f "$WORKSHOP/vmlinux" ] || [ ! -f "$WORKSHOP/rootfs.ext4" ]; then
    echo "⚠️ Infraestructura incompleta. Ejecutando ignición primero..."
    bash /home/soberano/NEXUS_ULTIMATE_CORE/scripts/ignicion_os_interno.sh
fi

# 2. Invocar Boot (Reiniciando el servicio para disparar la lógica de Rust)
echo "🚀 Disparando boot de MicroVM a través del Orquestador..."
systemctl --user restart nexus.service

echo "⏳ Esperando materialización del socket (max 10s)..."
for i in {1..10}; do
    if [ -S "$SOCKET" ]; then
        echo "✅ SOCKET DETECTADO en $SOCKET"
        echo "📡 Consultando API de Firecracker..."
        RESPONSE=$(curl --unix-socket "$SOCKET" -i -X GET http://localhost/version 2>/dev/null | grep "Firecracker")
        echo "🧬 Identidad MicroVM: $RESPONSE"
        echo "🟢 [ESTADO] Mundo Interno: OPERATIVO"
        exit 0
    fi
    sleep 1
done

echo "❌ ERROR: Timeout. El Mundo Interno no logró arrancar."
echo "🔍 Revisa: journalctl --user -u nexus.service -f"
exit 1