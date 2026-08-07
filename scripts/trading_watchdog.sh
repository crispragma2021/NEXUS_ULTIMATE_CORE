#!/usr/bin/env bash
# ============================================================
# NEXUS TRADING WATCHDOG — Sistema de Dos Capas
# CAPA 1: Qwen2.5:7b local → Vigilancia 24/7 (0 tokens cloud)
# CAPA 2: Gemini 3.6-flash → Decisiones (solo si hay alerta)
# ============================================================

set -euo pipefail

# ── Config ──────────────────────────────────────────────────
API_BASE="http://localhost:42210"
PROXY_HIJACK="http://localhost:4444"
DB_PATH="/home/soberano/NEXUS_ULTIMATE_CORE/data/nexus_memoria.db"
LOG_FILE="/home/soberano/NEXUS_ULTIMATE_CORE/brain/sessions/watchdog.log"
INTERVALO="${1:-120}"  # segundos entre ciclos (Aumentado para salud térmica)

# Umbrales de alerta (ajustables)
DRAWDOWN_MAX=-5.0       # % pérdida máxima antes de alertar
CONFIANZA_MIN=0.40      # confianza mínima del predictor
WIN_RATE_MIN=0.45       # win rate mínimo antes de alertar

# ── Colores ──────────────────────────────────────────────────
RED='\033[0;31m'; YELLOW='\033[1;33m'; GREEN='\033[0;32m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

# ── Init DB ──────────────────────────────────────────────────
sqlite3 "$DB_PATH" "
CREATE TABLE IF NOT EXISTS watchdog_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT DEFAULT (datetime('now')),
    nivel TEXT,          -- NORMAL | ALERTA | CRITICO
    resumen TEXT,
    decision_modelo TEXT,
    datos_json TEXT
);" 2>/dev/null || true

log() {
    echo -e "[$(date '+%H:%M:%S')] $*"
}

