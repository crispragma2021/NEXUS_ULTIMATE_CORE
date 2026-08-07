#!/bin/bash
# 🦾 NEXUS OMEGA-CLAW - Lanzador de Tareas Agénticas Autónomas
# Uso: ./scripts/nexus_agent.sh "Instrucción de la tarea" [proveedor]
# Proveedores válidos: vertex (default), google_ai_studio, deepseek-chat, deepseek-reasoner

TAREA=$1
PROVEEDOR=${2:-"vertex"}

if [ -z "$TAREA" ]; then
    echo "🔱 [NEXUS CLAW] Error: Por favor, especifica la tarea a realizar."
    echo "Uso: ./scripts/nexus_agent.sh \"Instrucción de la tarea\" [proveedor]"
    echo "Ejemplo: ./scripts/nexus_agent.sh \"Crea un script python que lea VRAM en /tmp\" deepseek-reasoner"
    exit 1
fi

echo "🦾 [NEXUS CLAW] Despachando tarea autónoma en silicio..."
echo "🤖 Proveedor Cognitivo: $PROVEEDOR"
echo "📝 Tarea: $TAREA"
echo "--------------------------------------------------"

# Realizar la petición HTTP al endpoint del orquestador
curl -s -X POST http://localhost:43211/api/agent/execute \
     -H "Content-Type: application/json" \
     -d "{\"task\": \"$TAREA\", \"provider\": \"$PROVEEDOR\"}" | jq .

echo ""
