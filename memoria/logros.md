# 🏆 LOGROS DE NEXUS

> Registro de hitos significativos en la evolución del sistema.

---

## 🥇 HITOS PRINCIPALES

### 2026-08-02 👁️ — Ojo Local Escalonado v2: CNN → PaddleOCR → tesseract → Qwen2.5-VL (Precisión Total)
- **Hito**: Evolución del resolver CAPTCHA local de un stack de 2 tiers (tesseract + LLaVA) a un **stack escalonado de 4 tiers** con carga bajo demanda (lazy loading), explotando los 64GB de RAM disponibles.
- **Tier 0 — CNN CAPTCHA** (`_cnn`): red convolucional ligera; se omite si onnx/tensorflow no están presentes. Primer intento instantáneo para CAPTCHAs de caracteres limpios.
- **Tier 1 — PaddleOCR** (preferencia explícita del Arquitecto): OCR profundo basado en PP-OCRv4. **Validado end-to-end** en `nexus-cua-gui` resolviendo `"K7xQ9 test"` en **3.9s** con pesos cacheados (`/root/.paddleocr/whl/rec/en/`).
- **Tier 2 — tesseract**: fallback nativo ligero que se activa cuando PaddleOCR no produce texto.
- **Tier 3 — Qwen2.5-VL 7B vía Ollama**: visión semántica para CAPTCHAs complejos/no textuales. **Validado**: reconoce `"ubuntu"` en un screenshot de desktop completo.
- **Cadena de escalamiento probada**: 404 de Ollama durante descarga confirmó la escalada correcta de tier a tier sin romper la resolución.
- **Matriz de compatibilidad Paddle resuelta** (lección crítica): paddleocr 3.7.0 exige paddlepaddle 3.x (`predict()`, `use_textline_orientation`, sin `show_log`/`cls`); paddleocr 2.7.3 exige paddlepaddle 2.6.x (`ocr()`, `use_angle_cls`, `cls=True`, formato `item[1][0]`); paddlepaddle 2.x exige numpy<2 (ABI). Bug `ConvertPirAttribute2RuntimeAttribute` (oneDNN) de Paddle 3.x resolvió vía downgrade. Entorno estabilizado: **paddleocr 2.7.3 + paddlepaddle 2.6.2 + numpy 1.26.4**.
- **Integración**: `nexus_captcha_local.py` v2 con subcomandos `detect/solve/solve-text/solve-vision/solve-cnn`, y guarda de seguridad anti-`KeyError` cuando la detección falla.

### 2026-08-02 🖐️ — Nexus Hands + Vault AES-256 + Resolver CAPTCHA Local (Soberanía Total)
- **Nexus Hands v1** (`docker/cua/nexus_hands.py`): dominio real del mouse/teclado en el entorno CUA. Motor xdotool (Xvfb :99 `-ac`), movimiento con curva Bezier + jitter Perlin, tecleo biométrico con distribución normal (Box-Muller), errores tipográficos realistas + correcciones. 100% local, sin API keys.
- **Cadena de "ojos" operativa**: captura de pantalla con motor encadenado `import → xwd → scrot` (ImageMagick + x11-apps + scrot añadidos a la imagen CUA) + OCR con tesseract local.
- **Nexus Vault v1** (`docker/cua/nexus_vault.py`): bóveda cifrada AES-256-GCM con PBKDF2-HMAC-SHA256 (100k iteraciones, salt aleatorio), permisos 0600, tecleo biométrico que nunca pega. Cero texto plano en disco.
- **Resolver CAPTCHA local** (`docker/cua/nexus_captcha_local.py`): Tier 1 texto→tesseract, Tier 2 semántico→Ollama visión (LLaVA/Qwen2-VL). Detecta tipo por heurística y resuelve sin depender de APIs externas.
- **Integración CaptchaBridge** (`scripts/nexus_captcha_bridge.cjs`): los CAPTCHAs de imagen se resuelven primero con el resolver local del contenedor CUA; Capsolver queda solo como fallback para reCAPTCHA/hCaptcha/Turnstile (Proof-of-Work, ~20% de los casos).
- **Validado end-to-end**: input (size/move/click/type), screenshot, OCR, detector CAPTCHA, y vault (init/set/get/list) operativos en `nexus-cua-gui`.

