// crates/extensions/script/src/emissions.rs

//! Émissions : séquence à capacité fixe produite par les instructions `Emit`.
//!
//! - Production append-only : `push` ajoute en fin ; aucune émission acceptée
//!   n'est modifiable ni adressable.
//! - Capacité fixée par le paramètre constant `CAP` : un dépassement est refusé
//!   explicitement par `Err(Full)`, sans croissance ni perte silencieuse
//!   (ADR-001, point 5 ; ADR-000, point 4).
//! - `clear` remet l'accumulateur dans un état indiscernable de l'état initial.
//!
//! Aucun ordre n'est défini entre instances : l'ordre observé est celui des
//! appels à `push`.

use core::fmt;

use verbum_assets::SCRIPT_MAX_STEPS_PER_TICK;

/// Refus d'un `push` : la capacité de l'accumulateur est atteinte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Full;

/// Émission : valeur `value` sur le canal `channel`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Emission {
    /// Canal de l'émission.
    pub channel: u16,
    /// Valeur émise.
    pub value: i32,
}

impl Emission {
    /// Valeur de remplissage des cases non acceptées.
    const FILL: Self = Self {
        channel: 0,
        value: 0,
    };
}

/// Séquence d'émissions à capacité fixe `CAP`, sans allocation.
#[derive(Clone)]
pub struct Emissions<const CAP: usize> {
    items: [Emission; CAP],
    len: usize,
}

impl<const CAP: usize> Emissions<CAP> {
    /// Accumulateur vide : état initial déterministe.
    // `clippy::new_without_default` est autorisé ici, et ici seulement : le plan
    // et le document d'exécution (U3) ne prévoient aucune implémentation
    // publique de `Default` pour `Emissions`, et aucune API n'est ajoutée au
    // delà de celle qu'ils fixent. L'état initial s'obtient par `new()` ;
    // `Default` ajouterait un second chemin d'initialisation sans besoin
    // concret (même traitement que `Contributions` dans `verbum-render`).
    #[allow(clippy::new_without_default)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            items: [Emission::FILL; CAP],
            len: 0,
        }
    }

    /// Ajoute une émission en fin de séquence.
    ///
    /// # Errors
    ///
    /// Rend `Err(Full)` si la capacité est atteinte ; le contenu, l'ordre, la
    /// longueur et la capacité restent alors inchangés.
    pub fn push(&mut self, emission: Emission) -> Result<(), Full> {
        if self.len == CAP {
            return Err(Full);
        }
        self.items[self.len] = emission;
        self.len += 1;
        Ok(())
    }

    /// Émissions acceptées, dans l'ordre d'acceptation.
    #[must_use]
    pub fn as_slice(&self) -> &[Emission] {
        &self.items[..self.len]
    }

    /// Nombre d'émissions acceptées.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Vrai si aucune émission n'est acceptée.
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
        self.items = [Emission::FILL; CAP];
        self.len = 0;
    }
}

/// Égalité sur le contenu observable (la tranche des éléments acceptés).
impl<const CAP: usize> PartialEq for Emissions<CAP> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<const CAP: usize> fmt::Debug for Emissions<CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

/// Vrai si `cap` couvre `instances` instances exécutant chacune au plus
/// `SCRIPT_MAX_STEPS_PER_TICK` instructions par tick, soit au plus une
/// émission par instruction.
///
/// Le produit est calculé par `checked_mul` : un débordement rend faux, jamais
/// un dépassement silencieux.
#[must_use]
pub const fn emissions_bound_holds(instances: usize, cap: usize) -> bool {
    match instances.checked_mul(SCRIPT_MAX_STEPS_PER_TICK) {
        Some(product) => product <= cap,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn emission(channel: u16, value: i32) -> Emission {
        Emission { channel, value }
    }

    fn same_capacity<const N: usize>(_: &Emissions<N>, _: &Emissions<N>) {}

    // EM-1 : accumulateur à capacité fixe.
    #[test]
    fn em1_accumulator() {
        let mut sink = Emissions::<3>::new();
        assert!(sink.is_empty());
        assert_eq!(sink.len(), 0);
        assert_eq!(sink.capacity(), 3);

        assert_eq!(sink.push(emission(1, 10)), Ok(()));
        assert_eq!(sink.push(emission(2, 20)), Ok(()));
        assert_eq!(sink.push(emission(3, 30)), Ok(()));
        assert_eq!(sink.len(), 3);
        assert!(!sink.is_empty());

        let before = sink.clone();
        assert_eq!(sink.push(emission(4, 40)), Err(Full));
        assert_eq!(sink.len(), 3);
        assert_eq!(
            sink.as_slice(),
            &[emission(1, 10), emission(2, 20), emission(3, 30)]
        );
        assert_eq!(sink, before);

        // Égalité sur la tranche observable.
        let mut shorter = Emissions::<3>::new();
        assert_eq!(shorter.push(emission(1, 10)), Ok(()));
        assert_ne!(sink, shorter);
        let mut other = Emissions::<3>::new();
        assert_eq!(other.push(emission(1, 10)), Ok(()));
        assert_eq!(other.push(emission(2, 20)), Ok(()));
        assert_eq!(other.push(emission(3, 31)), Ok(()));
        assert_ne!(sink, other);

        sink.clear();
        assert!(sink.is_empty());
        assert_eq!(sink, Emissions::<3>::new());
        assert_eq!(sink.push(emission(9, 9)), Ok(()));
        assert_eq!(sink.as_slice(), &[emission(9, 9)]);

        // CAP = 0 refuse tout.
        let mut none = Emissions::<0>::new();
        assert!(none.is_empty());
        assert_eq!(none.capacity(), 0);
        assert_eq!(none.push(emission(1, 1)), Err(Full));
        assert!(none.as_slice().is_empty());
        assert_eq!(none.len(), 0);
    }

    // EM-2 : borne des émissions, `checked_mul`.
    #[test]
    fn em2_bound() {
        assert!(emissions_bound_holds(4, 64));
        assert!(!emissions_bound_holds(4, 63));
        assert!(emissions_bound_holds(0, 0));
        assert!(!emissions_bound_holds(usize::MAX, usize::MAX));
    }

    // EM-3 : même capacité (fonction locale de test), `Clone` d'un plein.
    #[test]
    fn em3_same_capacity_and_clone() {
        let mut accumulator = Emissions::<2>::new();
        let transcript = Emissions::<2>::new();
        same_capacity(&accumulator, &transcript);

        assert_eq!(accumulator.push(emission(1, 1)), Ok(()));
        assert_eq!(accumulator.push(emission(2, 2)), Ok(()));
        let copy = accumulator.clone();
        assert_eq!(copy, accumulator);
        assert_eq!(copy.len(), 2);
        assert_eq!(copy.as_slice(), &[emission(1, 1), emission(2, 2)]);
        assert_ne!(copy, transcript);
    }
}
