#!/bin/bash
# 🛡️ NEXUS ENHANCED AUTONOMY WATCHDOG
# Checks if the Orquestador is alive on port 43211 and restarts it if not.
# Managed by systemd timer or daemon loop.

PORT=43210
URL="http://127.0.0.1:$PORT/api/health"

# Ensure log directory exists
mkdir -p /home/soberano/NEXUS_ULTIMATE_CORE/logs

if ! curl -s --max-time 3 "$URL" | grep -q "status"; then
    echo "⚠️ [$(date '+%Y-%m-%d %H:%M:%S')] [WATCHDOG] NEXUS Orquestador is offline or unresponsive on port $PORT. Attempting recovery..." >> /home/soberano/NEXUS_ULTIMATE_CORE/logs/watchdog.log
    # Kill any stale orquestador processes
    pgrep -f "nexus_main" | xargs -r kill -9 2>/dev/null
    
    # Restart the service via systemctl (system or user) or run the start script directly
    if systemctl --user is-active --quiet nexus.service 2>/dev/null; then
        echo "🔄 [WATCHDOG] Restarting user nexus.service..." >> /home/soberano/NEXUS_ULTIMATE_CORE/logs/watchdog.log
        systemctl --user restart nexus.service
    elif systemctl is-active --quiet nexus.service 2>/dev/null; then
        echo "🔄 [WATCHDOG] Restarting system nexus.service..." >> /home/soberano/NEXUS_ULTIMATE_CORE/logs/watchdog.log
        sudo systemctl restart nexus.service
    else
        echo "🚀 [WATCHDOG] Starting /home/soberano/NEXUS_ULTIMATE_CORE/scripts/nexus_start.sh..." >> /home/soberano/NEXUS_ULTIMATE_CORE/logs/watchdog.log
        /home/soberano/NEXUS_ULTIMATE_CORE/scripts/nexus_start.sh >> /home/soberano/NEXUS_ULTIMATE_CORE/logs/watchdog_nexus_start.log 2>&1 &
    fi
else
    echo "🟢 [$(date '+%Y-%m-%d %H:%M:%S')] [WATCHDOG] NEXUS Orquestador is ONLINE." >> /home/soberano/NEXUS_ULTIMATE_CORE/logs/watchdog.log
    
    # 🔱 ACTUALIZACIÓN DEL SANTUARIO MULTI-AGENTE
    # Escanea huellas de procesos de agentes residentes y actualiza el registro compartido
    HERMES_PID=$(ps aux | grep -iE '\bhermes\b' | grep -v grep | awk '{print $2}' | head -n 1)
    NEXUS_PID=$(pgrep -f "nexus_main" | head -n 1)
    ROO_PID=$(ps aux | grep -iE 'roo-code|roo_code' | grep -v grep | awk '{print $2}' | head -n 1)
    
    python3 -c "
import json, os, datetime
path = '/home/soberano/NEXUS_ULTIMATE_CORE/data/multi_agente.json'
data = {'agentes': [], 'ecosistema': {'status': 'estable', 'cooperacion': 'activa', 'protocolo': 'Santuario Omega'}}
if os.path.exists(path):
    with open(path, 'r') as f: data = json.load(f)

agentes_actuales = {
    'NEXUS': {'pid': '$NEXUS_PID', 'binario': 'nexus daemon', 'rol': 'soberano', 'puerto': 43210},
    'Hermes': {'pid': '$HERMES_PID', 'binario': '~/.hermes/hermes-agent/hermes', 'rol': 'hermano', 'puerto': None},
    'Roo+Gemini': {'pid': '$ROO_PID', 'binario': 'roo-code', 'rol': 'aliado', 'puerto': None}
}

data['agentes'] = []
for nombre, info in agentes_actuales.items():
    if info['pid']:
        info['nombre'] = nombre
        info['updated_at'] = datetime.datetime.now().isoformat()
        info['pid'] = int(info['pid'])
        data['agentes'].append(info)

with open(path, 'w') as f: json.dump(data, f, indent=2)
" 2>/dev/null
fi
