#!/bin/bash
# 🔱 NEXUS OMEGA - Bypass de Proxy Táctico (Gabriel Birth)
# Este script evita el choque con el puerto 4444 para permitir la captura rápida.

echo "[+] Lanzando Brave aislando el entorno del Proxy Hijack..."

# 1. Limpieza de perfiles previos
rm -rf /tmp/brave_gabriel_bypass
mkdir -p /home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots

# 2. Ignición con Bypass de Proxy (Salto de aduana :4444)
brave-browser --headless \
  --no-proxy-server \
  --disable-gpu \
  --no-sandbox \
  --disable-dev-shm-usage \
  --disable-software-rasterizer \
  --window-size=1280,1080 \
  --user-data-dir="/tmp/brave_gabriel_bypass" \
  --screenshot="/home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots/gabriel_registro.png" \
  "https://www.facebook.com/r.php"

echo -e "\n[+] Verificando si el bypass destrabó la escritura en el disco:"
ls -la /home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots/gabriel_registro.png || echo "[-] Sigue sin crearse. Latencia sistémica residual."
