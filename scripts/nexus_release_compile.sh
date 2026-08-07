#!/bin/bash

# Script de compilación optimizada para NEXUS UI (Tauri)
# Utiliza perfil 'release' con LTO y target-cpu para máximo rendimiento en Alder Lake.

# Asumimos que la compilación es para el crate 'nexus-ui' dentro del workspace
# y que se encuentra en src-tauri/.

# Limpiar compilaciones previas para asegurar una nueva construcción
echo "🧹 Limpiando compilaciones previas..."
cargo clean -p nexus-ui

# Compilar en modo release con todas las optimizaciones
echo "🚀 Iniciando compilación de NEXUS UI en modo release..."
cargo build --release -p nexus-ui

# Verificar si la compilación fue exitosa
if [ $? -eq 0 ]; then
    echo "✅ Compilación exitosa: El binario se encuentra en target/release/nexus-ui"
    echo "Para ejecutar: ./target/release/nexus-ui"
else
    echo "❌ Fallo en la compilación. Revisa los errores."
    exit 1
fi
