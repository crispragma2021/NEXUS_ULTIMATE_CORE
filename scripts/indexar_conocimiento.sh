#!/usr/bin/env bash
# ==========================================
# 📚 INDEXAR CONOCIMIENTO — NEXUS Omega
# ==========================================
# Trocea y registra reglas/skills/agentes/workflows/memoria en knowledge_base
# para búsqueda híbrida FTS5 + embeddings (Fase 0 de Carga Dinámica de Contexto)
#
# USO:
#   ./scripts/indexar_conocimiento.sh                # Indexa TODO
#   ./scripts/indexar_conocimiento.sh --dry-run      # Muestra qué se indexaría
#   ./scripts/indexar_conocimiento.sh --source rules # Solo reglas
# ==========================================

set -euo pipefail

NEXUS_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
NEXUS_MEMORIA_DB="$NEXUS_ROOT/memoria/nexus_memoria.db"
DRY_RUN=false
FILTER_SOURCE=""
CHUNK_COUNT=0
SKILL_COUNT=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run) DRY_RUN=true; shift ;;
        --source)  FILTER_SOURCE="$2"; shift 2 ;;
        *) echo "❌ Argumento desconocido: $1" >&2; exit 1 ;;
    esac
done

# ─── Util ──────────────────────────────────────────────
sql_escape() {
    sed "s/'/''/g"
}

sql_emit_chunk() {
    local source="$1"
    local category="$2"
    local section="$3"
    local content="$4"
    local priority="${5:-0}"

    local src_esc;    src_esc=$(printf '%s' "$source"   | sql_escape)
    local cat_esc;    cat_esc=$(printf '%s' "$category" | sql_escape)
    local sec_esc;    sec_esc=$(printf '%s' "$section"  | sql_escape)
    local cont_esc;   cont_esc=$(printf '%s' "$content" | sql_escape)

    printf "INSERT INTO knowledge_base(source,category,section,content,priority) VALUES('%s','%s','%s','%s',%d);\n" \
        "$src_esc" "$cat_esc" "$sec_esc" "$cont_esc" "$priority"
    ((CHUNK_COUNT++))
}

# ─── Trocear archivo por ## secciones ──────────────────
# Solo emite SQL INSERTs a stdout. Logs a stderr.
process_file() {
    local archivo="$1"
    local source_name="$2"
    local category="$3"
    local priority="${4:-0}"

    if [ ! -f "$archivo" ]; then
        echo "  ⚠️  No encontrado: $archivo" >&2
        return 1
    fi

    if [ "$DRY_RUN" = true ]; then
        echo "  📄 [$category] $source_name → $archivo" >&2
        return
    fi

    local content
    content=$(<"$archivo")
    local chunks_before=$CHUNK_COUNT

    # Sin ## headings → chunk único
    if ! echo "$content" | grep -q '^## '; then
        sql_emit_chunk "$source_name" "$category" "(completo)" "$content" "$priority"
        echo "  ✅ [$category] $source_name → $((CHUNK_COUNT - chunks_before)) chunks" >&2
        return
    fi

    local current_section=""
    local current_content=""
    local first=true

    while IFS= read -r line; do
        if echo "$line" | grep -q '^## '; then
            if [ "$first" = false ] && [ -n "$current_content" ]; then
                local sec_name
                sec_name=$(echo "$current_section" | sed 's/^## //;s/ *$//')
                sql_emit_chunk "$source_name" "$category" "$sec_name" "$current_content" "$priority"
            fi
            current_section="$line"
            current_content="$line"$'\n'
            first=false
        else
            current_content+="$line"$'\n'
        fi
    done <<< "$content"

    # Último chunk
    if [ -n "$current_content" ]; then
        local sec_name
        sec_name=$(echo "$current_section" | sed 's/^## //;s/ *$//')
        sql_emit_chunk "$source_name" "$category" "$sec_name" "$current_content" "$priority"
    fi

    echo "  ✅ [$category] $source_name → $((CHUNK_COUNT - chunks_before)) chunks" >&2
}

