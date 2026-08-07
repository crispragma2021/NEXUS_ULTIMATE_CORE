#!/bin/sh
# ============================================================
# RUN SOVEREIGN - NEXUS OMEGA LAUNCHER
# ============================================================
# Automatiza el arranque de eBPF en el Kernel (sudo) y la ventana
# gráfica SDF de la GPU en la sesión activa de Wayland.
# ============================================================

cd /home/soberano/NEXUS_ULTIMATE_CORE

echo "📡 [NEXUS LAUNCH] Iniciando sensor eBPF a nivel de Kernel..."
echo "nuevaera!" | sudo -S nix-shell --run "cargo run -p nexus_ebpf --bin nexus_ebpf" > /tmp/nexus_ebpf.log 2>&1 &

echo "🎨 [NEXUS LAUNCH] Detectando sesión gráfica e iniciando visualizador GPU..."
export WAYLAND_DISPLAY=wayland-0
export XDG_RUNTIME_DIR=/run/user/1000
export DISPLAY=:0

nix-shell --run "cargo run --bin demo_userland_sdf" > /tmp/nexus_sdf.log 2>&1 &

echo "✅ [NEXUS LAUNCH] Ambos procesos enviados al background. Revisa tu pantalla gráfica."
