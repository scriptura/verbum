// crates/schedule/src/lib.rs

//! Scheduler of the Verbum engine (stage 1).
//!
//! Builds an immutable execution model from system contracts alone
//! (ADR-003, ADR-004). `#![no_std]`, no `alloc` (ADR-000), no `unsafe`
//! (workspace lint).

#![no_std]

pub mod access;
pub mod contract;
pub mod data;
pub mod model;
pub mod order;
pub mod scheduler;
pub mod system;
