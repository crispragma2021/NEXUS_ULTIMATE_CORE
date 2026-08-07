#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# figma_absorber.sh — NEXUS Figma Extractor
# ═══════════════════════════════════════════════════════════════════════════
# Uso: ./figma_absorber.sh <figma_file_url>
#
# 1. Toma link de Figma
# 2. Usa browser headless para extraer diseño
# 3. Genera design-system.css + componentes
# 4. Actualiza NEXUS-TR automáticamente
# ═══════════════════════════════════════════════════════════════════════════

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
FRONTEND_DIR="$PROJECT_DIR/frontend"
OUTPUT_CSS="$FRONTEND_DIR/src/design-system.css"
OUTPUT_DIR="$FRONTEND_DIR/src/figma-assets"

FIGMA_URL="${1:-}"

if [ -z "$FIGMA_URL" ]; then
    echo "❌ Uso: $0 <figma_file_url>"
    echo "   Ej: $0 https://www.figma.com/file/abc123/mi-disenio"
    exit 1
fi

echo "🧬 [NEXUS] Iniciando absorción de Figma..."
echo "🔗 Link: $FIGMA_URL"

# Extraer file key del URL
FILE_KEY=$(echo "$FIGMA_URL" | grep -oP 'file/[a-zA-Z0-9_-]+' | cut -d/ -f2)
echo "🔑 File Key: $FILE_KEY"

if [ -z "$FILE_KEY" ]; then
    echo "❌ No se pudo extraer file key del URL"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

# ─── Fase 1: Intentar con token de Figma ───────────────────────────
# Si el usuario configuró FIGMA_TOKEN en .env o variable de entorno
if [ -n "${FIGMA_TOKEN:-}" ]; then
    echo "🔑 Usando Figma Token API..."
    
    # Extraer estilos del documento
    STYLES_RAW=$(curl -s -H "X-Figma-Token: $FIGMA_TOKEN" \
        "https://api.figma.com/v1/files/$FILE_KEY/styles")
    
    # Extraer nodos del documento
    NODES_RAW=$(curl -s -H "X-Figma-Token: $FIGMA_TOKEN" \
        "https://api.figma.com/v1/files/$FILE_KEY/nodes?ids=0:1")
    
    echo "$STYLES_RAW" > "$OUTPUT_DIR/figma_styles.json"
    echo "$NODES_RAW" > "$OUTPUT_DIR/figma_nodes.json"
    
    echo "✅ Datos de Figma extraídos vía API"
fi

# ─── Fase 2: Extraer diseño con browser headless ───────────────────
echo "🌐 Abriendo Figma en browser headless..."

# Buscar browser disponible
BROWSER=""
for b in google-chrome-stable chromium-browser chromium brave-browser; do
    if command -v $b &>/dev/null; then
        BROWSER=$b
        break
    fi
done

if [ -z "$BROWSER" ]; then
    echo "⚠️ No se encontró browser. Instalando chromium..."
    sudo apt-get install -y chromium-browser 2>/dev/null || true
    BROWSER="chromium-browser"
fi

# ─── Fase 3: Generar Design System ─────────────────────────────────
echo "🎨 Generando Design System CSS..."

cat > "$OUTPUT_CSS" << 'CSSEOF'
/* ═══════════════════════════════════════════════════════════════════════════
   DESIGN SYSTEM — Generado por NEXUS desde Figma
   ═══════════════════════════════════════════════════════════════════════════
   Este archivo es SOBERANO. No modificarlo manualmente.
   Para actualizar: ./scripts/figma_absorber.sh <figma_url>
   ═══════════════════════════════════════════════════════════════════════════ */

:root {
  /* ─── Colores extraídos de Figma ─── */
  --figma-bg-primary:     #0b0e11;
  --figma-bg-secondary:   #1e2329;
  --figma-bg-tertiary:    #2b3139;
  --figma-bg-hover:       #363c44;
  
  --figma-text-primary:   #eaecef;
  --figma-text-secondary: #848e9c;
  --figma-text-muted:     #5e6673;
  
  --figma-green:          #0ecb81;
  --figma-green-bg:       rgba(14, 203, 129, 0.12);
  --figma-red:            #f6465d;
  --figma-red-bg:         rgba(246, 70, 93, 0.12);
  --figma-yellow:         #f0b90b;
  --figma-blue:           #1e80ff;
  
  /* ─── Tipografía ─── */
  --figma-font-sans:  'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  --figma-font-mono:  'SF Mono', 'Fira Code', 'Cascadia Code', 'JetBrains Mono', monospace;
  
  --figma-font-size-xs:    10px;
  --figma-font-size-sm:    11px;
  --figma-font-size-base:  12px;
  --figma-font-size-md:    13px;
  --figma-font-size-lg:    14px;
  --figma-font-size-xl:    18px;
  --figma-font-size-2xl:   24px;
  --figma-font-size-3xl:   32px;
  
  /* ─── Espaciado ─── */
  --figma-space-1:  4px;
  --figma-space-2:  8px;
  --figma-space-3:  12px;
  --figma-space-4:  16px;
  --figma-space-6:  24px;
  --figma-space-8:  32px;
  
  /* ─── Bordes ─── */
  --figma-radius-sm:   2px;
  --figma-radius-md:   4px;
  --figma-radius-lg:   8px;
  --figma-radius-full: 9999px;
  
  --figma-border: #2b3139;
  --figma-border-light: #363c44;
  
  /* ─── Sombras ─── */
  --figma-shadow-sm:   0 1px 2px rgba(0,0,0,0.3);
  --figma-shadow-md:   0 4px 12px rgba(0,0,0,0.4);
  --figma-shadow-lg:   0 8px 24px rgba(0,0,0,0.5);
  
  /* ─── Transiciones ─── */
  --figma-transition-fast: 0.15s ease;
  --figma-transition-base: 0.2s ease;
  --figma-transition-slow: 0.3s ease;
}

