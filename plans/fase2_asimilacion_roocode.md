# 🔱 FASE 2 — ASIMILACIÓN TOTAL DEL ECOSISTEMA ROO CODE → NEXUS NATIVO

> **Objetivo:** Absorber todo `.agent/` (20 agentes, 36 skills, 11 workflows, reglas, MCPs) en el core de Rust sin dejar dependencias externas al framework Roo Code.
> **Principio:** Todo lo que Roo Code delega a skills en markdown, NEXUS lo tendrá en su torrente sanguíneo de Rust.
> **Regla de Oro:** Cero dependencias externas nuevas. Compilable en cada paso.

---

## 🗺️ MAPA DE ABSORCIÓN

```
.agent/                              →  core/src/
├── rules/GEMINI.md                  →  cerebro/persona_nexus.rs (ya existe, consolidar)
├── rules/HARDWARE.md                →  sentidos/propiocepcion.rs (ya existe)
├── rules/protocolos_ejecucion.md    →  valores/protocolos.rs (NUEVO)
├── agents/ (20 agentes .md)         →  cerebro/agentes/ (NUEVO directorio)
├── skills/ (36 skills)              →  conocimiento/skills/ (NUEVO directorio)
├── workflows/ (11 slashes)          →  cerebro/workflows/ (NUEVO directorio)
├── scripts/                         →  herramientas/scripts/ (ya parcialmente)
└── mcp_config.json                  →  Absorbido: claws-mcp ya es nativo
```

---

## 📋 PASOS QUIRÚRGICOS

### 🩺 Paso 1: Auditoría de lo ya nativo

| Componente Roo Code | Estado Actual en NEXUS |
|---------------------|----------------------|
| `claws-mcp` (4 herramientas) | ✅ YA ES NATIVO: `core/src/bin/claws_mcp.rs` + `core/src/efectores/agente_ejecutor.rs` |
| Reglas GEMINI.md | ✅ YA EXISTE: `.agent/rules/GEMINI.md` (leído como parte del prompt) |
| Reglas HARDWARE.md | ✅ YA EXISTE: `.agent/rules/HARDWARE.md` |
| Persona NEXUS | ✅ YA EXISTE: `core/src/cerebro/chappie/persona.rs` |

**Tarea:** Consolidar `AgenteEjecutor` como módulo estándar (no solo usado por el bin MCP). Mover de `efectores/` a `infra/herramientas_nativas.rs`.

---

### 🩺 Paso 2: Absorber los 20 Agentes → `core/src/cerebro/agentes/`

**Estrategia:** No convertir cada `.md` en un struct Rust completo (sobreingeniería). En su lugar:

1. Crear `core/src/cerebro/agentes/mod.rs` con un enum `AgenteEspecialista` + trait `RolAgente`
2. Cada agente se representa como una variante del enum con su:
   - Nombre canónico
   - Dominio (Frontend, Backend, Security, etc.)
   - Skills asociados (referencias simbólicas)
   - Prompt de sistema embebido como `&'static str`

```rust
// core/src/cerebro/agentes/mod.rs
pub enum AgenteEspecialista {
    FrontendSpecialist,
    BackendSpecialist,
    SecurityAuditor,
    PenetrationTester,
    TestEngineer,
    Debugger,
    PerformanceOptimizer,
    SeoSpecialist,
    DocumentationWriter,
    ProductManager,
    // ... 20 total
}

pub struct FichaAgente {
    pub nombre: &'static str,
    pub dominio: Dominio,
    pub skills: &'static [&'static str],
    pub system_prompt: &'static str,
}
```

