#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════
# 🔱 FORJA DEL IDE NEXUS SOBERANO — build automatizado
# Optimizado para Intel Core i7-12700F (12 Cores, 20 Threads)
#
# Flujo: get_repo.sh (clona fuentes VSCode) → prepare_vscode.sh (patches)
#        → build.sh (compila + empaqueta)
# ═══════════════════════════════════════════════════════════════════════
set -euo pipefail

# ─── Toolchain: activar Node 22.22.1 (requerido por VSCode 1.121) ─────
export NVM_DIR="$HOME/.nvm"
if [[ -s "$NVM_DIR/nvm.sh" ]]; then
  # shellcheck source=/dev/null
  . "$NVM_DIR/nvm.sh"
  nvm use 22.22.1 > /dev/null 2>&1 || nvm install 22.22.1 > /dev/null 2>&1
fi
echo "Node: $(node --version) | npm: $(npm --version)"

cd "$(dirname "$0")/../vscodium"
echo "🔱 Forjando NEXUS IDE en: $(pwd)"

# ─── Identidad NEXUS (branding OMEGA) ─────────────────────────────────
export APP_NAME="NEXUS IDE"
export APP_NAME_LC="nexus-ide"
export BINARY_NAME="nexus-ide"
export ORG_NAME="NEXUS"
export ASSETS_REPOSITORY="NEXUS/nexus-ide"
export GH_REPO_PATH="NEXUS/nexus-ide"
export TUNNEL_APP_NAME="nexus-ide-tunnel"
export GLOBAL_DIRNAME="nexus-ide"

# ─── Objetivo: stable Linux x64 ───────────────────────────────────────
export VSCODE_QUALITY="stable"
export VSCODE_ARCH="x64"
export OS_NAME="linux"
export CI_BUILD="no"
export SHOULD_BUILD="yes"
export SHOULD_BUILD_REH="no"
export SHOULD_BUILD_REH_WEB="no"
export DISABLE_UPDATE="yes"

# ─── Optimización i7-12700F (12 núcleos / 20 hilos) ──────────────────
export JOBS=16
export NODE_OPTIONS="--max-old-space-size=8192"
export RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C codegen-units=1"
# Si gulp usa JOBS, lo exportamos; el número de hilos de build lo marca ninja
export VSCODE_MAKER="ninja"

echo "── Identidad:"
echo "   APP_NAME=${APP_NAME}"
echo "   BINARY_NAME=${BINARY_NAME}"
echo "   Objetivo: ${VSCODE_QUALITY}-${OS_NAME}-${VSCODE_ARCH}"

# ─── Paso 1: Clonar/actualizar fuentes de VSCode ─────────────────────
if [[ ! -d "vscode/.git" ]]; then
  echo "── [1/4] Clonando fuentes de VSCode (Code OSS)..."
  bash get_repo.sh
else
  echo "── [1/4] Fuentes ya clonadas, actualizando..."
  bash get_repo.sh
fi

# ─── Paso 2: Aplicar patches de transmutación ────────────────────────
echo "── [2/4] Aplicando patches (branding OMEGA + sidebar + tema)..."
bash prepare_vscode.sh

# ─── Paso 3: Compilar (vscode-min-prepack) ───────────────────────────
echo "── [3/4] Compilando (vscode-min-prepack) con ${JOBS} jobs..."
cd vscode
npm run gulp vscode-min-prepack
cd ..

# ─── Paso 4: Empaquetar .deb (Linux) ─────────────────────────────────
echo "── [4/4] Empaquetando release Linux-x64..."
bash build.sh

echo "✅ FORJA COMPLETADA"
echo "   Artefacto esperado: ../VSCode-linux-x64/"
ls -la ../VSCode-linux-x64/ 2>/dev/null | head -20 || echo "   (revisar salida de build.sh)"
