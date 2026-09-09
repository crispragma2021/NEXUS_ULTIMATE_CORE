#!/bin/bash
cd ~/NEXUS_ULTIMATE_CORE/core/src/bin/

# Hacer copia de seguridad del código del proxy
cp proxy_hijack.rs proxy_hijack.rs.bak2

# Nota técnica: Modificamos el código para que si el dominio contiene "google" o "googleapis", 
# el proxy actúe en modo Passthrough (Túnel TCP directo) sin re-firmar con tu CA.
echo "Aplicando parche de Bypass SSL para dominios de Google..."

# Este script asume que usas una lógica estándar de proxy en Rust. 
# Si prefieres compilar el proxy limpio después, primero haz la prueba con el proxy apagado.