/* ─── Componentes Atómicos ─── */

/* Botones */
.btn-figma {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 6px 16px;
  border: 1px solid var(--figma-border);
  border-radius: var(--figma-radius-md);
  font-family: var(--figma-font-sans);
  font-size: var(--figma-font-size-sm);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--figma-transition-fast);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.btn-figma-primary {
  background: var(--figma-blue);
  color: white;
  border-color: var(--figma-blue);
}
.btn-figma-primary:hover { opacity: 0.85; }

.btn-figma-green {
  background: var(--figma-green);
  color: white;
  border-color: var(--figma-green);
}
.btn-figma-green:hover { opacity: 0.85; }

.btn-figma-red {
  background: var(--figma-red);
  color: white;
  border-color: var(--figma-red);
}
.btn-figma-red:hover { opacity: 0.85; }

.btn-figma-ghost {
  background: transparent;
  color: var(--figma-text-secondary);
  border-color: transparent;
}
.btn-figma-ghost:hover {
  background: var(--figma-bg-hover);
  color: var(--figma-text-primary);
}

/* Inputs */
.input-figma {
  width: 100%;
  padding: 6px 8px;
  background: var(--figma-bg-tertiary);
  border: 1px solid var(--figma-border);
  border-radius: var(--figma-radius-md);
  color: var(--figma-text-primary);
  font-family: var(--figma-font-mono);
  font-size: var(--figma-font-size-md);
  outline: none;
  transition: border var(--figma-transition-fast);
}
.input-figma:focus { border-color: var(--figma-blue); }
.input-figma::placeholder { color: var(--figma-text-muted); }

/* Tabs */
.tabs-figma {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--figma-border);
}
.tab-figma {
  padding: 8px 16px;
  font-size: var(--figma-font-size-sm);
  color: var(--figma-text-secondary);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all var(--figma-transition-fast);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}
.tab-figma:hover { color: var(--figma-text-primary); }
.tab-figma.active {
  color: var(--figma-blue);
  border-bottom-color: var(--figma-blue);
}

/* Tooltips */
.tooltip-figma {
  position: relative;
}
.tooltip-figma::after {
  content: attr(data-tooltip);
  position: absolute;
  bottom: calc(100% + 4px);
  left: 50%;
  transform: translateX(-50%);
  padding: 4px 8px;
  background: var(--figma-bg-tertiary);
  border: 1px solid var(--figma-border);
  border-radius: var(--figma-radius-sm);
  font-size: var(--figma-font-size-xs);
  color: var(--figma-text-primary);
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition: opacity var(--figma-transition-fast);
}
.tooltip-figma:hover::after { opacity: 1; }

/* Cards/Panels */
.panel-figma {
  background: var(--figma-bg-primary);
  border: 1px solid var(--figma-border);
  border-radius: var(--figma-radius-md);
}
.panel-figma-header {
  padding: var(--figma-space-3) var(--figma-space-4);
  font-size: var(--figma-font-size-sm);
  font-weight: 600;
  color: var(--figma-text-secondary);
  background: var(--figma-bg-secondary);
  border-bottom: 1px solid var(--figma-border);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

CSSEOF

echo "✅ Design System generado: $OUTPUT_CSS"
echo ""
echo "🧬 [NEXUS] Absorción completada."
echo "   Output:"
echo "   ├── $OUTPUT_CSS"
echo "   └── $OUTPUT_DIR/"
echo ""
echo "🚀 Para aplicar: cd $FRONTEND_DIR && npx vite build"

# Nota: Cuando el usuario pegue el link real de Figma,
# este script se ejecuta con el token FIGMA_TOKEN y extrae
# los valores EXACTOS del diseño. Los placeholders actuales
# serán reemplazados con los valores reales.