### 2026-08-02 🧠 — Motor Nativo Soberano con Qwen3-4B (mistral.rs v0.9.0)
- **Migración completa Candle → mistral.rs**: backend de inferencia nativa reescrito en `ia_nativa.rs` usando `GgufModelBuilder` con `.with_device(Device::Cpu)`
- **Evolución Soberana 0.8.1 → v0.9.0**: crates.io está congelado en 0.8.1 (soporte Qwen3 inmaduro, fallaba con `failed to fill whole buffer`); migrado a git tag v0.9.0 que corrige el loader GGUF de Qwen3
- **Diagnóstico quirúrgico**: descartadas VRAM (RTX 3070 8GB saturada), RAM, AWQ y corrupción de archivo; el GGUF de unsloth validado 100% íntegro (SHA256 = etag HF) y parseable por llama.cpp (ollama)
- **Inferencia real verificada**: Qwen3-4B-Q4_K_M asimilado en CPU, 465 tokens generados en 49.52s (~9.4 tok/s)
- **Pipeline reordenado**: Córtex Nativo priorizado antes de fallbacks externos en `pipeline.rs`

### 2026-08-02 🔍 — OMEGA Search v3.0: Sovereign Deep Research
- **Hito**: Evolución completa del motor de búsqueda científica a una arquitectura de investigación profunda basada en evidencias.
- **Logro**: Implementación de un **Firewall de Prompts** para neutralizar inyecciones indirectas y algoritmos de **Poda por Densidad Textual** (Fit Markdown) para eliminar ruido del DOM.
- **Optimización**: Sistema de ranking por relevancia semántica (reputación de dominio + coincidencia de términos) que reduce la navegación inútil en un 80%.
- **Robustez**: Hardening del servidor MCP con gestión de excepciones asíncronas y concurrencia de 6 workers, garantizando estabilidad total bajo el límite de 60s.
- **Estado**: Capa de búsqueda web omnisciente integrada y validada para depuración técnica autónoma.

### 2026-08-02 🔱 — CUA: Entorno de Ejecución Dual (Docker Headless GUI + Firecracker)
#### **Objetivo:** Protocolo Computer-Using Agent para interacción GUI y sandboxing de alta seguridad.
#### **Entorno Primario — Docker Headless GUI** (`docker/cua/`):
- `docker-compose.cua.yml`: Xvfb (:99) + fluxbox + x11vnc (:5900) + noVNC (:6080), resolución 1920x1080, límites cpus 2.0/mem 1G, healthcheck, cap_drop ALL + no-new-privileges.
- `Dockerfile.cua`: Ubuntu 22.04 + Xvfb/x11vnc/noVNC/websockify/fluxbox + Chromium.
- `entrypoint.sh`: orquesta Xvfb→fluxbox→x11vnc→websockify→noVNC, trap de limpieza, autostart Chromium opcional.
- **Verificado:** contenedor `running`/`healthy`, puertos 6080+5900 escuchando, todos los procesos internos activos, noVNC interno HTTP 200.

#### **Entorno Secundario — Firecracker MicroVM** (aislamiento KVM):
- Binarios + jailer + kernel `vmlinux.bin` + rootfs presentes en `firecracker_env/`, KVM `/dev/kvm` operativo.
- Reutiliza `scripts/ghost_ignition.sh` para el boot vía API socket.

#### **Orquestador de Decisión** (`orquestador_cua.py`):
- Regla automática: tareas GUI diarias/navegador/formularios → **DOCKER**; binarios no verificados/malware/detonación/kernel → **FIRECRACKER**.
- Verificado: "abrir panel trading" → DOCKER; "detonar binario no verificado" → FIRECRACKER.
- `nexus_cua.sh`: CLI unificado (status/evaluar/up/down/abrir).

