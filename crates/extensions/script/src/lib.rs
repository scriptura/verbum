// crates/extensions/script/src/lib.rs

//! Exécution des scripts de Verbum : état, émissions, table et fonction `step`.
//!
//! Crate `no_std` sans `alloc` (ADR-000, point 4) ; seule dépendance :
//! `verbum-assets` (lecteur de `script.bin`). Les types de ce crate sont
//! atteignables depuis `run_tick` : ils relèvent du noyau au sens d'ADR-000,
//! point 1, comme ceux de `verbum-render`.
//!
//! Ce crate ne nomme ni `World`, ni Scheduler, ni système : il fournit des
//! données et une fonction pure. Il contient :
//!
//! - [`ScriptState`] : état d'exécution d'une instance (composant `Copy`),
//!   avec son encodage canonique de [`ENCODED_LEN`] octets ;
//! - [`Emissions`] : accumulateur d'émissions à capacité fixe et [`Full`] ;
//! - [`ScriptTable`] : table de scripts indexée directement par le slot ;
//! - [`emissions_bound_holds`] : borne de capacité des émissions ;
//! - [`step`] : unique transition d'un état, sans état externe.

#![no_std]

mod emissions;
mod state;
mod step;
mod table;

pub use verbum_assets::{SCRIPT_MAX_STEPS_PER_TICK, SCRIPT_REGISTERS};

pub use emissions::{Emission, Emissions, Full, emissions_bound_holds};
pub use state::{ENCODED_LEN, Fault, ScriptState, SlotTooLarge, StateError, Status};
pub use step::step;
pub use table::{ScriptTable, TableError};
