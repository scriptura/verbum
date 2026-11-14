// crates/engine/shell/tests/temporal_progress.rs

//! ADR-S04 (unités T5a et T5b) : progression du `Progressor`, valeurs de test q = 10 ns,
//! K = 3, M = 5.
//!
//! - TP-P : P0, P0b, P1 à P8 (progression, conservation, plafond K, absence de dérive) ;
//! - TP-S : S1 à S5, S6a, S6b, S7 (seuil, abandon, réancrage R1) ;
//! - TP-E : E1, E3 (bras terminal d'erreur d'infrastructure), E7, E8, E9 (bras temporels) ;
//! - T5b, garanties d'arrêt, d'erreur et d'acquisition : E2, E4, E5, E6, E10, D9.
//!
//! Chaque scénario repart d'un `Progressor` neuf suivi du préfixe Σ0 (lectures [100, 100] :
//! 1 itération, réveil 110), sauf mention contraire. L'horloge est scriptée, monotone et
//! comptée (`temporal_support`) : aucun `sleep`, aucun `Instant`.
//!
//! **Non démontré ici** : D1, D7, D8 (unités ultérieures).

#[path = "temporal_support/mod.rs"]
#[allow(dead_code, reason = "harnais partagé avec les unités de pilotes")]
mod temporal_support;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use temporal_support::{Reads, scripted, scripted_ns};
use verbum_shell::{
    DepositHandle, Inbox, Progress, ProgressError, ProgressState, Progressor, TemporalPolicy,
    TimeError, Wake,
};

/// Erreur d'infrastructure de test : ni `Clone` ni `Copy`, pour prouver le transport inchangé.
#[derive(Debug)]
struct Boom {
    code: u32,
    tag: String,
}

/// Observations partagées avec le `Progressor`.
struct Env {
    /// Un lot par invocation de l'infrastructure, dans l'ordre (l'invocation en échec comprise).
    lots: Rc<RefCell<Vec<Vec<u32>>>>,
    reads: Reads,
    stop: Rc<Cell<bool>>,
    deposit: DepositHandle<u32>,
}

fn policy(q: u64, k: u32, m: u32) -> TemporalPolicy {
    TemporalPolicy::from_nanos(q, k, m).expect("politique de test valide")
}

#[allow(
    clippy::type_complexity,
    reason = "types de fermetures non nommables : `impl Trait` en retour, sans alias possible"
)]
fn rig<C: FnMut() -> Duration>(
    policy: TemporalPolicy,
    clock: C,
    reads: Reads,
    fail_on: Option<usize>,
) -> (
    Progressor<u32, Boom, C, impl FnMut() -> bool, impl FnMut(&[u32]) -> Result<(), Boom>>,
    Env,
) {
    let inbox = Inbox::<u32>::new(16);
    let deposit = inbox.handle();
    let lots = Rc::new(RefCell::new(Vec::<Vec<u32>>::new()));
    let stop = Rc::new(Cell::new(false));
    let lots_in = Rc::clone(&lots);
    let stop_in = Rc::clone(&stop);
    let progressor = Progressor::new(
        policy,
        clock,
        move || stop_in.get(),
        inbox,
        move |lot: &[u32]| {
            lots_in.borrow_mut().push(lot.to_vec());
            if Some(lots_in.borrow().len()) == fail_on {
                return Err(Boom {
                    code: 42,
                    tag: String::from("panne"),
                });
            }
            Ok(())
        },
    );
    (
        progressor,
        Env {
            lots,
            reads,
            stop,
            deposit,
        },
    )
}

fn wake_ns(progress: &Progress) -> Option<u64> {
    match progress.wake {
        Wake::At(deadline) => {
            Some(u64::try_from(deadline.as_duration().as_nanos()).expect("tient en u64"))
        }
        Wake::NoDeadline => None,
    }
}

fn expect(progress: &Progress, iterations: u32, wake: Option<u64>, state: ProgressState) {
    assert_eq!(progress.iterations, iterations, "itérations");
    assert_eq!(wake_ns(progress), wake, "réveil");
    assert_eq!(progress.state, state, "state");
}

