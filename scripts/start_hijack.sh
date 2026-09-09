#!/bin/bash
echo "--- INICIALIZANDO PROXY HIJACK EN PUERTO 4444 ---"

# 1. Limpiar cualquier proceso previo colgado en el puerto 4444
PREVIOUS_PID=$(sudo lsof -t -i:4444)
if [ ! -z "$PREVIOUS_PID" ]; then
    echo "Removiendo proceso residual en puerto 4444 (PID: $PREVIOUS_PID)..."
    sudo kill -9 $PREVIOUS_PID
    sleep 1
fi

# 2. Definir la ruta real del binario de release
REAL_BIN="./target/release/proxy_hijack"

# 3. Asegurar permisos de ejecución
chmod +x "$REAL_BIN"

# 4. Lanzar en segundo plano redirigiendo la salida al archivo de logs
echo "Levantando proxy con certificado CA activo..."
nohup "$REAL_BIN" > ./logs/proxy_hijack.log 2>&1 &

# Esperar estabilización del socket
sleep 2

echo -e "\n--- VERIFICACIÓN DE ARRANQUE ---"
sudo lsof -i :4444 && echo "PROXY HIJACK OPERATIVO EN PUERTO 4444" || echo "Error: Revisa ./logs/proxy_hijack.log"
