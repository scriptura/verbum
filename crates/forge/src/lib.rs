// crates/forge/src/lib.rs

//! Forge de Verbum : compilation AOT des formats humains en artefacts runtime.
//!
//! Crate `std`, extérieur au runtime : aucun crate runtime n'en dépend (I9). Seule la
//! Forge lit et valide les formats humains (I1, I2).

mod sprite;

pub use sprite::{ForgeError, compile_sprite};
