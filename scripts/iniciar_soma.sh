#!/usr/bin/env bash
# ==========================================
# 🧬 NEXUS SOMA — Iniciar Nervio Sensorial Periférico
# ==========================================
# Lanza el daemon de telemetría hardware en segundo plano.
# - Compila si es necesario (release para menor latencia)
# - Ejecuta en background
# - Guarda PID en /tmp/nexus_soma.pid
# - Se reinicia automáticamente si falla (hasta 3 veces)
#
# Uso:
#   ./iniciar_soma.sh          # Iniciar daemon
#   ./iniciar_soma.sh stop     # Detener daemon
#   ./iniciar_soma.sh restart  # Reiniciar daemon
#   ./iniciar_soma.sh status   # Ver estado
# ==========================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_NAME="nexus_soma_daemon"
BIN_PATH="$PROJECT_DIR/.cargo-cache/release/$BIN_NAME"
PID_FILE="/tmp/nexus_soma.pid"
LOCK_FILE="/tmp/nexus_soma.lock"
SOMA_FILE="/tmp/nexus_soma.json"
LOG_FILE="/tmp/nexus_soma.log"

# Colores
ROJO='\033[0;31m'
VERDE='\033[0;32m'
AMARILLO='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${CYAN}[SOMA]${NC} $1"; }
ok()    { echo -e "${VERDE}[SOMA ✓]${NC} $1"; }
warn()  { echo -e "${AMARILLO}[SOMA ⚠]${NC} $1"; }
err()   { echo -e "${ROJO}[SOMA ✗]${NC} $1"; }

# ──────────────────────────────────────────
# Verificar si ya está corriendo
# ──────────────────────────────────────────
is_running() {
    if [ -f "$PID_FILE" ]; then
        local pid
        pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            return 0  # Está corriendo
        fi
        # PID muerto, limpiar
        rm -f "$PID_FILE" "$LOCK_FILE"
    fi
    return 1
}

# ──────────────────────────────────────────
# Compilar (si no existe o si pasamos --force)
# ──────────────────────────────────────────
build() {
    local force="${1:-}"
    
    if [ -f "$BIN_PATH" ] && [ "$force" != "--force" ]; then
        ok "Binario ya compilado: $BIN_PATH"
        return 0
    fi

    info "Compilando $BIN_NAME (release)..."
    cd "$PROJECT_DIR"
    
    # Usar la configuración de cargo del workspace
    cargo build --release --bin "$BIN_NAME" -j "$(( $(nproc) - 2 ))" 2>&1 | tail -5
    
    if [ ! -f "$BIN_PATH" ]; then
        # Buscar en debug también
        BIN_PATH="$PROJECT_DIR/.cargo-cache/debug/$BIN_NAME"
        if [ ! -f "$BIN_PATH" ]; then
            err "Compilación fallida. No se encontró el binario."
            return 1
        fi
    fi
    
    ok "Compilación exitosa: $(du -h "$BIN_PATH" | cut -f1)"
}

# ──────────────────────────────────────────
# Iniciar el daemon
# ──────────────────────────────────────────
start() {
    if is_running; then
        warn "El daemon ya está corriendo (PID $(cat "$PID_FILE"))"
        return 0
    fi

    build "${1:-}"

    info "Iniciando SOMA Daemon..."

    # Crear directorio de logs si no existe
    touch "$LOG_FILE" 2>/dev/null || true

    # Ejecutar en background con nohup
    nohup "$BIN_PATH" > "$LOG_FILE" 2>&1 &
    local pid=$!
    echo "$pid" > "$PID_FILE"

    # Esperar a que el lock file aparezca (indica que arrancó)
    local timeout=10
    while [ $timeout -gt 0 ]; do
        if [ -f "$LOCK_FILE" ]; then
            ok "SOMA Daemon iniciado (PID $pid)"
            info "Telemetría: $SOMA_FILE"
            info "Logs: $LOG_FILE"
            info "Para monitorear: tail -f $LOG_FILE"
            
            # Mostrar primer latido
            sleep 1
            if [ -f "$SOMA_FILE" ]; then
                local cpu_temp
                cpu_temp=$(grep -o '"temp_c":[0-9.]*' "$SOMA_FILE" | head -1 | cut -d: -f2)
                info "🌡️  Temperatura CPU: ${cpu_temp}°C"
            fi
            return 0
        fi
        sleep 1
        timeout=$((timeout - 1))
    done

    err "Timeout esperando que el daemon arranque. Revisa: $LOG_FILE"
    return 1
}

# ──────────────────────────────────────────
# Detener el daemon
# ──────────────────────────────────────────
stop() {
    if ! is_running; then
        warn "El daemon no está corriendo."
        rm -f "$PID_FILE" 2>/dev/null || true
        return 0
    fi

    local pid
    pid=$(cat "$PID_FILE")
    info "Deteniendo SOMA Daemon (PID $pid)..."

    # SIGTERM primero (graceful)
    kill "$pid" 2>/dev/null || true
    
    # Esperar hasta 5 segundos
    local timeout=5
    while [ $timeout -gt 0 ]; do
        if ! kill -0 "$pid" 2>/dev/null; then
            ok "Daemon detenido."
            rm -f "$PID_FILE" 2>/dev/null || true
            return 0
        fi
        sleep 1
        timeout=$((timeout - 1))
    done

    # SIGKILL si no responde
    warn "Daemon no respondió a SIGTERM. Forzando kill -9..."
    kill -9 "$pid" 2>/dev/null || true
    rm -f "$PID_FILE" 2>/dev/null || true
    ok "Daemon forzado a detenerse."
}

