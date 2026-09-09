#!/bin/bash
# =================================================================
# 🧬 OMEGA BUILD - Sincronización Total NEXUS
# =================================================================
# Fusiona: build.sh (Frontend) + build_nexus.sh (Hardware Opt)
# Seguridad: Pilar 11 (Límite de hilos dinámico)
# =================================================================

# 1. Definir Ruta Maestra
NEXUS_PATH="/home/soberano/NEXUS_ULTIMATE_CORE"
cd "$NEXUS_PATH" || exit

# 2. Detección de Hardware
CPU_MODEL=$(grep -m1 "model name" /proc/cpuinfo | cut -d: -f2)
NPROC=$(nproc)
if [ "$NPROC" -gt 4 ]; then
    DEFAULT_JOBS=$((NPROC - 4))
else
    DEFAULT_JOBS=2
fi
JOBS=${1:-$DEFAULT_JOBS} # Límite dinámico para estabilidad (Pilar 11)

echo "🚀 Iniciando Fusión OMEGA en: $CPU_MODEL"
echo "⚡ Hilos asignados: $JOBS (de $NPROC hilos totales)"

# 3. Compilación de Órganos (Frontend)
echo "🎨 Compilando Interfaz (Frontend)..."
if [ -f "package.json" ]; then
    npm run build
fi

# 4. Configuración de ADN (Rust Flags)
if [[ "$CPU_MODEL" == *"i7"* ]]; then
    export RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C codegen-units=1"
else
    export RUSTFLAGS="-C opt-level=3"
fi

# 5. Compilación del Núcleo (Backend)
echo "🦀 Compilando Núcleo en modo Release con $JOBS hilos..."
cargo build --release --jobs "$JOBS"

echo "✅ Fusión completada con éxito."
