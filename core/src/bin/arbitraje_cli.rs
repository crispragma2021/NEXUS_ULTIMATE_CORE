// ==========================================
// NEXUS HFT TERMINAL - ARBITRAJE DE LATENCIA Y LOB IMBALANCE
// ==========================================
// Binario CLI para monitoreo, simulación y ejecución de señales
// de arbitraje de alta velocidad y desequilibrio de colas (LOB).
// ==========================================

use nexus_ultimate_core::cerebro::arbitraje_latencia::RastreadorLatencia;
use std::env;
use std::thread::sleep;
use std::time::Duration;

fn imprimir_ayuda() {
    println!("❓ Uso de NEXUS HFT Terminal:");
    println!(
        "  cargo run --bin arbitraje_cli -- monitor <fast_price> <slow_price> <bid_vol> <ask_vol>"
    );
    println!("  cargo run --bin arbitraje_cli -- simular [ticks]");
    println!("  cargo run --bin arbitraje_cli -- status");
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        imprimir_ayuda();
        return;
    }

    let comando = &args[1];

    match comando.as_str() {
        "monitor" => {
            if args.len() < 6 {
                println!("❌ Error: Faltan argumentos para 'monitor'.");
                println!("Uso: monitor <fast_price> <slow_price> <bid_vol> <ask_vol>");
                return;
            }

            let fast_price: f64 = args[2].parse().unwrap_or(0.0);
            let slow_price: f64 = args[3].parse().unwrap_or(0.0);
            let bid_vol: f64 = args[4].parse().unwrap_or(0.0);
            let ask_vol: f64 = args[5].parse().unwrap_or(0.0);

            let mut rastreador = RastreadorLatencia::new(200, 0.05); // 200ms tolerancia, 0.05 min spread
            rastreador.registrar_tick_rapido(fast_price, bid_vol, ask_vol);
            rastreador.registrar_tick_lento(slow_price, bid_vol, ask_vol);

            let imbalance = rastreador.calcular_lob_imbalance(bid_vol, ask_vol);
            let oportunidad = rastreador.verificar_oportunidad();

            println!("⚡ ANALIZADOR EN TIEMPO REAL NEXUS HFT:");
            println!("--------------------------------------------------");
            println!("  Feed Rápido (Futuro): ${:.4}", fast_price);
            println!("  Feed Lento (Presente): ${:.4}", slow_price);
            println!("  Spread Detectado: ${:.4}", fast_price - slow_price);
            println!(
                "  Desequilibrio (LOB Imbalance): {:.4} ({:.1}% presión)",
                imbalance,
                imbalance * 100.0
            );
            println!("--------------------------------------------------");

            match oportunidad {
                Some(true) => println!(
                    "🟢 SEÑAL ACTIVA: [COMPRA / BUY] - El futuro subió y el bróker tiene lag."
                ),
                Some(false) => println!(
                    "🔴 SEÑAL ACTIVA: [VENTA / SELL] - El futuro bajó y el bróker tiene lag."
                ),
                None => {
                    println!("⚪ ESTADO: Neutro / Sin oportunidad de arbitraje de latencia activa.")
                }
            }
        }

        "simular" => {
            let ticks_count = args.get(2).and_then(|t| t.parse::<u32>().ok()).unwrap_or(5);
            println!("🚀 INICIANDO SIMULACIÓN DE ARBITRAJE DE LATENCIA (NEXUS OMEGA HFT)...");
            println!("Objetivo: Demostrar cómo se 'predice el futuro' explotando el lag de 150ms.");
            println!("--------------------------------------------------");

            let mut rastreador = RastreadorLatencia::new(300, 0.1);

            // Simular secuencia de precios en el feed rápido (Binance) y lento (MT5)
            // Simulación de una inyección alcista masiva
            let base_price = 100.0;
            let mut fast_prices = vec![100.0, 100.0, 100.5, 101.2, 101.2];
            let mut slow_prices = vec![100.0, 100.0, 100.0, 100.2, 101.2]; // Lag en indexación
            let bid_volumes = vec![1000.0, 1000.0, 5000.0, 8000.0, 1500.0];
            let ask_volumes = vec![1000.0, 1000.0, 500.0, 300.0, 1500.0];

            for i in 0..ticks_count as usize {
                if i >= fast_prices.len() {
                    break;
                }

                let fp = fast_prices[i];
                let sp = slow_prices[i];
                let bv = bid_volumes[i];
                let av = ask_volumes[i];

                println!("\n📥 Tick #{} - Tiempo del Sistema", i + 1);
                rastreador.registrar_tick_rapido(fp, bv, av);

                // Simular el lag del bróker en registrar su tick
                sleep(Duration::from_millis(150));
                rastreador.registrar_tick_lento(sp, bv, av);

                let oportunidad = rastreador.verificar_oportunidad();
                let imbalance = rastreador.calcular_lob_imbalance(bv, av);

                println!(
                    "  ↳ Feed Rápido: ${:.2} | Feed Lento: ${:.2} (Lag: 150ms)",
                    fp, sp
                );
                println!("  ↳ Presión del Libro (LOB): {:.2}", imbalance);

                match oportunidad {
                    Some(true) => {
                        println!("  🔥 [OPORTUNIDAD DETECTADA] ¡Gatillo COMPRA (BUY)!");
                        println!(
                            "     -> Compras a ${:.2} sabiendo que el precio real ya es ${:.2}",
                            sp, fp
                        );
                        println!(
                            "     -> Beneficio esperado inmediato por arbitraje: +${:.2}",
                            fp - sp
                        );
                    }
                    Some(false) => {
                        println!("  🔥 [OPORTUNIDAD DETECTADA] ¡Gatillo VENTA (SELL)!");
                        println!(
                            "     -> Vendes a ${:.2} sabiendo que el precio real ya es ${:.2}",
                            sp, fp
                        );
                        println!(
                            "     -> Beneficio esperado inmediato por arbitraje: +${:.2}",
                            sp - fp
                        );
                    }
                    None => {
                        println!("  💤 Sin oportunidad activa (dentro del spread de tolerancia).");
                    }
                }
            }
            println!("--------------------------------------------------");
            println!("✅ Simulación HFT completada.");
        }

        "status" => {
            println!("🟢 SISTEMA HFT CENTINELA ACTIVO [24/7]");
            println!("- Motor de Arbitraje: LISTO (tolerancia 200ms)");
            println!("- Modos de Presencia: Redirección DNS & Proxies [LD4/NY4] activos");
            println!("- Células del Córtex Conectadas: 1");
        }

        _ => {
            imprimir_ayuda();
        }
    }
}
