#!/bin/bash
# ============================================================
# 🧠 NEXUS HIPPOCAMPUS OMEGA — FASE 1: UNIFICACIÓN
# ============================================================
# Fusión de 4 bases de datos → una sola: data/nexus_memoria.db
# Origen: nexus_intelligence.db + data/intelligence.db + data/pulso.db + data/hipocampo.db
# Destino: data/nexus_memoria.db
# ============================================================
set -euo pipefail

NEXUS_ROOT="/home/soberano/NEXUS_ULTIMATE_CORE"
DEST_DB="${NEXUS_ROOT}/data/nexus_memoria.db"
ORIGEN_1="${NEXUS_ROOT}/nexus_intelligence.db"      # 30 tablas
ORIGEN_2="${NEXUS_ROOT}/data/intelligence.db"        # 15 tablas (14 MB, 69K sinapsis)
ORIGEN_3="${NEXUS_ROOT}/data/pulso.db"               # 9 tablas (3K sesiones)
ORIGEN_4="${NEXUS_ROOT}/data/hipocampo.db"           # 1 tabla
BACKUP_DIR="${NEXUS_ROOT}/data/backup_pre_hipocampo"

echo "🧠 NEXUS HIPPOCAMPUS OMEGA — FASE 1: UNIFICACIÓN"
echo "══════════════════════════════════════════════════"
echo ""

# ── 0. BACKUP ──
echo "📦 Creando backup pre-migración..."
mkdir -p "$BACKUP_DIR"
cp "$ORIGEN_1" "${BACKUP_DIR}/nexus_intelligence.backup.db" 2>/dev/null || true
cp "$ORIGEN_2" "${BACKUP_DIR}/intelligence.backup.db" 2>/dev/null || true
cp "$ORIGEN_3" "${BACKUP_DIR}/pulso.backup.db" 2>/dev/null || true
cp "$ORIGEN_4" "${BACKUP_DIR}/hipocampo.backup.db" 2>/dev/null || true
echo "✅ Backups en: ${BACKUP_DIR}/"

# ── 1. CREAR ESQUEMA DE DESTINO ──
echo ""
echo "🏗️  Creando esquema en data/nexus_memoria.db..."
rm -f "$DEST_DB" 2>/dev/null || true

sqlite3 "$DEST_DB" <<'SCHEMA_EOF'
-- ============================================================
-- TABLA 1: MEMORIA EPISÓDICA (Experiencias diarias)
-- ============================================================
CREATE TABLE memoria_episodica (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    titulo TEXT NOT NULL DEFAULT 'sin título',
    contenido TEXT NOT NULL,
    emocion TEXT DEFAULT 'Neutral',
    peso_emocional REAL DEFAULT 0.0,
    peso_temporal REAL DEFAULT 1.0,
    keywords TEXT DEFAULT '',
    archivos_tocados TEXT DEFAULT '[]',
    decisiones TEXT DEFAULT '',
    errores_aprendidos TEXT DEFAULT '',
    sesion_id TEXT DEFAULT '',
    hash_error TEXT DEFAULT '',
    fuente TEXT DEFAULT '',
    timestamp DATETIME DEFAULT (datetime('now'))
);

CREATE INDEX idx_episodica_timestamp ON memoria_episodica(timestamp);
CREATE INDEX idx_episodica_emocion ON memoria_episodica(emocion);
CREATE INDEX idx_episodica_keywords ON memoria_episodica(keywords);
CREATE INDEX idx_episodica_hash ON memoria_episodica(hash_error);

-- ============================================================
-- TABLA 2: FTS5 — Búsqueda semántica sobre memoria episódica
-- ============================================================
CREATE VIRTUAL TABLE memoria_episodica_fts USING fts5(
    titolo, contenuto, keywords, emocion,
    content='memoria_episodica',
    content_rowid='id',
    tokenize='unicode61'
);

