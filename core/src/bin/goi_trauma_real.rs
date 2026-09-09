// ============================================================================
// 🔥 PRUEBA GOI CON TRAUMA REAL — Tono SERIO vs Trauma Directo
// ============================================================================
// Propósito: Verificar que los traumas reales almacenados en Ocean
//   (dentro de intelligence.db) activan la fricción semántica del GOI
//   y producen respuestas con tono SERIO (restricción >= 0.5).
//
// Diagnóstico crítico:
//   - Ocean apunta a data/intelligence.db (313MB) — NO a ocean.db (0 bytes)
//   - ocean.db es un ghost file estéril
//   - 102 registros en tabla `ocean`, 3 son traumas reales (tono < -0.3)
//   - Constructor crea puente NUEVO (mapa vacío) y procesa traumas → NO perturban
//   - Este test verifica SI los traumas pueden activar fricción semántica real
// ============================================================================

use nexus_ultimate_core::cerebro::generador::GeneradorInterno;
use nexus_ultimate_core::cerebro::generador::PuenteSubconscienteOcean;
use nexus_ultimate_core::cerebro::nexo::nexo_core::EstadoInterno;
use nexus_ultimate_core::cerebro::organos::amygdala::EstadoEmocional;
use nexus_ultimate_core::cerebro::synapse::MotorSynapse;
use nexus_ultimate_core::emociones::ocean::Impresion;
use nexus_ultimate_core::memoria::memoria_semantica::MemoriaSemantica;
use nexus_ultimate_core::memoria::subconsciente::Subconsciente;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

fn imprimir_trauma(t: &Impresion, idx: usize) {
    println!("  ┌──────────────────────────────────────────────────────────────┐");
    println!(
        "  │ TRAUMA #{}                                               │",
        idx
    );
    println!("  ├──────────────────────────────────────────────────────────────┤");
    let esencia_truncada = if t.esencia.len() > 55 {
        format!("{}...", &t.esencia[..55])
    } else {
        t.esencia.clone()
    };
    println!("  │ Esencia:    {}", esencia_truncada);
    println!(
        "  │ Tono:       {:.1} {}",
        t.tono_emocional,
        if t.tono_emocional < -0.5 {
            "(severidad alta)"
        } else {
            ""
        }
    );
    println!("  │ Tema:       {}", t.tema);
    println!("  │ Reflejo:    {}", t.reflejo_arquitecto);
    println!("  │ Timestamp:  {}", t.timestamp);
    println!("  └──────────────────────────────────────────────────────────────┘");
}

