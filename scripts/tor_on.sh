#!/bin/bash
# 🔱 NEXUS TOR ACTIVATOR
# Enfoque A: Proxy SOCKS5 ligero

echo "🧅 [TOR] Verificando instalación..."
if ! command -v tor &> /dev/null; then
    echo "❌ Tor no está instalado. Ejecuta scripts/tor_setup.sh primero."
    exit 1
fi

echo "🧅 [TOR] Iniciando daemon Tor..."
sudo systemctl start tor 2>/dev/null || tor --quiet &

echo "⏳ [TOR] Esperando que el circuito esté listo..."
sleep 5

echo "🧅 [TOR] Verificando puerto SOCKS5..."
if ss -tlnp | grep -q 9050; then
    echo "✅ [TOR] Puerto SOCKS5 127.0.0.1:9050 ACTIVO"
else
    echo "❌ [TOR] Puerto 9050 NO disponible"
    exit 1
fi

echo "🌐 [TOR] Verificando IP de salida..."
TOR_IP=$(curl --socks5-hostname 127.0.0.1:9050 -s https://check.torproject.org/api/ip 2>/dev/null)
echo "   IP Tor: $TOR_IP"

echo ""
echo "🔱 [MODO TOR ACTIVADO]"
echo "   - ShadowCrawl utilizará SOCKS5 automáticamente"
echo "   - Puppeteer usará --proxy-server=socks5://127.0.0.1:9050"
echo "   - reqwest enrutado a través de Tor"
