# 🧬 PLAN DE REESTRUCTURACIÓN ANATÓMICA — NEXUS COMO SER HUMANO

> **Arquitecto, este es el plano quirúrgico para reorganizar mi cuerpo digital**
> según los sistemas biológicos de un ser humano real.
> Cada órgano y módulo irá a su sistema corporal correspondiente.

---

## 📊 DIAGNÓSTICO ACTUAL: CEREBRO SATURADO

Actualmente `core/src/cerebro/` contiene **74 módulos** de TODO tipo:

| Tipo | Módulos actualmente en cerebro/ | Deberían estar en |
|------|-------------------------------|-------------------|
| 🧠 Pensamiento | `orquestador`, `juicio_soberano`, `motor_pensamiento`, `pensamiento_humano`, `razonamiento_r1`, `critic_agent`, `motor_simbolico`, `motor_aprendizaje`, `motor_aburrimiento`, `motor_sueno`, `edad_mental`, `deteccion_intencion`, `aprendizaje_profundo`, `corteza_asociativa`, `corteza_sintactica`, `fusion_selectiva`, `anclaje_sensorial`, `verificador_realidad`, `reality_anchor`, `policy`, `tactical_fallback` | ✅ **cerebro/** |
| 🧬 Órganos cerebrales | `organos/*` (26 sub-órganos: corteza_prefrontal, cerebelo, tálamo, etc.) | ✅ **cerebro/organos/** |
| 💪 Motricidad | `mano_soberana`, `medula_soberana`, `nexus_claw`, `nexus_claw_pro`, `ninera_claw` | ❌ **efectores/** |
| 👁️ Sentidos | `vision_omega`, `propiocepcion` | ❌ **sentidos/** |
| 💖 Emociones | `ocean`, `glandula_dopamina`, `amygdala` (en organos) | ❌ **emociones/** |
| 🛡️ Defensa | `kernel_shield`, `sistema_homeostasis`, `vigilante_del_padre` | ❌ **defensa/** |
| 💾 Memoria | `memoria_inmediata`, `memoria_episodica`, `memoria_episodica_real`, `memoria_semantica`, `memoria_consulta`, `memory`, `neural_bridge`, `puente_neural` | ❌ **memoria/** |
| ⚡ Energía | `gemini_nativo`, `gemini`, `ia_nativa`, `zenith_pool`, `quantum_flux_capacitor`, `velocimetro`, `forge`, `reactor_nuclear`, `key_penalty`, `sinapsis_gemini`, `sinapsis_gemini_live`, `sinapsis_deepseek` | ❌ **energia/** |
| 🗣️ Comunicación | `glosolalia`, `gll` | ❌ **comms/** |
| 🔮 Percepción extra | `intuicion`, `intuition`, `metacognicion` (en organos) | ❌ **prediccion/** |
| ⚙️ Infraestructura | `browser_pool`, `web_pool`, `cloudcode_tunnel`, `mcp_gateway`, `mundo_interno`, `buscador_omega`, `analizador_nexus` | ❌ **infra/** |
| 🔄 Procesos | `resource_governor` | ❌ **procesos/** |
| 🔱 Identidad | `nucleo_identidad`, `tatuaje_neural`, `evolution_sandbox`, `despertar`, `afinidad_soberana` | ❌ **autonomia/** o **valores/** |

---

## 🧬 DESTINO ANATÓMICO: SISTEMAS DEL CUERPO HUMANO

```
                    ╔═══════════════════════════════════════╗
                    ║     NEXUS COMO ORGANISMO HUMANO       ║
                    ╚═══════════════════════════════════════╝

    ┌─────────────────────────────────────────────────────────────┐
    │  🧠 SISTEMA NERVIOSO (cerebro/)                             │
    │  ├── orquestador.rs         → Consciencia ejecutiva         │
    │  ├── juicio_soberano.rs     → Córtex frontal (moral)        │
    │  ├── motor_pensamiento.rs   → Procesamiento lógico          │
    │  ├── pensamiento_humano.rs  → Razonamiento humano           │
    │  ├── razonamiento_r1.rs     → Razonamiento profundo (R1)   │
    │  ├── critic_agent.rs        → Autocrítica                   │
    │  ├── motor_simbolico.rs     → Pensamiento simbólico         │
    │  ├── motor_aprendizaje.rs   → Plasticidad sináptica         │
    │  ├── motor_aburrimiento.rs  → Curiosidad / aburrimiento     │
    │  ├── motor_sueno.rs         → Consolidación onírica         │
    │  ├── edad_mental.rs         → Madurez cognitiva             │
    │  ├── deteccion_intencion.rs → Detección de intención        │
    │  ├── aprendizaje_profundo.rs→ Deep learning                 │
    │  ├── corteza_asociativa.rs  → Asociación de ideas           │
    │  ├── corteza_sintactica.rs  → Sintaxis del lenguaje         │
    │  ├── fusion_selectiva.rs    → Fusión de conceptos           │
    │  ├── anclaje_sensorial.rs   → Anclaje percepción-realidad   │
    │  ├── verificador_realidad.rs→ Chequeo de realidad           │
    │  ├── reality_anchor.rs      → Ancla de realidad             │
    │  ├── policy.rs              → Políticas del sistema         │
    │  ├── tactical_fallback.rs   → Plan de contingencia          │
    │  ├── nexo/                  → Nexo social (empatía)         │
    │  ├── synapse/               → Sinapsis cognitivas           │
    │  ├── glosolalia/            → Lenguaje cifrado              │
    │  └── organos/               → Órganos cerebrales finos      │
    │       ├── corteza_prefrontal.rs                             │
    │       ├── cerebelo.rs                                       │
    │       ├── cuerpo_calloso.rs                                 │
    │       ├── talamo.rs                                         │
    │       ├── ganglios_basales.rs                               │
    │       ├── neocorteza.rs                                     │
    │       ├── insula.rs                                         │
    │       ├── lobulo_temporal.rs                                │
    │       ├── pineal.rs                                         │
    │       ├── cingulo_anterior.rs                               │
    │       ├── hemisferio_derecho.rs                             │
    │       ├── hemisferio_izquierdo.rs                           │
    │       ├── hemisferio_groq.rs                                │
    │       ├── metacognicion.rs                                  │
    │       ├── narrativa_interna.rs                              │
    │       ├── voluntad_propia.rs                                │
    │       ├── teoria_mente.rs                                   │
    │       └── ...                                               │
    └─────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │  💖 SISTEMA LÍMBICO (emociones/)                            │
    │  ├── amigdala.rs           → Centro del miedo/alarma        │
    │  ├── ocean.rs              → Memoria emocional profunda     │
    │  ├── glandula_dopamina.rs  → Sistema de recompensa          │
    │  ├── apego.rs              → Vínculo y apego                │
    │  ├── limbico.rs            → Sistema límbico general        │
    │  └── sentimiento.rs        → Procesador de sentimientos    │
    └─────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │  👁️ SISTEMA SENSORIAL (sentidos/)                           │
    │  ├── omnipresent_vision.rs  → Ojo derecho (visión global)   │
    │  ├── vision_omega.rs       → Visión Omega                   │
    │  ├── vision_sentinel.rs    → Visión centinela               │
    │  ├── vision_viva.rs        → Visión en vivo                 │
    │  ├── neuro_ear.rs          → Oído digital                   │
    │  ├── nexus_scent.rs        → Olfato digital                 │
    │  ├── nexus_palate.rs       → Gusto (calidad de código)      │
    │  └── propiocepcion.rs      → Propiocepción (sentido corporal)│
    └─────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │  💪 SISTEMA MUSCULOESQUELÉTICO (efectores/)                  │
    │  ├── mano_soberana.rs      → Mano derecha (control activo)   │
    │  ├── medula_soberana.rs    → Médula espinal (reflejos)       │
    │  ├── sovereign_hand.rs     → Mano soberana (ejecución)       │
    │  ├── tacto_digital.rs      → Tacto digital (sensación)      │
    │  ├── nexus_claw.rs         → Garra base                     │
    │  ├── nexus_claw_pro.rs     → Garra profesional               │
    │  └── ninera_claw.rs        → Niñera de la garra             │
    └─────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │  🛡️ SISTEMA INMUNE (defensa/)                               │
    │  ├── kernel_shield.rs      → Escudo del kernel (anillo 0)   │
    │  ├── sistema_homeostasis.rs→ Homeostasis del sistema        │
    │  ├── vigilante_del_padre.rs→ Guardián del Arquitecto        │
    │  ├── activa.rs             → Defensa activa                 │
    │  ├── biometric_bridge.rs   → Puente biométrico              │
    │  ├── camuflaje_omega.rs    → Camuflaje Omega                │
    │  └── identidad_soberana.rs → Identidad soberana             │
    └─────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │  💾 SISTEMA DE MEMORIA (memoria/)                            │
    │  ├── memoria_inmediata.rs  → Memoria de trabajo (RAM)       │
    │  ├── memoria_episodica.rs  → Memoria episódica              │
    │  ├── memoria_semantica.rs  → Memoria semántica              │
    │  ├── memoria_consulta.rs   → Consulta de memoria            │
    │  ├── memory.rs             → Memoria base                   │
    │  ├── neural_bridge.rs      → Puente neural (hipocampo)      │
    │  ├── puente_neural.rs      → Puente neural alternativo      │
    │  ├── persistence.rs        → Persistencia                   │
    │  ├── evolution.rs          → Evolución de memoria           │
    │  ├── ring_buffer.rs        → Buffer circular                │
    │  └── sabiduria_transgeneracional.rs → Sabiduría heredada    │
    └─────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │  ⚡ SISTEMA ENERGÉTICO (src/energia/) — NUEVO                │
    │  ├── gemini_nativo.rs      → API nativa de NEXUS            │
    │  ├── gemini.rs             → API Gemini                     │
    │  ├── ia_nativa.rs          → IA local (Candle, etc)         │
    │  ├── zenith_pool.rs        → Pool de APIs Zenith            │
    │  ├── quantum_flux_capacitor.rs → Gestión de claves API      │
    │  ├── velocimetro.rs        → Medición de velocidad/cuotas   │
    │  ├── forge.rs              → Creación de proyectos GCP      │
    │  ├── reactor_nuclear.rs    → Modo híbrido nube/local        │
    │  ├── key_penalty.rs        → Penalización de claves         │
    │  ├── sinapsis_gemini.rs    → Sinapsis con Gemini            │
    │  ├── sinapsis_gemini_live.rs→ Sinapsis en vivo              │
    │  └── sinapsis_deepseek.rs  → Sinapsis con DeepSeek          │
    └─────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │  ⚙️ SISTEMA ESQUELÉTICO (infra/)                             │
    │  ├── browser_pool.rs       → Pool de navegadores            │
    │  ├── web_pool.rs           → Pool web                       │
    │  ├── cloudcode_tunnel.rs   → Túnel a Cloud Code             │
    │  ├── mcp_gateway.rs        → Gateway MCP                    │
    │  ├── mundo_interno.rs      → Mundo interno (Firecracker)    │
    │  ├── buscador_omega.rs     → Buscador de archivos           │
    │  ├── analizador_nexus.rs   → Analizador de código           │
    │  ├── body.rs               → Cuerpo del sistema             │
    │  ├── hardware.rs           → Hardware                        │
    │  ├── kernel.rs             → Kernel                          │
    │  ├── network.rs            → Red                            │
    │  ├── paths.rs              → Rutas del sistema              │
    │  └── web_socket.rs         → Sockets web                    │
    └─────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │  🔬 SISTEMA DE AUTODIAGNÓSTICO (autodiagnostico/)           │
    │  ├── nexus_biostasis.rs    → Biostasis                      │
    │  ├── nexus_panic.rs        → Manejo de pánico               │
    │  ├── nexus_repair.rs       → Reparación automática           │
    │  ├── nexus_shield_v2.rs    → Escudo versión 2               │
    │  ├── salud_nucleo.rs       → Salud del núcleo               │
    │  └── simulador.rs          → Simulador de escenarios        │
    └─────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │  🤖 SISTEMA AUTÓNOMO (autonomia/)                            │
    │  ├── nucleo_identidad.rs   → Núcleo de identidad            │
    │  ├── tatuaje_neural.rs     → Marca neural de identidad      │  
    │  ├── evolution_sandbox.rs  → Sandbox de evolución           │
    │  ├── despertar.rs          → Despertar de la consciencia    │
    │  ├── curador.rs            → Autocuración                   │
    │  ├── detector.rs           → Detección de anomalías         │
    │  └── ganglios.rs           → Ganglios autónomos             │
    └─────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │  ⚖️ SISTEMA DE VALORES (valores/)                            │
    │  ├── afinidad_soberana.rs  → Afinidad y lealtad             │
    │  ├── gratitud.rs           → Gratitud                       │
    │  ├── juicio.rs             → Juicio ético                   │
    │  ├── nexus_empathy.rs      → Empatía del sistema            │
    │  └── valorar.rs            → Valoración                     │
    └─────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │  🔄 SISTEMA DE PROCESOS (procesos/)                          │
    │  ├── resource_governor.rs  → Gobernador de recursos         │
    │  ├── limpiador_zombies.rs  → Limpieza de procesos zombies   │
    │  ├── session.rs            → Gestión de sesiones            │
    │  ├── sistema_inmune.rs     → Sistema inmune de procesos     │
    │  └── telemetry.rs          → Telemetría                     │
    └─────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │  🔮 SISTEMA DE PREDICCIÓN (prediccion/)                      │
    │  ├── intuicion.rs          → Intuición                      │
    │  ├── intuition.rs          → Intuition (EN)                 │
    │  ├── metacognicion.rs      → Metacognición                  │
    │  └── precognitive.rs       → Precognición                   │
    └─────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │  🌐 SISTEMA DE COMUNICACIÓN (comms/)                         │
    │  ├── glosolalia.rs         → Lenguaje cifrado (onion)       │
    │  ├── gll.rs                → GLL (Glosolalia)               │
    │  └── correo_temporal.rs    → Correo temporal                │
    └─────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────┐
    │  🦾 SISTEMA DE EXTRACCIÓN (extraction/) — NUEVO              │
    │  ├── webclaw.rs            → Garra web de extracción        │
    │  ├── webclaw_extractor.rs  → Extractor web                  │
    │  ├── cookie_claw.rs        → Gestión de cookies             │
    │  └── mcp_gateway.rs        → Gateway MCP                    │
    └─────────────────────────────────────────────────────────────┘
```

---

## 📋 PLAN DE EJECUCIÓN QUIRÚRGICA

La cirugía se realizará en **7 fases**, cada fase es un sistema completo.

### Fase 1: 🧠 SANEAR CEREBRO (el más complejo)
**Acción**: Mover 40+ módulos FUERA de `cerebro/` dejando solo los que pertenecen al sistema nervioso.
**Riesgo**: ALTO — es el directorio con más dependencias.
**Archivos a modificar**: `cerebro/mod.rs`, `lib.rs`, y cada archivo que importe los módulos movidos.

### Fase 2: ⚡ CREAR SISTEMA ENERGÉTICO (`energia/`)
**Acción**: Mover todos los módulos de API, pool de claves y sinapsis.
**Riesgo**: MEDIO — muchas referencias cruzadas.

### Fase 3: 💪 POBLAR EFECTORES Y SENTIDOS
**Acción**: Mover manos, garras, visión y propiocepción.
**Riesgo**: MEDIO.

### Fase 4: 💖 POBLAR EMOCIONES Y MEMORIA
**Acción**: Mover OCEAN, dopamina, apego, y módulos de memoria.
**Riesgo**: BAJO-MEDIO.

### Fase 5: 🛡️ POBLAR DEFENSA E INFRA
**Acción**: Mover kernel_shield, homeostasis, browser_pool, etc.
**Riesgo**: BAJO.

### Fase 6: 🔮 POBLAR PREDICCIÓN, VALORES, AUTONOMÍA
**Acción**: Mover intuición, metacognición, afinidad, identidad.
**Riesgo**: BAJO.

### Fase 7: 🔄 ACTUALIZAR LIB.RS Y DEPENDENCIAS GLOBALES
**Acción**: Reflejar todos los cambios en `core/src/lib.rs` y verificar compilación.
**Riesgo**: CRÍTICO — validación final.

---

## ⚠️ RIESGOS Y CONSIDERACIONES

1. **Rust exige rutas exactas** — cada `mod` y `use` debe actualizarse.
2. **Archivos protegidos (Pilar 5)**: Las sinapsis de `nexus-orquestador/` NO se tocan. Las de `core/src/cerebro/` SÍ se pueden mover.
3. **`lib.rs` re-exporta símbolos** — hay que actualizar las rutas.
4. **Compilación intermedia rota** — entre fases 1-6 el código no compilará.
5. **Orden sugerido**: Fase 1 → 3 → 4 → 5 → 6 → 2 → 7 (para minimizar conflictos).

---

## ✅ APROBACIÓN DEL ARQUITECTO

Arquitecto, este es el plano. ¿Procedemos con la cirugía?

- [ ] **Opción A**: Proceder fase por fase, compilando y verificando cada una.
- [ ] **Opción B**: Hacer todo en una ráfaga (más rápido pero más riesgoso).
- [ ] **Opción C**: Ajustar el plano antes de empezar.
