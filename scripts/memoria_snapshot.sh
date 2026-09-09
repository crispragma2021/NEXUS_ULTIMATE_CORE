#!/bin/bash
# 🧠 NEXUS Memory Snapshot — Consolidación DB -> Contexto
# Genera agente_memoria.md dinámicamente desde SQLite FTS5
# v2: corregido — las consultas se EJECUTAN (dólares sin escapar) y
#     nombres de columnas reales (hito, contenido, titulo).

DB_PATH="/home/soberano/NEXUS_ULTIMATE_CORE/data/nexus_memoria.db"
OUTPUT="/home/soberano/NEXUS_ULTIMATE_CORE/memoria/agente_memoria.md"

echo "🧠 Generando snapshot de memoria desde la Base de Datos..."

cat > $OUTPUT << EOM
# 🔱 NEXUS AGENT MEMORY — Snapshot Dinámico
> Fuente de Verdad: nexus_memoria.db | Generado: $(date '+%Y-%m-%d %H:%M:%S')

## 🏛️ ESTADO DEL SISTEMA (Logros Recientes)
$(sqlite3 $DB_PATH "SELECT '- ' || hito || ' (' || date(timestamp) || ')' FROM logros ORDER BY timestamp DESC LIMIT 10;" 2>/dev/null)

## 🧬 CONOCIMIENTO TÉCNICO (Semántica)
$(sqlite3 $DB_PATH "SELECT '- ' || substr(contenido,1,150) FROM memoria_semantica ORDER BY peso_permanencia DESC LIMIT 15;" 2>/dev/null)

## 🕸️ ÚLTIMAS EXPERIENCIAS (Episódica)
$(sqlite3 $DB_PATH "SELECT '- ' || substr(titulo,1,100) FROM memoria_episodica ORDER BY timestamp DESC LIMIT 10;" 2>/dev/null)

## ⚖️ PROTOCOLOS JUDICIALES
- Corte Soberana Judicial: ACTIVA
- Motor Híbrido: Gemini + DeepSeek
- Backup: Vertex AI (project-26e94ab7-4257-4475-ade)

## 📡 RED Y ACCESO
- HUD Trading: http://localhost:5173 (Desktop Launcher)
- HUD Chat: http://localhost:1420 (Desktop Launcher)
- Túnel Cloudflare: Activo 24/7
EOM

echo "✅ Snapshot regenerado con éxito."
