#!/bin/bash
# 🔱 NEXUS TOR DEACTIVATOR

echo "🧅 [TOR] Deteniendo daemon Tor..."
sudo systemctl stop tor 2>/dev/null || killall tor 2>/dev/null

echo "✅ [TOR] Daemon Tor detenido."
echo ""
echo "🔱 [MODO TOR DESACTIVADO]"
