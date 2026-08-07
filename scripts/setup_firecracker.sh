#!/bin/bash

# NEXUS GHOST-VM - DESCARGA Y CONFIGURACIÓN DE RECURSOS
NEON_CYAN='\033[0;36m'
NEON_GREEN='\033[0;32m'
NEON_RED='\033[0;31m'
RESET='\033[0m'

echo -e "${NEON_CYAN}📥 Preparando directorios para Firecracker...${RESET}"
mkdir -p firecracker_env

# Descargar Firecracker v1.7.0 TGZ
if [ ! -f firecracker_env/firecracker ]; then
  echo -e "${NEON_CYAN}📥 Descargando y descomprimiendo binario de Firecracker v1.7.0...${RESET}"
  curl -L -o firecracker_env/firecracker.tgz https://github.com/firecracker-microvm/firecracker/releases/download/v1.7.0/firecracker-v1.7.0-x86_64.tgz
  tar -xzf firecracker_env/firecracker.tgz -C firecracker_env/
  mv firecracker_env/release-v1.7.0-x86_64/firecracker-v1.7.0-x86_64 firecracker_env/firecracker
  mv firecracker_env/release-v1.7.0-x86_64/jailer-v1.7.0-x86_64 firecracker_env/jailer
  rm -rf firecracker_env/firecracker.tgz firecracker_env/release-v1.7.0-x86_64
  chmod +x firecracker_env/firecracker firecracker_env/jailer
else
  echo -e "${NEON_GREEN}✓ Binario de Firecracker ya existe.${RESET}"
fi

# Descargar Kernel de prueba oficial
if [ ! -f firecracker_env/vmlinux.bin ]; then
  echo -e "${NEON_CYAN}📥 Descargando Kernel Linux minimalista...${RESET}"
  curl -L -o firecracker_env/vmlinux.bin https://s3.amazonaws.com/spec.ccfc.min/img/hello/kernel/hello-vmlinux.bin
else
  echo -e "${NEON_GREEN}✓ Kernel Linux ya existe.${RESET}"
fi

# Descargar RootFS de prueba oficial
if [ ! -f firecracker_env/hello-rootfs.ext4 ]; then
  echo -e "${NEON_CYAN}📥 Descargando sistema de archivos de prueba (RootFS)...${RESET}"
  curl -L -o firecracker_env/hello-rootfs.ext4 https://s3.amazonaws.com/spec.ccfc.min/img/hello/fsfiles/hello-rootfs.ext4
else
  echo -e "${NEON_GREEN}✓ RootFS ya existe.${RESET}"
fi

echo -e "${NEON_GREEN}🎉 Todos los recursos descargados con éxito en firecracker_env/${RESET}"
