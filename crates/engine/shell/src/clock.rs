// crates/engine/shell/src/clock.rs

//! Horloge monotone du Shell (ADR-S04, spécification r3 § 4.2 et C-14).
//!
//! Ce module est, avec le pilote bloquant par défaut (unité ultérieure), le **seul**
//! endroit qui lit l'horloge réelle. [`MonotonicClock`] porte une origine `Instant` et
//! rend des `Duration` écoulées depuis cette origine ; l'`Instant` ne franchit jamais le
//! `Progressor` : celui-ci ne manipule que des `Duration` puis des `u64` nanosecondes.
//!
//! **Précondition de monotonie (C-14).** Toute source de temps fournie au `Progressor`
//! est monotone (non décroissante), comme l'est cette horloge. Le comportement face à une
//! source qui décroît n'est pas spécifié et n'est pas un cas de fonctionnement. Les
//! sources substituées (tests, adaptateurs) respectent la même précondition.
//!
//! La conversion `Duration` → `u64` nanosecondes ([`duration_to_ns`]) est vérifiée : une
//! lecture au-delà de `u64::MAX` ns (environ 584 ans) rend
//! [`TimeError::ClockOutOfRange`], sans troncature ni saturation.

use std::time::{Duration, Instant};

use crate::schedule::TimeError;

/// Convertit une lecture d'horloge en nanosecondes `u64`.
///
/// # Erreurs
///
/// [`TimeError::ClockOutOfRange`] si la durée dépasse `u64::MAX` nanosecondes.
pub(crate) fn duration_to_ns(duration: Duration) -> Result<u64, TimeError> {
    match u64::try_from(duration.as_nanos()) {
        Ok(ns) => Ok(ns),
        Err(_) => Err(TimeError::ClockOutOfRange),
    }
}

/// Horloge monotone : origine `Instant` fixée à sa création.
///
/// `Copy` : un adaptateur peut en conserver une copie pour convertir une échéance
/// (durée depuis l'origine) en `Instant`, via [`origin`](Self::origin).
#[derive(Debug, Clone, Copy)]
pub struct MonotonicClock {
    origin: Instant,
}

impl MonotonicClock {
    /// Crée une horloge dont l'origine est l'instant courant.
    #[must_use]
    pub fn start() -> Self {
        Self {
            origin: Instant::now(),
        }
    }

    /// Origine de l'horloge.
    #[must_use]
    pub fn origin(&self) -> Instant {
        self.origin
    }

    /// Durée écoulée depuis l'origine. Non décroissante entre deux appels (C-14).
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.origin.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TP-A6 : durée au-delà de u64::MAX ns.
    #[test]
    fn a6_hors_plage_signale() {
        let max_ns = Duration::from_nanos(u64::MAX);
        assert_eq!(
            duration_to_ns(max_ns + Duration::from_nanos(1)),
            Err(TimeError::ClockOutOfRange)
        );
        assert_eq!(
            duration_to_ns(Duration::MAX),
            Err(TimeError::ClockOutOfRange)
        );
        // Une seconde de plus : jamais de repli sur une valeur plausible.
        assert_eq!(
            duration_to_ns(max_ns + Duration::from_secs(1)),
            Err(TimeError::ClockOutOfRange)
        );
    }

    // TP-A6 : bornes acceptées, valeur exacte (ni troncature ni saturation).
    #[test]
    fn a6_bornes_acceptees() {
        assert_eq!(duration_to_ns(Duration::ZERO), Ok(0));
        assert_eq!(duration_to_ns(Duration::from_nanos(1)), Ok(1));
        assert_eq!(duration_to_ns(Duration::from_secs(1)), Ok(1_000_000_000));
        assert_eq!(duration_to_ns(Duration::from_nanos(u64::MAX)), Ok(u64::MAX));
        assert_eq!(
            duration_to_ns(Duration::from_nanos(u64::MAX - 1)),
            Ok(u64::MAX - 1)
        );
        // Même borne exprimée en secondes + nanosecondes : 18 446 744 073 s et 709 551 615 ns.
        assert_eq!(
            duration_to_ns(Duration::new(18_446_744_073, 709_551_615)),
            Ok(u64::MAX)
        );
        assert_eq!(
            duration_to_ns(Duration::new(18_446_744_073, 709_551_616)),
            Err(TimeError::ClockOutOfRange)
        );
    }

    // Les causes temporelles restent distinctes (spec § 4.4, E9).
    #[test]
    fn cause_distincte_de_schedule_overflow() {
        assert_ne!(TimeError::ClockOutOfRange, TimeError::ScheduleOverflow);
        assert_eq!(
            duration_to_ns(Duration::MAX),
            Err(TimeError::ClockOutOfRange)
        );
    }

    #[test]
    fn horloge_monotone_et_copiable() {
        let horloge = MonotonicClock::start();
        let copie = horloge;
        assert_eq!(copie.origin(), horloge.origin());
        let mut precedent = horloge.elapsed();
        for _ in 0..1000 {
            let lecture = copie.elapsed();
            assert!(lecture >= precedent);
            precedent = lecture;
        }
    }

    // La durée lue se convertit sans erreur (une horloge réelle reste loin de 584 ans).
    #[test]
    fn lecture_reelle_convertible() {
        let horloge = MonotonicClock::start();
        assert!(duration_to_ns(horloge.elapsed()).is_ok());
    }
}
