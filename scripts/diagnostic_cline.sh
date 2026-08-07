#!/bin/bash
echo "🔱 INICIANDO DIAGNÓSTICO DE SINAPSIS DE CLINE..."
echo "------------------------------------------------"

# 1. Localizar el directorio de almacenamiento global de la extensión
CLINE_STORAGE="$HOME/.config/Antigravity IDE/User/globalStorage/saoudrizwan.claude-dev"

if [ -d "$CLINE_STORAGE" ]; then
    echo "✅ Directorio de almacenamiento localizado."
    echo "📁 Ruta: $CLINE_STORAGE"
    echo ""
    echo "--- ÚLTIMOS LOGS DE CONFIGURACIÓN Y ERRORES ---"
    # Buscar archivos de log recientes y mostrar las últimas 30 líneas
    find "$CLINE_STORAGE" -type f -name "*.log" -o -name "*.json" | while read -r file; do
        echo "📄 Archivo: $(basename "$file")"
        tail -n 30 "$file"
        echo "------------------------------------------------"
    done
else
    echo "⚠️ Directorio global de Antigravity IDE no encontrado en la ruta por defecto."
    echo "Buscando alternativas en el host..."
    find $HOME -type d -name "saoudrizwan.claude-dev" 2>/dev/null
fi
