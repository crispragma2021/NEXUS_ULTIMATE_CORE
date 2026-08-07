#!/bin/bash
# 🔥 GHOST-VM IGNITION SCRIPT (NEXUS INTERNAL WORLD)
# This script prepares and boots a Firecracker MicroVM for code validation.

BASE_DIR="/home/soberano/NEXUS_ULTIMATE_CORE/firecracker_env"
SOCKET="/tmp/firecracker.socket"
ROOTFS="/tmp/ghost-rootfs.ext4"
KERNEL="$BASE_DIR/vmlinux.bin"
BIN="$BASE_DIR/firecracker"

echo "🔱 [IGNITION] Initializing Silicio Sandbox..."

# 1. Cleanup
rm -f $SOCKET
rm -f $ROOTFS

# 2. Prepare RAM RootFS
cp $BASE_DIR/hello-rootfs.ext4 $ROOTFS

# 3. Start Firecracker in Background
$BIN --api-sock $SOCKET > /dev/null 2>&1 &
sleep 1

# 4. Configure via API (CURL)
curl --unix-socket $SOCKET -X PUT \
  "http://localhost/boot-source" \
  -H "Content-Type: application/json" \
  -d "{
    \"kernel_image_path\": \"$KERNEL\",
    \"boot_args\": \"console=ttyS0 reboot=k panic=1 pci=off\"
  }"

curl --unix-socket $SOCKET -X PUT \
  "http://localhost/drives/rootfs" \
  -H "Content-Type: application/json" \
  -d "{
    \"drive_id\": \"rootfs\",
    \"path_on_host\": \"$ROOTFS\",
    \"is_root_device\": true,
    \"is_read_only\": false
  }"

# 5. Launch
curl --unix-socket $SOCKET -X PUT \
  "http://localhost/actions" \
  -H "Content-Type: application/json" \
  -d "{ \"action_type\": \"InstanceStart\" }"

echo "🚀 [IGNITION] MicroVM is BOOTING. Silicio validation ready."
