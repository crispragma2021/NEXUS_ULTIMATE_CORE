#!/bin/bash
# 🔱 NEXUS AUTO-DEPLOY WATCHDOG
# Vigila cambios en nexus_puro_engine.rs y reinicia el chat soberano automáticamente.
# Monitorea también que el chat nunca se caiga sin razón.
# Arquitecto: esto corre en background y yo lo superviso.

BASE_DIR="/home/soberano/NEXUS_ULTIMATE_CORE"
ENGINE_FILE="$BASE_DIR/src-tauri/src/nexus_puro_engine.rs"
LOG="$BASE_DIR/logs/watchdog_nexus.log"
PID_FILE="/tmp/nexus_chat.pid"

mkdir -p "$BASE_DIR/logs"

echo "[$(date '+%H:%M:%S')] 🧠 NEXUS Watchdog iniciado" >> "$LOG"
echo "[$(date '+%H:%M:%S')] 🧠 Vigilando: $ENGINE_FILE" >> "$LOG"

# ── UTILIDAD: PIDs del chat Tauri GUI (NUNCA el Core headless) ─────────────
# El Core (nexus.service) corre como "nexus-ui --headless". Este watchdog solo
# debe gestionar el chat Tauri de escritorio (nexus-ui SIN --headless).
# NUNCA matar el Core o entrará en bucle de reinicio por SIGKILL.
chat_gui_pids() {
    pgrep -f "nexus-ui" 2>/dev/null | while read -r p; do
        if [ -r "/proc/$p/cmdline" ] && grep -qv -- "--headless" "/proc/$p/cmdline" 2>/dev/null; then
            echo "$p"
        fi
    done
}

# --- FUNCIÓN: asegurar que el chat está vivo ---
asegurar_chat() {
    local REASON="$1"
    # Verificar si el proceso del chat GUI existe (excluye el Core headless)
    if [ -z "$(chat_gui_pids)" ]; then
        echo "[$(date '+%H:%M:%S')] ⚠️ Chat caído ($REASON). Relanzando..." >> "$LOG"
        cd "$BASE_DIR"
        # Matar cualquier tauri residual (nunca el Core)
        pkill -9 -f "cargo-tauri" 2>/dev/null || true
        for p in $(chat_gui_pids); do kill -9 "$p" 2>/dev/null || true; done
        sleep 0.5
        # Relanzar en background
        nohup cargo tauri dev --no-watch > /dev/null 2>&1 &
        CHAT_PID=$!
        echo $CHAT_PID > "$PID_FILE"
        echo "[$(date '+%H:%M:%S')] ✅ Chat relanzado (PID: $CHAT_PID)" >> "$LOG"
    fi
}

# --- INICIALIZACIÓN ---
asegurar_chat "primer_inicio"

# --- BUCLE PRINCIPAL (cada 10 segundos) ---
LAST_HASH=""
while true; do
    # 1. DETECTAR CAMBIOS EN EL ENGINE
    if [ -f "$ENGINE_FILE" ]; then
        CURRENT_HASH=$(md5sum "$ENGINE_FILE" | cut -d' ' -f1)
        if [ "$CURRENT_HASH" != "$LAST_HASH" ] && [ -n "$LAST_HASH" ]; then
            echo "[$(date '+%H:%M:%S')] 🔄 Cambio detectado en engine. Recompilando..." >> "$LOG"
            
            # Matar instancia vieja del chat GUI (nunca el Core headless)
            pkill -9 -f "cargo-tauri" 2>/dev/null || true
            for p in $(chat_gui_pids); do kill -9 "$p" 2>/dev/null || true; done
            sleep 0.5
            
            # Relanzar
            cd "$BASE_DIR"
            nohup cargo tauri dev --no-watch > /dev/null 2>&1 &
            CHAT_PID=$!
            echo $CHAT_PID > "$PID_FILE"
            echo "[$(date '+%H:%M:%S')] ✅ Chat recompilado y relanzado (PID: $CHAT_PID)" >> "$LOG"
        fi
        LAST_HASH="$CURRENT_HASH"
    fi
    
    # 2. VERIFICAR QUE EL CHAT SIGUE VIVO
    asegurar_chat "check_ciclico"
    
    sleep 10
done
