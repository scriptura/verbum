// crates/engine/shell/src/progressor.rs

//! `Progressor` : progression persistante sous politique temporelle (ADR-S04, spécification r3 § 5).
//!
//! Un `Progressor` possède la politique ([`TemporalPolicy`]), l'horloge, la demande d'arrêt,
//! l'[`Inbox`] et l'infrastructure. Chaque appel à [`Progressor::progress`] :
//! 1. lit l'horloge (t_entrée) et détermine les échéances dues ;
//! 2. n'exécute rien si rien n'est dû (aucune coupure, aucune seconde lecture) ;
//! 3. exécute au plus K itérations, chacune par le cœur d'itération unique (`iterate`) ;
//! 4. lit l'horloge (t_fin) et décide du résiduel : conservé si R ≤ M, abandonné en entier
//!    avec réancrage à `t_fin + q` si R > M (R1) ;
//! 5. rend un compte rendu [`Progress`] : itérations exécutées, réveil, état.
//!
//! L'échéancier est initialisé **paresseusement** à t_entrée de la première progression
//! (I-3) : la première échéance est immédiatement due, sans rafale. Le réancrage R1 est un
//! chemin distinct.
//!
//! Terminal : un arrêt observé, une erreur d'infrastructure ou une erreur temporelle rend
//! l'objet terminal. Toute progression suivante rend `{0, NoDeadline, phase}` sans lire
//! l'horloge ni couper l'`Inbox`. La cause d'un échec se récupère une seule fois par
//! [`Progressor::take_error`].
//!
//! Ce module ne manipule jamais d'`Instant` : le temps y est un `u64` nanosecondes depuis
//! l'origine de l'horloge. Dispatch statique, aucune allocation dans `progress` ni
//! `take_error`.
//!
//! Choix d'exécution PO-1 : après une erreur survenue après l'invocation, l'échéancier n'est
//! pas avancé ; sans effet observable, l'état étant terminal.

use core::num::NonZeroU64;
use core::time::Duration;

use crate::Inbox;
use crate::clock::duration_to_ns;
use crate::policy::TemporalPolicy;
use crate::schedule::{TimeError, advance, due_capped, exceeds, reanchor};
use crate::shell::{Iteration, iterate};

/// Échéance restituée : durée depuis l'origine de l'horloge. Aucun `Instant` dans le contrat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Deadline(Duration);

impl Deadline {
    fn from_ns(ns: u64) -> Self {
        Self(Duration::from_nanos(ns))
    }

    /// Durée écoulée depuis l'origine de l'horloge à laquelle l'échéance est fixée.
    ///
    /// Peut être antérieure à l'instant de restitution (échéance déjà due).
    #[must_use]
    pub fn as_duration(&self) -> Duration {
        self.0
    }
}

/// Besoin de réveil restitué par une progression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wake {
    /// Prochaine échéance non exécutée (hors terminal : toujours restituée).
    At(Deadline),
    /// État terminal : plus aucune échéance.
    NoDeadline,
}

/// État du `Progressor` après une progression.
///
/// Nom provisoire (spécification r3 § 12.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Non terminal : d'autres progressions sont possibles.
    Running,
    /// Terminal : arrêt observé.
    Stopped,
    /// Terminal : erreur ; la cause se récupère par [`Progressor::take_error`].
    Failed,
}

/// Compte rendu d'une progression. N'est jamais un `Result` : les itérations exécutées
/// restent rapportées même lorsqu'une erreur survient.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// Itérations achevées par **cette** progression (≤ K).
    pub iterations: u32,
    /// Besoin de réveil.
    pub wake: Wake,
    /// État après la progression.
    pub phase: Phase,
}

/// Cause d'un échec : transportée, jamais transformée.
///
/// Aucun `Clone`, aucune exigence de trait sur `E`.
#[derive(Debug)]
pub enum ProgressError<E> {
    /// Erreur de l'infrastructure (invocation ou publication), **inchangée**.
    Infrastructure(E),
    /// Erreur du calcul temporel.
    Time(TimeError),
}

/// Progression persistante sous politique temporelle.
pub struct Progressor<Ev: Copy, E, C, S, I> {
    policy: TemporalPolicy,
    clock: C,
    stop: S,
    inbox: Inbox<Ev>,
    infrastructure: I,
    /// Prochaine échéance non exécutée ; `None` tant que la première progression n'a pas eu lieu.
    next: Option<u64>,
    phase: Phase,
    error: Option<ProgressError<E>>,
}

