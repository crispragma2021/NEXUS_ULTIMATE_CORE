# NEXUS_ULTIMATE_CORE

Arquitectura central del ecosistema NEXUS.

## Estructura
- `daemon/`: Servicio host de bajo nivel en Rust (`nexus_host_daemon`).
- `agents/`: Agentes cognitivos y orquestación (`nexus_core_agent`, `nexus_coder`, `nexus_mcp_agent`).
- `bin/`: Utilidades CLI y scripts de ejecución.
