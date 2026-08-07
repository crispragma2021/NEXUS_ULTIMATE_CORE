#!/bin/bash
# 🔱 NEXUS-LIVE: Soberana Ignición de la Faz OMEGA 🎭🚀✨
# Este comando lanza la instancia aislada de VSCodium con la piel de NEXUS.

BASE_DIR="/home/soberano/NEXUS_ULTIMATE_CORE"
BIN="$BASE_DIR/faz_omega/usr/share/codium/bin/codium"
EXTS="$BASE_DIR/faz_omega/extensions"
DATA="$BASE_DIR/brain/user_data_nexus" # Separación de datos para evitar colisiones
EXTRACTOR="$BASE_DIR/target/release/nexus-orquestador"

echo "🎭 [LIVE] Lanzando Faz OMEGA Soberana..."

# Asegurar que el entorno gráfico es accesible para las herramientas de visión
export DISPLAY=${DISPLAY:-:0}
export XAUTHORITY=${XAUTHORITY:-$HOME/.Xauthority}

# Asegurar que el directorio de datos existe
mkdir -p "$DATA"

# 🔌 NEXUS_VISION (Tauri) maneja ahora el core neural.
# if [ -f "$EXTRACTOR" ]; then
#     echo "🔌 [LIVE] Arrancando Daemon Extractor Rust (headless) en :43211..."
#     "$EXTRACTOR" --daemon &
#     EXTRACTOR_PID=$!
#     sleep 5
#     echo "✅ [LIVE] Extractor PID: $EXTRACTOR_PID"
# else
#     echo "⚠️ [LIVE] Extractor no encontrado."
# fi

# Cleanup al salir (matar el daemon extractor y limpiar zombies)
cleanup() {
    if [ -n "$EXTRACTOR_PID" ]; then
        echo "🛑 [LIVE] Deteniendo Daemon Extractor (PID: $EXTRACTOR_PID)..."
        kill "$EXTRACTOR_PID" 2>/dev/null
        sleep 1
        # PURGA DE ZOMBIES: Limpiar cualquier Chrome residual vinculado a los perfiles del proyecto
        echo "🧹 [LIVE] Purgando procesos Chrome residuales (Fantasmas/Zombies)..."
        pkill -9 -f "chrome.*--user-data-dir=$BASE_DIR/profiles"
        echo "✅ [LIVE] Limpieza completada."
    fi
}
trap cleanup EXIT

# Ignición
$BIN --extensions-dir "$EXTS" --user-data-dir "$DATA" --locale es "$BASE_DIR" "$@"
