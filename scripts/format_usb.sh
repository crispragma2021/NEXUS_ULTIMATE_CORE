#!/bin/bash
# ===============================================================================
# 🛠️ INTEL WIFI USB LOADER (FAT32 FOR WINDOWS/LINUX)
# ===============================================================================

# 1. Check root privileges
if [ "$EUID" -ne 0 ]; then
  echo "❌ Error: Este script requiere privilegios de root."
  echo "   Ejecútalo con: sudo bash $0"
  exit 1
fi

echo "🔍 Buscando la unidad del pendrive..."
# Detectar dispositivo
DEV="/dev/sdb1"
DISK="/dev/sdb"

if [ ! -b "$DEV" ]; then
  echo "❌ Error: No se encontró el dispositivo $DEV."
  echo "   Por favor asegúrate de que el pendrive está conectado."
  exit 1
fi

echo "✅ Pendrive detectado en $DEV ($DISK)."

# 2. Confirmación final de seguridad
echo ""
echo "⚠️  ADVERTENCIA: Esto borrará COMPLETAMENTE el pendrive $DEV."
echo "   ¿Estás seguro de que deseas proceder? (escribe 'si' para confirmar):"
read -r CONFIRM
if [ "$CONFIRM" != "si" ]; then
  echo "❌ Operación cancelada por el usuario."
  exit 0
fi

# 3. Desmontar todas las instancias activas del pendrive
echo "🔄 Desmontando $DEV..."
umount -f /run/media/nexus/NEXUS_DRIVE* 2>/dev/null
umount -f /run/media/nexus/WIFI_DRIVE* 2>/dev/null
umount -f /dev/sdb* 2>/dev/null

# 4. Formatear la partición a FAT32 (compatible con Windows y Linux)
echo "🧹 Formateando $DEV a FAT32..."
if ! command -v mkfs.vfat &> /dev/null; then
  echo "📦 Instalando dosfstools para soporte FAT32..."
  pacman -S --noconfirm dosfstools
fi

# Formatear
mkfs.vfat -F 32 -n "WIFI_DRIVE" "$DEV"
if [ $? -ne 0 ]; then
  echo "❌ Error al formatear la unidad a FAT32."
  exit 1
fi
echo "✅ Pendrive formateado con éxito como FAT32 (Etiqueta: WIFI_DRIVE)."

# 5. Montar temporalmente para copiar los drivers
MOUNT_DIR="/tmp/nexus_usb_mnt"
mkdir -p "$MOUNT_DIR"
echo "🔌 Montando pendrive en $MOUNT_DIR..."
mount -t vfat "$DEV" "$MOUNT_DIR"

if [ $? -ne 0 ]; then
  echo "❌ Error al montar el pendrive."
  exit 1
fi

# 6. Crear estructura de carpetas
echo "📁 Creando estructura de directorios..."
mkdir -p "$MOUNT_DIR/intel_wifi_drivers"

# 7. Copiar firmwares de Wi-Fi de Intel
echo "📦 Copiando controladores de Wi-Fi Intel AX201/AX211 desde tu sistema..."
if [ -d "/usr/lib/firmware" ]; then
  # Copiar archivos iwlwifi-so-a0 y iwlwifi-ty-a0 (AX201 / AX211)
  cp /usr/lib/firmware/iwlwifi-so-a0-* "$MOUNT_DIR/intel_wifi_drivers/" 2>/dev/null
  cp /usr/lib/firmware/iwlwifi-ty-a0-* "$MOUNT_DIR/intel_wifi_drivers/" 2>/dev/null
  cp /usr/lib/firmware/iwlwifi-Qu-* "$MOUNT_DIR/intel_wifi_drivers/" 2>/dev/null
  cp /usr/lib/firmware/iwlwifi-QuZ-* "$MOUNT_DIR/intel_wifi_drivers/" 2>/dev/null
  # Copiar otros archivos iwlwifi generales por seguridad
  cp /usr/lib/firmware/iwlwifi-* "$MOUNT_DIR/intel_wifi_drivers/" 2>/dev/null
  echo "✅ Controladores copiados con éxito."
