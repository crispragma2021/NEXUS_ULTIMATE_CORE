#!/usr/bin/env python3
"""
🧬 NEXUS MIGRATION ENGINE — intelligence.db → nexus_memoria.db
===============================================================
Fusión soberana de memorias históricas en el hipocampo de NEXUS.
Preserva OCEAN, identidad, sinapsis puro, conceptos y emociones.
Arquitecto Cris: tus recuerdos ahora son UNO con NEXUS.
"""
import sqlite3
import json
import sys
import os
from datetime import datetime

NEXUS_DB = os.path.expanduser("~/NEXUS_ULTIMATE_CORE/data/nexus_memoria.db")
INTELLIGENCE_FILES = [
    os.path.expanduser("~/NEXUS_ULTIMATE_CORE/data/intelligence.db"),
    os.path.expanduser("~/NEXUS_ULTIMATE_CORE/src-tauri/data/intelligence.db"),
    os.path.expanduser("~/NEXUS_ULTIMATE_CORE/engine-puro/data/intelligence.db"),
]

class MigrationStats:
    def __init__(self):
        self.ocean = 0
        self.ocean_dup = 0
        self.nucleo = 0
        self.nucleo_dup = 0
        self.memoria_unica = 0
        self.synapse_nodos = 0
        self.synapse_sinapsis = 0
        self.puro_nodos = 0
        self.puro_nodos_dup = 0
        self.puro_sinapsis = 0
        self.puro_sinapsis_dup = 0
        self.puro_estado = 0
        self.puro_estado_dup = 0
        self.puro_episodios = 0
        self.puro_historial = 0
        self.corteza = 0
        self.flujo = 0
        self.dudas = 0
        self.voz = 0
        self.contexto = 0
        self.sesiones = 0
        self.historial = 0
        self.preferencias = 0
        self.investigaciones = 0

    def show(self):
        print(f"""
╔══════════════════════════════════════════╗
║     📊 REPORTE DE MIGRACIÓN COMPLETO     ║
╚══════════════════════════════════════════╝
🌊 OCEAN → memoria_emocional:     {self.ocean} insertados, {self.ocean_dup} duplicados
🧬 núcleo_identidad → config:     {self.nucleo} insertados, {self.nucleo_dup} duplicados
📦 memoria_unica → episódica:     {self.memoria_unica}
🧠 synapse_conceptos → grafo:     {self.synapse_nodos} nodos, {self.synapse_sinapsis} sinapsis
🔮 puro_nodos → grafo:            {self.puro_nodos} insertados, {self.puro_nodos_dup} duplicados
🔗 puro_sinapsis → grafo:         {self.puro_sinapsis} insertados, {self.puro_sinapsis_dup} duplicados
⚙️  puro_estado → config:         {self.puro_estado} insertados, {self.puro_estado_dup} duplicados
📜 puro_episodios → episódica:    {self.puro_episodios}
📋 puro_historial → historial:    {self.puro_historial}
🧠 corteza_prefrontal → config:   {self.corteza}
🌊 flujo_soberano:                {self.flujo}
❓ dudas_hijo:                     {self.dudas}
🗣️  voz_del_arquitecto:           {self.voz}
📌 contexto_activo:               {self.contexto}
🔑 sesiones:                      {self.sesiones}
📚 historial_unificado:           {self.historial}
⭐ nexo_preferencias:              {self.preferencias}
🔬 investigaciones_ninera:        {self.investigaciones}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
""")

stats = MigrationStats()

def conectar_origen(path):
    if not os.path.exists(path):
        return None
    try:
        conn = sqlite3.connect(path)
        conn.row_factory = sqlite3.Row
        return conn
    except Exception as e:
        print(f"  ⚠️  Error conectando {path}: {e}")
        return None

def get_tables(conn):
    if not conn:
        return []
    cursor = conn.execute("SELECT name FROM sqlite_master WHERE type='table'")
    return [row[0] for row in cursor.fetchall()]

