#!/bin/bash
# 🔱 NEXUS Proxy Redirection Deactivator
# Uso: source scripts/proxy_off.sh

unset HTTP_PROXY
unset HTTPS_PROXY
unset http_proxy
unset https_proxy
unset no_proxy
unset NO_PROXY

echo "🔱 [PROXY] Redirección desactivada. Conexión directa a internet restaurada."
