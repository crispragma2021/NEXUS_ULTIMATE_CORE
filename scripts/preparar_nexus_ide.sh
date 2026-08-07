#!/bin/bash
# NEXUS IDE - Script de Preparación OMEGA
# Propósito: Materializar el entorno de desarrollo soberano.

set -e

echo "🤖 NEXUS: Iniciando Preparación de Entorno para NEXUS IDE..."

# 1. Verificación de Dependencias
echo "🔍 Verificando herramientas de forja..."
for cmd in node npm git python3 make g++; do
    if ! command -v $cmd &> /dev/null; then
        echo "❌ Error: $cmd no está instalado. Abortando misión."
        exit 1
    fi
done

# 2. Instalación de Yarn si no existe
if ! command -v yarn &> /dev/null; then
    echo "📦 Instalando Yarn globalmente..."
    sudo npm install -g yarn
fi

# 3. Clonación de VSCodium (Núcleo del IDE)
if [ ! -d "vscodium" ]; then
    echo "📡 Clonando núcleo VSCodium en la raíz..."
    git clone --depth 1 https://github.com/VSCodium/vscodium.git vscodium
else
    echo "✅ El núcleo VSCodium ya está presente."
fi

# 4. Preparación de dependencias del núcleo
cd vscodium
echo "🛠️ Configurando dependencias del núcleo..."
# El entorno requiere ciertas herramientas para la compilación nativa
# Referencia: VSCodium build instructions

echo "✅ Entorno preparado para 'yarn install' y transmutación de branding."
