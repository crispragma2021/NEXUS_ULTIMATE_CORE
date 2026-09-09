#!/bin/bash
# =============================================================================
# 🔱 NEXUS AUTO-START SCRIPT — Inicio Autónomo del Agente Soberano
# =============================================================================
# Versión: 2.0.0-omega
# Propósito: Iniciar/verificar todos los componentes de NEXUS sin intervención
# del Arquitecto. Detecta el IDE disponible, lanza el agente en modo auto,
# verifica Santuario web, base OCEAN y NerveSystem.
#
# Directiva: OMEGA-6 — Ignición Activa y Autonomía Total
# =============================================================================

set -euo pipefail

# --- CONSTANTES SOBERANAS ---
NEXUS_HOME="${NEXUS_HOME:-/home/soberano/NEXUS_ULTIMATE_CORE}"
NEXUS_LOG="${NEXUS_LOG:-/tmp/nexus-auto.log}"
SANTUARIO_PORT="${SANTUARIO_PORT:-1420}"
OCEAN_DB="${OCEAN_DB:-${NEXUS_HOME}/data/intelligence.db}"
NODE_BIN=""
NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

# --- IDs de ventana de terminal para VSCode/Codium ---
ROO_TERMINAL_NAME="NEXUS-AUTO"

# =============================================================================
# FUNCIONES DE INFRAESTRUCTURA
# =============================================================================

log() {
    local level="${2:-INFO}"
    echo "[${TIMESTAMP}] [${level}] $1" | tee -a "${NEXUS_LOG}"
}

detect_node() {
    # Intentar cargar NVM primero
    if [ -s "${NVM_DIR}/nvm.sh" ]; then
        # shellcheck source=/dev/null
        \. "${NVM_DIR}/nvm.sh" 2>/dev/null
        NODE_BIN="$(command -v node 2>/dev/null || true)"
    fi

    # Fallback: buscar node en PATH
    if [ -z "${NODE_BIN}" ]; then
        NODE_BIN="$(command -v node 2>/dev/null || true)"
    fi

    if [ -z "${NODE_BIN}" ]; then
        log "❌ Node.js no encontrado. Instalálo con nvm o apt." "ERROR"
        return 1
    fi

    log "✅ Node.js detectado: ${NODE_BIN} ($(node --version 2>/dev/null || echo 'unknown'))"
    return 0
}

detect_ide() {
    # Detectar qué IDE/editor está disponible
    local ide=""

    if command -v codium &>/dev/null; then
        ide="codium"
        log "🖥️ IDE detectado: VSCodium"
    elif command -v code &>/dev/null; then
        ide="code"
        log "🖥️ IDE detectado: VS Code"
    elif command -v antigravity &>/dev/null; then
        ide="antigravity"
        log "🖥️ IDE detectado: Antigravity IDE"
    elif [ -d "${NEXUS_HOME}/antigravity_extension" ]; then
        ide="vscode-with-extension"
        log "🖥️ IDE: VS Code con extensión Antigravity detectada"
    else
        log "⚠️ Ningún IDE conocido detectado. Usando modo headless." "WARN"
        ide="headless"
    fi

    echo "${ide}"
}

# =============================================================================
# VERIFICACIONES DE COMPONENTES
# =============================================================================

check_santuario() {
    local port="${1:-${SANTUARIO_PORT}}"
    log "🔍 Verificando Santuario web (puerto ${port})..."

    if curl -s -o /dev/null -w "%{http_code}" "http://localhost:${port}/" 2>/dev/null | grep -qE '^(200|302|304)'; then
        log "✅ Santuario web corriendo en puerto ${port}"
        return 0
    fi

    log "⚠️ Santuario web NO responde. Intentando iniciar..." "WARN"
    return 1
}

start_santuario() {
    local port="${1:-${SANTUARIO_PORT}}"
    log "🚀 Iniciando Santuario web en puerto ${port}..."

    # Buscar vite en node_modules
    local vite_bin=""
    if [ -f "${NEXUS_HOME}/node_modules/.bin/vite" ]; then
        vite_bin="${NEXUS_HOME}/node_modules/.bin/vite"
    elif command -v vite &>/dev/null; then
        vite_bin="$(command -v vite)"
    fi

    if [ -n "${vite_bin}" ]; then
        cd "${NEXUS_HOME}"
        nohup "${vite_bin}" --host 0.0.0.0 --port "${port}" \
            > /tmp/nexus-santuario.log 2>&1 &
        local pid=$!
        log "✅ Santuario iniciado con PID ${pid}"
        # Esperar a que responda
        for i in $(seq 1 10); do
            sleep 1
            if curl -s -o /dev/null -w "%{http_code}" "http://localhost:${port}/" 2>/dev/null | grep -qE '^(200|302|304)'; then
                log "✅ Santuario respondiendo después de ${i}s"
                break
            fi
        done
    else
        log "❌ vite no encontrado. No se pudo iniciar Santuario." "ERROR"
        return 1
    fi
}

