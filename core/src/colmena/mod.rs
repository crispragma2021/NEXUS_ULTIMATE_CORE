// 🐝 COLMENA — Enjambre gRPC Distribuido
// Madre + Hijos sincronizados por protocolo SwarmControl.
// Adaptado del legacy: reemplaza tokio-stream por futures::stream::unfold.

pub mod hijo;
pub mod madre;

pub use hijo::ColmenaHijo;
pub use madre::ColmenaMadre;

/// Módulo generado por tonic desde colmena.proto
pub mod proto {
    tonic::include_proto!("colmena");
}
