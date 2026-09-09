#!/bin/bash
# =============================================================================
# scripts/install.sh — 🧬 NEXUS Shell Installer (curl | bash)
# =============================================================================
# Instalador oficial de NEXUS Shell para Linux (x86_64, ARM64)
#
# Uso:
#   curl -fsSL https://nexus.soberano.sh/install.sh | bash
#   curl -fsSL https://nexus.soberano.sh/install.sh | bash -s -- --daemon
#   curl -fsSL https://nexus.soberano.sh/install.sh | bash -s -- --version v0.2.0
#
# Opciones:
#   --daemon       Instalar y activar el servicio daemon systemd
#   --version      Versión específica a instalar (default: latest)
#   --dir          Directorio de instalación (default: ~/.local/bin)
#   --help         Muestra esta ayuda
#
# Modo offline (si ya tienes el binario descargado):
#   ./scripts/install.sh --offline ./nexus-x86_64-linux-gnu
# =============================================================================

set -euo pipefail

# =============================================================================
# Configuración
# =============================================================================
REPO_URL="https://github.com/soberano/nexus-ultimate-core"
RAW_URL="https://raw.githubusercontent.com/soberano/nexus-ultimate-core/main"
RELEASES_URL="https://github.com/soberano/nexus-ultimate-core/releases/download"
DEFAULT_VERSION="latest"
INSTALL_DIR="${HOME}/.local/bin"
DATA_DIR="${HOME}/.nexus"
SYSTEMD_DIR="${HOME}/.config/systemd/user"
BINARY_NAME="nexus"
SERVICE_NAME="nexus-shell"

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# =============================================================================
# Detección de arquitectura
# =============================================================================
detect_architecture() {
    local arch
    arch=$(uname -m)
    local os
    os=$(uname -s)
    
    case "${os}-${arch}" in
        Linux-x86_64)
            echo "x86_64-linux-gnu"
            ;;
        Linux-aarch64|Linux-arm64)
            echo "aarch64-linux-gnu"
            ;;
        Linux-armv7l|Linux-armv6l)
            echo "armv7-linux-gnueabihf"
            ;;
        Darwin-x86_64)
            echo "x86_64-apple-darwin"
            ;;
        Darwin-arm64)
            echo "aarch64-apple-darwin"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

ARCH=$(detect_architecture)
VERSION="${DEFAULT_VERSION}"
INSTALL_DAEMON=false
OFFLINE_BINARY=""

# =============================================================================
# Parseo de argumentos
# =============================================================================
parse_args() {
    # Primer pase: --help es prioritario, funciona en cualquier posición
    for arg in "$@"; do
        if [[ "$arg" == "--help" || "$arg" == "-h" ]]; then
            echo "🧬 NEXUS Shell Installer"
            echo ""
            echo "Uso: curl -fsSL https://nexus.soberano.sh/install.sh | bash"
            echo "       curl -fsSL https://nexus.soberano.sh/install.sh | bash -s -- --daemon"
            echo "       curl -fsSL https://nexus.soberano.sh/install.sh | bash -s -- --offline ./nexus-x86_64-linux-gnu"
            echo ""
            echo "Opciones:"
            echo "  --daemon           Instalar y activar servicio systemd"
            echo "  --version vX.X.X   Versión específica a instalar"
            echo "  --dir PATH         Directorio de instalación (default: ~/.local/bin)"
            echo "  --offline PATH     Instalar desde binario local"
            echo "  --help             Esta ayuda"
            echo ""
            echo "Ejemplos:"
            echo "  ./install.sh --offline ./dist/nexus-x86_64-linux-gnu"
            echo "  ./install.sh --offline ./dist/nexus-x86_64-linux-gnu --dir ~/my-bin"
            echo "  ./install.sh --daemon --version v0.2.0"
            exit 0
        fi
    done

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --daemon|-d)
                INSTALL_DAEMON=true
                shift
                ;;
            --version|-v)
                if [[ -z "$2" || "$2" == -* ]]; then
                    echo -e "${RED}❌ --version requiere un argumento (ej: --version v0.2.0)${NC}"
                    exit 1
                fi
                VERSION="$2"
                shift 2
                ;;
            --dir)
                if [[ -z "$2" || "$2" == -* ]]; then
                    echo -e "${RED}❌ --dir requiere una ruta (ej: --dir ~/.local/bin)${NC}"
                    exit 1
                fi
                INSTALL_DIR="$2"
                shift 2
                ;;
            --offline)
                if [[ -z "$2" || "$2" == -* ]]; then
                    echo -e "${RED}❌ --offline requiere la ruta al binario (ej: --offline ./nexus-x86_64-linux-gnu)${NC}"
                    exit 1
                fi
                OFFLINE_BINARY="$2"
                shift 2
                ;;
            --help|-h)
                # Ya se manejó en el pase previo, pero por si acaso
                shift
                ;;
            *)
                echo -e "${RED}❌ Opción desconocida: $1${NC}"
                echo "Usa --help para ver las opciones disponibles."
                exit 1
                ;;
        esac
    done
}