else
  echo "⚠️ Advertencia: No se encontró la carpeta /usr/lib/firmware en el host."
fi

# 8. Escribir script de instalación automatizado dentro del pendrive
echo "📝 Creando script de instalación 'instalar_wifi.sh' dentro del pendrive..."
cat << 'EOF' > "$MOUNT_DIR/instalar_wifi.sh"
#!/bin/bash
# ===============================================================================
# 📡 INSTALADOR OFFLINE DE WI-FI INTEL
# ===============================================================================

if [ "$EUID" -ne 0 ]; then
  echo "❌ Error: Ejecuta este script con sudo:"
  echo "   sudo bash instalar_wifi.sh"
  exit 1
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
DRIVERS_DIR="$SCRIPT_DIR/intel_wifi_drivers"
TARGET_DIR="/usr/lib/firmware"

echo "🚀 Copiando firmwares a $TARGET_DIR..."
cp "$DRIVERS_DIR"/iwlwifi-* "$TARGET_DIR"/

echo "🔄 Recargando el módulo del Kernel (iwlwifi)..."
modprobe -r iwlwifi 2>/dev/null
modprobe iwlwifi 2>/dev/null

echo "================================================================="
# Intento de verificar el estado de la interfaz
WIFI_DEV=$(ip link | grep -oE "wlan[0-9]|wlp[0-9]s[0-9]")
if [ -n "$WIFI_DEV" ]; then
  echo "✅ ¡Wi-Fi Activado! Dispositivo detectado: $WIFI_DEV"
  ip link set "$WIFI_DEV" up 2>/dev/null
else
  echo "⚠️ Firmware copiado. Si no se activó el Wi-Fi de inmediato,"
  echo "   por favor reinicia tu PC físicamente."
fi
echo "================================================================="
EOF

chmod +x "$MOUNT_DIR/instalar_wifi.sh"

# 9. Crear archivo de instrucciones
echo "📝 Creando archivo de instrucciones 'INSTRUCCIONES.txt'..."
cat << 'EOF' > "$MOUNT_DIR/INSTRUCCIONES.txt"
===============================================================================
📡 INSTRUCCIONES DE INSTALACIÓN OFF-LINE DE WI-FI INTEL (GENERACIÓN 12)
===============================================================================

Este pendrive ha sido formateado en FAT32 (compatible con Windows y Linux) y 
cargado únicamente con los controladores oficiales de Wi-Fi de Intel (iwlwifi) 
para tu PC.

Para activar el Wi-Fi en tu PC con Linux sin conexión a internet, sigue estos pasos:

1. Conecta el pendrive en la PC objetivo.
2. Abre una terminal dentro del pendrive.
3. Ejecuta el script de instalación automática:
   
   sudo bash instalar_wifi.sh

4. El script copiará los firmwares a la carpeta del sistema (/usr/lib/firmware)
   y reactivará el controlador de Wi-Fi en tiempo real.
5. ¡Listo! Tu red Wi-Fi debería encenderse de inmediato y estar lista para conectarse.

Nota: Si por algún motivo la interfaz no enciende sola, simplemente reinicia la PC.
===============================================================================
EOF

# 10. Limpieza y desmontaje seguro
echo "🔄 Sincronizando búferes de escritura..."
sync

echo "🔌 Desmontando pendrive..."
umount "$MOUNT_DIR"
rmdir "$MOUNT_DIR"

echo "================================================================="
echo "✅ ¡PENDRIVE PREPARADO CON ÉXITO!"
echo "================================================================="
echo "El pendrive ahora es FAT32 y contiene ÚNICAMENTE los controladores de Wi-Fi."
echo "Puedes retirarlo de forma segura."
echo "================================================================="