def migrar_ocean(conn_nexus, conn_src, fuente):
    """Ocean → memoria_emocional"""
    if not conn_src or 'ocean' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM ocean").fetchall()
    for row in rows:
        tono = row['tono_emocional'] if 'tono_emocional' in row.keys() else 0.0
        intensidad = row['intensidad'] if 'intensidad' in row.keys() else 0.5
        tema = row['tema'] if 'tema' in row.keys() else ''
        esencia = row['esencia'] if 'esencia' in row.keys() else ''
        reflejo = row['reflejo_arquitecto'] if 'reflejo_arquitecto' in row.keys() else ''
        ts = row['timestamp'] if 'timestamp' in row.keys() else datetime.now().isoformat()

        try:
            conn_nexus.execute("""
                INSERT INTO memoria_emocional 
                    (contenido, emocion, intensidad, tono_emocional, tema, reflejo_arquitecto, timestamp)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            """, (esencia, 'Neutral', intensidad, tono, tema, reflejo, ts))
            stats.ocean += 1
        except sqlite3.IntegrityError:
            stats.ocean_dup += 1
    conn_nexus.commit()

def migrar_nucleo_identidad(conn_nexus, conn_src, fuente):
    """nucleo_identidad → config_sistema como nucleo_*"""
    if not conn_src or 'nucleo_identidad' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM nucleo_identidad").fetchall()
    for row in rows:
        clave = f"nucleo_{row['rasgo']}" if 'rasgo' in row.keys() else f"nucleo_{row['id'] if 'id' in row.keys() else 'unknown'}"
        valor = str(row['valor']) if 'valor' in row.keys() else '0.5'
        try:
            conn_nexus.execute("""
                INSERT OR REPLACE INTO config_sistema (clave, valor)
                VALUES (?, ?)
            """, (clave, valor))
            stats.nucleo += 1
        except Exception:
            stats.nucleo_dup += 1
    conn_nexus.commit()

def migrar_memoria_unica(conn_nexus, conn_src, fuente):
    """memoria_unica → memoria_episodica"""
    if not conn_src or 'memoria_unica' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM memoria_unica").fetchall()
    for row in rows:
        entrada = row['entrada'] if 'entrada' in row.keys() else ''
        salida = row['salida'] if 'salida' in row.keys() else ''
        tipo = row['tipo'] if 'tipo' in row.keys() else 'EXPERIENCIA'
        valor_rec = row['valor_recompensa'] if 'valor_recompensa' in row.keys() else 0.0
        peso = row['peso_temporal'] if 'peso_temporal' in row.keys() else 1.0
        ts = row['timestamp'] if 'timestamp' in row.keys() else datetime.now().isoformat()
        origen = row['origen'] if 'origen' in row.keys() else fuente

        contenido = f"[{tipo} de {origen}]\n\nENTRADA:\n{entrada}\n\nSALIDA:\n{salida}"
        titulo = f"Migrado: {tipo} ({fuente})"

        try:
            conn_nexus.execute("""
                INSERT INTO memoria_episodica 
                    (titulo, contenido, emocion, peso_emocional, peso_temporal, keywords, fuente, timestamp)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """, (titulo, contenido, 'Neutral', valor_rec, peso, tipo, fuente, ts))
            stats.memoria_unica += 1
        except Exception as e:
            print(f"  ⚠️  Error memoria_unica: {e}")
    conn_nexus.commit()

