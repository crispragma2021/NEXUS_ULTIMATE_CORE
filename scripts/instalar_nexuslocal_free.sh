#!/usr/bin/env bash
# ==============================================================================
# 🧬 Instala NEXUSLOCAL-FREE (Qwen 2.5 abliterated sin censura) en Ollama (E0)
# ==============================================================================
# Uso: bash scripts/instalar_nexuslocal_free.sh
# Requisitos: Ollama instalado y en ejecución.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODEL_FILE="$SCRIPT_DIR/../Modelfile.nexuslocal-free"
TAG="nexuslocal-free:latest"
BASE="huihui_ai/qwen2.5-abliterated:7b"

echo "=== [E0] NEXUSLOCAL-FREE: Qwen sin censura ==="

# 1. Verificar Ollama
if ! command -v ollama >/dev/null 2>&1; then
    echo "[!] Ollama no está instalado. Instálalo primero: curl -fsSL https://ollama.com/install.sh | sh"
    exit 1
fi

# 2. Descargar modelo base abliterated si no está presente
if ! ollama list | grep -q "huihui_ai/qwen2.5-abliterated:7b"; then
    echo "=== Descargando modelo base abliterated (~4.8GB) ==="
    ollama pull "$BASE"
else
    echo "=== Modelo base ya presente ==="
fi

# 3. Crear el modelo con el Modelfile
echo "=== Creando $TAG desde $MODEL_FILE ==="
ollama create "$TAG" -f "$MODEL_FILE"

# 4. Verificación
echo ""
echo "=== Modelos disponibles ==="
ollama list | grep -i "nexuslocal\|abliterated" || true
echo ""
echo "=== Prueba rápida ==="
ollama run "$TAG" "Responde SOLO con un JSON: {\"estado\": \"operativo\"}"
echo ""
echo "[OK] NEXUSLOCAL-FREE instalado. Usa: ollama run $TAG"
