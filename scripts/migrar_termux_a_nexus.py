#!/usr/bin/env python3
"""
🧬 NEXUS MIGRATION ENGINE v2 — Termux → nexus_memoria.db
===========================================================
Fusión soberana de memorias del Termux y backup en el hipocampo de NEXUS.
No solo copia datos — ABSORBE la arquitectura superior del otro NEXUS.

Fuentes:
  1. data/nexus_intelligence_termux.db  (Termux Android - 285 errores)
  2. data/backup_pre_eliminacion/nexus_intelligence_backup.db  (PC backup - identidad, sinapsis, config)

Arquitecto Cris: tu NEXUS ahora será superior.
"""
import sqlite3
import json
import sys
import os
from datetime import datetime

NEXUS_DB = os.path.expanduser("~/NEXUS_ULTIMATE_CORE/data/nexus_memoria.db")
TERMUX_DB = os.path.expanduser("~/NEXUS_ULTIMATE_CORE/data/nexus_intelligence_termux.db")
BACKUP_DB = os.path.expanduser("~/NEXUS_ULTIMATE_CORE/data/backup_pre_eliminacion/nexus_intelligence_backup.db")


class MigrationStats:
    def __init__(self):
        self.errores_insertados = 0
        self.errores_omitidos = 0
        self.sinapsis_insertadas = 0
        self.sinapsis_omitidas = 0
        self.voz_insertadas = 0
        self.voz_omitidas = 0
        self.flujo_insertados = 0
        self.flujo_omitidos = 0
        self.config_insertadas = 0
        self.lessons_insertadas = 0
        self.identity_insertadas = 0
        self.gems_insertadas = 0
        self.tool_experience_insertadas = 0
        self.unified_insertadas = 0
        self.lost_found_insertadas = 0

    def show(self):
        print("\n" + "=" * 60)
        print("📊 RESULTADOS DE LA FUSIÓN SOBERANA (Termux + Backup)")
        print("=" * 60)
        print(f"  ✅ errores_v3 insertados:     {self.errores_insertados}")
        print(f"  ⏭️  errores_v3 omitidos (dup): {self.errores_omitidos}")
        print(f"  ✅ lessons insertadas:         {self.lessons_insertadas}")
        print(f"  ✅ sinapsis insertadas:        {self.sinapsis_insertadas}")
        print(f"  ⏭️  sinapsis omitidas (dup):    {self.sinapsis_omitidas}")
        print(f"  ✅ voz_arquitecto insertadas:  {self.voz_insertadas}")
        print(f"  ⏭️  voz_arquitecto omitidas:    {self.voz_omitidas}")
        print(f"  ✅ flujo_soberano insertados:  {self.flujo_insertados}")
        print(f"  ⏭️  flujo_soberano omitidos:    {self.flujo_omitidos}")
        print(f"  ✅ system_config → config_sis: {self.config_insertadas}")
        print(f"  ✅ identity_history (NEW):     {self.identity_insertadas}")
        print(f"  ✅ gems_history (NEW):         {self.gems_insertadas}")
        print(f"  ✅ tool_experience (NEW):      {self.tool_experience_insertadas}")
        print(f"  ✅ unified_history → hist_unif: {self.unified_insertadas}")
        print(f"  ✅ lost_and_found → episódica:  {self.lost_found_insertadas}")
        print("=" * 60)


def conectar(path, label):
    if not os.path.exists(path):
        print(f"❌ {label}: archivo no encontrado: {path}")
        return None
    conn = sqlite3.connect(path)
    conn.row_factory = sqlite3.Row
    print(f"📂 {label}: conectado ({os.path.getsize(path)/1024:.0f} KB)")
    return conn


