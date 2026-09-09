#!/usr/bin/env bash
# ============================================================================
# limpiar_disco.sh — Limpieza automática de basura regenerable
# Ejecuta: cargo-sweep (builds >30 días), journal vacuum, apt clean, bleachbit
# Programado semanalmente por systemd-timer (ver limpiar-diskos.timer/service)
# ============================================================================
set -uo pipefail

FECHA=$(date '+%Y-%m-%d %H:%M:%S')
LOG="/home/soberano/.cache/limpiar_disco.log"
mkdir -p "$(dirname "$LOG")"
echo "=== [$FECHA] INICIO ===" >> "$LOG"

# Solo si el disco supera 70% de uso (evita trabajo innecesario)
USO=$(df --output=pcent / | tail -1 | tr -d ' %')
if [ "${USO:-0}" -lt 70 ]; then
  echo "Disco al ${USO}% (<70%) — sin limpieza necesaria." >> "$LOG"
  exit 0
fi
echo "Disco al ${USO}% — limpiando..." >> "$LOG"

# 1) cargo-sweep: borra builds de Cargo con más de 30 días en ~/.cargo-target
if command -v cargo-sweep >/dev/null 2>&1; then
  cd /home/soberano/NEXUS_ULTIMATE_CORE || exit 1
  echo "--- cargo-sweep (~/.cargo-target) ---" >> "$LOG"
  cargo sweep --time 30 >> "$LOG" 2>&1 || true

  echo "--- cargo-sweep (target local) ---" >> "$LOG"
  env -u CARGO_TARGET_DIR cargo sweep --time 30 >> "$LOG" 2>&1 || true
else
  echo "cargo-sweep no instalado — omitido" >> "$LOG"
fi

# 2) journal de systemd: mantener a máximo 400 MB
if [ -x "$(command -v journalctl)" ]; then
  echo "--- journal vacuum ---" >> "$LOG"
  sudo journalctl --vacuum-size=400M >> "$LOG" 2>&1 || true
fi

# 3) apt: limpiar caché de paquetes
if [ -x "$(command -v apt-get)" ]; then
  echo "--- apt clean ---" >> "$LOG"
  sudo apt-get clean >> "$LOG" 2>&1 || true
fi

# 4) bleachbit: cachés de sistema, trash, temporales (sin journal — no existe en 5.0)
if [ -x "$(command -v bleachbit)" ]; then
  echo "--- bleachbit ---" >> "$LOG"
  sudo bleachbit --clean system.cache system.trash system.tmp >> "$LOG" 2>&1 || true
fi

# Resumen final
echo "--- Resultado ---" >> "$LOG"
df -h / | tail -1 >> "$LOG"
echo "=== [$FECHA] FIN ===" >> "$LOG"
