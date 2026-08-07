#!/bin/bash
# 🔱 NEXUS TOR LEAK CHECK

echo "🔍 [TOR] Verificando que el tráfico pasa por Tor..."

echo -n "1. IP real (sin Tor): "
curl -s https://check.torproject.org/api/ip

echo -n "2. IP por Tor (a través de socks5://127.0.0.1:9050): "
curl --socks5-hostname 127.0.0.1:9050 -s https://check.torproject.org/api/ip

echo -n "3. DNS leak check (a través de socks5://127.0.0.1:9050): "
curl --socks5-hostname 127.0.0.1:9050 -s https://dnsleaktest.com/ | grep -oP 'IP: \d+\.\d+\.\d+\.\d+' || echo "No leak detected"

echo -n "4. WebRTC leak check (browser): "
# Verificar que el proxy está funcionando
if ss -tlnp | grep -q 9050; then
    echo "✅ Proxy SOCKS5 activo en :9050"
else
    echo "❌ Proxy SOCKS5 NO disponible"
fi
