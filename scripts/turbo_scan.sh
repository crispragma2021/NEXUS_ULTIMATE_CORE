#!/bin/bash
# NEXUS SKILL: TURBO SCAN v1.0
# "El Ojo de Red"

echo "📡 Scanning Local Network (Passive Mode)..."

# Use ARP table for fast, non-intrusive discovery
if command -v arp >/dev/null 2>&1; then
    arp -a | awk '{print "🖥️  Dev: " $1 "  IP: " $2 "  MAC: " $4}'
elif command -v ip >/dev/null 2>&1; then
    ip neigh | awk '{print "🖥️  IP: " $1 "  MAC: " $5 "  State: " $6}'
else
    echo "❌ Error: No network tools found (arp/ip)."
fi

echo "✅ Scan Complete."
