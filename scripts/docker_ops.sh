#!/bin/bash
# NEXUS SKILL: DOCKER OPS v1.0
# "El Ingeniero de Contenedores"

COMMAND=$1
TARGET=$2

case "$COMMAND" in
    "list")
        echo "🐳 Active Containers:"
        docker ps --format "table {{.ID}}\t{{.Names}}\t{{.Status}}\t{{.Ports}}"
        ;;
    "restart")
        if [ -z "$TARGET" ]; then
            echo "❌ Error: Specify container name or ID."
        else
            echo "🔄 Restarting $TARGET..."
            docker restart "$TARGET" && echo "✅ $TARGET restarted successfully."
        fi
        ;;
    "logs")
        if [ -z "$TARGET" ]; then
            echo "❌ Error: Specify container name or ID."
        else
            echo "📜 Last 20 lines of logs for $TARGET:"
            docker logs --tail 20 "$TARGET"
        fi
        ;;
    "nuke")
        echo "☢️  WARNING: Stopping ALL containers..."
        docker stop $(docker ps -q) 2>/dev/null
        echo "✅ System Nuked. Use 'docker ps' to verify silence."
        ;;
    *)
        echo "Usage: $0 {list|restart <id>|logs <id>|nuke}"
        ;;
esac
