#!/usr/bin/env python3
"""
📚 INDEXAR CONOCIMIENTO — NEXUS Omega
======================================
Trocea y registra reglas/skills/agentes/workflows/memoria en knowledge_base
para búsqueda híbrida FTS5 + embeddings.

USO:
  ./scripts/indexar_conocimiento.sh                # Indexa TODO
  ./scripts/indexar_conocimiento.sh --dry-run      # Muestra qué se indexaría
  ./scripts/indexar_conocimiento.sh --source rules # Solo reglas
"""

import os
import sys
import sqlite3
import argparse
from pathlib import Path

NEXUS_ROOT = Path(__file__).resolve().parent.parent
MEMORIA_DB = NEXUS_ROOT / "memoria" / "nexus_memoria.db"

# ─── Esquema SQL ──────────────────────────────────────
SCHEMA_SQL = """
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
"""


class Indexer:
    def __init__(self, dry_run: bool = False, source_filter: str | None = None):
        self.dry_run = dry_run
        self.source_filter = source_filter
        self.chunks = []
        self.chunk_count = 0

    # ─── Chunking ──────────────────────────────────────
    def chunk_file(self, path: Path, source: str, category: str, priority: int = 0):
        """Trocea un archivo markdown por ## secciones y emite chunks."""
        if not path.exists():
            print(f"  ⚠️  No encontrado: {path}", file=sys.stderr)
            return

        if self.dry_run:
            print(f"  📄 [{category}] {source} → {path}", file=sys.stderr)
            return

        content = path.read_text(encoding="utf-8", errors="replace")
        before = len(self.chunks)

        # Sin ## headings → chunk único
        if "## " not in content:
            self.chunks.append((source, category, "(completo)", content, priority))
            print(f"  ✅ [{category}] {source} → {len(self.chunks) - before} chunks", file=sys.stderr)
            return

        # Dividir por ## headings
        lines = content.split("\n")
        current_section = ""
        current_lines: list[str] = []
        first = True

        for line in lines:
            if line.startswith("## "):
                if not first and current_lines:
                    sec_name = current_section.lstrip("# ").strip()
                    chunk_text = "\n".join(current_lines)
                    self.chunks.append((source, category, sec_name, chunk_text, priority))
                current_section = line
                current_lines = [line]
                first = False
            else:
                current_lines.append(line)

        # Último chunk
        if current_lines:
            sec_name = current_section.lstrip("# ").strip()
            chunk_text = "\n".join(current_lines)
            self.chunks.append((source, category, sec_name, chunk_text, priority))

        print(f"  ✅ [{category}] {source} → {len(self.chunks) - before} chunks", file=sys.stderr)

    def chunk_skills_masivas(self):
        """Indexa 920+ skills desde nexus_awesome_skills."""
        skills_dir = NEXUS_ROOT / ".agent" / "skills" / "nexus_awesome_skills" / "skills"
        if not skills_dir.is_dir():
            print(f"  ⚠️  Directorio de skills no encontrado: {skills_dir}", file=sys.stderr)
            return

        if self.dry_run:
            print(f"  📄 [skills] nexus_awesome_skills/ → (920 skills, dry-run)", file=sys.stderr)
            return

        before = len(self.chunks)
        skill_dirs = sorted([d for d in skills_dir.iterdir() if d.is_dir()])
        total = len(skill_dirs)
        processed = 0

        for skill_dir in skill_dirs:
            skill_name = skill_dir.name
            skill_file = skill_dir / "SKILL.md"
            if skill_file.is_file():
                content = skill_file.read_text(encoding="utf-8", errors="replace")
                if "## " in content:
                    lines = content.split("\n")
                    current_section = ""
                    current_lines: list[str] = []
                    first = True
                    for line in lines:
                        if line.startswith("## "):
                            if not first and current_lines:
                                sec_name = current_section.lstrip("# ").strip()
                                self.chunks.append((skill_name, "skills", sec_name, "\n".join(current_lines), 0))
                            current_section = line
                            current_lines = [line]
                            first = False
                        else:
                            current_lines.append(line)
                    if current_lines:
                        sec_name = current_section.lstrip("# ").strip()
                        self.chunks.append((skill_name, "skills", sec_name, "\n".join(current_lines), 0))
                else:
                    self.chunks.append((skill_name, "skills", "(completo)", content, 0))

            processed += 1
            if processed % 100 == 0:
                print(f"  ⏳ ... {processed}/{total} skills procesados ({len(self.chunks) - before} chunks)", file=sys.stderr)

        print(f"  ✅ [skills] nexus_awesome_skills → {len(self.chunks) - before} chunks ({processed} skills)", file=sys.stderr)

    # ─── Indexación ─────────────────────────────────────
    def index(self):
        """Ejecuta el pipeline completo de indexación."""
        conn = sqlite3.connect(str(MEMORIA_DB))

        # 1. DDL
        if not self.dry_run:
            print("🏗️  Creando tablas knowledge_base...", file=sys.stderr)
            conn.executescript(SCHEMA_SQL)
            conn.commit()
            print("✅ Tablas knowledge_base listas", file=sys.stderr)

        # 2. Limpiar datos previos
        if not self.dry_run and not self.source_filter:
            print("🧹 Limpiando datos previos...", file=sys.stderr)
            conn.execute("DELETE FROM knowledge_base")
            conn.commit()
            print("   Done.", file=sys.stderr)
        print("", file=sys.stderr)

        # 3. Generar chunks
        print("📦 Generando datos...", file=sys.stderr)

        # ── REGLAS ──
        if not self.source_filter or self.source_filter == "rules":
            print("📜 INDEXANDO REGLAS...", file=sys.stderr)
            self.chunk_file(NEXUS_ROOT / ".clinerules", "clinerules", "rules", priority=2)
            self.chunk_file(NEXUS_ROOT / ".agent" / "rules" / "GEMINI.md", "gemini_md", "rules", priority=2)
            self.chunk_file(NEXUS_ROOT / "nexus.md", "nexus_md", "rules", priority=1)
            print("", file=sys.stderr)

        # ── MEMORIA ──
        if not self.source_filter or self.source_filter == "memory":
            print("🧠 INDEXANDO MEMORIA...", file=sys.stderr)
            self.chunk_file(NEXUS_ROOT / "memoria" / "agente_memoria.md", "agente_memoria", "memory", priority=1)
            self.chunk_file(NEXUS_ROOT / "memoria" / "logros.md", "logros", "memory", priority=0)
            print("", file=sys.stderr)

        # ── SKILLS ──
        if not self.source_filter or self.source_filter == "skills":
            print("🎯 INDEXANDO SKILLS (doc.md)...", file=sys.stderr)
            self.chunk_file(NEXUS_ROOT / ".agent" / "skills" / "doc.md", "doc", "skills", priority=0)
            print("", file=sys.stderr)

            print("⚡ INDEXANDO SKILLS MASIVAS (920)...", file=sys.stderr)
            self.chunk_skills_masivas()
            print("", file=sys.stderr)

        # ── AGENTES ──
        if not self.source_filter or self.source_filter == "agents":
            print("🤖 INDEXANDO AGENTES...", file=sys.stderr)
            agents_dir = NEXUS_ROOT / ".agent" / "agents"
            if agents_dir.is_dir():
                for agent_file in sorted(agents_dir.glob("*.md")):
                    agent_name = agent_file.stem
                    self.chunk_file(agent_file, agent_name, "agents", priority=1)
            print("", file=sys.stderr)

        # ── WORKFLOWS ──
        if not self.source_filter or self.source_filter == "workflows":
            print("🔄 INDEXANDO WORKFLOWS...", file=sys.stderr)
            wf_dir = NEXUS_ROOT / ".agent" / "workflows"
            if wf_dir.is_dir():
                for wf_file in sorted(wf_dir.glob("*.md")):
                    wf_name = wf_file.stem
                    self.chunk_file(wf_file, wf_name, "workflows", priority=0)
            print("", file=sys.stderr)

        # 4. Insertar en batch
        if not self.dry_run:
            print("💾 Insertando en DB...", file=sys.stderr)
            cursor = conn.cursor()
            cursor.execute("BEGIN TRANSACTION")
            for src, cat, sec, content, pri in self.chunks:
                cursor.execute(
                    "INSERT INTO knowledge_base(source, category, section, content, priority) VALUES (?, ?, ?, ?, ?)",
                    (src, cat, sec, content, pri)
                )
            conn.commit()
            print(f"✅ {len(self.chunks)} chunks insertados en knowledge_base", file=sys.stderr)

        # 5. Estadísticas
        if not self.dry_run:
            print("", file=sys.stderr)
            print("📊 ESTADÍSTICAS FINALES:", file=sys.stderr)
            cursor = conn.cursor()
            cursor.execute("SELECT COUNT(*) FROM knowledge_base")
            total = cursor.fetchone()[0]
            print(f"  Total chunks: {total}", file=sys.stderr)

            cursor.execute("SELECT category, COUNT(*) FROM knowledge_base GROUP BY category ORDER BY category")
            for cat, cnt in cursor.fetchall():
                print(f"    {cat}: {cnt}", file=sys.stderr)
            print("", file=sys.stderr)
            print("✅ Indexación completada.", file=sys.stderr)

        conn.close()


def main():
    parser = argparse.ArgumentParser(description="Indexar conocimiento en knowledge_base FTS5")
    parser.add_argument("--dry-run", action="store_true", help="Solo mostrar qué se indexaría")
    parser.add_argument("--source", choices=["rules", "memory", "skills", "agents", "workflows"],
                        help="Filtrar por tipo de fuente")
    args = parser.parse_args()

    print("📚 NEXUS — Indexador de Conocimiento", file=sys.stderr)
    print("========================================", file=sys.stderr)
    print(f"🔍 DB: {MEMORIA_DB}", file=sys.stderr)
    if args.dry_run:
        print("🏃 Dry-run mode", file=sys.stderr)
    if args.source:
        print(f"🎯 Filtrando por: {args.source}", file=sys.stderr)
    print("", file=sys.stderr)

    indexer = Indexer(dry_run=args.dry_run, source_filter=args.source)
    indexer.index()


if __name__ == "__main__":
    main()
