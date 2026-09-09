use nexus_ultimate_core::cerebro::organos::amygdala::EstadoEmocional;
use nexus_ultimate_core::cerebro::organos::cuerpo_calloso::CuerpoCalloso;
use nexus_ultimate_core::emociones::ocean::Impresion;
use nexus_ultimate_core::valores::juicio_soberano::JuicioSoberano;

#[tokio::main]
async fn main() {
    let calloso = CuerpoCalloso::new();
    let juicio = JuicioSoberano::new();

    let logica = Some("El proceso 43211 está saturando los E-Cores.".to_string());
    let creativa =
        Some("Los hilos de eficiencia susurran una sobrecarga en el corazón del i7.".to_string());

    println!("--- 🧪 PRUEBA DE ESTRÉS: CUERPO CALLOSO ---");

    println!("\n[SÍNTESIS BAJO MIEDO]");
    let s_miedo = calloso.unificar(logica.clone(), creativa.clone(), EstadoEmocional::Miedo);
    println!("{}", s_miedo.sintesis);

    println!("\n[SÍNTESIS BAJO ALERTA]");
    let s_interes = calloso.unificar(logica.clone(), creativa.clone(), EstadoEmocional::Alerta);
    println!("{}", s_interes.sintesis);

    println!("\n--- ⚖️ SIMULACIÓN: MEMORIA DEL DOLOR ---");

    let recuerdos = vec![(
        Impresion {
            id: 1,
            esencia: "El sistema entró en pánico tras purgar logs".into(),
            tono_emocional: -0.85,
            tema: "error_kernel".into(),
            reflejo_arquitecto: "Frustración".into(),
            timestamp: "2026".into(),
        },
        0.9, // Similitud del 90%
    )];

    let riesgo = juicio.evaluar_riesgo_por_experiencia(0.5, &recuerdos);
    println!("Acción: Purgar logs del sistema");
    println!("Riesgo calculado: {:.2}", riesgo);
    if !juicio.dictaminar("purgar logs", riesgo) {
        println!("✅ ÉXITO: El Juicio Soberano ha bloqueado la acción por trauma previo.");
    }
}
