#!/bin/bash
# /home/soberano/NEXUS_ULTIMATE_CORE/scripts/lanzar_gabriel_wa.sh
# 🔱 NEXUS OMEGA - Identidad Gabriel (WhatsApp Aislado)

DATA_DIR="/home/soberano/NEXUS_ULTIMATE_CORE/data/gabriel_profile"

mkdir -p "$DATA_DIR"

echo "🔐 Lanzando WhatsApp Gabriel (Instancia Aislada)..."

# Usamos Brave por su escudo de privacidad nativo
# --app: lanza como aplicación sin barras de navegación
# --user-data-dir: asegura que las cookies y sesión sean totalmente diferentes a las tuyas
/snap/bin/brave --app=https://web.whatsapp.com --user-data-dir="$DATA_DIR" --name="Gabriel_NEXUS" &

echo "✅ Gabriel está en línea en una ventana independiente."