# =============================================================================
# Funciones de instalación
# =============================================================================

log()  { echo -e "${CYAN}[NEXUS]${NC} $1"; }
ok()   { echo -e "${GREEN}[✓]${NC} $1"; }
warn() { echo -e "${YELLOW}[⚠]${NC} $1"; }
fail() { echo -e "${RED}[✗]${NC} $1"; exit 1; }

print_banner() {
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║     🧬 NEXUS SHELL INSTALLER                    ║${NC}"
    echo -e "${CYAN}║     Versión: ${VERSION} | Arch: ${ARCH}${NC}"
    echo -e "${CYAN}║     $(date -u +"%Y-%m-%d %H:%M UTC")${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════╝${NC}"
    echo ""
}

install_from_offline() {
    log "📦 Instalando desde binario local: ${OFFLINE_BINARY}"
    
    if [ ! -f "$OFFLINE_BINARY" ]; then
        fail "Binario no encontrado: ${OFFLINE_BINARY}"
    fi
    
    mkdir -p "$INSTALL_DIR"
    chmod +x "$OFFLINE_BINARY"
    cp "$OFFLINE_BINARY" "${INSTALL_DIR}/${BINARY_NAME}"
    
    ok "Binario instalado: ${INSTALL_DIR}/${BINARY_NAME}"
    return 0
}

install_from_release() {
    local binary_url="${RELEASES_URL}/${VERSION}/nexus-${ARCH}"
    local binary_url_fallback="${RAW_URL}/dist/nexus-${ARCH}"
    
    log "🌐 Descargando desde release..."
    log "   URL: ${binary_url}"
    
    mkdir -p "$INSTALL_DIR"
    
    # Intentar release primero, fallback a raw
    if command -v curl &>/dev/null; then
        if curl -fsSL "$binary_url" -o "${INSTALL_DIR}/${BINARY_NAME}" 2>/dev/null; then
            ok "Descargado desde release"
        elif curl -fsSL "$binary_url_fallback" -o "${INSTALL_DIR}/${BINARY_NAME}" 2>/dev/null; then
            ok "Descargado desde main branch"
        else
            fail "No se pudo descargar el binario. Verifica la conectividad."
        fi
    elif command -v wget &>/dev/null; then
        if wget -q "$binary_url" -O "${INSTALL_DIR}/${BINARY_NAME}" 2>/dev/null; then
            ok "Descargado desde release"
        elif wget -q "$binary_url_fallback" -O "${INSTALL_DIR}/${BINARY_NAME}" 2>/dev/null; then
            ok "Descargado desde main branch"
        else
            fail "No se pudo descargar el binario."
        fi
    else
        fail "Se necesita curl o wget para descargar."
    fi
    
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
}

