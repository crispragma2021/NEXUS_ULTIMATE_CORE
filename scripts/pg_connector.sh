#!/bin/bash
# NEXUS_SKILL: PostgreSQL Connector
# Connects to Postgres databases (Native or Docker fallback)

ACTION=$1
DB_URL=$2
QUERY=$3

# Load env variables
if [ -f ../.env ]; then
    export $(grep -v '^#' ../.env | xargs)
fi

# Fallback to Docker if psql is missing
USE_DOCKER=0
if ! command -v psql &> /dev/null; then
    USE_DOCKER=1
fi

execute_query() {
    local url=$1
    local sql=$2
    
    if [ "$USE_DOCKER" -eq 1 ]; then
        # Run ephemeral postgres container just for psql client
        # Requires network access to host if DB is on host (use --network host)
        docker run --rm --network host -e PGPASSWORD=$PGPASSWORD postgres:alpine psql "$url" -c "$sql"
    else
        psql "$url" -c "$sql"
    fi
}

case "$ACTION" in
    "query")
        echo "🐘 Executing Query on $DB_URL..."
        execute_query "$DB_URL" "$QUERY"
        ;;
    "list_tables")
        echo "🐘 Listing Tables..."
        execute_query "$DB_URL" "\dt"
        ;;
    *)
        echo "Usage: $0 {query <db_url> <sql>|list_tables <db_url>}"
        echo "Note: If psql is missing, Docker will be used."
        exit 1
        ;;
esac
