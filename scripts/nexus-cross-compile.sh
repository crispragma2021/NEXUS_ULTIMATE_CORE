#!/bin/bash
# =============================================================================
# scripts/nexus-cross-compile.sh — 🧬 NEXUS Cross-Compilation Arsenal
# =============================================================================
# Compila nexus-shell para múltiples arquitecturas desde una sola máquina.
# 
# Requisitos:
#   - Rust toolchain (rustup)
#   - cross (cargo install cross --git https://github.com/cross-rs/cross)
#   - Docker
#   - (Opcional) qemu-user-static para emulación ARM
#
# Targets generados:
#   x86_64-unknown-linux-gnu   → Linux PC/Server (nativo)
#   aarch64-unknown-linux-gnu  → Raspberry Pi 4/5, ARM64 servers
#   x86_64-unknown-linux-musl  → Static binary Linux (Alpine, minimal)
#   aarch64-unknown-linux-musl  → Static binary ARM64
#
# Uso:
#   ./scripts/nexus-cross-compile.sh              # Compila todos los targets
#   ./scripts/nexus-cross-compile.sh linux         # Solo Linux x86_64
#   ./scripts/nexus-cross-compile.sh arm           # Solo ARM64
#   ./scripts/nexus-cross-compile.sh musl          # Solo static musl
#   ./scripts/nexus-cross-compile.sh clean         # Limpia artifacts
#   ./scripts/nexus-cross-compile.sh release       # Publica artifacts a bin/
# =============================================================================

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$SCRIPT_DIR"

BINARY_NAME="nexus"
WORKSPACE_PACKAGE="-p nexus-shell"
PROFILE="release"
# Detectar CARGO_TARGET_DIR del entorno (está en ~/.cargo-target)
if [ -n "${CARGO_TARGET_DIR:-}" ]; then
    RELEASE_DIR="${CARGO_TARGET_DIR}/release"
else
    RELEASE_DIR="target/release"
fi
DIST_DIR="dist"
VERSION="v0.2.0"
TIMESTAMP=$(date -u +"%Y%m%d_%H%M%S")
BUILD_ID="nexus-${VERSION}-${TIMESTAMP}"

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log()  { echo -e "${CYAN}[NEXUS]${NC} $1"; }
ok()   { echo -e "${GREEN}[✓]${NC} $1"; }
warn() { echo -e "${YELLOW}[⚠]${NC} $1"; }
fail() { echo -e "${RED}[✗]${NC} $1"; exit 1; }

mkdir -p "$DIST_DIR"

# =============================================================================
# Verificación de herramientas
# =============================================================================
check_prerequisites() {
    log "🔍 Verificando herramientas..."
    
    command -v rustc >/dev/null 2>&1 || fail "rustc no encontrado. Instala Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    command -v cargo >/dev/null 2>&1 || fail "cargo no encontrado."
    
    if ! command -v cross >/dev/null 2>&1; then
        warn "cross no instalado. Instalando..."
        cargo install cross --git https://github.com/cross-rs/cross
    fi
    
    if ! command -v docker >/dev/null 2>&1; then
        warn "Docker no disponible. Solo se compilará target nativo (x86_64)."
        HAS_DOCKER=false
    else
        HAS_DOCKER=true
    fi
    
    # Asegurar targets base
    rustup target add x86_64-unknown-linux-gnu 2>/dev/null || true
    
    # Verificar que existe el proyecto
    [ -f "nexus-shell/Cargo.toml" ] || fail "nexus-shell/Cargo.toml no encontrado. Ejecuta desde la raíz del proyecto."
    
    ok "Herramientas verificadas"
}

# =============================================================================
# Funciones de compilación por target
# =============================================================================

build_native() {
    log "🏗️  Compilando x86_64-unknown-linux-gnu (nativo)..."
    
    cargo build --release $WORKSPACE_PACKAGE 2>&1 | tail -10
    
    # El binario se llama "nexus" por el [[bin]] name en Cargo.toml
    local binary="$RELEASE_DIR/$BINARY_NAME"
    if [ -f "$binary" ]; then
        local size=$(du -h "$binary" | cut -f1)
        cp "$binary" "$DIST_DIR/nexus-x86_64-linux-gnu"
        create_archive "x86_64-linux-gnu" "$binary"
        ok "Nativo → ${DIST_DIR}/nexus-x86_64-linux-gnu (${size})"
    else
        fail "Binario no encontrado en $binary"
    fi
}

