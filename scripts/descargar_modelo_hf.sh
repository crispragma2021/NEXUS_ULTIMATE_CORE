#!/usr/bin/env bash
# =============================================================================
# scripts/descargar_modelo_hf.sh — 🧬 Descargador Soberano de modelos HuggingFace
# -----------------------------------------------------------------------------
# Método de autenticación HF para NEXUS (rompe el bloqueo HTTP 401).
#
# El bloqueo diagnóstico mostró que huggingface.co exige autenticación
# (www-authenticate: Bearer) incluso para repos públicos. Para autentificarse,
# NEXUS envía el header `Authorization: Bearer <HF_TOKEN>`.
#
# FUENTE DEL TOKEN (en orden de prioridad):
#   1. Argumento:            --token hf_xxxxxxxx
#   2. Variable de entorno:  HF_TOKEN        (o HUGGING_FACE_HUB_TOKEN)
#   3. Archivo .env del proyecto:  HF_TOKEN=hf_xxxx
#   4. Archivo CLI HF:       ~/.cache/huggingface/token
#
# USO:
#   HF_TOKEN=hf_xxx ./scripts/descargar_modelo_hf.sh \
#       --repo Qwen/Qwen3-4B-Instruct-GGUF \
#       --file qwen3-4b-instruct-q4_k_m.gguf
#   ./scripts/descargar_modelo_hf.sh --token hf_xxx --help-repo
# =============================================================================
set -euo pipefail

MODELOS_DIR="${MODELOS_DIR:-/home/soberano/NEXUS_ULTIMATE_CORE/brain/swarm/models}"
ENV_FILE="${ENV_FILE:-/home/soberano/NEXUS_ULTIMATE_CORE/.env}"
DESTINO=""
REPO=""
FILE=""
TOKEN=""
FORCE=""
NO_VERIFY=""

# ── Parseo de argumentos ─────────────────────────────────────────────────────
usage() {
  cat <<'EOF'
USO: descargar_modelo_hf.sh [opciones]

OPCIONES:
  --repo <org/repo>     Repositorio HF (ej: Qwen/Qwen3-4B-Instruct-GGUF)
  --file <nombre.gguf>  Archivo exacto dentro del repo
  --dest <ruta>         Directorio destino (default: brain/swarm/models)
  --token <hf_xxxx>     Token HF (alternativa a HF_TOKEN/.env)
  --force               Sobrescribir si el archivo existe
  --no-verify           Omitir verificación de tamaño HTTP HEAD
  --help-repo           Listar archivos del repo (requiere token)
  -h, --help            Mostrar esta ayuda
EOF
}

# ── Resolución del token: múltiples fuentes sin barreras ─────────────────────
resolver_token() {
  if [[ -n "$TOKEN" ]]; then echo "$TOKEN"; return; fi
  if [[ -n "${HF_TOKEN:-}" ]]; then echo "$HF_TOKEN"; return; fi
  if [[ -n "${HUGGING_FACE_HUB_TOKEN:-}" ]]; then echo "$HUGGING_FACE_HUB_TOKEN"; return; fi
  if [[ -f "$ENV_FILE" ]]; then
    local t
    t=$(grep -E '^\s*HF_TOKEN=' "$ENV_FILE" | head -1 | sed -E 's/^\s*HF_TOKEN=//; s/^["'"'"']//; s/["'"'"']$//' || true)
    if [[ -n "$t" && "$t" != "hf_xxxx" ]]; then echo "$t"; return; fi
  fi
  if [[ -f "$HOME/.cache/huggingface/token" ]]; then
    cat "$HOME/.cache/huggingface/token" | tr -d '[:space:]'; return
  fi
  return 1
}

# ── URL de descarga (endpoints autorizados por HF) ───────────────────────────
# https://huggingface.co/<repo>/resolve/main/<file>
# con query ?download=true para forzar descarga binaria sin HTML.
url_descarga() {
  echo "https://huggingface.co/${REPO}/resolve/main/${FILE}?download=true"
}

