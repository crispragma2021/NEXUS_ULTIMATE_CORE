#!/bin/bash
# ==========================================
# 🔱 UNIFICACIÓN TOTAL DE MEMORIA — NEXUS OMEGA
# ==========================================
# Migra los ~25 registros restantes de nexus_intelligence.legacy.db
# y datos de hipocampo.db/pulso.db a nexus_memoria.db
# Luego elimina archivos residuales.
# ==========================================

set -euo pipefail
cd "$(dirname "$0")/.."
NEXUS="data/nexus_memoria.db"
LEGACY="data/nexus_intelligence.legacy.db"
BACKUP_DIR="data/legacy"
LOG="scripts/unificar_memoria_completa.log"

echo "🔱 UNIFICACIÓN TOTAL DE MEMORIA — $(date)" | tee "$LOG"

# ── 0. Backup de archivos a eliminar ──
echo "📦 Creando backups en $BACKUP_DIR ..." | tee -a "$LOG"
mkdir -p "$BACKUP_DIR"
for f in data/intelligence.db data/pulso.db data/hipocampo.db data/memoria_episodica.db data/ocean.db data/lancedb; do
    if [ -e "$f" ]; then
        cp -r "$f" "$BACKUP_DIR/$(basename $f).bak" 2>/dev/null && echo "  ✅ Backup: $f" || true
    fi
done

# ── 1. Migrar unified_history → historial_unificado ──
echo "" | tee -a "$LOG"
echo "📋 1. Migrando unified_history (3 registros) → historial_unificado ..." | tee -a "$LOG"
sqlite3 "$NEXUS" <<'SQL'
ATTACH DATABASE 'data/nexus_intelligence.legacy.db' AS legacy;
INSERT OR IGNORE INTO historial_unificado (sesion_id, alias, prompt, respuesta, metadata, timestamp)
SELECT 'legacy_' || id, alias, prompt, response, COALESCE(metadata, ''), timestamp
FROM legacy.unified_history;
DETACH DATABASE legacy;
SQL

# ── 2. Migrar lessons_sovereign → memoria_semantica ──
echo "📋 2. Migrando lessons_sovereign (2 registros) → memoria_semantica ..." | tee -a "$LOG"
sqlite3 "$NEXUS" <<'SQL'
ATTACH DATABASE 'data/nexus_intelligence.legacy.db' AS legacy;
INSERT OR IGNORE INTO memoria_semantica (titulo, contenido, prioridad, timestamp)
SELECT title, content, priority, created_at
FROM legacy.lessons_sovereign;
DETACH DATABASE legacy;
SQL

# ── 3. Migrar system_config → config_sistema ──
echo "📋 3. Migrando system_config (12 registros) → config_sistema ..." | tee -a "$LOG"
sqlite3 "$NEXUS" <<'SQL'
ATTACH DATABASE 'data/nexus_intelligence.legacy.db' AS legacy;
INSERT OR IGNORE INTO config_sistema (clave, valor)
SELECT clave, valor
FROM legacy.system_config
WHERE clave NOT IN (SELECT clave FROM config_sistema);
DETACH DATABASE legacy;
SQL

# ── 4. Migrar tool_experience (7 registros) → memoria_episodica ──
echo "📋 4. Migrando tool_experience (7 registros) → memoria_episodica ..." | tee -a "$LOG"
sqlite3 "$NEXUS" <<'SQL'
ATTACH DATABASE 'data/nexus_intelligence.legacy.db' AS legacy;
INSERT OR IGNORE INTO memoria_episodica (titulo, contenido, timestamp)
SELECT tool_name || ' [' || status || ']', details, COALESCE(timestamp, datetime('now'))
FROM legacy.tool_experience;
DETACH DATABASE legacy;
SQL

# ── 5. Migrar memorias (1 registro) → memoria_episodica ──
echo "📋 5. Migrando memorias (1 registro) → memoria_episodica ..." | tee -a "$LOG"
sqlite3 "$NEXUS" <<'SQL'
ATTACH DATABASE 'data/nexus_intelligence.legacy.db' AS legacy;
INSERT OR IGNORE INTO memoria_episodica (titulo, contenido, timestamp)
SELECT 'Legado: ' || substr(contenido, 1, 80), contenido, COALESCE(timestamp, datetime('now'))
FROM legacy.memorias;
DETACH DATABASE legacy;
SQL

