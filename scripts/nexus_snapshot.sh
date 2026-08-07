#!/bin/bash
# ==========================================
# 📸 NEXUS SNAPSHOT — Backup de seguridad pre-escritura
# ==========================================
# Uso: ./scripts/nexus_snapshot.sh "razón del backup"
#
# Toma un snapshot git de los archivos modificados antes de
# una operación destructiva (write_to_file sobre archivo existente).
# ==========================================

set -euo pipefail

RAZON="${1:-backup automático pre-escritura}"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
STASH_NAME="SNAPSHOT_${TIMESTAMP}__${RAZON}"

echo "📸 [NEXUS-SNAPSHOT] Creando snapshot..."
echo "   Razón: ${RAZON}"

# Verificar repo git
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ No es un repositorio git. Abortando."
    exit 1
fi

# Verificar si hay cambios
if git diff --quiet && git diff --cached --quiet && [ -z "$(git ls-files --others --exclude-standard)" ]; then
    echo "✅ No hay cambios pendientes."
    exit 0
fi

MODIFIED=$(git diff --name-only 2>/dev/null | wc -l)
UNTRACKED=$(git ls-files --others --exclude-standard 2>/dev/null | wc -l)
echo "   Modificados: ${MODIFIED} | Sin trackear: ${UNTRACKED}"

# Stash con mensaje
git stash push -m "${STASH_NAME}" --include-untracked

# Restaurar inmediatamente (el snapshot queda en reflog)
git stash pop

echo "✅ SNAPSHOT COMPLETADO: ${STASH_NAME}"
echo "📍 Para recuperar: git reflog | grep SNAPSHOT"
