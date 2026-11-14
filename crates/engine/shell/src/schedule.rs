// crates/engine/shell/src/schedule.rs

//! Arithmétique d'échéancier (ADR-S04, spécification r3 § 4.3).
//!
//! Fonctions **pures** sur des entiers : aucun état, aucune horloge, aucun `Instant`.
//! Les instants et `next` sont des `u64` nanosecondes depuis l'origine de l'horloge.
//!
//! Propriétés tenues par toutes les fonctions :
//! - temps constant : aucune boucle, en particulier aucune boucle bornée par le retard ;
//! - aucun flottant, aucune allocation, aucune panique sur entrée valide ;
//! - addition `checked` : un dépassement de `u64` rend [`TimeError::ScheduleOverflow`],
//!   jamais une saturation ni un repli silencieux ;
//! - aucune soustraction non gardée : `t - next` n'est calculée qu'après le test `t >= next`.
//!
//! Précondition (C-14) : les instants fournis proviennent d'une source monotone. Le
//! comportement face à une source décroissante n'est pas spécifié ; les fonctions ne
//! paniquent pas pour autant.
//!
//! Les éléments sont `pub(crate)` et consommés par le `Progressor` (unité ultérieure) ;
//! d'ici là, ils ne sont utilisés que par les tests.

#![cfg_attr(not(test), expect(dead_code))]

use core::num::{NonZeroU32, NonZeroU64};

/// Erreur de fonctionnement du calcul temporel (état terminal côté `Progressor`).
///
/// Variante unique à ce stade ; elle sera étendue par l'unité qui introduit l'horloge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeError {
    /// `next + q` ou `t_fin + q` dépasse `u64::MAX`.
    ScheduleOverflow,
}

/// Nombre d'itérations à exécuter : `min(K, |D|)`, où D est l'ensemble des échéances
/// dues à l'instant `t` (`next`, `next + q`, … ≤ `t`).
///
/// Formulation sans `s + 1` (qui déborde pour q = 1 et écart maximal) :
/// `0` si `t < next` ; `K` si `s ≥ K` ; sinon `s + 1`, avec `s = (t - next) / q`.
#[must_use]
pub(crate) fn due_capped(next: u64, q: NonZeroU64, t: u64, k: NonZeroU32) -> u32 {
    if t < next {
        return 0;
    }
    let s = (t - next) / q.get();
    match u32::try_from(s) {
        // `s < k <= u32::MAX` : `s + 1` ne déborde pas.
        Ok(s) if s < k.get() => s + 1,
        // `s >= k`, ou `s` ne tient pas en `u32` (donc `s > k`).
        _ => k.get(),
    }
}

/// Test « R > M » : vrai si le résiduel R d'échéances dues à `t_fin` dépasse M.
///
/// `next` est l'échéance suivante non exécutée (après avancement). R ≥ 1 ⇔
/// `t_fin >= next` ; R > M ⇔ `t_fin >= next` **et** `(t_fin - next) / q >= M`.
#[must_use]
pub(crate) fn exceeds(next: u64, q: NonZeroU64, t_fin: u64, m: NonZeroU32) -> bool {
    t_fin >= next && (t_fin - next) / q.get() >= u64::from(m.get())
}

/// Avancement après une itération comptée : `next + q`.
///
/// # Erreurs
///
/// [`TimeError::ScheduleOverflow`] si la somme dépasse `u64::MAX`.
pub(crate) fn advance(next: u64, q: NonZeroU64) -> Result<u64, TimeError> {
    match next.checked_add(q.get()) {
        Some(n) => Ok(n),
        None => Err(TimeError::ScheduleOverflow),
    }
}

