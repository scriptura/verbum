// crates/shell/src/lib.rs

//! Shell du moteur Verbum.
//!
//! Ce crate héberge la boucle externe qui pilote le moteur : cadencement des
//! itérations (pas fixe provisoire), acquisition des événements externes et
//! appel de la fermeture d'infrastructure une fois par tick.
//!
//! Il est piloté par des fermetures uniquement : il n'expose aucun trait
//! public et aucune abstraction de la plateforme. Il ne nomme aucun type du
//! noyau et n'a aucune dépendance.
//!
//! État actuel (unité U4) : boucle bornée ([`run_loop`]), cadencement à pas fixe
//! **provisoire** ([`FixedStepPacing`]) et boîte de réception à capacité fixe
//! ([`Inbox`], [`DepositHandle`], [`InboxFull`]) dont la coupure alimente le lot
//! remis à l'infrastructure à chaque itération.

mod inbox;
mod pacing;
mod shell;

pub use inbox::{DepositHandle, Inbox, InboxFull};
pub use pacing::FixedStepPacing;
pub use shell::run_loop;