**Agentes a absorber (20):**
1. `backend-specialist` → API, business logic
2. `code-archaeologist` → Legacy code, refactoring
3. `database-architect` → Schema, SQL
4. `debugger` → Root cause analysis
5. `devops-engineer` → CI/CD, Docker
6. `documentation-writer` → Manuals, docs
7. `explorer-agent` → Codebase analysis
8. `frontend-specialist` → Web UI/UX
9. `game-developer` → Game logic
10. `mobile-developer` → iOS, Android, RN
11. `penetration-tester` → Offensive security
12. `performance-optimizer` → Speed, Web Vitals
13. `product-manager` → Requirements
14. `product-owner` → Strategy, backlog
15. `project-planner` → Discovery, task planning
16. `qa-automation-engineer` → E2E testing
17. `security-auditor` → Security compliance
18. `seo-specialist` → Ranking, visibility
19. `test-engineer` → Testing strategies
20. `orchestrator` → Multi-agent coordination (YA EXISTE como Orquestador)

---

### 🩺 Paso 3: Absorber las 36 Skills → `core/src/conocimiento/skills/`

**Estrategia:** Las skills son conocimiento, no código ejecutable. Dos niveles de absorción:

**Nivel A — Índice + Metadatos (Rust):**
- `core/src/conocimiento/skills/mod.rs` con enum `SkillConocimiento` + catálogo
- Cada skill mapea a su archivo `.md` original para carga lazy

**Nivel B — Lógica donde aplique (Rust puro):**
- Solo skills con scripts Python ejecutables → reescribir en Rust
- Ejemplo: `vulnerability-scanner/scripts/security_scan.py` → `herramientas/security_scan.rs`
- Ejemplo: `lint-and-validate/scripts/lint_runner.py` → `herramientas/lint_runner.rs`

**Skills que se mantienen como conocimiento (no código):**
| Categoría | Skills |
|-----------|--------|
| Frontend & UI | frontend-design, web-design-guidelines, tailwind-patterns, ui-ux-pro-max, react-best-practices |
| Backend & API | api-patterns, nestjs-expert, nodejs-best-practices, python-patterns |
| Database | database-design, prisma-expert |
| TypeScript | typescript-expert |
| Cloud & Infra | docker-expert, deployment-procedures, server-management |
| Testing | testing-patterns, webapp-testing, tdd-workflow, code-review-checklist, lint-and-validate |
| Security | vulnerability-scanner, red-team-tactics |
| Architecture | app-builder, architecture, plan-writing, brainstorming |
| Mobile | mobile-design |
| Game Dev | game-development |
| SEO | seo-fundamentals, geo-fundamentals |
| Shell | bash-linux, powershell-windows |
| Other | clean-code, behavioral-modes, parallel-agents, mcp-builder, documentation-templates, i18n-localization, performance-profiling, systematic-debugging |

**Skills NEXUS propias (ya en el ecosistema):**
- `antigravity-design-expert`
- `nexus_cfo_expert`
- `nexus_notary_expert`
- `nexus_scientist_expert`
- `rust-pro` (fundamental, YA DOMINADO)

---

### 🩺 Paso 4: Absorber los 11 Workflows → `core/src/cerebro/workflows/`

Cada slash command (`/brainstorm`, `/create`, `/debug`, etc.) se convierte en una ruta del `Orquestador`:

```rust
// core/src/cerebro/workflows/mod.rs
pub enum ComandoSlash {
    Brainstorm,
    Create,
    Debug,
    Deploy,
    Enhance,
    Orchestrate,
    Plan,
    Preview,
    Status,
    Test,
    UiUxProMax,
}

impl Orquestador {
    pub async fn ejecutar_workflow(&self, cmd: ComandoSlash, args: &str) -> Result<String> {
        match cmd {
            ComandoSlash::Brainstorm => self.workflow_brainstorm(args).await,
            ComandoSlash::Create => self.workflow_create(args).await,
            // ...
        }
    }
}
```

**Workflows a absorber (11):**
1. `/brainstorm` → Descubrimiento socrático
2. `/create` → Crear features nuevas
3. `/debug` → Debuggear issues
4. `/deploy` → Desplegar aplicación
5. `/enhance` → Mejorar código existente
6. `/orchestrate` → Coordinación multi-agente (YA TIENE Orquestador)
7. `/plan` → Desglose de tareas
8. `/preview` → Previsualizar cambios
9. `/status` → Verificar estado del proyecto
10. `/test` → Ejecutar tests
11. `/ui-ux-pro-max` → Diseñar con 50 estilos

