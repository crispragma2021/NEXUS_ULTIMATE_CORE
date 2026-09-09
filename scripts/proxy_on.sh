#!/bin/bash
# 🔱 NEXUS Proxy Redirection Activator
# Uso: source scripts/proxy_on.sh [tor]

if [ "$1" = "tor" ]; then
    export PROXY_HIJACK_TOR=1
    echo "🔱 [PROXY HIJACK] Modo Tor activado: proxy_hijack encadenado a SOCKS5 :9050"
else
    export PROXY_HIJACK_TOR=0
fi

export HTTP_PROXY="http://127.0.0.1:4444"
export HTTPS_PROXY="http://127.0.0.1:4444"
export http_proxy="http://127.0.0.1:4444"
export https_proxy="http://127.0.0.1:4444"
export no_proxy="localhost,127.0.0.1"
export NO_PROXY="localhost,127.0.0.1"

echo "🔱 [PROXY] Redirección activada hacia http://127.0.0.1:4444"
echo "   (Nota: Las peticiones HTTPS genéricas que no sean interceptadas por el proxy local pueden fallar)"

