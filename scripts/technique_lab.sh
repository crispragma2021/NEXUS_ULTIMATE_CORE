#!/bin/bash
# NEXUS_SKILL: Technique Lab v1.0
# "El Ejecutor de Sombras"
# Misión: Inyectar y probar técnicas en el Laboratorio Docker.

ACTION=$1
SCRIPT_NAME=$2
SCRIPT_CONTENT=$3

CONTAINER_NAME="nexus_laboratorio"
WORKSPACE_DIR="LABORATORIO/workspace"

case "$ACTION" in
    "inject")
        if [ -z "$SCRIPT_NAME" ]; then
            echo "❌ Error: Nombre de script requerido."
            exit 1
        fi
        echo "💉 Inyectando técnica: $SCRIPT_NAME..."
        echo "$SCRIPT_CONTENT" > "$WORKSPACE_DIR/$SCRIPT_NAME"
        chmod +x "$WORKSPACE_DIR/$SCRIPT_NAME"
        echo "✅ Técnica lista en /workspace/$SCRIPT_NAME"
        ;;
    "run")
        if [ -z "$SCRIPT_NAME" ]; then
            echo "❌ Error: Nombre de script requerido."
            exit 1
        fi
        echo "🧪 Lanzando técnica en entorno aislado..."
        docker exec $CONTAINER_NAME bash -c "/workspace/$SCRIPT_NAME"
        ;;
    "cleanup")
        echo "🧹 Limpiando técnicas del laboratorio..."
        rm -rf $WORKSPACE_DIR/*.sh $WORKSPACE_DIR/*.py
        echo "✨ Espacio purificado."
        ;;
    *)
        echo "Uso: $0 {inject <name> <content> | run <name> | cleanup}"
        ;;
esac
