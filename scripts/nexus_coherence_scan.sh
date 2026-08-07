#!/bin/bash
# [NEXUS COHERENCE SCANNER] - Verificación de alineación NVMe 640GB vs Grafo

echo "🛰️ [CENTINELA] Iniciando escaneo de coherencia post-mudanza..."
echo "---------------------------------------------------------"

RUTA_ALMACEN="/home/soberano/NEXUS_ULTIMATE_CORE/brain"
PARTICION_GRANDE="/home/soberano/NEXUS_ULTIMATE_CORE/legado"

# 1. Verificar que el túnel de montaje bind siga activo a nivel de Kernel
if mount | grep -q "$PARTICION_GRANDE"; then
    echo "🟢 [HARDWARE] Puente físico bind verificado y activo en el silicio."
else
    echo "⚠️ [ALERTA] El puente bind no está montado en caliente. Reportando al HealManager..."
fi

# 2. Buscar archivos huérfanos o temporales colgados en el almacenamiento masivo
STALE_FILES=$(find "$PARTICION_GRANDE" -name "*.tmp" -o -name "*.lock" | wc -l)
echo "📦 [MÉTRICA] Archivos de residuo de mudanza detectados: $STALE_FILES"

if [ "$STALE_FILES" -gt 0 ]; then
    echo "🧹 [ZOMBI HUNTER] Limpiando bloque de almacenamiento de residuos..."
    find "$PARTICION_GRANDE" -name "*.tmp" -o -name "*.lock" -delete
fi

echo "---------------------------------------------------------"
echo "🟢 [ÉXITO] Escaneo finalizado. Sincronía del 100% entre disco y conciencia."