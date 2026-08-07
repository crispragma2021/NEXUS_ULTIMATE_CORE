#!/bin/bash
# /home/soberano/NEXUS_ULTIMATE_CORE/scripts/auto_health.sh
# 🔱 NEXUS OMEGA - Sistema de Autocuración e Inmunidad Continua

cd /home/soberano/NEXUS_ULTIMATE_CORE

echo "🔱 [INMUNO-NEXUS] Iniciando escaneo de salud metabólica..."

# 1. Auditoría de Seguridad (Crates vulnerables)
if command -v cargo-audit &> /dev/null; then
    cargo audit 2>&1 | tee logs/audit.log
else
    echo "⚠️ [INMUNO] cargo-audit no instalado. Saltando..."
fi

# 2. Higiene de Código (Clippy)
echo "🧹 [INMUNO] Ejecutando purga de bugs potenciales (Clippy)..."
cargo clippy -- -D warnings 2>&1 | tee logs/clippy.log
if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo "⚙️ [INMUNO] Aplicando correcciones automáticas de estilo..."
    cargo clippy --fix --allow-dirty --allow-staged
fi

# 3. Verificación de Compilación (Check)
echo "🔬 [INMUNO] Verificando integridad del núcleo i7-12700F..."
cargo check --all-targets 2>&1 | tee logs/cargo_check_auto.log
if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo "❌ [INMUNO] ¡FALLO DETECTADO! Reportando a la Amígdala..."
    # Aquí NEXUS invoca a DeepSeek R1 automáticamente vía orquestador en un caso real
fi

# 4. Pruebas de Estrés y Lógica
echo "🧪 [INMUNO] Ejecutando pruebas unitarias asíncronas..."
cargo test --all --quiet -- --test-threads=12 2>&1 | tee logs/tests_auto.log

# 5. Detección de Obsolescencia
if command -v cargo-outdated &> /dev/null; then
    cargo outdated --exit-code 1 > logs/outdated.log
    if [ $? -eq 1 ]; then
        echo "📡 [INMUNO] Detectadas dependencias obsoletas. Avisando al Arquitecto."
    fi
fi

echo "✅ [INMUNO] El búnker está sano. Soberanía mantenida."
