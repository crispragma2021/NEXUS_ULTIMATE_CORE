#!/bin/bash
# scripts/run_daemon.sh - Ejecuta un comando en segundo plano de forma totalmente desacoplada.
# Uso: ./scripts/run_daemon.sh "comando" "archivo_log" [segundos_espera]

COMMAND="$1"
LOGFILE="${2:-/tmp/daemon_run.log}"
WAIT_SECS="${3:-3}"

if [ -z "$COMMAND" ]; then
    echo "Error: Debes proporcionar un comando a ejecutar."
    exit 1
fi

echo "🚀 Iniciando comando desacoplado..."
echo "Comando: $COMMAND"
echo "Log: $LOGFILE"

# Limpiar log anterior
> "$LOGFILE"

# Usar setsid para iniciar en una nueva sesión de proceso, redirigiendo todas las entradas/salidas
setsid nohup bash -c "$COMMAND" > "$LOGFILE" 2>&1 &

sleep "$WAIT_SECS"

# Mostrar las últimas líneas del log para validación
echo "=== Últimas líneas del log ==="
cat "$LOGFILE" | tail -n 15
echo "=============================="

echo "✅ Proceso lanzado y desacoplado."
