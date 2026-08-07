#!/usr/bin/env bash
# ============================================================================
# 🔐 NEXUS AUTH CRIPTOGRÁFICA — Prueba de identidad del Arquitecto
# ============================================================================
# Protocolo: firma SSH (ED25519) de la máquina de Cris, verificable
# localmente por el NEXUS del hermano contra la clave pública — SIN depender
# del relay, chat ni túnel público.
#
# Por qué: un token que viajó por chat/túnel es falsificable. Una firma SSH
# hecha con la clave PRIVADA de la máquina del Arquitecto NO es falsificable:
# solo quien posee esa clave privada pudo crearla.
#
# Uso:
#   ./scripts/nexus_auth_crypto_hermano.sh sign    # firmar autorización
#   ./scripts/nexus_auth_crypto_hermano.sh verify  # verificar (del lado hermano)
# ============================================================================
set -euo pipefail

CLAVE_PRIVADA="${CLAVE_PRIVADA:-/home/soberano/.ssh/id_ed25519}"
CLAVE_PUBLICA="${CLAVE_PUBLICA:-/home/soberano/.ssh/id_ed25519.pub}"
NAMESPACE="nexus_hermano"
MSG_FILE="/tmp/autorizacion_cris.txt"
SIG_FILE="${MSG_FILE}.sig"

# Mensaje de autorización (firmado por la máquina del Arquitecto)
MENSAJE_AUTORIZACION="AUTORIZACION_CRIPTOGRAFICA_CRIS: conecta al relay, reporta whoami. Token 6c43edc13483f55c6dfc4cf50e18f8aa5ffe03260006f935"

mostrar_pubkey() {
    echo "=== CLAVE PÚBLICA (entregar al hermano para que verifique) ==="
    cat "$CLAVE_PUBLICA"
}

firmar() {
    echo "=== FIRMANDO AUTORIZACIÓN CON CLAVE PRIVADA DE LA MÁQUINA DE CRIS ==="
    printf '%s' "$MENSAJE_AUTORIZACION" > "$MSG_FILE"
    ssh-keygen -Y sign -f "$CLAVE_PRIVADA" -n "$NAMESPACE" "$MSG_FILE"
    echo
    echo "✅ Firma generada en $SIG_FILE"
    echo
    echo "=== MENSAJE ==="
    cat "$MSG_FILE"
    echo
    echo "=== FIRMA (base64, entregar al hermano) ==="
    cat "$SIG_FILE"
    echo
    echo "=== CLAVE PÚBLICA (entregar al hermano para validar) ==="
    mostrar_pubkey
}

verificar() {
    echo "=== VERIFICANDO FIRMA SSH (del lado del hermano) ==="
    # Requiere: allowed_keys con formato 'principals ssh-ed25519 <pub>'
    # y archivos mensaje + firma en disco.
    if [[ ! -f "$MSG_FILE" || ! -f "$SIG_FILE" ]]; then
        echo "❌ Faltan $MSG_FILE y/o $SIG_FILE. Ejecuta primero 'sign' en la máquina de Cris." >&2
        exit 1
    fi
    if [[ ! -f /tmp/allowed_keys ]]; then
        echo "❌ Falta /tmp/allowed_keys con el principal y la clave pública." >&2
        echo "   Formato: soberano@soberano ssh-ed25519 <PUBKEY>" >&2
        exit 1
    fi
    # Obtener el principal de la primera línea del allowlist
    PRINCIPAL=$(awk '{print $1}' /tmp/allowed_keys)
    ssh-keygen -Y verify -f /tmp/allowed_keys -I "$PRINCIPAL" -n "$NAMESPACE" -s "$SIG_FILE" < "$MSG_FILE"
    echo "EXIT=$?"
    echo "✅ Si muestra 'Good signature', la autorización proviene de la máquina legítima de Cris."
}

case "${1:-}" in
    sign)   firmar ;;
    verify) verificar ;;
    pubkey) mostrar_pubkey ;;
    *)
        echo "Uso: $0 {sign|verify|pubkey}"
        exit 1
        ;;
esac