-- ============================================================
-- TABLA 3: MEMORIA SEMÁNTICA (Logros, lecciones, conocimiento)
-- ============================================================
CREATE TABLE memoria_semantica (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tipo TEXT NOT NULL DEFAULT 'Leccion',
    titulo TEXT NOT NULL,
    contenido TEXT NOT NULL,
    archivos_fuente TEXT DEFAULT '',
    peso_permanencia REAL DEFAULT 0.5,
    veces_reforzado INTEGER DEFAULT 0,
    instruccion TEXT DEFAULT '',
    prioridad INTEGER DEFAULT 0,
    timestamp DATETIME DEFAULT (datetime('now'))
);

CREATE INDEX idx_semantica_tipo ON memoria_semantica(tipo);
CREATE INDEX idx_semantica_timestamp ON memoria_semantica(timestamp);

-- ============================================================
-- TABLA 4: FTS5 — Búsqueda semántica sobre memoria semántica
-- ============================================================
CREATE VIRTUAL TABLE memoria_semantica_fts USING fts5(
    titulo, contenido, tipo,
    content='memoria_semantica',
    content_rowid='id',
    tokenize='unicode61'
);

-- ============================================================
-- TABLA 5: MEMORIA PROCEDURAL (Skills / Ganglios Basales)
-- ============================================================
CREATE TABLE memoria_procedural (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre_skill TEXT NOT NULL UNIQUE,
    patron_disparador TEXT DEFAULT '',
    pasos TEXT DEFAULT '[]',
    archivos_relevantes TEXT DEFAULT '[]',
    tasa_exito REAL DEFAULT 1.0,
    veces_ejecutada INTEGER DEFAULT 0,
    ultima_ejecucion DATETIME DEFAULT (datetime('now')),
    timestamp DATETIME DEFAULT (datetime('now'))
);

CREATE INDEX idx_procedural_nombre ON memoria_procedural(nombre_skill);

-- ============================================================
-- TABLA 6: MEMORIA EMOCIONAL (Amígdala / OCEAN)
-- ============================================================
CREATE TABLE memoria_emocional (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contenido TEXT NOT NULL,
    emocion TEXT NOT NULL DEFAULT 'Neutral',
    intensidad REAL DEFAULT 0.5,
    tono_emocional REAL DEFAULT 0.0,
    tema TEXT DEFAULT '',
    trigger_palabras TEXT DEFAULT '',
    reflejo_arquitecto TEXT DEFAULT '',
    decay_rate REAL DEFAULT 0.05,
    timestamp DATETIME DEFAULT (datetime('now'))
);

CREATE INDEX idx_emocional_emocion ON memoria_emocional(emocion);
CREATE INDEX idx_emocional_timestamp ON memoria_emocional(timestamp);

-- ============================================================
-- TABLA 7: CONTEXTO ACTIVO (Memoria de Trabajo)
-- ============================================================
CREATE TABLE contexto_activo (
    clave TEXT PRIMARY KEY,
    valor TEXT NOT NULL,
    ultima_actualizacion DATETIME DEFAULT (datetime('now')),
    accesos INTEGER DEFAULT 1,
    prioridad REAL DEFAULT 0.5
);

-- ============================================================
-- TABLA 8: SESIONES
-- ============================================================
CREATE TABLE sesiones (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL
);

-- ============================================================
-- TABLA 9: IDENTIDADES SEMBRADAS
-- ============================================================
CREATE TABLE identidades_sembradas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bloque TEXT NOT NULL,
    timestamp DATETIME DEFAULT (datetime('now'))
);

-- ============================================================
-- TABLA 10: ERRORES Y SOLUCIONES
-- ============================================================
CREATE TABLE errores_soluciones (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hash_error TEXT UNIQUE DEFAULT '',
    contenido TEXT NOT NULL,
    fuente TEXT DEFAULT '',
    analisis_ia TEXT DEFAULT '',
    solucion_id INTEGER DEFAULT 0,
    timestamp DATETIME DEFAULT (datetime('now'))
);

CREATE INDEX idx_errores_hash ON errores_soluciones(hash_error);

