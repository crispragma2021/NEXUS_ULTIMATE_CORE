#!/bin/bash
# NEXUS Core - Skill de Visión de Escritorio Local para el Agente

OUTPUT_DIR="/home/soberano/NEXUS_ULTIMATE_CORE/brain/sessions"
OUTPUT_FILE="$OUTPUT_DIR/screen_latest.png"

# Asegurar que el directorio de sesión exista
mkdir -p "$OUTPUT_DIR"

echo "[Visión] Capturando pantalla del sistema local..."

# Capturar la pantalla actual usando el servidor gráfico X11 de Ubuntu
# DISPLAY=:0 asegura que tome la pantalla principal del usuario
DISPLAY=:0 scrot -u -z "$OUTPUT_FILE" 2>/dev/null || DISPLAY=:0 scrot -z "$OUTPUT_FILE"

if [ -f "$OUTPUT_FILE" ]; then
    echo "[OK] Pantalla capturada con éxito en: $OUTPUT_FILE"
    echo "Procediendo a describir la imagen para el contexto del agente."
else
    echo "[ERROR] No se pudo acceder al servidor gráfico o capturar la pantalla."
    exit 1
fi