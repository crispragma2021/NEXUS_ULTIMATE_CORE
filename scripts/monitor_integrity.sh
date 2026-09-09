#!/bin/bash

# Este script monitorea la integridad de binarios críticos comparando sus checksums SHA256.

NEXUS_CORE_DIR="/home/soberano/NEXUS_ULTIMATE_CORE"
CHECKSUMS_FILE="${NEXUS_CORE_DIR}/reports/audit/checksums.txt"
LOG_FILE="${NEXUS_CORE_DIR}/logs/integrity_monitor.log"

# Asegurarse de que el directorio de logs exista
mkdir -p "${NEXUS_CORE_DIR}/logs"

log_message() {
  echo "$(date '+%Y-%m-%d %H:%M:%S') - INTEGRITY MONITOR: $1" >> "${LOG_FILE}"
}

if [ ! -f "$CHECKSUMS_FILE" ]; then
  log_message "Error: Archivo de checksums no encontrado en '$CHECKSUMS_FILE'. Genera los checksums primero."
  exit 1
fi

log_message "Iniciando monitoreo de integridad de binarios críticos."

while IFS= read -r line; do
  OLD_CHECKSUM=$(echo "$line" | awk '{print $1}')
  FILE_PATH=$(echo "$line" | awk '{print $2}')

  if [ -f "$FILE_PATH" ]; then
    CURRENT_CHECKSUM=$(sha256sum "$FILE_PATH" | awk '{print $1}')
    if [ "$OLD_CHECKSUM" != "$CURRENT_CHECKSUM" ]; then
      log_message "ALERTA DE INTEGRIDAD: Checksum de '$FILE_PATH' ha cambiado. OLD: $OLD_CHECKSUM, NEW: $CURRENT_CHECKSUM"
    fi
  else
    log_message "ALERTA DE INTEGRIDAD: Archivo '$FILE_PATH' no encontrado."
  fi
done < "$CHECKSUMS_FILE"

log_message "Monitoreo de integridad finalizado."
