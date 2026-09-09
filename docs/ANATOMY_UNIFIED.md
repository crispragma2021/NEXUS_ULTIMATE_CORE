# 🏛️ NEXUS: ANATOMÍA UNIFICADA (POST-FUSIÓN OMEGA)

## 🧠 Núcleo de Procesamiento (Cerebro Maestro)
- **Directorio**: `nexus-orquestador/`
- **Componentes**: 
  - Orquestación SOJ (Sensory, Cognitive, Executive).
  - Inteligencia de Enjambres (Swarms).
  - Sistema Inmunológico Avanzado (`seguridad/inmunidad.rs`).
  - Motor de Planificación Gemini/DeepSeek unificado.

## 🦴 Soma y Conexiones (Médula Espinal)
- **Directorio**: `core/src/`
- **Componentes**:
  - `memoria/`: Gestión de Ring Buffer y persistencia atómica.
  - `infra/`: Detección de hardware agnóstica y drivers.
  - `vision/`: Procesamiento visual local.
  - `bin/proxy_hijack.rs`: Puente de comunicación interceptora.

## 👁️ Sentidos Activos
- **Visión**: `nexus-orquestador/src/sentidos_vision.rs` y `core/src/vision/`.
- **Audición**: `nexus-orquestador/src/audio_capturer.rs`.
- **Kernel-Pulse**: `nexus_ebpf/` (Monitoreo de bajo nivel).

## 🚪 Portales HUD
- **Chat Soberano**: `core/src/ui/` (Puerto 1420).
- **Infiltración**: `proxy_hijack` (Puerto 4444).

## 🗄️ Búnker Histórico (Legacy)
- **Directorio**: `legacy/`
- **Estado**: Todos los componentes obsoletos, duplicados o succionados residen aquí en estado de hibernación.