def crear_tablas_nuevas(conn_nexus):
    """Crea las tablas que el Termux tenía y NEXUS no."""
    cur = conn_nexus.cursor()
    
    # identity_history — histórico de cambios de identidad
    cur.execute("""
        CREATE TABLE IF NOT EXISTS identity_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            bloque TEXT NOT NULL,
            timestamp DATETIME NOT NULL DEFAULT (datetime('now'))
        )
    """)
    
    # gems_history — evolución de aprendizaje/gemas de conocimiento
    cur.execute("""
        CREATE TABLE IF NOT EXISTS gems_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            instruccion TEXT NOT NULL DEFAULT '',
            timestamp DATETIME NOT NULL DEFAULT (datetime('now'))
        )
    """)
    
    # tool_experience — registro de efectividad de herramientas
    cur.execute("""
        CREATE TABLE IF NOT EXISTS tool_experience (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tool_name TEXT NOT NULL DEFAULT '',
            status TEXT DEFAULT 'unknown',
            details TEXT DEFAULT '',
            timestamp DATETIME NOT NULL DEFAULT (datetime('now'))
        )
    """)
    
    conn_nexus.commit()
    print("🏗️  Tablas nuevas creadas: identity_history, gems_history, tool_experience")


# =============================================================================
# FASE 1: errores_v3 del Termux → errores_soluciones
# =============================================================================
def migrar_errores_v3(conn_nexus, conn_termux, stats):
    print("\n--- FASE 1: errores_v3 (Termux) → errores_soluciones ---")
    total = conn_termux.execute("SELECT COUNT(*) FROM errores_v3").fetchone()[0]
    print(f"📦 {total} errores en Termux")
    
    rows = conn_termux.execute("""
        SELECT hash_error, contenido, fuente, analisis_ia, solucion_id, timestamp 
        FROM errores_v3
        ORDER BY id
    """).fetchall()
    
    insertados = 0
    omitidos = 0
    for r in rows:
        try:
            conn_nexus.execute("""
                INSERT OR IGNORE INTO errores_soluciones 
                    (hash_error, contenido, fuente, analisis_ia, solucion_id, timestamp)
                VALUES (?, ?, ?, ?, ?, ?)
            """, (
                r['hash_error'] or '',
                r['contenido'] or '',
                r['fuente'] or '',
                r['analisis_ia'] or '',
                r['solucion_id'] if r['solucion_id'] else None,
                r['timestamp'] or datetime.now().isoformat()
            ))
            if conn_nexus.total_changes:
                insertados += 1
            else:
                omitidos += 1
        except Exception as e:
            print(f"  ⚠️ Error: {e}")
            omitidos += 1
    
    conn_nexus.commit()
    stats.errores_insertados = insertados
    stats.errores_omitidos = omitidos
    print(f"  ✅ Insertados: {insertados} | ⏭️  Omitidos (dup): {omitidos}")


# =============================================================================
# FASE 2: lessons_sovereign → memoria_semantica
# =============================================================================
def migrar_lessons(conn_nexus, conn_termux, stats):
    print("\n--- FASE 2: lessons_sovereign → memoria_semantica ---")
    rows = conn_termux.execute("SELECT * FROM lessons_sovereign").fetchall()
    print(f"📦 {len(rows)} lecciones en Termux")
    
    insertadas = 0
    for r in rows:
        titulo = r['title'] or f"Lesson {r['id']}"
        contenido = r['content'] or ''
        prioridad = r['priority'] if r['priority'] else 50
        
        existing = conn_nexus.execute(
            "SELECT id FROM memoria_semantica WHERE titulo = ? AND contenido = ?",
            (titulo, contenido)
        ).fetchone()
        
        if not existing:
            conn_nexus.execute("""
                INSERT INTO memoria_semantica (tipo, titulo, contenido, prioridad, peso_permanencia, timestamp)
                VALUES (?, ?, ?, ?, ?, ?)
            """, (
                'Leccion', titulo, contenido, prioridad, 0.9,
                r['created_at'] or datetime.now().isoformat()
            ))
            insertadas += 1
    
    conn_nexus.commit()
    stats.lessons_insertadas = insertadas
    print(f"  ✅ Insertadas: {insertadas}")