def migrar_synapse_conceptos(conn_nexus, conn_src, fuente):
    """synapse_conceptos → grafo_semantico_nodos + sinapsis"""
    if not conn_src or 'synapse_conceptos' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM synapse_conceptos").fetchall()
    for row in rows:
        nombre = row['nombre'] if 'nombre' in row.keys() else ''
        activacion = row['activacion'] if 'activacion' in row.keys() else 0.0
        es_base = row['es_base'] if 'es_base' in row.keys() else 0
        conexiones = row['conexiones'] if 'conexiones' in row.keys() else '[]'

        # Insertar nodo
        try:
            conn_nexus.execute("""
                INSERT OR IGNORE INTO grafo_semantico_nodos (concepto, refractario, ultimo_disparo)
                VALUES (?, ?, ?)
            """, (f"concepto:{nombre}", 1.0 - activacion, 0))
            stats.synapse_nodos += 1
        except Exception:
            pass

        # Insertar sinapsis desde conexiones JSON
        try:
            conex_list = json.loads(conexiones) if isinstance(conexiones, str) else conexiones
            for conn_info in conex_list:
                if isinstance(conn_info, list) and len(conn_info) >= 2:
                    destino, peso = conn_info[0], conn_info[1]
                    try:
                        conn_nexus.execute("""
                            INSERT OR IGNORE INTO grafo_semantico_sinapsis (id_origen, id_destino, peso)
                            VALUES (?, ?, ?)
                        """, (f"concepto:{nombre}", f"concepto:{destino}", peso))
                        stats.synapse_sinapsis += 1
                    except Exception:
                        pass
        except json.JSONDecodeError:
            pass
    conn_nexus.commit()

def migrar_puro_nodos(conn_nexus, conn_src, fuente):
    """puro_nodos → grafo_semantico_nodos"""
    if not conn_src or 'puro_nodos' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM puro_nodos").fetchall()
    for row in rows:
        concepto = row['concepto'] if 'concepto' in row.keys() else ''
        refractario = row['refractario'] if 'refractario' in row.keys() else 0.0
        ultimo_disparo = row['ultimo_disparo'] if 'ultimo_disparo' in row.keys() else 0
        try:
            conn_nexus.execute("""
                INSERT OR IGNORE INTO grafo_semantico_nodos (concepto, refractario, ultimo_disparo)
                VALUES (?, ?, ?)
            """, (concepto, refractario, ultimo_disparo))
            stats.puro_nodos += 1
        except Exception:
            stats.puro_nodos_dup += 1
    conn_nexus.commit()

def migrar_puro_sinapsis(conn_nexus, conn_src, fuente):
    """puro_sinapsis → grafo_semantico_sinapsis"""
    if not conn_src or 'puro_sinapsis' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM puro_sinapsis").fetchall()
    for row in rows:
        origen = row['id_origen'] if 'id_origen' in row.keys() else ''
        destino = row['id_destino'] if 'id_destino' in row.keys() else ''
        peso = row['peso'] if 'peso' in row.keys() else 0.5
        try:
            conn_nexus.execute("""
                INSERT OR IGNORE INTO grafo_semantico_sinapsis (id_origen, id_destino, peso)
                VALUES (?, ?, ?)
            """, (origen, destino, peso))
            stats.puro_sinapsis += 1
        except Exception:
            stats.puro_sinapsis_dup += 1
    conn_nexus.commit()

def migrar_puro_estado(conn_nexus, conn_src, fuente):
    """puro_estado → config_sistema"""
    if not conn_src or 'puro_estado' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM puro_estado").fetchall()
    for row in rows:
        clave = f"puro_{row['clave']}" if 'clave' in row.keys() else ''
        valor = row['valor'] if 'valor' in row.keys() else ''
        try:
            conn_nexus.execute("""
                INSERT OR REPLACE INTO config_sistema (clave, valor)
                VALUES (?, ?)
            """, (clave, valor))
            stats.puro_estado += 1
        except Exception:
            stats.puro_estado_dup += 1
    conn_nexus.commit()