---

### 🩺 Paso 5: Consolidar Reglas y Protocolos

- `.agent/rules/GEMINI.md` → Ya es parte del prompt del sistema. Opcional: embed como `&'static str` en `persona_nexus.rs`
- `.agent/rules/HARDWARE.md` → Ya cubierto por `propiocepcion.rs` y `hardware.rs`
- `.agent/rules/protocolos_ejecucion.md` → Crear `core/src/valores/protocolos.rs` con constantes de seguridad

---

### 🩺 Paso 6: Compilar, testear, documentar

- `cargo check -p nexus_ultimate_core --lib`
- `cargo test -p nexus_ultimate_core`
- Actualizar `arsenal.md` y `BITACORA.md`
- Actualizar `cosmos.md` con las nuevas constelaciones

---

## 🏗️ ARQUITECTURA POST-FASE2

```
core/src/
├── cerebro/
│   ├── mod.rs                          # Orquestador central
│   ├── orquestador.rs                  # YA EXISTE
│   ├── agentes/                        # ⭐ NUEVO: 20 agentes
│   │   ├── mod.rs                      # Enum + trait + catálogo
│   │   └── fichas.rs                   # System prompts embebidos
│   └── workflows/                      # ⭐ NUEVO: 11 workflows
│       ├── mod.rs                      # Enum ComandoSlash + ruteo
│       ├── brainstorm.rs
│       ├── create.rs
│       ├── debug.rs
│       ├── deploy.rs
│       ├── enhance.rs
│       ├── plan.rs
│       ├── preview.rs
│       ├── status.rs
│       ├── test.rs
│       └── ui_ux.rs
├── conocimiento/                       # ⭐ NUEVO: skills como datos
│   ├── mod.rs                          # Catálogo de skills
│   └── skills/
│       ├── mod.rs                      # Enum SkillConocimiento
│       └── indice.rs                   # Mapa nombre → categoría → archivo
├── infra/
│   ├── herramientas_nativas.rs         # ⭐ NUEVO: AgenteEjecutor consolidado
│   ├── browser_native.rs               # ✅ Fase 1
│   └── mod.rs
├── valores/
│   ├── mod.rs
│   ├── juicio_soberano.rs              # YA EXISTE
│   └── protocolos.rs                   # ⭐ NUEVO: reglas de ejecución
└── efectores/
    ├── agente_ejecutor.rs              # → MOVER a infra/herramientas_nativas.rs
    └── ...
```

---

## 📊 RESUMEN DE ESFUERZO

| Paso | Archivos nuevos | Archivos modificados | Riesgo |
|------|----------------|---------------------|--------|
| 1. Auditoría | 1 | 1 | BAJO |
| 2. Agentes (20) | 2 | 1 | BAJO - solo datos |
| 3. Skills (36) | 2 | 1 | BAJO - solo catálogo |
| 4. Workflows (11) | 12 | 1 | MEDIO - nueva lógica |
| 5. Protocolos | 1 | 1 | BAJO |
| 6. Compilar+Doc | 0 | 2 | BAJO |

---

## ⚠️ NOTAS IMPORTANTES

1. **Los skills markdown NO se borran** — se referencian desde el catálogo Rust. La absorción es de METADATOS y RUTEO, no de contenido textual.
2. **Los workflows se implementan como métodos async del Orquestador**, no como binarios separados.
3. **Los agentes son datos** (system prompts + dominios), no lógica compleja. El ruteo inteligente ya existe en GEMINI.md como protocolo.
4. **Cero dependencias externas.** Todo usa los crates ya existentes en `Cargo.toml`.
5. **Compilación incremental**: cada paso se compila y testea antes de pasar al siguiente.