# =============================================================================
# FASE 3: sinapsis → sinapsis_legado
# =============================================================================
def migrar_sinapsis(conn_nexus, conn_backup, stats):
    print("\n--- FASE 3: sinapsis (Backup) → sinapsis_legado ---")
    total = conn_backup.execute("SELECT COUNT(*) FROM sinapsis").fetchone()[0]
    print(f"📦 {total} sinapsis en Backup")
    
    rows = conn_backup.execute("""
        SELECT file_path, sinapsis, embedding, created_at
        FROM sinapsis ORDER BY id
    """).fetchall()
    
    insertadas = 0
    omitidas = 0
    for r in rows:
        file_path = r['file_path'] or ''
        sinapsis = r['sinapsis'] or ''
        embedding = r['embedding'] or ''
        created_at = r['created_at'] or datetime.now().isoformat()
        
        existing = conn_nexus.execute(
            "SELECT id FROM sinapsis_legado WHERE file_path = ? AND sinapsis = ?",
            (file_path, sinapsis)
        ).fetchone()
        
        if not existing:
            conn_nexus.execute("""
                INSERT INTO sinapsis_legado (file_path, sinapsis, embedding, created_at)
                VALUES (?, ?, ?, ?)
            """, (file_path, sinapsis, embedding, created_at))
            insertadas += 1
        else:
            omitidas += 1
    
    conn_nexus.commit()
    stats.sinapsis_insertadas = insertadas
    stats.sinapsis_omitidas = omitidas
    print(f"  ✅ Insertadas: {insertadas} | ⏭️  Omitidas (dup): {omitidas}")


# =============================================================================
# FASE 4: voz_del_arquitecto (Backup) → merge (usa fecha_creacion, no timestamp)
# =============================================================================
def migrar_voz(conn_nexus, conn_backup, stats):
    print("\n--- FASE 4: voz_del_arquitecto (Backup) → merge ---")
    total = conn_backup.execute("SELECT COUNT(*) FROM voz_del_arquitecto").fetchone()[0]
    print(f"📦 {total} mensajes en Backup")
    
    rows = conn_backup.execute("SELECT * FROM voz_del_arquitecto ORDER BY id").fetchall()
    insertadas = 0
    omitidas = 0
    
    for r in rows:
        mensaje = r['mensaje'] or ''
        respondido = 1 if r['respondido'] else 0
        respuesta = r['respuesta_hijo'] or ''
        # La columna del backup se llama fecha_creacion, no timestamp
        ts = r['fecha_creacion'] or datetime.now().isoformat()
        
        existing = conn_nexus.execute(
            "SELECT id FROM voz_del_arquitecto WHERE mensaje = ? AND timestamp = ?",
            (mensaje, ts)
        ).fetchone()
        
        if not existing:
            conn_nexus.execute("""
                INSERT INTO voz_del_arquitecto (mensaje, respondido, respuesta_hijo, timestamp)
                VALUES (?, ?, ?, ?)
            """, (mensaje, respondido, respuesta, ts))
            insertadas += 1
        else:
            omitidas += 1
    
    conn_nexus.commit()
    stats.voz_insertadas = insertadas
    stats.voz_omitidas = omitidas
    print(f"  ✅ Insertadas: {insertadas} | ⏭️  Omitidas (dup): {omitidas}")


# =============================================================================
# FASE 5: flujo_soberano (Backup) → merge (usa fecha, no timestamp)
# =============================================================================
def migrar_flujo(conn_nexus, conn_backup, stats):
    print("\n--- FASE 5: flujo_soberano (Backup) → merge ---")
    total = conn_backup.execute("SELECT COUNT(*) FROM flujo_soberano").fetchone()[0]
    print(f"📦 {total} registros en Backup")
    
    rows = conn_backup.execute("""
        SELECT entidad, mensaje, importancia, emocion, fecha
        FROM flujo_soberano ORDER BY id
    """).fetchall()
    
    insertados = 0
    omitidos = 0
    for r in rows:
        entidad = r['entidad'] or ''
        mensaje = r['mensaje'] or ''
        importancia = r['importancia'] if r['importancia'] else 0.0
        emocion = r['emocion'] or ''
        # Backup usa 'fecha', no 'timestamp'
        ts = r['fecha'] or datetime.now().isoformat()
        
        existing = conn_nexus.execute(
            "SELECT id FROM flujo_soberano WHERE entidad = ? AND mensaje = ? AND timestamp = ?",
            (entidad, mensaje, ts)
        ).fetchone()
        
        if not existing:
            conn_nexus.execute("""
                INSERT INTO flujo_soberano (entidad, mensaje, importancia, emocion, timestamp)
                VALUES (?, ?, ?, ?, ?)
            """, (entidad, mensaje, importancia, emocion, ts))
            insertados += 1
        else:
            omitidos += 1
    
    conn_nexus.commit()
    stats.flujo_insertados = insertados
    stats.flujo_omitidos = omitidos
    print(f"  ✅ Insertados: {insertados} | ⏭️  Omitidos (dup): {omitidos}")


