#!/usr/bin/env bash
# ==========================================
# 🔱 NEXUS TELEGRAM CTL — control del daemon de Telegram
# ==========================================
# start   → lanza el daemon + watchdog de inactividad
# stop    → mata daemon y watchdog
# status  → estado del daemon
#
# El watchdog cierra el daemon tras NEXUS_TG_IDLE_MIN minutos
# sin actividad (default: 10). La actividad se mide por el mtime
# del log: cada mensaje entrante/respuesta escribe en él.
# ==========================================

set -u

REPO="/home/soberano/NEXUS_ULTIMATE_CORE"
BIN="$HOME/.cargo-target/release/nexus_telegram_daemon"
PIDFILE="/tmp/nexus_telegram_daemon.pid"
WATCHPID="/tmp/nexus_telegram_watchdog.pid"
LOGDIR="$HOME/.local/share/nexus/logs"
LOG="$LOGDIR/telegram_daemon.log"
IDLE_MIN="${NEXUS_TG_IDLE_MIN:-10}"
IDLE_SEC=$((IDLE_MIN * 60))

mkdir -p "$LOGDIR"

is_running() {
    [ -f "$PIDFILE" ] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null
}

start() {
    if is_running; then
        echo "✅ Daemon ya está corriendo (PID $(cat "$PIDFILE"))"
        return 0
    fi
    if [ ! -x "$BIN" ]; then
        echo "❌ Binario no encontrado: $BIN — compilando..."
        (cd "$REPO/core" && cargo build --release --bin nexus_telegram_daemon) || { echo "❌ Compilación falló"; exit 1; }
    fi
    # El daemon carga .env relativo (../.env desde core/), así que lanzamos con cwd=core
    cd "$REPO/core" || exit 1
    nohup "$BIN" >>"$LOG" 2>&1 &
    echo $! > "$PIDFILE"
    # Watchdog: mata el daemon si el log no se toca en IDLE_SEC
    (
        while kill -0 "$(cat "$PIDFILE")" 2>/dev/null; do
            if [ -f "$LOG" ] && [ $(( $(date +%s) - $(stat -c %Y "$LOG") )) -ge "$IDLE_SEC" ]; then
                echo "[$(date '+%F %T')] ⏳ ${IDLE_MIN} min sin actividad — cerrando daemon de Telegram" >> "$LOG"
                kill "$(cat "$PIDFILE")" 2>/dev/null
                rm -f "$PIDFILE"
                exit 0
            fi
            sleep 30
        done
        rm -f "$PIDFILE"
    ) &
    echo $! > "$WATCHPID"
    echo "🚀 Daemon lanzado (PID $(cat "$PIDFILE")) — log: $LOG"
    echo "   Watchdog: se cierra tras ${IDLE_MIN} min sin actividad"
}

stop() {
    if [ -f "$WATCHPID" ]; then kill "$(cat "$WATCHPID")" 2>/dev/null; rm -f "$WATCHPID"; fi
    if is_running; then
        kill "$(cat "$PIDFILE")"
        rm -f "$PIDFILE"
        echo "🛑 Daemon detenido"
    else
        echo "ℹ️  Daemon no estaba corriendo"
    fi
}

status() {
    if is_running; then
        echo "✅ Daemon ACTIVO (PID $(cat "$PIDFILE"))"
        [ -f "$LOG" ] && echo "   Última actividad: $(stat -c %y "$LOG" | cut -d. -f1)"
    else
        echo "💤 Daemon INACTIVO"
    fi
    [ -f "$WATCHPID" ] && kill -0 "$(cat "$WATCHPID")" 2>/dev/null && echo "   Watchdog: activo"
}

case "${1:-}" in
    start) start ;;
    stop) stop ;;
    status) status ;;
    *) echo "Uso: $0 {start|stop|status}"; exit 1 ;;
esac
