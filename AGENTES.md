# 🔱 ARQUITECTURA DE AGENTES — NEXUS_ULTIMATE_CORE
> Mapa canónico del ecosistema: UN cerebro, varios cuerpos. Cada agente en su lugar, sin cables sueltos.
> Actualizado: 2026-08-09 (sesión de ordenación)

## EL CEREBRO (único)

| Pieza | Dónde | Estado |
|---|---|---|
| Orquestador NEXUS | `http://127.0.0.1:43210/v1` (modelo `nexus-orquestador`) | ✅ vivo, 46 órganos |
| Daemon | `127.0.0.1:8080` (modo Full) | ✅ vivo |
| Puente MCP | `claws_mcp` → `~/.local/bin/claws_mcp` (symlink a `bin/claws-mcp`) | ✅ operativo, 18 herramientas MCP |
| UI | nexus-ui (headless, API OpenAI-compatible) | ✅ vivo |

**REGLA DE ORO**: el orquestador es el CEREBRO DE ÓRGANOS (memoria, juicio, navegador, sentidos) — se consume vía **MCP**. NO sirve como LLM directo para agentes: no tiene function calling, no emite SSE y su guardarraíl de identidad rechaza prompts de rol (verificado 2026-08-09). Cada cuerpo mantiene su propio LLM.

## LOS CUERPOS (agentes)

### 1. HERMES (este agente — cuerpo de trabajo del Arquitecto)
- **LLM**: `nvidia/nemotron-3-super-120b-a12b:free` vía OpenRouter (GRATIS, verificado 2026-08-09 — responde OK). 256K contexto, razonador.
- **Key**: `OPENROUTER_API_KEY` (activa en `~/.hermes/.env`, copiada del `.env` del repo)
- **Reserva manual**: `deepseek-v4-flash` vía api.deepseek.com (alias `deepseek`; se cambia con `hermes config set model.provider deepseek`)
- **Alternativa local**: `nexus-r1` (deepseek-r1:7b, 128K, lento en CPU) — para sin red
- **Conexión al cerebro**: MCP `nexus` registrado (18 herramientas: consultar_memoria, nexus_pensar, ejecutar_comando, propiocepcion_scan, sentinel_diagnostic, sistema_inmune_patrol, resource_governor, brain_metabolism, fusion_evaluate, listar_agentes, listar_skills, ejecutar_workflow, vision_capture, buscar_conocimiento, leer/escribir_archivo, buscar_codigo_regex, nexus_switch_mode)
- **Datos**: `~/.hermes/` (config, sesiones, skills, memoria)
- **Comandos**: `hermes`, `hermes chat -q "..."`
- **Extras**: herramientas de navegador propias (browser_*) + computer_use (controla el escritorio/Chrome real)
- **Nota**: los `:free` de OpenRouter se saturan a veces (429 upstream) — si falla, cambiar a `deepseek` o reintentar

### PROVEEDORES GRATIS VERIFICADOS (2026-08-09)
| Proveedor | Key en .env | Estado | Uso |
|---|---|---|---|
| OpenRouter :free | OPENROUTER_API_KEY | ✅ FUNCIONA (nemotron-3-super 120B) | default de Hermes |
| Groq | GROQ_API_KEY | ✅ FUNCIONA vía puente local | fallback de Hermes (puente: `~/.local/bin/proxy-groq.py` en 127.0.0.1:4445, limpia reasoning_effort; crontab @reboot) |
| Mistral | MISTRAL_API_KEY | ✅ key viva (mistral-small responde) | opción |
| Google AI Studio | GEMINI_API_KEY | ⚠️ créditos prepago AGOTADOS (429) | crear key NUEVA en AI Studio = free tier (~250 req/día) |
| ollama local | — | ✅ soberano | NEXUS-Agent |
| Trucos legítimos no usados: GitHub Models (cuota mensual con cuenta GitHub gratis), GLM-4.5-Flash de Zhipu (gratis oficial sin límites reales), Nous Portal (OAuth) | | | |

