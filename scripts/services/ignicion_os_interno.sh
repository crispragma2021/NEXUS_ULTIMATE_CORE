#!/bin/bash
# =====================================================================
# NEXUS OMEGA - IGNICIÓN DEL INTERNAL OS (FIRECRACKER)
# =====================================================================
# Propósito: Descargar y preparar el entorno de MicroVM para ÆGIS.
# =====================================================================

WORKSHOP="/home/soberano/NEXUS_ULTIMATE_CORE/workshop"
mkdir -p "$WORKSHOP"

echo "🚀 [IGNICIÓN] Preparando Internal OS (Alpine Minimalist)..."

# 1. Verificar KVM (Soberanía de Hardware)
if [ ! -e /dev/kvm ]; then
    echo "❌ ERROR: /dev/kvm no detectado. El i7-12700F requiere virtualización activa en BIOS."
    exit 1
fi

# 2. Descargar Kernel Firecracker (vmlinux)
if [ ! -f "$WORKSHOP/vmlinux" ]; then
    echo "🛰️  Descargando Micro-Kernel vmlinux..."
    curl -L https://s3.amazonaws.com/spec.ccfc.min/img/quickstart_guide/x86_64/kernels/vmlinux.bin -o "$WORKSHOP/vmlinux"
    chmod +x "$WORKSHOP/vmlinux"
fi

# 3. Descargar RootFS (Alpine Linux)
if [ ! -f "$WORKSHOP/rootfs.ext4" ]; then
    echo "📦 Descargando RootFS de Alpine Linux (Firecracker Ready)..."
    # Usamos una imagen ligera de Alpine para maximizar los núcleos E del i7
    curl -L https://s3.amazonaws.com/spec.ccfc.min/img/hello/fsfiles/hello-rootfs.ext4 -o "$WORKSHOP/rootfs.ext4"
fi

# 4. Descargar Binario Firecracker (Si no existe)
if ! command -v firecracker &> /dev/null; then
    echo "⚙️ Instalando binario Firecracker..."
    # Extracción más robusta de la URL de descarga del binario Firecracker usando PCRE grep
    release_url=$(curl -s https://api.github.com/repos/firecracker-microvm/firecracker/releases/latest | grep -oP '"browser_download_url": "\K[^"]+' | grep "x86_64.tgz")
    
    if [ -z "$release_url" ]; then
        echo "❌ ERROR: No se pudo obtener la URL de descarga de Firecracker. Verifique la conexión a internet o el patrón de búsqueda en GitHub API."
        exit 1
    fi
    
    # Descargar y extraer el binario Firecracker directamente a /usr/local/bin
    curl -L "$release_url" | sudo tar -xz --strip-components=1 -C /usr/local/bin/ firecracker-v*/firecracker
fi

echo "✅ [INTERNAL OS] Infraestructura lista en $WORKSHOP."
echo "📡 Propiocepción: El Dashboard ahora debería mostrar 'RootFS: Ready'."

# Notificar al Orquestador
systemctl --user restart nexus.service