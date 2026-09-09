#!/bin/bash
# NEXUS SKILL: LABORATORIO v1.0
# "El Guardián de Experimentos"

COMMAND=$1
ARG=$2

IMAGE_NAME="nexus-lab-img"
CONTAINER_NAME="nexus_laboratorio"

case "$COMMAND" in
    "build")
        echo "🛠️  Construyendo el Laboratorio..."
        docker build -t $IMAGE_NAME -f LABORATORIO/Dockerfile.lab LABORATORIO/
        ;;
    "start")
        echo "🚀 Iniciando Laboratorio..."
        if [ "$(docker ps -aq -f name=$CONTAINER_NAME)" ]; then
            docker start $CONTAINER_NAME
        else
            docker run -d --name $CONTAINER_NAME \
                -v "$(pwd)/LABORATORIO/workspace:/workspace" \
                $IMAGE_NAME
        fi
        echo "✅ Laboratorio Activo."
        ;;
    "stop")
        echo "🛑 Deteniendo Laboratorio..."
        docker stop $CONTAINER_NAME
        docker rm $CONTAINER_NAME
        ;;
    "exec")
        if [ -z "$ARG" ]; then
            echo "❌ Error: Define un comando para ejecutar en el lab."
        else
            echo "🧪 Ejecutando en Laboratorio: $ARG"
            docker exec $CONTAINER_NAME bash -c "$ARG"
        fi
        ;;
    "status")
        docker ps -f name=$CONTAINER_NAME
        ;;
    *)
        echo "Uso: $0 {build|start|stop|exec 'comando'|status}"
        ;;
esac
