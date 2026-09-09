#!/bin/bash
echo "[+] Lanzando Brave de forma limpia y segura como usuario soberano..."
mkdir -p /home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots

brave-browser --headless --disable-gpu --no-sandbox --window-size=1280,1080 \
  --user-data-dir="/tmp/brave_gabriel_sano" \
  --screenshot="/home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots/gabriel_registro.png" \
  "https://www.facebook.com/r.php"

echo "[+] Proceso finalizado. Comprobando archivo generado:"
ls -la /home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots/gabriel_registro.png
