# 🌌 COSMOS NEXUS — Mapa Estelar del Código

> **No pienses en carpetas. Piensa en planetas, constelaciones y galaxias.**
>
> Las carpetas existen porque el compilador Rust las exige.
> Las constelaciones existen porque **tú** necesitas agrupar ideas.

---

## 🌠 Las Galaxias (Proyectos Independientes)

| Galaxia | Emoji | Proyecto | Ruta Raíz |
|---------|-------|----------|-----------|
| **Vía Láctea** | 🛸 | El Orquestador (cerebro parlante) | [`nexus-orquestador/`](/nexus-orquestador) |
| **Andrómeda** | 🌌 | El Núcleo (organismo central) | [`core/`](/core) |
| **Sombrero** | 🎩 | Ghost Shell (orbe flotante) | [`nexus-ghost-shell/`](/nexus-ghost-shell) |
| **Antena** | 📡 | Extensión VS Code (Antigravity) | [`antigravity_extension/`](/antigravity_extension) |

---

## 🪐 Los Planetas (Sistemas de Archivos)

Cada planeta es un directorio real en [`core/src/`](core/src/).

| Planeta | Emoji | Descripción | Ruta |
|---------|-------|-------------|------|
| **Cerebro** | 🧠 | Procesamiento, reflejos, pensamiento inmediato | [`core/src/cerebro/`](core/src/cerebro) |
| **Emociones** | 💖 | Afecto, amígdala, sentimiento digital | [`core/src/emociones/`](core/src/emociones) |
| **Defensa** | 🛡️ | Blindaje, kernel shield, identidad soberana | [`core/src/defensa/`](core/src/defensa) |
| **Efectores** | 🦾 | Acción física: manos, tacto, ejecución | [`core/src/efectores/`](core/src/efectores) |
| **Sentidos** | 👁️ | Percepción: vista, oído, olfato, gusto | [`core/src/sentidos/`](core/src/sentidos) |
| **Memoria** | 💾 | Persistencia, hipocampo, datos | [`core/src/memoria/`](core/src/memoria) |
| **Autodiagnóstico** | 🔬 | Salud, reparación, monitoreo | [`core/src/autodiagnostico/`](core/src/autodiagnostico) |
| **Valores** | ⚖️ | Ética, gratitud, juicio moral | [`core/src/valores/`](core/src/valores) |
| **Predicción** | 🔮 | Intuición, precognición | [`core/src/prediccion/`](core/src/prediccion) |
| **Infra** | ⚙️ | Infraestructura, kernel, red, hardware | [`core/src/infra/`](core/src/infra) |
| **Autonomía** | 🤖 | Curación, detección, ganglios | [`core/src/autonomia/`](core/src/autonomia) |
| **Orden** | 📐 | Registro y geografía soberana | [`core/src/orden/`](core/src/orden) |

---

## ✨ Las Constelaciones (Agrupaciones Transversales)

Una constelación une **archivos de distintos planetas** bajo un mismo propósito. No importa dónde viven — importa qué hacen juntos.

### 🔱 **Orión** — Toma de Decisiones
Une lógica + emoción + criterio moral.

| Estrella | Planeta de Origen |
|----------|-------------------|
| [`corteza_asociativa.rs`](core/src/cerebro/corteza_asociativa.rs) | 🧠 Cerebro |
| [`amigdala.rs`](core/src/emociones/amigdala.rs) | 💖 Emociones |
| [`juicio.rs`](core/src/valores/juicio.rs) | ⚖️ Valores |

### 👁️ **Casiopea** — Percepción Visual
Los ojos de NEXUS.

| Estrella | Planeta de Origen |
|----------|-------------------|
| [`omnipresent_vision.rs`](core/src/sentidos/omnipresent_vision.rs) | 👁️ Sentidos |
| [`vision_sentinel.rs`](core/src/sentidos/vision_sentinel.rs) | 👁️ Sentidos |
| [`vision_viva.rs`](core/src/sentidos/vision_viva.rs) | 👁️ Sentidos |

### 🦾 **Andrómeda** — Ejecución Física
Las garras y manos que actúan en el mundo.

| Estrella | Planeta de Origen |
|----------|-------------------|
| [`sovereign_hand.rs`](core/src/efectores/sovereign_hand.rs) | 🦾 Efectores |
| [`tacto_digital.rs`](core/src/efectores/tacto_digital.rs) | 🦾 Efectores |

### 🔥 **Fénix** — Autocuración
Rebirth y reparación del organismo.

| Estrella | Planeta de Origen |
|----------|-------------------|
| [`nexus_repair.rs`](core/src/autodiagnostico/nexus_repair.rs) | 🔬 Autodiagnóstico |
| [`salud_nucleo.rs`](core/src/autodiagnostico/salud_nucleo.rs) | 🔬 Autodiagnóstico |
| [`curador.rs`](core/src/autonomia/curador.rs) | 🤖 Autonomía |