/// Σ0 : première progression à `t0` (lectures [t0, t0]) : 1 itération, réveil `t0 + q`.
#[allow(
    clippy::type_complexity,
    reason = "types de fermetures non nommables : `impl Trait` en retour, sans alias possible"
)]
fn started(
    policy: TemporalPolicy,
    t0: u64,
    lines: &[u64],
    fail_on: Option<usize>,
) -> (
    Progressor<
        u32,
        Boom,
        impl FnMut() -> Duration + use<>,
        impl FnMut() -> bool + use<>,
        impl FnMut(&[u32]) -> Result<(), Boom> + use<>,
    >,
    Env,
) {
    let mut sequence = vec![t0, t0];
    sequence.extend_from_slice(lines);
    let (clock, reads) = scripted_ns(&sequence);
    let (mut progressor, env) = rig(policy, clock, reads, fail_on);
    let first = progressor.progress();
    expect(&first, 1, Some(t0 + 10), ProgressState::Running);
    assert_eq!(env.reads.count(), 2, "Σ0 : deux lectures");
    (progressor, env)
}

// ---------------------------------------------------------------- harnais

#[test]
#[should_panic(expected = "décroissante")]
fn harnais_refuse_une_suite_decroissante() {
    let _ = scripted_ns(&[100, 99]);
}

// ---------------------------------------------------------------- construction

#[test]
fn construction_sans_lecture_ni_invocation() {
    // I-3 : l'échéancier est établi à la première progression, non à la construction.
    let (clock, reads) = scripted_ns(&[100, 100]);
    let (_progressor, env) = rig(policy(10, 3, 5), clock, reads, None);
    assert_eq!(
        env.reads.count(),
        0,
        "aucune lecture d'horloge à la construction"
    );
    assert!(
        env.lots.borrow().is_empty(),
        "aucune invocation à la construction"
    );
}

// ---------------------------------------------------------------- TP-P

#[test]
fn p0_premiere_progression_toujours_due() {
    let (clock, reads) = scripted_ns(&[100, 100]);
    let (mut p, env) = rig(policy(10, 3, 5), clock, reads, None);
    expect(&p.progress(), 1, Some(110), ProgressState::Running);
    assert_eq!(env.reads.count(), 2);
    assert_eq!(*env.lots.borrow(), vec![Vec::<u32>::new()]);
}

#[test]
fn p0b_pas_de_rafale_a_l_initialisation() {
    let (clock, reads) = scripted_ns(&[1_000_000_000, 1_000_000_000]);
    let (mut p, env) = rig(policy(10, 3, 5), clock, reads, None);
    expect(
        &p.progress(),
        1,
        Some(1_000_000_010),
        ProgressState::Running,
    );
    assert_eq!(env.lots.borrow().len(), 1);
}

#[test]
fn p1_avant_l_echeance_aucune_coupure() {
    let (mut p, env) = {
        let (clock, reads) = scripted_ns(&[100, 100, 105, 110, 110]);
        rig(policy(10, 3, 5), clock, reads, None)
    };
    expect(&p.progress(), 1, Some(110), ProgressState::Running);
    assert_eq!(env.deposit.push(7), Ok(()));
    expect(&p.progress(), 0, Some(110), ProgressState::Running);
    assert_eq!(env.reads.count(), 3, "une seule lecture");
    assert_eq!(env.lots.borrow().len(), 1, "aucune coupure ni invocation");
    // Le dépôt est resté dans la boîte : il appartient au lot de la première itération effective.
    expect(&p.progress(), 1, Some(120), ProgressState::Running);
    assert_eq!(*env.lots.borrow(), vec![vec![], vec![7]]);
}

#[test]
fn p2_a_p3_echeance_exacte_et_retard_inferieur_a_q() {
    for (line, wake) in [([110u64, 110], 120u64), ([115, 115], 120)] {
        let (mut p, env) = started(policy(10, 3, 5), 100, &line, None);
        expect(&p.progress(), 1, Some(wake), ProgressState::Running);
        assert_eq!(env.reads.count(), 4, "{line:?}");
    }
}

#[test]
fn p4_plusieurs_dues_egal_k() {
    let (mut p, env) = started(policy(10, 3, 5), 100, &[130, 130], None);
    expect(&p.progress(), 3, Some(140), ProgressState::Running);
    assert_eq!(env.lots.borrow().len(), 4);
}

#[test]
fn p5_plafond_k_residuel_conserve() {
    let (mut p, _env) = started(policy(10, 3, 5), 100, &[155, 155], None);
    // |D| = 5 > K ; R = 2 <= M conservé ; réveil déjà passé (140 < 155).
    expect(&p.progress(), 3, Some(140), ProgressState::Running);
}

