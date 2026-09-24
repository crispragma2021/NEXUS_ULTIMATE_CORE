#!/bin/bash
# NEXUS Core - Motor de Automatización Óptica, Percepción Semántica AT-SPI2 y Control de Periféricos (uinput/xdotool)

ACCION="${1}"
PARAM1="${2}"
PARAM2="${3}"

OUTPUT_DIR="${HOME}/NEXUS_ULTIMATE_CORE/brain/sessions"
SCREENSHOT="$OUTPUT_DIR/screen_latest.png"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

mkdir -p "$OUTPUT_DIR"

case "$ACCION" in
    "ver")
        # Captura la pantalla de la ventana activa o root
        WID=$(DISPLAY=:0 xdotool search --all --name "" 2>/dev/null | while read id; do
          name=$(DISPLAY=:0 xdotool getwindowname "$id" 2>/dev/null)
          if echo "$name" | grep -qiE "calculadora|gnome-calculator|chrome|visual studio"; then
            echo "$id"
          fi
        done | tail -n 1)

        if [ -n "$WID" ]; then
            DISPLAY=:0 maim -i "$WID" "$SCREENSHOT" 2>/dev/null || DISPLAY=:0 scrot -u -z "$SCREENSHOT" 2>/dev/null
        else
            DISPLAY=:0 maim "$SCREENSHOT" 2>/dev/null || DISPLAY=:0 scrot -z "$SCREENSHOT" 2>/dev/null
        fi
        echo "[VISION] Captura de pantalla actualizada en: $SCREENSHOT"
        ;;
        
    "atspi_click")
        # Clic semántico directo vía árbol de accesibilidad AT-SPI2 DBus
        APP="$PARAM1"
        ELEMENTO="$PARAM2"
        python3 "$SCRIPT_DIR/nexus_atspi_control.py" click "$APP" "$ELEMENTO"
        ;;

    "atspi_tree")
        # Inspección del árbol semántico de la app
        APP="$PARAM1"
        python3 "$SCRIPT_DIR/nexus_atspi_control.py" list "$APP"
        ;;

    "click")
        # Clic en coordenadas X e Y exactas
        X=$PARAM1
        Y=$PARAM2
        DISPLAY=:0 xdotool mousemove $X $Y click 1 2>/dev/null || YDOTOOL_SOCKET=/tmp/ydotool.socket ydotool mousemove -- $X $Y && YDOTOOL_SOCKET=/tmp/ydotool.socket ydotool click 0xC0
        echo "[INPUT] Clic izquierdo ejecutado en: X=$X, Y=$Y"
        ;;
        
    "escribir")
        # Inyectar texto vía xdotool / ydotool (uinput)
        TEXTO=$PARAM1
        DISPLAY=:0 xdotool type --delay 50 "$TEXTO" 2>/dev/null || YDOTOOL_SOCKET=/tmp/ydotool.socket ydotool type "$TEXTO"
        echo "[INPUT] Texto inyectado en el teclado."
        ;;
        
    "tecla")
        # Enviar combinación de teclas
        TECLA=$PARAM1
        DISPLAY=:0 xdotool key "$TECLA" 2>/dev/null || YDOTOOL_SOCKET=/tmp/ydotool.socket ydotool key "$TECLA"
        echo "[INPUT] Pulsación de tecla enviada: $TECLA"
        ;;
        
    *)
        echo "Uso: ./scripts/nexus_os_control.sh {ver | atspi_click app elemento | atspi_tree app | click X Y | escribir 'texto' | tecla 'tecla'}"
        exit 1
        ;;
esac