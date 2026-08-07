#!/bin/bash

# Este script limpia metadatos básicos de archivos, si es posible.
# Requiere 'exiftool' o 'mat2' para una limpieza profunda, pero
# intentará una limpieza básica en archivos de texto si no están disponibles.

if [ -z "$1" ]; then
  echo "Uso: $0 <ruta_al_archivo>"
  exit 1
fi

FILE_PATH="$1"

if [ ! -f "$FILE_PATH" ]; then
  echo "Error: Archivo no encontrado en '$FILE_PATH'"
  exit 1
fi

echo "Limpiando metadatos de: $FILE_PATH"

# Intentar usar mat2 si está disponible
if command -v mat2 &> /dev/null; then
  echo "Usando mat2 para limpiar metadatos."
  mat2 "$FILE_PATH"
  echo "Mat2: Metadatos limpiados."
  exit 0
fi

# Intentar usar exiftool si está disponible
if command -v exiftool &> /dev/null; then
  echo "Usando exiftool para limpiar metadatos."
  exiftool -all= "$FILE_PATH"
  echo "Exiftool: Metadatos limpiados."
  exit 0
fi

# Limpieza básica para archivos de texto (eliminar líneas comunes de metadatos)
if file --mime-type "$FILE_PATH" | grep -q "text/"; then
  echo "Limpieza básica de metadatos para archivo de texto."
  # Eliminar líneas que contengan "Created by", "Author", "Generator", "Date", etc.
  # Esta es una implementación muy básica y puede no ser exhaustiva.
  sed -i '/Created by/d' "$FILE_PATH"
  sed -i '/Author/d' "$FILE_PATH"
  sed -i '/Generator/d' "$FILE_PATH"
  sed -i '/Date:/d' "$FILE_PATH"
  echo "Limpieza básica: Metadatos de texto eliminados."
else
  echo "No se pudo realizar limpieza de metadatos. 'mat2' o 'exiftool' no encontrados, y no es un archivo de texto simple."
fi
