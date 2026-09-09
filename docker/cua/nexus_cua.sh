#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# 🔱 NEXUS CUA — CONTROL UNIFICADO
# Interfaz CLI para el Computer-Using Agent.
#
#   ./nexus_cua.sh status                    # estado de ambos entornos
#   ./nexus_cua.sh evaluar "abrir formulario"  # decide docker vs firecracker
#   ./nexus_cua.sh up [--env docker|firecracker]
#   ./nexus_cua.sh down
#   ./nexus_cua.sh abrir [url]               # lanza Chromium en noVNC
# ═══════════════════════════════════════════════════════════════════════════
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ORQ="$DIR/orquestador_cua.py"
COMPOSE="$DIR/docker-compose.cua.yml"
NOVNC_PORT="${NOVNC_PORT:-6080}"

info()  { echo -e "\033[0;36m🔱\033[0m $*"; }
ok()    { echo -e "\033[0;32m✅\033[0m $*"; }
err()   { echo -e "\033[0;31m❌\033[0m $*" >&2; }

case "${1:-help}" in
  status)
    python3 "$ORQ" status
    ;;

  evaluar)
    shift
    python3 "$ORQ" evaluar "$@"
    ;;

  up)
    shift
    env_sel="${1:-docker}"
    if [ "$env_sel" = "firecracker" ]; then
      info "Arrancando Firecracker MicroVM..."
      python3 "$ORQ" ejecutar --entorno firecracker
    else
      info "Arrancando Docker Headless GUI..."
      docker compose -f "$COMPOSE" up -d --build
      for i in $(seq 1 30); do
        st=$(docker inspect -f '{{.State.Health.Status}}' nexus-cua-gui 2>/dev/null || echo "creando")
        [ "$st" = "healthy" ] && break
        sleep 1
      done
      ok "noVNC: http://localhost:${NOVNC_PORT}/vnc.html  (password: nexus)"
    fi
    ;;

  down)
    docker compose -f "$COMPOSE" down 2>/dev/null || true
    ok "Entorno Docker detenido."
    ;;

  abrir)
    shift
    url="${1:-about:blank}"
    info "Lanzando Chromium en noVNC → $url"
    docker exec nexus-cua-gui bash -c \
      "DISPLAY=:99 chromium-browser --no-sandbox --disable-dev-shm-usage '$url' >/dev/null 2>&1 &"
    ok "Abierto en http://localhost:${NOVNC_PORT}/vnc.html"
    ;;

  *)
    echo "Uso: $0 {status|evaluar|up [--env docker|firecracker]|down|abrir [url]}"
    ;;
esac
