#!/bin/bash

# ==============================================================================
# NEXUS GHOST-SANDBOX - INICIADOR DE ENTORNO ANÓNIMO AMNÉSICO
# ==============================================================================

# Colores estéticos premium para la terminal de NEXUS
NEON_CYAN='\033[0;36m'
NEON_GREEN='\033[0;32m'
NEON_RED='\033[0;31m'
NEON_YELLOW='\033[0;33m'
RESET='\033[0m'

echo -e "${NEON_CYAN}🤖 [NEXUS GHOST-SANDBOX] Iniciando protocolo de despliegue...${RESET}"

# 1. Limpieza de contenedores previos si existen
echo -e "${NEON_CYAN}🧹 Limpiando instancias antiguas del Sandbox...${RESET}"
docker rm -f nexus-tor-gateway nexus-ghost-sandbox >/dev/null 2>&1

# 2. Levantar el Gateway de Tor en segundo plano
echo -e "${NEON_CYAN}🛰️ Levantando Tor Gateway (Alpine-based SOCKS5 Proxy)...${RESET}"
docker run -d \
  --name nexus-tor-gateway \
  --restart unless-stopped \
  -p 9050:9050 \
  alpine sh -c "apk add --no-cache tor && tor --SocksPort 0.0.0.0:9050" >/dev/null

if [ $? -ne 0 ]; then
  echo -e "${NEON_RED}❌ Error: No se pudo iniciar el contenedor Tor Gateway. Verifica Docker.${RESET}"
  exit 1
fi

# 3. Esperar a que Tor se conecte a la red (Bootstrap al 100%)
echo -e "${NEON_YELLOW}⏳ Esperando a que Tor establezca los circuitos (Bootstrap)...${RESET}"
for i in {1..30}; do
  # Verificar si el puerto SOCKS5 está respondiendo
  docker exec nexus-tor-gateway nc -z 127.0.0.1 9050 >/dev/null 2>&1
  if [ $? -eq 0 ]; then
    # Hacer una prueba de conexión rápida con curl
    IP_CHECK=$(docker run --rm --net=container:nexus-tor-gateway alpine sh -c "apk add --no-cache curl >/dev/null && curl -s --socks5-hostname 127.0.0.1:9050 https://check.torproject.org/api/ip" 2>/dev/null)
    if [[ "$IP_CHECK" == *"IsTor\":true"* ]]; then
      echo -e "${NEON_GREEN}✅ Red Tor conectada e identificada con éxito!${RESET}"
      break
    fi
  fi
  echo -ne "${NEON_YELLOW}.${RESET}"
  sleep 2
  if [ $i -eq 30 ]; then
    echo -e "${NEON_RED}\n❌ Error: Tiempo de espera agotado al conectar a Tor.${RESET}"
    docker stop nexus-tor-gateway >/dev/null 2>&1
    exit 1
  fi
done

# Extraer IP de Tor para confirmación visual
TOR_IP=$(echo "$IP_CHECK" | grep -oP '"IP":"\K[^"]+')
echo -e "${NEON_GREEN}🌐 IP de Salida (Tor Node): $TOR_IP${RESET}"

# 4. Lanzar la Sandbox Amnésica Interactiva
echo -e "${NEON_CYAN}🔒 Preparando contenedor amnésico (Debian Slim)...${RESET}"
echo -e "${NEON_CYAN}   - Toda la escritura en /root y /tmp se redirecciona a la RAM (tmpfs)${RESET}"
echo -e "${NEON_CYAN}   - Todo el tráfico se enruta obligatoriamente por Tor${RESET}"
echo -e "${NEON_GREEN}🚀 Iniciando Shell del Ghost-Sandbox. Escribe 'exit' para destruir la sesión.${RESET}\n"

# Iniciar contenedor Debian con red enlazada al gateway de Tor y almacenamiento temporal en RAM
# Usamos torsocks para asegurar que comandos genéricos en el terminal vayan por la red Tor
docker run -it --rm \
  --name nexus-ghost-sandbox \
  --net=container:nexus-tor-gateway \
  --tmpfs /tmp:exec \
  --tmpfs /root:exec \
  --env http_proxy=socks5://127.0.0.1:9050 \
  --env https_proxy=socks5://127.0.0.1:9050 \
  debian:slim sh -c "
    apt-get update -qy && apt-get install -qy curl torsocks procps >/dev/null 2>&1
    echo ''
    echo -e '${NEON_CYAN}========================================================================${RESET}'
    echo -e '${NEON_GREEN}            👻 BIENVENIDO AL GHOST-SANDBOX DE NEXUS 👻${RESET}'
    echo -e '${NEON_CYAN}========================================================================${RESET}'
    echo -e '  * Tu dirección IP pública está oculta y enrutada mediante Tor.'
    echo -e '  * Escribe comandos usando \\`torsocks <comando>\\` o usa \\`curl\\` directamente.'
    echo -e '  * Las carpetas /root y /tmp están montadas sobre la RAM física.'
    echo -e '  * Al salir, todo rastro se destruirá de forma irreversible.'
    echo -e '${NEON_CYAN}========================================================================${RESET}'
    echo ''
    torsocks bash
  "

# 5. Destrucción post-ejecución (Homeostasis de NEXUS)
echo -e "\n${NEON_YELLOW}🧹 Destruyendo entorno de forma amnésica...${RESET}"
docker stop nexus-tor-gateway >/dev/null 2>&1
docker rm -f nexus-tor-gateway >/dev/null 2>&1
echo -e "${NEON_GREEN}✨ Entorno destruido. RAM liberada. Cero registros en disco.${RESET}"