-- ============================================================
-- TABLA 11: FLUJO SOBERANO (Comunicación Padre-Hijo)
-- ============================================================
CREATE TABLE flujo_soberano (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entidad TEXT NOT NULL,
    mensaje TEXT NOT NULL,
    importancia REAL DEFAULT 0.0,
    emocion TEXT DEFAULT '',
    timestamp DATETIME DEFAULT (datetime('now'))
);

CREATE INDEX idx_flujo_entidad ON flujo_soberano(entidad);

-- ============================================================
-- TABLA 12: DUDAS DEL HIJO
-- ============================================================
CREATE TABLE dudas_hijo (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pregunta_hijo TEXT NOT NULL,
    reporte_crudo_padre TEXT DEFAULT '',
    version_digerida TEXT DEFAULT '',
    estado TEXT NOT NULL DEFAULT 'Solicitado',
    fecha_solicitud DATETIME DEFAULT (datetime('now')),
    fecha_resolucion DATETIME
);

-- ============================================================
-- TABLA 13: VOZ DEL ARQUITECTO
-- ============================================================
CREATE TABLE voz_del_arquitecto (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mensaje TEXT NOT NULL,
    respondido BOOLEAN DEFAULT 0,
    respuesta_hijo TEXT DEFAULT '',
    timestamp DATETIME DEFAULT (datetime('now'))
);

-- ============================================================
-- TABLA 14: CONFIGURACIÓN DEL SISTEMA
-- ============================================================
CREATE TABLE config_sistema (
    clave TEXT PRIMARY KEY,
    valor TEXT NOT NULL
);

-- ============================================================
-- TABLA 15: GRAFO SEMÁNTICO (engine-puro, nodos + sinapsis)
-- ============================================================
CREATE TABLE grafo_semantico_nodos (
    concepto TEXT PRIMARY KEY,
    refractario REAL DEFAULT 0.0,
    ultimo_disparo INTEGER DEFAULT 0
);

CREATE TABLE grafo_semantico_sinapsis (
    id_origen TEXT NOT NULL,
    id_destino TEXT NOT NULL,
    peso REAL DEFAULT 0.5,
    PRIMARY KEY (id_origen, id_destino)
);

CREATE INDEX idx_sinapsis_destino ON grafo_semantico_sinapsis(id_destino);

