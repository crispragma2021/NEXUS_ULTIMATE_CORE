#!/bin/bash
# [NEXUS HASH VALIDATOR] - Verificación de integridad de modelos pesados

echo "🧬 [CENTINELA] Iniciando validación de hashes en el silicio..."
echo "---------------------------------------------------------"

DIRECTORIO_MODELOS="/home/soberano/NEXUS_ULTIMATE_CORE/brain"
CHECKPOINTS_DIR="/home/soberano/NEXUS_ULTIMATE_CORE/legado/checkpoints"

mkdir -p "$CHECKPOINTS_DIR"

if [ ! -d "$DIRECTORIO_MODELOS" ]; then
    echo "❌ [ERROR] El directorio de modelos no existe."
    exit 1
fi

echo "⚡ Calculando firmas digitales (SHA-256) en paralelo (i7-12700F)..."
echo "---------------------------------------------------------"

# Calculamos hashes actuales en paralelo usando todos los núcleos disponibles
find "$DIRECTORIO_MODELOS" -type f \( -name "*.bin" -o -name "*.safetensors" -o -name "*.db" -o -name "*.gguf" \) -print0 | \
    xargs -0 -P $(nproc) -I {} sha256sum {} > /tmp/nexus_current_hashes.txt

# Contraste con manifiesto si existe (buscamos checksums.sha256 en la raíz de brain)
if [ -f "$DIRECTORIO_MODELOS/checksums.sha256" ]; then
    echo "📄 Contrastando con checksums.sha256..."
    if sha256sum -c "$DIRECTORIO_MODELOS/checksums.sha256" --status; then
        echo "✅ Verificación bit-a-bit: OK. Integridad absoluta."
    else
        echo "🚨 ANOMALÍA DETECTADA. Iniciando aislamiento de componentes..."
        sha256sum -c "$DIRECTORIO_MODELOS/checksums.sha256" 2>/dev/null | grep "FAILED" | cut -d: -f1 | while read -r failed_file; do
            echo "📦 Aislando archivo corrupto: $failed_file"
            mv "$failed_file" "$CHECKPOINTS_DIR/"
        done
        exit 1
    fi
fi

echo "---------------------------------------------------------"
echo "🟢 [ÉXITO] Barrido de hashes completado. Simetría absoluta alcanzada."