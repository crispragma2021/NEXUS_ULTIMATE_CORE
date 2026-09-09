#!/bin/bash
# 🔱 NEXUS OMEGA - Ráfaga de Fuego de Gabriel (Infiltración Limpia)
# Este script purga bloqueos y captura el registro de Gabriel.

echo "[+] Limpiando rastros y ejecutando ráfaga limpia..."

# 1. Purgar perfiles previos para eliminar SingletonLock pegados
rm -rf /tmp/brave_gabriel_final
rm -rf /tmp/brave_gabriel_diagnostic
mkdir -p /home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots

# 2. Ignición de Brave con Flags de Estabilidad Absoluta
brave-browser --headless \
  --disable-gpu \
  --no-sandbox \
  --disable-dev-shm-usage \
  --disable-software-rasterizer \
  --window-size=1280,1080 \
  --user-data-dir="/tmp/brave_gabriel_final" \
  --screenshot="/home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots/gabriel_registro.png" \
  "https://www.facebook.com/r.php"

echo -e "\n[+] Verificando la captura generada en el disco:"
ls -la /home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots/gabriel_registro.png || echo "[-] Sigue sin crearse. Sistema bajo latencia residual."
