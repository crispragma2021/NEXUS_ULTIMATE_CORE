#!/bin/bash
# =====================================================================
# NEXUS OMEGA - UNIFICACIÓN DE ALMACENAMIENTO DE OLLAMA
# =====================================================================
# Propósito: Mover la carpeta pesada de modelos de Ollama a la partición 
#            de almacenamiento (/mnt/almacenamiento) y crear un enlace 
#            simbólico para liberar espacio en el disco raíz (/).
# =====================================================================

# Verificar que se ejecuta como root
if [ "$EUID" -ne 0 ]; then
  echo "❌ ERROR: Por favor, ejecuta este script como root o usando sudo:"
  echo "   sudo bash $0"
  exit 1
fi

ORIGEN="/usr/share/ollama/.ollama"
DESTINO="/mnt/almacenamiento/ollama_data"

echo "⏳ 1. Deteniendo el servicio de Ollama..."
systemctl stop ollama

if [ -d "$ORIGEN" ]; then
  echo "📦 2. Moviendo datos de Ollama ($ORIGEN) a partición grande ($DESTINO)..."
  mkdir -p "$DESTINO"
  # Mapear y mover el contenido de la carpeta de Ollama
  cp -rp "$ORIGEN/." "$DESTINO/"
  
  echo "🧹 3. Respaldando y removiendo directorio original..."
  mv "$ORIGEN" "${ORIGEN}.bak"
  
  echo "🔗 4. Creando enlace simbólico unificado..."
  ln -s "$DESTINO" "$ORIGEN"
  
  # Asegurar permisos correctos para el usuario ollama
  chown -h ollama:ollama "$ORIGEN"
  chown -R ollama:ollama "$DESTINO"
  
  echo "🔄 5. Reiniciando servicio de Ollama..."
  systemctl start ollama
  
  echo "✅ PROCESO COMPLETADO EXITOSAMENTE."
  echo "💾 Los modelos ahora se almacenan en /mnt/almacenamiento, liberando espacio en tu disco raíz (/). Puedes borrar el respaldo antiguo con: rm -rf ${ORIGEN}.bak"
else
  echo "❌ ERROR: No se encontró la ruta por defecto de Ollama en $ORIGEN."
  echo "Intentando buscar como usuario regular..."
  
  # Fallback si se instaló a nivel de usuario en ~/.ollama
  USER_ORIGEN="/home/soberano/.ollama"
  if [ -d "$USER_ORIGEN" ]; then
     echo "📦 Detectada instalación de usuario en $USER_ORIGEN. Moviendo..."
     mkdir -p "$DESTINO"
     cp -rp "$USER_ORIGEN/." "$DESTINO/"
     mv "$USER_ORIGEN" "${USER_ORIGEN}.bak"
     ln -s "$DESTINO" "$USER_ORIGEN"
     chown -R soberano:soberano "$DESTINO"
     systemctl start ollama
     echo "✅ PROCESO DE USUARIO COMPLETADO EXITOSAMENTE."
  else
     echo "❌ ERROR: No se pudo localizar la carpeta de datos de Ollama."
  fi
fi
