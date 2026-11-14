// crates/engine/shell/tests/temporal_drivers.rs

//! ADR-S04 (unité T6) : pilote bloquant à politique `run_blocking`, valeurs de test
//! q = 10 ns, K = 3, M = 5.
//!
//! - D1, moitié « pilote bloquant » : script d'observations dont les lots, les invocations,
//!   les échéances transmises à l'attenteur et l'issue sont comparés à des littéraux ;
//! - D2 : l'attenteur reçoit `wake`, y compris quand la progression n'a rien exécuté ; une
//!   échéance déjà passée est transmise telle quelle, sans progression supplémentaire ;
//! - issues : arrêt (total cumulé), erreur d'infrastructure inchangée, erreurs temporelles,
//!   violation de la précondition d'entrée ; possession du `Progressor`.
//!
//! L'horloge et l'attenteur sont scriptés (`temporal_support`) : aucun `sleep`, aucun
//! `Instant`. Une lecture d'horloge ou une progression en trop est un échec de test, car
//! l'horloge panique hors de sa suite.
//!
//! **Non démontré ici** : l'enveloppe réelle `sleep_until` (sommeil) ; son calcul est testé
//! dans `src/driver.rs`. D1 complet (pilote événementiel), D3, D6 : unité T7.

#[path = "temporal_support/mod.rs"]
#[allow(dead_code, reason = "harnais partagé avec les unités de pilotes")]
mod temporal_support;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use temporal_support::{Reads, scripted, scripted_ns};
use verbum_shell::{
    Deadline, Inbox, ProgressError, ProgressState, Progressor, TemporalPolicy, TimeError,
    run_blocking,
};

/// Erreur d'infrastructure de test : ni `Clone` ni `Copy`, pour prouver le transport inchangé.
#[derive(Debug)]
struct Boom {
    code: u32,
    tag: String,
}

/// Événement observé, dans l'ordre : lot d'une invocation, ou attente d'une échéance.
#[derive(Debug, PartialEq, Eq)]
enum Entry {
    Lot(Vec<u32>),
    Wait(u64),
}

fn lot(valeurs: &[u32]) -> Entry {
    Entry::Lot(valeurs.to_vec())
}

fn policy() -> TemporalPolicy {
    TemporalPolicy::from_nanos(10, 3, 5).expect("politique de test valide")
}

fn deadline_ns(deadline: Deadline) -> u64 {
    u64::try_from(deadline.as_duration().as_nanos()).expect("tient en u64")
}

struct Outcome {
    result: Result<u64, ProgressError<Boom>>,
    log: Vec<Entry>,
    reads: usize,
}

/// Exécute `run_blocking` sur un `Progressor` neuf.
///
/// - l'arrêt est demandé dès que `stop_at` invocations ont eu lieu (`0` : dès le départ) ;
/// - l'invocation de rang `fail_on` (à partir de 1) échoue ;
/// - si `deposit_in_wait`, l'attenteur dépose la valeur `n` à son `n`-ième appel.
fn run_with<C: FnMut() -> Duration>(
    clock: C,
    reads: &Reads,
    stop_at: usize,
    fail_on: Option<usize>,
    deposit_in_wait: bool,
) -> Outcome {
    let inbox = Inbox::<u32>::new(16);
    let deposit = inbox.handle();
    let log = Rc::new(RefCell::new(Vec::<Entry>::new()));
    let invocations = Rc::new(Cell::new(0usize));
    let (log_infra, inv_infra, inv_stop) = (
        Rc::clone(&log),
        Rc::clone(&invocations),
        Rc::clone(&invocations),
    );
    let progressor = Progressor::new(
        policy(),
        clock,
        move || inv_stop.get() >= stop_at,
        inbox,
        move |lot: &[u32]| {
            inv_infra.set(inv_infra.get() + 1);
            log_infra.borrow_mut().push(Entry::Lot(lot.to_vec()));
            if Some(inv_infra.get()) == fail_on {
                return Err(Boom {
                    code: 42,
                    tag: String::from("panne"),
                });
            }
            Ok(())
        },
    );
    let log_wait = Rc::clone(&log);
    let mut waits = 0u32;
    let result = run_blocking(progressor, move |deadline: Deadline| {
        log_wait
            .borrow_mut()
            .push(Entry::Wait(deadline_ns(deadline)));
        waits += 1;
        if deposit_in_wait {
            assert_eq!(deposit.push(waits), Ok(()));
        }
    });
    // Possession : le `Progressor` et l'attenteur sont libérés au retour, avec leurs captures.
    assert_eq!(Rc::strong_count(&log), 1, "fermetures libérées au retour");
    assert_eq!(Rc::strong_count(&invocations), 1);
    Outcome {
        result,
        log: Rc::try_unwrap(log).expect("seul détenteur").into_inner(),
        reads: reads.count(),
    }
}

fn run(sequence: &[u64], stop_at: usize, fail_on: Option<usize>, deposit_in_wait: bool) -> Outcome {
    let (clock, reads) = scripted_ns(sequence);
    run_with(clock, &reads, stop_at, fail_on, deposit_in_wait)
}

fn hors_plage() -> Duration {
    Duration::from_nanos(u64::MAX) + Duration::from_nanos(1)
}

// ---------------------------------------------------------------- D1 (pilote bloquant)