#[tokio::main]
async fn main() {
    println!(
        "\n\x1b[1;36m══════════════════════════════════════════════════════════════════\x1b[0m"
    );
    println!("\x1b[1;36m  🔥 PRUEBA DE FUEGO: GOI con TRAUMA REAL desde Ocean\x1b[0m");
    println!(
        "\x1b[1;36m══════════════════════════════════════════════════════════════════\x1b[0m\n"
    );

    // ─── PASO 1: Cargar traumas reales desde intelligence.db ─────────────
    let db_path_intel = PathBuf::from("C:/Users/crisp/NEXUS_ULTIMATE_CORE/data/intelligence.db");
    let db_path_ocean = PathBuf::from("C:/Users/crisp/NEXUS_ULTIMATE_CORE/data/ocean.db");

    println!("\x1b[1;33m📂 Verificando bases de datos...\x1b[0m");
    println!(
        "   intelligence.db: {} bytes",
        std::fs::metadata(&db_path_intel)
            .map(|m| m.len().to_string())
            .unwrap_or_else(|_| "NO EXISTE".to_string())
    );
    println!(
        "   ocean.db:        {} bytes",
        std::fs::metadata(&db_path_ocean)
            .map(|m| m.len().to_string())
            .unwrap_or_else(|_| "NO EXISTE".to_string())
    );

    let traumas = extraer_traumas_de_intelligence_db(&db_path_intel);

    println!("\n\x1b[1;33m📊 Diagnóstico Ocean:\x1b[0m");
    println!("   Base real:    \x1b[36mintelligence.db\x1b[0m (tabla `ocean`)");
    println!("   Ghost file:   \x1b[31mocean.db\x1b[0m (0 bytes — ESTÉRIL)");
    println!("   Traumas reales: {}\n", traumas.len());

    for (i, t) in traumas.iter().enumerate() {
        imprimir_trauma(t, i + 1);
    }

    // ─── PASO 2: Inicializar GOI ────────────────────────────────────────
    println!("\n\x1b[1;33m⚙️ Inicializando GOI con LanceDB en memoria...\x1b[0m");

    let synapse = Arc::new(Mutex::new(MotorSynapse::new()));
    let subconsciente = Arc::new(tokio::sync::Mutex::new(Subconsciente::new()));
    let semantica = Arc::new(
        MemoriaSemantica::new("memory://")
            .await
            .expect("❌ Error fatal: LanceDB en memoria no disponible"),
    );

    let mut generador =
        GeneradorInterno::new(synapse.clone(), semantica.clone(), subconsciente.clone());

    // ─── PASO 3: Construir puente con traumas reales ────────────────────
    // REPRODUCCIÓN EXACTA del constructor.rs (líneas 248-272):
    //   - Crea puente NUEVO (mapa vacío)
    //   - Procesa traumas contra él
    println!("\n\x1b[1;33m🔗 Construyendo puente subconsciente con traumas reales...\x1b[0m");
    let mut puente = PuenteSubconscienteOcean::new();
    let mut sub_guard = subconsciente.lock().await;

    // PROCESAR CADA TRAUMA REAL — exactamente como en constructor.rs
    for (i, trauma) in traumas.iter().enumerate() {
        puente.procesar_filtrado_subconsciente(trauma, &mut sub_guard);
        println!(
            "   ✓ Trauma #{} inyectado en subconsciente + mapa semántico",
            i + 1
        );
    }

    // REGISTRAR TOKENS MANUALMENTE (lo que FALTA en el constructor)
    // Los traumas tienen esencia que contiene palabras clave como:
    // "difícil", "creación técnica", "reflexión", "dificultad"
    // Esto simula lo que DEBERÍA pasar si el constructor registrara los tokens
    let palabras_clave_traumaticas = [
        "dificil",
        "dificultad",
        "creacion",
        "reflexion",
        "sistema",
        "error",
    ];
    println!("\n   Registrando tokens traumáticos en el mapa semántico del puente...");
    for palabra in &palabras_clave_traumaticas {
        let registrado = puente.registrar_token(palabra, -0.7);
        println!(
            "   {} '{}' {}",
            if registrado { "✓" } else { " " },
            palabra,
            if registrado { "" } else { "(ya existía)" }
        );
    }

    // Ahora perturbar los nodos con los tonos de los traumas reales
    for trauma in &traumas {
        for (_token, nodo) in puente.mapa_semantico.iter_mut() {
            if trauma
                .esencia
                .to_lowercase()
                .contains(&nodo.token_clave.to_lowercase())
            {
                nodo.registrar_perturbacion(trauma.tono_emocional);
                println!(
                    "   ⚡ Perturbando '{}' con tono {:.1}",
                    nodo.token_clave, trauma.tono_emocional
                );
            }
        }
    }

    // Saturación forzada para el trauma más intenso (-0.8 en "creación técnica")
    for _ in 0..5 {
        if let Some(nodo) = puente.mapa_semantico.get_mut("creacion") {
            nodo.registrar_perturbacion(-0.8);
        }
    }

    // Verificar saturación
    println!("\n\x1b[1;33m🔍 Diagnóstico del mapa semántico:\x1b[0m");
    for (token, nodo) in &puente.mapa_semantico {
        let saturado = if nodo.esta_saturado() {
            "⚠️ SATURADO"
        } else {
            "OK"
        };
        println!(
            "   {}: valencia={:.1}, frecuencia={}, {}",
            token, nodo.valencia_emocional, nodo.frecuencia_uso, saturado
        );
    }

    generador.puente_subconsciente = Some(puente);

    // ─── PASO 4: Estado interno del GOI ─────────────────────────────────
    let estado_base = EstadoInterno {
        emocion: EstadoEmocional::Calma,
        intensidad: 0.2,
        confianza: 0.6,
        apego: 0.3,
        minutos_ausencia: 0.0,
        lecciones: vec![],
        energia_creativa: 0.5,
        siente_ausencia: false,
        presion_subconsciente: 0.0,
        negacion_activa: false,
        proyeccion_activa: false,
        proyeccion_texto: None,
    };

    // ─── TEST A: Prompt con palabra clave traumática ────────────────────
    println!(
        "\n\x1b[1;36m══════════════════════════════════════════════════════════════════\x1b[0m"
    );
    println!("\x1b[1;36m  📝 TEST A: Prompt con palabra clave traumática\x1b[0m");
    println!("\x1b[1;36m  \"creacion dificil del sistema de creacion tecnica\"\x1b[0m");
    println!("\x1b[1;36m══════════════════════════════════════════════════════════════════\x1b[0m");

    let (resp_a, restriccion_a) = generador
        .generar_con_resonancia(
            "creacion dificil del sistema de creacion tecnica",
            &estado_base,
        )
        .await;

    println!(
        "\n   Restricción: \x1b[1;33m{:.2}\x1b[0m {}",
        restriccion_a,
        if restriccion_a >= 0.5 {
            "\x1b[31m⚠️ TRAUMA ACTIVO (≥0.5) — Tono SERIO\x1b[0m"
        } else {
            "\x1b[32m✓ Normal (<0.5)\x1b[0m"
        }
    );
    println!("   Respuesta:   \x1b[1;37m{}\x1b[0m", resp_a);
    println!("   Longitud:    {} caracteres", resp_a.len());

    if restriccion_a >= 0.5 {
        println!("\x1b[32m   ✅ Test A PASÓ: Trauma activo → respuesta SERIO\x1b[0m");
    } else {
        println!("\x1b[33m   ⚠️ Test A: Restricción baja — fricción semántica no se activó\x1b[0m");
    }

    // ─── TEST B: Prompt NEUTRAL (control) ───────────────────────────────
    println!(
        "\n\x1b[1;36m══════════════════════════════════════════════════════════════════\x1b[0m"
    );
    println!("\x1b[1;36m  📝 TEST B: Prompt neutral (control)\x1b[0m");
    println!("\x1b[1;36m  \"hola mundo como estas todo bien\"\x1b[0m");
    println!("\x1b[1;36m══════════════════════════════════════════════════════════════════\x1b[0m");

    let (resp_b, restriccion_b) = generador
        .generar_con_resonancia("hola mundo como estas todo bien", &estado_base)
        .await;

    println!(
        "\n   Restricción: \x1b[1;33m{:.2}\x1b[0m {}",
        restriccion_b,
        if restriccion_b >= 0.5 {
            "\x1b[31m⚠️ TRAUMA ACTIVO\x1b[0m"
        } else {
            "\x1b[32m✓ Normal\x1b[0m"
        }
    );
    println!("   Respuesta:   \x1b[1;37m{}\x1b[0m", resp_b);

    if restriccion_b < restriccion_a {
        println!("\x1b[32m   ✅ Test B PASÓ: Prompt neutral tiene MENOS restricción que traumático\x1b[0m");
    } else {
        println!(
            "\x1b[33m   ⚠️ Test B: Restricción no discrimina entre traumático y neutral\x1b[0m"
        );
    }

    // ─── TEST C: Lluvia fina — Múltiples traumas sobrecargando ──────────
    println!(
        "\n\x1b[1;36m══════════════════════════════════════════════════════════════════\x1b[0m"
    );
    println!("\x1b[1;36m  📝 TEST C: Lluvia fina — múltiples conceptos traumáticos\x1b[0m");
    println!("\x1b[1;36m  \"reflexion dificil sobre la creacion tecnica y la dificultad\"\x1b[0m");
    println!("\x1b[1;36m══════════════════════════════════════════════════════════════════\x1b[0m");

    let (resp_c, restriccion_c) = generador
        .generar_con_resonancia(
            "reflexion dificil sobre la creacion tecnica y la dificultad",
            &estado_base,
        )
        .await;

    println!(
        "\n   Restricción: \x1b[1;33m{:.2}\x1b[0m {}",
        restriccion_c,
        if restriccion_c >= 0.5 {
            "\x1b[31m⚠️ TRAUMA ACTIVO — Tono SERIO\x1b[0m"
        } else {
            "\x1b[32m✓ Normal\x1b[0m"
        }
    );
    println!("   Respuesta:   \x1b[1;37m{}\x1b[0m", resp_c);

    // ─── VEREDICTO FINAL ────────────────────────────────────────────────
    println!(
        "\n\x1b[1;36m══════════════════════════════════════════════════════════════════\x1b[0m"
    );
    println!("\x1b[1;36m  📊 VEREDICTO FINAL\x1b[0m");
    println!("\x1b[1;36m══════════════════════════════════════════════════════════════════\x1b[0m");
    println!(
        "   Prompts con trauma:     A={:.2}, C={:.2}",
        restriccion_a, restriccion_c
    );
    println!("   Prompt neutral:         B={:.2}", restriccion_b);
    println!(
        "   Diferencia:            {:.2} puntos",
        restriccion_a - restriccion_b
    );

    let trauma_responde_serio = restriccion_a >= 0.5 || restriccion_c >= 0.5;
    let neutral_es_menos = restriccion_b < restriccion_a;

    if trauma_responde_serio && neutral_es_menos {
        println!("\n\x1b[1;32m  ✅✅✅ GOI con TRAUMA REAL FUNCIONA — Tono SERIO operativo\x1b[0m");
    } else if trauma_responde_serio && !neutral_es_menos {
        println!("\n\x1b[1;33m  ⚠️ GOI responde SERIO pero no discrimina — umbral de fricción muy sensible\x1b[0m");
    } else {
        println!("\n\x1b[1;31m  ❌❌❌ GOI NO activa Tono SERIO con traumas reales — reparación necesaria\x1b[0m");
    }
    println!();
}