impl<Ev, E, C, S, I> Progressor<Ev, E, C, S, I>
where
    Ev: Copy,
    C: FnMut() -> Duration,
    S: FnMut() -> bool,
    I: FnMut(&[Ev]) -> Result<(), E>,
{
    /// Construit un `Progressor` non initialisé : aucune lecture d'horloge, aucune échéance.
    ///
    /// La politique est valide par construction (C-1) et immuable pour l'instance (C-2).
    pub fn new(
        policy: TemporalPolicy,
        clock: C,
        stop: S,
        inbox: Inbox<Ev>,
        infrastructure: I,
    ) -> Self {
        Self {
            policy,
            clock,
            stop,
            inbox,
            infrastructure,
            next: None,
            phase: Phase::Running,
            error: None,
        }
    }

    /// Exécute une progression (voir la documentation du module).
    pub fn progress(&mut self) -> Progress {
        if self.phase != Phase::Running {
            return Progress {
                iterations: 0,
                wake: Wake::NoDeadline,
                phase: self.phase,
            };
        }
        let quantum: NonZeroU64 = self.policy.quantum_ns();

        // (1) t_entrée, puis initialisation paresseuse I-3 : une seule fois, sans lecture en plus.
        let t_entry = match self.read() {
            Ok(t) => t,
            Err(error) => return self.fail(ProgressError::Time(error), 0),
        };
        let mut next = *self.next.get_or_insert(t_entry);

        // (2) rien dû : aucune coupure, aucune seconde lecture.
        let due = due_capped(next, quantum, t_entry, self.policy.budget());
        if due == 0 {
            return Progress {
                iterations: 0,
                wake: Wake::At(Deadline::from_ns(next)),
                phase: Phase::Running,
            };
        }

        // (3) au plus K itérations ; l'ensemble exécuté est fixé à l'entrée.
        let mut done: u32 = 0;
        for _ in 0..due {
            match iterate(&mut self.stop, &mut self.inbox, &mut self.infrastructure) {
                Iteration::Stopped => {
                    self.phase = Phase::Stopped;
                    return Progress {
                        iterations: done,
                        wake: Wake::NoDeadline,
                        phase: Phase::Stopped,
                    };
                }
                Iteration::Failed(error) => {
                    return self.fail(ProgressError::Infrastructure(error), done);
                }
                Iteration::Completed => {
                    done += 1;
                    next = match advance(next, quantum) {
                        Ok(advanced) => advanced,
                        Err(error) => return self.fail(ProgressError::Time(error), done),
                    };
                }
            }
        }

        // (4) t_fin, puis (5) résiduel : conservé si R <= M, abandonné avec réancrage si R > M.
        let t_end = match self.read() {
            Ok(t) => t,
            Err(error) => return self.fail(ProgressError::Time(error), done),
        };
        if exceeds(next, quantum, t_end, self.policy.threshold()) {
            next = match reanchor(t_end, quantum) {
                Ok(anchored) => anchored,
                Err(error) => return self.fail(ProgressError::Time(error), done),
            };
        }
        self.next = Some(next);
        Progress {
            iterations: done,
            wake: Wake::At(Deadline::from_ns(next)),
            phase: Phase::Running,
        }
    }

    /// Récupère la cause de l'échec, **une seule fois** (déplacement, aucun `Clone`).
    ///
    /// Un second appel rend `None` ; la phase reste `Failed`.
    pub fn take_error(&mut self) -> Option<ProgressError<E>> {
        self.error.take()
    }

    fn read(&mut self) -> Result<u64, TimeError> {
        duration_to_ns((self.clock)())
    }

    fn fail(&mut self, error: ProgressError<E>, done: u32) -> Progress {
        self.error = Some(error);
        self.phase = Phase::Failed;
        Progress {
            iterations: done,
            wake: Wake::NoDeadline,
            phase: Phase::Failed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // E1, volet `Inbox` : un arrêt observé à la première itération ne coupe pas la boîte.
    // Le `Progressor` n'expose aucun accesseur (spécification r3 § 5.3) : la boîte est lue
    // ici directement, depuis le module.
    #[test]
    fn e1_arret_avant_la_premiere_iteration_laisse_la_boite_intacte() {
        let inbox = Inbox::<u32>::new(8);
        let depot = inbox.handle();
        for e in [5, 3, 9] {
            assert_eq!(depot.push(e), Ok(()));
        }
        let policy = TemporalPolicy::from_nanos(10, 3, 5).expect("politique valide");
        let mut invocations = 0u32;
        let mut p = Progressor::new(
            policy,
            || Duration::from_nanos(100),
            || true,
            inbox,
            |_lot: &[u32]| {
                invocations += 1;
                Ok::<(), ()>(())
            },
        );
        let r = p.progress();
        assert_eq!(r.iterations, 0);
        assert_eq!(r.phase, Phase::Stopped);
        assert_eq!(r.wake, Wake::NoDeadline);
        assert_eq!(p.inbox.cut(), [5, 3, 9]);
        drop(p);
        assert_eq!(invocations, 0);
    }
}
