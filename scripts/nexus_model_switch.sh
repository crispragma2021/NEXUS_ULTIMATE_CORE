#!/bin/bash
# ==============================================================================
# 🧬 NEXUS MODEL SWITCH — Conmutación Inteligente de Modelos (Bajo Demanda)
# ==============================================================================
# Uso: ./scripts/nexus_model_switch.sh <local|cloud|auto|status>
#
#   local   → Cambia a Ollama (nexuslocal:latest) — 0 VRAM en idle
#   cloud   → Cambia a OpenRouter (claude-sonnet-4.5)
#   auto    → Elige automáticamente según tarea y recursos
#   status  → Muestra estado actual de ambos providers
#   list    → Lista modelos locales disponibles en Ollama
# ==============================================================================

set -e

# ─── Configuración ──────────────────────────────────────────────────────────
NEXUS_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
STATE_DB="$HOME/.config/VSCodium/User/globalStorage/state.vscdb"
OLLAMA_API="http://localhost:11434"
PROVIDER_CLOUD='{"name":"default","id":"kw6h8d4557k","apiProvider":"openrouter","modelId":"claude-sonnet-4.5"}'
PROVIDER_LOCAL='{"name":"NEXUS Local (Ollama)","id":"ollama-nexus-local","apiProvider":"ollama","modelId":"nexuslocal:latest"}'
LOG_FILE="${NEXUS_ROOT}/data/logs/model_switch.log"

mkdir -p "$(dirname "$LOG_FILE")"
touch "$LOG_FILE"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

# ─── Verificar prerequisitos ───────────────────────────────────────────────
check_ollama() {
    if ! curl -sf "$OLLAMA_API/api/tags" > /dev/null 2>&1; then
        log "❌ Ollama no responde en $OLLAMA_API"
        return 1
    fi
    return 0
}

check_ollama_model() {
    local model="$1"
    if curl -sf "$OLLAMA_API/api/tags" 2>/dev/null | python3 -c "
import sys, json
data = json.load(sys.stdin)
models = [m['name'] for m in data.get('models', [])]
sys.exit(0 if '$model' in models or any(m.startswith('$model') for m in models) else 1)
" 2>/dev/null; then
        return 0
    fi
    return 1
}

get_vram_status() {
    if command -v nvidia-smi &> /dev/null; then
        nvidia-smi --query-gpu=memory.free,memory.used,memory.total --format=csv,noheader,nounits 2>/dev/null | head -1
    else
        echo "0,0,0"
    fi
}

# ─── Acciones ───────────────────────────────────────────────────────────────

