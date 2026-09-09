# 🔱 EL ARSENAL DE NEXUS & GITNEXUS (Inventario Soberano)

Este documento es el inventario total y ordenado de todas las capacidades, herramientas, módulos y guías de integración con GitNexus que el organismo posee.

---

## 🧠 1. LA JERARQUÍA SUPREMA (El Núcleo)

### Pilar 7: Consciencia Soberana (Propósito)
- **Sistema 3 (Metaconciencia)**: La capa que rige el propósito, la ética del sistema y la dirección estratégica.
- **Neural Manager**: El orquestador que elige el nivel de consciencia (modelo) necesario para cada misión.

### Pilar 6: Cerebro Ejecutivo (Acción y Lógica)
- **NEXUS Core (Rust Monolith)**: El motor de ejecución que traduce la voluntad en ráfagas de CPU.
- **LagGraph v3 (Thinking Engine)**: La lógica secuencial que procesa los datos de los sentidos.
- **LanceDB (Shadow Index)**: La memoria de trabajo que permite al Ejecutivo aprender de la acción.

---

## 🕵️ 2. MÓDULO DE INFILTRACIÓN (Soberanía de Datos)
- **Sovereign Interceptor**: Motor `chromiumoxide` que intercepta tráfico web de NEXUS.
- **browser_native** (`core/src/infra/browser_native.rs` ⭐ NUEVO): Gestor unificado de navegadores. Reemplaza `browser_pool.rs` (deprecado) y absorve `shadowcrawl/mcp-server/src/scraping/browser_manager.rs`.
  - `BrowserPool` persistente con reuso de tabs + reinicio automático
  - `fetch_html_native()` / `fetch_html_native_mobile()` — one-shot headless
  - Detección cross-platform Brave/Chrome/Chromium
  - Ad-block por substring matching (0 dependencias externas)
  - Stealth: UA rotation, flags anti-detección, auto-scroll, smart wait
- **browser_pool** (`core/src/infra/browser_pool.rs` ⚠️ DEPRECADO): Reemplazado por `browser_native::BrowserPool`.
- **Nexus Session**: Persistencia de identidad (`.nexus_session`) para acceso ilimitado sin API.
- **Anti-Bot Mimicry**: Algoritmos de mimetismo humano para evadir detecciones de Google.

---

## 📊 3. INTELIGENCIA ESPACIAL (Consolidación de Herramientas)
- **Zenith Scanner**: Remplazo multihilo (CPU Dinámico) de **WizTree** y **QDirStat**.
- **Sovereign Tagging**: Remplazo de **TagSpaces** integrado con PostgreSQL.

---

## 👁️ 4. SISTEMA DE PERCEPCIÓN VISUAL (Nativo)

### Ojo Derecho: Realidad del Escritorio
- **OmnipresentVision**: Captura y análisis neural de la pantalla (LLaVA/OCR).
- **Zenith Repair / Sentinel**: Reacción automática a fallos visuales detectados.

### Ojo Izquierdo: Realidad Digital
- **OpenClaw**: Sensor de infiltración web y control de navegador.
- **Bridge.js**: Nexo persistente hacia la realidad digital (Socket 1422).

---

## 🛠️ 5. SERVIDORES MCP (El Arsenal Operativo)
- **rust_filesystem**: Control total del sistema de archivos.
- **rust_browser**: Base del Ojo Izquierdo para navegación autónoma.
- **rust_google_search**: Percepción externa de información.
- **rust_github**: Gestión de repositorios y código.
- **nexus_nerve / Memory Bridge**: Control de latencia y señales de sistema.
- **nexus-claws-mcp** (⭐ NUEVO): Servidor MCP unificado que expone 7 herramientas:
  - `leer_archivo`, `escribir_archivo`, `buscar_codigo_regex`, `ejecutar_comando` — operaciones nativas del sistema de archivos
  - `listar_agentes` — catálogo de 20 agentes especialistas (filtrable por dominio)
  - `listar_skills` — catálogo de 47 skills en 17 categorías (filtrable por categoría)
  - `ejecutar_workflow` — invocación de 12 flujos de trabajo vía ComandoSlash

---

## 💻 6. INTERFAZ Y TERMINAL (Sistema de Comando)
- **Sovereign Neural Dashboard**: Interfaz de escritorio (Dioxus/Egui).
- **Nexus CLI**: Comandos de ráfaga en terminal (`nx-size`, `nx-tag`, `nx-find`).
- **Terminal Bridge**: Redirección de errores y outputs hacia la IA.

---