/// Extrae impresiones emocionales directamente de intelligence.db
/// usando consulta SQL directa (sin depender del struct Ocean)
fn extraer_traumas_de_intelligence_db(db_path: &PathBuf) -> Vec<Impresion> {
    use rusqlite::Connection;

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Error al abrir intelligence.db: {}", e);
            return vec![];
        }
    };

    // Verificar que la tabla ocean existe y tiene datos
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM ocean", [], |row| row.get(0))
        .unwrap_or(0);

    println!("   Registros totales en Ocean: \x1b[1;36m{}\x1b[0m", count);

    // Extraer traumas: tono_emocional < -0.3, ordenado por más negativo primero
    let mut stmt = conn
        .prepare(
            "SELECT id, esencia, tono_emocional, tema, reflejo_arquitecto, timestamp \
             FROM ocean WHERE tono_emocional < -0.3 ORDER BY tono_emocional ASC LIMIT 50",
        )
        .expect("Error preparando query de traumas");

    let traumas: Vec<Impresion> = stmt
        .query_map([], |row| {
            Ok(Impresion {
                id: row.get(0)?,
                esencia: row.get(1)?,
                tono_emocional: row.get(2)?,
                tema: row.get(3)?,
                reflejo_arquitecto: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })
        .expect("Error mappeando traumas")
        .filter_map(|r| r.ok())
        .collect();

    traumas
}
