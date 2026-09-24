#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# NEXUS-Agent — Lanzador Directo Unificado
# ═══════════════════════════════════════════════════════════════════════════

export NODE_PATH="/home/nexus/.hermes/hermes-agent/node_modules"
export NEXUS_AGENT_RAIZ="/home/nexus/NEXUS_ULTIMATE_CORE"

BIN_AGENT="$HOME/.local/bin/nexus-agent"

if [ ! -f "$BIN_AGENT" ]; then
    BIN_AGENT="/home/nexus/NEXUS_ULTIMATE_CORE/target/release/nexus-agent"
fi

if [ -z "$1" ]; then
    exec "$BIN_AGENT"
else
    exec "$BIN_AGENT" --comando "$*"
fi