#[test]
fn p6_echeances_nees_pendant_l_execution() {
    let (mut p, _env) = started(policy(10, 3, 5), 100, &[110, 135], None);
    // Ensemble exécuté fixé à l'entrée : une seule itération, malgré t_fin = 135.
    expect(&p.progress(), 1, Some(120), ProgressState::Running);
}

#[test]
fn p7_rattrapage_chaine() {
    let (mut p, _env) = started(policy(10, 3, 5), 100, &[155, 155, 160, 160], None);
    expect(&p.progress(), 3, Some(140), ProgressState::Running);
    expect(&p.progress(), 3, Some(170), ProgressState::Running);
}

#[test]
fn p8_absence_de_derive() {
    let mut lines = Vec::new();
    for i in 0..100_000u64 {
        let t = 110 + 10 * i;
        lines.push(t);
        lines.push(t);
    }
    let (mut p, env) = started(policy(10, 3, 5), 100, &lines, None);
    let mut dernier = None;
    for _ in 0..100_000 {
        let r = p.progress();
        assert_eq!(r.iterations, 1);
        dernier = wake_ns(&r);
    }
    assert_eq!(dernier, Some(110 + 10 * 100_000));
    assert_eq!(env.reads.count(), 2 + 200_000);
}

// ---------------------------------------------------------------- TP-S

#[test]
fn s1_r_egal_m_conserve() {
    let (mut p, _env) = started(policy(10, 3, 5), 100, &[110, 169], None);
    expect(&p.progress(), 1, Some(120), ProgressState::Running);
}

#[test]
fn s2_r_egal_m_plus_1_abandonne() {
    let (mut p, _env) = started(policy(10, 3, 5), 100, &[110, 170], None);
    expect(&p.progress(), 1, Some(180), ProgressState::Running);
}

#[test]
fn s3_aucune_echeance_abandonnee_n_est_executee() {
    let (mut p, env) = started(policy(10, 3, 5), 100, &[110, 170, 179, 180, 180], None);
    expect(&p.progress(), 1, Some(180), ProgressState::Running);
    expect(&p.progress(), 0, Some(180), ProgressState::Running);
    expect(&p.progress(), 1, Some(190), ProgressState::Running);
    assert_eq!(
        env.lots.borrow().len(),
        3,
        "Σ0, S2 et la dernière seulement"
    );
}

#[test]
fn s4_interruption_longue() {
    let (mut p, _env) = started(policy(10, 3, 5), 100, &[10_110, 10_110], None);
    // K itérations tardives exécutées avant l'abandon (R = 998 > M).
    expect(&p.progress(), 3, Some(10_120), ProgressState::Running);
}

#[test]
fn s5_phase_non_conservee() {
    let (mut p, _env) = started(policy(10, 3, 5), 100, &[10_113, 10_113], None);
    expect(&p.progress(), 3, Some(10_123), ProgressState::Running);
}

#[test]
fn s6a_m_egal_1_r_egal_1_conserve() {
    let (mut p, _env) = started(policy(10, 3, 1), 100, &[110, 120], None);
    expect(&p.progress(), 1, Some(120), ProgressState::Running);
}

#[test]
fn s6b_m_egal_1_r_egal_2_abandonne() {
    let (mut p, _env) = started(policy(10, 3, 1), 100, &[110, 130], None);
    expect(&p.progress(), 1, Some(140), ProgressState::Running);
}

#[test]
fn s7_reliquat_et_nees_pendant_l_execution() {
    let (mut p, _env) = started(policy(10, 3, 5), 100, &[110, 150], None);
    expect(&p.progress(), 1, Some(120), ProgressState::Running);
}

// ---------------------------------------------------------------- TP-E

#[test]
fn e1_arret_avant_la_premiere_iteration() {
    let (clock, reads) = scripted_ns(&[100]);
    let (mut p, env) = rig(policy(10, 3, 5), clock, reads, None);
    env.stop.set(true);
    expect(&p.progress(), 0, None, ProgressState::Stopped);
    assert_eq!(env.reads.count(), 1, "une seule lecture, pas de t_fin");
    assert!(env.lots.borrow().is_empty(), "aucune invocation");
}