/// Script de référence : Σ0, échéance exacte, plafond K avec réveil déjà passé, puis arrêt.
///
/// Lectures : [100, 100] Σ0 (1 itération, réveil 110) ; [110, 110] (1, réveil 120) ;
/// [155, 155] (|D| = 5 > K : 3 itérations, R = 1 ≤ M conservé, réveil 150 déjà passé) ;
/// [155, 155] (1 itération, réveil 160) ; [160] arrêt observé. L'attenteur dépose `n` à son
/// `n`-ième appel : chaque dépôt entre dans le lot de l'itération suivante.
#[test]
fn d1_script_de_reference_lots_invocations_attentes_et_issue() {
    let sequence = [100, 100, 110, 110, 155, 155, 155, 155, 160];
    let outcome = run(&sequence, 6, None, true);

    assert!(matches!(outcome.result, Ok(6)), "total cumulé : 6");
    assert_eq!(
        outcome.log,
        [
            lot(&[]),
            Entry::Wait(110),
            lot(&[1]),
            Entry::Wait(120),
            lot(&[2]),
            lot(&[]),
            lot(&[]),
            Entry::Wait(150),
            lot(&[3]),
            Entry::Wait(160),
        ]
    );
    assert_eq!(outcome.reads, 9, "aucune lecture en trop ni en moins");
}

// ---------------------------------------------------------------- D2

/// D2 : réveil déjà passé (140 < 155). Il est transmis tel quel, sans progression
/// supplémentaire avant le retour de l'attenteur ; l'horloge scriptée panique à toute
/// lecture hors script.
#[test]
fn d2_reveil_deja_passe_transmis_tel_quel() {
    let outcome = run(&[100, 100, 155, 155, 155], 4, None, false);

    assert!(matches!(outcome.result, Ok(4)));
    assert_eq!(
        outcome.log,
        [
            lot(&[]),
            Entry::Wait(110),
            lot(&[]),
            lot(&[]),
            lot(&[]),
            Entry::Wait(140),
        ]
    );
    assert_eq!(outcome.reads, 5);
}

/// D2 : une progression qui n'exécute rien (rien dû, cas P1) rappelle bien l'attenteur,
/// avec la même échéance.
#[test]
fn d2_progression_sans_iteration_rappelle_l_attenteur() {
    let outcome = run(&[100, 100, 105, 110, 110, 120], 2, None, false);

    assert!(matches!(outcome.result, Ok(2)));
    assert_eq!(
        outcome.log,
        [
            lot(&[]),
            Entry::Wait(110),
            Entry::Wait(110),
            lot(&[]),
            Entry::Wait(120),
        ]
    );
    assert_eq!(outcome.reads, 6);
}

// ---------------------------------------------------------------- issues

#[test]
fn arret_immediat_total_nul_et_aucune_attente() {
    let outcome = run(&[100], 0, None, false);

    assert!(matches!(outcome.result, Ok(0)));
    assert!(outcome.log.is_empty(), "ni invocation ni attente");
    assert_eq!(outcome.reads, 1);
}

#[test]
fn erreur_d_infrastructure_rendue_inchangee_sans_attente_ensuite() {
    // Invocation 1 : Σ0 ; invocation 2 : en échec.
    let outcome = run(&[100, 100, 110, 110], usize::MAX, Some(2), false);

    match outcome.result {
        Err(ProgressError::Infrastructure(Boom { code, tag })) => {
            assert_eq!((code, tag.as_str()), (42, "panne"));
        }
        autre => panic!("issue inattendue : {autre:?}"),
    }
    assert_eq!(
        outcome.log,
        [lot(&[]), Entry::Wait(110), lot(&[])],
        "aucune attente après l'échec"
    );
    assert_eq!(outcome.reads, 3, "pas de t_fin après une erreur");
}

#[test]
fn horloge_hors_plage_rendue_comme_erreur_temporelle() {
    let (clock, reads) = scripted(&[
        Duration::from_nanos(100),
        Duration::from_nanos(100),
        hors_plage(),
    ]);
    let outcome = run_with(clock, &reads, usize::MAX, None, false);

    assert!(matches!(
        outcome.result,
        Err(ProgressError::Time(TimeError::ClockOutOfRange))
    ));
    assert_eq!(outcome.log, [lot(&[]), Entry::Wait(110)]);
    assert_eq!(outcome.reads, 3);
}

#[test]
fn debordement_d_echeancier_rendu_comme_erreur_temporelle() {
    let max = u64::MAX;
    // Σ0 à `max - 25` (réveil `max - 15`), puis deux itérations dont la seconde déborde.
    let outcome = run(&[max - 25, max - 25, max - 5], usize::MAX, None, false);

    assert!(matches!(
        outcome.result,
        Err(ProgressError::Time(TimeError::ScheduleOverflow))
    ));
    assert_eq!(
        outcome.log,
        [lot(&[]), Entry::Wait(max - 15), lot(&[]), lot(&[])],
        "les itérations achevées avant le débordement ont bien été invoquées"
    );
    assert_eq!(outcome.reads, 3);
}

// ---------------------------------------------------------------- précondition d'entrée

/// Violation de la précondition : un `Progressor` déjà en échec dont l'erreur a été prélevée
/// par l'appelant. `run_blocking` ne rend ni succès fictif ni erreur de substitution : il
/// panique. Le message attendu distingue ce panic de celui de l'attenteur.
#[test]
#[should_panic(expected = "précondition violée")]
fn pilote_refuse_un_progressor_en_echec_dont_l_erreur_est_prelevee() {
    let (clock, _reads) = scripted_ns(&[100]);
    let mut progressor = Progressor::new(
        policy(),
        clock,
        || false,
        Inbox::<u32>::new(4),
        |_lot: &[u32]| {
            Err::<(), Boom>(Boom {
                code: 1,
                tag: String::from("déjà en échec"),
            })
        },
    );
    let report = progressor.progress();
    assert_eq!(report.state, ProgressState::Failed);
    assert!(progressor.take_error().is_some());

    let _ = run_blocking(progressor, |_deadline: Deadline| {
        panic!("aucune attente attendue");
    });
}
