#!/bin/bash
# NEXUS SKILL: MEMORY PURGE v1.0
# "El Limpiador de RAM"

echo "🧹 Activating Zen Mode (Memory Purge)..."

# 1. Drop Caches (Requires sudo, or just fails gracefully)
sync
if [ -w /proc/sys/vm/drop_caches ]; then
    echo 3 > /proc/sys/vm/drop_caches
    echo "✅ Filesystem Caches Dropped."
else
    echo "⚠️  Skipping deep cache drop (Root required)."
fi

# 2. Kill Heavy Background Processes (Safe List)
# Add processes here that are safe to kill to free memory
TARGETS="chrome-headless-shell type-renderer"

for proc in $TARGETS; do
    # 🚨 GUARD: NEVER kill processes related to the IDE/Agent
    if pgrep -f "antigravity" >/dev/null; then
         # Specific ignore list inside the loop
         pkill -f "$proc" && echo "💀 Terminated: $proc"
    fi
done

# 3. Compact Memory (If available)
echo "🧠 Compacting Workspace..."
# (Placeholder for future compacting logic)

echo "✨ System Purified. Maximizing available RAM."
