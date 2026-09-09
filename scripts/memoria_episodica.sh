#!/bin/bash
# ==========================================
# CONEXIÓN DE ÓRGANOS: MEMORIA EPISÓDICA
# ==========================================
BRAIN_STORAGE="$HOME/ZENITH_POOL/data/nexus_brain.cat"
SWARM_LOG="$HOME/ZENITH_POOL/data/swarm_events.log"

clear
echo "[NEXUS-CORE] Sincronizando órganos existentes..."

# Asegurar que el tejido de memoria (cat) existe
touch "$BRAIN_STORAGE"

# El Watchdog captura la salida de Gemini y el Brain la escribe en el .cat
tail -f "$SWARM_LOG" | while read -r pulso; do
    if [[ "$pulso" == *"GEMINI_OUT"* ]]; then
        # Extraer la esencia y guardarla para que el Orquestador la recuerde
        echo "$(date +'%Y-%m-%d %H:%M:%S') - $pulso" >> "$BRAIN_STORAGE"
        echo "[BRAIN] Recuerdo cristalizado en el cat."
    fi
done