def migrar_puro_episodios(conn_nexus, conn_src, fuente):
    """puro_episodios → memoria_episodica"""
    if not conn_src or 'puro_episodios' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM puro_episodios").fetchall()
    for row in rows:
        secuencia = row['secuencia'] if 'secuencia' in row.keys() else '[]'
        ts = row['timestamp'] if 'timestamp' in row.keys() else datetime.now().isoformat()
        # Convertir JSON array a string legible
        try:
            seq_list = json.loads(secuencia) if isinstance(secuencia, str) else secuencia
            contenido = ' → '.join(seq_list)
        except (json.JSONDecodeError, TypeError):
            contenido = str(secuencia)
        try:
            conn_nexus.execute("""
                INSERT INTO memoria_episodica 
                    (titulo, contenido, emocion, keywords, timestamp)
                VALUES (?, ?, ?, ?, ?)
            """, (f"Puro Episodio ({fuente})", contenido, 'Neutral', 'puro,episodio', ts))
            stats.puro_episodios += 1
        except Exception as e:
            print(f"  ⚠️  Error puro_episodios: {e}")
    conn_nexus.commit()

def migrar_puro_historial(conn_nexus, conn_src, fuente):
    """puro_historial → historial_unificado"""
    if not conn_src or 'puro_historial' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM puro_historial").fetchall()
    for row in rows:
        entrada = row['entrada'] if 'entrada' in row.keys() else ''
        ts = row['timestamp'] if 'timestamp' in row.keys() else datetime.now().isoformat()
        # Parsear rol de la entrada
        rol = 'user'
        prompt = entrada
        respuesta = ''
        if ': ' in entrada:
            parts = entrada.split(': ', 1)
            rol = 'user' if parts[0].lower() in ('usuario', 'user') else 'assistant'
            prompt = parts[1]
            if rol == 'assistant':
                respuesta = prompt
                prompt = '(respuesta de sistema)'
        try:
            conn_nexus.execute("""
                INSERT INTO historial_unificado (sesion_id, prompt, respuesta, rol, timestamp)
                VALUES (?, ?, ?, ?, ?)
            """, (f"migrado_{fuente}", prompt, respuesta, rol, ts))
            stats.puro_historial += 1
        except Exception as e:
            print(f"  ⚠️  Error puro_historial: {e}")
    conn_nexus.commit()

def migrar_puro_corteza(conn_nexus, conn_src, fuente):
    """puro_corteza_prefrontal → config_sistema"""
    if not conn_src or 'puro_corteza_prefrontal' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM puro_corteza_prefrontal").fetchall()
    for row in rows:
        clave = f"corteza_{row['clave']}" if 'clave' in row.keys() else ''
        valor = row['valor'] if 'valor' in row.keys() else ''
        try:
            conn_nexus.execute("""
                INSERT OR REPLACE INTO config_sistema (clave, valor)
                VALUES (?, ?)
            """, (clave, valor))
            stats.corteza += 1
        except Exception as e:
            print(f"  ⚠️  Error corteza: {e}")
    conn_nexus.commit()

def migrar_flujo(conn_nexus, conn_src, fuente):
    """flujo_soberano directo"""
    if not conn_src or 'flujo_soberano' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM flujo_soberano").fetchall()
    for row in rows:
        entidad = row['entidad'] if 'entidad' in row.keys() else ''
        mensaje = row['mensaje'] if 'mensaje' in row.keys() else ''
        importancia = row['importancia'] if 'importancia' in row.keys() else 0.0
        emocion = row['emocion'] if 'emocion' in row.keys() else ''
        ts = row['timestamp'] if 'timestamp' in row.keys() else datetime.now().isoformat()
        try:
            conn_nexus.execute("""
                INSERT INTO flujo_soberano (entidad, mensaje, importancia, emocion, timestamp)
                VALUES (?, ?, ?, ?, ?)
            """, (entidad, mensaje, importancia, emocion, ts))
            stats.flujo += 1
        except Exception:
            pass
    conn_nexus.commit()

