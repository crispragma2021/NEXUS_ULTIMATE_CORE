#!/bin/bash
# =============================================================================
# scripts/docker-cross-compile.sh — 🧬 NEXUS Docker Cross-Compilación Robusta
# =============================================================================
# Compila nexus-shell para múltiples arquitecturas usando Docker.
#
# Estrategia:
#   - ARM64: Emulación QEMU + `--platform linux/arm64` con rust oficial
#   - x86_64-musl: Cross-compilación estática con target musl
#
# Requisitos:
#   - Docker 20.10+
#   - QEMU binfmt registrado (multiarch/qemu-user-static)
#
# Uso:
#   ./scripts/docker-cross-compile.sh arm64    # Compila para ARM64 (QEMU)
#   ./scripts/docker-cross-compile.sh musl     # Compila x86_64-musl (estático)
#   ./scripts/docker-cross-compile.sh docker   # Construye imagen Docker
#   ./scripts/docker-cross-compile.sh all      # Compila todo
#   ./scripts/docker-cross-compile.sh help     # Muestra ayuda
#
# NOTA: La compilación ARM64 vía QEMU es MUY lenta (30-60min).
#       Para desarrollo diario, usa: ./scripts/nexus-cross-compile.sh linux
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$SCRIPT_DIR"

DIST_DIR="dist"
mkdir -p "$DIST_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log()  { echo -e "${CYAN}[NEXUS]${NC} $1"; }
ok()   { echo -e "${GREEN}[✓]${NC} $1"; }
warn() { echo -e "${YELLOW}[⚠]${NC} $1"; }
fail() { echo -e "${RED}[✗]${NC} $1"; exit 1; }

BINARY_NAME="nexus"
VERSION="${VERSION:-0.2.0}"

# Dependencias nativas para core crate (wgpu, cpal, enigo, pipewire, etc.)
NATIVE_DEPS="pkg-config libssl-dev cmake g++ ca-certificates \
    libegl1-mesa-dev libgles2-mesa-dev libgbm-dev \
    libxcb1-dev libxcb-shape0-dev libxcb-xfixes0-dev libxdo-dev \
    libasound2-dev libwayland-dev libpipewire-0.3-dev libdbus-1-dev \
    libclang-dev llvm-dev libpulse-dev"

# =============================================================================
# ARM64 — Emulación QEMU nativa
# =============================================================================
build_arm64_qemu() {
    log "🏗️  Compilando para ARM64 vía QEMU emulación..."
    log "    Usando rust:latest --platform linux/arm64"
    log "    ⚠️  Esto toma 30-60min. Prepara café."

    local container_name="nexus-build-arm64-$$"

    # Crear contenedor ARM64
    if ! docker run --platform linux/arm64 -d \
        --name "$container_name" \
        rust:latest \
        bash -c "tail -f /dev/null" 2>&1; then
        fail "No se pudo crear contenedor ARM64. ¿QEMU está registrado?"
    fi

    log "Contenedor ARM64 activo: $container_name"

    # Instalar dependencias nativas
    log "Instalando dependencias nativas (${NATIVE_DEPS})..."
    docker exec "$container_name" bash -c "
        export DEBIAN_FRONTEND=noninteractive
        apt-get update -qq
        apt-get install -y -qq --no-install-recommends ${NATIVE_DEPS} 2>/dev/null
        echo 'Dependencias ARM64 instaladas'
    " || warn "Algunas dependencias no se instalaron"

    # Copiar código al contenedor (método tar para evitar docker cp overhead)
    log "Copiando código fuente al contenedor..."
    docker exec "$container_name" mkdir -p /build
    log "Comprimiendo y enviando código (esto puede tomar un momento)..."
    tar cf - . \
        --exclude=.git \
        --exclude=target \
        --exclude=dist \
        --exclude=downloads \
        --exclude=data \
        --exclude=brain \
        --exclude=node_modules \
        --exclude=.vite \
        --exclude=reports \
        --exclude=secrets \
        --exclude=tools \
        --exclude=templates \
        --exclude=antigravity_extension | \
        docker exec -i "$container_name" bash -c 'cd /build && tar xf -'
    ok "Código copiado"

    # Verificar que el workspace member nexus-shell existe
    docker exec "$container_name" ls -la /build/nexus-shell/Cargo.toml >/dev/null 2>&1 || \
        fail "nexus-shell/Cargo.toml no encontrado en el build context"

    # Compilar
    log "⚡ Compilando ARM64 (QEMU)..."
    log "    Monitorea con: docker logs -f ${container_name}"
    if docker exec -w /build "$container_name" \
        cargo build --release -p nexus-shell 2>&1 | tail -30; then
        ok "Compilación ARM64 completada"
    else
        warn "Compilación ARM64 falló. Ver logs con: docker logs ${container_name}"
        # No salimos, intentamos extraer lo que haya
    fi

    # Extraer binario
    log "Extrayendo binario..."
    local binary_found=false

    # Buscar en ubicaciones típicas
    for candidate in \
        "/build/target/release/$BINARY_NAME" \
        "/build/target/aarch64-unknown-linux-gnu/release/$BINARY_NAME"; do
        if docker exec "$container_name" test -f "$candidate" 2>/dev/null; then
            docker cp "$container_name:$candidate" "$DIST_DIR/nexus-aarch64-linux-gnu" 2>/dev/null
            binary_found=true
            break
        fi
    done

    # Si no se encuentra, buscar recursivamente
    if [ "$binary_found" = false ]; then
        log "Buscando binario en el contenedor..."
        docker exec "$container_name" find /build/target -name "$BINARY_NAME" -type f 2>/dev/null | head -5
    fi

    # Limpiar
    docker rm -f "$container_name" 2>/dev/null || true

    # Verificar
    if [ -f "$DIST_DIR/nexus-aarch64-linux-gnu" ]; then
        local size
        size=$(du -h "$DIST_DIR/nexus-aarch64-linux-gnu" | cut -f1)
        file "$DIST_DIR/nexus-aarch64-linux-gnu"
        ok "ARM64 → dist/nexus-aarch64-linux-gnu (${size})"
    else
        warn "ARM64 build no produjo binario. Revisa los logs."
    fi
}

