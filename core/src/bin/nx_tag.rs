use nexus_ultimate_core::spatial::SpatialEngine;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("❌ Uso: nx-tag <archivo> <etiqueta>");
        return Ok(());
    }

    let path = PathBuf::from(&args[1]);
    let tag = &args[2];

    let engine = SpatialEngine::new(std::env::current_dir()?);
    engine.tag_file(&path, tag);

    Ok(())
}
