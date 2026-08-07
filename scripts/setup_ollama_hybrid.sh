#!/usr/bin/env bash
# ==============================================================================
#                 🧬 SETUP OLLAMA HYBRID - ARCHITECTURE CONVERSOR
# ==============================================================================
# Configura el entorno local híbrido de Ollama para optimizar la VRAM (8GB GPU)
# y la RAM (64GB CPU) con los modelos Qwen 3.6 e Llama 3.1 de forma inteligente.
# ==============================================================================

set -euo pipefail

# Colores para salida visual limpia
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0;0m'

echo -e "${BLUE}⚡ [NEXUS HYBRID SETUP] Inicializando arquitectura híbrida de Ollama...${NC}"

# 1. Detectar si Ollama está instalado y accesible
if ! command -v ollama &> /dev/null; then
    echo -e "${YELLOW}⚠️  Ollama no se encuentra en el PATH de la sesión actual.${NC}"
    echo -e "Intentando ubicar el binario alternativo..."
    
    # Comprobar ruta alternativa del proyecto
    if [ -f "/home/soberano/NEXUS_ULTIMATE_CORE/nexusclaw/ollama" ] && [ -s "/home/soberano/NEXUS_ULTIMATE_CORE/nexusclaw/ollama" ]; then
        OLLAMA_CMD="/home/soberano/NEXUS_ULTIMATE_CORE/nexusclaw/ollama"
        echo -e "${GREEN}✓ Binario de Ollama encontrado en el workspace: $OLLAMA_CMD${NC}"
    else
        echo -e "${RED}❌ Ollama no está instalado o habilitado en este entorno NixOS.${NC}"
        echo -e "Para habilitarlo en NixOS, asegúrate de añadir lo siguiente a tu configuración de Nix:"
        echo -e "${YELLOW}  services.ollama.enable = true;${NC}"
        echo -e "${YELLOW}  services.ollama.acceleration = \"rocm\"; # O \"cuda\" si usas drivers Nvidia propietarios${NC}"
        exit 1
    fi
else
    OLLAMA_CMD="ollama"
    echo -e "${GREEN}✓ Comando 'ollama' global detectado.${NC}"
fi

# 2. Verificar que el daemon de Ollama esté activo
echo -e "${BLUE}🔍 Verificando conexión con el servidor de Ollama (localhost:11434)...${NC}"
if ! curl -s http://localhost:11434/ &> /dev/null; then
    echo -e "${YELLOW}⚠️  El servidor de Ollama no está corriendo.${NC}"
    echo -e "Intentando iniciar el servicio..."
    
    # Intentar arrancar servicio de systemd si existe
    if systemctl --user list-unit-files | grep -q "ollama.service"; then
        systemctl --user start ollama.service
        sleep 2
    elif systemctl list-unit-files | grep -q "ollama.service"; then
        echo -e "Iniciando servicio de sistema (puede requerir sudo)..."
        sudo systemctl start ollama.service
        sleep 2
    else
        echo -e "Ejecutando daemon de Ollama en segundo plano..."
        $OLLAMA_CMD serve &
        sleep 3
    fi
    
    # Comprobar nuevamente
    if ! curl -s http://localhost:11434/ &> /dev/null; then
        echo -e "${RED}❌ No se pudo conectar con el servidor de Ollama. Inícialo manualmente antes de correr este script.${NC}"
        exit 1
    fi
fi
echo -e "${GREEN}✓ Servidor de Ollama conectado con éxito.${NC}"

# 3. Pull de modelos base
echo -e "\n${BLUE}📥 Descargando modelos base de Ollama...${NC}"
echo -e "${YELLOW}[Paso 1/3] Descargando Qwen 3.6 (14B) para VRAM...${NC}"
$OLLAMA_CMD pull qwen3.6:14b

echo -e "\n${YELLOW}[Paso 2/3] Descargando Qwen 3.6 (27B) para RAM...${NC}"
$OLLAMA_CMD pull qwen3.6:27b

echo -e "\n${YELLOW}[Paso 3/3] Descargando Llama 3.1 Abliterated (8B) para Chat sin censura (VRAM)...${NC}"
$OLLAMA_CMD pull mannix/llama3.1-8b-abliterated:latest

# 4. Creación de los perfiles híbridos
echo -e "\n${BLUE}⚙️  Creando perfiles personalizados en Ollama...${NC}"

# VRAM/GPU Coder
echo -e "Construyendo modelo ${GREEN}nexuslocal-vram${NC} (Qwen 3.6 para GPU/RTX 3070)..."
$OLLAMA_CMD create nexuslocal-vram -f /home/soberano/NEXUS_ULTIMATE_CORE/config/ollama/Modelfile.vram

# RAM/CPU Worker
echo -e "Construyendo modelo ${GREEN}nexuslocal-ram${NC} (Qwen 3.6 para RAM 64GB / CPU)..."
$OLLAMA_CMD create nexuslocal-ram -f /home/soberano/NEXUS_ULTIMATE_CORE/config/ollama/Modelfile.ram

# Llama 3.1 Chat/Alineación libre en VRAM
echo -e "Construyendo modelo ${GREEN}nexuslocal-llama3.1${NC} (Llama 3.1 para GPU/Chat)..."
$OLLAMA_CMD create nexuslocal-llama3.1 -f /home/soberano/NEXUS_ULTIMATE_CORE/config/ollama/Modelfile.llama3.1

# 5. Resumen de ejecución
echo -e "\n${GREEN}🚀 ¡CONFIGURACIÓN DE ARQUITECTURA HÍBRIDA COMPLETADA CON ÉXITO!${NC}"
echo -e "=========================================================================="
echo -e " 1. ${BLUE}nexuslocal-vram${NC}     -> Qwen 3.6 14B (GPU/VRAM). Desarrollo rápido."
echo -e "    Comando manual: ${YELLOW}ollama run nexuslocal-vram${NC}"
echo -e " 2. ${BLUE}nexuslocal-ram${NC}      -> Qwen 3.6 27B (RAM/CPU). Análisis de fondo."
echo -e "    Comando manual: ${YELLOW}ollama run nexuslocal-ram${NC}"
echo -e " 3. ${BLUE}nexuslocal-llama3.1${NC}  -> Llama 3.1 8B Abliterated (GPU/VRAM). Chat libre."
echo -e "    Comando manual: ${YELLOW}ollama run nexuslocal-llama3.1${NC}"
echo -e "=========================================================================="
echo -e "Todos los modelos están configurados con ${YELLOW}keep_alive: 0${NC}, lo que garantiza"
echo -e "que se descargarán de inmediato cuando terminen de responder (0% de impacto en reposo)."
echo -e "=========================================================================="
