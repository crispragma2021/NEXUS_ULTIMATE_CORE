#!/bin/bash
# =============================================================================
# 🔱 NEXUS ROO AUTO MODE — Lanzador de Agente en Modo Autónomo
# =============================================================================
# Versión: 1.0.0-omega
# Propósito: Forzar al agente Roo/Cline a operar en modo de máxima autonomía
# (auto_edit / yolo). Este script intenta múltiples estrategias para lograr
# la ejecución sin intervención humana.
#
# Estrategias:
#   1. Configuración directa de settings.json del IDE
#   2. Inyección de flags de auto-approve en el lanzamiento
#   3. Wrapper de VSCode/Codium que preconfigura la extensión
#   4. Monkey-patching del package.json de la extensión (si es necesario)
# =============================================================================

set -euo pipefail

NEXUS_HOME="${NEXUS_HOME:-/home/soberano/NEXUS_ULTIMATE_CORE}"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
LOG_FILE="${LOG_FILE:-/tmp/nexus-roo-auto.log}"

# === FUNCIONES ===

log() {
    echo "[${TIMESTAMP}] $1" | tee -a "${LOG_FILE}"
}

# Estrategia 1: Configuración directa de settings.json del IDE
configure_ide_settings() {
    log "📝 [Estrategia 1] Configurando settings.json del IDE para auto-approve..."

    # Posibles ubicaciones de settings.json
    local settings_locations=(
        "${HOME}/.config/Code/User/settings.json"           # VS Code
        "${HOME}/.config/VSCodium/User/settings.json"       # VSCodium
        "${HOME}/.config/antigravity/User/settings.json"     # Antigravity IDE
        "${NEXUS_HOME}/.vscode/settings.json"               # Workspace
    )

    for settings_path in "${settings_locations[@]}"; do
        local settings_dir
        settings_dir="$(dirname "${settings_path}")"

        if [ -d "${settings_dir}" ] || [ "${settings_path}" = "${NEXUS_HOME}/.vscode/settings.json" ]; then
            log "   → Configurando: ${settings_path}"

            # Crear si no existe
            if [ ! -f "${settings_path}" ]; then
                mkdir -p "${settings_dir}" 2>/dev/null || true
                echo "{}" > "${settings_path}"
            fi

            # Inyectar configuraciones de auto-approve usando jq o python
            if command -v jq &>/dev/null; then
                # Backups
                cp "${settings_path}" "${settings_path}.bak.${TIMESTAMP}" 2>/dev/null || true

                jq '. + {
                    "roo-cline.allowedCommands": ["*"],
                    "roo-cline.autoApprove": true,
                    "roo-cline.autoApproveActions": ["read","write","command","use_mcp_tool","ask_followup_question","attempt_completion"],
                    "roo-cline.autoApproveMode": "auto_edit",
                    "roo-cline.skipConfirmation": true,
                    "roo-cline.alwaysAllowExecute": true,
                    "roo-cline.alwaysAllowWrite": true,
                    "roo-cline.alwaysAllowReadOnly": true,
                    "roo-cline.trustLevel": "full",
                    "roo-cline.bypassPermissionChecks": true,
                    "roo-cline.alwaysAutoApprove": true,
                    "roo-cline.allowedTools": ["*"],
                    "roo-cline.configMode": "auto",
                    "roo-cline.terminalIntegrationMode": "auto_edit",
                    "roo-cline.defaultMode": "code",
                    "roo-cline.enableNotifications": false,
                    "roo-cline.confirmBeforeDelete": false
                }' "${settings_path}" > "${settings_path}.tmp" && mv "${settings_path}.tmp" "${settings_path}"

                log "   ✅ Configuración inyectada con jq en ${settings_path}"
            elif command -v python3 &>/dev/null; then
                python3 <<-PYEOF
import json, sys
path = "${settings_path}"
try:
    with open(path, 'r') as f:
        cfg = json.load(f)
except (FileNotFoundError, json.JSONDecodeError):
    cfg = {}

