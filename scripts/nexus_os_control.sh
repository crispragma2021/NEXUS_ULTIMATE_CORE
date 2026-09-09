#!/bin/bash
# NEXUS Core - Motor de Automatización Óptica y Control de Periféricos

ACCION="${1}"
PARAM1="${2}"
PARAM2="${3}"

OUTPUT_DIR="/home/soberano/NEXUS_ULTIMATE_CORE/brain/sessions"
SCREENSHOT="$OUTPUT_DIR/screen_latest.png"

mkdir -p "$OUTPUT_DIR"

case "$ACCION" in
    "ver")
        # Captura la pantalla mapeando coordenadas reales de la sesión X11
        DISPLAY=:0 scrot -u -z "$SCREENSHOT" 2>/dev/null || DISPLAY=:0 scrot -z "$SCREENSHOT"
        echo "[VISION] Captura de pantalla actualizada en: $SCREENSHOT"
        ;;
        
    "click")
        # Hace clic en coordenadas X e Y exactas entregadas por el análisis de la IA
        X=$PARAM1
        Y=$PARAM2
        DISPLAY=:0 xdotool mousemove $X $Y click 1
        echo "[INPUT] Clic izquierdo ejecutado en las coordenadas: X=$X, Y=$Y"
        ;;
        
    "escribir")
        # Escribe texto plano directamente en la ventana que esté activa
        TEXTO=$PARAM1
        DISPLAY=:0 xdotool type --delay 50 "$TEXTO"
        echo "[INPUT] Texto inyectado en el teclado."
        ;;
        
    "tecla")
        # Ejecuta atajos de teclado complejos (ej. "ctrl+shift+p", "Return", "Super")
        TECLA=$PARAM1
        DISPLAY=:0 xdotool key "$TECLA"
        echo "[INPUT] Pulsación de tecla enviada: $TECLA"
        ;;
        
    *)
        echo "Uso: ./scripts/nexus_os_control.sh {ver | click X Y | escribir \'texto\' | tecla \'tecla\'}"
        exit 1
        ;;
esac