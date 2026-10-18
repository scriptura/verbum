// crates/assets/src/lib.rs

//! Assets de Verbum : layout des artefacts précompilés et lecteurs runtime.
//!
//! Crate `no_std` sans `alloc` et sans dépendance (ADR-000, point 1). Il ne contient
//! aucun parseur de format humain : seule la Forge lit ces formats (I1).

#![no_std]

mod load;
mod manifest;
mod sprite;

pub use load::{LoadError, load};
pub use manifest::{
    ArtifactIndex, AssetId, BindingError, Entries, MANIFEST_ENTRY_LEN, MANIFEST_HEADER_LEN,
    MANIFEST_SIGNATURE, MANIFEST_VERSION, Manifest, ManifestEntry, ManifestError, SlotIndex,
};

pub use sprite::{
    ARTIFACT_LEN, FORMAT_ID, FORMAT_VERSION, HEADER_LEN, PAYLOAD_LEN, ReadError, Sprite,
};
