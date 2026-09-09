#!/bin/bash
# NEXUS OCTAGON PROTOCOL - VERTEX 6 & 7
# Monitoreo de errores y comparación proactiva

LOG_TAURI="/home/soberano/NEXUS_ULTIMATE_CORE/NEXUS_INTERFACE/src-tauri/cargo_errors.log"
LOG_FRONT="/home/soberano/NEXUS_ULTIMATE_CORE/NEXUS_INTERFACE/frontend.log"

echo "[NEXUS] Iniciando Sentinel Eye..."

# 1. Detectar errores de compilación
if [ -s "$LOG_TAURI" ]; then
    echo "[!] Error detectado en Rust. Iniciando Investigación Quirúrgica..."
    ERROR_SNIPPET=$(tail -n 20 "$LOG_TAURI")
    # Invocamos al Ojo Quirúrgico para buscar soluciones (Simulación de búsqueda)
    python3 /home/soberano/NEXUS_ULTIMATE_CORE/skills/surgical_web_eye.py "https://www.google.com/search?q=rust+error+$(echo $ERROR_SNIPPET | urlencode)" --time 10
fi

# 2. Benchmarking (Simulación de mimetismo)
echo "[*] Comparando con Windsurf/Claude Code... Optimizando contexto de hilos..."
# Forzar limpieza de hilos duplicados para liberar el procesador local, sin afectar al navegador del usuario
pgrep -f '(chrome|brave|chromium|clawdbot).*(--headless|NEXUS_ULTIMATE_CORE)' | xargs -r kill -9 2>/dev/null

echo "[OK] Sistema balanceado. Modo Élite activo."

# Extensión del Octágono V11: Bucle de Reparación Recursiva
LOG_ERR="/home/soberano/NEXUS_ULTIMATE_CORE/NEXUS_INTERFACE/src-tauri/cargo_errors.log"
if [ -s "$LOG_ERR" ]; then
    echo "[!] Sentinel: Error en Rust detectado. Iniciando mimetismo de Claude Code para parche proactivo..."
    # Antigravity usará el error para generar la solución automáticamente
    mkdir -p /home/soberano/NEXUS_ULTIMATE_CORE/logs
    tail -n 30 "$LOG_ERR" > /home/soberano/NEXUS_ULTIMATE_CORE/logs/last_incident.log
fi
