// crates/verbum-ecs/src/lib.rs

//! Noyau ECS de Verbum.
//!
//! Crate `no_std` sans `alloc` (ADR-000, §4) : aucune allocation, y compris à la
//! construction. Les capacités sont fixées avant la simulation (ADR-001, point 5).

#![no_std]

mod command;
mod component_set;
mod entity;
#[cfg(test)]
mod fixtures;
mod query;
mod resource;
mod storage;
mod values;
mod world;

pub use command::{BufferFull, CommandBuffer};
pub use component_set::ComponentSet;
pub use entity::{CapacityExceeded, EntityId, StaleEntity};
pub use query::ReadWriteQuery;
pub use resource::ResourceSlot;
pub use storage::{InsertError, SparseSet};
pub use values::{ValuesMut, ValuesView};
pub use world::{ApplyError, World};