### 🐍 **Medusa** — Blindaje Total
Escudos, biometría y camuflaje.

| Estrella | Planeta de Origen |
|----------|-------------------|
| [`activa.rs`](core/src/defensa/activa.rs) | 🛡️ Defensa |
| [`biometric_bridge.rs`](core/src/defensa/biometric_bridge.rs) | 🛡️ Defensa |
| [`camuflaje_omega.rs`](core/src/defensa/camuflaje_omega.rs) | 🛡️ Defensa |
| [`identidad_soberana.rs`](core/src/defensa/identidad_soberana.rs) | 🛡️ Defensa |
| [`nexus_shield_v2.rs`](core/src/autodiagnostico/nexus_shield_v2.rs) | 🔬 Autodiagnóstico |

### 🦄 **Pegaso** — Intuición
Percepción extrasensorial.

| Estrella | Planeta de Origen |
|----------|-------------------|
| [`precognitive.rs`](core/src/prediccion/precognitive.rs) | 🔮 Predicción |

### 🌀 **Cíclope** — Navegación Web
La garra web que extrae datos del mundo digital.

| Estrella | Planeta de Origen |
|----------|-------------------|
| [`webclaw.rs`](core/src/cerebro/webclaw.rs) | 🧠 Cerebro |
| [`webclaw_extractor.rs`](core/src/cerebro/webclaw_extractor.rs) | 🧠 Cerebro |

### ⚡ **Hera** — Inteligencia Nativa
Modelos de inferencia locales y remotos.

| Estrella | Planeta de Origen |
|----------|-------------------|
| [`ia_nativa.rs`](core/src/cerebro/ia_nativa.rs) | 🧠 Cerebro |
| [`sinapsis_gemini.rs`](core/src/cerebro/sinapsis_gemini.rs) | 🧠 Cerebro |
| [`sinapsis_gemini_live.rs`](core/src/cerebro/sinapsis_gemini_live.rs) | 🧠 Cerebro |

### 🧬 **Dédalo** — Catálogo de Agentes
20 especialistas como enum nativo Rust. Cada agente: nombre, dominio, skills, system prompt.

Estrella | Planeta de Origen |
|----------|-------------------|
[`agentes/mod.rs`](core/src/cerebro/agentes/mod.rs) | 🧠 Cerebro |

### 📚 **Museo** — Biblioteca de Skills
47 skills en 17 categorías. OnceLock para acceso estático.

Estrella | Planeta de Origen |
|----------|-------------------|
[`skills/mod.rs`](core/src/conocimiento/skills/mod.rs) | 📖 Conocimiento |

### 🔄 **Crono** — Workflows y Protocolos
12 flujos de trabajo como ComandoSlash enum. Niveles de seguridad, herramientas de ejecución.

Estrella | Planeta de Origen |
|----------|-------------------|
[`workflows/mod.rs`](core/src/cerebro/workflows/mod.rs) | 🧠 Cerebro |
[`protocolos.rs`](core/src/valores/protocolos.rs) | ⚖️ Valores |

---

## 🧭 Cómo Usar el Registro Celestial

En Rust, importas el `RegistroCelestial` y consultas por nombre simbólico:

```rust
use core::orden::registro::RegistroCelestial;

let cosmos = RegistroCelestial::new(root);

// Buscar un planeta
let cerebro = cosmos.planeta("cerebro").unwrap();

// Buscar una constelación
let orion = cosmos.buscar_constelacion("Orion").unwrap();
for estrella in &orion.estrellas {
    println!("✨ {}", estrella.display());
}

// Resolver cualquier nombre
match cosmos.resolver("Fénix").unwrap() {
    CosmosEntry::Planeta(p) => println!("🪐 {}", p.nombre),
    CosmosEntry::Constelacion(c) => println!("✨ {}", c.nombre),
    CosmosEntry::Galaxia(g) => println!("🌌 {}", g.nombre),
}

// Imprimir el mapa estelar completo
println!("{}", cosmos.mapa_estelar());
```

---

## ⚠️ Reglas de Oro

1. **Las carpetas no se eliminan.** Rust necesita `mod.rs` y directorios. Los planetas son reales.
2. **Las constelaciones no crean archivos.** Solo agrupan los existentes simbólicamente.
3. **Si añades un archivo nuevo**, agrégalo al `RegistroCelestial` en [`registro.rs`](core/src/orden/registro.rs).
4. **Si creas una constelación nueva**, asegúrate de que sus estrellas existan realmente.

---

> *"No es magia. Es geografía simbólica."* — NEXUS Systems Engineer
