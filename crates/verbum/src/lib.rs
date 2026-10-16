// crates/verbum/src/lib.rs

//! Façade publique de Verbum : réexporte les crates du moteur, ne contient aucune logique.
//!
//! Les dépendances vont de la façade vers les crates, jamais l'inverse.

#![no_std]

pub use verbum_ecs as ecs;
pub use verbum_schedule as schedule;
