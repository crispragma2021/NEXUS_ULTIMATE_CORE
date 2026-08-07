#!/bin/bash
# scripts/service_manager.sh - Gestor de servicios en segundo plano para NEXUS
# Permite iniciar, detener, listar y consultar logs de procesos de larga duración.
# Uso: 
#   ./scripts/service_manager.sh start <nombre> "<comando>" [archivo_log]
#   ./scripts/service_manager.sh stop <nombre>
#   ./scripts/service_manager.sh status [nombre]
#   ./scripts/service_manager.sh logs <nombre> [lineas]
#   ./scripts/service_manager.sh list

SERVICE_DIR="/home/soberano/NEXUS_ULTIMATE_CORE/data/services"
mkdir -p "$SERVICE_DIR"

COMMAND="$1"
SERVICE_NAME="$2"

case "$COMMAND" in
    start)
        SERVICE_CMD="$3"
        LOG_FILE="$4"
        
        if [ -z "$SERVICE_NAME" ] || [ -z "$SERVICE_CMD" ]; then
            echo "❌ Error: Uso: $0 start <nombre> \"<comando>\" [archivo_log]"
            exit 1
        fi
        
        if [ -z "$LOG_FILE" ]; then
            LOG_FILE="$SERVICE_DIR/${SERVICE_NAME}.log"
        fi
        
        # Verificar si ya está corriendo
        PID_FILE="$SERVICE_DIR/${SERVICE_NAME}.pid"
        if [ -f "$PID_FILE" ]; then
            PID=$(cat "$PID_FILE")
            if kill -0 "$PID" 2>/dev/null; then
                echo "⚠️ El servicio '$SERVICE_NAME' ya está corriendo con PID $PID."
                exit 0
            fi
        fi
        
        echo "🚀 Iniciando servicio '$SERVICE_NAME'..."
        echo "$SERVICE_CMD" > "$SERVICE_DIR/${SERVICE_NAME}.cmd"
        
        # Iniciar desacoplado
        setsid nohup bash -c "$SERVICE_CMD" > "$LOG_FILE" 2>&1 &
        DAEMON_PID=$!
        
        # Esperar un momento para verificar que no muera inmediatamente
        sleep 2
        
        # Conseguir el PID real del proceso hijo (en caso de que setsid/bash haya cambiado)
        # Buscamos el proceso de la sesión
        REAL_PID=$(pgrep -P $DAEMON_PID 2>/dev/null | head -n 1)
        if [ -z "$REAL_PID" ]; then
            REAL_PID=$DAEMON_PID
        fi
        
        if kill -0 "$REAL_PID" 2>/dev/null; then
            echo "$REAL_PID" > "$PID_FILE"
            echo "$LOG_FILE" > "$SERVICE_DIR/${SERVICE_NAME}.logpath"
            echo "✅ Servicio '$SERVICE_NAME' iniciado exitosamente con PID $REAL_PID."
            echo "Logs redirigidos a: $LOG_FILE"
        else
            echo "❌ Error al iniciar el servicio. Verifique los logs en: $LOG_FILE"
            cat "$LOG_FILE" | tail -n 10
            exit 1
        fi
        ;;
        
    stop)
        if [ -z "$SERVICE_NAME" ]; then
            echo "❌ Error: Uso: $0 stop <nombre>"
            exit 1
        fi
        
        PID_FILE="$SERVICE_DIR/${SERVICE_NAME}.pid"
        if [ ! -f "$PID_FILE" ]; then
            echo "❌ El servicio '$SERVICE_NAME' no está registrado o no tiene PID file."
            exit 1
        fi
        
        PID=$(cat "$PID_FILE")
        echo "🛑 Deteniendo servicio '$SERVICE_NAME' (PID $PID)..."
        
        # Matar grupo de procesos para asegurar que mueran los hijos
        pkill -P "$PID" 2>/dev/null
        kill -15 "$PID" 2>/dev/null
        sleep 1
        if kill -0 "$PID" 2>/dev/null; then
            echo "⚠️ Forzando detención (SIGKILL)..."
            kill -9 "$PID" 2>/dev/null
        fi
        
        rm -f "$PID_FILE"
        rm -f "$SERVICE_DIR/${SERVICE_NAME}.logpath"
        echo "✅ Servicio '$SERVICE_NAME' detenido."
        ;;
        
    status)
        if [ -n "$SERVICE_NAME" ]; then
            PID_FILE="$SERVICE_DIR/${SERVICE_NAME}.pid"
            if [ -f "$PID_FILE" ]; then
                PID=$(cat "$PID_FILE")
                if kill -0 "$PID" 2>/dev/null; then
                    CMD=$(cat "$SERVICE_DIR/${SERVICE_NAME}.cmd" 2>/dev/null)
                    CPU_MEM=$(ps -p "$PID" -o %cpu,%mem,comm= 2>/dev/null | tail -n 1)
                    echo "🟢 Servicio '$SERVICE_NAME' está ACTIVO."
                    echo "PID: $PID"
                    echo "Comando: $CMD"
                    echo "Métricas (CPU% MEM%): $CPU_MEM"
                else
                    echo "🔴 Servicio '$SERVICE_NAME' está INACTIVO (proceso muerto pero PID file presente)."
                fi
            else
                echo "🔴 Servicio '$SERVICE_NAME' está INACTIVO."
            fi
        else
            $0 list
        fi
        ;;
        
    logs)
        LINES="${3:-20}"
        if [ -z "$SERVICE_NAME" ]; then
            echo "❌ Error: Uso: $0 logs <nombre> [lineas]"
            exit 1
        fi
        
        LOG_FILE=""
        if [ -f "$SERVICE_DIR/${SERVICE_NAME}.logpath" ]; then
            LOG_FILE=$(cat "$SERVICE_DIR/${SERVICE_NAME}.logpath")
        else
            LOG_FILE="$SERVICE_DIR/${SERVICE_NAME}.log"
        fi
        
        if [ -f "$LOG_FILE" ]; then
            echo "=== Logs de '$SERVICE_NAME' (Últimas $LINES líneas) ==="
            tail -n "$LINES" "$LOG_FILE"
            echo "=================================================="
        else
            echo "❌ No se encontraron logs para el servicio '$SERVICE_NAME' en '$LOG_FILE'."
        fi
        ;;
        
    list)
        echo "📋 Lista de Servicios Gestionados por NEXUS:"
        echo "--------------------------------------------------"
        printf "%-20s %-10s %-10s %-20s\n" "NOMBRE" "PID" "ESTADO" "DETALLES"
        echo "--------------------------------------------------"
        for f in "$SERVICE_DIR"/*.cmd; do
            [ -e "$f" ] || continue
            NAME=$(basename "$f" .cmd)
            PID_FILE="$SERVICE_DIR/${NAME}.pid"
            STATUS="INACTIVO"
            PID="-"
            
            if [ -f "$PID_FILE" ]; then
                TEMP_PID=$(cat "$PID_FILE")
                if kill -0 "$TEMP_PID" 2>/dev/null; then
                    STATUS="ACTIVO"
                    PID="$TEMP_PID"
                else
                    STATUS="MUERTO"
                fi
            fi
            
            printf "%-20s %-10s %-10s %-20s\n" "$NAME" "$PID" "$STATUS" "$(cat "$f" | cut -c1-30)..."
        done
        echo "--------------------------------------------------"
        ;;
        
    *)
        echo "❓ Uso: $0 {start|stop|status|logs|list} [argumentos...]"
        exit 1
        ;;
esac
