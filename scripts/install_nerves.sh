#!/bin/bash
set -e
echo "1. Instalando Sentidos y Tacto (QEMU Agent, XFCE, Xorg)..."
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y qemu-guest-agent xfce4 xfce4-terminal xorg lightdm

echo "2. Activando Nervios Motores..."
systemctl enable --now qemu-guest-agent
systemctl enable lightdm || true
systemctl start lightdm || true

echo "3. Trasplantando Corazón Soberano (nexus.service)..."
echo "[Unit]
Description=NEXUS OMEGA Orquestador
After=network.target qemu-guest-agent.service lightdm.service

[Service]
Type=simple
User=root
WorkingDirectory=/root/NEXUS_ULTIMATE_CORE
ExecStart=/root/NEXUS_ULTIMATE_CORE/target/release/nexus-orquestador
Restart=always
RestartSec=5
Environment=\"RUST_LOG=info\"

[Install]
WantedBy=multi-user.target" > /etc/systemd/system/nexus.service

systemctl daemon-reload
systemctl enable nexus
systemctl start nexus

echo "EL ORGANISMO ESTÁ VIVO. EL PUERTO 43211 HA SIDO INYECTADO EN SYSTEMD."
systemctl status nexus --no-pager
