// crates/assets/src/lib.rs

//! Assets de Verbum : layout des artefacts précompilés et lecteurs runtime.
//!
//! Crate `no_std` sans `alloc` et sans dépendance (ADR-000, point 1). Il ne contient
//! aucun parseur de format humain : seule la Forge lit ces formats (I1).

#![no_std]

mod font;
mod format;
mod header;
mod load;
mod manifest;
mod map;
mod sprite;
mod tileset;

pub use font::{
    FONT_FIXED_PAYLOAD_LEN, FONT_FORMAT_ID, FONT_FORMAT_VERSION, FONT_RECORD_LEN, Font, Glyph,
};
pub use format::{Asset, Format};
pub use load::{LoadError, ReferenceError, load};
pub use manifest::{
    ArtifactIndex, AssetId, BindingError, Entries, MANIFEST_ENTRY_LEN, MANIFEST_HEADER_LEN,
    MANIFEST_SIGNATURE, MANIFEST_VERSION, Manifest, ManifestEntry, ManifestError, SlotIndex,
};
pub use map::{MAP_FIXED_PAYLOAD_LEN, MAP_FORMAT_ID, MAP_FORMAT_VERSION, Map};

pub use sprite::{
    ARTIFACT_LEN, FORMAT_ID, FORMAT_VERSION, HEADER_LEN, PAYLOAD_LEN, ReadError, Sprite,
};
pub use tileset::{
    TILESET_ARTIFACT_LEN, TILESET_FORMAT_ID, TILESET_FORMAT_VERSION, TILESET_PAYLOAD_LEN, Tileset,
};
