#!/bin/bash
# ==========================================
# MEMOCLAW: MOTOR DE PERSISTENCIA Y MEMORIA
# ==========================================
MEMORIA_DIR="$HOME/ZENITH_POOL/data"
MEMORIA_DB="$MEMORIA_DIR/nexus_memoclaw.sqlite"
LOG_ENJAMBRE="$MEMORIA_DIR/swarm_events.log"

echo "[NEXUS-CORE] Revelando estructuras ocultas de Antigravity..."
echo " -> [MEMOCLAW] Forjando lóbulo de memoria persistente..."

# 1. Crear el tejido de almacenamiento
mkdir -p "$MEMORIA_DIR"

# 2. Inicializar base de datos ligera (SQLite a través de CLI)
# Si SQLite no está, usa un archivo estructurado como respaldo provisional
if command -v sqlite3 >/dev/null 2>&1; then
    sqlite3 "$MEMORIA_DB" "CREATE TABLE IF NOT EXISTS recuerdos (id INTEGER PRIMARY KEY, timestamp TEXT, agente TEXT, evento TEXT);"
    echo " -> [MEMOCLAW] Motor SQLite activado en $MEMORIA_DB."
else
    touch "$MEMORIA_DB.json"
    echo " -> [MEMOCLAW] SQLite no detectado. Operando en modo JSON fallback."
fi

# 3. Crear el Bus de Eventos (Log centralizado para los agentes)
touch "$LOG_ENJAMBRE"
echo " -> [IPC] Bus de comunicación materializado en $LOG_ENJAMBRE."

echo "[NEXUS-CORE] El tejido conectivo está listo. El Orquestador y el Watchdog ahora comparten una memoria unificada."
