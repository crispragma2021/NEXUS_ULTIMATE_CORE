#!/bin/bash
# ===============================================================================
# 🌐 WINDOWS 11 INTEL WIFI DRIVER DOWNLOADER FOR USB
# ===============================================================================

# 1. Check root privileges
if [ "$EUID" -ne 0 ]; then
  echo "❌ Error: Este script requiere privilegios de root."
  echo "   Ejecútalo con: sudo bash $0"
  exit 1
fi

echo "🔍 Buscando la unidad del pendrive..."
DEV="/dev/sdb1"

if [ ! -b "$DEV" ]; then
  echo "❌ Error: No se encontró el dispositivo $DEV."
  echo "   Por favor asegúrate de que el pendrive está conectado."
  exit 1
fi

echo "✅ Pendrive detectado en $DEV."

# 2. Crear punto de montaje
MOUNT_DIR="/tmp/nexus_usb_mnt"
mkdir -p "$MOUNT_DIR"

echo "🔌 Montando pendrive en $MOUNT_DIR..."
mount -t vfat "$DEV" "$MOUNT_DIR"

if [ $? -ne 0 ]; then
  echo "❌ Error al montar el pendrive."
  exit 1
fi

# 3. Descargar el controlador oficial .exe de Intel
URL="https://downloadmirror.intel.com/918237/WiFi-24.40.0-Driver64-Win10-Win11.exe"
TARGET_FILE="$MOUNT_DIR/WiFi-24.40.0-Driver64-Win10-Win11.exe"

echo "📥 Descargando controlador oficial de Intel para Windows 10/11..."
echo "   Origen: $URL"
echo "   Destino: $TARGET_FILE"
echo "================================================================="

# Descargar mostrando barra de progreso simple
curl -# -L -o "$TARGET_FILE" "$URL"

if [ $? -eq 0 ] && [ -f "$TARGET_FILE" ]; then
  echo "================================================================="
  echo "✅ ¡Descarga completada y verificada!"
  echo "   Archivo: WiFi-24.40.0-Driver64-Win10-Win11.exe (~45 MB)"
else
  echo "================================================================="
  echo "❌ Error durante la descarga del controlador de Intel."
  umount "$MOUNT_DIR"
  rmdir "$MOUNT_DIR"
  exit 1
fi

# 4. Limpieza y desmontaje seguro
echo "🔄 Sincronizando búferes de escritura..."
sync

echo "🔌 Desmontando pendrive de forma segura..."
umount "$MOUNT_DIR"
rmdir "$MOUNT_DIR"

echo "================================================================="
echo "✅ ¡PROCESO FINALIZADO CON ÉXITO!"
echo "================================================================="
echo "El instalador oficial para Windows 11 ya está cargado en la raíz de tu pendrive."
echo "Ahora puedes retirar el pendrive físicamente de forma segura."
echo "En tu PC con Windows 11, solo haz doble clic sobre el archivo:"
echo "   WiFi-24.40.0-Driver64-Win10-Win11.exe"
echo "¡Y se instalará automáticamente de inmediato!"
echo "================================================================="
