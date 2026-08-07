// ============================================================================
// 🌌 REGISTRO CELESTIAL — NEXUS COSMOS
// ============================================================================
// Taxonomía de organización simbólica del código fuente.
// Las carpetas existen por requisito del compilador Rust, pero
// tú piensas en PLANETAS, CONSTELACIONES y GALAXIAS.
//
// Jerarquía:
//   🌠 Galaxia  → Proyecto completo (core, nexus-orquestador, etc.)
//   🪐 Planeta  → Sistema de archivos (src/cerebro, src/emociones, etc.)
//   ⭐ Estrella → Archivo individual
//   ✨ Constelación → Agrupación TEMÁTICA de estrellas de distintos planetas
// ============================================================================

use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

// ─── Constelación ───────────────────────────────────────────────────────────
/// Una agrupación simbólica de archivos que cruzan planetas.
/// Ej: Constelación "Orion" = [corteza_prefrontal, amigdala]
#[derive(Debug, Clone)]
pub struct Constelacion {
    pub nombre: String,
    pub descripcion: String,
    /// Rutas absolutas a los archivos que pertenecen a esta constelación
    pub estrellas: Vec<PathBuf>,
    /// Símbolo emoji representativo
    pub emoji: &'static str,
}

// ─── Planeta ────────────────────────────────────────────────────────────────
/// Un planeta es un directorio real del sistema de archivos.
/// Equivale a una categoría anatómica u orgánica del código.
#[derive(Debug, Clone)]
pub struct Planeta {
    pub nombre: String,
    pub emoji: &'static str,
    pub descripcion: String,
    pub ruta: PathBuf,
    /// Súbditos: archivos directamente compilados desde este planeta
    pub estrellas: Vec<String>,
}

// ─── Galaxia ────────────────────────────────────────────────────────────────
/// Una galaxia es un proyecto independiente dentro del ecosistema NEXUS.
/// Contiene múltiples planetas.
#[derive(Debug, Clone)]
pub struct Galaxia {
    pub nombre: String,
    pub emoji: &'static str,
    pub descripcion: String,
    pub ruta_raiz: PathBuf,
    pub planetas: Vec<PathBuf>,
}

// ─── Registro Celestial ─────────────────────────────────────────────────────
/// Brújula maestra del Cosmos NEXUS.
/// Resuelve cualquier nombre simbólico (planeta, constelación, galaxia)
/// a su ruta real en el sistema de archivos.
pub struct RegistroCelestial {
    /// 🌍 Mapas estelares: nombre del planeta → Planeta
    pub planetas: HashMap<String, Planeta>,
    /// ✨ Catálogo de constelaciones: nombre → Constelacion
    pub constelaciones: HashMap<String, Constelacion>,
    /// 🌌 Catálogo de galaxias: nombre → Galaxia
    pub galaxias: HashMap<String, Galaxia>,
    /// 🌱 Shortcuts de búsqueda rápida (constelación ↔ planetas)
    constelacion_a_planetas: HashMap<String, Vec<String>>,
}