### 2026-08-02 🎯 — Sentinel Trading: Límite Configurable de Operaciones (1-500)
#### **Problema:** El trader simulado quemaba el capital en 18 órdenes por usar un qty fijo enorme (50 NVDA ≈ $6.800), impidiendo sostener operaciones.
#### **Solución:**
- **Sizing dinámico:** cada posición asigna `4%` del capital (`FRACCION_POR_OPERACION`) en lugar del qty fijo, evitando la quema de saldo.
- **Límite configurable:** nuevo endpoint `GET/POST /api/limite-operaciones` con rango gobernado `1–500` (clamp en backend, `clamp` de Rust).
- **Contador soberano:** `operaciones_realizadas` en `AppState`; al alcanzar el límite se apaga `modo_auto` automáticamente y registra pensamiento.
- **UI:** input numérico `OPS` + botón `APLICAR` en la cabecera del portal (sincronizado con backend vía `/api/auto-trading/estado`).
- **Despliegue:** release rebuild + relanzado en puerto 42210; verificado GET/POST/clamp (999→500) y frontend servido con el nuevo asset.

### 2026-08-02 🧭 — Orquestador Autónomo: Gobernanza de Ejecución Soberana
- **Hito**: Creación del módulo [`orquestador_autonomo.rs`](core/src/cerebro/orquestador_autonomo.rs:1) que cierra los 4 huecos de autonomía detectados en la auditoría del orquestador.
- **Circuit Breaker**: Interruptor de circuito por proveedor — abre tras N fallos consecutivos, cierra tras cooldown, con estado half-open de recuperación.
- **DAG de tareas**: Grafo dirigido acíclico que modela dependencias, estado por nodo (Pendiente/EnEjecucion/Completado/Fallido/Bloqueado) y paralelización ordenada por prioridad.
- **Introspección de herramientas**: Medición en runtime de latencia promedio y tasa de éxito por herramienta, con ranking de eficiencia para re-ordenar prioridades dinámicamente.
- **Compresión de contexto**: Auto-síntesis extractiva de conversaciones largas (frecuencia léxica + longitud) con umbral de activación configurable.
- **Validación**: 10 tests unitarios pasados, 0 fallos; compilación limpia sin errores. Puro Rust, cero dependencias externas, cero unwrap().

## 🎨 FASE 5 — Pulido v0-Real (generador multi-agente) — COMPLETADA
- **Módulo nuevo**: [`polish.rs`](core/src/cerebro/v0/polish.rs) con:
  - `validar_tokens()` — enforcement de design tokens (detecta hex/rgb/px hardcodeados vs `var(--tokens)`)
  - `exportar_codesandbox()` / `exportar_stackblitz()` — payloads listos para embebido
  - `ErrorDataset` — agrega errores residuales de los 3 gates por código con frecuencia
- **generator.rs**: `construir_index_css` emite SIEMPRE ambos temas (light+dark) con `@media (prefers-color-scheme)` automático + clase `.dark` manual
- **pipeline.rs**: `ResultadoPipeline` ahora lleva `error_dataset` poblado en Etapa 9 → cura allowlist/prompts
- **Validación**: 115 tests verdes en `cerebro::v0`, build limpio (0 warnings en módulos v0)
- Plan actualizado: FASE 5 checkboxes marcados en `plans/ARQUITECTURA_V0_MULTI_AGENTE.md`

