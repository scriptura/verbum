// crates/engine/shell/src/driver.rs

//! Pilote bloquant à politique temporelle (ADR-S04, spécification r3 § 9.2).
//!
//! [`run_blocking`] possède un [`Progressor`] et le conduit jusqu'à son état terminal :
//! il appelle `progress()`, puis attend jusqu'à l'échéance restituée par une fermeture
//! **injectée**, puis recommence. Il ne contient aucune logique d'échéancier, de plafond K
//! ni de seuil M : toute la politique est dans `Progressor::progress` (la sémantique n'existe
//! qu'une fois, S02 pt 3).
//!
//! [`sleep_until`] fournit l'attente par défaut : elle dort la durée restante avant
//! l'échéance, jamais négative. Elle lit l'horloge par [`MonotonicClock`] ; ce module ne
//! manipule aucun `Instant` : une échéance est un décalage depuis l'origine de l'horloge,
//! et la durée restante se calcule sur des `Duration`.
//!
//! **Précondition d'origine commune.** Le `Progressor` et l'attenteur doivent partager la
//! même origine d'horloge, c'est-à-dire la même [`MonotonicClock`] (elle est `Copy`). Aucun
//! type ne l'impose. Usage :
//!
//! ```text
//! let clock = MonotonicClock::start();
//! let progressor = Progressor::new(policy, move || clock.elapsed(), stop, inbox, infrastructure);
//! let total = run_blocking(progressor, sleep_until(clock))?;
//! ```
//!
//! Différence avec [`run_loop`](crate::run_loop) : celle-ci est la boucle historique à
//! cadence externe, hors politique S04 ; `run_blocking` applique la politique (q, K, M).

use core::time::Duration;

use crate::clock::MonotonicClock;
use crate::progressor::{Deadline, ProgressError, ProgressState, Progressor, Wake};

/// Conduit un [`Progressor`] jusqu'à son état terminal, en attendant chaque échéance par
/// `wait_until`.
///
/// Séquence : `progress()` ; si le compte rendu est `Running` avec une échéance, appel de
/// `wait_until(échéance)` **même si la progression n'a exécuté aucune itération** ; une
/// échéance déjà passée est transmise telle quelle, sans progression supplémentaire avant
/// le retour de l'attenteur. Aucun appel à `wait_until` après un état terminal.
///
/// Le pilote prend possession du `Progressor` : celui-ci, avec ses fermetures et son
/// `Inbox`, est libéré au retour, quelle que soit l'issue.
///
/// Rend `Ok(total)` à l'arrêt observé : `total` cumule les itérations de toutes les
/// progressions.
///
/// # Errors
///
/// `Err(erreur)` en cas d'échec : l'erreur prélevée par `take_error()` est rendue **telle
/// quelle** (ni transformation ni clonage). Le total n'est pas rendu (spéc. § 5.2).
///
/// # Panics
///
/// Exige un `Progressor` **fraîchement construit** : aucun appel préalable à `progress()` ni
/// à `take_error()`. Si un appelant a déjà conduit le `Progressor` à l'échec et prélevé son
/// erreur, `run_blocking` constate un échec sans cause et panique (violation de la
/// précondition ; aucun succès fictif, aucune erreur de substitution). Elle panique aussi
/// si un état non terminal ne porte aucune échéance, ce que le contrat C-10 exclut.
pub fn run_blocking<Ev, E, C, S, I, W>(
    mut progressor: Progressor<Ev, E, C, S, I>,
    mut wait_until: W,
) -> Result<u64, ProgressError<E>>
where
    Ev: Copy,
    C: FnMut() -> Duration,
    S: FnMut() -> bool,
    I: FnMut(&[Ev]) -> Result<(), E>,
    W: FnMut(Deadline),
{
    // Le cumul ne peut pas déborder : chaque itération avance l'échéancier d'au moins une
    // nanoseconde dans un `u64` (un dépassement est une erreur temporelle), donc le total
    // reste borné par `u64::MAX`.
    let mut total: u64 = 0;
    loop {
        let report = progressor.progress();
        total += u64::from(report.iterations);
        match (report.state, report.wake) {
            (ProgressState::Running, Wake::At(deadline)) => wait_until(deadline),
            (ProgressState::Running, Wake::NoDeadline) => {
                unreachable!(
                    "invariant violé : un état non terminal porte toujours une échéance (C-10)"
                )
            }
            (ProgressState::Stopped, _) => return Ok(total),
            (ProgressState::Failed, _) => match progressor.take_error() {
                Some(error) => return Err(error),
                None => unreachable!(
                    "précondition violée : Progressor déjà en échec dont l'erreur a été prélevée avant run_blocking"
                ),
            },
        }
    }
}

/// Attenteur par défaut : dort jusqu'à l'échéance restituée.
///
/// La durée restante est l'échéance moins le temps écoulé depuis l'origine de l'horloge,
/// ramenée à zéro si l'échéance est passée ; `thread::sleep` n'est appelé que si elle est
/// strictement positive.
///
/// **Précondition** : `clock` est la même [`MonotonicClock`] (même origine) que celle qui
/// alimente le `Progressor` ; sinon les échéances sont comparées à une autre origine.
pub fn sleep_until(clock: MonotonicClock) -> impl FnMut(Deadline) {
    move |deadline| {
        let remaining = time_until(deadline.as_duration(), clock.elapsed());
        if !remaining.is_zero() {
            std::thread::sleep(remaining);
        }
    }
}

/// Durée restante avant `target` à l'instant `now` (tous deux depuis la même origine),
/// nulle si `target` est déjà passé.
fn time_until(target: Duration, now: Duration) -> Duration {
    target.saturating_sub(now)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ns(valeur: u64) -> Duration {
        Duration::from_nanos(valeur)
    }

    #[test]
    fn echeance_future_duree_restante_exacte() {
        assert_eq!(time_until(ns(110), ns(100)), ns(10));
        assert_eq!(time_until(ns(101), ns(100)), ns(1));
        assert_eq!(
            time_until(Duration::from_secs(2), Duration::from_secs(1)),
            Duration::from_secs(1)
        );
    }

    #[test]
    fn echeance_exacte_attente_nulle() {
        assert_eq!(time_until(ns(100), ns(100)), Duration::ZERO);
        assert_eq!(time_until(Duration::ZERO, Duration::ZERO), Duration::ZERO);
    }

    #[test]
    fn echeance_passee_attente_nulle_sans_debordement() {
        assert_eq!(time_until(ns(99), ns(100)), Duration::ZERO);
        assert_eq!(time_until(ns(140), ns(155)), Duration::ZERO);
        assert_eq!(time_until(Duration::ZERO, Duration::MAX), Duration::ZERO);
    }

    #[test]
    fn extremes() {
        assert_eq!(time_until(Duration::MAX, Duration::ZERO), Duration::MAX);
        assert_eq!(time_until(Duration::MAX, Duration::MAX), Duration::ZERO);
        assert_eq!(
            time_until(ns(u64::MAX), ns(u64::MAX - 1)),
            ns(1),
            "écart d'une nanoseconde en bout de plage"
        );
    }
}
