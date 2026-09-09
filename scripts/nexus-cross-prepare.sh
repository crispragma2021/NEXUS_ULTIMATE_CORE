#!/bin/bash
# scripts/nexus-cross-prepare.sh — Pre-build hook para cross-compilación
# Se ejecuta dentro del contenedor cross ANTES de compilar
set -euo pipefail

echo "🧬 [PREPARE] Instalando dependencias del sistema para cross-compilación NEXUS..."
echo "   Host: $(uname -m) | Target: ${CROSS_TARGET:-unknown}"

if command -v apt-get &>/dev/null; then
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq 2>/dev/null || true
    apt-get install -y -qq --no-install-recommends \
        pkg-config \
        libssl-dev \
        cmake \
        ca-certificates \
        tini \
        2>/dev/null || echo "⚠️  [PREPARE] Algunos paquetes no se instalaron (no crítico)"
fi

echo "✅ [PREPARE] Ready: $(rustc --version 2>/dev/null || echo 'rustc listo')"
