//! Noyau ECS de Verbum.
//!
//! Crate `no_std` sans `alloc` (ADR-000, §4) : aucune allocation, y compris à la
//! construction. Les capacités sont fixées avant la simulation (ADR-001, point 5).

#![no_std]

mod command;
mod entity;

// Le stockage n'est consommé par aucun code non-test avant la Phase 4b (premier
// `SparseSet` intégré au `World`). Attente satisfaite tant que le module est inutilisé ;
// à retirer en Phase 4b.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "consommé par le World à partir de la Phase 4b")
)]
mod storage;

mod world;

pub use command::{BufferFull, CommandBuffer};
pub use entity::{CapacityExceeded, EntityId, StaleEntity};
pub use world::World;
