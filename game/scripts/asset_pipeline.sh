#!/usr/bin/env bash
# ============================================================
# NEXUS PROTOCOL — PIPELINE DE ASSETS 2D (ImageMagick)
# Post-procesa los assets generados por IA (Scenario/Pixellab)
# para que entren limpios a Godot: PNG transparente, grid de
# sprites, sprite sheets y tilesets verificados.
#
# Uso:
#   asset_pipeline.sh spritesheet <carpeta> [cols]   → monta PNGs en hoja (4x1 default)
#   asset_pipeline.sh limpiar <archivo|carpeta>      → fondo blanco→transparente + recorte
#   asset_pipeline.sh tileset <carpeta> <tam>        → verifica que todos los tiles tengan el grid
# ============================================================
set -euo pipefail

MAGICK="magick"
LOG="$(pwd)/asset_pipeline.log"

log() { echo "[$(date '+%H:%M:%S')] $*" | tee -a "$LOG"; }

limpiar() {
    local target="$1"
    if [ -d "$target" ]; then
        for f in "$target"/*.png "$target"/*.jpg "$target"/*.jpeg "$target"/*.webp; do
            [ -f "$f" ] || continue
            limpiar_uno "$f"
        done
    else
        limpiar_uno "$target"
    fi
}

limpiar_uno() {
    local f="$1"
    local out="${f%.*}.png"
    log "🧹 limpiando: $(basename "$f")"
    # Fondo blanco → transparente (fuzz para bordes suaves) + recorte al contenido
    "$MAGICK" "$f" -fuzz 12% -transparent white -trim +repage -resize 512x512\> "$out"
}

spritesheet() {
    local dir="$1"
    local cols="${2:-4}"
    local out="${dir%/}_spritesheet.png"
    # Orden estable y solo PNGs ya limpios
    local imgs
    imgs=$(find "$dir" -maxdepth 1 -name "*.png" ! -name "*_spritesheet*" | sort)
    if [ -z "$imgs" ]; then
        log "⚠️  sin PNGs en $dir"
        return 1
    fi
    # Todas al mismo alto y montadas en hoja de $cols columnas (espaciado 4px)
    # shellcheck disable=SC2046
    montage -background none -tile "${cols}x1" -geometry +4+4 \
        -resize x64 $(echo "$imgs" | tr '\n' ' ') "$out"
    log "✅ sprite sheet: $out ($(echo "$imgs" | wc -l) imágenes)"
}

tileset() {
    local dir="$1"
    local tam="${2:-32}"
    local ok=0; local mal=0
    for f in "$dir"/*.png; do
        [ -f "$f" ] || continue
        local w h
        w=$("$MAGICK" identify -format "%w" "$f")
        h=$("$MAGICK" identify -format "%h" "$f")
        if [ "$((w % tam))" -eq 0 ] && [ "$((h % tam))" -eq 0 ]; then
            ok=$((ok + 1))
        else
            mal=$((mal + 1))
            log "⚠️  tile fuera de grid ($w x $h): $(basename "$f")"
        fi
    done
    log "📐 tileset $dir: $ok en grid $tam px, $mal fuera"
    [ "$mal" -eq 0 ]
}

case "${1:-}" in
    limpiar)     limpiar "${2:?uso: limpiar <archivo|carpeta>}" ;;
    spritesheet) spritesheet "${2:?uso: spritesheet <carpeta> [cols]}" "${3:-4}" ;;
    tileset)     tileset "${2:?uso: tileset <carpeta> <tam>}" "${3:-32}" ;;
    *) echo "uso: $0 {limpiar|spritesheet|tileset} ..."; exit 1 ;;
esac
