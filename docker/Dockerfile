# 🔱 NEXUS OMEGA - Dockerfile de Dispositivo Efímero Desechable
FROM debian:bookworm-slim

# Evitar diálogos interactivos
ENV DEBIAN_FRONTEND=noninteractive

# Instalar dependencias esenciales, navegador Chromium y Xvfb para emulación de pantalla
RUN apt-get update && apt-get install -y --no-install-recommends \
    chromium \
    chromium-driver \
    xvfb \
    x11vnc \
    fluxbox \
    dbus-x11 \
    ca-certificates \
    curl \
    python3 \
    python3-pip \
    nodejs \
    npm \
    sudo \
    && rm -rf /var/lib/apt/lists/*

# Crear usuario no privilegiado para emular comportamiento humano real
RUN useradd -m -s /bin/bash soberano_anon \
    && echo "soberano_anon ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

WORKDIR /home/soberano_anon
USER soberano_anon

# Configurar variables de simulación de pantalla
ENV DISPLAY=:99
ENV RESOLUTION=1280x1024x24

# Copiar el script de inicio
COPY --chown=soberano_anon:soberano_anon entrypoint.sh ./entrypoint.sh
RUN chmod +x ./entrypoint.sh

ENTRYPOINT ["./entrypoint.sh"]