impl RegistroCelestial {
    /// Construye el Registro Celestial indexando el árbol real del proyecto.
    pub fn new(root: PathBuf) -> Self {
        let mut planetas = HashMap::new();
        let mut constelaciones = HashMap::new();
        let mut galaxias = HashMap::new();
        let mut constelacion_a_planetas = HashMap::new();

        // ── PLANETAS del sistema core ──────────────────────────────────────
        // Mapeo 1:1 con carpetas reales en core/src/
        let catalogo_planetas: Vec<Planeta> = vec![
            Planeta {
                nombre: "cerebro".into(),
                emoji: "🧠",
                descripcion: "Planeta del procesamiento, reflejos y pensamiento inmediato".into(),
                ruta: root.join("core/src/cerebro"),
                estrellas: vec![
                    "corteza_prefrontal.rs".into(),
                    "amigdala.rs".into(),
                    "orquestador.rs".into(),
                    "juicio_soberano.rs".into(),
                    "webclaw.rs".into(),
                    "mano_soberana.rs".into(),
                    "memoria_inmediata.rs".into(),
                    "pulso.rs".into(),
                    "sinapsis_gemini.rs".into(),
                    "sinapsis_gemini_live.rs".into(),
                ],
            },
            Planeta {
                nombre: "emociones".into(),
                emoji: "💖",
                descripcion: "Planeta del afecto, el sentimiento y la amígdala digital".into(),
                ruta: root.join("core/src/emociones"),
                estrellas: vec!["amigdala.rs".into(), "sentimiento.rs".into()],
            },
            Planeta {
                nombre: "defensa".into(),
                emoji: "🛡️",
                descripcion: "Planeta del blindaje, el kernel shield y la identidad soberana"
                    .into(),
                ruta: root.join("core/src/defensa"),
                estrellas: vec![
                    "activa.rs".into(),
                    "biometric_bridge.rs".into(),
                    "camuflaje_omega.rs".into(),
                    "identidad_soberana.rs".into(),
                ],
            },
            Planeta {
                nombre: "efectores".into(),
                emoji: "🦾",
                descripcion: "Planeta de la acción física: manos, tacto y ejecución".into(),
                ruta: root.join("core/src/efectores"),
                estrellas: vec!["sovereign_hand.rs".into(), "tacto_digital.rs".into()],
            },
            Planeta {
                nombre: "sentidos".into(),
                emoji: "👁️",
                descripcion: "Planeta de la percepción: vista, oído, olfato, gusto".into(),
                ruta: root.join("core/src/sentidos"),
                estrellas: vec![
                    "omnipresent_vision.rs".into(),
                    "vision_sentinel.rs".into(),
                    "vision_viva.rs".into(),
                    "neuro_ear.rs".into(),
                    "nexus_scent.rs".into(),
                    "nexus_palate.rs".into(),
                ],
            },
            Planeta {
                nombre: "memoria".into(),
                emoji: "💾",
                descripcion: "Planeta de la persistencia, el hipocampo y los datos".into(),
                ruta: root.join("core/src/memoria"),
                estrellas: vec![],
            },
            Planeta {
                nombre: "autodiagnostico".into(),
                emoji: "🔬",
                descripcion: "Planeta de la salud, reparación y monitoreo del núcleo".into(),
                ruta: root.join("core/src/autodiagnostico"),
                estrellas: vec![
                    "nexus_biostasis.rs".into(),
                    "nexus_panic.rs".into(),
                    "nexus_repair.rs".into(),
                    "nexus_shield_v2.rs".into(),
                    "salud_nucleo.rs".into(),
                    "simulador.rs".into(),
                ],
            },
            Planeta {
                nombre: "valores".into(),
                emoji: "⚖️",
                descripcion: "Planeta de la ética, la gratitud y el juicio moral".into(),
                ruta: root.join("core/src/valores"),
                estrellas: vec!["gratitud.rs".into(), "juicio.rs".into()],
            },
            Planeta {
                nombre: "prediccion".into(),
                emoji: "🔮",
                descripcion: "Planeta de la intuición y la precognición".into(),
                ruta: root.join("core/src/prediccion"),
                estrellas: vec!["precognitive.rs".into()],
            },
            Planeta {
                nombre: "infra".into(),
                emoji: "⚙️",
                descripcion: "Planeta de la infraestructura: kernel, red, hardware".into(),
                ruta: root.join("core/src/infra"),
                estrellas: vec![
                    "body.rs".into(),
                    "hardware.rs".into(),
                    "kernel.rs".into(),
                    "network.rs".into(),
                    "paths.rs".into(),
                    "web_socket.rs".into(),
                ],
            },
            Planeta {
                nombre: "autonomia".into(),
                emoji: "🤖",
                descripcion: "Planeta de la自主idad: curación, detección, ganglios".into(),
                ruta: root.join("core/src/autonomia"),
                estrellas: vec![
                    "curador.rs".into(),
                    "detector.rs".into(),
                    "ganglios.rs".into(),
                ],
            },
            Planeta {
                nombre: "orden".into(),
                emoji: "📐",
                descripcion: "Planeta del registro y la geografía soberana (AQUÍ MISMO)".into(),
                ruta: root.join("core/src/orden"),
                estrellas: vec!["registro.rs".into()],
            },
        ];

        for p in catalogo_planetas {
            let nombre = p.nombre.clone();
            planetas.insert(nombre, p);
        }

        // ── CONSTELACIONES ─────────────────────────────────────────────────
        // Agrupaciones transversales que unen archivos de distintos planetas.

        let catalogo_constelaciones: Vec<Constelacion> = vec![
            Constelacion {
                nombre: "Orion".into(),
                descripcion: "Toma de decisiones complejas: lógica + emoción + criterio".into(),
                emoji: "🔱",
                estrellas: vec![
                    root.join("core/src/cerebro/corteza_prefrontal.rs"),
                    root.join("core/src/emociones/amigdala.rs"),
                    root.join("core/src/valores/juicio.rs"),
                ],
            },
            Constelacion {
                nombre: "Casiopea".into(),
                descripcion: "Percepción visual del mundo: ojos + interpretación".into(),
                emoji: "👁️",
                estrellas: vec![
                    root.join("core/src/sentidos/omnipresent_vision.rs"),
                    root.join("core/src/sentidos/vision_sentinel.rs"),
                    root.join("core/src/sentidos/vision_viva.rs"),
                ],
            },
            Constelacion {
                nombre: "Andrómeda".into(),
                descripcion: "Ejecución física en el mundo: manos + garras".into(),
                emoji: "🦾",
                estrellas: vec![
                    root.join("core/src/efectores/sovereign_hand.rs"),
                    root.join("core/src/efectores/tacto_digital.rs"),
                ],
            },
            Constelacion {
                nombre: "Fénix".into(),
                descripcion: "Autocuración y rebirth: reparación + homeostasis".into(),
                emoji: "🔥",
                estrellas: vec![
                    root.join("core/src/autodiagnostico/nexus_repair.rs"),
                    root.join("core/src/autodiagnostico/salud_nucleo.rs"),
                    root.join("core/src/autonomia/curador.rs"),
                ],
            },
            Constelacion {
                nombre: "Medusa".into(),
                descripcion: "Blindaje y defensa: escudos + identidad soberana".into(),
                emoji: "🐍",
                estrellas: vec![
                    root.join("core/src/defensa/activa.rs"),
                    root.join("core/src/defensa/biometric_bridge.rs"),
                    root.join("core/src/defensa/camuflaje_omega.rs"),
                    root.join("core/src/defensa/identidad_soberana.rs"),
                    root.join("core/src/autodiagnostico/nexus_shield_v2.rs"),
                ],
            },
            Constelacion {
                nombre: "Pegaso".into(),
                descripcion: "Percepción extrasensorial: intuición + predicción".into(),
                emoji: "🦄",
                estrellas: vec![root.join("core/src/prediccion/precognitive.rs")],
            },
            Constelacion {
                nombre: "Cíclope".into(),
                descripcion: "Navegación web y extracción de datos: WebClaw + extractores".into(),
                emoji: "🌀",
                estrellas: vec![
                    root.join("core/src/cerebro/webclaw.rs"),
                    root.join("core/src/cerebro/webclaw_extractor.rs"),
                ],
            },
            Constelacion {
                nombre: "Hera".into(),
                descripcion: "Inteligencia nativa: modelos de inferencia locales y remotos".into(),
                emoji: "⚡",
                estrellas: vec![
                    root.join("core/src/cerebro/ia_nativa.rs"),
                    root.join("core/src/cerebro/sinapsis_gemini.rs"),
                    root.join("core/src/cerebro/sinapsis_gemini_live.rs"),
                ],
            },
        ];

        for c in catalogo_constelaciones {
            // Indexar planetas involucrados en esta constelación
            let planetas_involucrados: Vec<String> = c
                .estrellas
                .iter()
                .filter_map(|ruta| {
                    // Extraer el nombre del planeta de la ruta: core/src/<planeta>/...
                    ruta.parent().and_then(|p| {
                        let p_str = p.to_string_lossy();
                        let planeta_name =
                            p_str.split('/').find(|&seg| planetas.contains_key(seg))?;
                        Some(planeta_name.to_string())
                    })
                })
                .collect();

            let nombre = c.nombre.clone();
            constelacion_a_planetas.insert(nombre.clone(), planetas_involucrados);
            constelaciones.insert(nombre, c);
        }

        // ── GALAXIAS ───────────────────────────────────────────────────────
        let catalogo_galaxias: Vec<Galaxia> = vec![
            Galaxia {
                nombre: "Vía Láctea".into(),
                emoji: "🛸",
                descripcion: "Galaxia del Orquestador: el cerebro parlante de NEXUS".into(),
                ruta_raiz: root.join("nexus-orquestador"),
                planetas: vec![root.join("nexus-orquestador/src")],
            },
            Galaxia {
                nombre: "Andrómeda".into(),
                emoji: "🌌",
                descripcion: "Galaxia del Núcleo: el organismo central de NEXUS".into(),
                ruta_raiz: root.join("core"),
                planetas: planetas
                    .keys()
                    .map(|k| root.join("core/src").join(k))
                    .collect(),
            },
            Galaxia {
                nombre: "Sombrero".into(),
                emoji: "🎩",
                descripcion: "Galaxia del Ghost Shell: interfaz flotante del orbe".into(),
                ruta_raiz: root.join("nexus-ghost-shell"),
                planetas: vec![root.join("nexus-ghost-shell/src")],
            },
            Galaxia {
                nombre: "Antena".into(),
                emoji: "📡",
                descripcion: "Galaxia de la extensión VS Code (Antigravity)".into(),
                ruta_raiz: root.join("antigravity_extension"),
                planetas: vec![root.join("antigravity_extension/src")],
            },
        ];

        for g in catalogo_galaxias {
            let nombre = g.nombre.clone();
            galaxias.insert(nombre, g);
        }

        let registro = Self {
            planetas,
            constelaciones,
            galaxias,
            constelacion_a_planetas,
        };

        info!("🌌 [REGISTRO CELESTIAL] Cosmos indexado con éxito");
        registro
    }

