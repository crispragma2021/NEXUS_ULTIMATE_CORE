#!/bin/bash
# ==============================================================================
#                 🧬 NEXUS OMEGA PURGE & CONSOLIDATION SCRIPT
# ==============================================================================
# Objetivo: Limpiar y reestructurar /home/soberano/NEXUS_ULTIMATE_CORE para alcanzar la
# pureza soberana de la arquitectura. Elimina grasa, bases de datos obsoletas,
# logs sueltos y agrupa los scripts utilitarios en /scripts/.
# ==============================================================================

set -euo pipefail

PROJECT_DIR="/home/soberano/NEXUS_ULTIMATE_CORE"
SCRIPTS_DIR="$PROJECT_DIR/scripts"
LOGS_DIR="$PROJECT_DIR/logs"
ARCHIVE_DIR="$PROJECT_DIR/archive"
BACKUP_DIR="$PROJECT_DIR/backups"

echo "🛡️  Iniciando saneamiento sistémico NEXUS..."

# 1. Crear directorios fundamentales si no existen
mkdir -p "$SCRIPTS_DIR"
mkdir -p "$LOGS_DIR"
mkdir -p "$ARCHIVE_DIR"
mkdir -p "$BACKUP_DIR"

# 2. Respaldar y eliminar bases de datos corruptas / duplicadas
echo "🛢️  Saneando bases de datos redundantes..."
if [ -f "$PROJECT_DIR/nexus_intelligence.db.corrupted" ]; then
    echo "   -> Archivando nexus_intelligence.db.corrupted..."
    mv "$PROJECT_DIR/nexus_intelligence.db.corrupted" "$BACKUP_DIR/nexus_intelligence.db.corrupted.bak"
fi

if [ -f "$PROJECT_DIR/nexus_intelligence_repaired_v2.db" ]; then
    echo "   -> Archivando nexus_intelligence_repaired_v2.db redundante..."
    mv "$PROJECT_DIR/nexus_intelligence_repaired_v2.db" "$BACKUP_DIR/nexus_intelligence_repaired_v2.db.bak"
fi

if [ -f "$PROJECT_DIR/recovery.sql" ]; then
    echo "   -> Archivando recovery.sql..."
    mv "$PROJECT_DIR/recovery.sql" "$BACKUP_DIR/recovery.sql.bak"
fi

if [ -f "$PROJECT_DIR/init_zenith.sql" ]; then
    echo "   -> Conservando init_zenith.sql en backups..."
    mv "$PROJECT_DIR/init_zenith.sql" "$BACKUP_DIR/init_zenith.sql"
fi

# 3. Eliminar directorios de memoria duplicados / obsoletos
echo "🧠 Saneando directorios de conocimiento fragmentado..."
# Consolidar archivos de memories/ o memoria/ si tienen logros u otros a carpetas oficiales
if [ -d "$PROJECT_DIR/memories" ] && [ "$(ls -A "$PROJECT_DIR/memories" 2>/dev/null)" ]; then
    echo "   -> Respaldando contenido de 'memories' en backups..."
    cp -r "$PROJECT_DIR/memories/"* "$BACKUP_DIR/" || true
    rm -rf "$PROJECT_DIR/memories"
elif [ -d "$PROJECT_DIR/memories" ]; then
    rm -rf "$PROJECT_DIR/memories"
fi

if [ -d "$PROJECT_DIR/nexus_knowledge" ] && [ "$(ls -A "$PROJECT_DIR/nexus_knowledge" 2>/dev/null)" ]; then
    echo "   -> Respaldando nexus_knowledge..."
    cp -r "$PROJECT_DIR/nexus_knowledge/"* "$BACKUP_DIR/" || true
    rm -rf "$PROJECT_DIR/nexus_knowledge"
elif [ -d "$PROJECT_DIR/nexus_knowledge" ]; then
    rm -rf "$PROJECT_DIR/nexus_knowledge"
fi

# 4. Eliminar directorios target alternativos (Grasa masiva Rust)
echo "💾 Removiendo caché Rust redundante (target_alt, target_omega_v2)..."
if [ -d "$PROJECT_DIR/target_alt" ]; then
    echo "   -> Purgando target_alt..."
    rm -rf "$PROJECT_DIR/target_alt"
fi

if [ -d "$PROJECT_DIR/target_omega_v2" ]; then
    echo "   -> Purgando target_omega_v2..."
    rm -rf "$PROJECT_DIR/target_omega_v2"
fi

# 5. Archivar daemons heredados (.d)
echo "📁 Archivando daemons heredados (.d) en la ruta de legado..."
mkdir -p "$ARCHIVE_DIR/legacy_daemons.d"
for dir in "$PROJECT_DIR"/*.d; do
    if [ -d "$dir" ]; then
        dir_name=$(basename "$dir")
        echo "   -> Moviendo $dir_name a archive/legacy_daemons.d/..."
        mv "$dir" "$ARCHIVE_DIR/legacy_daemons.d/"
    fi
done

# 6. Mover logs sueltos de la raíz a su santuario central
echo "📜 Centralizando registros y logs en /logs/..."
for logfile in "$PROJECT_DIR"/*.log; do
    if [ -f "$logfile" ]; then
        log_name=$(basename "$logfile")
        # No mover logs/ si está ahí
        echo "   -> Relocalizando log: $log_name..."
        mv "$logfile" "$LOGS_DIR/"
    fi
done

# 7. Consolidar y relocalizar scripts utilitarios en /scripts/
echo "🐚 Consolidando scripts de control de procesos en /scripts/..."
SCRIPTS_TO_MOVE=(
    "nexus_purge.sh"
    "nexus_clean_purge.sh"
    "nexus_final_audit.sh"
    "nexus_android_audit.sh"
    "omega_build.sh"
    "strap.sh"
    "install_nerves.sh"
    "nexus-live.sh"
    "nexus_portal.sh"
    "nexus_start.sh"
    "mem_guard.sh"
    "nexus_chat.sh"
    "nexus_guardian.sh"
)

for script in "${SCRIPTS_TO_MOVE[@]}"; do
    if [ -f "$PROJECT_DIR/$script" ]; then
        echo "   -> Relocalizando script: $script..."
        mv "$PROJECT_DIR/$script" "$SCRIPTS_DIR/"
        chmod +x "$SCRIPTS_DIR/$script"
    fi
done

echo "✅  Saneamiento completado con éxito. El santuario digital está en orden."
