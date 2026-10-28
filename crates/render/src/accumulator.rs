// crates/render/src/accumulator.rs

//! Accumulateur de contributions : séquence à capacité fixe.
//!
//! C'est le contenu de la Ressource transitoire alimentée par les systèmes
//! producteurs (ADR-R02, points 1 et 2) et consommée par la seule composition
//! (ADR-R02, point 3).
//!
//! - Production append-only : `push` ajoute en fin ; aucune contribution déjà
//!   acceptée n'est modifiable ni adressable (aucune clé, aucun accès publié
//!   comme identification, ADR-R01, point 1).
//! - Capacité fixée par le paramètre constant `CAP` : un dépassement est refusé
//!   explicitement par `Err(Full)`, sans croissance ni perte silencieuse
//!   (ADR-001, point 5 ; ADR-000, point 4).
//! - Remise à l'état initial par `clear` : l'état vidé est indiscernable de
//!   l'état initial (`01-runtime-ecs.md`, §4).

use core::fmt;

use crate::contribution::Contribution;

/// Refus d'un `push` : la capacité de l'accumulateur est atteinte.
///
/// Ce type ne signale que ce refus. Il n'est pas une erreur de Command Buffer
/// et ne doit pas être converti en `BufferFull` (plan C7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Full;

/// Séquence de contributions à capacité fixe `CAP`, sans allocation.
#[derive(Clone)]
pub struct Contributions<const CAP: usize> {
    items: [Contribution; CAP],
    len: usize,
}

impl<const CAP: usize> Contributions<CAP> {
    /// Accumulateur vide : état initial déterministe.
    // `clippy::new_without_default` est autorisé ici, et ici seulement : le plan
    // (§ 6) et le document d'exécution (U2) ne prévoient aucune implémentation
    // publique de `Default` pour `Contributions`, et aucune API n'est ajoutée au
    // delà du plan (§ 2). L'état initial s'obtient par `new()` ; `Default`
    // ajouterait un second chemin d'initialisation sans besoin concret.
    #[allow(clippy::new_without_default)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            items: [Contribution::FILL; CAP],
            len: 0,
        }
    }

    /// Ajoute une contribution en fin de séquence.
    ///
    /// # Errors
    ///
    /// Rend `Err(Full)` si la capacité est atteinte ; le contenu, l'ordre, la
    /// longueur et la capacité restent alors inchangés.
    pub fn push(&mut self, contribution: Contribution) -> Result<(), Full> {
        if self.len == CAP {
            return Err(Full);
        }
        self.items[self.len] = contribution;
        self.len += 1;
        Ok(())
    }

    /// Contributions acceptées, dans l'ordre d'acceptation.
    #[must_use]
    pub fn as_slice(&self) -> &[Contribution] {
        &self.items[..self.len]
    }

    /// Nombre de contributions acceptées.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Vrai si aucune contribution n'est acceptée.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Capacité fixe `CAP`.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        CAP
    }

    /// Remet l'accumulateur à l'état initial : vide, réutilisable.
    pub fn clear(&mut self) {
        self.items = [Contribution::FILL; CAP];
        self.len = 0;
    }
}

/// Égalité sur le contenu observable (la tranche des éléments acceptés).
impl<const CAP: usize> PartialEq for Contributions<CAP> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<const CAP: usize> fmt::Debug for Contributions<CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}
