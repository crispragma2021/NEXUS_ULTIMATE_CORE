#!/bin/bash
export DISPLAY=:0
pkill -f gnome-calculator || true
GDK_BACKEND=x11 gnome-calculator &
sleep 6
WID=$(wmctrl -l | grep "Calculadora" | head -n 1 | awk '{print $1}')
if [ -z "$WID" ]; then
    echo "⚠️ [CNS] Fallo al localizar ventana de Calculadora."
    exit 1
fi
echo "🦾 [CNS] Ventana localizada: $WID. Activando..."
wmctrl -i -a $WID
xdotool windowactivate --sync $WID
xdotool sleep 1 type "1985"
echo "✅ [CNS] Ráfaga 1985 manifestada."
scrot /home/soberano/NEXUS_ULTIMATE_CORE/data/vision/1985_OMEGA_FINAL.png
