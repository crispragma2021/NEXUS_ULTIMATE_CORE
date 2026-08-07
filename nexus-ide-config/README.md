# 🧬 Nexus IDE Config — ADN de configuración del agente

> **Origen:** copias versionadas de las configuraciones globales que viven en `~/.config/` del usuario `soberano`.
> Estas configuraciones NO forman parte del repo original porque están fuera de la carpeta del proyecto,
> pero son parte del ADN de NEXUS: definen los modos, modelos y tools MCP del agente.

## Contenido

| Carpeta | IDE | Extensión | Archivos |
|---|---|---|---|
| `VSCodium-roo-plus/` | VSCodium | Roo Plus (xavier-arosemena) | `custom_modes.yaml` (1.2 MB — activo), `mcp_settings.json` |
| `VSCodium-roo-cline/` | VSCodium | Roo Cline (rooveterinaryinc) | `custom_modes.yaml`, `mcp_settings.json` |
| `Code-roo-cline/` | VS Code | Roo Cline (rooveterinaryinc) | `custom_modes.yaml`, `mcp_settings.json` |
| `Antigravity-roo-cline/` | Antigravity IDE | Roo Cline (rooveterinaryinc) | `custom_modes.yaml`, `mcp_settings.json` |

## Cómo restaurar en una máquina nueva

```bash
# Ejemplo: restaurar la config activa de VSCodium (Roo Plus)
mkdir -p ~/.config/VSCodium/User/globalStorage/xavier-arosemena.roo-plus/settings
cp nexus-ide-config/VSCodium-roo-plus/custom_modes.yaml ~/.config/VSCodium/User/globalStorage/xavier-arosemena.roo-plus/settings/
cp nexus-ide-config/VSCodium-roo-plus/mcp_settings.json ~/.config/VSCodium/User/globalStorage/xavier-arosemena.roo-plus/settings/
```

## Seguridad

- Los `custom_modes.yaml` **no contienen claves API reales** (las coincidencias de `api_key` son solo ejemplos de código dentro de prompts).
- Los `mcp_settings.json` pueden referenciar binarios locales (`claws-mcp`, etc.) que se reconstruyen con `cargo build`.
- Los modelos de Ollama (`*.gguf`, `.ollama/`) NO se versionan: se descargan en cada máquina.