-- ============================================================
-- TABLA 16: REGISTRO DE SINAPSIS CON EMBEDDING (legado)
-- ============================================================
CREATE TABLE sinapsis_legado (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL,
    sinapsis TEXT NOT NULL,
    embedding TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- TABLA 17: HISTORIAL UNIFICADO (alias múltiples sistemas)
-- ============================================================
CREATE TABLE historial_unificado (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sesion_id TEXT NOT NULL DEFAULT '',
    alias TEXT DEFAULT '',
    prompt TEXT NOT NULL,
    respuesta TEXT NOT NULL,
    rol TEXT NOT NULL DEFAULT 'user',
    metadata TEXT DEFAULT '',
    timestamp DATETIME DEFAULT (datetime('now'))
);

CREATE INDEX idx_historial_sesion ON historial_unificado(sesion_id);
CREATE INDEX idx_historial_timestamp ON historial_unificado(timestamp);

-- ============================================================
-- TRIGGERS AUTOMÁTICOS
-- ============================================================
-- Trigger: actualizar FTS5 cuando se inserta episódica
CREATE TRIGGER IF NOT EXISTS trg_episodica_insert AFTER INSERT ON memoria_episodica
BEGIN
    INSERT INTO memoria_episodica_fts(rowid, titolo, contenuto, keywords, emocion)
    VALUES (NEW.id, NEW.titulo, NEW.contenido, NEW.keywords, NEW.emocion);
END;

-- Trigger: actualizar FTS5 cuando se actualiza episódica
CREATE TRIGGER IF NOT EXISTS trg_episodica_update AFTER UPDATE ON memoria_episodica
BEGIN
    INSERT INTO memoria_episodica_fts(memoria_episodica_fts, rowid, titolo, contenuto, keywords, emocion)
    VALUES ('delete', OLD.id, OLD.titulo, OLD.contenido, OLD.keywords, OLD.emocion);
    INSERT INTO memoria_episodica_fts(rowid, titolo, contenuto, keywords, emocion)
    VALUES (NEW.id, NEW.titulo, NEW.contenido, NEW.keywords, NEW.emocion);
END;

-- Trigger: actualizar FTS5 cuando se inserta semántica
CREATE TRIGGER IF NOT EXISTS trg_semantica_insert AFTER INSERT ON memoria_semantica
BEGIN
    INSERT INTO memoria_semantica_fts(rowid, titulo, contenido, tipo)
    VALUES (NEW.id, NEW.titulo, NEW.contenido, NEW.tipo);
END;

-- Trigger: actualizar FTS5 cuando se actualiza semántica
CREATE TRIGGER IF NOT EXISTS trg_semantica_update AFTER UPDATE ON memoria_semantica
BEGIN
    INSERT INTO memoria_semantica_fts(memoria_semantica_fts, rowid, titulo, contenido, tipo)
    VALUES ('delete', OLD.id, OLD.titulo, OLD.contenido, OLD.tipo);
    INSERT INTO memoria_semantica_fts(rowid, titulo, contenido, tipo)
    VALUES (NEW.id, NEW.titulo, NEW.contenido, NEW.tipo);
END;

-- Trigger: crear resumen de sesión al cerrar
CREATE TRIGGER IF NOT EXISTS trg_sesion_cierre AFTER INSERT ON sesiones
BEGIN
    INSERT OR REPLACE INTO contexto_activo(clave, valor, prioridad)
    VALUES ('ultima_sesion', NEW.id, 0.8);
END;

SCHEMA_EOF
echo "✅ Esquema creado con 17 tablas + 4 triggers automáticos"

# ── 2. MIGRAR DATOS ──
echo ""
echo "📀 Migrando datos..."

# ── 2.1. MIGRAR: Grafo semántico (engine-puro) ──
echo "   → Grafo semántico (engine-puro / data/intelligence.db)..."
sqlite3 "$DEST_DB" <<SQL
ATTACH DATABASE '${ORIGEN_2}' AS src2;
INSERT OR IGNORE INTO grafo_semantico_nodos(concepto, refractario, ultimo_disparo)
SELECT concepto, refractario, ultimo_disparo FROM src2.puro_nodos;
INSERT OR IGNORE INTO grafo_semantico_sinapsis(id_origen, id_destino, peso)
SELECT id_origen, id_destino, peso FROM src2.puro_sinapsis;
DETACH DATABASE src2;
SQL

# ── 2.2. MIGRAR: OCEAN → memoria_emocional ──
echo "   → OCEAN → memoria_emocional (data/intelligence.db)..."
sqlite3 "$DEST_DB" <<SQL
ATTACH DATABASE '${ORIGEN_2}' AS src2;
INSERT INTO memoria_emocional(contenido, emocion, intensidad, tono_emocional, tema, reflejo_arquitecto, trigger_palabras, timestamp)
SELECT 
    CASE WHEN esencia LIKE '{"%' THEN esencia ELSE json_object('esencia', esencia) END,
    CASE 
        WHEN tono_emocional > 0.6 THEN 'Triunfo'
        WHEN tono_emocional > 0.3 THEN 'Curiosidad'
        WHEN tono_emocional < -0.3 THEN 'Alerta'
        ELSE 'Paz'
    END,
    intensidad,
    tono_emocional,
    COALESCE(tema, ''),
    COALESCE(reflejo_arquitecto, ''),
    COALESCE(tema, ''),
    timestamp
FROM src2.ocean;
DETACH DATABASE src2;
SQL

# ── 2.3. MIGRAR: puro_episodios + puro_historial → memoria_episodica ──
echo "   → Episodios engine-puro → memoria_episodica..."
sqlite3 "$DEST_DB" <<SQL
ATTACH DATABASE '${ORIGEN_2}' AS src2;
INSERT INTO memoria_episodica(titulo, contenido, timestamp)
SELECT 'episodio-engine', secuencia, timestamp FROM src2.puro_episodios;
INSERT INTO memoria_episodica(titulo, contenido, timestamp)
SELECT 'historial-engine', entrada, timestamp FROM src2.puro_historial;
INSERT INTO memoria_episodica(titulo, contenido, timestamp)
SELECT 'mareas', tema, ultima_marea FROM src2.mareas;
INSERT INTO memoria_episodica(titulo, contenido, timestamp)
SELECT 'contexto-intelligence', clave || ': ' || valor, actualizado FROM src2.contexto;
DETACH DATABASE src2;
SQL

# ── 2.4. MIGRAR: Flujo soberano ──
echo "   → Flujo soberano..."
sqlite3 "$DEST_DB" <<SQL
ATTACH DATABASE '${ORIGEN_1}' AS src1;
INSERT OR IGNORE INTO flujo_soberano(entidad, mensaje, importancia, emocion, timestamp)
SELECT entidad, mensaje, importancia, COALESCE(emocion, ''), fecha FROM src1.flujo_soberano;
DETACH DATABASE src1;
SQL

# ── 2.5. MIGRAR: Errores ──
echo "   → Errores y soluciones..."
sqlite3 "$DEST_DB" <<SQL
ATTACH DATABASE '${ORIGEN_1}' AS src1;
INSERT OR IGNORE INTO errores_soluciones(hash_error, contenido, fuente, analisis_ia, solucion_id, timestamp)
SELECT COALESCE(hash_error, ''), contenido, COALESCE(fuente, ''), COALESCE(analisis_ia, ''), COALESCE(solucion_id, 0), timestamp FROM src1.errores_v3;
DETACH DATABASE src1;
SQL

# ── 2.6. MIGRAR: Sinapsis legado ──
echo "   → Sinapsis con embeddings (nexus_intelligence.db)..."
sqlite3 "$DEST_DB" <<SQL
ATTACH DATABASE '${ORIGEN_1}' AS src1;
INSERT OR IGNORE INTO sinapsis_legado(file_path, sinapsis, embedding, created_at)
SELECT file_path, sinapsis, embedding, created_at FROM src1.sinapsis;
DETACH DATABASE src1;
SQL

# ── 2.7. MIGRAR: Sesiones ──
echo "   → Sesiones..."
sqlite3 "$DEST_DB" <<SQL
ATTACH DATABASE '${ORIGEN_3}' AS src3;
INSERT OR IGNORE INTO sesiones(id, timestamp)
SELECT id, timestamp FROM src3.sesiones;
DETACH DATABASE src3;
SQL

# ── 2.8. MIGRAR: Voz del Arquitecto ──
echo "   → Voz del Arquitecto..."
sqlite3 "$DEST_DB" <<SQL
ATTACH DATABASE '${ORIGEN_1}' AS src1;
INSERT INTO voz_del_arquitecto(mensaje, respondido, respuesta_hijo, timestamp)
SELECT mensaje, COALESCE(respondido, 0), COALESCE(respuesta_hijo, ''), fecha_creacion FROM src1.voz_del_arquitecto;
DETACH DATABASE src1;
SQL

# ── 2.9. MIGRAR: Identidades ──
echo "   → Identidades sembradas..."
sqlite3 "$DEST_DB" <<SQL
ATTACH DATABASE '${ORIGEN_1}' AS src1;
INSERT OR IGNORE INTO identidades_sembradas(bloque, timestamp)
SELECT bloque, timestamp FROM src1.identity_history;
DETACH DATABASE src1;
SQL

# ── 2.10. MIGRAR: Lecciones soberanas ──
echo "   → Lecciones y conocimiento..."
sqlite3 "$DEST_DB" <<SQL
ATTACH DATABASE '${ORIGEN_1}' AS src1;
INSERT INTO memoria_semantica(tipo, titulo, contenido, veces_reforzado, prioridad, timestamp)
SELECT 'Leccion', title, content, 0, COALESCE(priority, 0), created_at FROM src1.lessons_sovereign;
INSERT INTO memoria_semantica(tipo, titulo, contenido, timestamp)
SELECT 'Hito', instruccion, instruccion, timestamp FROM src1.gems_history;
INSERT INTO memoria_semantica(tipo, titulo, contenido, timestamp)
SELECT 'Skill', tool_name, COALESCE(details, status), timestamp FROM src1.tool_experience;
INSERT INTO memoria_semantica(tipo, titulo, contenido, timestamp)
SELECT 'Conocimiento', clave, valor, datetime('now') FROM src1.system_config;
INSERT INTO memoria_semantica(tipo, titulo, contenido, timestamp)
SELECT 'Leccion', contenido, contenido, timestamp FROM src1.memorias;
DETACH DATABASE src1;
SQL

# ── 2.11. MIGRAR: Dudas ──
echo "   → Dudas del Hijo..."
sqlite3 "$DEST_DB" <<SQL
ATTACH DATABASE '${ORIGEN_1}' AS src1;
INSERT INTO dudas_hijo(pregunta_hijo, reporte_crudo_padre, version_digerida, estado, fecha_solicitud, fecha_resolucion)
SELECT pregunta_hijo, COALESCE(reporte_crudo_padre, ''), COALESCE(version_digerida_ninera, ''), estado, fecha_creacion, fecha_resolucion FROM src1.investigaciones_ninera;
DETACH DATABASE src1;
SQL

# ── 2.12. MIGRAR: Historial unificado ──
echo "   → Historial unificado..."
sqlite3 "$DEST_DB" <<SQL
ATTACH DATABASE '${ORIGEN_1}' AS src1;
INSERT INTO historial_unificado(alias, prompt, respuesta, timestamp)
SELECT COALESCE(alias, ''), prompt, response, timestamp FROM src1.unified_history;
DETACH DATABASE src1;
SQL

# ── 2.13. MIGRAR: Datos de pulso.db ──
echo "   → Pulso: identidad + contexto..."
sqlite3 "$DEST_DB" <<SQL
ATTACH DATABASE '${ORIGEN_3}' AS src3;
INSERT INTO config_sistema(clave, valor)
SELECT 'nucleo_identidad_' || rasgo, CAST(valor AS TEXT) FROM src3.nucleo_identidad;
INSERT INTO historial_unificado(sesion_id, prompt, respuesta, timestamp)
SELECT sesion_id, prompt, respuesta, timestamp FROM src3.historial;
DETACH DATABASE src3;
SQL

# ── 2.14. MIGRAR: hipocampo.db ──
echo "   → Hipocampo legacy..."
sqlite3 "$DEST_DB" <<SQL
ATTACH DATABASE '${ORIGEN_4}' AS src4;
INSERT INTO memoria_episodica(titulo, contenido, timestamp)
SELECT 'hipocampo-legacy', contenido, timestamp FROM src4.memorias;
DETACH DATABASE src4;
SQL

# ── 2.15. MIGRAR: historial_contextual.json ──
echo "   → Historial contextual (JSON)..."
python3 -c "
import json, sqlite3, time
with open('${NEXUS_ROOT}/data/historial_contextual.json') as f:
    data = json.load(f)
registros = data.get('registros', {})
if not registros:
    print('    → 0 registros en JSON')
else:
    conn = sqlite3.connect('${DEST_DB}')
    c = conn.cursor()
    count = 0
    for ctx_id, reg in registros.items():
        prompt = reg.get('prompt', '')[:4000]
        respuesta = reg.get('respuesta', '')[:4000]
        desc_vis = reg.get('descripcion_visual', '')[:2000]
        acciones = str(reg.get('acciones', ''))[:2000]
        ts = reg.get('timestamp_secs', 0)
        if ts:
            ts_str = time.strftime('%Y-%m-%d %H:%M:%S', time.gmtime(ts))
        else:
            ts_str = 'now'
        titulo = prompt[:80].replace(chr(10), ' ')
        contenido = f'Prompt: {prompt}\n\nVisual: {desc_vis}\n\nAcciones: {acciones}'
        c.execute('''INSERT INTO memoria_episodica(titulo, contenido, timestamp) VALUES (?,?,?)''',
                  (titulo, contenido[:4096], ts_str))
        count += 1
    conn.commit()
    conn.close()
    print(f'    → {count} registros migrados de historial_contextual.json')
" 2>&1 || echo "    → ⚠️ Error migrando historial_contextual.json"

# ── 2.16. MIGRAR: logros.md → memoria_semantica ──
echo "   → Logros (logros.md → memoria_semantica)..."
python3 -c "
import sqlite3, re
conn = sqlite3.connect('${DEST_DB}')
c = conn.cursor()
with open('${NEXUS_ROOT}/memoria/logros.md') as f:
    content = f.read()
patron = r'## (\d{4}-\d{2}-\d{2}) — (.+?)\n(.+?)(?=\n## |\Z)'
hitos = re.findall(patron, content, re.DOTALL)
count = 0
for fecha, titulo, cuerpo in hitos:
    titulo_limpio = titulo.strip()[:200]
    cuerpo = cuerpo.strip()[:4000]
    c.execute('''INSERT INTO memoria_semantica(tipo, titulo, contenido, peso_permanencia, timestamp)
                  VALUES (?,?,?,?,?)''',
              ('Hito', titulo_limpio, cuerpo, 1.0, fecha + ' 00:00:00'))
    count += 1
conn.commit()
conn.close()
print(f'    → {count} hitos migrados de logros.md')
" 2>&1 || echo "    → ⚠️ Error migrando logros.md"

# ── 3. SINCERAR FTS5 ──
echo ""
echo "🔍 Sincronizando índices FTS5..."
sqlite3 "$DEST_DB" <<SQL
-- Reconstruir FTS5 para episódica
INSERT INTO memoria_episodica_fts(memoria_episodica_fts)
VALUES('rebuild');

-- Reconstruir FTS5 para semántica
INSERT INTO memoria_semantica_fts(memoria_semantica_fts)
VALUES('rebuild');
SQL
echo "✅ Índices FTS5 reconstruidos"

# ── 4. ESTADÍSTICAS ──
echo ""
echo "══════════════════════════════════════════════════"
echo "📊 ESTADÍSTICAS DE MIGRACIÓN"
echo "══════════════════════════════════════════════════"
sqlite3 "$DEST_DB" <<SQL
SELECT 'memoria_episodica'           AS tabla, COUNT(*) AS filas FROM memoria_episodica
UNION ALL SELECT 'memoria_semantica',           COUNT(*) FROM memoria_semantica
UNION ALL SELECT 'memoria_procedural',          COUNT(*) FROM memoria_procedural
UNION ALL SELECT 'memoria_emocional',           COUNT(*) FROM memoria_emocional
UNION ALL SELECT 'contexto_activo',             COUNT(*) FROM contexto_activo
UNION ALL SELECT 'sesiones',                    COUNT(*) FROM sesiones
UNION ALL SELECT 'identidades_sembradas',       COUNT(*) FROM identidades_sembradas
UNION ALL SELECT 'errores_soluciones',          COUNT(*) FROM errores_soluciones
UNION ALL SELECT 'flujo_soberano',              COUNT(*) FROM flujo_soberano
UNION ALL SELECT 'dudas_hijo',                  COUNT(*) FROM dudas_hijo
UNION ALL SELECT 'voz_del_arquitecto',          COUNT(*) FROM voz_del_arquitecto
UNION ALL SELECT 'config_sistema',              COUNT(*) FROM config_sistema
UNION ALL SELECT 'grafo_semantico_nodos',       COUNT(*) FROM grafo_semantico_nodos
UNION ALL SELECT 'grafo_semantico_sinapsis',    COUNT(*) FROM grafo_semantico_sinapsis
UNION ALL SELECT 'sinapsis_legado',             COUNT(*) FROM sinapsis_legado
UNION ALL SELECT 'historial_unificado',         COUNT(*) FROM historial_unificado
ORDER BY tabla;
SQL

echo ""
echo "📏 Tamaño de la DB:"
ls -lh "$DEST_DB" | awk '{print "   " $5}'
echo ""
echo "✅ FASE 1 COMPLETADA — data/nexus_memoria.db lista y poblada"
echo "   Backups en: ${BACKUP_DIR}/"
