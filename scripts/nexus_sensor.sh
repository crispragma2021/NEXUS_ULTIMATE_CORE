#!/bin/bash
# 🌡️ NEXUS STATE SENSOR
# Gathers system telemetry (CPU load, temperature, memory) and logs it.
# Invoked every 5 minutes by systemd timer.

LOG_FILE="/home/soberano/NEXUS_ULTIMATE_CORE/logs/nexus_sensor.log"
mkdir -p /home/soberano/NEXUS_ULTIMATE_CORE/logs

# Get Current Date/Time
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

# 1. CPU Load Average
CPU_LOAD=$(cat /proc/loadavg | awk '{print $1" "$2" "$3}')

# 2. RAM Availability (MB)
MEM_FREE=$(awk '/MemAvailable/{print int($2/1024)}' /proc/meminfo)
MEM_TOTAL=$(awk '/MemTotal/{print int($2/1024)}' /proc/meminfo)
MEM_USED=$((MEM_TOTAL - MEM_FREE))

# 3. CPU Temperature
CPU_TEMP="N/A"
if [ -f "/sys/class/thermal/thermal_zone0/temp" ]; then
    RAW_TEMP=$(cat /sys/class/thermal/thermal_zone0/temp)
    CPU_TEMP=$(echo "scale=1; $RAW_TEMP / 1000" | bc 2>/dev/null || echo "$((RAW_TEMP / 1000))")
fi

# 4. Active processes count
PROC_COUNT=$(ps -e | wc -l)

# Log the state entry
echo "[$TIMESTAMP] LOAD: $CPU_LOAD | RAM: ${MEM_USED}/${MEM_TOTAL}MB (${MEM_FREE}MB free) | TEMP: ${CPU_TEMP}°C | PROCS: $PROC_COUNT" >> "$LOG_FILE"

# Keep the log file under 2000 lines (clean purge)
tail -n 2000 "$LOG_FILE" > "${LOG_FILE}.tmp" && mv "${LOG_FILE}.tmp" "$LOG_FILE"
