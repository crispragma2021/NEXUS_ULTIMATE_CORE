#!/bin/bash

# ==============================================================================
# NEXUS GHOST-VM - ORQUESTADOR NATIVO DE MICROVM (FIRECRACKER)
# ==============================================================================

NEON_CYAN='\033[0;36m'
NEON_GREEN='\033[0;32m'
NEON_RED='\033[0;31m'
NEON_YELLOW='\033[0;33m'
RESET='\033[0m'

echo -e "${NEON_CYAN}🤖 [NEXUS GHOST-VM] Inicializando MicroVM Amnésica...${RESET}"

# Directorio de trabajo y archivos temporales
FC_DIR="/home/soberano/NEXUS_ULTIMATE_CORE/firecracker_env"
SOCKET="/tmp/firecracker.socket"
CONFIG_FILE="/tmp/firecracker_config.json"
TMP_ROOTFS="/tmp/ghost-rootfs.ext4"

# 1. Limpieza de procesos y sockets previos
sudo rm -f "$SOCKET" "$CONFIG_FILE"
sudo pkill -9 firecracker

# 2. Configurar interfaz de red TAP en el host
echo -e "${NEON_CYAN}🔌 Configurando interfaz de red TAP (tap0)...${RESET}"
sudo ip link del tap0 >/dev/null 2>&1
sudo ip tuntap add dev tap0 mode tap
sudo ip addr add 172.16.0.1/24 dev tap0
sudo ip link set tap0 up

# Habilitar forwarding de IP en el host para enrutar el tráfico
echo 1 | sudo tee /proc/sys/net/ipv4/ip_forward >/dev/null

# 🔱 BLINDAJE TRANSPROXY DE NEXUS
# Limpiar reglas previas de redirección de tap0
sudo iptables -t nat -F PREROUTING 2>/dev/null || true
sudo iptables -F FORWARD 2>/dev/null || true

# Redirigir consultas DNS (puerto 53) de la interfaz de la VM (tap0) a DNSPort de Tor (5353)
sudo iptables -t nat -A PREROUTING -i tap0 -p udp --dport 53 -j REDIRECT --to-ports 5353
sudo iptables -t nat -A PREROUTING -i tap0 -p tcp --dport 53 -j REDIRECT --to-ports 5353

# Redirigir todo el tráfico TCP de la interfaz de la VM (tap0) a TransPort de Tor (9040)
sudo iptables -t nat -A PREROUTING -i tap0 -p tcp --syn -j REDIRECT --to-ports 9040

# 🛡️ KILL-SWITCH ABSOLUTO
# Permitir únicamente tráfico establecido/relacionado y reenvíos locales a los servicios del Host
sudo iptables -A FORWARD -i tap0 -o tap0 -j ACCEPT
sudo iptables -A FORWARD -m state --state ESTABLISHED,RELATED -j ACCEPT

# Bloquear cualquier reenvío directo a internet (Evita fugas si Tor se cae o si intentan conexiones directas)
sudo iptables -A FORWARD -i tap0 -j REJECT --reject-with icmp-port-unreachable
sudo iptables -A FORWARD -o tap0 -j REJECT --reject-with icmp-port-unreachable

# 3. Preparar RootFS Amnésico en RAM (/tmp)
echo -e "${NEON_CYAN}🧠 Copiando RootFS al disco RAM temporal (/tmp)...${RESET}"
rm -f "$TMP_ROOTFS"
cp "$FC_DIR/hello-rootfs.ext4" "$TMP_ROOTFS"

# 🔱 INYECCIÓN DE BLINDAJE ANTIMALWARE Y DNS SEGURO EN EL ROOTFS
echo -e "${NEON_CYAN}🛡️ Inyectando políticas DNS seguro y deshabilitando IPv6 en el RootFS...${RESET}"
MNT_DIR="/tmp/ghost_mnt"
sudo mkdir -p "$MNT_DIR"
sudo mount -o loop "$TMP_ROOTFS" "$MNT_DIR"

# Enjutar resolv.conf para forzar las consultas DNS a la IP del Host (Tor)
echo "nameserver 172.16.0.1" | sudo tee "$MNT_DIR/etc/resolv.conf" >/dev/null

# Desactivar IPv6 en el guest (evita fugas si el software de la VM intenta conexiones IPv6)
echo -e "net.ipv6.conf.all.disable_ipv6 = 1\nnet.ipv6.conf.default.disable_ipv6 = 1" | sudo tee -a "$MNT_DIR/etc/sysctl.conf" >/dev/null

# Desmontar imagen limpia
sudo umount "$MNT_DIR"
sudo rmdir "$MNT_DIR"

# 4. Generar archivo de configuración JSON para el arranque inmediato
echo -e "${NEON_CYAN}⚙️ Generando archivo de configuración para Firecracker...${RESET}"
cat <<EOF > "$CONFIG_FILE"
{
  "boot-source": {
    "kernel_image_path": "$FC_DIR/vmlinux.bin",
    "boot_args": "console=ttyS0 reboot=k panic=1 pci=off ip=172.16.0.2::172.16.0.1:255.255.255.0:hs:eth0:off"
  },
  "drives": [
    {
      "drive_id": "rootfs",
      "path_on_host": "$TMP_ROOTFS",
      "is_root_device": true,
      "is_read_only": false
    }
  ],
  "network-interfaces": [
    {
      "iface_id": "eth0",
      "host_dev_name": "tap0"
    }
  ]
}
EOF

# 5. Iniciar la MicroVM en el primer plano con la configuración
echo -e "${NEON_GREEN}🏁 Iniciando la ejecución de la MicroVM...${RESET}"
echo -e "${NEON_YELLOW}Para salir del terminal y apagar la VM: Escribe 'reboot' en la VM o presiona Ctrl+C en esta terminal.${RESET}\n"

# Arrancamos firecracker directamente con sudo para evitar problemas de permisos con /dev/kvm
sudo "$FC_DIR/firecracker" --api-sock "$SOCKET" --config-file "$CONFIG_FILE"

# 6. Destrucción post-ejecución (Homeostasis de NEXUS)
echo -e "\n${NEON_YELLOW}🧹 Destruyendo entorno de forma amnésica...${RESET}"
sudo rm -f "$SOCKET" "$CONFIG_FILE" "$TMP_ROOTFS"
sudo ip link del tap0 >/dev/null 2>&1
echo -e "${NEON_GREEN}✨ Entorno destruido. RAM liberada. Cero registros en disco.${RESET}"
