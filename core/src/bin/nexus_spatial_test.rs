use nexus_ultimate_core::spatial::SpatialEngine;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    let engine = SpatialEngine::new(root);

    let results = engine.full_scan().await;

    // Agrupar por directorio padre para ver los "Top 10"
    let mut dir_sizes: HashMap<String, u64> = HashMap::new();

    for item in results {
        let parent = item
            .path
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "/".to_string());

        *dir_sizes.entry(parent).or_insert(0) += item.size;
    }

    let mut sorted_dirs: Vec<_> = dir_sizes.into_iter().collect();
    sorted_dirs.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\n📊 [ZENITH] Top 10 Directorios más pesados:");
    println!("========================================");
    for (i, (path, size)) in sorted_dirs.into_iter().take(10).enumerate() {
        let size_mb = size as f64 / 1024.0 / 1024.0;
        println!("{}. [{:.2} MB] {}", i + 1, size_mb, path);
    }

    Ok(())
}
