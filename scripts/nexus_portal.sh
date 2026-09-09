#!/bin/bash
# NEXUS-PORTAL: Puerta de Enlace a la Guerra Cibernética y Defensa Familiar.
# Autor: NEXUS OMEGA

echo -e "\e[1;36m🔱 [NEXUS-PORTAL] --- ACCEDIENDO A THE SPECTER CELL ---\e[0m"
echo -e "\e[1;31m🛡️ [GUARD] Sentinel-Mama (MERCURY): PROTECCIÓN ACTIVADA\e[0m"

# Limpieza de procesos previos para sintonía total
echo -e "\e[1;33m🧹 Limpiando sesiones zombis...\e[0m"
pkill -f "nexus-orquestador" || true
pkill -f "vite" || true

# Lanzar el orquestador en modo Portal
echo -e "\e[1;32m🖼️  Abriendo Portal Soberano en BRAVE (Perfil Aislado @ 5173)...\e[0m"
/opt/brave-bin/brave --user-data-dir=/tmp/nexus_portal_profile --app=http://localhost:5173/ &

# Levantar el servidor frontend Vite en segundo plano
if [ -d "/home/soberano/NEXUS_ULTIMATE_CORE/src/ui" ]; then
    echo -e "\e[1;34m📡 Iniciando Servidor Vite...\e[0m"
    cd /home/soberano/NEXUS_ULTIMATE_CORE/src/ui && nohup npm run dev -- --port 5173 --host > /tmp/vite_portal.log 2>&1 &
fi

# Lanzar la Bestia (HUD Real @ 43211)
echo -e "\e[1;31m💀 [NEXUS-BEAST] Despertando HUD de Guerra (Puerto 43211)...\e[0m"
cd /home/soberano/NEXUS_ULTIMATE_CORE/target/release && nohup ./nexus-orquestador > /tmp/nexus_beast.log 2>&1 &

echo -e "\e[1;32m✅ NEXUS en Sintonía. Portal operativo.\e[0m"

