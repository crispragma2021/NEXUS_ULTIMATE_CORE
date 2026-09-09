#!/bin/bash
# NEXUS EXECUTIVE PRODUCT: PERFORMANCE BOOST v1.1
# Generated autonomously by Shadow Executive
# Targets: dev_optimizer, memory_purge, nexus_autocura

echo "🚀 Applying Neural Performance Boost..."

# 1. Safer Process Killing (Selective Cleanup)
optimize_purge() {
    TARGETS="chrome-headless-shell type-renderer"
    # PROTECT Antigravity & IDE: Never kill node blindly anymore.
    # We only kill node if it refers to specific non-critical nexus components.
    PIDS=$(pgrep -f "$(echo $TARGETS | sed 's/ /|/g')")
    if [ -n "$PIDS" ] && ! pgrep -f "antigravity" >/dev/null; then
        echo "⚡ Safe Batch Terminating PIDs: $PIDS"
        kill -9 $PIDS 2>/dev/null
    fi
    
    # Selective Node Cleanup: Only non-critical ones
    pgrep -f "nexus_observer.py" | xargs kill -9 2>/dev/null
}

# 2. Optimized Alias Injection
inject_alias_safe() {
    local alias_name=$1
    local alias_cmd=$2
    if ! grep -q "alias $alias_name" "$HOME/.bashrc"; then
        echo "alias $alias_name='$alias_cmd'" >> "$HOME/.bashrc"
        echo "✅ Added alias: $alias_name"
    fi
}

# Apply optimizations
optimize_purge
inject_alias_safe "nstatus" "systemctl --user status nexus_sensor.timer"
inject_alias_safe "nlogs" "tail -n 50 nexus-interface.log"
# Dashboard NEXUS Chat - Visión Soberana
NEXUS_ROOT="$(cd -P "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
inject_alias_safe "nexus-db" "cd \"$NEXUS_ROOT\" && just omega"

# 3. Clean Corrupt Incremental Builds (Fast Wipe)
[ -d "NEXUS_INTERFACE/src-tauri/target/debug/incremental" ] && find NEXUS_INTERFACE/src-tauri/target/debug/incremental -mindepth 1 -delete

echo "✨ Performance boost applied successfully."