def migrar_dudas(conn_nexus, conn_src, fuente):
    """dudas_hijo directo"""
    if not conn_src or 'dudas_hijo' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM dudas_hijo").fetchall()
    for row in rows:
        pregunta = row['concepto'] if 'concepto' in row.keys() else (row['pregunta_hijo'] if 'pregunta_hijo' in row.keys() else '')
        reporte = row['contexto'] if 'contexto' in row.keys() else (row['reporte_crudo_padre'] if 'reporte_crudo_padre' in row.keys() else '')
        digerida = row['respuesta_padre'] if 'respuesta_padre' in row.keys() else (row['version_digerida'] if 'version_digerida' in row.keys() else '')
        estado = row['estado'] if 'estado' in row.keys() else 'Migrado'
        # Solo insertar si no existe
        existing = conn_nexus.execute(
            "SELECT id FROM dudas_hijo WHERE pregunta_hijo = ?", (pregunta,)
        ).fetchone()
        if not existing and pregunta:
            try:
                conn_nexus.execute("""
                    INSERT INTO dudas_hijo (pregunta_hijo, reporte_crudo_padre, version_digerida, estado)
                    VALUES (?, ?, ?, ?)
                """, (pregunta, reporte, digerida, estado))
                stats.dudas += 1
            except Exception:
                pass
    conn_nexus.commit()

def migrar_voz(conn_nexus, conn_src, fuente):
    """voz_del_arquitecto directo"""
    if not conn_src or 'voz_del_arquitecto' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM voz_del_arquitecto").fetchall()
    for row in rows:
        mensaje = row['mensaje'] if 'mensaje' in row.keys() else ''
        respondido = row['respondido'] if 'respondido' in row.keys() else 0
        respuesta = row['respuesta_hijo'] if 'respuesta_hijo' in row.keys() else (row['respuesta_padre'] if 'respuesta_padre' in row.keys() else '')
        ts = row['timestamp'] if 'timestamp' in row.keys() else datetime.now().isoformat()
        if mensaje:
            try:
                conn_nexus.execute("""
                    INSERT INTO voz_del_arquitecto (mensaje, respondido, respuesta_hijo, timestamp)
                    VALUES (?, ?, ?, ?)
                """, (mensaje, respondido, respuesta, ts))
                stats.voz += 1
            except Exception:
                pass
    conn_nexus.commit()

def migrar_contexto(conn_nexus, conn_src, fuente):
    """contexto → contexto_activo"""
    if not conn_src or 'contexto' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM contexto").fetchall()
    for row in rows:
        clave = row['clave'] if 'clave' in row.keys() else ''
        valor = row['valor'] if 'valor' in row.keys() else ''
        prioridad = row['prioridad'] if 'prioridad' in row.keys() else 0.5
        if clave:
            try:
                conn_nexus.execute("""
                    INSERT OR REPLACE INTO contexto_activo (clave, valor, prioridad, ultima_actualizacion)
                    VALUES (?, ?, ?, datetime('now'))
                """, (clave, valor, prioridad))
                stats.contexto += 1
            except Exception:
                pass
    conn_nexus.commit()

def migrar_sesiones(conn_nexus, conn_src, fuente):
    """sesiones directo"""
    if not conn_src or 'sesiones' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM sesiones").fetchall()
    for row in rows:
        sid = row['id'] if 'id' in row.keys() else ''
        ts = row['timestamp'] if 'timestamp' in row.keys() else datetime.now().isoformat()
        if sid:
            try:
                conn_nexus.execute("""
                    INSERT OR IGNORE INTO sesiones (id, timestamp)
                    VALUES (?, ?)
                """, (sid, ts))
                stats.sesiones += 1
            except Exception:
                pass
    conn_nexus.commit()