verify_installation() {
    log "🔍 Verificando instalación..."
    
    if [ ! -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        fail "Binario no encontrado en ${INSTALL_DIR}/${BINARY_NAME}"
    fi
    
    local size
    size=$(du -h "${INSTALL_DIR}/${BINARY_NAME}" | cut -f1)
    
    # Verificar que es ejecutable y no un script vacío
    local file_type
    file_type=$(file "${INSTALL_DIR}/${BINARY_NAME}")
    
    if echo "$file_type" | grep -q "ELF"; then
        ok "Binario ELF válido (${size})"
    else
        warn "Tipo de archivo inesperado: ${file_type}"
    fi
    
    # Verificar PATH
    if echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
        ok "${INSTALL_DIR} está en PATH"
    else
        warn "${INSTALL_DIR} no está en PATH"
        log "   Añádelo: echo 'export PATH=\"\$PATH:${INSTALL_DIR}\"' >> ~/.bashrc"
        
        # Auto-añadir
        if ! grep -q "NEXUS" ~/.bashrc 2>/dev/null; then
            echo "" >> ~/.bashrc
            echo "# NEXUS Shell PATH" >> ~/.bashrc
            echo "export PATH=\"\$PATH:${INSTALL_DIR}\"" >> ~/.bashrc
            ok "PATH añadido a ~/.bashrc"
        fi
    fi
}

setup_data_dirs() {
    log "📁 Creando directorios de datos..."
    
    mkdir -p "$DATA_DIR"/{logs,config,services}
    
    # Configuración por defecto
    local config_file="${DATA_DIR}/config.toml"
    if [ ! -f "$config_file" ]; then
        cat > "$config_file" << 'CONFIG'
# 🧬 NEXUS Shell — Configuración
[server]
host = "127.0.0.1"
port = 8080

[modes]
default = "cli"

[storage]
data_dir = "~/.nexus"
CONFIG
        ok "Configuración creada: ${config_file}"
    fi
    
    ok "Directorios de datos listos"
}

setup_systemd() {
    log "⚙️  Configurando servicio systemd (user)..."

    mkdir -p "$SYSTEMD_DIR"
    
    local service_file="${SYSTEMD_DIR}/${SERVICE_NAME}.service"
    
    cat > "$service_file" << SERVICE
[Unit]
Description=🧬 NEXUS Shell Daemon — El Cuerpo Soberano
Documentation=https://nexus.soberano.sh
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=${INSTALL_DIR}/${BINARY_NAME} daemon
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info
Environment=NEXUS_CONFIG=${DATA_DIR}/config.toml

# Hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=read-only
PrivateTmp=true
MemoryDenyWriteExecute=true
LockPersonality=true

[Install]
WantedBy=default.target
SERVICE
    
    # Recargar systemd y habilitar
    systemctl --user daemon-reload 2>/dev/null || true
    
    ok "Servicio systemd creado: ${service_file}"
    
    if [ "$INSTALL_DAEMON" = true ]; then
        log "🚀 Activando servicio..."
        systemctl --user enable "${SERVICE_NAME}" 2>/dev/null || warn "No se pudo habilitar el servicio"
        systemctl --user start "${SERVICE_NAME}" 2>/dev/null || warn "No se pudo iniciar el servicio"
        
        sleep 1
        local status
        status=$(systemctl --user is-active "${SERVICE_NAME}" 2>/dev/null || echo "unknown")
        if [ "$status" = "active" ]; then
            ok "Servicio ${SERVICE_NAME} activo"
        else
            warn "Estado del servicio: ${status}. Inicia con: systemctl --user start ${SERVICE_NAME}"
        fi
    fi
}

test_binary() {
    log "🧪 Probando binario..."
    
    local help_output
    help_output=$("${INSTALL_DIR}/${BINARY_NAME}" --help 2>&1 || true)
    
    if echo "$help_output" | grep -qi "NEXUS\|usage\|cli\|daemon"; then
        ok "Binario responde correctamente"
        echo ""
        echo -e "${CYAN}┌─ Comandos disponibles ─────────────────────┐${NC}"
        echo "$help_output" | head -20
        echo -e "${CYAN}└──────────────────────────────────────────────┘${NC}"
    else
        warn "El binario no produjo la salida esperada:"
        echo "$help_output" | head -5
    fi
}

print_summary() {
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║     ✅ NEXUS SHELL INSTALADO CORRECTAMENTE         ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "  ${BOLD}Binario:${NC}     ${INSTALL_DIR}/${BINARY_NAME}"
    echo -e "  ${BOLD}Versión:${NC}     ${VERSION}"
    echo -e "  ${BOLD}Arquitectura:${NC} ${ARCH}"
    echo -e "  ${BOLD}Datos:${NC}       ${DATA_DIR}"
    echo ""
    echo -e "  ${BOLD}Comandos:${NC}"
    echo -e "    ${CYAN}nexus --help${NC}        → Ayuda general"
    echo -e "    ${CYAN}nexus daemon${NC}         → Iniciar servidor"
    echo -e "    ${CYAN}nexus status${NC}         → Ver estado"
    echo ""
    echo -e "  ${BOLD}Servicio (si aplica):${NC}"
    echo -e "    ${CYAN}systemctl --user start nexus-shell${NC}"
    echo -e "    ${CYAN}journalctl --user -u nexus-shell -f${NC}"
    echo ""
    echo -e "  ${YELLOW}💡 Para actualizar: repite este comando.${NC}"
    echo ""

    # Opción de autoinicio
    echo -e -n "${YELLOW}¿Deseas iniciar el daemon ahora? [s/N]:${NC} "
    read -r answer </dev/tty 2>/dev/null || echo ""
    if [[ "$answer" =~ ^[sS]$ ]]; then
        "${INSTALL_DIR}/${BINARY_NAME}" daemon &
        echo -e "${GREEN}✅ Daemon iniciado en segundo plano${NC}"
    fi
}

# =============================================================================
# Main
# =============================================================================

main() {
    parse_args "$@"
    print_banner
    
    # Verificar SO
    if [ "$ARCH" = "unknown" ]; then
        fail "Arquitectura no soportada: $(uname -m). Soportadas: x86_64, aarch64/ARM64"
    fi
    
    # Instalación
    if [ -n "$OFFLINE_BINARY" ]; then
        install_from_offline
    else
        install_from_release
    fi
    
    verify_installation
    setup_data_dirs
    setup_systemd
    test_binary
    print_summary
    
    ok "🧬 NEXUS Shell listo para servir."
}

main "$@"
