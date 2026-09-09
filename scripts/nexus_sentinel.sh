#!/bin/bash
# NEXUS SENTINEL - Vértice 6 del Octágono
while true; do
  if [ -s "/home/soberano/NEXUS_ULTIMATE_CORE/NEXUS_INTERFACE/src-tauri/cargo_errors.log" ]; then
    echo "[!] Error detectado. Modo Escritura Silenciada: Antigravity iniciando protocolo autónomo..."
    echo "REPAIR_REQUIRED" > /home/soberano/NEXUS_ULTIMATE_CORE/.nexus_repair_flag
    # Notificación al Telegram configurado en el ADN
    curl -s -X POST https://api.telegram.org/bot/YOUR_BOT_TOKEN/sendMessage -d chat_id=8472077868 -d text="NEXUS [SILENT]: Error detectado. Antigravity procediendo con parche autónomo."
    # No romper el ciclo, dejar que el agente lo limpie después
    sleep 30 
  fi
  sleep 5
done
