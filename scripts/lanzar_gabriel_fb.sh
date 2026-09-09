#!/bin/bash
# /home/soberano/NEXUS_ULTIMATE_CORE/scripts/lanzar_gabriel_fb.sh
# 🔱 NEXUS OMEGA - Identidad Gabriel (Facebook Aislado)

DATA_DIR="/home/soberano/NEXUS_ULTIMATE_CORE/data/gabriel_profile"

mkdir -p "$DATA_DIR"

echo "🔐 Lanzando Facebook Gabriel (Instancia Aislada)..."

# Usamos Brave por su escudo de privacidad nativo
# --app: lanza como aplicación sin barras de navegación para una experiencia inmersiva
# --user-data-dir: asegura el aislamiento total de tus cookies personales
/snap/bin/brave --app=https://www.facebook.com --user-data-dir="$DATA_DIR" --name="Gabriel_FB_NEXUS" &

echo "✅ Facebook Gabriel está en línea en una ventana independiente."
