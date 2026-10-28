// crates/render/src/frame.rs

//! Représentation de frame : valeur composée, immuable, à capacité fixe.
//!
//! Une `Frame` est construite d'un seul coup par `Frame::compose` (la
//! composition, ADR-R02, point 3) ou vaut `Frame::empty()` (état initial
//! déterministe). Son API publique ne contient aucune méthode `&mut self` :
//! une fois composée, elle ne change plus (ADR-R02, point 4) et se transmet
//! telle quelle en aval, par emprunt `&Frame` (plan A-2).
//!
//! La `Frame` possède ses données : elle ne référence pas l'accumulateur dont
//! elle provient. Elle ne contient aucune référence, aucun handle et aucune
//! propriété partagée (ADR-000, point 3).
//!
//! La séquence suit l'ordre d'émission, déterministe mais **non contractuel** :
//! aucun consommateur ne doit s'y fier (plan C8).
//!
//! # Aucune modification à travers `&Frame` (RP-5)
//!
//! Les blocs `compile_fail` ci-dessous ne portent **aucun code d'erreur** :
//! rustdoc ne contrôle pas ce code sur Rust stable. Un bloc `compile_fail`
//! garantit seulement que le code échoue à la compilation ; chacun a un jumeau
//! qui compile, et leur sensibilité est établie par mutation (voir (b)).
//! Les codes observés par compilation directe (E0594, E0599, E0594) sont un
//! constat ponctuel, non une assertion de test.
//!
//! (a) Affectation d'un élément de la tranche observée à travers `&Frame` :
//!
//! ```compile_fail
//! use verbum_render::{Contribution, Frame};
//!
//! fn tente(frame: &Frame<2>) {
//!     frame.as_slice()[0] = Contribution {
//!         x: 1,
//!         y: 1,
//!         width: 1,
//!         height: 1,
//!         color: [0; 4],
//!     };
//! }
//! ```
//!
//! Jumeau qui compile (lecture seule) :
//!
//! ```
//! use verbum_render::Frame;
//!
//! fn lit(frame: &Frame<2>) -> usize {
//!     frame.as_slice().len()
//! }
//!
//! assert_eq!(lit(&Frame::<2>::empty()), 0);
//! ```
//!
//! (b) Appel de `push` sur une `Frame` **possédée et mutable**. Ce code échoue
//! tant que l'API de `Frame` ne contient aucun mutateur. Si un
//! `push(&mut self, …)` est ajouté à `Frame`, il compile et ce doctest
//! échoue : c'est cette sensibilité qui est vérifiée.
//!
//! ```compile_fail
//! use verbum_render::{Contribution, Frame};
//!
//! let mut frame = Frame::<2>::empty();
//! frame.push(Contribution {
//!     x: 1,
//!     y: 1,
//!     width: 1,
//!     height: 1,
//!     color: [0; 4],
//! });
//! ```
//!
//! Jumeau qui compile (méthode existante, même forme possédée et mutable) :
//!
//! ```
//! use verbum_render::Frame;
//!
//! let mut frame = Frame::<2>::empty();
//! assert_eq!(frame.len(), 0);
//! frame = Frame::<2>::empty();
//! assert!(frame.is_empty());
//! ```
//!
//! (c) Fermeture `FnMut(&Frame<N>)` jouant le rôle d'un Backend qui tente de
//! modifier la frame reçue :
//!
//! ```compile_fail
//! use verbum_render::{Contribution, Frame};
//!
//! fn presente<F: FnMut(&Frame<2>)>(mut backend: F, frame: &Frame<2>) {
//!     backend(frame);
//! }
//!
//! let frame = Frame::<2>::empty();
//! presente(
//!     |recue: &Frame<2>| {
//!         recue.as_slice()[0] = Contribution {
//!             x: 1,
//!             y: 1,
//!             width: 1,
//!             height: 1,
//!             color: [0; 4],
//!         };
//!     },
//!     &frame,
//! );
//! ```
//!
//! Jumeau qui compile (Backend qui lit seulement) :
//!
//! ```
//! use verbum_render::Frame;
//!
//! fn presente<F: FnMut(&Frame<2>)>(mut backend: F, frame: &Frame<2>) {
//!     backend(frame);
//! }
//!
//! let frame = Frame::<2>::empty();
//! let mut vues = 0;
//! presente(|recue: &Frame<2>| vues += recue.len() + 1, &frame);
//! assert_eq!(vues, 1);
//! ```

use core::fmt;

use crate::{Contributions, contribution::Contribution};

/// Représentation de frame à capacité fixe `CAP`, sans allocation.
///
/// Ni `Copy` ni `Ord` : voir la documentation du module.
#[derive(Clone)]
pub struct Frame<const CAP: usize> {
    items: [Contribution; CAP],
    len: usize,
}

impl<const CAP: usize> Frame<CAP> {
    /// Frame vide : état initial déterministe (sans notion d'« absent »).
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            items: [Contribution::FILL; CAP],
            len: 0,
        }
    }

    /// Compose la frame à partir de l'accumulateur : copie de toutes les
    /// contributions, sans tri, sans déduplication, sans écart ni arbitrage
    /// (ADR-R01, point 4).
    ///
    /// Le débordement est impossible par construction : l'accumulateur et la
    /// frame partagent la même constante `CAP`.
    #[must_use]
    pub fn compose(source: &Contributions<CAP>) -> Self {
        let source = source.as_slice();
        let mut items = [Contribution::FILL; CAP];
        items[..source.len()].copy_from_slice(source);
        Self {
            items,
            len: source.len(),
        }
    }

    /// Contributions de la frame, dans l'ordre d'émission (non contractuel).
    #[must_use]
    pub fn as_slice(&self) -> &[Contribution] {
        &self.items[..self.len]
    }

    /// Nombre de contributions.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Vrai si la frame ne contient aucune contribution.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Capacité fixe `CAP`.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        CAP
    }
}

/// Égalité sur le contenu observable (la tranche des éléments composés).
impl<const CAP: usize> PartialEq for Frame<CAP> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<const CAP: usize> Eq for Frame<CAP> {}

impl<const CAP: usize> fmt::Debug for Frame<CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}
