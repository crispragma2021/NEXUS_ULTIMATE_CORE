#!/bin/bash
# =====================================================================
# NEXUS OMEGA - REGENERADOR DE TOKEN VERTEX AI
# =====================================================================
# Misión: Obtener un access token fresco y actualizar el Santuario.

ENV_FILE="/home/soberano/NEXUS_ULTIMATE_CORE/.env"
echo "🛰️  Solicitando nuevo token a Google Cloud..."

NEW_TOKEN=$(gcloud auth print-access-token 2>/dev/null)

if [ -n "$NEW_TOKEN" ]; then
    # Actualización atómica en el archivo .env
    sed -i "s|^VERTEX_TOKEN=.*|VERTEX_TOKEN=$NEW_TOKEN|" "$ENV_FILE"
    echo "✅ Token actualizado en .env. Reiniciando orquestador..."
    systemctl --user restart nexus.service
else
    echo "❌ FALLO CRÍTICO: gcloud no devolvió un token. Ejecuta 'gcloud auth login'."
    exit 1
fi