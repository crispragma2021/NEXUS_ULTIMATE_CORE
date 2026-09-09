#!/bin/bash
# ============================================================================
# 🧠 LANZADOR DEL APRENDIZ AUTÓNOMO — Puente de Aprendizaje Continuo
# ============================================================================
# Wrapper para service_manager.sh: localiza el binario compilado, garantiza
# la API key de OpenRouter y ejecuta el daemon de aprendizaje dirigido que
# corre SIN PARAR en segundo plano.
#
#   rumiar → LLM guía → explorar web → LLM destila → paso_tutor → guardar
#
# Uso:
#   ./scripts/lanzar_aprendiz.sh            # lanza el daemon (bloqueante)
#   ./scripts/service_manager.sh start aprendiz-autonomo "./scripts/lanzar_aprendiz.sh"
# ============================================================================

set -e

# ── Localizar el binario compilado ─────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_DIR="$ROOT_DIR/engine-puro"

# Cargo puede usar CARGO_TARGET_DIR global (p.ej. ~/.cargo-target) en vez de
# target/ dentro del proyecto. Resolvemos el directorio de destino real.
TARGET_DIR="${CARGO_TARGET_DIR:-$PROJECT_DIR/target}"

# Preferir release (optimizado); fallback a debug
BIN=""
for cand in "$TARGET_DIR/release/aprendiz-autonomo" "$TARGET_DIR/debug/aprendiz-autonomo"; do
    if [ -f "$cand" ]; then
        BIN="$cand"
        break
    fi
done

# Si no existe, compilar en debug (rápido) apuntando al mismo TARGET_DIR
if [ -z "$BIN" ]; then
    echo "🔨 Compilando aprendiz-autonomo (primera ejecución)..."
    CARGO_TARGET_DIR="$TARGET_DIR" cargo build --manifest-path "$PROJECT_DIR/Cargo.toml" --bin aprendiz-autonomo
    BIN="$TARGET_DIR/debug/aprendiz-autonomo"
fi

# ── Cargar API key de Groq desde .env ──────────────────────────────────────
# La key se lee del archivo .env del proyecto (GROQ_API_KEY). El daemon la
# usará para guiar y destilar el aprendizaje con un modelo grande e inteligente.
ENV_FILE="$ROOT_DIR/.env"
if [ -z "${GROQ_API_KEY:-}" ] && [ -f "$ENV_FILE" ]; then
    GROQ_VALUE="$(grep '^GROQ_API_KEY=' "$ENV_FILE" | head -n1 | cut -d= -f2- | tr -d '"' | tr -d ' ' | tr -d '\r')"
    if [ -n "$GROQ_VALUE" ]; then
        export GROQ_API_KEY="$GROQ_VALUE"
        echo "✅ GROQ_API_KEY cargada desde .env"
    fi
fi
if [ -z "${GROQ_API_KEY:-}" ]; then
    echo "⚠️  GROQ_API_KEY no encontrada en .env — el daemon correrá en modo degradado"
fi

# ── Trabajar desde el directorio del proyecto (rutas relativas de persistencia)
cd "$PROJECT_DIR"

echo "🚀 Lanzando APRENDIZ AUTÓNOMO desde: $BIN"
exec "$BIN"
