#!/bin/bash
# NEXUS_SKILL: Web Search (Brave/Google)
# ADN Puro: Sin dependencias externas pesadas

QUERY=$1
echo "🔎 Nexus buscando: $QUERY"

# Simulación de llamada al MCP de búsqueda 
# (Aquí es donde luego conectaremos el binario de Rust)
curl -s "https://api.search.brave.com/res/v1/web/search?q=$QUERY" \
     -H "Accept: application/json" \
     -H "X-Subscription-Token: $BRAVE_API_KEY" > /tmp/nexus_search_res.json

echo "✅ Datos indexados en /tmp/nexus_search_res.json"
