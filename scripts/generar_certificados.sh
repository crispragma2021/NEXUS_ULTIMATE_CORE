#!/bin/bash
# 🔱 NEXUS CA and Certificate Generator
# Este script genera la Autoridad de Certificación (CA) local de NEXUS
# y crea el certificado HTTPS para interceptar los dominios de Google.

set -e

WORKSPACE_DIR="/home/soberano/NEXUS_ULTIMATE_CORE"
SECRETS_DIR="${WORKSPACE_DIR}/secrets"

mkdir -p "$SECRETS_DIR"
cd "$SECRETS_DIR"

echo "🔱 [1/4] Generando clave privada y certificado de NEXUS Root CA..."
if [ ! -f "nexus-ca.key" ]; then
    openssl genrsa -out nexus-ca.key 2048
    openssl req -x509 -new -nodes -key nexus-ca.key -sha256 -days 3650 \
        -out nexus-ca.pem \
        -subj "/CN=NEXUS Sovereign CA/O=NEXUS Ultimate Core/C=CL"
    echo "✅ NEXUS Root CA generada."
else
    echo "⚠️  nexus-ca.key ya existe. Omitiendo generación de la CA."
fi

echo "🔱 [2/4] Creando extensión de configuración DNS para los dominios..."
cat << 'EOF' > nexus-dns.ext
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = cloudcode-pa.googleapis.com
DNS.2 = generativelanguage.googleapis.com
DNS.3 = localhost
IP.1 = 127.0.0.1
EOF
echo "✅ Archivo nexus-dns.ext creado."

echo "🔱 [3/4] Generando clave privada y CSR del servidor..."
openssl genrsa -out nexus-server.key 2048
openssl req -new -key nexus-server.key -out nexus-server.csr \
    -subj "/CN=cloudcode-pa.googleapis.com/O=NEXUS Ultimate Core/C=CL"

echo "🔱 [4/4] Firmando certificado del servidor con la CA raíz de NEXUS..."
openssl x509 -req -in nexus-server.csr -CA nexus-ca.pem -CAkey nexus-ca.key \
    -CAcreateserial -out nexus-server.pem -days 365 -sha256 -extfile nexus-dns.ext

# Limpiar CSR y serial
rm -f nexus-server.csr nexus-ca.srl

# Asegurar permisos correctos
chmod 600 nexus-ca.key nexus-server.key
chmod 644 nexus-ca.pem nexus-server.pem

echo "🎉 Certificados generados con éxito en $SECRETS_DIR:"
echo "   - nexus-ca.pem (Certificado Raíz de la CA)"
echo "   - nexus-ca.key (Clave Privada de la CA - Mantener segura)"
echo "   - nexus-server.pem (Certificado HTTPS del Servidor)"
echo "   - nexus-server.key (Clave Privada HTTPS del Servidor)"
