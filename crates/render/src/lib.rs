// crates/render/src/lib.rs

//! Modèle de données du Renderer de Verbum.
//!
//! Crate `no_std` sans `alloc` (ADR-000, point 4) : aucune allocation, aucune
//! dépendance normale. Les types de ce crate sont atteignables depuis
//! `run_tick` (la composition de la représentation de frame est un système,
//! ADR-R02, point 3) : ils relèvent du noyau au sens d'ADR-000, point 1.
//!
//! Ce crate ne nomme ni le Shell ni aucun Backend (ADR-000, point 3 ;
//! ADR-R03, point 1). Il fournit :
//!
//! - [`Contribution`] : donnée plate émise par un système producteur ;
//! - [`Contributions`] : accumulateur à capacité fixe (contenu de la Ressource
//!   transitoire alimentée par les producteurs) et [`Full`], son refus de
//!   dépassement ;
//! - [`Frame`] : représentation composée, immuable une fois construite.
//!
//! Aucune règle d'ordre, aucune clé de tri, aucune notion de couche : le
//! corpus n'en fournit pas (plan C8).

#![no_std]

mod accumulator;
mod contribution;
mod frame;

pub use accumulator::{Contributions, Full};
pub use contribution::Contribution;
pub use frame::Frame;
