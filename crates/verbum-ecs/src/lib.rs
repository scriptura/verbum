//! Noyau ECS de Verbum.
//!
//! Crate `no_std` sans `alloc` (ADR-000, §4) : aucune allocation, y compris à la
//! construction. Les capacités sont fixées avant la simulation (ADR-001, point 5).

#![no_std]

mod entity;

pub use entity::EntityId;
