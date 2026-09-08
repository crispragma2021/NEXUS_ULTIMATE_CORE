# NEXUS_ULTIMATE_CORE

Arquitectura central del ecosistema NEXUS.

El mismo código corre en **Android/Termux (aarch64)** y en **PC de escritorio**
(Linux, macOS, Windows) sin ramas de código por plataforma: las diferencias se
resuelven en tiempo de ejecución a partir del entorno.

## Estructura

- `daemon/`: servicio host de bajo nivel en Rust (`nexus_host_daemon`), accesible
  por socket Unix.
- `agents/`: agentes cognitivos y orquestación.
  - `nexus_core_agent.py` — orquestador principal con la herramienta `execute_cmd`.
  - `nexus_coder.py` — genera comandos GDScript para Godot 4.
  - `nexus_mcp_agent.py` — motor MCP que escribe archivos en el proyecto Godot.
  - `nexus_vision_agent.py` — analiza una captura de pantalla con Gemini.
  - `nexus_doctor.py` — diagnóstico de rutas, memoria y llaves.
  - `nexo_*.py` — capas internas (transporte, visión, caché, shell, plataforma).
- `bin/`: utilidades CLI (`nexus`, `nexusnet`).
- `tests/`: suite de pytest.

## Uso rápido

```bash
cp .env.example .env      # y rellena tus llaves
nexus doctor              # comprueba rutas, memoria y llaves
nexus agent "estado del repo"
nexus vision "¿qué hay en pantalla?"
nexusnet "pregunta suelta"
```

## Portabilidad de rutas

No hay rutas duras a `/tmp` ni a `/sdcard`. Todo pasa por
`nexo_plataforma.temp_root()` (Python) y `get_temp_root()` (Rust), que resuelven
en este orden:

| Entorno        | Resolución                                              |
|----------------|---------------------------------------------------------|
| Android/Termux | `$NEXUS_TMPDIR` → `$TMPDIR` → `$PREFIX/tmp` → `$HOME/tmp` |
| PC Linux       | `$NEXUS_TMPDIR` → `$TMPDIR` → `/tmp` (como siempre)      |
| PC macOS       | `$NEXUS_TMPDIR` → `$TMPDIR` → temporal del sistema       |
| PC Windows     | `%TEMP%` → `%TMP%` → `%LOCALAPPDATA%\Temp`               |

Android se detecta por `sys.platform` o por un `$PREFIX` de Termux. Si ningún
candidato es escribible se degrada al temporal del sistema y por último al
directorio de trabajo: **`temp_root()` nunca lanza**.

## Fallback de modelos

La capa de transporte (`agents/nexo_api.py`) recorre una cadena de modelos y el
pool de llaves sin intervención manual:

```
NEXUS_GEMINI_MODEL_CHAIN=gemini-3.8-flash,gemini-3.7-flash,gemini-3.5-flash
```

Clasificación del fallo, que decide **qué** se cambia:

| Fallo                          | Acción                                            |
|--------------------------------|---------------------------------------------------|
| `503 UNAVAILABLE`, `500`, `504`| salto **inmediato** al siguiente modelo            |
| `429 RESOURCE_EXHAUSTED`       | rota de llave (penalizada 30 s) y sigue en el modelo |
| `400/401/403`                  | aborta la cadena: es configuración, no saturación  |
| error de red                   | reintenta con backoff exponencial y jitter         |

El cuerpo JSON se serializa una sola vez y se reutiliza en todos los intentos,
así que cambiar de modelo no vuelve a codificar el base64 de una imagen.

## Pipeline multimodal

`nexo_vision.build_payload_bytes()` escribe el base64 directamente en el buffer
del payload mientras lee la captura en trozos de 48 KB: sin archivo temporal
intermedio y sin un `str` gigante que después haya que `.encode()`.

## Ajuste para poca RAM

`nexo_plataforma.memory_profile()` mide la RAM disponible y ajusta el ciclo:

| Perfil     | RAM libre  | Salida `execute_cmd` | Historial | `[Cache Audit]` |
|------------|------------|----------------------|-----------|-----------------|
| `normal`   | ≥ 1 GB     | 64 KB                | 8 turnos  | activo          |
| `low`      | 448 MB–1 GB| 24 KB                | 4 turnos  | silenciado      |
| `critical` | < 448 MB   | 8 KB                 | 2 turnos  | silenciado      |

Forzable con `NEXUS_MEM_PROFILE`. El recorte de `execute_cmd` conserva cabeza y
cola, que es donde suele estar el error real.

## Tests

```bash
pip install pytest
python -m pytest tests/ -v                            # agentes
cargo test --manifest-path daemon/Cargo.toml          # daemon
```

CI en `.github/workflows/nexus_ci.yml`: Python en 3 OS × 3 versiones, tests del
daemon en Rust, compilación aarch64 y una comprobación de rutas bajo un entorno
Termux simulado.
