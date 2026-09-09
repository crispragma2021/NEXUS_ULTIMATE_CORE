#!/bin/bash

# Este script despliega señuelos de red (entradas falsas en /etc/hosts)
# y configura un listener básico para registrar conexiones.

LURES_FILE="/etc/hosts.lures"
NEXUS_CORE_DIR="/home/soberano/NEXUS_ULTIMATE_CORE"
LOG_FILE="${NEXUS_CORE_DIR}/logs/network_lures.log"

# Asegurarse de que el directorio de logs exista
mkdir -p "${NEXUS_CORE_DIR}/logs"

# Limpiar entradas previas para evitar duplicados
echo nexus | sudo -S sed -i '/# NEXUS_LURE_START/,/# NEXUS_LURE_END/d' /etc/hosts

# Añadir entradas falsas a /etc/hosts (simulando servicios atractivos)
echo -e "\n# NEXUS_LURE_START" | sudo tee -a /etc/hosts > /dev/null
echo "127.0.0.1\tapp.nexus-dev.local" | sudo tee -a /etc/hosts > /dev/null
echo "127.0.0.1\tdb.nexus-prod.local" | sudo tee -a /etc/hosts > /dev/null
echo "127.0.0.1\tadmin.nexus-portal.local" | sudo tee -a /etc/hosts > /dev/null
echo "# NEXUS_LURE_END" | sudo tee -a /etc/hosts > /dev/null

log_message() {
  echo "$(date '+%Y-%m-%d %H:%M:%S') - NETWORK LURE: $1" >> "${LOG_FILE}"
}

echo "Desplegando señuelos de red... Entradas añadidas a /etc/hosts."
echo "Iniciando listener de señuelos en segundo plano (puertos 80, 443, 3306, 5432, 22):"

# Listener para puertos comunes. Usar netcat o un script simple para simular un servicio.
# Este es un listener muy básico, solo registra la conexión.
# Para evitar bloquear el script, se ejecuta en un subshell en segundo plano.

(
while true; do
    # Puerto HTTP/S
    echo nexus | sudo -S nc -l -p 80 -c 'echo -e "HTTP/1.1 200 OK\r\n\r\nHello from honeypot (port 80)"' >> "${LOG_FILE}" 2>&1 &
    echo nexus | sudo -S nc -l -p 443 -c 'echo -e "HTTP/1.1 200 OK\r\n\r\nHello from honeypot (port 443)"' >> "${LOG_FILE}" 2>&1 &
    # Puertos de DB
    echo nexus | sudo -S nc -l -p 3306 -c 'echo -e "MySQL Honeypot: Access Denied"' >> "${LOG_FILE}" 2>&1 &
    echo nexus | sudo -S nc -l -p 5432 -c 'echo -e "PostgreSQL Honeypot: Authentication Failed"' >> "${LOG_FILE}" 2>&1 &
    # Puerto SSH
    echo nexus | sudo -S nc -l -p 22 -c 'echo -e "SSH Honeypot: Connection refused (simulated)"' >> "${LOG_FILE}" 2>&1 &

    wait # Esperar a que todos los netcat terminen (por ejemplo, después de una conexión)
    sleep 1 # Pequeña pausa antes de relanzar los listeners
done
) &

log_message "Network lures deployed and listeners started."
echo "Monitoreando conexiones a señuelos en ${LOG_FILE}"
echo "Recuerda que este listener se ejecuta en segundo plano. Para detenerlo, busca el proceso 'nc -l' o 'deploy_network_lures.sh' y mátalo."