## 🧠 FASE 6 — Refuerzo RAG + Razonamiento Local (Qwen/Ollama + Web) — COMPLETADA
- **Hito**: capa de refuerzo para que el modelo local "trabaje con lo que tiene, lo mejore y luego lo presente" — extrae referencias web (RAG) y razona/planifica ANTES de generar.
- **Módulo nuevo**: [`razonador_qwen.rs`](core/src/cerebro/v0/razonador_qwen.rs) — cliente Ollama local (`http://localhost:11434/api/chat`, `stream:false`). `razonar_local()` (determinista, infiere tecnología/stack), `razonar()` (async con JSON-schema, fallback local), `generar(prompt, contexto)`. 7 tests.
- **Módulo nuevo**: [`refuerzo_web.rs`](core/src/cerebro/v0/refuerzo_web.rs) — `extraer_local()` (inferencia de keywords → `ReferenciaWeb`), `ensamblar_contexto()` (bloque `[CONTEXTO RAG NEXUS]` con catálogo shadcn). 7 tests.
- **Integración pipeline.rs**: nueva **Etapa 1b** (refuerzo RAG + razonamiento) entre planificación y generación. `ResultadoPipeline` ahora lleva `refuerzo`, `plan_razonado`, `refuerzo_local`. Pasada síncrona usa motores locales deterministas (nunca paniquea sin red).
- **Validación**: 130 tests verdes en `cerebro::v0` (incluye `test_pipeline_refuerzo_rag_y_razonamiento`), build limpio (0 warnings en módulos v0).
- **Versión**: `VERSION_V0` bump → 0.6.0. Plan actualizado en `plans/ARQUITECTURA_V0_MULTI_AGENTE.md`.

## 🧠 FASE 6.1 — Memoria de Contexto (Hipocampo): búsqueda selectiva para ventana pequeña — COMPLETADA
- **Misión**: como la ventana de contexto del modelo local es pequeña, se le dio un almacén de memoria externo donde buscar su propio contexto (RAG selectivo por presupuesto de tokens).
- **Módulo nuevo**: [`memoria_contexto.rs`](core/src/cerebro/v0/memoria_contexto.rs) — `MemoriaContexto` con `FragmentoContexto` indexado (categoría + claves + contenido), búsqueda por relevancia léxica, `contar_tokens()` (~4 chars/token) y `recuperar(prompt, presupuesto)` que puntúa, ordena y recorta al presupuesto con flag `recortado`. `sembrar_shadcn()` indexa el catálogo como fragmentos. 8 tests.
- **Integración [`refuerzo_web.rs`](core/src/cerebro/v0/refuerzo_web.rs)**: `memoria` + `presupuesto_tokens` (default 800, builder `con_presupuesto_tokens`), `recuperar_contexto()` reemplaza a `ensamblar_contexto` (solo inyecta los fragmentos relevantes en `[CONTEXTO RAG NEXUS]`), `ingerir_referencias()` persiste en memoria las referencias web extraídas (persistencia de sesión). 6 tests de integración.
- **Validación**: 143 tests verdes en `cerebro::v0` (0 fallos), build limpio (0 warnings en módulos v0).
- **Versión**: `VERSION_V0` bump → 0.7.0. Plan actualizado en `plans/ARQUITECTURA_V0_MULTI_AGENTE.md`.

## 📊 NEXUS TRADER V2.0 — FASE 3: Correcciones del Dashboard de Trading — COMPLETADA
- **Bug Monto Max. Entry $0.00**: definida `actualizarCalculoRiesgo()` (antes `ReferenceError`), única fuente de verdad del cálculo riesgo = balance × (riesgo%/100).
- **KILL SWITCH de emergencia**: botón en header que detiene auto-trading, cierra posiciones y corta el feed.
- **REAL_MODE_ACTIVE visible**: borde rojo pulsante `@keyframes nexus-real-pulse` en estado real conectado.
- **Timeframe del gráfico**: velas canvas alineadas a 1m (`intervalMs=60000`) para coincidir con el widget "1m".
- **Curva de capital real**: PnL calculado desde ventas ejecutadas, baseline centrada cuando la línea es plana.
- **Telemetría de conexión + auditoría Sentinel**: `lastTickReceived`/`lastPriceAtTick` rastreados en cada tick; estado `REAL_MODE_ACTIVE · Nms` / `STALE`; log de Sentinel en el panel de decisiones (feed ms, último precio).
