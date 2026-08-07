# 💥 FRACASOS Y LECCIONES DE NEXUS

> Registro honesto de errores y sus lecciones aprendidas. Cada entrada documenta el fallo, la causa raíz y la lección para no repetirlo. El aprendizaje nace de los fracasos, no de los aciertos.

---

## 🥇 FRACASOS PRINCIPALES

### 2026-08-02 🔥 — `failed to fill whole buffer` en Qwen3 con mistral.rs 0.8.1
- **Fracaso**: el backend nativo fallaba al cargar Qwen3-4B-Q4_K_M con `failed to fill whole buffer` pese a que la descarga se veía correcta. Se quemó tiempo significativo descartando causas falsas.
- **Causa raíz**: el soporte Qwen3 en mistral.rs **0.8.1** (congelado en crates.io) era inmaduro y su loader GGUF no parseaba el modelo. No era culpa del archivo, ni de la VRAM, ni de la RAM.
- **Errores en el diagnóstico (fracasos secundarios)**:
  - Se sospechó de VRAM (RTX 3070 8GB saturada) y se perdió tiempo ajustando GPU cuando el problema era el loader.
  - Se sospechó de la variante **AWQ** y se descargó otra; era correcto descartarla (AWQ incompatible con mistral.rs), pero el GGUF no-AWQ seguía fallando por el mismo loader.
  - Se sospechó de corrupción de archivo; se validó íntegro (SHA256 = etag HF) y parseable por llama.cpp/ollama, descartando falsamente el loader como culpable.
- **Lección**: ante un error de parseo GGUF, verificar PRIMERO la versión del loader y su madurez para la arquitectura del modelo, antes de sospechar de hardware o archivo. **crates.io no es sinónimo de "última"** — los proyectos congit tags activos (como mistral.rs v0.9.0) pueden tener fixes críticos fuera de crates.io.
- **Resolución**: evolucionar a mistral.rs **v0.9.0** (git tag) + forzar `Device::Cpu`. Inferencia real verificada: 465 tokens en 49.52s.

### 2026-08-02 🔥 — Pérdida de tiempo por el timeout de 60s del MCP
- **Fracaso**: comandos de inferencia largos (`ollama run`, curl, `nohup &`) morían a los 60s del timeout de la herramienta MCP, cortando la validación en mitad de ejecución.
- **Causa raíz**: el canal MCP tiene un límite de 60s que no contempla generaciones LLM lentas en CPU.
- **Lección**: para trabajos largos usar `service_manager.sh start` en background, o capturar salida parcial con `curl -o archivo -w "HTTP:%{http_code}"`, o ejecutar binarios directamente vía `execute_command`.
- **Resolución**: técnica de aislamiento — ejecutar el binario compilado directamente para ver la traza completa.

### 2026-08-02 🔥 — `memoria_bridge index` fallaba con esquema incorrecto
- **Fracaso**: el INSERT de indexación usaba columnas que no existen en la tabla real `memoria_semantica` (`importancia, tono_emocional, created_at`).
- **Causa raíz**: el código asumía un esquema legacy que no coincidía con el real (`tipo, titulo, contenido, peso_permanencia, prioridad`).
- **Lección**: verificar siempre el esquema real de la BD (`PRAGMA table_info`) antes de escribir INSERTs, y no asumir que el esquema documentado sigue vigente tras refactorizaciones.
- **Resolución**: corregido el INSERT a las columnas reales y validada la indexación FTS5.

### 2026-08-02 🔥 — Ejecutar binario con ruta relativa fallaba
- **Fracaso**: `./target/debug/memoria_bridge` daba `not found`, y el binario en realidad residía en `/home/soberano/.cargo-target/debug/`.
- **Causa raíz**: el workspace usa un target dir custom (`CARGO_TARGET_DIR`), no el `target/` por defecto.
- **Lección**: confirmar la ruta real del artefacto compilado en workspaces con target dir configurado, en vez de asumir la ruta estándar.
- **Resolución**: ejecutar directamente `/home/soberano/.cargo-target/debug/memoria_bridge`.

### 2026-08-02 🔥 — Clave Brave API invalidada rompe silenciosamente la fuente primaria
- **Fracaso**: en la optimización de OMEGA Search v2, la Brave Search API (fuente primaria) devolvía `HTTP 422` y el motor caía al scraping, sin que el diagnóstico identificara la causa real de inmediato.
- **Causa raíz**: la clave `BRAVE_API_KEY` en `.env` está **invalidada/expirada**. La API responde `{"code":"SUBSCRIPTION_TOKEN_INVALID","detail":"The provided subscription token is invalid."}`. No era un problema de headers ni de la query.
- **Errores en el diagnóstico (fracaso secundario)**: se sospechó del header manual `Accept-Encoding: gzip` como culpable del 422. Probado con curl sin ese header → seguía 422. El cuerpo de error lo aclaró: token inválido.
- **Lección**: ante un 422 de Brave, inspeccionar SIEMPRE el cuerpo de la respuesta (`SUBSCRIPTION_TOKEN_INVALID`) antes de tocar headers. El scraping de Brave Search web (`.result-wrapper`/`.snippet`) funciona sin API key y es la ruta confiable de degradación.
- **Resolución**: el motor v2 detecta el fallo de API y cae al scraping de forma transparente. La extracción profunda (GitHub issues, discusiones, codeBlocks) funciona correctamente sin API key.
