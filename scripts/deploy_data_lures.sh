#!/bin/bash

# Este script despliega señuelos de datos (honeytokens) en ubicaciones atractivas.

NEXUS_CORE_DIR="/home/soberano/NEXUS_ULTIMATE_CORE"
LOG_FILE="${NEXUS_CORE_DIR}/logs/data_lures.log"

# Asegurarse de que el directorio de logs exista
mkdir -p "${NEXUS_CORE_DIR}/logs"

log_message() {
  echo "$(date '+%Y-%m-%d %H:%M:%S') - DATA LURE: $1" >> "${LOG_FILE}"
}

echo "Desplegando señuelos de datos (honeytokens)..."

# Honeytoken 1: Credenciales de base de datos falsas
echo "[database]" > "${NEXUS_CORE_DIR}/data/db_credentials.conf"
echo "host=192.168.1.100" >> "${NEXUS_CORE_DIR}/data/db_credentials.conf"
echo "user=admin_db_prod" >> "${NEXUS_CORE_DIR}/data/db_credentials.conf"
echo "password=FakePassw0rdForHoneyPot_DoNotUse_Alert_123" >> "${NEXUS_CORE_DIR}/data/db_credentials.conf"
echo "port=5432" >> "${NEXUS_CORE_DIR}/data/db_credentials.conf"
echo "database=production_db" >> "${NEXUS_CORE_DIR}/data/db_credentials.conf"
log_message "Honeytoken 'db_credentials.conf' desplegado en data/"

# Honeytoken 2: Clave API de prueba en un archivo .env.example
echo "TEST_API_KEY=test_api_key_IfYouSeeThisAlertNEXUS_456" > "${NEXUS_CORE_DIR}/.env.example"
echo "TEST_SECRET=test_secret_IfYouSeeThisAlertNEXUS_789" >> "${NEXUS_CORE_DIR}/.env.example"
log_message "Honeytoken '.env.example' desplegado"

# Honeytoken 3: Documento sensible simulado
echo "CONFIDENTIAL: Client Data Export - Project Chimera" > "${NEXUS_CORE_DIR}/docs/client_data_export_2026_Q2.txt"
echo "Contains sensitive client information. Access is restricted." >> "${NEXUS_CORE_DIR}/docs/client_data_export_2026_Q2.txt"
echo "Sample Entry: Client ID: C001, Name: John Doe, Email: john.doe@example.com, Honeytoken: ThisDataIsFake_Alert_789" >> "${NEXUS_CORE_DIR}/docs/client_data_export_2026_Q2.txt"
log_message "Honeytoken 'client_data_export_2026_Q2.txt' desplegado en docs/"

# Honeytoken 4: Clave SSH privada falsa
# Generar una clave RSA falsa
ssh-keygen -t rsa -b 4096 -f "${NEXUS_CORE_DIR}/secrets/fake_id_rsa" -N "" -C "fake_nexus_key@honeypot" > /dev/null 2>&1
echo "# This is a fake SSH key for honeypot purposes. If accessed, ALERT NEXUS." >> "${NEXUS_CORE_DIR}/secrets/fake_id_rsa"
log_message "Honeytoken 'fake_id_rsa' desplegado en secrets/"


echo "Señuelos de datos desplegados. El log de acceso a estos señuelos se registrará en ${LOG_FILE}"
echo "Recuerda que la detección de acceso a estos archivos debe hacerse externamente (ej. monitoreando accesos a archivos o uso de credenciales)."