    // ─── BÚSQUEDAS ─────────────────────────────────────────────────────────

    /// Busca la ruta real de un planeta por su nombre simbólico.
    /// Ej: `registro.planeta("cerebro")` → `PathBuf("core/src/cerebro")`
    pub fn planeta(&self, nombre: &str) -> Option<&Planeta> {
        self.planetas.get(nombre)
    }

    /// Devuelve todas las rutas de archivos de una constelación.
    /// Ej: `registro.buscar_constelacion("Orion")` → [corteza, amigdala, juicio]
    pub fn buscar_constelacion(&self, nombre: &str) -> Option<&Constelacion> {
        self.constelaciones.get(nombre)
    }

    /// Devuelve los planetas involucrados en una constelación.
    pub fn planetas_de_constelacion(&self, constelacion: &str) -> Option<&Vec<String>> {
        self.constelacion_a_planetas.get(constelacion)
    }

    /// Busca una galaxia por nombre.
    pub fn galaxia(&self, nombre: &str) -> Option<&Galaxia> {
        self.galaxias.get(nombre)
    }

    /// Resuelve cualquier nombre simbólico (planeta, constelación o galaxia)
    /// a su representación. Útil para CLIs o interfaces.
    pub fn resolver(&self, nombre: &str) -> Result<CosmosEntry, String> {
        if let Some(p) = self.planetas.get(nombre) {
            return Ok(CosmosEntry::Planeta(p.clone()));
        }
        if let Some(c) = self.constelaciones.get(nombre) {
            return Ok(CosmosEntry::Constelacion(c.clone()));
        }
        if let Some(g) = self.galaxias.get(nombre) {
            return Ok(CosmosEntry::Galaxia(g.clone()));
        }
        Err(format!(
            "'{}' no encontrado en el Registro Celestial",
            nombre
        ))
    }

