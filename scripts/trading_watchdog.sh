#!/usr/bin/env bash
# ============================================================
# NEXUS TRADING WATCHDOG — Sistema de Dos Capas
# CAPA 1: Qwen2.5:7b local → Vigilancia 24/7 (0 tokens cloud)
# CAPA 2: Gemini 2.5-flash → Decisiones (solo si hay alerta)
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
- ANOMALÍA (usa senales_resumen): desbalance extremo compra/venta (una dirección >80% del total),
  confianza promedio < 0.40, o concentración anómala en pocos símbolos

Snapshot: $snapshot

Responde solo el JSON, sin texto adicional."

    # [FIX CPU]: Usar la API REST de Ollama (no `ollama run` interactivo).
    # `ollama run` deja sesiones colgadas que mantienen llama-server al 100% CPU.
    # La API REST con --max-time garantiza que cada ciclo termina sí o sí.
    # [FIX ARG_MAX]: El snapshot puede ser enorme; el payload se pasa a curl
    # desde archivo (--data-binary @) como en Capa 2, no como argumento.
    local tmp_prompt="/tmp/nexus_watchdog_capa1_prompt.txt"
    local tmp_payload="/tmp/nexus_watchdog_capa1_payload.json"
    echo "$prompt" > "$tmp_prompt"

    python3 -c "import json, sys; print(json.dumps({'model': 'qwen2.5:7b-instruct-q4_K_M', 'prompt': open(sys.argv[1]).read(), 'stream': False, 'keep_alive': '5m', 'options': {'temperature': 0.2, 'num_predict': 256}}))" "$tmp_prompt" > "$tmp_payload"

    local respuesta
    respuesta=$(curl -s --max-time 90 -X POST http://localhost:11434/api/generate \
        -H "Content-Type: application/json" \
        --data-binary @"$tmp_payload" \
        | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('response',''))" 2>/dev/null \
        | tr -d '\n' | grep -o '{.*}' | head -1) || true

    rm -f "$tmp_prompt" "$tmp_payload"
    echo "${respuesta:-{\"nivel\":\"NORMAL\",\"razon\":\"sin datos\",\"metricas_preocupantes\":[]}}"
}

# ── CAPA 2: Motor de Decisión (Gemini 2.5-flash) ────────────
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
    python3 -c "import json, sys; print(json.dumps({'model': 'gemini-2.5-flash', 'messages': [{'role': 'user', 'content': open(sys.argv[1]).read()}]}))" "$tmp_prompt" > "$tmp_payload"
    
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
    local cartera prediccion auto_trading
    cartera=$(curl -s "$API_BASE/api/cartera" 2>/dev/null || echo '{}')
    auto_trading=$(curl -s "$API_BASE/api/auto-trading/estado" 2>/dev/null || echo '{}')
    prediccion=$(curl -s "$API_BASE/api/prediccion/reporte" 2>/dev/null || echo '{}')

    # [ADELGAZAMIENTO]: /api/senales devuelve ~65k filas (~20 MB) pero el modelo
    # solo tiene contexto de 4096 tokens (~20 KB) → hoy ve el 0.1% al azar.
    # Se resume todo en un agregado compacto + las 20 señales más recientes:
    # el modelo pasa de ver 65 filas al azar a ver el 100% de la información útil.
    # (Se usa archivo temporal para evitar el error ARG_MAX con payloads grandes.)
    local tmp_senales="/tmp/nexus_watchdog_senales.json"
    curl -s "$API_BASE/api/senales" 2>/dev/null > "$tmp_senales" || true
    [ -s "$tmp_senales" ] || echo '[]' > "$tmp_senales"

    local senales
    senales=$(python3 -c "
import json, sys

try:
    with open('$tmp_senales') as f:
        datos = json.load(f)
except Exception:
    datos = []

if not isinstance(datos, list) or not datos:
    print(json.dumps({'total': 0, 'compras': 0, 'ventas': 0,
                      'confianza_promedio': 0, 'mejor': None, 'peor': None,
                      'recientes': []}))
    sys.exit(0)

def compactar(s):
    return {'simbolo': s.get('simbolo'), 'accion': s.get('accion'),
            'confianza': round(s.get('confianza', 0), 3),
            'precio_entrada': round(s.get('precio_entrada', 0), 2)}

compras = [s for s in datos if s.get('accion') == 'compra']
ventas  = [s for s in datos if s.get('accion') == 'venta']
confianzas = [s.get('confianza', 0) for s in datos if isinstance(s.get('confianza'), (int, float))]
conf_prom = round(sum(confianzas)/len(confianzas), 4) if confianzas else 0

ordenados = sorted(datos, key=lambda s: s.get('timestamp', 0), reverse=True)
recientes = [compactar(s) for s in ordenados[:20]]

mejor = max(datos, key=lambda s: s.get('confianza', 0)) if datos else None
peor  = min(datos, key=lambda s: s.get('confianza', 0)) if datos else None

print(json.dumps({
    'total': len(datos),
    'compras': len(compras),
    'ventas': len(ventas),
    'confianza_promedio': conf_prom,
    'mejor': compactar(mejor) if mejor else None,
    'peor': compactar(peor) if peor else None,
    'recientes': recientes,
}))
" 2>/dev/null || echo '{"total":0,"compras":0,"ventas":0,"confianza_promedio":0,"mejor":null,"peor":null,"recientes":[]}')

    rm -f "$tmp_senales"

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

    echo "{\"cartera\": $cartera, \"senales_resumen\": $senales, \"auto_trading\": $auto_trading, \"confianza_media\": $conf_media, \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}"
}

# ── Loop principal ────────────────────────────────────────────
log "${CYAN}${BOLD}🛡️  NEXUS TRADING WATCHDOG INICIADO${NC}"
log "   Intervalo: ${INTERVALO}s | DB: $DB_PATH"
log "   Capa 1: Qwen2.5:7b (local) | Capa 2: Gemini 2.5-flash (cloud)"
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
            log "   🌐 Capa 2: Escalando a Gemini 2.5-flash..."
            DECISION=$(capa2_decision "$NIVEL" "$RAZON" "$SNAPSHOT")
            log "   📋 Decisión Gemini: $DECISION"
            guardar_en_db "ALERTA" "$RAZON" "$DECISION" "$SNAPSHOT"
            # Notificación de escritorio
            notify-send "⚠️ NEXUS TRADER — ALERTA" "$RAZON" --urgency=normal 2>/dev/null || true
            ;;

        "CRITICO")
            log "   ${RED}🚨 CRÍTICO${NC} — $RAZON"
            log "   🌐 Capa 2: Escalando a Gemini 2.5-flash (URGENTE)..."
            DECISION=$(capa2_decision "$NIVEL" "$RAZON" "$SNAPSHOT")
            log "   📋 Decisión Gemini: $DECISION"
            guardar_en_db "CRITICO" "$RAZON" "$DECISION" "$SNAPSHOT"
            # Notificación urgente
            notify-send "🚨 NEXUS TRADER — CRÍTICO" "$RAZON\n$DECISION" --urgency=critical 2>/dev/null || true
            ;;
    esac

    sleep "$INTERVALO"
done
