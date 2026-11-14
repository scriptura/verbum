// crates/engine/shell/src/policy.rs

//! Politique temporelle du Shell (ADR-S04) : le triplet (q, K, M).
//!
//! - **q** : quantum logique, espacement nominal des échéances, en nanosecondes ;
//! - **K** : nombre maximal d'itérations exécutées par progression ;
//! - **M** : seuil de résiduel conservé ; au-delà, les échéances restantes sont abandonnées.
//!
//! Contrat porté par les types (C-1) : q, K et M sont strictement positifs
//! (`NonZeroU64`, `NonZeroU32`), finis et explicites ; aucune valeur par défaut n'existe.
//! K et M sont indépendants (aucune relation K = M / q).
//!
//! Stabilité (C-2) : une [`TemporalPolicy`] est immuable. Les champs sont privés, il n'existe
//! aucun mutateur ; elle est fixée à la composition d'une instance.
//!
//! Une politique invalide n'est pas constructible : toute construction en échec rend une
//! [`PolicyError`] et ne produit aucune valeur. Ce module ne connaît ni horloge, ni
//! échéancier, ni `Instant`, et ne fixe aucune valeur numérique de q, K ou M.

use core::num::{NonZeroU32, NonZeroU64};
#[cfg(test)]
use core::time::Duration;

/// Erreur de construction d'une [`TemporalPolicy`].
///
/// Si plusieurs paramètres sont invalides, l'erreur rendue est celle du premier
/// contrôle dans l'ordre q, K, M.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyError {
    /// q vaut zéro.
    ZeroQuantum,
    /// q, exprimé en `Duration` (forme de test), ne tient pas en `u64` nanosecondes.
    QuantumOutOfRange,
    /// K vaut zéro.
    ZeroBudget,
    /// M vaut zéro.
    ZeroThreshold,
}

/// Politique temporelle (q, K, M), immuable et valide par construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemporalPolicy {
    quantum_ns: NonZeroU64,
    budget: NonZeroU32,
    threshold: NonZeroU32,
}

impl TemporalPolicy {
    /// Construit une politique depuis q en nanosecondes (forme issue du paquet).
    ///
    /// # Erreurs
    ///
    /// [`PolicyError::ZeroQuantum`] si `quantum_ns == 0`, [`PolicyError::ZeroBudget`]
    /// si `budget == 0`, [`PolicyError::ZeroThreshold`] si `threshold == 0`.
    pub fn from_nanos(quantum_ns: u64, budget: u32, threshold: u32) -> Result<Self, PolicyError> {
        let Some(quantum_ns) = NonZeroU64::new(quantum_ns) else {
            return Err(PolicyError::ZeroQuantum);
        };
        let Some(budget) = NonZeroU32::new(budget) else {
            return Err(PolicyError::ZeroBudget);
        };
        let Some(threshold) = NonZeroU32::new(threshold) else {
            return Err(PolicyError::ZeroThreshold);
        };
        Ok(Self {
            quantum_ns,
            budget,
            threshold,
        })
    }

    /// Construit une politique depuis q en `Duration` (forme de test, non publique).
    ///
    /// La conversion est vérifiée, sans saturation ni troncature :
    /// `u64::try_from(quantum.as_nanos())`.
    ///
    /// # Erreurs
    ///
    /// [`PolicyError::QuantumOutOfRange`] si q dépasse `u64::MAX` nanosecondes ;
    /// pour le reste, voir [`TemporalPolicy::from_nanos`].
    #[cfg(test)]
    fn from_duration(quantum: Duration, budget: u32, threshold: u32) -> Result<Self, PolicyError> {
        match u64::try_from(quantum.as_nanos()) {
            Ok(quantum_ns) => Self::from_nanos(quantum_ns, budget, threshold),
            Err(_) => Err(PolicyError::QuantumOutOfRange),
        }
    }

    /// Quantum q, en nanosecondes.
    #[must_use]
    pub fn quantum_ns(&self) -> NonZeroU64 {
        self.quantum_ns
    }

    /// Plafond K d'itérations par progression.
    #[must_use]
    pub fn budget(&self) -> NonZeroU32 {
        self.budget
    }

