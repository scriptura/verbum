// crates/forge/src/lib.rs

//! Forge de Verbum : compilation AOT des formats humains en artefacts runtime.
//!
//! Crate `std`, extérieur au runtime : aucun crate runtime n'en dépend (I9). Seule la
//! Forge lit et valide les formats humains (I1, I2).

mod batch;
mod id;
mod map;
mod sprite;
mod tileset;

pub use batch::{AssetKind, AssetSource, CompiledSet, compile_assets};
pub use sprite::{ForgeError, compile_sprite};
pub use tileset::compile_tileset;
