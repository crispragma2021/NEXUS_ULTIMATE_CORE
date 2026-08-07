#!/bin/bash
# 🔱 NEXUS OMEGA - Diagnóstico de Infiltración Avanzado (Gabriel Birth)
# Forzando logs de error y estabilidad de SHM.

echo "[+] Lanzando Brave con Flags de Diagnóstico Avanzado..."

# 🛠️ Flags de Estabilidad OMEGA:
# --headless=new: Motor nativo para segundo plano.
# --disable-dev-shm-usage: Evita el colapso por falta de espacio en /dev/shm.
# --disable-software-rasterizer: Previene fallos de renderizado sin GPU.
# --enable-logging: Captura el dolor del kernel en stderr.

brave-browser --headless=new \
  --disable-gpu \
  --no-sandbox \
  --disable-dev-shm-usage \
  --disable-software-rasterizer \
  --window-size=1280,1080 \
  --user-data-dir="/tmp/brave_gabriel_diagnostic" \
  --enable-logging=stderr --v=1 \
  --screenshot="/home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots/gabriel_registro.png" \
  "https://www.facebook.com/r.php" 2>&1 | head -n 20

echo -e "\n[+] Verificando si esta vez sobrevivió el archivo:"
ls -la /home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots/gabriel_registro.png 2>/dev/null || echo "[-] Sigue sin crearse. Revisar los errores de arriba."