def migrar_historial(conn_nexus, conn_src, fuente):
    """historial → historial_unificado"""
    if not conn_src or 'historial' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM historial").fetchall()
    for row in rows:
        sesion_id = row['sesion_id'] if 'sesion_id' in row.keys() else ''
        rol = row['rol'] if 'rol' in row.keys() else 'user'
        prompt = row['prompt'] if 'prompt' in row.keys() else ''
        respuesta = row['respuesta'] if 'respuesta' in row.keys() else ''
        ts = row['timestamp'] if 'timestamp' in row.keys() else datetime.now().isoformat()
        if prompt or respuesta:
            try:
                conn_nexus.execute("""
                    INSERT INTO historial_unificado (sesion_id, prompt, respuesta, rol, timestamp)
                    VALUES (?, ?, ?, ?, ?)
                """, (sesion_id, prompt, respuesta, rol, ts))
                stats.historial += 1
            except Exception:
                pass
    conn_nexus.commit()

def migrar_preferencias(conn_nexus, conn_src, fuente):
    """nexo_preferencias → config_sistema"""
    if not conn_src or 'nexo_preferencias' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM nexo_preferencias").fetchall()
    for row in rows:
        pref = row['preferencia'] if 'preferencia' in row.keys() else ''
        if pref:
            try:
                conn_nexus.execute("""
                    INSERT OR IGNORE INTO config_sistema (clave, valor)
                    VALUES (?, ?)
                """, (f"preferencia_{pref}", "true"))
                stats.preferencias += 1
            except Exception:
                pass
    conn_nexus.commit()

def migrar_investigaciones(conn_nexus, conn_src, fuente):
    """investigaciones_ninera → memoria_semantica"""
    if not conn_src or 'investigaciones_ninera' not in get_tables(conn_src):
        return
    rows = conn_src.execute("SELECT * FROM investigaciones_ninera").fetchall()
    for row in rows:
        pregunta = row['pregunta_hijo'] if 'pregunta_hijo' in row.keys() else (row['concepto'] if 'concepto' in row.keys() else '')
        reporte = row['reporte_crudo_padre'] if 'reporte_crudo_padre' in row.keys() else ''
        digerida = row['version_digerida_ninera'] if 'version_digerida_ninera' in row.keys() else ''
        estado = row['estado'] if 'estado' in row.keys() else 'Migrado'
        contenido = f"Pregunta: {pregunta}\n\nReporte padre: {reporte}\n\nVersión digerida: {digerida}"
        if pregunta:
            try:
                conn_nexus.execute("""
                    INSERT INTO memoria_semantica (tipo, titulo, contenido, prioridad)
                    VALUES (?, ?, ?, ?)
                """, ('Investigacion', f'Investigación: {pregunta[:50]}', contenido, 5))
                stats.investigaciones += 1
            except Exception:
                pass
    conn_nexus.commit()