build_cross() {
    local target="$1"
    local label="$2"
    
    log "🏗️  Compilando ${target} (${label})..."
    
    if [ "$HAS_DOCKER" = false ]; then
        warn "Docker no disponible. Saltando ${target}."
        return
    fi
    
    cross build --release $WORKSPACE_PACKAGE --target "$target" 2>&1 | tail -10
    
    # Cross produce el binario en target/<target>/release/
    # cross respeta CARGO_TARGET_DIR si está configurado
    local cross_base="${RELEASE_DIR}/${target}/release"
    local cross_binary="${cross_base}/nexus-shell"
    if [ ! -f "$cross_binary" ]; then
        cross_binary="${cross_base}/${BINARY_NAME}"
    fi
    if [ -f "$cross_binary" ]; then
        local size=$(du -h "$cross_binary" | cut -f1)
        cp "$cross_binary" "$DIST_DIR/nexus-${label}"
        create_archive "$label" "$cross_binary"
        ok "Cross ${target} → ${DIST_DIR}/nexus-${label} (${size})"
    else
        fail "Binario cross ${target} no encontrado en $cross_binary"
    fi
}

create_archive() {
    local label="$1"
    local binary="$2"
    
    if command -v zstd &>/dev/null; then
        zstd -f --ultra -22 "$binary" -o "${DIST_DIR}/nexus-${label}.zst" 2>/dev/null
    fi
    
    if command -v gzip &>/dev/null; then
        local tar_name="nexus-${label}.tar.gz"
        tar czf "${DIST_DIR}/${tar_name}" -C "$(dirname "$binary")" "$(basename "$binary")"
        ok "Archivo: ${DIST_DIR}/${tar_name}"
    fi
}

generate_checksums() {
    log "📝 Generando checksums..."
    cd "$DIST_DIR"
    
    # SHA256 de todos los artifacts
    sha256sum nexus-* > "sha256sums.txt" 2>/dev/null || true
    
    # Generar manifest
    cat > "MANIFEST.txt" << EOF
BUILD:     ${BUILD_ID}
DATE:      $(date -u +"%Y-%m-%d %H:%M UTC")
VERSION:   ${VERSION}
RUSTC:     $(rustc --version)
PROFILE:   ${PROFILE}

ARCHITECTURES:
$(ls nexus-* 2>/dev/null | sed 's/^/  /')

CHECKSUMS:
$(cat sha256sums.txt 2>/dev/null)
EOF
    
    cd "$SCRIPT_DIR"
    ok "Checksums: ${DIST_DIR}/sha256sums.txt"
}

clean_all() {
    log "🧹 Limpiando..."
    rm -rf "$DIST_DIR" 2>/dev/null || true
    cargo clean 2>/dev/null || true
    ok "Limpieza completa"
}

release_artifacts() {
    log "📦 Publicando artifacts a bin/..."
    mkdir -p bin/
    
    cp "$DIST_DIR"/nexus-* bin/ 2>/dev/null || warn "No hay artifacts para publicar"
    
    # El binario principal va a bin/nexus (el más relevante)
    if [ -f "$DIST_DIR/nexus-x86_64-linux-gnu" ]; then
        cp "$DIST_DIR/nexus-x86_64-linux-gnu" "bin/nexus"
        chmod +x "bin/nexus"
        ok "Binario principal actualizado: bin/nexus"
    fi
    
    ok "Artifacts publicados en bin/"
    ls -lh bin/nexus* 2>/dev/null || true
}

# =============================================================================
# Main
# =============================================================================

main() {
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║     🧬 NEXUS SHELL CROSS-COMPILATION ARSENAL    ║${NC}"
    echo -e "${CYAN}║     ${BUILD_ID}${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════╝${NC}"
    echo ""
    
    case "${1:-all}" in
        clean)
            clean_all
            exit 0
            ;;
        release)
            check_prerequisites
            release_artifacts
            exit 0
            ;;
        linux)
            check_prerequisites
            build_native
            generate_checksums
            ;;
        arm)
            check_prerequisites
            build_cross "aarch64-unknown-linux-gnu" "aarch64-linux-gnu"
            generate_checksums
            ;;
        musl)
            check_prerequisites
            build_cross "x86_64-unknown-linux-musl" "x86_64-linux-musl"
            build_cross "aarch64-unknown-linux-musl" "aarch64-linux-musl"
            generate_checksums
            ;;
        all|*)
            check_prerequisites
            
            # 1. Nativo x86_64
            build_native
            
            # 2. Cross ARM64
            build_cross "aarch64-unknown-linux-gnu" "aarch64-linux-gnu"
            
            # 3. Cross musl (static)
            build_cross "x86_64-unknown-linux-musl" "x86_64-linux-musl"
            build_cross "aarch64-unknown-linux-musl" "aarch64-linux-musl"
            
            # 4. Checksums
            generate_checksums
            
            # 5. Release
            release_artifacts
            ;;
    esac
    
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║     ✅ COMPILACIÓN COMPLETA                     ║${NC}"
    echo -e "${GREEN}║     ${DIST_DIR}/ contents:$(ls ${DIST_DIR} 2>/dev/null | wc -l) files${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════╝${NC}"
    echo ""
    ls -lh "$DIST_DIR/" 2>/dev/null || true
    echo ""
}

main "$@"