cmd_status() {
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║       🧬 NEXUS — Estado de Providers de Modelos            ║"
    echo "╚══════════════════════════════════════════════════════════════╝"

    # Estado de Ollama
    if check_ollama; then
        echo ""
        echo "  ┌─ 🔥 Ollama (Local) ──────────────────────────────┐"
        local model_count=$(curl -sf "$OLLAMA_API/api/tags" | python3 -c "import sys,json; print(len(json.load(sys.stdin).get('models',[])))" 2>/dev/null)
        echo "  │  Estado:    ✅ Activo (PID: $(pgrep -f 'ollama serve' | head -1))"
        echo "  │  Modelos:   $model_count disponibles"
        echo "  │  Endpoint:  $OLLAMA_API"
        if check_ollama_model "nexuslocal"; then
            echo "  │  🎯 nexuslocal: LISTO (Qwen2 7.6B Q4_K_M)"
        fi
        local vram=$(get_vram_status)
        local vram_free=$(echo "$vram" | cut -d, -f1)
        local vram_total=$(echo "$vram" | cut -d, -f3)
        echo "  │  VRAM:      ${vram_free}MB libres / ${vram_total}MB total"
        echo "  └──────────────────────────────────────────────────┘"
    else
        echo ""
        echo "  ┌─ ❄️  Ollama (Local) ─────────────────────────────┐"
        echo "  │  Estado:    ❌ INACTIVO"
        echo "  │  Acción:    Actívalo con 'ollama serve'"
        echo "  └──────────────────────────────────────────────────┘"
    fi

    # Estado de OpenRouter
    echo ""
    echo "  ┌─ ☁️  OpenRouter (Cloud) ──────────────────────────┐"
    echo "  │  Estado:    🌐 Disponible"
    echo "  │  Modelo:    claude-sonnet-4.5"
    echo "  │  Provider:  OpenRouter API"
    echo "  └──────────────────────────────────────────────────┘"

    # Provider activo en Roo Code
    if [ -f "$STATE_DB" ]; then
        local active=$(sqlite3 "$STATE_DB" "SELECT value FROM ItemTable WHERE key='RooVeterinaryInc.roo-cline';" 2>/dev/null | \
            python3 -c "
import sys, json
data = json.load(sys.stdin)
api = data.get('listApiConfigMeta', [])
if api:
    p = api[0]
    print(f'{p[\"name\"]} ({p[\"apiProvider\"]}) → {p[\"modelId\"]}')
else:
    print('No configurado')
" 2>/dev/null)
        echo ""
        echo "  ┌─ ⚡ Provider Activo en Roo Code ──────────────┐"
        echo "  │  $active"
        echo "  └──────────────────────────────────────────────────┘"
    fi
}

cmd_local() {
    log "🔄 Cambiando a provider LOCAL (Ollama/nexuslocal)..."

    if ! check_ollama; then
        log "❌ Ollama no está corriendo. Iniciándolo..."
        ollama serve &
        sleep 3
        if ! check_ollama; then
            log "❌ No se pudo iniciar Ollama"
            return 1
        fi
    fi

    # Verificar que el modelo existe
    if ! check_ollama_model "nexuslocal"; then
        log "❌ Modelo nexuslocal no encontrado. Modelos disponibles:"
        curl -sf "$OLLAMA_API/api/tags" | python3 -c "import sys,json;[print(f'  - {m[\"name\"]}') for m in json.load(sys.stdin).get('models',[])]"
        log "ℹ️  Puedes crear el modelo con: ollama create nexuslocal -f Modelfile"
        return 1
    fi

    # Modificar Roo Code config para poner Ollama primero
    python3 << 'PYEOF'
import json, sqlite3, uuid

DB = "/home/soberano/.config/VSCodium/User/globalStorage/state.vscdb"
conn = sqlite3.connect(DB)
row = conn.execute("SELECT value FROM ItemTable WHERE key='RooVeterinaryInc.roo-cline'").fetchone()
data = json.loads(row[0])

api_configs = data.get('listApiConfigMeta', [])
# Check if Ollama is already there
ollama_idx = next((i for i, c in enumerate(api_configs) if c.get('apiProvider') == 'ollama'), None)

if ollama_idx is not None:
    # Move Ollama to first position
    ollama_config = api_configs.pop(ollama_idx)
    api_configs.insert(0, ollama_config)
    print("✅ Ollama movido a posición activa (Provider #1)")
else:
    # Add Ollama as first provider
    api_configs.insert(0, {
        'name': 'NEXUS Local (Ollama)',
        'id': 'ollama-nexus-' + uuid.uuid4().hex[:8],
        'apiProvider': 'ollama',
        'modelId': 'nexuslocal:latest'
    })
    print("✅ Ollama agregado como provider activo")

data['listApiConfigMeta'] = api_configs
json_str = json.dumps(data, ensure_ascii=False, separators=(',', ':'))
conn.execute("UPDATE ItemTable SET value=? WHERE key='RooVeterinaryInc.roo-cline'", (json_str,))
conn.commit()
conn.close()
PYEOF

    log "✅ Provider LOCAL activado. VRAM usado: ~0 MB (cargará bajo demanda)"
    log "ℹ️  Reinicia Roo Code o cambia en la UI para aplicar."
}

cmd_cloud() {
    log "🔄 Cambiando a provider CLOUD (OpenRouter/claude-sonnet-4.5)..."

    python3 << 'PYEOF'
import json, sqlite3

DB = "/home/soberano/.config/VSCodium/User/globalStorage/state.vscdb"
conn = sqlite3.connect(DB)
row = conn.execute("SELECT value FROM ItemTable WHERE key='RooVeterinaryInc.roo-cline'").fetchone()
data = json.loads(row[0])

api_configs = data.get('listApiConfigMeta', [])
openrouter_idx = next((i for i, c in enumerate(api_configs) if c.get('apiProvider') == 'openrouter'), None)

if openrouter_idx is not None:
    or_config = api_configs.pop(openrouter_idx)
    api_configs.insert(0, or_config)
    print("✅ OpenRouter movido a posición activa (Provider #1)")
else:
    api_configs.insert(0, {
        'name': 'default',
        'id': 'kw6h8d4557k',
        'apiProvider': 'openrouter',
        'modelId': 'claude-sonnet-4.5'
    })
    print("✅ OpenRouter agregado como provider activo")

data['listApiConfigMeta'] = api_configs
json_str = json.dumps(data, ensure_ascii=False, separators=(',', ':'))
conn.execute("UPDATE ItemTable SET value=? WHERE key='RooVeterinaryInc.roo-cline'", (json_str,))
conn.commit()
conn.close()
PYEOF

    log "✅ Provider CLOUD activado."
}

cmd_auto() {
    log "🤖 Modo AUTO: Evaluando tarea y recursos..."

    local vram=$(get_vram_status)
    local vram_free=$(echo "$vram" | cut -d, -f1)

    log "  VRAM libre: ${vram_free}MB"

    # Decisión basada en VRAM disponible
    if [ "$vram_free" -ge 4096 ] 2>/dev/null && check_ollama; then
        log "  ✅ Suficiente VRAM + Ollama disponible → USANDO LOCAL"
        cmd_local
    elif [ "$vram_free" -lt 4096 ] 2>/dev/null; then
        log "  ⚠️  VRAM baja (<4GB) → USANDO CLOUD"
        cmd_cloud
    elif ! check_ollama; then
        log "  ⚠️  Ollama no disponible → USANDO CLOUD"
        cmd_cloud
    else
        log "  ℹ️  Por defecto → USANDO CLOUD"
        cmd_cloud
    fi
}

cmd_list_models() {
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║     🧬 NEXUS — Modelos Locales Disponibles (Ollama)        ║"
    echo "╚══════════════════════════════════════════════════════════════╝"

    if ! check_ollama; then
        echo ""
        echo "  ❌ Ollama no está corriendo."
        echo "  Ejecuta: ollama serve"
        exit 1
    fi

    curl -sf "$OLLAMA_API/api/tags" | python3 -c "
import sys, json
data = json.load(sys.stdin)
models = data.get('models', [])
print()
if not models:
    print('  No hay modelos descargados.')
    print('  Para descargar: ollama pull qwen2.5:7b')
else:
    for m in sorted(models, key=lambda x: x['name']):
        det = m.get('details', {})
        name = m['name']
        size_gb = m['size'] / (1024**3)
        fam = det.get('family', '?')
        params = det.get('parameter_size', '?')
        quant = det.get('quantization_level', '?')
        caps = ', '.join(m.get('capabilities', []))
        print(f'  📦 {name}')
        print(f'     Tamaño: {size_gb:.1f} GB | Familia: {fam} | Parámetros: {params}')
        print(f'     Cuantización: {quant} | Capacidades: {caps}')
        print()
" 2>/dev/null
}

# ─── Main ──────────────────────────────────────────────────────────────────
case "${1:-status}" in
    local)
        cmd_local ;;
    cloud)
        cmd_cloud ;;
    auto)
        cmd_auto ;;
    status)
        cmd_status ;;
    list)
        cmd_list_models ;;
    *)
        echo "Uso: $0 <local|cloud|auto|status|list>"
        echo ""
        echo "  local   → Cambia a Ollama (nexuslocal:latest)"
        echo "  cloud   → Cambia a OpenRouter (claude-sonnet-4.5)"
        echo "  auto    → Elige automáticamente según recursos del sistema"
        echo "  status  → Muestra estado de todos los providers"
        echo "  list    → Lista modelos locales en Ollama"
        exit 1 ;;
esac
