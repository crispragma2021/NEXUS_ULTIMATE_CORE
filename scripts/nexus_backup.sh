#!/bin/bash

# --- NEXUS SOVEREIGN BACKUP ENGINE (Incremental) ---
# Este script realiza respaldos usando hardlinks para ahorrar espacio.
# Inspirado en la eficiencia de Git, pero optimizado para binarios de IA.

SOURCE="/home/soberano/NEXUS_ULTIMATE_CORE"
BACKUP_ROOT="/home/soberano/NEXUS_ULTIMATE_CORE/backups"
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
LATEST_LINK="$BACKUP_ROOT/latest"
CURRENT_BACKUP="$BACKUP_ROOT/$TIMESTAMP"

mkdir -p "$BACKUP_ROOT"

echo "⏳ NEXUS: Iniciando Respaldo Incremental [$TIMESTAMP]..."

# Si existe un respaldo anterior, lo usamos como base para hardlinks
RSYNC_OPTS="-av --delete --exclude='backups' --exclude='*.tar.gz' --exclude='target'"

if [ -L "$LATEST_LINK" ]; then
    echo "🔗 Usando respaldo previo como base (ahorrando espacio)..."
    rsync $RSYNC_OPTS --link-dest="$LATEST_LINK" "$SOURCE/" "$CURRENT_BACKUP"
else
    echo "🌑 Primer respaldo: Creando base completa..."
    rsync $RSYNC_OPTS "$SOURCE/" "$CURRENT_BACKUP"
fi

# Actualizar el enlace 'latest'
rm -f "$LATEST_LINK"
ln -s "$CURRENT_BACKUP" "$LATEST_LINK"

# Limpieza: Mantener solo los últimos 7 respaldos para proteger el disco
ls -1t "$BACKUP_ROOT" | grep -v "latest" | tail -n +8 | xargs -I {} rm -rf "$BACKUP_ROOT/{}"

echo "✅ Respaldo Completado: $CURRENT_BACKUP"
echo "📊 Espacio total usado en la carpeta de backups:"
du -sh "$BACKUP_ROOT"