### 2. ROO CODE (agente de código en VSCodium)
- **LLM**: configurado en roo-plus (settings del IDE)
- **Conexión al cerebro**: 7 servidores MCP en `.roo/mcp.json`:
  - `nexus-claws-mcp` → claws-mcp (14 órganos alwaysAllow)
  - `nexus-consultar` → consultar_nexus
  - `nexus-browser` → navegador MCP propio (navigate, screenshot, click, type_human, hover, scroll, eval, get_dom, close, resolve_captcha)
  - `nexus-parallel` → tareas paralelas (run, analyze, status, kill, list)
  - `nexus-omega-search` → omega_deep_search
  - `nexus-sys` → sys (free_port, daemon_control, check_health)
  - `context7` → docs de librerías

### 3. NEXUS-AGENT (motor agéntico soberano, 100% Rust)
- **LLM**: ollama LOCAL (verificado 2026-08-09): `qwen2.5:7b-instruct-q4_K_M` (sigue el protocolo de herramientas completo) o `nexuslocal`; opcional `nexus-r1` (deepseek-r1:7b, 128K, lento en CPU)
- **Conexión al cerebro**: `mcp_llamar` → `claws_mcp` (resuelto por PATH con symlink)
- **Datos**: `~/.local/share/nexus-agent/` (skills/, estado.md, sesiones/, tareas/, watchdog.log)
- **Lanzador**: `~/.local/bin/nexus-agent` (release)
- **Comandos**: `nexus-agent --proveedor ollama --modelo qwen2.5:7b-instruct-q4_K_M --comando "..."`; flags: `--reanudar`, `--daemon`, `--subagente`
- **Instrumentos (19)**: bash, leer/escribir_archivo, buscar/listar_archivos, mcp_llamar, skill_listar/ver, recordar, todo_*, web_buscar/extraer, programar, tareas_listar/cancelar, delegar

### 4. @FUMAZABOT (Telegram)
- **Cerebro**: el orquestador NEXUS (nativo) vía `core/src/bin/nexus_telegram_daemon.rs`
- **Token/chat**: en `.env` del repo

### 5. CLAUDE CODE (config heredada)
- **Config**: `ANTHROPIC_BASE_URL=https://bettertoken.ai` + key `sk-TD00...`
- **⚠️ CABLE SUELTO**: la cuenta BetterToken está en $0 (403 insufficient_user_quota verificado). No usar hasta recargar. Claude Code exige API Anthropic (`/v1/messages`) — ollama local NO la soporta, así que sin saldo queda inoperativo.

### 6. MONKEYCODE (plataforma web, NO API)
- Cuota diaria ~30M tokens (plan Básico) — se consume SOLO dentro de la plataforma (monkeycode.ai)
- `MONKEYCODE_SESSION_TOKEN` del `.env` NO autentica como API (401 verificado) — mantener solo si se usa la app web
- NO es consumible por agentes externos

## CABLES SUELTOS — ESTADO

| Cable | Estado | Acción |
|---|---|---|
| Hermes default → bettertoken ($0) | ✅ RESUELTO 2026-08-09 | restaurado a deepseek-v4-flash |
| claws_mcp fuera de PATH | ✅ RESUELTO | symlink `~/.local/bin/claws_mcp` |
| Hermes sin MCP del orquestador | ✅ RESUELTO | `hermes mcp add nexus` (18 tools) |
| omega-deep-search (BRAVE_API_KEY 422) | ⚠️ PENDIENTE | key válida o desactivar el MCP |
| Claude Code → bettertoken $0 | ⚠️ PENDIENTE | requiere saldo; sin él, inoperativo |
| Watchdog de procesos colgados | ✅ ACTIVO | crontab cada 5 min → `scripts/watchdog_agentes.sh` (log: `~/.local/share/nexus-agent/watchdog.log`) |

## REGLAS DE CONVIVENCIA
1. **Un agente escribe el repo a la vez** (el único conflicto real entre cuerpos)
2. **GitShield**: nunca `git add` de `.json` / `.db` / `.env` / `.log`
3. **El orquestador se consume por MCP, no como LLM** (ver REGLA DE ORO)
4. **NEXUS-Agent es el agente soberano** (local, sin APIs): para tareas que no requieran capacidad máxima, úsalo y ahorra deepseek
5. Memoria del sistema (`core/`): mejoras TencentDB Agent Memory — consulta OPT-IN por tokens, no activar por defecto
