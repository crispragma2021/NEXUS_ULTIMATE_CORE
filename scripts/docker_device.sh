#!/bin/bash
# 🔱 NEXUS OMEGA - Docker Fake Device Controller (Headless)
# Uso: ./docker_device.sh {create|compromised|rebuild} <device_id>

COMMAND=$1
DEVICE_ID=$2

if [ -z "$COMMAND" ] || [ -z "$DEVICE_ID" ]; then
    echo "Usage: $0 {create|compromised|rebuild} <device_id>"
    exit 1
fi

IMAGE_NAME="nexus-fake-device"
CONTAINER_NAME="nexus_sandbox_${DEVICE_ID}"

case "$COMMAND" in
    "create")
        echo "🐳 Creando dispositivo falso con Docker en modo Headless: $CONTAINER_NAME..."
        
        # Validar si la imagen ya existe, si no, compilarla
        if [[ "$(docker images -q $IMAGE_NAME 2> /dev/null)" == "" ]]; then
            echo "   [!] Compilando imagen base de Docker..."
            docker build -t $IMAGE_NAME /home/soberano/NEXUS_ULTIMATE_CORE/docker/
        fi

        # Generar MAC aleatoria para evadir rastreo físico
        MAC_ADDR=$(printf '02:42:ac:11:%02x:%02x' $((RANDOM%256)) $((RANDOM%256)))
        echo "   [+] Asignando dirección MAC virtual: $MAC_ADDR"

        # Lanzar contenedor headless en segundo plano
        docker run -d \
            --name "$CONTAINER_NAME" \
            --mac-address "$MAC_ADDR" \
            --memory="2g" \
            --cpus="2" \
            --network=bridge \
            -v "/tmp/nexus_shared_${DEVICE_ID}:/home/soberano_anon/shared" \
            $IMAGE_NAME

        echo "✅ Dispositivo $CONTAINER_NAME levantado con éxito."
        ;;

    "compromised")
        echo "⚠️  [ALERT] Dispositivo $CONTAINER_NAME comprometido. Purgando rastro físico..."
        
        # Parar y eliminar de inmediato
        docker stop -t 1 "$CONTAINER_NAME" 2>/dev/null
        docker rm -f "$CONTAINER_NAME" 2>/dev/null
        
        # Limpiar volúmenes temporales de sesión
        rm -rf "/tmp/nexus_shared_${DEVICE_ID}"
        
        echo "✅ Dispositivo destruido. Silencio absoluto restablecido."
        ;;

    "rebuild")
        echo "🔄 Reconstruyendo dispositivo $CONTAINER_NAME..."
        $0 compromised "$DEVICE_ID"
        $0 create "$DEVICE_ID"
        ;;

    *)
        echo "Comando no reconocido."
        exit 1
        ;;
esac