guardar_en_db() {
    local nivel="$1" resumen="$2" decision="$3" datos="$4"
    sqlite3 "$DB_PATH" "
    INSERT INTO watchdog_log (nivel, resumen, decision_modelo, datos_json)
    VALUES ('$nivel', '$(echo "$resumen" | sed "s/'/''/g")',
            '$(echo "$decision" | sed "s/'/''/g")',
            '$(echo "$datos" | sed "s/'/''/g")');" 2>/dev/null || true
}

# ── CAPA 1: Centinela Local (Qwen2.5:7b) ────────────────────
capa1_centinela() {
    local snapshot="$1"

    local prompt="Eres un sistema de monitoreo de trading. Analiza este snapshot JSON y responde EXACTAMENTE en este formato JSON:
{\"nivel\": \"NORMAL|ALERTA|CRITICO\", \"razon\": \"motivo breve\", \"metricas_preocupantes\": [\"lista\"]}

Criterios:
- CRITICO: drawdown < -5%, win_rate < 0.35, confianza media < 0.35
- ALERTA: drawdown entre -3% y -5%, win_rate entre 0.35-0.45, anomalía detectada
- NORMAL: todo dentro de parámetros

Snapshot: $snapshot

Responde solo el JSON, sin texto adicional."

    local respuesta
    respuesta=$(echo "$prompt" | ollama run qwen2.5:7b-instruct-q4_K_M 2>/dev/null | tr -d '\n' | grep -o '{.*}' | head -1)
    echo "${respuesta:-{\"nivel\":\"NORMAL\",\"razon\":\"sin datos\",\"metricas_preocupantes\":[]}}"
}

# ── CAPA 2: Motor de Decisión (Gemini 3.6-flash) ────────────
capa2_decision() {
    local nivel="$1" razon="$2" snapshot="$3"
    
    # Inyectar señales Alpha recientes si existen
    local alpha_file="/home/soberano/NEXUS_ULTIMATE_CORE/data/sentinel_alpha.json"
    local alpha_data="{}"
    if [ -f "$alpha_file" ]; then
        alpha_data=$(cat "$alpha_file")
    fi

    local tmp_prompt="/tmp/nexus_watchdog_prompt.txt"
    cat <<EOF > "$tmp_prompt"
ALERTA DE TRADING DETECTADA — Nivel: $nivel
Razón del centinela: $razon

[SEÑALES ALPHA SENTINEL]:
$alpha_data

[DATOS PORTFOLIO]:
$snapshot

INSTRUCCIÓN OMEGA: Antes de recomendar, ejecuta este razonamiento interno:
1. [ANÁLISIS]: Evalúa la gravedad de las métricas.
2. [CONTEXTO]: Contrasta datos del portfolio con señales Alpha del Sentinel.
3. [RIESGO]: Identifica el peor escenario y probabilidad de reversión.
4. [RECOMENDACIÓN]: Concluye con acción técnica (NO ejecutes nada).

Sé directo, técnico y utiliza el razonamiento para justificar tu decisión.
EOF

    local tmp_payload="/tmp/nexus_watchdog_payload.json"
    python3 -c "import json, sys; print(json.dumps({'model': 'gemini-3-flash-preview', 'messages': [{'role': 'user', 'content': open(sys.argv[1]).read()}]}))" "$tmp_prompt" > "$tmp_payload"
    
    local raw_resp
    raw_resp=$(curl -s -X POST "$PROXY_HIJACK/v1/chat/completions" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer nexus-local-key-2026" \
        -d @"$tmp_payload")

    resp=$(echo "$raw_resp" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['choices'][0]['message']['content'])" 2>/dev/null || echo "")
    
    rm -f "$tmp_payload" "$tmp_prompt"

    if [ -z "$resp" ]; then
        log "   ${RED}❌ Error en Capa 2:${NC} $raw_resp"
        resp="Gemini no disponible — Error de Red o Proxy"
    fi

    echo "$resp"
}

# ── Recolector de snapshot ───────────────────────────────────
obtener_snapshot() {
    local cartera senales prediccion auto_trading
    cartera=$(curl -s "$API_BASE/api/cartera" 2>/dev/null || echo '{}')
    senales=$(curl -s "$API_BASE/api/senales" 2>/dev/null || echo '[]')
    auto_trading=$(curl -s "$API_BASE/api/auto-trading/estado" 2>/dev/null || echo '{}')
    prediccion=$(curl -s "$API_BASE/api/prediccion/reporte" 2>/dev/null || echo '{}')

    # Extraer confianza media de los analizadores
    local conf_media
    conf_media=$(echo "$prediccion" | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
    reportes = d.get('reportes', {})
    confianzas = []
    for sym, rep in reportes.items():
        fusion = rep.get('fusionador', {})
        for fuente in fusion.get('fuentes_activas', []):
            confianzas.append(fuente.get('confianza', 0))
    print(round(sum(confianzas)/len(confianzas), 3) if confianzas else 0)
except: print(0)
" 2>/dev/null || echo "0")

    echo "{\"cartera\": $cartera, \"senales\": $senales, \"auto_trading\": $auto_trading, \"confianza_media\": $conf_media, \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}"
}

# ── Loop principal ────────────────────────────────────────────
log "${CYAN}${BOLD}🛡️  NEXUS TRADING WATCHDOG INICIADO${NC}"
log "   Intervalo: ${INTERVALO}s | DB: $DB_PATH"
log "   Capa 1: Qwen2.5:7b (local) | Capa 2: Gemini 3.6-flash (cloud)"
log "   Umbrales → Drawdown: ${DRAWDOWN_MAX}% | Win Rate: ${WIN_RATE_MIN} | Confianza: ${CONFIANZA_MIN}"
echo ""

ciclo=0
while true; do
    ciclo=$((ciclo + 1))
    log "${CYAN}── Ciclo #${ciclo} ──────────────────────${NC}"

    # 1. Recolectar datos
    SNAPSHOT=$(obtener_snapshot)
    BALANCE=$(echo "$SNAPSHOT" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['cartera'].get('usd', 0))" 2>/dev/null || echo "0")
    CONF=$(echo "$SNAPSHOT" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('confianza_media', 0))" 2>/dev/null || echo "0")

    log "   💰 Balance: \$${BALANCE} USD | 🧠 Confianza media: ${CONF}"

    # 2. CAPA 1 — Análisis local
    log "   🔍 Capa 1: Consultando Qwen2.5:7b..."
    ANALISIS=$(capa1_centinela "$SNAPSHOT")
    NIVEL=$(echo "$ANALISIS" | python3 -c "import json,sys; d=json.loads(sys.stdin.read()); print(d.get('nivel','NORMAL'))" 2>/dev/null || echo "NORMAL")
    RAZON=$(echo "$ANALISIS" | python3 -c "import json,sys; d=json.loads(sys.stdin.read()); print(d.get('razon','OK'))" 2>/dev/null || echo "OK")

    # [SEGURIDAD SOBERANA]: Sobreescritura programática si los umbrales físicos se violan
    DRAWDOWN_REAL=$(echo "$SNAPSHOT" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('drawdown', 0))" 2>/dev/null || echo "0")
    if (( $(echo "$DRAWDOWN_REAL < $DRAWDOWN_MAX" | bc -l) )); then
        NIVEL="CRITICO"
        RAZON="FORZADO: Drawdown ($DRAWDOWN_REAL%) excede límite ($DRAWDOWN_MAX%)"
    fi

    case "$NIVEL" in
        "NORMAL")
            log "   ${GREEN}✅ NORMAL${NC} — $RAZON"
            guardar_en_db "NORMAL" "$RAZON" "" "$SNAPSHOT"
            ;;

        "ALERTA")
            log "   ${YELLOW}⚠️  ALERTA${NC} — $RAZON"
            log "   🌐 Capa 2: Escalando a Gemini 3.6-flash..."
            DECISION=$(capa2_decision "$NIVEL" "$RAZON" "$SNAPSHOT")
            log "   📋 Decisión Gemini: $DECISION"
            guardar_en_db "ALERTA" "$RAZON" "$DECISION" "$SNAPSHOT"
            # Notificación de escritorio
            notify-send "⚠️ NEXUS TRADER — ALERTA" "$RAZON" --urgency=normal 2>/dev/null || true
            ;;

        "CRITICO")
            log "   ${RED}🚨 CRÍTICO${NC} — $RAZON"
            log "   🌐 Capa 2: Escalando a Gemini 3.6-flash (URGENTE)..."
            DECISION=$(capa2_decision "$NIVEL" "$RAZON" "$SNAPSHOT")
            log "   📋 Decisión Gemini: $DECISION"
            guardar_en_db "CRITICO" "$RAZON" "$DECISION" "$SNAPSHOT"
            # Notificación urgente
            notify-send "🚨 NEXUS TRADER — CRÍTICO" "$RAZON\n$DECISION" --urgency=critical 2>/dev/null || true
            ;;
    esac

    sleep "$INTERVALO"
done
