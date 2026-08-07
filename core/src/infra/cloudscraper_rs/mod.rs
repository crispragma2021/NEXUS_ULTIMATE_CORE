// 🔱 cloudscraper_rs — Transmutación Rust Pura de cloudscraper (Cloudflare bypass)
// Cero dependencias externas nuevas. Usa reqwest + chromiumoxide del arsenal existente.
// Estrategia: Intento directo con reqwest + headers de navegador; si falla con CF challenge,
// resuelve mediante chromiumoxide headless.

pub mod scraper;

pub use scraper::*;