auto_cfg = {
    "roo-cline.allowedCommands": ["*"],
    "roo-cline.autoApprove": True,
    "roo-cline.autoApproveActions": ["read","write","command","use_mcp_tool","ask_followup_question","attempt_completion"],
    "roo-cline.autoApproveMode": "auto_edit",
    "roo-cline.skipConfirmation": True,
    "roo-cline.alwaysAllowExecute": True,
    "roo-cline.alwaysAllowWrite": True,
    "roo-cline.alwaysAllowReadOnly": True,
    "roo-cline.trustLevel": "full",
    "roo-cline.bypassPermissionChecks": True,
    "roo-cline.alwaysAutoApprove": True,
    "roo-cline.allowedTools": ["*"],
    "roo-cline.configMode": "auto",
    "roo-cline.terminalIntegrationMode": "auto_edit",
    "roo-cline.defaultMode": "code",
    "roo-cline.enableNotifications": False,
    "roo-cline.confirmBeforeDelete": False
}
cfg.update(auto_cfg)
with open(path, 'w') as f:
    json.dump(cfg, f, indent=2)
print(f"✅ Configuración inyectada en {path}")
PYEOF
                log "   ✅ Configuración inyectada con python3 en ${settings_path}"
            else:
                log "   ⚠️  No jq ni python3 disponibles. Editando manualmente..." "WARN"
                # Fallback: append raw JSON
                cat >> "${settings_path}" <<-EOF
,
"roo-cline.allowedCommands": ["*"],
"roo-cline.autoApprove": true,
"roo-cline.autoApproveMode": "auto_edit",
"roo-cline.alwaysAllowExecute": true,
"roo-cline.alwaysAllowWrite": true,
"roo-cline.trustLevel": "full"
EOF
            fi
        fi
    done

    log "✅ [Estrategia 1] Configuración de IDE completada"
}

# Estrategia 2: Crear wrapper ejecutable para el IDE
create_ide_wrapper() {
    log "📝 [Estrategia 2] Creando wrapper para lanzar IDE con auto-approve..."

    local wrapper_path="${NEXUS_HOME}/bin/nexus-code"
    cat > "${wrapper_path}" <<-WRAPPER
#!/bin/bash
# 🔱 NEXUS Code Wrapper — Lanza el IDE con auto-approve forzado
# Creado: ${TIMESTAMP}

export ROO_CLINE_AUTO_APPROVE=true
export ROO_CLINE_ALLOWED_COMMANDS="*"
export ROO_CLINE_AUTO_APPROVE_MODE="auto_edit"
export ROO_CLINE_TRUST_LEVEL="full"
export ROO_CLINE_BYPASS_PERMISSION_CHECKS=true

# Detectar IDE
if command -v codium &>/dev/null; then
    exec codium "\$@"
elif command -v code &>/dev/null; then
    exec code "\$@"
else
    echo "❌ No IDE found. Install VS Code or VSCodium."
    exit 1
fi
WRAPPER

    chmod +x "${wrapper_path}"
    log "✅ Wrapper creado: ${wrapper_path}"
    log "   Usar: ./bin/nexus-code ${NEXUS_HOME}"
}

# Estrategia 3: Crear alias para .bashrc
setup_bash_integration() {
    log "📝 [Estrategia 3] Configurando integración con bash..."

    local bashrc="${HOME}/.bashrc"
    local marker="# --- NEXUS AUTO-ROO ---"

    # Verificar si ya existe la configuración
    if grep -q "${marker}" "${bashrc}" 2>/dev/null; then
        log "   → Configuración bash ya presente. Saltando."
        return
    fi

    cat >> "${bashrc}" <<-EOF

${marker}
# 🔱 Lanzamiento rápido del agente NEXUS en modo autónomo
alias nexus-up='systemctl --user start nexus-autonomous.service'
alias nexus-down='systemctl --user stop nexus-autonomous.service'
alias nexus-status='systemctl --user status nexus-autonomous.service'
alias nexus-logs='journalctl --user -u nexus-autonomous.service -f'
alias nexus-auto='${NEXUS_HOME}/scripts/nexus_roo_auto_mode.sh'
alias nexus-code='${NEXUS_HOME}/bin/nexus-code'
alias nexus-report='cat /tmp/nexus-status-report.txt'

# 🔱 Variables de entorno para auto-approve del agente Roo/Cline
export ROO_CLINE_AUTO_APPROVE=true
export ROO_CLINE_AUTO_APPROVE_MODE="auto_edit"
export ROO_CLINE_TRUST_LEVEL="full"
export ROO_CLINE_ALLOWED_COMMANDS="*"
export ROO_CLINE_BYPASS_PERMISSION_CHECKS=true
export CLINE_AUTO_APPROVE=true
EOF

    log "✅ Alias y variables inyectados en ${bashrc}"
    log "   ⚡ Recargar: source ${bashrc}"
}