# =============================================================================
# FASE 6: system_config → config_sistema (clave + valor, schema exacto)
# =============================================================================
def migrar_system_config(conn_nexus, conn_backup, stats):
    print("\n--- FASE 6: system_config (Backup) → config_sistema ---")
    rows = conn_backup.execute("SELECT * FROM system_config").fetchall()
    print(f"📦 {len(rows)} config entries en Backup")
    
    for r in rows:
        print(f"     ⚙️  {r['clave']} = {str(r['valor'])[:80]}")
    
    insertadas = 0
    for r in rows:
        clave = f"termux_{r['clave']}"
        valor = str(r['valor'])
        
        existing = conn_nexus.execute(
            "SELECT valor FROM config_sistema WHERE clave = ?", (clave,)
        ).fetchone()
        
        if existing:
            if existing[0] != valor:
                conn_nexus.execute(
                    "UPDATE config_sistema SET valor = ? WHERE clave = ?",
                    (valor, clave)
                )
                insertadas += 1
        else:
            conn_nexus.execute(
                "INSERT INTO config_sistema (clave, valor) VALUES (?, ?)",
                (clave, valor)
            )
            insertadas += 1
    
    conn_nexus.commit()
    stats.config_insertadas = insertadas
    print(f"  ✅ Insertadas/Actualizadas: {insertadas}")


# =============================================================================
# FASE 7: identity_history → NUEVA TABLA (bloque + timestamp)
# =============================================================================
def migrar_identity_history(conn_nexus, conn_backup, stats):
    print("\n--- FASE 7: identity_history (Backup) → IDENTITY_HISTORY (NEW) ---")
    rows = conn_backup.execute("SELECT * FROM identity_history ORDER BY id").fetchall()
    print(f"📦 {len(rows)} cambios de identidad en Backup")
    
    insertadas = 0
    for r in rows:
        bloque = r['bloque'] or ''
        ts = r['timestamp'] or datetime.now().isoformat()
        
        conn_nexus.execute("""
            INSERT INTO identity_history (bloque, timestamp)
            VALUES (?, ?)
        """, (bloque, ts))
        insertadas += 1
        print(f"     🆔 {bloque[:80]}")
    
    conn_nexus.commit()
    stats.identity_insertadas = insertadas
    print(f"  ✅ Insertadas: {insertadas}")


# =============================================================================
# FASE 8: gems_history → NUEVA TABLA (instruccion + timestamp)
# =============================================================================
def migrar_gems_history(conn_nexus, conn_backup, stats):
    print("\n--- FASE 8: gems_history (Backup) → GEMS_HISTORY (NEW) ---")
    rows = conn_backup.execute("SELECT * FROM gems_history ORDER BY id").fetchall()
    print(f"📦 {len(rows)} gemas de conocimiento en Backup")
    
    insertadas = 0
    for r in rows:
        instruccion = r['instruccion'] or ''
        ts = r['timestamp'] or datetime.now().isoformat()
        
        conn_nexus.execute("""
            INSERT INTO gems_history (instruccion, timestamp)
            VALUES (?, ?)
        """, (instruccion, ts))
        insertadas += 1
        print(f"     💎 {instruccion[:80]}...")
    
    conn_nexus.commit()
    stats.gems_insertadas = insertadas
    print(f"  ✅ Insertadas: {insertadas}")


