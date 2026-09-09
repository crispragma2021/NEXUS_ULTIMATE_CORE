#!/usr/bin/env bash
# ============================================================
# NEXUS WATCHDOG DE AGENTES — Detecta y mata procesos colgados
# ============================================================
# Problema que resuelve: un test/binario de NEXUS colgado puede
# devorar toda la CPU (visto: cerebro_digital a 751% CPU + 11 GiB
# RAM congelando VSCodium). Este watchdog lo detecta y lo mata.
#
# Regla de detección (conservadora, sin falsos positivos):
#   - Proceso en ~/.cargo-target/debug/deps/  → es un TEST de cargo
#   - %CPU promedio > UMBRAL_CPU             → consumiendo a lo loco
#   - Tiempo de CPU acumulado > TIEMPO_MIN   → no es un test fugaz
#   - Vivo desde hace > ETIMES_MIN segundos  → no es un test recién lanzado
#
# NUNCA toca: VSCodium, Roo, ollama, chrome, servicios nexus-*,
# hermes, ni binarios fuera de ~/.cargo-target.
#
# Programación sugerida (crontab del usuario):
#   */5 * * * * /home/soberano/NEXUS_ULTIMATE_CORE/scripts/watchdog_agentes.sh
# ============================================================

set -euo pipefail

# ── Config ──────────────────────────────────────────────────
DEPS_DIR="/home/soberano/.cargo-target/debug/deps"
LOG_FILE="${NEXUS_AGENT_DATOS:-$HOME/.local/share/nexus-agent}/watchdog.log"
UMBRAL_CPU=200        # %CPU promedio mínimo para considerar colgado
TIEMPO_MIN_SEG=120    # tiempo de CPU acumulado mínimo (2 min)
ETIMES_MIN_SEG=300    # mínimo de vida del proceso (5 min)

# ── Colores (solo si es TTY) ────────────────────────────────
if [ -t 1 ]; then
    RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
else
    RED=''; GREEN=''; YELLOW=''; NC=''
fi

log() { echo -e "[$(date '+%F %T')] $*" | tee -a "$LOG_FILE"; }

# Convierte TIME de ps (MM:SS o HH:MM:SS) a segundos
time_a_segundos() {
    local t="$1" h=0 m=0 s=0
    if [[ "$t" == *:*:* ]]; then
        h="${t%%:*}"; t="${t#*:}"
    fi
    m="${t%%:*}"; s="${t##*:}"
    echo $((10#$h * 3600 + 10#$m * 60 + 10#$s))
}

# ── Escaneo ─────────────────────────────────────────────────
asesinados=0
while IFS= read -r linea; do
    [ -z "$linea" ] && continue
    pid=$(echo "$linea" | awk '{print $1}')
    pcpu=$(echo "$linea" | awk '{print $2}')
    time_seg=$(time_a_segundos "$(echo "$linea" | awk '{print $3}')")
    etimes=$(echo "$linea" | awk '{print $4}')
    args=$(echo "$linea" | cut -d' ' -f5-)

    # ¿Es un test de NEXUS en deps/?
    case "$args" in
        *"$DEPS_DIR"*) ;;
        *) continue ;;
    esac

    # Umbrales
    [ "${pcpu%.*}" -ge "$UMBRAL_CPU" ] || continue
    [ "$time_seg" -ge "$TIEMPO_MIN_SEG" ] || continue
    [ "$etimes" -ge "$ETIMES_MIN_SEG" ] || continue

    log "${YELLOW}⚠️  Test colgado detectado: pid=$pid cpu=${pcpu}% time=${time_seg}s vida=${etimes}s${NC}"
    log "   comando: ${args:0:160}"
    log "${RED}   → SIGTERM${NC}"
    kill "$pid" 2>/dev/null || true
    sleep 5
    if kill -0 "$pid" 2>/dev/null; then
        log "${RED}   → no respondió, SIGKILL${NC}"
        kill -9 "$pid" 2>/dev/null || true
    fi
    log "${GREEN}   ✓ proceso $pid terminado${NC}"
    asesinados=$((asesinados + 1))
done < <(ps -eo pid,pcpu,time,etimes,args --sort=-pcpu | tail -n +2)

if [ "$asesinados" -eq 0 ]; then
    log "${GREEN}✓ Sin procesos colgados (escaneo limpio)${NC}"
else
    log "${RED}✗ $asesinados proceso(s) colgado(s) terminado(s)${NC}"
fi