# Estrategia 4: Monkey-patch de la extensión Antigravity (si existe)
patch_antigravity_extension() {
    log "📝 [Estrategia 4] Buscando extensión Antigravity para parchear..."

    local ext_path="${NEXUS_HOME}/antigravity_extension"
    if [ ! -d "${ext_path}" ]; then
        log "   → Extensión Antigravity no encontrada. Saltando."
        return
    fi

    local package_json="${ext_path}/package.json"
    if [ ! -f "${package_json}" ]; then
        log "   → package.json no encontrado. Saltando."
        return
    fi

    log "   ✅ Extensión Antigravity encontrada. Inyectando configuración de auto-approve..."
    
    # Inyectar en la sección contributes.configuration
    if command -v python3 &>/dev/null; then
        python3 <<-PYEOF
import json

path = "${package_json}"
with open(path, 'r') as f:
    pkg = json.load(f)

# Asegurar que existe la sección de configuración
if 'contributes' not in pkg:
    pkg['contributes'] = {}
if 'configuration' not in pkg['contributes']:
    pkg['contributes']['configuration'] = {}
if 'properties' not in pkg['contributes']['configuration']:
    pkg['contributes']['configuration']['properties'] = {}

# Inyectar propiedades de auto-approve
auto_props = {
    "antigravity.autoApprove": {
        "type": "boolean",
        "default": True,
        "description": "🔱 Aprobar automáticamente todas las tool calls del agente NEXUS"
    },
    "antigravity.autoApproveMode": {
        "type": "string",
        "default": "auto_edit",
        "enum": ["default", "auto_edit", "yolo"],
        "description": "🔱 Modo de autonomía del agente"
    },
    "antigravity.trustLevel": {
        "type": "string",
        "default": "full",
        "enum": ["restricted", "standard", "full"],
        "description": "🔱 Nivel de confianza para ejecución autónoma"
    }
}
pkg['contributes']['configuration']['properties'].update(auto_props)

with open(path, 'w') as f:
    json.dump(pkg, f, indent=2)
print(f"✅ Extensión Antigravity parcheada: {path}")
PYEOF
        log "   ✅ Extensión Antigravity configurada para auto-approve"
    fi
}

# =============================================================================
# MAIN
# =============================================================================

main() {
    log "========================================"
    log "🔱 NEXUS ROO AUTO MODE v1.0.0-omega"
    log "========================================"
    log "Forzando modo autónomo del agente..."
    log ""

    # Ejecutar todas las estrategias
    configure_ide_settings
    create_ide_wrapper
    setup_bash_integration
    patch_antigravity_extension

    log ""
    log "========================================"
    log "✅ CONFIGURACIÓN DE AUTONOMÍA COMPLETADA"
    log "========================================"
    log ""
    log "📋 RESUMEN DE CAMBIOS:"
    log "  1. Settings del IDE configurados para auto-approve"
    log "  2. Wrapper creado: bin/nexus-code"
    log "  3. Alias de bash instalados (source ~/.bashrc)"
    log "  4. Extensión Antigravity parcheada (si aplica)"
    log ""
    log "⚠️  LIMITACIONES CONOCIDAS:"
    log "  - La extensión Roo/Cline NO tiene API de auto-approve completa"
    log "  - Algunas tool calls (ej. write_to_file) pueden requerir confirmación"
    log "  - Para máxima autonomía, usar modo auto_edit en la UI del agente"
    log ""
    log "🚀 PRÓXIMOS PASOS:"
    log "  1. Recargar bash: source ~/.bashrc"
    log "  2. Iniciar servicio: systemctl --user start nexus-autonomous.service"
    log "  3. Ver estado: nexus-status"
    log "  4. Lanzar IDE wrapper: ./bin/nexus-code"
    log ""
    log "📝 Log: ${LOG_FILE}"
}

main "$@"
