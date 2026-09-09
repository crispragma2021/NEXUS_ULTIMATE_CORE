#!/bin/bash
echo "=== CONFIGURANDO REDIRECCIÓN SELECTIVA DE REPETICIÓN (IPTABLES) ==="

PROXY_PORT=4444

# 1. Limpiar reglas previas en la tabla NAT
echo "-> Limpiando la tabla NAT de iptables..."
sudo iptables -t nat -F

# 2. RESOLVER Y DECLARAR IPS DE GOOGLE A EXCLUIR
echo "-> Identificando rangos y resolviendo dominios críticos de Google..."
DOMINIOS=(
    "oauth2.googleapis.com"
    "googleapis.com"
    "vertexai.googleapis.com"
    "google.com"
    "accounts.google.com"
)

# Verificar si 'dig' está instalado
if ! command -v dig &> /dev/null; then
    echo "❌ Error: 'dnsutils' (dig) no está instalado. Ejecuta: sudo apt install dnsutils"
    exit 1
fi

for dominio in "${DOMINIOS[@]}"; do
    IPS=$(dig +short $dominio | grep -E '^[0-9.]+$')
    for ip in $IPS; do
        echo "   [BYPASS] Excluyendo IP: $ip ($dominio)"
        sudo iptables -t nat -A OUTPUT -p tcp -d $ip -j ACCEPT
    done
done

# 3. EXCLUSIONES MANUALES DE SUBREDES DE GOOGLE
sudo iptables -t nat -A OUTPUT -p tcp -d 172.217.0.0/16 -j ACCEPT
sudo iptables -t nat -A OUTPUT -p tcp -d 142.250.0.0/15 -j ACCEPT

# 4. REDIRECCIÓN TRANSPARENTE GENERAL
echo "-> Redirigiendo tráfico HTTP/HTTPS restante al Proxy $PROXY_PORT..."
sudo iptables -t nat -A OUTPUT -p tcp --dport 80 -j REDIRECT --to-ports $PROXY_PORT
sudo iptables -t nat -A OUTPUT -p tcp --dport 443 -j REDIRECT --to-ports $PROXY_PORT

echo -e "\n=== ARQUITECTURA DE RED INMUTABLE Y ACTIVA ==="