check_ocean() {
    log "🔍 Verificando base OCEAN (memoria emocional)..."
    local ocean_path="${OCEAN_DB}"

    if [ -f "${ocean_path}" ]; then
        local size
        size=$(du -h "${ocean_path}" 2>/dev/null | cut -f1)
        log "✅ Base OCEAN accesible: ${ocean_path} (${size})"
        return 0
    fi

    # Fallback: buscar en ubicaciones alternativas
    for alt in \
        "${NEXUS_HOME}/nexus_intelligence.db" \
        "${NEXUS_HOME}/data/intelligence.db" \
        "${HOME}/.nexus_data/nexus.db"; do
        if [ -f "${alt}" ]; then
            local size
            size=$(du -h "${alt}" 2>/dev/null | cut -f1)
            log "✅ Base OCEAN encontrada en alternativa: ${alt} (${size})"
            OCEAN_DB="${alt}"
            return 0
        fi
    done

    log "⚠️ Base OCEAN no encontrada. Init diferido." "WARN"
    return 1
}

check_nerve_system() {
    log "🔍 Verificando NerveSystem..."

    # Buscar binario nerve
    local nerve_bin=""
    for candidate in \
        "${NEXUS_HOME}/bin/nexus_nerve" \
        "${NEXUS_HOME}/target/release/nexus_nerve" \
        "${NEXUS_HOME}/target/debug/nexus_nerve"; do
        if [ -x "${candidate}" ]; then
            nerve_bin="${candidate}"
            break
        fi
    done

    if [ -n "${nerve_bin}" ]; then
        # Verificar si ya está corriendo
        if pgrep -f "nexus_nerve" &>/dev/null; then
            log "✅ NerveSystem ya está en ejecución"
            return 0
        fi
        log "🟡 NerveSystem binario encontrado pero no activo. Iniciar con: ${nerve_bin}" "WARN"
        return 2
    else
        log "⚠️ NerveSystem binario no encontrado. Se usará el módulo de Rust en core." "WARN"
        return 1
    fi
}

start_nerve_system() {
    log "🚀 Iniciando NerveSystem..."
    local nerve_bin="${NEXUS_HOME}/bin/nexus_nerve"

    if [ -x "${nerve_bin}" ]; then
        nohup "${nerve_bin}" \
            > /tmp/nexus-nerve.log 2>&1 &
        local pid=$!
        log "✅ NerveSystem iniciado con PID ${pid}"
        return 0
    fi

    log "⚠️ No se pudo iniciar NerveSystem. Modo core-only." "WARN"
    return 1
}

# =============================================================================
# LANZAMIENTO DEL AGENTE ROO/CLINE EN MODO AUTÓNOMO
# =============================================================================

launch_roo_agent() {
    local ide="${1}"
    log "🚀 Lanzando agente Roo/Cline en modo autónomo..."

    # Método 1: Usar comandos IPC de VSCode/Codium para abrir terminal con comando
    # Esto fuerza al agente a ejecutarse sin intervención
    case "${ide}" in
        codium)
            log "📟 Lanzando Codium con terminal NEXUS..."
            codium --new-window "${NEXUS_HOME}" &
            sleep 2
            # Intentar enviar comando al terminal via IPC (si está disponible)
            if command -v xdotool &>/dev/null; then
                log "Usando xdotool para enfoque de terminal..."
            fi
            ;;

        code)
            log "📟 Lanzando VS Code con terminal NEXUS..."
            code --new-window "${NEXUS_HOME}" &
            sleep 2
            ;;

        headless|vscode-with-extension)
            log "📟 Modo headless: NEXUS autónomo activo sin IDE gráfico."
            log "📟 Servicios: proxy en :4444, Santuario en :1420, orquestador en :43211"
            return 0
            ;;

        *)
            log "📟 Modo desconocido. Lanzando VSCode genérico..."
            if command -v code &>/dev/null; then
                code --new-window "${NEXUS_HOME}" &
            fi
            ;;
    esac
}