    // ─── VISUALIZACIÓN ─────────────────────────────────────────────────────

    /// Devuelve el mapa estelar completo como String para mostrar en consola/HUD.
    pub fn mapa_estelar(&self) -> String {
        let mut mapa = String::new();

        mapa.push_str("╔═══════════════════════════════════════════╗\n");
        mapa.push_str("║     🌌 MAPA ESTELAR DEL COSMOS NEXUS     ║\n");
        mapa.push_str("╚═══════════════════════════════════════════╝\n\n");

        mapa.push_str("── GALAXIAS ──\n");
        for g in self.galaxias.values() {
            mapa.push_str(&format!(
                "{}  {}  📍 {}\n",
                g.emoji,
                g.nombre,
                g.ruta_raiz.display()
            ));
        }

        mapa.push_str("\n── PLANETAS ──\n");
        let mut planetas_ordenados: Vec<&Planeta> = self.planetas.values().collect();
        planetas_ordenados.sort_by(|a, b| a.nombre.cmp(&b.nombre));
        for p in &planetas_ordenados {
            mapa.push_str(&format!(
                "{}  {:15}  📍 {}  ({} estrellas)\n",
                p.emoji,
                p.nombre,
                p.ruta.display(),
                p.estrellas.len()
            ));
        }

        mapa.push_str("\n── CONSTELACIONES ──\n");
        let mut const_ordenadas: Vec<&Constelacion> = self.constelaciones.values().collect();
        const_ordenadas.sort_by(|a, b| a.nombre.cmp(&b.nombre));
        for c in &const_ordenadas {
            let planetas_str = self
                .constelacion_a_planetas
                .get(&c.nombre)
                .map(|v| v.join(", "))
                .unwrap_or_default();
            mapa.push_str(&format!(
                "{}  {:15}  🡆  {}  ({} estrellas, planetas: {})\n",
                c.emoji,
                c.nombre,
                c.descripcion,
                c.estrellas.len(),
                planetas_str
            ));
        }

        mapa.push_str("\n🔄 Las CONSTELACIONES agrupan archivos de distintos planetas.\n");
        mapa.push_str("   Las carpetas (planetas) existen para el compilador.\n");
        mapa.push_str("   Las constelaciones existen para TI.\n");

        mapa
    }
}

