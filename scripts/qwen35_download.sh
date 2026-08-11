#!/bin/bash
# ==============================================================================
# Descarga Qwen3.6-35B-A3B Uncensored (HauhauCS) IQ4_NL + mmproj de visión
# Fuente: Hugging Face - HauhauCS/Qwen3.6-35B-A3B-Uncensored-HauhauCS-Aggressive
# Con reanudación (-c) para tolerar cortes en ~20GB.
# ==============================================================================
set -u

BASE="https://huggingface.co/HauhauCS/Qwen3.6-35B-A3B-Uncensored-HauhauCS-Aggressive/resolve/main"
DEST="/home/soberano/qwen35_models"
mkdir -p "$DEST"

MODEL="Qwen3.6-35B-A3B-Uncensored-HauhauCS-Aggressive-IQ4_NL.gguf"
VISION="mmproj-Qwen3.6-35B-A3B-Uncensored-HauhauCS-Aggressive-f16.gguf"

log(){ echo "[$(date '+%H:%M:%S')] $*"; }

# 1. Descargar el projector de visión (0.9GB) - rápido, primero
if [ -f "$DEST/$VISION" ] && [ $(stat -c%s "$DEST/$VISION") -gt 800000000 ]; then
    log "VISION ya descargado: $VISION"
else
    log "Descargando VISION ($VISION)..."
    wget -c -q --show-progress -O "$DEST/$VISION.part" "$BASE/$VISION" && mv "$DEST/$VISION.part" "$DEST/$VISION"
    log "VISION descargado. Tamaño: $(stat -c%s "$DEST/$VISION") bytes"
fi

# 2. Descargar el modelo principal (19.7GB)
if [ -f "$DEST/$MODEL" ] && [ $(stat -c%s "$DEST/$MODEL") -gt 19700000000 ]; then
    log "MODELO ya descargado: $MODEL"
else
    log "Descargando MODELO ($MODEL) ~19.7GB..."
    wget -c -q --show-progress -O "$DEST/$MODEL.part" "$BASE/$MODEL" && mv "$DEST/$MODEL.part" "$DEST/$MODEL"
    log "MODELO descargado. Tamaño: $(stat -c%s "$DEST/$MODEL") bytes"
fi

log "=== DESCARGA COMPLETADA ==="
ls -lh "$DEST"
