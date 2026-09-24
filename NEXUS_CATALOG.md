# Catálogo de Componentes y Servicios NEXUS

| Dominio | Módulo / Servicio | Ubicación | Capacidad existente |
| :--- | :--- | :--- | :--- |
| **Visión** | `vision.rs` / `take_screenshot` | `nexus-agent/src/vision.rs` | Capturas vía Playwright y Maim |
| **Dashboard** | `review_dashboard` | `pipeline/review_dashboard/` | Servidor Node/Tailwind (Puerto 5180) |
| **Engine 2D** | Nexus Engine | Local | Puerto 3000 |
| **Autopublicador**| Servicio Autopublicador | Local | Puerto 3001 |
| **Torre Core** | Torre Central | Local | Puerto 9000 |
| **Inferencia** | Extractor UI & Arquitecto | `pipeline/agent_ui/`, `pipeline/agent_architect/` | Generación de `ui_spec`, `erd`, `openapi`, `schema.sql` |
| **Backend Gen** | Generador Axum | `pipeline/agent_backend/` | Plantilla y compilador en Rust |
| **Escritorio Remoto** | GNOME Remote Desktop (RDP) | Local (`gnome-remote-desktop`) | Puerto 3389 (Nombre: Nexus) |
