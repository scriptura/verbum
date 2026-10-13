// crates/verbum-schedule/src/lib.rs

//! Scheduler of the Verbum engine (stage 1).
//!
//! Builds an immutable execution model from system contracts alone
//! (ADR-003, ADR-004). `#![no_std]`, no `alloc` (ADR-000), no `unsafe`
//! (workspace lint).

#![no_std]