# ─── Skills masivas (920 skills) ───────────────────────
process_skills_masivas() {
    local skills_dir="$NEXUS_ROOT/.agent/skills/nexus_awesome_skills/skills"
    if [ ! -d "$skills_dir" ]; then
        echo "  ⚠️  Directorio de skills no encontrado: $skills_dir" >&2
        return
    fi

    if [ "$DRY_RUN" = true ]; then
        echo "  📄 [skills] nexus_awesome_skills/ → (920 skills, dry-run)" >&2
        return
    fi

    local chunks_before=$CHUNK_COUNT
    local processed=0

    for skill_dir in "$skills_dir"/*/; do
        [ -d "$skill_dir" ] || continue
        local skill_name; skill_name=$(basename "$skill_dir")
        local skill_file="$skill_dir/SKILL.md"

        if [ -f "$skill_file" ]; then
            local content; content=$(<"$skill_file")
            if echo "$content" | grep -q '^## '; then
                local current_section=""
                local current_content=""
                local first=true
                while IFS= read -r line; do
                    if echo "$line" | grep -q '^## '; then
                        if [ "$first" = false ] && [ -n "$current_content" ]; then
                            local sec_name
                            sec_name=$(echo "$current_section" | sed 's/^## //;s/ *$//')
                            sql_emit_chunk "$skill_name" "skills" "$sec_name" "$current_content" 0
                        fi
                        current_section="$line"
                        current_content="$line"$'\n'
                        first=false
                    else
                        current_content+="$line"$'\n'
                    fi
                done <<< "$content"
                if [ -n "$current_content" ]; then
                    local sec_name
                    sec_name=$(echo "$current_section" | sed 's/^## //;s/ *$//')
                    sql_emit_chunk "$skill_name" "skills" "$sec_name" "$current_content" 0
                fi
            else
                sql_emit_chunk "$skill_name" "skills" "(completo)" "$content" 0
            fi
        fi

        ((processed++))
        if ((processed % 100 == 0)); then
            echo "  ⏳ ... $processed/920 skills procesados ($((CHUNK_COUNT - chunks_before)) chunks)" >&2
        fi
    done

    echo "  ✅ [skills] nexus_awesome_skills → $((CHUNK_COUNT - chunks_before)) chunks ($processed skills)" >&2
}

# ═══════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════

echo "📚 NEXUS — Indexador de Conocimiento" >&2
echo "========================================" >&2
echo "🔍 DB: $NEXUS_MEMORIA_DB" >&2
[ "$DRY_RUN" = true ] && echo "🏃 Dry-run mode" >&2
[ -n "$FILTER_SOURCE" ] && echo "🎯 Filtrando por: $FILTER_SOURCE" >&2
echo "" >&2

if [ "$DRY_RUN" = true ]; then
    echo "📄 [rules] .clinerules" >&2
    echo "📄 [rules] .agent/rules/GEMINI.md" >&2
    echo "📄 [rules] nexus.md" >&2
    echo "📄 [memory] memoria/agente_memoria.md" >&2
    echo "📄 [memory] memoria/logros.md" >&2
    echo "📄 [skills] .agent/skills/doc.md" >&2
    echo "📄 [skills] .agent/skills/nexus_awesome_skills/skills/ (920 skills)" >&2
    echo "📄 [agents] .agent/agents/ (22 agentes)" >&2
    echo "📄 [workflows] .agent/workflows/ (12 workflows)" >&2
    echo "" >&2
    echo "🏁 Dry-run. Ejecuta sin --dry-run para indexar." >&2
    exit 0
fi

# === FASE 1: DDL ====================================
echo "🏗️  Creando tablas knowledge_base..." >&2
sqlite3 "$NEXUS_MEMORIA_DB" <<'SQL'
CREATE TABLE IF NOT EXISTS knowledge_base (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,
    category TEXT NOT NULL,
    section TEXT NOT NULL,
    content TEXT NOT NULL,
    embedding BLOB,
    priority INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_base_fts USING fts5(
    source, category, section, content,
    content=knowledge_base, content_rowid=id,
    tokenize='unicode61'
);

CREATE TRIGGER IF NOT EXISTS kb_ai AFTER INSERT ON knowledge_base BEGIN
    INSERT INTO knowledge_base_fts(rowid, source, category, section, content)
    VALUES (new.id, new.source, new.category, new.section, new.content);
END;

CREATE TRIGGER IF NOT EXISTS kb_ad AFTER DELETE ON knowledge_base BEGIN
    INSERT INTO knowledge_base_fts(knowledge_base_fts, rowid, source, category, section, content)
    VALUES ('delete', old.id, old.source, old.category, old.section, old.content);
END;

CREATE TRIGGER IF NOT EXISTS kb_au AFTER UPDATE ON knowledge_base BEGIN
    INSERT INTO knowledge_base_fts(knowledge_base_fts, rowid, source, category, section, content)
    VALUES ('delete', old.id, old.source, old.category, old.section, old.content);
    INSERT INTO knowledge_base_fts(rowid, source, category, section, content)
    VALUES (new.id, new.source, new.category, new.section, new.content);
END;
SQL
echo "✅ Tablas knowledge_base listas" >&2

# === FASE 2: Limpieza ================================
if [ -z "$FILTER_SOURCE" ]; then
    echo "🧹 Limpiando datos previos..." >&2
    sqlite3 "$NEXUS_MEMORIA_DB" "DELETE FROM knowledge_base;"
    echo "   Done." >&2
fi
echo "" >&2

# === FASE 3: Generar SQL y ejecutar en batch =========
echo "📦 Generando datos..." >&2

{
    echo "BEGIN TRANSACTION;"

    # ── REGLAS ──
    if [ -z "$FILTER_SOURCE" ] || [ "$FILTER_SOURCE" = "rules" ]; then
        echo "📜 INDEXANDO REGLAS..." >&2
        process_file "$NEXUS_ROOT/.clinerules" "clinerules" "rules" 2
        process_file "$NEXUS_ROOT/.agent/rules/GEMINI.md" "gemini_md" "rules" 2
        process_file "$NEXUS_ROOT/nexus.md" "nexus_md" "rules" 1
        echo "" >&2
    fi

    # ── MEMORIA ──
    if [ -z "$FILTER_SOURCE" ] || [ "$FILTER_SOURCE" = "memory" ]; then
        echo "🧠 INDEXANDO MEMORIA..." >&2
        process_file "$NEXUS_ROOT/memoria/agente_memoria.md" "agente_memoria" "memory" 1
        process_file "$NEXUS_ROOT/memoria/logros.md" "logros" "memory" 0
        echo "" >&2
    fi

    # ── SKILLS ──
    if [ -z "$FILTER_SOURCE" ] || [ "$FILTER_SOURCE" = "skills" ]; then
        echo "🎯 INDEXANDO SKILLS (doc.md)..." >&2
        process_file "$NEXUS_ROOT/.agent/skills/doc.md" "doc" "skills" 0
        echo "" >&2
        echo "⚡ INDEXANDO SKILLS MASIVAS (920)..." >&2
        process_skills_masivas
        echo "" >&2
    fi

    # ── AGENTES ──
    if [ -z "$FILTER_SOURCE" ] || [ "$FILTER_SOURCE" = "agents" ]; then
        echo "🤖 INDEXANDO AGENTES..." >&2
        for agent_file in "$NEXUS_ROOT/.agent/agents/"*.md; do
            [ -f "$agent_file" ] || continue
            local agent_name; agent_name=$(basename "$agent_file" .md)
            process_file "$agent_file" "$agent_name" "agents" 1
        done
        echo "" >&2
    fi

    # ── WORKFLOWS ──
    if [ -z "$FILTER_SOURCE" ] || [ "$FILTER_SOURCE" = "workflows" ]; then
        echo "🔄 INDEXANDO WORKFLOWS..." >&2
        for wf_file in "$NEXUS_ROOT/.agent/workflows/"*.md; do
            [ -f "$wf_file" ] || continue
            local wf_name; wf_name=$(basename "$wf_file" .md)
            process_file "$wf_file" "$wf_name" "workflows" 0
        done
        echo "" >&2
    fi

    echo "COMMIT;"
} | sqlite3 "$NEXUS_MEMORIA_DB"

echo "" >&2
echo "✅ $CHUNK_COUNT chunks insertados en knowledge_base" >&2

# === FASE 4: Estadísticas ============================
echo "" >&2
echo "📊 ESTADÍSTICAS FINALES:" >&2
sqlite3 "$NEXUS_MEMORIA_DB" <<'SQL'
SELECT 'Total chunks:' AS metric, CAST(COUNT(*) AS TEXT) AS value FROM knowledge_base
UNION ALL
SELECT '  Por categoría:', category || ' = ' || CAST(COUNT(*) AS TEXT) FROM knowledge_base GROUP BY category
ORDER BY metric;
SQL

echo "" >&2
echo "✅ Indexación completada." >&2
