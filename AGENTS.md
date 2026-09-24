# AGENTS.md · NEXUS CORE & Lovable Architecture Specification

## 🚀 Overview
NEXUS CORE is an autonomous, high-performance AI development workspace and Control Engine designed to generate, inspect, and deploy fullstack web applications matching **Lovable.dev** standards.

---

## 🎨 UI & Layout Conventions
1. **2-Column Lovable Layout**:
   - **Left Sidebar (`w-[420px]`)**:
     - Project Title (`Blooming Icon Studio · NEXUS Assistant`).
     - Cascade AI Model Selector (OpenRouter 70B, Claude 3.5 Sonnet, GPT-4o, DeepSeek R1, Llama 3.3).
     - Detailed Chat History with formatted response cards (`Generated Image >`, code badges, file links, thumbs feedback).
     - Suggestion Action Pills (`Descarga en SVG`, `Crea versión monocromo`, `Genera variantes de color`, `Exporta en tamaños`, `Agrega menú`).
     - Bottom Floating Prompt Bar: `+` Menu Popup (Search, Adjuntar, Diseño, Conectores, BDs), `Crear ˅` vs `Plan` mode selector (`Alt + P`), Dictado por voz `🎤`, and Prompt submission.
   - **Right Viewport (`flex-grow`)**:
     - Navigation Header Bar: `🌐 Vista previa`, `📄 Archivos`, `</> Código`, `🥞 Capas / ERD`.
     - Device & Utility controls: `💻` Desktop, `📱` Mobile, `🔄` Reload, Page Dropdown, `↗` External Link.
     - Top Action Buttons: `Compartir` (Share link), `Publicar` (Vibrant deployment modal).
     - Main Canvas / File Explorer / Code Tree Viewer / Visual Backend Connector Studio.

---

## 🦀 Backend Architecture (Rust Tokio + Axum)
- **Engine**: High-concurrency Rust Tokio async runtime.
- **REST Framework**: Axum 0.7.
- **Database**: PostgreSQL with automatic ERD graph synchronization (Mermaid.js).
- **Contracts**: OpenAPI 3.0 JSON specification automatically exported.

---

## ⚡ Deployment & Publishing
- **1-Click Publishing**: Supports multi-target compilation to Rust Axum Tokio native servers, Vercel Edge, or Cloudflare Workers.
- **Bundle Export**: Downloads complete project codebase as a single JSON/ZIP bundle.

---

---

## 🏎️ High Performance & Autonomous Execution Directives
1. **Navegación Headless Nativa (Zero-GUI)**: Usar siempre el motor nativo de Rust (`ShadowCrawlAPI` / `cloudscraper_rs`) para investigación y scraping web. Prohibido abrir navegadores GUI pesados a menos que se solicite explícitamente.
2. **Depuración en Tiempo Real (`inotify`)**: Utilizar el crate `notify 6.1` para escuchar eventos del Kernel de Linux a 0% de uso de CPU sin polling.
3. **Inspección Visual en Caliente (`MiradaHumana`)**: Validar páginas y componentes en modo headless usando `MiradaHumana` y `AnalisisVisual` antes de entregar el resultado.
4. **Directiva Internal-First (Zero-Duplicates)**: Consultar siempre las sinapsis, memoria y el `CatalogoHipocampico` de NEXUS antes de generar código nuevo. Reutilizar artefactos existentes.
5. **Lienzo de Capas / ERD Interactivo (`ReactFlow`)**:
   - Las constantes de nodos/bordes (`DEMO_NODES`, `DEMO_EDGES`) deben declararse siempre fuera del ciclo de renderizado de los componentes React (`App.jsx`) para preservar las coordenadas de arrastre sin reseteos.
   - Todo nodo debe ser 100% arrastrable, seleccionable y conectar libremente.
6. **Ecosistema Multi-Servicio & Puertos Activos**:
   - NEXUS Studio UI: `http://localhost:5173`
   - Autopublicador FB: `http://localhost:5174`
   - Nexus Engine 2D: `http://localhost:5175`
   - Daemon Rust PTY Backend: `http://127.0.0.1:43210`
7. **Catálogo de Modelos Desinhibidos (Hermes / Pentesting)**:
   - Hermes Desktop y Antigravity priorizan modelos sin censura de alta capacidad (`nousresearch/hermes-3-llama-3.1-70b` y `abacusai/dracarys-llama-3.1-70b-instruct`). No bloquear por etiquetas incompletas de proveedores de inferencia.
