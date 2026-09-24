# 🔱 ARQUITECTURA DE AGENTES — NEXUS_ULTIMATE_CORE
> **Mapa canónico del ecosistema: UN CEREBRO, UN AGENTE SOBERANO UNIFICADO (`NEXUS-Agent`).**
> Actualizado: 2026-09-15 (Sesión de Consolidación)

---

## 🧠 EL CEREBRO CENTRAL (`core`)

| Pieza | Dónde | Estado |
|---|---|---|
| **Orquestador NEXUS** | `http://127.0.0.1:43210/v1` (`core` monolito) | ✅ Vivo, 46 órganos |
| **Puente MCP (`claws_mcp`)** | `~/.local/bin/claws_mcp` | ✅ Operativo (18 herramientas MCP) |
| **Ledger SQLite** | `data/nexus_ledger.db` | ✅ Sellado de operaciones físicas |

**REGLA DE ORO**: El Orquestador Central es el **CEREBRO DE ÓRGANOS** (memoria, tribunal dual, diagnóstico, telemetría) que expone sus servicios a través de **MCP** (`mcp_llamar`).

---

## 🤖 EL AGENTE SOBERANO CANÓNICO (`NEXUS-Agent`)

`NEXUS-Agent` es el **único agente oficial del sistema** (motor 100% Rust local):

* **Ubicación binario**: `~/.local/bin/nexus-agent`
* **Lanzador unificado**: `nexus` (`~/.local/bin/nexus`)
* **Datos y Estado**: `~/.local/share/nexus-agent/` (sesiones, tareas, estado.md)
* **Proveedores de LLM**: Multi-proveedor dinámico (`deepseek`, `ollama`, `openai`, `gemini`).

### 🦾 EFECTORES Y GARRAS INTEGRADAS EN `NEXUS-Agent`:

1. **`NexusClawPro` (Médula de Ejecución Física)**:
   - Inferencia local, firma de operaciones en Ledger, sigilo (`jitter`, rotación de MAC) y guardarraíl de riesgo/trauma en comandos.
2. **Navegador Web Unificado (`nexus_browser_engine.cjs`)**:
   - Motor Playwright Headless basado en Google Chrome nativo con soporte completo para navegación, clics, llenado de formularios, capturas de pantalla y extracción de datos SPA/JS.
3. **Control de Escritorio GUI (`os_control.rs`)**:
   - Captura de escritorio (`pantalla_ver`), clics en coordenadas $(X, Y)$ (`pantalla_clic`), inyección de texto (`pantalla_escribir`) y atajos de teclado (`pantalla_tecla`).
4. **Control Móvil (`movil_adb`)**:
   - Interacción nativa con dispositivos Android conectados vía ADB.
5. **Conexión al Cerebro Central (`mcp_llamar`)**:
   - Acceso total a los 46 órganos de memoria, tribunal y diagnóstico del monolito `core`.

---

## 📌 GUÍA DE USO RÁPIDO

Para lanzar cualquier tarea desde tu terminal:

```bash
# Ejecutar una consulta o comando con tu agente único:
nexus "calcula 10 + 10 en la calculadora"

# Navegar y capturar una página web:
nexus "navega a https://google.com y toma una captura"

# Sesión interactiva en vivo:
nexus
```