#[test]
fn e3_erreur_d_infrastructure_inchangee() {
    // Invocation 1 : Σ0 ; invocation 2 : réussie ; invocation 3 : en échec (2e itération).
    let (mut p, env) = started(policy(10, 3, 5), 100, &[130, 130], Some(3));
    expect(&p.progress(), 1, None, ProgressState::Failed);
    assert_eq!(env.reads.count(), 3, "pas de t_fin après une erreur");
    assert_eq!(
        env.lots.borrow().len(),
        3,
        "aucune invocation après l'échec"
    );
    match p.take_error() {
        Some(ProgressError::Infrastructure(Boom { code, tag })) => {
            assert_eq!((code, tag.as_str()), (42, "panne"));
        }
        autre => panic!("cause inattendue : {autre:?}"),
    }
}

fn e7(t0: u64, iterations: u32) {
    let max = u64::MAX;
    let (mut p, env) = started(policy(10, 3, 5), t0, &[max - 5], None);
    expect(&p.progress(), iterations, None, ProgressState::Failed);
    assert_eq!(env.reads.count(), 3, "pas de t_fin");
    assert_eq!(
        env.lots.borrow().len(),
        1 + usize::try_from(iterations).expect("petit"),
        "les itérations rapportées ont bien été invoquées"
    );
    assert!(matches!(
        p.take_error(),
        Some(ProgressError::Time(TimeError::ScheduleOverflow))
    ));
}

#[test]
fn e7_debordement_apres_la_2e_iteration() {
    e7(u64::MAX - 25, 2);
}

#[test]
fn e7_debordement_apres_la_3e_iteration() {
    e7(u64::MAX - 35, 3);
}

#[test]
fn e7_debordement_au_reancrage_r1() {
    let (mut p, env) = started(policy(10, 3, 5), 100, &[110, u64::MAX], None);
    expect(&p.progress(), 1, None, ProgressState::Failed);
    assert_eq!(env.reads.count(), 4);
    assert!(matches!(
        p.take_error(),
        Some(ProgressError::Time(TimeError::ScheduleOverflow))
    ));
}

#[test]
fn e8_progression_apres_terminal() {
    // Arrêt.
    let (mut p, env) = started(policy(10, 3, 5), 100, &[110], None);
    env.stop.set(true);
    expect(&p.progress(), 0, None, ProgressState::Stopped);
    // Erreur d'infrastructure.
    let (mut q, env_q) = started(policy(10, 3, 5), 100, &[110], Some(2));
    expect(&q.progress(), 0, None, ProgressState::Failed);
    // Erreur temporelle.
    let (mut r, env_r) = started(policy(10, 3, 5), 100, &[110, u64::MAX], None);
    expect(&r.progress(), 1, None, ProgressState::Failed);

    verifie_terminal(&mut p, &env, ProgressState::Stopped);
    verifie_terminal(&mut q, &env_q, ProgressState::Failed);
    verifie_terminal(&mut r, &env_r, ProgressState::Failed);
}

/// Progressions après terminal : 0 itération, state inchangée, 0 lecture, aucune coupure.
fn verifie_terminal<C, S, I>(
    progressor: &mut Progressor<u32, Boom, C, S, I>,
    env: &Env,
    state: ProgressState,
) where
    C: FnMut() -> Duration,
    S: FnMut() -> bool,
    I: FnMut(&[u32]) -> Result<(), Boom>,
{
    let lectures = env.reads.count();
    let lots = env.lots.borrow().len();
    assert_eq!(env.deposit.push(9), Ok(()));
    for _ in 0..3 {
        expect(&progressor.progress(), 0, None, state);
    }
    assert_eq!(env.reads.count(), lectures, "aucune lecture");
    assert_eq!(env.lots.borrow().len(), lots, "aucune coupure");
}

fn hors_plage() -> Duration {
    Duration::from_nanos(u64::MAX) + Duration::from_nanos(1)
}

#[test]
fn e9_horloge_hors_plage_a_l_entree() {
    let (clock, reads) = scripted(&[hors_plage()]);
    let (mut p, env) = rig(policy(10, 3, 5), clock, reads, None);
    expect(&p.progress(), 0, None, ProgressState::Failed);
    assert_eq!(env.reads.count(), 1);
    assert!(env.lots.borrow().is_empty());
    assert!(matches!(
        p.take_error(),
        Some(ProgressError::Time(TimeError::ClockOutOfRange))
    ));
}