# ──────────────────────────────────────────
# Mostrar estado y telemetría
# ──────────────────────────────────────────
status() {
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════${NC}"
    echo -e "${CYAN}   🧬 SOMA DAEMON — Estado Actual${NC}"
    echo -e "${CYAN}═══════════════════════════════════════${NC}"

    if is_running; then
        local pid
        pid=$(cat "$PID_FILE")
        echo -e "  Estado:    ${VERDE}✅ ACTIVO${NC}"
        echo -e "  PID:       $pid"
        echo -e "  Binario:   $BIN_PATH"
        
        local uptime_secs
        uptime_secs=$(ps -o etimes= -p "$pid" 2>/dev/null | tr -d ' ')
        echo -e "  Uptime:    ${uptime_secs:-?} segundos"
        
        local cpu_mem
        cpu_mem=$(ps -o %cpu,%mem,rsz= -p "$pid" 2>/dev/null | tr -s ' ')
        echo -e "  CPU/MEM:   $cpu_mem"
    else
        echo -e "  Estado:    ${ROJO}❌ INACTIVO${NC}"
    fi

    if [ -f "$SOMA_FILE" ]; then
        echo ""
        echo -e "${CYAN}  📊 Último latido:${NC}"
        local timestamp
        timestamp=$(grep -o '"timestamp_utc":"[^"]*"' "$SOMA_FILE" | cut -d'"' -f4)
        echo -e "  🕐  $timestamp"
        
        local cpu_temp cpu_usage ram_used
        cpu_temp=$(grep -oP '"temp_c":\K[0-9.]+' "$SOMA_FILE" | head -1)
        cpu_usage=$(grep -oP '"global_usage_pct":\K[0-9.]+' "$SOMA_FILE" | head -1)
        ram_used=$(grep -oP '"used_pct":\K[0-9.]+' "$SOMA_FILE" | head -1)
        
        echo -e "  🌡️  CPU: ${cpu_temp:-?}°C | Uso: ${cpu_usage:-?}%"
        echo -e "  🧮 RAM: ${ram_used:-?}% usado"
        
        local gpu_temp
        gpu_temp=$(grep -oP '"temp_c":\K[0-9.]+' "$SOMA_FILE" | tail -1)
        echo -e "  🎮 GPU: ${gpu_temp:-Offline}°C"
        
        echo ""
        echo -e "  Archivo: $SOMA_FILE ($(du -h "$SOMA_FILE" | cut -f1))"
    fi

    echo -e "${CYAN}═══════════════════════════════════════${NC}"
}

# ──────────────────────────────────────────
# Instalar servicio systemd (opcional)
# ──────────────────────────────────────────
install_systemd() {
    local service_name="nexus-soma-daemon"
    local service_path="/etc/systemd/system/${service_name}.service"

    if [ "$EUID" -ne 0 ]; then
        warn "Se necesita sudo para instalar el servicio systemd."
        warn "Ejecuta: sudo $0 install-systemd"
        return 1
    fi

    info "Instalando servicio systemd: $service_name"

    cat > "$service_path" << EOF
[Unit]
Description=🧬 NEXUS SOMA Daemon — Nervio Sensorial Periférico
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$PROJECT_DIR
ExecStart=$BIN_PATH
Restart=always
RestartSec=5
StandardOutput=append:$LOG_FILE
StandardError=append:$LOG_FILE

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable "$service_name"
    systemctl start "$service_name"

    ok "Servicio systemd instalado e iniciado."
    info "Comandos:"
    info "  systemctl status $service_name"
    info "  journalctl -u $service_name -f"
}

# ──────────────────────────────────────────
# MAIN
# ──────────────────────────────────────────
case "${1:-start}" in
    start)
        start "${2:-}"
        ;;
    stop)
        stop
        ;;
    restart)
        stop
        sleep 1
        start "${2:-}"
        ;;
    status)
        status
        ;;
    build)
        build --force
        ;;
    install-systemd)
        install_systemd
        ;;
    *)
        echo "Uso: $0 {start|stop|restart|status|build|install-systemd} [--force]"
        echo ""
        echo "  start             Iniciar el daemon (compila si es necesario)"
        echo "  start --force     Forzar recompilación antes de iniciar"
        echo "  stop              Detener el daemon (graceful)"
        echo "  restart           Reiniciar el daemon"
        echo "  status            Mostrar estado y telemetría actual"
        echo "  build             Recompilar el binario"
        echo "  install-systemd   Instalar como servicio systemd (requiere sudo)"
        exit 1
        ;;
esac
