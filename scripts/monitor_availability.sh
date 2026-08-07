#!/bin/bash

# Este script monitorea la disponibilidad de servicios críticos.

LOG_FILE="/home/soberano/NEXUS_ULTIMATE_CORE/logs/availability_monitor.log"
NEXUS_CORE_DIR="/home/soberano/NEXUS_ULTIMATE_CORE"

# Asegurarse de que el directorio de logs exista
mkdir -p "${NEXUS_CORE_DIR}/logs"

log_message() {
  echo "$(date '+%Y-%m-%d %H:%M:%S') - AVAILABILITY MONITOR: $1" >> "${LOG_FILE}"
}

log_message "Iniciando monitoreo de disponibilidad de servicios."

# Monitorear puerto SSH (22)
if nc -z -w 1 127.0.0.1 22; then
  log_message "Servicio SSH (Puerto 22) en 127.0.0.1: OK"
else
  log_message "ALERTA DE DISPONIBILIDAD: Servicio SSH (Puerto 22) en 127.0.0.1: NO DISPONIBLE"
fi

# Monitorear proxy_hijack (4444)
if nc -z -w 1 127.0.0.1 4444; then
  log_message "Servicio proxy_hijack (Puerto 4444) en 127.0.0.1: OK"
else
  log_message "ALERTA DE DISPONIBILIDAD: Servicio proxy_hijack (Puerto 4444) en 127.0.0.1: NO DISPONIBLE"
fi

# Monitorear TLS Terminator (8443)
if nc -z -w 1 127.0.0.1 8443; then
  log_message "Servicio TLS Terminator (Puerto 8443) en 127.0.0.1: OK"
else
  log_message "ALERTA DE DISPONIBILIDAD: Servicio TLS Terminator (Puerto 8443) en 127.0.0.1: NO DISPONIBLE"
fi

log_message "Monitoreo de disponibilidad finalizado."
