#!/bin/bash
# 🧬 NEXUS USER SERVICES INSTALLER FOR NIXOS
# Deploys systemd units as user-level services under ~/.config/systemd/user/

USER_SYSTEMD_DIR="$HOME/.config/systemd/user"
mkdir -p "$USER_SYSTEMD_DIR"

# Dynamically resolve the absolute repository root
REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

echo "⚙️ Deploying user-level services..."
for service in systemd/*.service systemd/*.timer; do
    name=$(basename "$service")
    # Clean the service files of User= lines because user services must not contain User= settings.
    # Also substitute the hardcoded repository path with the dynamic REPO_ROOT.
    sed -E '/^(User|Group)=/d' "$service" | sed "s|/home/soberano/NEXUS_ULTIMATE_CORE|$REPO_ROOT|g" | sed "s|/home/soberano/NEXUS_ULTIMATE_CORE|$REPO_ROOT|g" > "$USER_SYSTEMD_DIR/$name"
    echo "  -> Deployed $name (user-optimized to $REPO_ROOT)"
done

# Copy any root nexus.service if it exists
[ -f nexus.service ] && sed -E '/^(User|Group)=/d' nexus.service | sed "s|/home/soberano/NEXUS_ULTIMATE_CORE|$REPO_ROOT|g" | sed "s|/home/soberano/NEXUS_ULTIMATE_CORE|$REPO_ROOT|g" > "$USER_SYSTEMD_DIR/nexus.service" || true

echo "🔄 Reloading systemd user daemon..."
systemctl --user daemon-reload

echo "🚀 Enabling and starting timers and core services..."
systemctl --user enable nexus-watchdog.timer nexus_sensor.timer nexus.service nexus-proxy.service nexus-dashboard.service 2>/dev/null
systemctl --user start nexus-watchdog.timer nexus_sensor.timer nexus.service nexus-proxy.service nexus-dashboard.service 2>/dev/null

echo "🟢 Installation complete! Check status with 'systemctl --user status nexus.service nexus-proxy.service'"
