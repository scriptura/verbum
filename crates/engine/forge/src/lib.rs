// crates/engine/forge/src/lib.rs

//! Forge de Verbum : compilation AOT des formats humains en artefacts runtime.
//!
//! Crate `std`, extérieur au runtime : aucun crate runtime n'en dépend (I9). Seule la
//! Forge lit et valide les formats humains (I1, I2).

mod batch;
mod font;
mod id;
mod map;
mod script;
mod sprite;
mod tileset;

pub use batch::{AssetKind, AssetSource, CompiledSet, compile_assets};
pub use font::compile_font;
pub use script::compile_script;
pub use sprite::{ForgeError, compile_sprite};
pub use tileset::compile_tileset;