# ── 6. Migrar hipocampo.db (2 registros) → memoria_episodica ──
echo "📋 6. Migrando hipocampo.db (2 registros) → memoria_episodica ..." | tee -a "$LOG"
sqlite3 "$NEXUS" <<'SQL'
ATTACH DATABASE 'data/hipocampo.db' AS hipo;
INSERT OR IGNORE INTO memoria_episodica (titulo, contenido, timestamp)
SELECT 'Hipocampo: ' || substr(contenido, 1, 80), contenido, COALESCE(timestamp, datetime('now'))
FROM hipo.memorias;
DETACH DATABASE hipo;
SQL

# ── 7. Migrar pulso.db sesiones → sesiones ──
echo "📋 7. Migrando pulso.db sesiones (7 registros) → sesiones ..." | tee -a "$LOG"
sqlite3 "$NEXUS" <<'SQL'
ATTACH DATABASE 'data/pulso.db' AS pulso;
INSERT OR IGNORE INTO sesiones (id, timestamp)
SELECT id, COALESCE(timestamp, datetime('now'))
FROM pulso.sesiones
WHERE id NOT IN (SELECT id FROM sesiones);
DETACH DATABASE pulso;
SQL

# ── 8. Verificar integridad ──
echo "" | tee -a "$LOG"
echo "✅ VERIFICANDO INTEGRIDAD..." | tee -a "$LOG"
sqlite3 "$NEXUS" "PRAGMA integrity_check;" | tee -a "$LOG"

echo "" | tee -a "$LOG"
echo "📊 CONTEO FINAL:" | tee -a "$LOG"
for t in memoria_episodica memoria_semantica historial_unificado config_sistema sesiones; do
    count=$(sqlite3 "$NEXUS" "SELECT COUNT(*) FROM \"$t\";")
    printf "  %-25s %s\n" "$t:" "$count" | tee -a "$LOG"
done

# ── 9. Archivar y eliminar legacy ──
echo "" | tee -a "$LOG"
echo "🧹 9. LIMPIANDO ARCHIVOS RESIDUALES..." | tee -a "$LOG"

# Mover legacy a backup
mv "$LEGACY" "$BACKUP_DIR/nexus_intelligence.legacy.db.bak" && echo "  ✅ legacy → backup"
mv "data/intelligence.db" "$BACKUP_DIR/intelligence.db.bak" 2>/dev/null && echo "  ✅ intelligence.db → backup" || echo "  ⚠️  intelligence.db no existe"
mv "data/pulso.db" "$BACKUP_DIR/pulso.db.bak" 2>/dev/null && echo "  ✅ pulso.db → backup" || echo "  ⚠️  pulso.db no existe"
mv "data/hipocampo.db" "$BACKUP_DIR/hipocampo.db.bak" 2>/dev/null && echo "  ✅ hipocampo.db → backup" || echo "  ⚠️  hipocampo.db no existe"
mv "data/memoria_episodica.db" "$BACKUP_DIR/memoria_episodica.db.bak" 2>/dev/null && echo "  ✅ memoria_episodica.db → backup" || echo "  ⚠️  memoria_episodica.db no existe"
mv "data/ocean.db" "$BACKUP_DIR/ocean.db.bak" 2>/dev/null && echo "  ✅ ocean.db → backup" || echo "  ⚠️  ocean.db no existe"
rm -rf "data/lancedb" && echo "  ✅ lancedb/ → eliminado" || echo "  ⚠️  lancedb/ no existe"

# ── 10. Resumen final ──
echo "" | tee -a "$LOG"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$LOG"
echo "🔱 UNIFICACIÓN COMPLETADA" | tee -a "$LOG"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$LOG"
echo "📁 Único archivo activo: data/nexus_memoria.db" | tee -a "$LOG"
du -sh "$NEXUS" | tee -a "$LOG"
echo "📁 Backups en: $BACKUP_DIR/" | tee -a "$LOG"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a "$LOG"
