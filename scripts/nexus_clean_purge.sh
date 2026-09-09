#!/data/data/com.termux/files/usr/bin/bash
echo "Iniciando purga de residuos en Poco X5 Pro..."
# Eliminar todos los APKs descargados que son focos de infección
rm -f /sdcard/Download/*.apk
# Limpiar caché de Termux y archivos temporales
rm -rf $TMPDIR/*
echo "Limpieza de archivos completada."
echo "-----------------------------------"
echo "CONSEJO TÉCNICO: Reinicia el teléfono ahora para vaciar la memoria Swap."