/// Réancrage après abandon (R1) : `t_fin + q`.
///
/// # Erreurs
///
/// [`TimeError::ScheduleOverflow`] si la somme dépasse `u64::MAX`.
pub(crate) fn reanchor(t_fin: u64, q: NonZeroU64) -> Result<u64, TimeError> {
    match t_fin.checked_add(q.get()) {
        Some(n) => Ok(n),
        None => Err(TimeError::ScheduleOverflow),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(v: u64) -> NonZeroU64 {
        NonZeroU64::new(v).expect("q non nul")
    }

    fn n32(v: u32) -> NonZeroU32 {
        NonZeroU32::new(v).expect("valeur non nulle")
    }

    // Définition naïve : énumère les échéances next, next + q, … ≤ t.
    fn naive_due(next: u64, q: u64, t: u64) -> u64 {
        let mut count = 0;
        let mut d = next;
        while d <= t {
            count += 1;
            d += q;
        }
        count
    }

    // TP-A1
    #[test]
    fn a1_iterations_executees() {
        let attendu = [(99, 0), (100, 1), (109, 1), (110, 2), (119, 2), (120, 3)];
        for (t, e) in attendu {
            assert_eq!(due_capped(100, q(10), t, n32(u32::MAX)), e, "t = {t}");
        }
    }

    // TP-A2
    #[test]
    fn a2_plafond_k() {
        for (t, e) in [(119, 2), (120, 3), (145, 3)] {
            assert_eq!(due_capped(100, q(10), t, n32(3)), e, "t = {t}");
        }
    }

    // TP-A3
    #[test]
    fn a3_extremes_sans_debordement() {
        assert_eq!(due_capped(0, q(1), u64::MAX, n32(7)), 7);
        assert_eq!(due_capped(0, q(1), u64::MAX, n32(u32::MAX)), u32::MAX);
        assert_eq!(due_capped(0, q(u64::MAX), u64::MAX, n32(7)), 2);
        assert_eq!(due_capped(u64::MAX, q(1), u64::MAX, n32(7)), 1);
        assert_eq!(due_capped(u64::MAX, q(1), 0, n32(7)), 0);
        // Autour du plafond (q = 1, next = 0, K = 5) : s = t.
        for (t, e) in [(3, 4), (4, 5), (5, 5), (6, 5)] {
            assert_eq!(due_capped(0, q(1), t, n32(5)), e, "t = {t}");
        }
        // s ne tient pas en u32.
        assert_eq!(
            due_capped(0, q(1), u64::from(u32::MAX) + 10, n32(u32::MAX)),
            u32::MAX
        );
    }

    // TP-A4
    #[test]
    fn a4_seuil_m() {
        assert!(!exceeds(130, q(10), 129, n32(5)));
        assert!(!exceeds(130, q(10), 179, n32(5)));
        assert!(exceeds(130, q(10), 180, n32(5)));
        // R = 5 conservé, R = 6 abandonné (comptage direct).
        assert_eq!(naive_due(130, 10, 179), 5);
        assert_eq!(naive_due(130, 10, 180), 6);
    }

    #[test]
    fn a4_extremes_sans_panique() {
        assert!(exceeds(0, q(1), u64::MAX, n32(u32::MAX)));
        assert!(!exceeds(u64::MAX, q(1), 0, n32(1)));
        assert!(!exceeds(u64::MAX, q(1), u64::MAX, n32(1)));
        assert!(!exceeds(0, q(u64::MAX), u64::MAX, n32(2)));
    }

    // TP-A5
    #[test]
    fn a5_debordement_signale() {
        assert_eq!(
            advance(u64::MAX - 9, q(10)),
            Err(TimeError::ScheduleOverflow)
        );
        assert_eq!(advance(u64::MAX, q(1)), Err(TimeError::ScheduleOverflow));
        assert_eq!(
            reanchor(u64::MAX - 9, q(10)),
            Err(TimeError::ScheduleOverflow)
        );
        assert_eq!(reanchor(u64::MAX, q(1)), Err(TimeError::ScheduleOverflow));
    }

    #[test]
    fn a5_pas_de_saturation_silencieuse() {
        // Frontière exacte : u64::MAX atteint sans dépassement est accepté.
        assert_eq!(advance(u64::MAX - 10, q(10)), Ok(u64::MAX));
        assert_eq!(reanchor(u64::MAX - 10, q(10)), Ok(u64::MAX));
        assert_eq!(advance(100, q(10)), Ok(110));
        assert_eq!(reanchor(135, q(10)), Ok(145));
        // Dépassement d'un seul cran : erreur, jamais `Ok(u64::MAX)` ni repli.
        assert_ne!(advance(u64::MAX - 9, q(10)), Ok(u64::MAX));
        assert_ne!(reanchor(u64::MAX - 9, q(10)), Ok(u64::MAX));
    }

    // TP-A8 : énumération déterministe contre la définition naïve.
    #[test]
    fn a8_enumeration_contre_definition_naive() {
        let mut cas = 0u32;
        for qv in 1..=5u64 {
            for next in 0..20u64 {
                for t in 0..60u64 {
                    for k in 1..=5u32 {
                        let attendu = naive_due(next, qv, t).min(u64::from(k));
                        assert_eq!(
                            u64::from(due_capped(next, q(qv), t, n32(k))),
                            attendu,
                            "e : q={qv} next={next} t={t} K={k}"
                        );
                        cas += 1;
                    }
                    for m in 1..=7u32 {
                        let attendu = naive_due(next, qv, t) > u64::from(m);
                        assert_eq!(
                            exceeds(next, q(qv), t, n32(m)),
                            attendu,
                            "R > M : q={qv} next={next} t={t} M={m}"
                        );
                        cas += 1;
                    }
                }
            }
        }
        assert_eq!(cas, 5 * 20 * 60 * (5 + 7));
    }

    // Absence de dérive : l'avancement n'ajoute que q, sans reprise de t.
    #[test]
    fn avancement_sans_derive() {
        let mut next = 110u64;
        for _ in 0..100_000 {
            next = advance(next, q(10)).expect("pas de débordement");
        }
        assert_eq!(next, 110 + 10 * 100_000);
    }
}
