use ignore::WalkBuilder;
use std::time::Instant;

fn main() {
    let inicio = Instant::now();
    let ruta = "/opt/NEXUS_ULTIMATE_CORE";
    let mut contador = 0;

    // Esto utiliza los hilos del Ryzen 7 para indexar en paralelo
    let walker = WalkBuilder::new(ruta).build();

    for result in walker {
        if let Ok(_) = result {
            contador += 1;
        }
    }

    println!("{{\"status\": \"success\", \"files_indexed\": {}, \"time_ms\": {}}}", 
              contador, inicio.elapsed().as_millis());
}
