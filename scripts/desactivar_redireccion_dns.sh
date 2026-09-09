#!/bin/bash
# 🔱 NEXUS Blindaje de Red (Desactivar)
# Este script restaura la configuración original de red.

set -e

if [ "$EUID" -ne 0 ]; then
  echo "❌ Error: Debe ejecutarse con sudo."
  exit 1
fi

WORKSPACE_DIR="/home/soberano/NEXUS_ULTIMATE_CORE"
DOMINIOS=(
  "cloudcode-pa.googleapis.com"
  "generativelanguage.googleapis.com"
)

for dom in "${DOMINIOS[@]}"; do
  IP_GOOGLE=$(dig +short "$dom" | grep -E '^[0-9.]+$' | head -1)
  if [ -n "$IP_GOOGLE" ]; then
    if iptables -t nat -C OUTPUT -p tcp -d "$IP_GOOGLE" --dport 443 -j REDIRECT --to-ports 8443 2>/dev/null; then
      iptables -t nat -D OUTPUT -p tcp -d "$IP_GOOGLE" --dport 443 -j REDIRECT --to-ports 8443
      echo "✅ Redirección eliminada para $dom ($IP_GOOGLE)."
    else
      echo "   ~ No se detectó regla activa para $dom."
    fi
  fi
done

# Compatibilidad con la regla antigua de interfaz local (loopback)
if iptables -t nat -C OUTPUT -p tcp -o lo --dport 443 -j REDIRECT --to-ports 8443 2>/dev/null; then
  iptables -t nat -D OUTPUT -p tcp -o lo --dport 443 -j REDIRECT --to-ports 8443
  echo "✅ Antigua regla loopback eliminada."
fi

echo "🔱 [2/2] Nota: Las entradas en /etc/hosts permanecen para mimetismo."
echo "🎉 Tráfico SSL directo restaurado, Arquitecto."