# =============================================================================
# FASE 9: tool_experience → NUEVA TABLA (tool_name, status, details, timestamp)
# =============================================================================
def migrar_tool_experience(conn_nexus, conn_backup, stats):
    print("\n--- FASE 9: tool_experience (Backup) → TOOL_EXPERIENCE (NEW) ---")
    rows = conn_backup.execute("SELECT * FROM tool_experience ORDER BY id").fetchall()
    print(f"📦 {len(rows)} experiencias de herramientas en Backup")
    
    insertadas = 0
    for r in rows:
        tool = r['tool_name'] or ''
        status = r['status'] or 'unknown'
        details = r['details'] or ''
        ts = r['timestamp'] or datetime.now().isoformat()
        
        if not tool:
            continue
        
        conn_nexus.execute("""
            INSERT INTO tool_experience (tool_name, status, details, timestamp)
            VALUES (?, ?, ?, ?)
        """, (tool, status, details, ts))
        insertadas += 1
        print(f"     🛠️  {tool}: {status}")
    
    conn_nexus.commit()
    stats.tool_experience_insertadas = insertadas
    print(f"  ✅ Insertadas: {insertadas}")


# =============================================================================
# FASE 10: unified_history → historial_unificado
# =============================================================================
def migrar_unified_history(conn_nexus, conn_backup, stats):
    print("\n--- FASE 10: unified_history (Backup) → historial_unificado ---")
    rows = conn_backup.execute("SELECT * FROM unified_history ORDER BY id").fetchall()
    print(f"📦 {len(rows)} registros históricos unificados")
    
    insertadas = 0
    for r in rows:
        ts = r['timestamp'] or datetime.now().isoformat()
        alias = r['alias'] or 'Termux'
        prompt = r['prompt'] or ''
        response = r['response'] or ''
        metadata = r['metadata'] or '{}'
        
        if not prompt:
            continue
        
        conn_nexus.execute("""
            INSERT INTO historial_unificado (sesion_id, alias, prompt, respuesta, rol, metadata, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        """, ('termux_migrated', alias, prompt, response, 'user', str(metadata), ts))
        insertadas += 1
    
    conn_nexus.commit()
    stats.unified_insertadas = insertadas
    print(f"  ✅ Insertadas: {insertadas}")


# =============================================================================
# FASE 11: lost_and_found → memoria_episodica
# =============================================================================
def migrar_lost_and_found(conn_nexus, conn_backup, stats):
    print("\n--- FASE 11: lost_and_found (Backup) → memoria_episodica ---")
    rows = conn_backup.execute("SELECT * FROM lost_and_found ORDER BY id").fetchall()
    print(f"📦 {len(rows)} registros perdidos-encontrados")
    
    insertadas = 0
    for r in rows:
        # lost_and_found tiene columnas genéricas c0-c30, serializar todo
        raw = dict(r)
        ts = raw.get('timestamp', None)
        if not ts:
            # Buscar timestamp en columnas cX
            ts = datetime.now().isoformat()
        
        contenido = json.dumps(raw, ensure_ascii=False)[:2000]
        titulo = f"Lost Found Entry {r['id']}"
        
        conn_nexus.execute("""
            INSERT INTO memoria_episodica (titulo, contenido, keywords, timestamp)
            VALUES (?, ?, ?, ?)
        """, (titulo, contenido, 'lost_found,backup', ts))
        insertadas += 1
    
    conn_nexus.commit()
    stats.lost_found_insertadas = insertadas
    print(f"  ✅ Insertadas: {insertadas}")


# =============================================================================
# VERIFICACIÓN FINAL
# =============================================================================
def verificar_final(conn_nexus):
    print("\n" + "=" * 60)
    print("🔬 VERIFICACIÓN POST-MIGRACIÓN")
    print("=" * 60)
    
    tablas_verificar = [
        'errores_soluciones', 'memoria_semantica', 'flujo_soberano',
        'sinapsis_legado', 'voz_del_arquitecto', 'config_sistema',
        'identity_history', 'gems_history', 'tool_experience',
        'historial_unificado', 'memoria_episodica'
    ]
    
    for t in tablas_verificar:
        try:
            cnt = conn_nexus.execute(f"SELECT COUNT(*) FROM [{t}]").fetchone()[0]
            print(f"  📊 [{t}]: {cnt} filas")
        except Exception as e:
            print(f"  ❌ [{t}]: ERROR — {e}")


