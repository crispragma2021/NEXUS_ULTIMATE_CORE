// ──────────────────────────────────────────────
// 👁️  SENTIDOS: Los 7 Sentidos de NEXUS
// ──────────────────────────────────────────────

// --- VISTA (Visión/Ojos) ---
pub mod omnipresent_vision;
pub mod vision_grafica;
pub mod vision_omega;
pub mod vision_sentinel;
pub mod vision_viva;

// --- 👻 VISIÓN FANTASMA (Sigilo Stealth) ---
pub mod vision_fantasma;

// --- 🖥️ OS COWORKER (Contexto de SO) ---
pub mod os_cowork;

// --- OÍDO (Audición/Oídos) ---
pub mod neuro_ear;

// --- OLFATO (Nariz) ---
pub mod nexus_scent;

// --- GUSTO (Lengua) ---
pub mod nexus_palate;

// --- PROPIOCEPCIÓN (Sentido corporal / kinestésico) ---
pub mod propiocepcion;

// Aliases de compatibilidad para el core legacy
pub mod anclaje_sensorial;
pub mod nexus_acoustic;
pub use neuro_ear as hearing;
pub use nexus_palate as taste;
pub use nexus_scent as smell;