# ── Autorización condicional: solo añade header si hay token ─────────────────
auth_header() {
  local tok="$1"
  echo "Authorization: Bearer ${tok}"
}

# ── Verificar que el archivo existe en el repo (HEAD) ────────────────────────
verificar_remoto() {
  local tok="$1"
  local url; url="https://huggingface.co/${REPO}/resolve/main/${FILE}"
  local status
  local curl_args=(-s -o /dev/null -w '%{http_code}' -I -L)
  if [[ -n "$tok" ]]; then
    curl_args+=(-H "$(auth_header "$tok")")
  fi
  status=$(curl "${curl_args[@]}" --max-time 30 "$url" 2>/dev/null || echo "000")
  case "$status" in
    200|206) return 0 ;;
    401|403)
      echo "✋ HTTP $status: requiere autenticación HF. Proporciona un token válido."
      echo "   Obtén uno gratis en: https://huggingface.co/settings/tokens"
      return 1 ;;
    404) echo "✋ HTTP 404: repo/archivo no encontrado. Verifica --repo y --file."; return 1 ;;
    *) echo "✋ HTTP $status: respuesta inesperada."; return 1 ;;
  esac
}

# ── Descarga principal con reintentos y header Bearer ────────────────────────
descargar() {
  local tok="$1"
  mkdir -p "$DESTINO"
  local out="$DESTINO/$FILE"
  local url; url="$(url_descarga)"

  if [[ -z "$FORCE" && -f "$out" && -s "$out" ]]; then
    echo "✅ Ya existe: $out"
    return 0
  fi

  echo "⬇️  Descargando $FILE"
  echo "   → $REPO"
  echo "   → $out"
  [[ -n "$tok" ]] && echo "   🔑 Autenticado con token HF (Bearer)"

  local curl_args=(-L --fail --progress-bar -o "$out" --retry 3 --retry-delay 2 -C -)
  if [[ -n "$tok" ]]; then
    curl_args+=(-H "$(auth_header "$tok")")
  fi
  if curl "${curl_args[@]}" "$url"; then
    echo "✅ Descarga completada: $out ($(du -h "$out" | cut -f1))"
    return 0
  fi
  rm -f "$out"   # limpiar descarga parcial corrupta
  echo "❌ Falló la descarga. Verifica token y red."
  return 1
}

# ── Listar archivos de un repo (útil para encontrar el .gguf exacto) ─────────
listar_repo() {
  local tok="$1"
  local url="https://huggingface.co/api/models/${REPO}"
  local curl_args=(-s)
  [[ -n "$tok" ]] && curl_args+=(-H "$(auth_header "$tok")")
  curl "${curl_args[@]}" "$url" | python3 -c '
import sys, json
try:
    data = json.load(sys.stdin)
except Exception:
    print("⚠️  No se pudo parsear la respuesta. ¿Token válido?")
    sys.exit(1)
for s in data.get("siblings", []):
    fn = s.get("rfilename", "")
    if fn.endswith(".gguf"):
        print("  📦", fn)
'
}

# ── Flujo principal ──────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo) REPO="$2"; shift 2 ;;
    --file) FILE="$2"; shift 2 ;;
    --dest) DESTINO="$2"; shift 2 ;;
    --token) TOKEN="$2"; shift 2 ;;
    --force) FORCE=1; shift ;;
    --no-verify) NO_VERIFY=1; shift ;;
    --help-repo) listar_repo "$(resolver_token || true)"; exit 0 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "❌ Argumento desconocido: $1"; usage; exit 1 ;;
  esac
done

[[ -n "$REPO" && -n "$FILE" ]] || { usage; exit 1; }
[[ -n "$DESTINO" ]] || DESTINO="$MODELOS_DIR"

TOKEN_EFECTIVO="$(resolver_token || true)"

if [[ -z "$NO_VERIFY" ]]; then
  verificar_remoto "$TOKEN_EFECTIVO" || exit 1
fi

descargar "$TOKEN_EFECTIVO"
