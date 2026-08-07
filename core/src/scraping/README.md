# 🕸️ NEXUS Scraping Pipeline

Pipeline de web scraping en dos capas (Two-Tier) escrito en **Rust**, integrado en el orquestador `NEXUS_ULTIMATE_CORE`.

```
[ Web ] → fetch → clean (HTML→Markdown) → route (≤4k tokens?) → infer → persistir (SQLite)
                                                          ├── ≤4k → Tier-2 Nube (CloudAdapter)
                                                          └── >4k → Tier-1 SLM Local (Ollama Map-Reduce) → resumen → Tier-2
```

## Características

| Módulo | Función |
|---|---|
| [`pipeline/`](pipeline/mod.rs:1) | Orquestador `Pipeline::run()` / `process_html()` |
| [`fetcher.rs`](pipeline/fetcher.rs:1) | Captura HTTP (`reqwest`) con rotación de User-Agent y respeto a `robots.txt` |
| [`cleaner.rs`](pipeline/cleaner.rs:1) | Limpieza determinista HTML → Markdown (100% código, sin LLM) |
| [`router.rs`](pipeline/router.rs:1) | Threshold switch 4,000 tokens + chunker 1,500 tokens |
| [`ollama_client.rs`](pipeline/ollama_client.rs:1) | Cliente Ollama local (`num_ctx=32768`, formato JSON) |
| [`map_reduce.rs`](pipeline/map_reduce.rs:1) | Bucle Map-Reduce para el caso masivo |
| [`scratchpad.rs`](pipeline/scratchpad.rs:1) | Bloc de notas `.jsonl` temporal |
| [`cloud_adapter.rs`](pipeline/cloud_adapter.rs:1) | Fallback nube: OpenRouter → Gemini → DeepSeek, con circuit breaker y timeouts |
| [`provider_circuit.rs`](pipeline/provider_circuit.rs:1) | Circuit breaker por proveedor (3 fallos → pausa 5 min) |
| [`daemon.rs`](pipeline/daemon.rs:1) | Demonio tokio con polling, señales y graceful shutdown |
| [`rate_limiter.rs`](pipeline/rate_limiter.rs:1) | Rate limiting por dominio (2s, máx 3 concurrentes) |
| [`db.rs`](pipeline/db.rs:1) | SQLite: `tasks`, `extracted_data`, `robots_cache`, `rate_limit_state` |
| [`metrics.rs`](pipeline/metrics.rs:1) | Métricas: tasks, tiers, tokens, latencia, circuit opens |
| [`logging.rs`](pipeline/logging.rs:1) | Logs JSON Lines o humano |

## Requisitos

- **Rust** 1.75+ (toolchain del workspace)
- **Ollama** (opcional, para tier-1): `curl -fsSL https://ollama.com/install.sh | sh`
- **Modelo local** (recomendado): Qwen 2.5 sin censura → [`Modelfile.nexuslocal-free`](../../../Modelfile.nexuslocal-free:1)

## Instalación

```bash
cd NEXUS_ULTIMATE_CORE

# 1. Compilar (lib + binario del demonio)
cargo build -p nexus_ultimate_core --release --bin scraper-daemon

# 2. (Opcional) Instalar el Qwen sin censura en Ollama
bash scripts/instalar_nexuslocal_free.sh
```

## Uso rápido (demonio)

```bash
# Desarrollo con Ollama local (tier-1)
cargo run -p nexus_ultimate_core --bin scraper-daemon -- \
    --db scraper.db --ollama-model qwen2.5:7b

# Con nube OpenRouter (tier-2), keys repetibles para el pool rotatorio
cargo run -p nexus_ultimate_core --bin scraper-daemon -- \
    --db scraper.db \
    --openrouter-key sk-or-v1-xxx \
    --openrouter-key sk-or-v1-yyy
```

### Alimentar el demonio

El demonio procesa tareas `pending` en la tabla `tasks`. Inserta una tarea:

```bash
sqlite3 scraper.db "INSERT INTO tasks (task_id, url, strategy) \
  VALUES ('t1', 'https://example.com/articulo', 'http');"
```

El demonio la tomará en el siguiente ciclo de polling.

## Uso como librería

```rust
use nexus_ultimate_core::scraping::pipeline::{
    Fetcher, Pipeline, PipelineConfig, TaskSchema, Strategy,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let fetcher = std::sync::Arc::new(Fetcher::new(None)?);
    let pipeline = Pipeline::new(fetcher, None, None, None, PipelineConfig::default());

    let task = TaskSchema {
        task_id: "t-1".into(),
        url: "https://example.com".into(),
        strategy: Strategy::Http,
        ..Default::default()
    };

    let result = pipeline.run(&task).await;
    println!("{:?}", result.status);
    Ok(())
}
```

> Nota: `TaskSchema` usa `#[serde(default)]` en la mayoría de campos, pero
> `task_id`, `url` y `strategy` son obligatorios.

## Despliegue

### systemd

```bash
cargo build -p nexus_ultimate_core --release --bin scraper-daemon
sudo cp systemd/scraper.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now scraper
```

### Docker Compose

```bash
docker compose -f docker/docker-compose.scraper.yml up -d --build
docker compose -f docker/docker-compose.scraper.yml logs -f
```

Ollama se ejecuta como servicio **externo** (en el host) para no duplicar la GPU; el daemon se conecta vía `host.docker.internal:11434`.

## Observabilidad

### Logs JSON Lines

```bash
NEXUS_LOG_JSON=1 RUST_LOG=info cargo run -p nexus_ultimate_core --bin scraper-daemon
```

Cada línea incluye: `timestamp`, `level`, `target`, `fields`.

### Métricas

El pipeline expone métricas vía [`metrics.rs`](pipeline/metrics.rs:1) (snapshot JSON y formato Prometheus):

- `tasks_total`, `tasks_success`, `tasks_failed`, ...
- `tier_usage_ratio`, `tier1_calls`, `tier2_calls`
- `tokens_processed_total`, `avg_inference_latency_ms`
- `provider_circuit_opens` (por proveedor)
- `errors_by_category`

## Configuración de tier-1 (Ollama)

Umbral de enrutamiento: **4,000 tokens** (aprox. 12,000 caracteres).
- Markdown ≤ 4k → directo a nube.
- Markdown > 4k → Map-Reduce local (chunks de 1,500 tokens, overlap 100, máx 20) → resumen → nube.

Parámetros del modelo local (ver [`Modelfile.nexuslocal-free`](../../../Modelfile.nexuslocal-free:1)):

```
num_ctx 32768
temperature 0.1
```

## Tests

```bash
cargo test -p nexus_ultimate_core --lib scraping
```

Cubre: schemas, DB, fetcher (robots.txt), cleaner (HTML→Markdown), token counter,
router (threshold + chunker), Ollama client, scratchpad, Map-Reduce, CloudAdapter
(fallback + timeouts + circuit breaker), rate limiter, daemon, metrics, logging.

## Arquitectura de referencia

Los planes de diseño están en [`plans/`](../../../plans/):

- [`pipeline-spec.md`](../../../plans/pipeline-spec.md) — especificación completa
- [`adr.md`](../../../plans/adr.md) — decisiones de arquitectura (ADR 0013 Rust-first, ADR 0014 sin rotación de cuentas)
- [`implementation-plan.md`](../../../plans/implementation-plan.md) — fases F0–F8
- [`nexus-epic-roadmap.md`](../../../plans/nexus-epic-roadmap.md) — capas épicas E1–E6
