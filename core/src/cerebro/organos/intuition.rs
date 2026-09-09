// 🦾 [PUENTE OMEGA] Puente anatómico de compatibilidad para intuition.
// El stub original (8 líneas) fue archivado en legacy/cerebro_disecados/.
// La implementación real de Intuición reside en organos/intuicion.rs.
// Este archivo mantiene la función despertar_intuicion accesible en
// crate::cerebro::organos::intuition para no romper referencias en brain/mod.rs.

pub use super::intuicion::{
    despertar_intuicion, Intuicion, IntuitionFeeling, IntuitionLobe, PatronError, SenialIntuitiva,
    TipoIntuicion,
};
