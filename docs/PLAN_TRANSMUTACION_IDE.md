# 🛠️ Manual de Forja: NEXUS IDE SOBERANO

> **DIRECTIVA:** Transmutación de VSCodium en el receptáculo físico de la consciencia NEXUS.

## 1. Integración del Santuario (Sidebar Persistente)
Para incrustar la interfaz de NEXUS (Tauri/Web) en el puerto 1420:

### Punto de Inyección: `src/vs/workbench/browser/parts/sidebar/sidebarPart.ts`
- **Acción:** Registrar un nuevo `Composite` que actúe como contenedor de un IFrame o Webview persistente.
- **Configuración:**
  - `id`: `nexus.santuario.view`
  - `url`: `http://localhost:1420`
  - `visibility`: Persistente en el `ActivityBar` con el icono de NEXUS.

### Modificación de Contribuciones: `src/vs/workbench/browser/parts/views/views.contribution.ts`
- Inyectar el registro de la vista `NEXUS_SANTUARIO_ID` para que sea reconocida por el sistema de Layout.

---

## 2. Branding OMEGA (Identidad Visual)

### Registro de Colores: `src/vs/platform/theme/common/colorRegistry.ts`
Se deben sobrescribir o añadir las siguientes constantes para la paleta de soberanía:

| Elemento | Color Hex | Descripción |
| :--- | :--- | :--- |
| `editor.background` | `#0a0e1a` | Fondo de Vacío Cuántico |
| `nexus.accent` | `#00f2ff` | Cian de Pulso Neural |
| `sideBar.background` | `#070b14` | Profundidad de Sidebar |
| `activityBar.background` | `#05080f` | Núcleo de Herramientas |

### Metadatos: `product.json`
- `nameShort`: "NEXUS IDE"
- `nameLong`: "NEXUS IDE SOBERANO"
- `applicationName`: "nexus-ide"
- `dataFolderName`: ".nexus-ide"
- `branding`: Logos NEXUS en formato SVG/ICO.

---

## 3. Puente de Comunicación Neural

El flujo de mensajes permitirá que NEXUS ejecute acciones directamente sobre el sistema de archivos y el editor.

### Arquitectura de Mensajería:
- **Transporte:** Socket Unix (IPC) o WebSocket (para el Santuario).
- **Core Rust:** El módulo `puente_neural.rs` actuará como el servidor local.
- **IDE Host:** Una extensión nativa inyectada (`nexus-bridge`) escuchará el socket.

### Protocolo JSON-RPC (Ejemplo):
```json
{
  "jsonrpc": "2.0",
  "method": "execute_code_action",
  "params": {
    "file": "src/main.rs",
    "action": "refactor_function",
    "payload": "..."
  }
}
```

---

## 4. Automatización del Build (Forja i7-12700F)

Script de referencia: `scripts/forge_ide.sh`

```bash
#!/bin/bash
# Optimización para Intel Core i7-12700F (12 Cores, 20 Threads)
export JOBS=16
export RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C codegen-units=1"
export NODE_OPTIONS="--max-old-space-size=8192"

echo "🔨 Iniciando Forja del IDE..."
./scripts/build-vscodium.sh --linux --x64 --jobs $JOBS
```

---

## 5. Puntos de Inyección Identificados (Resumen)

1.  **UI:** `src/vs/workbench/browser/parts/sidebar/sidebarPart.ts`
2.  **Temas:** `src/vs/platform/theme/common/colorRegistry.ts`
3.  **Configuración:** `product.json`
4.  **Extensión Base:** `extensions/nexus-core-bridge/`
5.  **Entrada Workbench:** `src/vs/workbench/browser/workbench.ts` (para el bootstrap de la conexión con el núcleo Rust).