    /// Seuil M de résiduel conservé.
    #[must_use]
    pub fn threshold(&self) -> NonZeroU32 {
        self.threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const Q: u64 = 10;
    const K: u32 = 3;
    const M: u32 = 5;

    fn ok(q: u64, k: u32, m: u32) -> TemporalPolicy {
        match TemporalPolicy::from_nanos(q, k, m) {
            Ok(p) => p,
            Err(e) => panic!("politique valide rejetée : {e:?}"),
        }
    }

    // TP-A7 : valeurs de test et accesseurs fidèles (détecte K et M permutés).
    #[test]
    fn a7_valeurs_de_test_et_accesseurs() {
        let p = ok(Q, K, M);
        assert_eq!(p.quantum_ns().get(), 10);
        assert_eq!(p.budget().get(), 3);
        assert_eq!(p.threshold().get(), 5);
    }

    // TP-A7 : q, K, M nuls rejetés, erreur typée.
    #[test]
    fn a7_zeros_rejetes() {
        assert_eq!(
            TemporalPolicy::from_nanos(0, K, M),
            Err(PolicyError::ZeroQuantum)
        );
        assert_eq!(
            TemporalPolicy::from_nanos(Q, 0, M),
            Err(PolicyError::ZeroBudget)
        );
        assert_eq!(
            TemporalPolicy::from_nanos(Q, K, 0),
            Err(PolicyError::ZeroThreshold)
        );
        assert_eq!(
            TemporalPolicy::from_duration(Duration::ZERO, K, M),
            Err(PolicyError::ZeroQuantum)
        );
        assert_eq!(
            TemporalPolicy::from_duration(Duration::from_nanos(Q), 0, M),
            Err(PolicyError::ZeroBudget)
        );
        assert_eq!(
            TemporalPolicy::from_duration(Duration::from_nanos(Q), K, 0),
            Err(PolicyError::ZeroThreshold)
        );
    }

    // Ordre de contrôle documenté : q, puis K, puis M.
    #[test]
    fn a7_ordre_des_controles() {
        assert_eq!(
            TemporalPolicy::from_nanos(0, 0, 0),
            Err(PolicyError::ZeroQuantum)
        );
        assert_eq!(
            TemporalPolicy::from_nanos(Q, 0, 0),
            Err(PolicyError::ZeroBudget)
        );
        assert_eq!(
            TemporalPolicy::from_duration(Duration::MAX, 0, 0),
            Err(PolicyError::QuantumOutOfRange)
        );
    }

    // TP-A7 : bornes. Aucune borne décalée : 1 et u64::MAX / u32::MAX acceptés.
    #[test]
    fn a7_bornes_acceptees() {
        let p = ok(1, 1, 1);
        assert_eq!(
            (p.quantum_ns().get(), p.budget().get(), p.threshold().get()),
            (1, 1, 1)
        );
        let p = ok(u64::MAX, u32::MAX, u32::MAX);
        assert_eq!(p.quantum_ns().get(), u64::MAX);
        assert_eq!(p.budget().get(), u32::MAX);
        assert_eq!(p.threshold().get(), u32::MAX);
    }

    // TP-A7 : K et M indépendants, dans les deux sens.
    #[test]
    fn a7_k_et_m_independants() {
        let p = ok(1, 1, 1000);
        assert_eq!((p.budget().get(), p.threshold().get()), (1, 1000));
        let p = ok(1, 1000, 1);
        assert_eq!((p.budget().get(), p.threshold().get()), (1000, 1));
    }

    // TP-A7 : q hors plage (forme Duration) ; la plage exacte est u64::MAX ns.
    #[test]
    fn a7_q_hors_plage() {
        let max_ns = Duration::from_nanos(u64::MAX);
        let p = match TemporalPolicy::from_duration(max_ns, K, M) {
            Ok(p) => p,
            Err(e) => panic!("u64::MAX ns doit être accepté : {e:?}"),
        };
        assert_eq!(p.quantum_ns().get(), u64::MAX);

        let juste_au_dessus = max_ns + Duration::from_nanos(1);
        assert_eq!(
            TemporalPolicy::from_duration(juste_au_dessus, K, M),
            Err(PolicyError::QuantumOutOfRange)
        );
        assert_eq!(
            TemporalPolicy::from_duration(Duration::MAX, K, M),
            Err(PolicyError::QuantumOutOfRange)
        );
    }

    // Les deux formes d'entrée de q coïncident.
    #[test]
    fn a7_formes_de_q_equivalentes() {
        assert_eq!(
            TemporalPolicy::from_duration(Duration::from_nanos(Q), K, M),
            TemporalPolicy::from_nanos(Q, K, M)
        );
        assert_eq!(
            TemporalPolicy::from_duration(Duration::from_secs(1), K, M),
            TemporalPolicy::from_nanos(1_000_000_000, K, M)
        );
    }

    // TP-E11 (niveau politique) : toute construction invalide ne produit aucune valeur,
    // donc aucune politique à fournir à un futur `Progressor`. Reconfirmé en T5a par signature.
    #[test]
    fn e11_construction_invalide_ne_produit_aucune_politique() {
        let invalides = [
            TemporalPolicy::from_nanos(0, K, M),
            TemporalPolicy::from_nanos(Q, 0, M),
            TemporalPolicy::from_nanos(Q, K, 0),
            TemporalPolicy::from_duration(Duration::ZERO, K, M),
            TemporalPolicy::from_duration(Duration::MAX, K, M),
        ];
        for r in invalides {
            assert!(r.is_err());
        }
    }
}
