#!/bin/bash
# =====================================================================
# NEXUS OMEGA - MANUAL IGNITION SYSTEM
# =====================================================================

SOCKET="/tmp/nexus_internal_os.sock"
WORKSHOP="/home/soberano/NEXUS_ULTIMATE_CORE/workshop"
FIRECRACKER="/home/soberano/NEXUS_ULTIMATE_CORE/bin/firecracker"

# Limpieza inicial
rm -f "$SOCKET"

echo "🔥 Iniciando proceso Firecracker..."
"$FIRECRACKER" --api-sock "$SOCKET" > /home/soberano/NEXUS_ULTIMATE_CORE/logs/firecracker.log 2>&1 &
FC_PID=$!

sleep 2

if [ ! -S "$SOCKET" ]; then
    echo "❌ Error: El socket no se creó. Revisa logs/firecracker.log"
    exit 1
fi

echo "✅ Socket detectado (PID: $FC_PID). Configurando recursos..."

# 1. Configurar Kernel
curl --unix-socket "$SOCKET" -i \
  -X PUT 'http://localhost/boot-source' \
  -H 'Accept: application/json' \
  -H 'Content-Type: application/json' \
  -d "{
        \"kernel_image_path\": \"$WORKSHOP/vmlinux\",
        \"boot_args\": \"console=ttyS0 reboot=k panic=1 pci=off\"
    }"

# 2. Configurar RootFS
curl --unix-socket "$SOCKET" -i \
  -X PUT 'http://localhost/drives/rootfs' \
  -H 'Accept: application/json' \
  -H 'Content-Type: application/json' \
  -d "{
        \"drive_id\": \"rootfs\",
        \"path_on_host\": \"$WORKSHOP/rootfs.ext4\",
        \"is_root_device\": true,
        \"is_read_only\": false
    }"

# 3. Ignición (InstanceStart)
echo "🚀 Disparando InstanceStart..."
curl --unix-socket "$SOCKET" -i \
  -X PUT 'http://localhost/actions' \
  -H 'Accept: application/json' \
  -H 'Content-Type: application/json' \
  -d '{
        "action_type": "InstanceStart"
    }'

echo "✨ Micro-VM OMEGA en ejecución."
echo "PID: $FC_PID"
echo "Socket: $SOCKET"