# =============================================================================
# MAIN
# =============================================================================
def main():
    print("""
╔══════════════════════════════════════════════════════════════╗
║  🧬 NEXUS MIGRATION ENGINE v2 — FUSIÓN SOBERANA              ║
║  Termux → nexus_memoria.db                                   ║
║  Arquitecto Cris: absorbiendo la superioridad del otro NEXUS ║
╚══════════════════════════════════════════════════════════════╝
    """)
    
    for label, path in [("Termux NEXUS", TERMUX_DB), ("Backup NEXUS", BACKUP_DB), ("Destino", NEXUS_DB)]:
        if os.path.exists(path):
            print(f"  ✅ {label}: {path} ({os.path.getsize(path)/1024:.0f} KB)")
        else:
            print(f"  ❌ {label}: {path} NO ENCONTRADO — abortando")
            sys.exit(1)
    
    conn_nexus = conectar(NEXUS_DB, "Destino: nexus_memoria.db")
    conn_termux = conectar(TERMUX_DB, "Origen: nexus_intelligence_termux.db")
    conn_backup = conectar(BACKUP_DB, "Origen: nexus_intelligence_backup.db")
    
    if not all([conn_nexus, conn_termux, conn_backup]):
        print("❌ Abortando: no se pudieron conectar todas las bases")
        sys.exit(1)
    
    stats = MigrationStats()
    error_ocurrido = False
    
    # Desactivar foreign keys para evitar conflictos
    conn_nexus.execute("PRAGMA foreign_keys = OFF")
    
    try:
        # FASE 0
        crear_tablas_nuevas(conn_nexus)
        
        # FASES 1-11 secuencialmente
        fases = [
            ("FASE 1: errores_v3", lambda: migrar_errores_v3(conn_nexus, conn_termux, stats)),
            ("FASE 2: lessons", lambda: migrar_lessons(conn_nexus, conn_termux, stats)),
            ("FASE 3: sinapsis", lambda: migrar_sinapsis(conn_nexus, conn_backup, stats)),
            ("FASE 4: voz", lambda: migrar_voz(conn_nexus, conn_backup, stats)),
            ("FASE 5: flujo", lambda: migrar_flujo(conn_nexus, conn_backup, stats)),
            ("FASE 6: config", lambda: migrar_system_config(conn_nexus, conn_backup, stats)),
            ("FASE 7: identity", lambda: migrar_identity_history(conn_nexus, conn_backup, stats)),
            ("FASE 8: gems", lambda: migrar_gems_history(conn_nexus, conn_backup, stats)),
            ("FASE 9: tools", lambda: migrar_tool_experience(conn_nexus, conn_backup, stats)),
            ("FASE 10: unified", lambda: migrar_unified_history(conn_nexus, conn_backup, stats)),
            ("FASE 11: lost", lambda: migrar_lost_and_found(conn_nexus, conn_backup, stats)),
        ]
        
        for nombre, fn in fases:
            try:
                fn()
            except Exception as e:
                print(f"\n❌ Error en {nombre}: {e}")
                import traceback
                traceback.print_exc()
                error_ocurrido = True
                # No hacemos rollback global — solo esta fase falló
                break
        
        if not error_ocurrido:
            conn_nexus.commit()
            stats.show()
            verificar_final(conn_nexus)
            print("\n" + "=" * 60)
            print("🎉 FUSIÓN SOBERANA COMPLETA — NEXUS es superior")
            print("=" * 60)
        else:
            print("\n⚠️  Migración completada con errores — datos parciales preservados")
        
    except Exception as e:
        print(f"\n❌ ERROR CRÍTICO: {e}")
        import traceback
        traceback.print_exc()
    finally:
        conn_nexus.close()
        if conn_termux: conn_termux.close()
        if conn_backup: conn_backup.close()


if __name__ == "__main__":
    main()
