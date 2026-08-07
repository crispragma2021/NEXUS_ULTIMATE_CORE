#!/bin/bash
echo "=== GENERANDO AUTORIDAD DE CERTIFICACIÓN LOCAL (NEXUS CA) ==="

# 1. Crear directorio en el Santuario OMEGA
mkdir -p /home/soberano/NEXUS_ULTIMATE_CORE/secrets && cd /home/soberano/NEXUS_ULTIMATE_CORE/secrets

# 2. Generar la llave privada de la CA
openssl genrsa -out nexusCA.key 4096

# 3. Generar el certificado de la CA (Válido por 10 años)
openssl req -x509 -new -nodes -key nexusCA.key -sha256 -days 3650 \
    -out nexusCA.pem \
    -subj "/C=PY/L=Asuncion/O=NEXUS CORE/CN=Nexus Sovereign CA"

# 4. Instalar en el almacén de confianza del sistema
echo "-> Instalando certificado en el sistema operativo..."
sudo cp nexusCA.pem /usr/local/share/ca-certificates/nexusCA.crt
sudo update-ca-certificates

echo -e "\n✅ Certificado CA generado e instalado en /home/soberano/NEXUS_ULTIMATE_CORE/secrets"