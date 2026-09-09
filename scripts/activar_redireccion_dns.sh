#!/bin/bash
# 🔱 NEXUS Blindaje de Red (Activar)
# Este script debe ejecutarse con sudo.
# Instala la CA en el sistema, añade las entradas de hosts e inyecta la regla iptables.

set -e

if [ "$EUID" -ne 0 ]; then
  echo "❌ Error: Este script debe ser ejecutado con privilegios de root (sudo)."
  exit 1
fi

WORKSPACE_DIR="/home/soberano/NEXUS_ULTIMATE_CORE"
SECRETS_DIR="${WORKSPACE_DIR}/secrets"
CA_FILE="${SECRETS_DIR}/nexus-ca.pem"

if [ ! -f "$CA_FILE" ]; then
  echo "❌ Error: No se encontró el certificado de la CA en $CA_FILE"
  echo "   Por favor, ejecuta primero ./scripts/generar_certificados.sh sin sudo."
  exit 1
fi

echo "🔱 [1/3] Instalando NEXUS Root CA en el almacén del sistema..."
cp "$CA_FILE" /usr/local/share/ca-certificates/nexus-ca.crt
update-ca-certificates
echo "✅ CA raíz instalada."

echo "🔱 [2/3] Configurando secuestro DNS en /etc/hosts..."
# Definir entradas
DOMINIOS=(
  "cloudcode-pa.googleapis.com"
  "generativelanguage.googleapis.com"
)

for dom in "${DOMINIOS[@]}"; do
  if ! grep -q "$dom" /etc/hosts; then
    echo "127.0.0.1 $dom" >> /etc/hosts
    echo "   + Añadido $dom a /etc/hosts"
  else
    echo "   ~ $dom ya existe en /etc/hosts"
  fi
done
echo "✅ Redirección DNS establecida."

echo "🔱 [3/3] Configurando regla iptables para desviar puerto 443 local a 8443..."
# Definir dominios y desviar dinámicamente sus IPs reales
for dom in "${DOMINIOS[@]}"; do
  IP_GOOGLE=$(dig +short "$dom" | grep -E '^[0-9.]+$' | head -1)
  if [ -n "$IP_GOOGLE" ]; then
    if ! iptables -t nat -C OUTPUT -p tcp -d "$IP_GOOGLE" --dport 443 -j REDIRECT --to-ports 8443 2>/dev/null; then
      iptables -t nat -A OUTPUT -p tcp -d "$IP_GOOGLE" --dport 443 -j REDIRECT --to-ports 8443
      echo "✅ Regla de redirección iptables añadida para $dom ($IP_GOOGLE)."
    else
      echo "   ~ La regla de redirección iptables para $dom ($IP_GOOGLE) ya está activa."
    fi
  fi
done

echo ""
echo "🎉 ¡Blindaje OMEGA Activo!"
echo "   - Tráfico HTTPS de Google Cloud Code redirigido y descifrado en el puerto 8443."
echo "   - Recuerda reiniciar VS Code para limpiar cualquier sesión de red cacheada."
echo "   - Ya puedes configurar 'http.proxyStrictSSL' a true en tus ajustes."
echo ""
