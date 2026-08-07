#!/bin/bash
# NEXUS Memory Guardian - i7 12700F + 64GB RAM Optimization
# Gathers available system memory details and logs alerts.

THRESHOLD_CRITICAL=4000000   # 4GB libre = CRÍTICO (64GB Total)
THRESHOLD_WARNING=8000000    # 8GB libre = ADVERTENCIA
NEXUS_API="http://127.0.0.1:43211"
LOG_TAG="NEXUS-MEM-GUARD"

while true; do
    AVAIL=$(awk "/MemAvailable/{print \$2}" /proc/meminfo)
    TOTAL=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
    USED=$((TOTAL - AVAIL))
    USED_GB=$(awk "BEGIN{printf \"%.1f\", $USED/1048576}")
    AVAIL_GB=$(awk "BEGIN{printf \"%.1f\", $AVAIL/1048576}")
    
    # Proceso que más RAM consume en este momento
    TOP_PROC=$(ps aux --sort=-%mem | awk "NR==2{print \$11\" (\"int(\$4)\"% RAM)\"}")

    if [ "$AVAIL" -lt "$THRESHOLD_CRITICAL" ]; then
        MSG="🚨 NIVEL CRÍTICO: Solo ${AVAIL_GB}GB libres. Usado: ${USED_GB}GB. Culpable probable: ${TOP_PROC}"
        # 1. Grita al journal del sistema (lo ve journalctl)
        logger -t "$LOG_TAG" -p daemon.crit "$MSG"
        # 2. Toca al orquestador en su API
        curl -sf -X POST "${NEXUS_API}/internal/alert" \
            -H "Content-Type: application/json" \
            -d "{\"level\":\"critical\",\"source\":\"mem_guard\",\"message\":\"${MSG}\"}" \
            2>/dev/null || logger -t "$LOG_TAG" "API no respondió - orquestador posiblemente caído"
        # 3. Escribe en el log soberano
        echo "[$(date "+%Y-%m-%d %H:%M:%S")] CRITICO | RAM libre: ${AVAIL_GB}GB | ${TOP_PROC}" \
            >> /home/soberano/NEXUS_ULTIMATE_CORE/logs/mem_guard.log

    elif [ "$AVAIL" -lt "$THRESHOLD_WARNING" ]; then
        MSG="⚠️  ADVERTENCIA: ${AVAIL_GB}GB libres. Usado: ${USED_GB}GB. Proceso pesado: ${TOP_PROC}"
        logger -t "$LOG_TAG" -p daemon.warning "$MSG"
        curl -sf -X POST "${NEXUS_API}/internal/alert" \
            -H "Content-Type: application/json" \
            -d "{\"level\":\"warning\",\"source\":\"mem_guard\",\"message\":\"${MSG}\"}" \
            2>/dev/null
        echo "[$(date "+%Y-%m-%d %H:%M:%S")] ADVERTENCIA | RAM libre: ${AVAIL_GB}GB | ${TOP_PROC}" \
            >> /home/soberano/NEXUS_ULTIMATE_CORE/logs/mem_guard.log
    fi

    sleep 30
done
