// crates/engine/shell/tests/temporal_support/mod.rs

//! Harnais d'horloge scriptée pour les tests de la politique temporelle (ADR-S04).
//!
//! Inclus par `#[path]` depuis les fichiers de tests ; réutilisé par les unités de pilotes.
//!
//! - l'horloge rend la suite scriptée de `Duration`, dans l'ordre, et **compte ses lectures** ;
//! - la suite doit être **non décroissante** : le harnais **panique à la construction** pour
//!   toute suite décroissante (précondition de monotonie C-14) ;
//! - une lecture au-delà de la suite scriptée panique : une lecture d'horloge en trop est
//!   donc un échec de test, et le nombre exact de lectures est vérifiable ;
//! - aucun `sleep`, aucun `Instant`, aucune horloge murale.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

/// Compteur de lectures partagé avec l'horloge scriptée.
pub struct Reads(Rc<Cell<usize>>);

impl Reads {
    /// Nombre de lectures effectuées jusqu'ici.
    pub fn count(&self) -> usize {
        self.0.get()
    }
}

/// Horloge scriptée depuis des durées. Panique si la suite décroît.
pub fn scripted(sequence: &[Duration]) -> (impl FnMut() -> Duration + use<>, Reads) {
    for pair in sequence.windows(2) {
        assert!(
            pair[0] <= pair[1],
            "suite d'horloge décroissante refusée : {:?} puis {:?}",
            pair[0],
            pair[1]
        );
    }
    let sequence = sequence.to_vec();
    let count = Rc::new(Cell::new(0usize));
    let shared = Rc::clone(&count);
    let clock = move || {
        let index = shared.get();
        assert!(index < sequence.len(), "lecture d'horloge hors script");
        shared.set(index + 1);
        sequence[index]
    };
    (clock, Reads(count))
}

/// Horloge scriptée depuis des nanosecondes.
pub fn scripted_ns(sequence: &[u64]) -> (impl FnMut() -> Duration + use<>, Reads) {
    let durations: Vec<Duration> = sequence
        .iter()
        .map(|&ns| Duration::from_nanos(ns))
        .collect();
    scripted(&durations)
}