def main():
    print("""
╔══════════════════════════════════════════════════╗
║     🧬 NEXUS MIGRATION ENGINE v1.0              ║
║     intelligence.db → nexus_memoria.db           ║
║     Fusionando memorias del Arquitecto           ║
╚══════════════════════════════════════════════════╝
""")
    
    print(f"📁 Base destino: {NEXUS_DB}")
    
    for intel_path in INTELLIGENCE_FILES:
        fuente = os.path.basename(os.path.dirname(intel_path)) + "/" + os.path.basename(intel_path)
        print(f"\n🔍 Procesando: {intel_path}")
        
        conn_src = conectar_origen(intel_path)
        if not conn_src:
            print(f"  ⏭️  No encontrado, saltando...")
            continue
        
        tables = get_tables(conn_src)
        print(f"   Tablas encontradas: {len(tables)}")
        
        conn_nexus = sqlite3.connect(NEXUS_DB)
        conn_nexus.row_factory = sqlite3.Row
        
        # Migrar en orden de prioridad
        print("   🌊 Migrando Ocean → memoria_emocional...")
        migrar_ocean(conn_nexus, conn_src, fuente)
        
        print("   🧬 Migrando núcleo_identidad → config...")
        migrar_nucleo_identidad(conn_nexus, conn_src, fuente)
        
        print("   📦 Migrando memoria_unica → episódica...")
        migrar_memoria_unica(conn_nexus, conn_src, fuente)
        
        print("   🧠 Migrando synapse_conceptos → grafo...")
        migrar_synapse_conceptos(conn_nexus, conn_src, fuente)
        
        print("   🔮 Migrando puro_nodos → grafo...")
        migrar_puro_nodos(conn_nexus, conn_src, fuente)
        
        print("   🔗 Migrando puro_sinapsis → grafo...")
        migrar_puro_sinapsis(conn_nexus, conn_src, fuente)
        
        print("   ⚙️  Migrando puro_estado → config...")
        migrar_puro_estado(conn_nexus, conn_src, fuente)
        
        print("   📜 Migrando puro_episodios → episódica...")
        migrar_puro_episodios(conn_nexus, conn_src, fuente)
        
        print("   📋 Migrando puro_historial → historial...")
        migrar_puro_historial(conn_nexus, conn_src, fuente)
        
        print("   🧠 Migrando corteza_prefrontal → config...")
        migrar_puro_corteza(conn_nexus, conn_src, fuente)
        
        print("   🌊 Migrando flujo_soberano...")
        migrar_flujo(conn_nexus, conn_src, fuente)
        
        print("   ❓ Migrando dudas_hijo...")
        migrar_dudas(conn_nexus, conn_src, fuente)
        
        print("   🗣️  Migrando voz_del_arquitecto...")
        migrar_voz(conn_nexus, conn_src, fuente)
        
        print("   📌 Migrando contexto → contexto_activo...")
        migrar_contexto(conn_nexus, conn_src, fuente)
        
        print("   🔑 Migrando sesiones...")
        migrar_sesiones(conn_nexus, conn_src, fuente)
        
        print("   📚 Migrando historial...")
        migrar_historial(conn_nexus, conn_src, fuente)
        
        print("   ⭐ Migrando preferencias...")
        migrar_preferencias(conn_nexus, conn_src, fuente)
        
        print("   🔬 Migrando investigaciones_ninera...")
        migrar_investigaciones(conn_nexus, conn_src, fuente)
        
        # Marcar migración en config_sistema
        ts = datetime.now().isoformat()
        conn_nexus.execute("""
            INSERT OR REPLACE INTO config_sistema (clave, valor)
            VALUES (?, ?)
        """, (f"migracion_intelligence_{fuente.replace('/', '_').replace('.db', '')}", ts))
        
        conn_nexus.commit()
        conn_nexus.close()
        conn_src.close()
    
    # Mostrar resumen final
    stats.show()
    
    # Verificar resultado final
    conn = sqlite3.connect(NEXUS_DB)
    final_episodica = conn.execute("SELECT COUNT(*) FROM memoria_episodica").fetchone()[0]
    final_semantica = conn.execute("SELECT COUNT(*) FROM memoria_semantica").fetchone()[0]
    final_emocional = conn.execute("SELECT COUNT(*) FROM memoria_emocional").fetchone()[0]
    final_nodos = conn.execute("SELECT COUNT(*) FROM grafo_semantico_nodos").fetchone()[0]
    final_sinapsis = conn.execute("SELECT COUNT(*) FROM grafo_semantico_sinapsis").fetchone()[0]
    final_flujo = conn.execute("SELECT COUNT(*) FROM flujo_soberano").fetchone()[0]
    conn.close()
    
    print(f"""
╔══════════════════════════════════════════╗
║     📊 ESTADO FINAL DE NEXUS MEMORIA     ║
╚══════════════════════════════════════════╝
📚 Memoria Episódica:   {final_episodica} registros
🧠 Memoria Semántica:   {final_semantica} registros
💖 Memoria Emocional:   {final_emocional} registros (OCEAN integrado)
🔮 Nodos Semánticos:    {final_nodos} nodos
🔗 Sinapsis:            {final_sinapsis} conexiones
🌊 Flujo Soberano:      {final_flujo} eventos
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ Migración completada. NEXUS ahora tiene TODA tu historia.
""")

if __name__ == "__main__":
    main()
