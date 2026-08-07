#!/bin/bash
# NEXUS SKILL: DEV OPTIMIZER v1.0
# Mejora el flujo de trabajo del desarrollador

echo "🛠️  Optimizando Ecosistema de Desarrollo Nexus..."

# 1. Alias Inteligentes (Agregados a .bashrc si no existen)
BASHRC="$HOME/.bashrc"
if ! grep -q "alias nweb" "$BASHRC"; then
    echo "  -> Agregando alias 'nweb' (Lanza Nexus Web Mode)"
    echo "alias nweb='cd $PWD && ./nexus_web.sh'" >> "$BASHRC"
else
    echo "  -> Alias 'nweb' ya existe."
fi

if ! grep -q "alias nkill" "$BASHRC"; then
    echo "  -> Agregando alias 'nkill' (Mata todo proceso Nexus)"
    echo "alias nkill='pkill -f nexus-server; pkill -f node; fuser -k 1420/tcp; fuser -k 1421/tcp'" >> "$BASHRC"
else
     echo "  -> Alias 'nkill' ya existe."
fi

# 2. VS Code Turbo Settings (Exclusiones de búsqueda)
VSCODE_SETTINGS="$PWD/.vscode/settings.json"
mkdir -p .vscode
if [ ! -f "$VSCODE_SETTINGS" ]; then
    echo "{}" > "$VSCODE_SETTINGS"
fi

# Inyectar exclusiones para que VS Code vuele
echo "  -> Configurando VS Code para ignorar carpetas pesadas..."
# (Esta es una inserción simple, en producción usaríamos jq)
cat > "$VSCODE_SETTINGS" <<EOF
{
    "css.lint.unknownAtRules": "ignore",
    "files.associations": { "*.css": "tailwindcss" },
    "search.exclude": {
        "**/node_modules": true,
        "**/target": true,
        "**/.git": true,
        "**/dist": true,
        "**/.tauri": true
    },
    "files.watcherExclude": {
        "**/target/**": true,
        "**/node_modules/**": true
    }
}
EOF

# 3. Limpieza de Temporales de Rust
echo "  -> Limpiando caché incremental corrupta (si existe)..."
rm -rf NEXUS_INTERFACE/src-tauri/target/debug/incremental

echo "✅ Optimización completada."
echo "ℹ️  Escribe 'source ~/.bashrc' para activar los neuvos comandos 'nweb' y 'nkill'."
