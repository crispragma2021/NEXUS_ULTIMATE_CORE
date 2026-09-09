#!/bin/bash

# NEXUS VRAM Guard - Blindaje total de VRAM para Ollama Qwen2.5-7B
# Este script mata procesos GPU innecesarios y configura variables de entorno
# para asegurar que solo Ollama utilice la VRAM dedicada.

echo "[NEXUS VRAM Guard] Iniciando blindaje de VRAM..."

# --- 1. Matar procesos GPU conocidos que puedan usar VRAM ---
# Lista de procesos comunes que usan VRAM y que no son esenciales para NEXUS
PROCESSES_TO_KILL=(
    "chromium" # Navegador web
    "firefox"  # Navegador web
    "electron" # Aplicaciones Electron (VSCode, Discord, etc.)
    "code"     # VSCode
    "discord"  # Discord
    "krita"    # Edición de imagen/arte
    "blender"  # Modelado 3D
    "gimp"     # Edición de imagen
    "inkscape" # Edición vectorial
    "steam"    # Juegos
    "nvidia-smi" # Puede estar corriendo para monitoreo, no esencial
    # "Xorg"     # Servidor X (puede ser problemático matar si es la sesión principal)
    # Aquí puedes añadir otros procesos específicos de tu entorno que sepas que usan GPU
    # "your_custom_app"
)

for proc in "${PROCESSES_TO_KILL[@]}"; do
    if pgrep -x "$proc" > /dev/null; then
        echo "[NEXUS VRAM Guard] Terminando proceso: $proc..."
        pkill -9 "$proc" # Usar -9 (SIGKILL) para asegurar la terminación
    fi
done

# --- 2. Configurar variables de entorno para Ollama y otros frameworks ---
# Estas variables aseguran que Ollama use la GPU de forma óptima
# y que otras librerías prefieran la CPU si es posible.

# Ollama: Limitar capas en GPU a ~35 para dejar ~1.5GB de VRAM libres
# (análisis de margen: NO saturar los 8GB o el driver NVIDIA expulsa a RAM
# y colapsa FPS del juego y tokens/s del modelo). Usar Flash Attention.
export OLLAMA_NUM_GPU=35
export OLLAMA_FLASH_ATTENTION=1
export OLLAMA_KV_CACHE_TYPE=q8_0
export OLLAMA_CONTEXT_LENGTH=4096

# PyTorch/TensorFlow (si usas otras librerías de ML, forzar CPU)
# Nota: Esto es un ejemplo, las variables exactas pueden variar.
export CUDA_VISIBLE_DEVICES="" # Deshabilita CUDA para otras apps
export TF_ENABLE_ONEDNN_OPTS=0 # Deshabilita optimizaciones específicas de Intel para TF si no quieres
export JAX_PLATFORM_NAME=cpu   # Para JAX

# Otros frameworks/herramientas que puedan usar GPU
# Ejemplo para FFmpeg (si usa CUDA/VAAPI)
export VDPAU_DRIVER="none" # Desactiva VDPAU
export LIBVA_DRIVER_NAME="none" # Desactiva VAAPI

echo "[NEXUS VRAM Guard] Variables de entorno configuradas."
echo "[NEXUS VRAM Guard] Blindaje de VRAM completado."
echo "[NEXUS VRAM Guard] Verificando uso de VRAM..."
nvidia-smi -q -d MEMORY | grep "Used" # Mostrar uso de VRAM después del blindaje