# =============================================================================
# x86_64-musl — Cross-compilación estática
# =============================================================================
build_musl() {
    log "🏗️  Compilando x86_64-unknown-linux-musl..."
    log "    ⚠️  NOTA: MUSL no soporta todas las dependencias nativas (EGL, Wayland, ALSA)."
    log "    El build puede fallar por dependencias del core crate (wgpu, pipewire, cpal)."

    local target="x86_64-unknown-linux-musl"
    local image="ghcr.io/cross-rs/${target}:latest"

    log "Usando imagen cross: ${image}"

    # Pull de la imagen
    docker pull "${image}" 2>&1 | tail -3

    # Cross.toml ya existe con la configuración
    if [ -f "Cross.toml" ]; then
        log "Usando configuración de Cross.toml existente"
    fi

    # Construir con cross CLI (no docker manual) para mejor integración
    if command -v cargo-cross &>/dev/null || cargo cross --help &>/dev/null 2>&1; then
        log "Usando cargo-cross (recomendado)..."
        CROSS_CONTAINER_ENGINE=docker cargo cross build --release -p nexus-shell --target "$target" 2>&1 | tail -20
    else
        log "cargo-cross no disponible. Usando cross Docker manual..."
        # Crear contenedor
        local container_name="nexus-build-musl-$$"
        docker run -d --name "$container_name" "${image}" bash -c "tail -f /dev/null" 2>&1

        # Copiar código
        docker exec "$container_name" mkdir -p /build
        tar cf - . \
            --exclude=.git --exclude=target --exclude=dist \
            --exclude=downloads --exclude=data --exclude=brain \
            --exclude=node_modules --exclude=.vite | \
            docker exec -i "$container_name" bash -c 'cd /build && tar xf -'

        # Compilar
        docker exec -w /build "$container_name" \
            cargo build --release -p nexus-shell --target "${target}" 2>&1 | tail -30

        # Extraer binario
        docker exec "$container_name" find /build/target -name "$BINARY_NAME" -type f 2>/dev/null | head -3
        for candidate in \
            "/build/target/${target}/release/$BINARY_NAME" \
            "/build/target/release/$BINARY_NAME"; do
            if docker exec "$container_name" test -f "$candidate" 2>/dev/null; then
                docker cp "$container_name:$candidate" "$DIST_DIR/nexus-x86_64-linux-musl" 2>/dev/null || true
            fi
        done

        docker rm -f "$container_name" 2>/dev/null || true
    fi

    if [ -f "$DIST_DIR/nexus-x86_64-linux-musl" ]; then
        local size
        size=$(du -h "$DIST_DIR/nexus-x86_64-linux-musl" | cut -f1)
        file "$DIST_DIR/nexus-x86_64-linux-musl"
        ok "MUSL → dist/nexus-x86_64-linux-musl (${size})"
    else
        warn "MUSL build no disponible. Las dependencias nativas del core crate (EGL, Wayland, ALSA)"
        warn "impiden la compilación estática completa. Usa el Dockerfile para despliegue."
    fi
}