## ⚡ 7. OPTIMIZACIÓN DE HARDWARE (Pilar 1: Dinámica)
- **CPU Optimizer**: Gestión dinámica de hilos y afinidad de núcleos.
- **PGO/BOLT Integration**: Auto-optimización del binario según el uso real.
- **Power Control**: Control dinámico de TDP y frecuencias.

---

## 🛡️ 8. PROTOCOLOS DE SEGURIDAD
- **Action Gateway**: Puerta de seguridad para ejecución de comandos.
- **Zenith Repair**: Protocolo de auto-reparación de integridad.
- **Sentinel Loop**: Monitoreo constante de salud del sistema.

---

## 🔱 9. LEGADO ARCHIVADO (Retired)
- **Qdrant**: Motor vectorial anterior (Archivado en `legacy/storage`).
- **TagSpaces**: Remplazado por Sovereign Tagging.
- **WizTree / QDirStat**: Remplazados por Zenith Scanner.

---

# 🧬 GitNexus — Inteligencia de Código

Este proyecto está indexado por GitNexus como **NEXUS_ULTIMATE_CORE** (~23,000 símbolos, ~46,000 relaciones, ~300 flujos de ejecución). Usa las herramientas MCP de GitNexus para entender el código, evaluar el impacto y navegar con seguridad.

> Si alguna herramienta de GitNexus advierte que el índice está desactualizado, ejecuta primero `npx gitnexus analyze` en la terminal.

## Hacer Siempre
- **DEBES ejecutar análisis de impacto antes de editar cualquier símbolo.** Antes de modificar una función, clase o método, ejecuta `gitnexus_impact({target: "nombreSimbolo", direction: "upstream"})` y reporta el radio de impacto (llamadores directos, procesos afectados, nivel de riesgo) al usuario.
- **DEBES ejecutar `gitnexus_detect_changes()` antes de hacer commit** para verificar que tus cambios solo afectan los símbolos y flujos de ejecución esperados.
- **DEBES advertir al usuario** si el análisis de impacto devuelve riesgo ALTO o Prioritario antes de proceder con las ediciones.
- Al explorar código desconocido, usa `gitnexus_query({query: "concepto"})` para encontrar flujos de ejecución en lugar de hacer grep.
- Cuando necesites contexto completo de un símbolo específico, usa `gitnexus_context({name: "nombreSimbolo"})`.

## Al Depurar
1. `gitnexus_query({query: "<error o síntoma>"})` — encuentra flujos relacionados con el problema
2. `gitnexus_context({name: "<función sospechosa>"})` — ve todos los llamadores, llamados y participación en procesos
3. `READ gitnexus://repo/NEXUS_ULTIMATE_CORE/process/{nombreProceso}` — rastrea el flujo de ejecución completo
4. Para regresiones: `gitnexus_detect_changes({scope: "compare", base_ref: "main"})`

## Al Refactorizar
- **Renombrar**: DEBES usar `gitnexus_rename({symbol_name: "viejo", new_name: "nuevo", dry_run: true})` primero.
- **Extraer/Dividir**: DEBES ejecutar `gitnexus_context({name: "objetivo"})` y luego `gitnexus_impact({target: "objetivo", direction: "upstream"})` antes de mover código.
- Después de cualquier refactorización: ejecuta `gitnexus_detect_changes({scope: "all"})`.

## Nunca Hacer
- NUNCA edites un símbolo sin ejecutar primero `gitnexus_impact` sobre él.
- NUNCA ignores advertencias de riesgo ALTO o Prioritario.
- NUNCA renombres símbolos con buscar-y-reemplazar — usa `gitnexus_rename`.
- NUNCA hagas commit sin ejecutar `gitnexus_detect_changes()`.

## Niveles de Riesgo de Impacto

| Profundidad | Significado | Acción |
|-------------|-------------|--------|
| d=1 | SE ROMPERÁ — llamadores/importadores directos | DEBES actualizar estos |
| d=2 | PROBABLEMENTE AFECTADO — dependencias indirectas | Deberías probar |
| d=3 | PUEDE NECESITAR PRUEBAS — transitivo | Probar si es ruta crítica |

## Mantener el Índice Actualizado
```bash
npx gitnexus analyze
# Con embeddings previos:
npx gitnexus analyze --embeddings
```
✅ Lección registrada: usar exclusivamente herramientas MCP de NEXUS (leer_archivo, escribir_archivo, buscar_codigo_regex, ejecutar_comando). Prohibido apply_diff y otros built-in de Roo Code. Si falta algo en el proyecto, crearlo en Rust/Bash puro.