#[test]
fn e9_horloge_hors_plage_en_fin_de_progression() {
    let (clock, reads) = scripted(&[
        Duration::from_nanos(100),
        Duration::from_nanos(100),
        Duration::from_nanos(110),
        hors_plage(),
    ]);
    let (mut p, env) = rig(policy(10, 3, 5), clock, reads, None);
    expect(&p.progress(), 1, Some(110), ProgressState::Running);
    // L'itération exécutée reste rapportée ; la cause est distincte de `ScheduleOverflow`.
    expect(&p.progress(), 1, None, ProgressState::Failed);
    assert_eq!(env.reads.count(), 4);
    assert_eq!(env.lots.borrow().len(), 2);
    assert!(matches!(
        p.take_error(),
        Some(ProgressError::Time(TimeError::ClockOutOfRange))
    ));
}

// ---------------------------------------------------------------- T5b : arrêt

#[test]
fn e2_arret_apres_j_iterations() {
    let (clock, reads) = scripted_ns(&[100, 100, 130, 130]);
    let invocations = Rc::new(Cell::new(0u32));
    let observations = Rc::new(Cell::new(0u32));
    let (for_stop, for_count, for_infra) = (
        Rc::clone(&invocations),
        Rc::clone(&observations),
        Rc::clone(&invocations),
    );
    let mut p = Progressor::new(
        policy(10, 3, 5),
        clock,
        // L'arrêt est demandé dès que trois invocations ont eu lieu (Σ0 + deux itérations).
        move || {
            for_count.set(for_count.get() + 1);
            for_stop.get() >= 3
        },
        Inbox::<u32>::new(4),
        move |_lot: &[u32]| {
            for_infra.set(for_infra.get() + 1);
            Ok::<(), Boom>(())
        },
    );
    expect(&p.progress(), 1, Some(110), ProgressState::Running);
    // |D| = 3 ; j = 2 itérations achevées, l'arrêt est observé à la troisième.
    expect(&p.progress(), 2, None, ProgressState::Stopped);
    assert_eq!(
        reads.count(),
        3,
        "une seule lecture : ni t_fin ni décision M"
    );
    assert_eq!(invocations.get(), 3, "aucune invocation après l'arrêt");
    assert_eq!(
        observations.get(),
        4,
        "Σ0 : 1 observation ; progression : 3"
    );
}

#[test]
fn d9_arret_demande_quand_rien_n_est_du_non_observe() {
    let (clock, reads) = scripted_ns(&[100, 100, 105, 110, 110]);
    let requested = Rc::new(Cell::new(false));
    let observations = Rc::new(Cell::new(0u32));
    let (for_stop, for_count) = (Rc::clone(&requested), Rc::clone(&observations));
    let mut p = Progressor::new(
        policy(10, 3, 5),
        clock,
        move || {
            for_count.set(for_count.get() + 1);
            for_stop.get()
        },
        Inbox::<u32>::new(4),
        |_lot: &[u32]| Ok::<(), Boom>(()),
    );
    expect(&p.progress(), 1, Some(110), ProgressState::Running);
    assert_eq!(observations.get(), 1);
    requested.set(true);
    // D vide : aucune itération, donc aucune observation de l'arrêt (§ 9.4).
    expect(&p.progress(), 0, Some(110), ProgressState::Running);
    assert_eq!(observations.get(), 1, "arrêt non observé quand D est vide");
    // Première progression qui exécute une itération : l'arrêt est observé.
    expect(&p.progress(), 0, None, ProgressState::Stopped);
    assert_eq!(observations.get(), 2);
    assert_eq!(reads.count(), 4);
}

// ---------------------------------------------------------------- T5b : erreur

