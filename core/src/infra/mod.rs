// ⚙️ SISTEMA ESQUELÉTICO DE NEXUS
// Infraestructura, kernel, red, navegadores, gateway MCP

pub mod analizador_nexus;
pub mod arsenal;
pub mod body;
pub mod boot;
pub mod browser_native;
pub mod buscador_omega;
pub mod buscador_web;
pub mod cloudcode_tunnel;
pub mod curador_datos;
pub mod geo_hunter;
pub mod ghost_vm;
pub mod hardware;
pub mod herramientas_nativas;
pub mod ingesta_mercado;
pub mod kernel;
pub mod mcp_gateway;
pub mod metricas_financieras;
pub mod mundo_interno;
pub mod navegador_soberano;
pub mod network;
pub mod paths;
pub mod policy;
pub mod puente_ipc;
pub mod shadowcrawl;
pub mod sms_activate;
pub mod web_pool;
pub mod web_socket;

// 🔱 Cloudflare Bypass — Transmutación de cloudscraper a Rust puro
pub mod cloudscraper_rs;

// 🔱 Verificador OSINT de emails — Transmutación de holehe a Rust puro
pub mod holehe_rs;

// 🔱 Arsenal de Trading Soberano — ccxt_rs
pub mod trading;

pub use crate::nexus_telegram;