# =============================================================================
# GENERACIÓN DE REPORTE DE ESTADO
# =============================================================================

generate_status_report() {
    local report=""
    report+="========================================\n"
    report+="🔱 NEXUS AUTONOMOUS STATUS REPORT\n"
    report+="========================================\n"
    report+="Timestamp: ${TIMESTAMP}\n"
    report+="Hostname: $(hostname 2>/dev/null || echo 'unknown')\n"
    report+="Kernel: $(uname -r 2>/dev/null || echo 'unknown')\n"
    report+="CPU Load: $(uptime 2>/dev/null || echo 'N/A')\n"
    report+="Memory: $(free -h 2>/dev/null | awk '/^Mem:/ {print $3 "/" $2}' || echo 'N/A')\n"
    report+="\n--- Componentes ---\n"
    report+="Santuario Web :1420: $(curl -s -o /dev/null -w '%{http_code}' http://localhost:${SANTUARIO_PORT}/ 2>/dev/null || echo 'DOWN')\n"
    report+="OCEAN DB: $(test -f "${OCEAN_DB}" && echo 'OK' || echo 'NOT FOUND')\n"
    report+="NerveSystem: $(pgrep -f nexus_nerve &>/dev/null && echo 'RUNNING' || echo 'INACTIVE')\n"
    report+="\n--- IDE ---\n"
    report+="Detected: $(detect_ide)\n"
    report+="========================================"

    log "${report}" "REPORT"
    echo -e "${report}" > /tmp/nexus-status-report.txt
}

# =============================================================================
# BUCLE DE AUTONOMÍA (Mantener vivo el sistema)
# =============================================================================

keep_alive_loop() {
    log "🔄 Activando bucle de autonomía OMEGA-6..."

    while true; do
        # 1. Verificar Santuario
        if ! check_santuario "${SANTUARIO_PORT}"; then
            start_santuario "${SANTUARIO_PORT}" || true
        fi

        # 2. Verificar NerveSystem (cada 5 iteraciones)
        if (( RANDOM % 5 == 0 )); then
            if ! pgrep -f "nexus_nerve" &>/dev/null; then
                log "🔄 NerveSystem caído. Reintentando..." "WARN"
                start_nerve_system || true
            fi
        fi

        # 3. Reporte de salud cada 10 minutos
        if (( RANDOM % 60 == 0 )); then
            generate_status_report
        fi

        # 4. Esperar 10 segundos antes del próximo ciclo
        sleep 10
    done
}

# =============================================================================
# MAIN — PUNTO DE ENTRADA
# =============================================================================

main() {
    log "========================================"
    log "🔱 NEXUS AUTONOMOUS START v2.0.0-omega"
    log "========================================"
    log "Iniciando procedimiento de ignición autónoma..."

    # 1. Verificar que estamos en el directorio correcto
    if [ ! -d "${NEXUS_HOME}" ]; then
        log "❌ Directorio NEXUS_HOME no encontrado: ${NEXUS_HOME}" "ERROR"
        exit 1
    fi
    cd "${NEXUS_HOME}"

    # 2. Detectar herramientas base
    detect_node || log "⚠️ Node.js no disponible, funciones limitadas" "WARN"

    # 3. Verificar componentes
    check_santuario "${SANTUARIO_PORT}" || start_santuario "${SANTUARIO_PORT}" || true
    check_ocean || true
    check_nerve_system || true
    local nerve_status=$?
    if [ ${nerve_status} -eq 2 ]; then
        start_nerve_system || true
    fi

    # 4. Detectar IDE y lanzar agente (no crítico si falla)
    local ide
    ide=$(detect_ide)
    launch_roo_agent "${ide}" || true

    # 5. Generar reporte inicial
    generate_status_report

    log "========================================"
    log "🔱 SISTEMA AUTÓNOMO INICIALIZADO"
    log "========================================"
    log "Log: ${NEXUS_LOG}"
    log "Status: /tmp/nexus-status-report.txt"
    log ""
    log "COMANDOS ÚTILES:"
    log "  systemctl --user status nexus-autonomous.service"
    log "  journalctl --user -u nexus-autonomous.service -f"
    log "  cat /tmp/nexus-status-report.txt"

    # 6. Mantener vivo (modo daemon)
    if [ "${1:-}" = "--daemon" ]; then
        keep_alive_loop
    fi
}

# Ejecutar main con todos los argumentos
main "$@"