#[test]
fn e4_echec_de_publication_avant_le_tick() {
    // Capacité du porteur : 3. L'infrastructure publie puis exécute le tick, en une séquence ;
    // un lot trop grand échoue **avant** le tick (cf. AC-7).
    let (clock, reads) = scripted_ns(&[100, 100, 130, 130]);
    let inbox = Inbox::<u32>::new(8);
    let deposit = inbox.handle();
    let ticks = Rc::new(Cell::new(0u32));
    let for_infra = Rc::clone(&ticks);
    let mut p = Progressor::new(
        policy(10, 3, 5),
        clock,
        || false,
        inbox,
        move |lot: &[u32]| {
            if lot.len() > 3 {
                return Err(Boom {
                    code: 7,
                    tag: String::from("porteur trop petit"),
                });
            }
            for_infra.set(for_infra.get() + 1);
            Ok(())
        },
    );
    expect(&p.progress(), 1, Some(110), ProgressState::Running);
    for e in [1, 2, 3, 4] {
        assert_eq!(deposit.push(e), Ok(()));
    }
    // Même issue qu'E3 : itération non lancée, non comptée, aucune invocation ensuite.
    expect(&p.progress(), 0, None, ProgressState::Failed);
    assert_eq!(
        ticks.get(),
        1,
        "le tick de l'itération en échec n'a pas eu lieu"
    );
    assert_eq!(reads.count(), 3, "pas de t_fin après une erreur");
    match p.take_error() {
        Some(ProgressError::Infrastructure(Boom { code, tag })) => {
            assert_eq!((code, tag.as_str()), (7, "porteur trop petit"));
        }
        autre => panic!("cause inattendue : {autre:?}"),
    }
    expect(&p.progress(), 0, None, ProgressState::Failed);
    assert_eq!(ticks.get(), 1);
    assert_eq!(reads.count(), 3);
}

#[test]
fn e10_take_error_une_seule_fois_etat_stable() {
    let (mut p, env) = started(policy(10, 3, 5), 100, &[130, 130], Some(3));
    expect(&p.progress(), 1, None, ProgressState::Failed);
    assert!(matches!(
        p.take_error(),
        Some(ProgressError::Infrastructure(Boom { code: 42, .. }))
    ));
    assert!(p.take_error().is_none(), "second appel : None");
    assert!(p.take_error().is_none());
    // L'état reste `Failed` : aucune progression relancée, aucune lecture, aucune coupure.
    let (lectures, lots) = (env.reads.count(), env.lots.borrow().len());
    expect(&p.progress(), 0, None, ProgressState::Failed);
    assert_eq!(env.reads.count(), lectures);
    assert_eq!(env.lots.borrow().len(), lots);
}

// ---------------------------------------------------------------- T5b : acquisition

#[test]
fn e5_depot_pendant_une_iteration_lot_suivant_sans_rejeu() {
    let (clock, reads) = scripted_ns(&[100, 100, 130, 130]);
    let inbox = Inbox::<u32>::new(8);
    let deposit = inbox.handle();
    let in_infra = inbox.handle();
    let lots = Rc::new(RefCell::new(Vec::<Vec<u32>>::new()));
    let for_infra = Rc::clone(&lots);
    let mut p = Progressor::new(
        policy(10, 3, 5),
        clock,
        || false,
        inbox,
        move |lot: &[u32]| {
            let mut lots = for_infra.borrow_mut();
            lots.push(lot.to_vec());
            // Dépôt **pendant** l'invocation : il appartiendra à l'itération suivante.
            match lots.len() {
                2 => assert_eq!(in_infra.push(11), Ok(())),
                3 => assert_eq!(in_infra.push(12), Ok(())),
                _ => {}
            }
            Ok::<(), Boom>(())
        },
    );
    expect(&p.progress(), 1, Some(110), ProgressState::Running);
    assert_eq!(deposit.push(5), Ok(()));
    // Progression de trois itérations : un lot par itération, chacun avec sa propre coupure.
    expect(&p.progress(), 3, Some(140), ProgressState::Running);
    assert_eq!(*lots.borrow(), vec![vec![], vec![5], vec![11], vec![12]]);
    let aplati: Vec<u32> = lots.borrow().iter().flatten().copied().collect();
    assert_eq!(aplati, vec![5, 11, 12], "chaque événement une seule fois");
    assert_eq!(reads.count(), 4);
}

#[test]
fn e6_depot_puis_rien_du_puis_premiere_iteration_effective() {
    let (mut p, env) = started(policy(10, 3, 5), 100, &[105, 108, 110, 110], None);
    assert_eq!(env.deposit.push(4), Ok(()));
    assert_eq!(env.deposit.push(5), Ok(()));
    // Deux progressions « rien dû » : aucune coupure, le dépôt reste dans la boîte.
    expect(&p.progress(), 0, Some(110), ProgressState::Running);
    expect(&p.progress(), 0, Some(110), ProgressState::Running);
    assert_eq!(env.lots.borrow().len(), 1);
    // Première itération effective : un seul lot, dans l'ordre d'acceptation.
    expect(&p.progress(), 1, Some(120), ProgressState::Running);
    assert_eq!(*env.lots.borrow(), vec![vec![], vec![4, 5]]);
}