# =============================================================================
# Docker Image Build
# =============================================================================
build_docker_image() {
    log "🏗️  Construyendo imagen Docker nexus-shell:latest..."

    # Verificar que existe el binario compilado nativamente
    if [ ! -f "dist/nexus" ]; then
        warn "No se encontró dist/nexus. Compilando primero..."
        if command -v cargo &>/dev/null; then
            cargo build --release -p nexus-shell 2>&1 | tail -5
            local target_dir="${CARGO_TARGET_DIR:-target}"
            cp "${target_dir}/release/nexus" "dist/nexus" 2>/dev/null || {
                warn "No se pudo copiar el binario. Compila manualmente: cargo build --release -p nexus-shell"
            }
        else
            fail "cargo no disponible. Compila primero el binario nativo."
        fi
    fi

    docker build -f docker/Dockerfile.nexus-shell \
        -t nexus-shell:latest \
        -t nexus-shell:"${VERSION}" \
        . 2>&1 | tail -10

    if docker image inspect nexus-shell:latest >/dev/null 2>&1; then
        local size
        size=$(docker images nexus-shell:latest --format '{{.Size}}')
        ok "Docker image → nexus-shell:latest (${size})"
    else
        warn "Docker build falló. Revisa docker/Dockerfile.nexus-shell"
    fi
}

# =============================================================================
# Help
# =============================================================================
print_help() {
    echo "🧬 NEXUS Docker Cross-Compiler v${VERSION}"
    echo ""
    echo "Uso: $0 {arm64|musl|docker|all|help}"
    echo ""
    echo "Subcomandos:"
    echo "  arm64    Compila para ARM64 (aarch64-linux-gnu) usando QEMU"
    echo "           REQUIERE: QEMU binfmt registrado"
    echo "           TIEMPO: 30-60min (QEMU es lento)"
    echo ""
    echo "  musl     Compila x86_64-musl (estático)"
    echo "           NOTA: Limitado por dependencias nativas (EGL, ALSA, etc.)"
    echo ""
    echo "  docker   Construye imagen Docker local desde dist/nexus"
    echo "           RÁPIDO: Solo copia binario precompilado"
    echo ""
    echo "  all      Compila arm64 + docker (salta musl por limitaciones)"
    echo ""
    echo "  help     Muestra esta ayuda"
    echo ""
    echo "Variables de entorno:"
    echo "  VERSION  Versión para etiquetar Docker (default: 0.2.0)"
    echo ""
    echo "Ejemplo rápido:"
    echo "  # Primero compila nativo:"
    echo "  ./scripts/nexus-cross-compile.sh linux"
    echo ""
    echo "  # Luego construye Docker:"
    echo "  ./scripts/docker-cross-compile.sh docker"
    echo ""
    echo "  # Para ARM64 (si tienes tiempo):"
    echo "  ./scripts/docker-cross-compile.sh arm64"
}

# =============================================================================
# Pre-flight checks
# =============================================================================
preflight_check() {
    if ! command -v docker &>/dev/null; then
        fail "Docker no encontrado. Instálalo primero."
    fi

    if ! docker info >/dev/null 2>&1; then
        fail "Docker daemon no está corriendo."
    fi

    log "🐳 Docker disponible"
}

# =============================================================================
# Main
# =============================================================================
case "${1:-help}" in
    arm64)
        preflight_check
        build_arm64_qemu
        ;;
    musl)
        preflight_check
        build_musl
        ;;
    docker)
        preflight_check
        build_docker_image
        ;;
    all)
        preflight_check
        build_arm64_qemu
        build_docker_image
        ;;
    help|--help|-h)
        print_help
        ;;
    *)
        warn "Comando desconocido: ${1:-}"
        echo "Usa: $0 {arm64|musl|docker|all|help}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}✅ Proceso completado${NC}"
echo "Artefactos en dist/:"
ls -lh "$DIST_DIR/" 2>/dev/null || true