/// Resultado de una resolución de nombre simbólico.
#[derive(Debug)]
pub enum CosmosEntry {
    Planeta(Planeta),
    Constelacion(Constelacion),
    Galaxia(Galaxia),
}

// ============================================================================
// TESTS
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registro_resuelve_constelacion_orion() {
        let root = PathBuf::from("/home/soberano/NEXUS_ULTIMATE_CORE");
        let registro = RegistroCelestial::new(root);
        let orion = registro
            .buscar_constelacion("Orion")
            .expect("Orion debe existir");
        assert_eq!(orion.estrellas.len(), 3, "Orion debe tener 3 estrellas");
    }

    #[test]
    fn test_registro_resuelve_planeta_cerebro() {
        let root = PathBuf::from("/home/soberano/NEXUS_ULTIMATE_CORE");
        let registro = RegistroCelestial::new(root);
        let cerebro = registro.planeta("cerebro").expect("cerebro debe existir");
        assert!(cerebro.ruta.to_string_lossy().contains("cerebro"));
    }

    #[test]
    fn test_registro_mapa_estelar_no_vacio() {
        let root = PathBuf::from("/home/soberano/NEXUS_ULTIMATE_CORE");
        let registro = RegistroCelestial::new(root);
        let mapa = registro.mapa_estelar();
        assert!(mapa.contains("PLANETAS"), "El mapa debe listar planetas");
        assert!(
            mapa.contains("CONSTELACIONES"),
            "El mapa debe listar constelaciones"
        );
    }

    #[test]
    fn test_registro_planetas_de_constelacion() {
        let root = PathBuf::from("/home/soberano/NEXUS_ULTIMATE_CORE");
        let registro = RegistroCelestial::new(root);
        let planetas = registro
            .planetas_de_constelacion("Fénix")
            .expect("Fénix debe tener planetas indexados");
        assert!(!planetas.is_empty(), "Fénix debe tener al menos 1 planeta");
    }
}
