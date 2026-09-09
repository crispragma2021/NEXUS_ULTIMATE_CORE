#!/bin/bash
# NEXUS SOVEREIGN GATEWAY - Terminal Communication Channel
# Uso: ./nexus_chat.sh "Hola Nexus"

COMMAND=$1
if [ -z "$COMMAND" ]; then
    echo "🔱 [NEXUS L1] Por favor, ingresa un comando."
    exit 1
fi

curl -X POST http://localhost:4444/api/santuario/chat \
     -H "Content-Type: application/json" \
     -d "{\"command\":\"$COMMAND\"}"

echo ""
