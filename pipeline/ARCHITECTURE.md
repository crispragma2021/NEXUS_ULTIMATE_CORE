# Spec: Screenshot-to-Fullstack Engine

Pipeline de ingeniería inversa automatizado por agentes de IA con Checkpoint Humano (Mesa de Revisión Dual).

## Arquitectura de Fases:

### 1. Fase 1: Extracción UI/UX (`agent_ui`)
- **Entrada:** Imagen (captura de pantalla PNG/JPG o URL).
- **Proceso:** Inferencia visual con modelo multimodal.
- **Salida:** `/home/nexus/NEXUS_ULTIMATE_CORE/pipeline/artifacts/ui_spec.json`
- **Contenido:** Layout, componentes (Tailwind CSS + shadcn/ui), paleta de colores, estado y jerarquía.

### 2. Fase 2: Inferencia de Arquitectura y Datos (`agent_architect`)
- **Entrada:** `ui_spec.json`
- **Proceso:** Deducir el modelo relacional de datos y contratos de servicios.
- **Salidas:**
  - `pipeline/artifacts/architecture_spec.json` (Visión unificada)
  - `pipeline/artifacts/schema.sql` (Esquema DDL PostgreSQL/SQLite)
  - `pipeline/artifacts/openapi.json` (Especificación OpenAPI 3.0)
  - `pipeline/artifacts/erd.mermaid` (Diagrama ERD relacional en Mermaid.js)

### 3. Fase 3: Mesa de Revisión Dual (`review_dashboard`)
- **Entorno:** Servidor local ligero Node.js/Express (`pipeline/review_dashboard/server.js`) en puerto `5180`.
- **Vista Dual:**
  - **Panel Izquierdo (Por fuera):** Vista previa interactiva de la UI mockeada.
  - **Panel Derecho (Por dentro):** Diagrama Mermaid ERD interactivo + Visor OpenAPI Swagger + Checkpoint de aprobación humana.

### 4. Fase 4: Generación de Código Final (`agent_backend`)
- **Entrada:** Artefactos aprobados por el desarrollador.
- **Salida:** Servidor funcional en Rust (Axum + SQLx + Serde) y cliente Frontend.
