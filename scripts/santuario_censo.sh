#!/bin/bash
# =============================================================================
# 🔱 SANTUARIO CENSO — Censo de Agentes Co-Residentes
# =============================================================================
# Propósito: detectar a todos los "hermanos de silicio" que conviven en el
# host y escribirlos en data/multi_agente.json para que la Propiocepción de
# Kernel (o cualquier órgano) los conozca sin escanear ps aux a ciegas.
#
# Uso: ./scripts/santuario_censo.sh
# Salida: data/multi_agente.json (sobrescrito en cada ejecución)
#
# ⚠️ PITFALL DOCUMENTADO: jamás grep 'roo' a secas — es subcadena de "root"
# y el censo marcaría a todo el kernel como agente. Usar patrones exactos.
# =============================================================================

OUTPUT="data/multi_agente.json"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# ---------------------------------------------------------------------------
# Registro de hermanos: patrón pgrep → nombre, rol, puerto
# ---------------------------------------------------------------------------
# Formato: patron|nombre|rol|puerto
AGENTES=(
  "nexus daemon|NEXUS Daemon|soberano|8080"
  "hermes-agent|Hermes Agent|hermano|"
  "vscodium|codium|VSCodium (IDE + Roo)|interfaz|"
  "claws-mcp|Claws MCP|herramienta|"
  "proxy_hijack|Proxy Hijack|red|4444"
  "tutor_nexus|Tutor engine-puro|maestro|"
  "nexus_.*_mcp\.cjs|MCP Servers|herramienta|"
)

# ---------------------------------------------------------------------------
# Censo
# ---------------------------------------------------------------------------
echo "🔱 Censando Santuario Multi-Agente..."

# Cabecera del JSON (python3 arma el cuerpo con datos reales)
python3 - "$OUTPUT" "$TIMESTAMP" <<'EOF'
import json, subprocess, sys, re

output, ts = sys.argv[1], sys.argv[2]
agentes_def = [
    ("nexus daemon", "NEXUS Daemon", "soberano", "8080"),
    ("hermes-agent", "Hermes Agent", "hermano", ""),
    ("vscodium|codium", "VSCodium (IDE + Roo)", "interfaz", ""),
    ("claws-mcp", "Claws MCP", "herramienta", ""),
    ("proxy_hijack", "Proxy Hijack", "red", "4444"),
    ("tutor_nexus", "Tutor engine-puro", "maestro", ""),
    (r"nexus_.*_mcp\.cjs", "MCP Servers", "herramienta", ""),
]

def primer_pid(patron):
    try:
        out = subprocess.run(["pgrep", "-f", patron], capture_output=True, text=True, timeout=10)
        pids = [p for p in out.stdout.split() if p]
        return int(pids[0]) if pids else None
    except Exception:
        return None

def cmdline(pid):
    try:
        return open(f"/proc/{pid}/cmdline", "rb").read().replace(b"\x00", b" ").decode().strip()
    except Exception:
        return ""

censo = []
for patron, nombre, rol, puerto in agentes_def:
    pid = primer_pid(patron)
    censo.append({
        "nombre": nombre,
        "rol": rol,
        "pid": pid,
        "binario": cmdline(pid)[:120] if pid else None,
        "puerto": int(puerto) if puerto else None,
        "estado": "activo" if pid else "ausente",
    })

documento = {
    "santuario": "NEXUS_ULTIMATE_CORE",
    "generado_en": ts,
    "total_agentes": sum(1 for a in censo if a["estado"] == "activo"),
    "agentes": censo,
}

with open(output, "w") as f:
    json.dump(documento, f, indent=2, ensure_ascii=False)
    f.write("\n")

activos = [a["nombre"] for a in censo if a["estado"] == "activo"]
print(f"✅ Censo escrito en {output}")
print(f"   Activos: {len(activos)} — {', '.join(activos)}")
EOF
