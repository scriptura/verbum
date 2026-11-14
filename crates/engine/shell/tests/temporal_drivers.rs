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
//! ADR-S04 (unité T7) : harnais du pilote événementiel, sans `winit`.
//!
//! - D1 complet : chaque scénario est exécuté deux fois, indépendamment, par `run_blocking`
//!   et par un harnais qui simule `about_to_wait` ; les deux exécutions sont comparées entre
//!   elles **et** à des littéraux (deux pilotes pourraient partager une même erreur). Le
//!   harnais événementiel enregistre une intention d'attente en `Deadline` : D1 n'utilise
//!   aucun `Instant` ;
//! - D3 : conversion `Deadline → Instant` par `checked_add` (une seule origine lue) ;
//! - D6 : dépôt depuis un autre fil entre deux progressions, jointure, sans `sleep`.
//!
//! L'horloge et l'attenteur sont scriptés (`temporal_support`) : aucun `sleep`. Une lecture
//! d'horloge ou une progression en trop est un échec de test, car l'horloge panique hors de
//! sa suite. Seul D3 lit une horloge réelle (par `MonotonicClock::start`, pour l'origine).
//!
//! **Non démontré ici** : l'enveloppe réelle `sleep_until` (sommeil) ; son calcul est testé
//! dans `src/driver.rs`. `Deadline` non représentable en `Instant` (PO-3) et plateformes
//! réelles : adaptateur natif.

#[path = "temporal_support/mod.rs"]
#[allow(dead_code, reason = "harnais partagé avec les unités de pilotes")]
mod temporal_support;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use temporal_support::{Reads, scripted, scripted_ns};
use verbum_shell::{
    Deadline, Inbox, MonotonicClock, ProgressError, ProgressState, Progressor, TemporalPolicy,
    TimeError, Wake, run_blocking,
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
    /// État terminal observé : seul le harnais événementiel le lit (`run_blocking` ne le
    /// rend pas ; il se déduit de l'issue).
    terminal: Option<ProgressState>,
}

/// Pilote sous test.
#[derive(Debug, Clone, Copy)]
enum Driver {
    Blocking,
    Event,
}

/// Ce que `about_to_wait` peut demander à la plateforme simulée : une échéance d'attente
/// (en `Deadline`, sans `Instant`) ou la sortie de la boucle.
#[derive(Default)]
struct FakeEventLoop {
    wait_until: Option<Deadline>,
    exit: bool,
}

/// Application de plateforme simulée : possède le `Progressor` sur le fil de la boucle.
struct EventApp<Ev: Copy, E, C, S, I> {
    progressor: Progressor<Ev, E, C, S, I>,
    total: u64,
    last_state: ProgressState,
    result: Option<Result<u64, ProgressError<E>>>,
}

impl<Ev, E, C, S, I> EventApp<Ev, E, C, S, I>
where
    Ev: Copy,
    C: FnMut() -> Duration,
    S: FnMut() -> bool,
    I: FnMut(&[Ev]) -> Result<(), E>,
{
    fn new(progressor: Progressor<Ev, E, C, S, I>) -> Self {
        Self {
            progressor,
            total: 0,
            last_state: ProgressState::Running,
            result: None,
        }
    }

    /// Point d'accroche simulé : une progression, puis la consigne rendue à la plateforme.
    fn about_to_wait(&mut self, event_loop: &mut FakeEventLoop) {
        let report = self.progressor.progress();
        self.total += u64::from(report.iterations);
        self.last_state = report.state;
        match (report.state, report.wake) {
            (ProgressState::Running, Wake::At(deadline)) => event_loop.wait_until = Some(deadline),
            (ProgressState::Running, Wake::NoDeadline) => {
                panic!("état non terminal sans échéance (C-10)")
            }
            (ProgressState::Stopped, _) => {
                self.result = Some(Ok(self.total));
                event_loop.exit = true;
            }
            (ProgressState::Failed, _) => {
                let error = self.progressor.take_error().expect("cause de l'échec");
                self.result = Some(Err(error));
                event_loop.exit = true;
            }
        }
    }
}

/// Boucle de plateforme simulée : rappelle `about_to_wait`, attend l'échéance demandée par
/// `on_wait`, sort quand la sortie est demandée. Rend l'issue et le dernier état observé.
fn run_event_loop<Ev, E, C, S, I>(
    mut app: EventApp<Ev, E, C, S, I>,
    mut on_wait: impl FnMut(Deadline),
) -> (Result<u64, ProgressError<E>>, ProgressState)
where
    Ev: Copy,
    C: FnMut() -> Duration,
    S: FnMut() -> bool,
    I: FnMut(&[Ev]) -> Result<(), E>,
{
    let mut event_loop = FakeEventLoop::default();
    while !event_loop.exit {
        app.about_to_wait(&mut event_loop);
        if let Some(deadline) = event_loop.wait_until.take() {
            on_wait(deadline);
        }
    }
    let result = app.result.take().expect("issue produite avant la sortie");
    (result, app.last_state)
}

/// Projection de l'issue propre au test : `Boom` n'est ni `Clone` ni `PartialEq`.
#[derive(Debug, PartialEq, Eq)]
enum Issue {
    Total(u64),
    Infrastructure { code: u32, tag: String },
    Time(String),
}

fn issue(result: &Result<u64, ProgressError<Boom>>) -> Issue {
    match result {
        Ok(total) => Issue::Total(*total),
        Err(ProgressError::Infrastructure(Boom { code, tag })) => Issue::Infrastructure {
            code: *code,
            tag: tag.clone(),
        },
        Err(ProgressError::Time(error)) => Issue::Time(format!("{error:?}")),
    }
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
    run_with_driver(
        Driver::Blocking,
        clock,
        reads,
        stop_at,
        fail_on,
        deposit_in_wait,
    )
}

/// Comme [`run_with`], pour le pilote choisi. L'attenteur (ou la plateforme simulée) enregistre
/// l'échéance demandée et dépose la valeur `n` à sa `n`-ième attente.
fn run_with_driver<C: FnMut() -> Duration>(
    driver: Driver,
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
    let on_wait = move |deadline: Deadline| {
        log_wait
            .borrow_mut()
            .push(Entry::Wait(deadline_ns(deadline)));
        waits += 1;
        if deposit_in_wait {
            assert_eq!(deposit.push(waits), Ok(()));
        }
    };
    let (result, terminal) = match driver {
        Driver::Blocking => (run_blocking(progressor, on_wait), None),
        Driver::Event => {
            let (result, state) = run_event_loop(EventApp::new(progressor), on_wait);
            (result, Some(state))
        }
    };
    // Possession : le `Progressor` et l'attenteur sont libérés au retour, avec leurs captures.
    assert_eq!(Rc::strong_count(&log), 1, "fermetures libérées au retour");
    assert_eq!(Rc::strong_count(&invocations), 1);
    Outcome {
        result,
        log: Rc::try_unwrap(log).expect("seul détenteur").into_inner(),
        reads: reads.count(),
        terminal,
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

// ---------------------------------------------------------------- T7 : D1 complet

/// Scénario exécuté à l'identique par chaque pilote, sur une horloge scriptée neuve.
struct Scenario {
    script: Vec<Duration>,
    stop_at: usize,
    fail_on: Option<usize>,
    deposit_in_wait: bool,
}

impl Scenario {
    fn ns(sequence: &[u64], stop_at: usize) -> Self {
        Self {
            script: sequence.iter().map(|&n| Duration::from_nanos(n)).collect(),
            stop_at,
            fail_on: None,
            deposit_in_wait: false,
        }
    }

    fn run(&self, driver: Driver) -> Outcome {
        let (clock, reads) = scripted(&self.script);
        run_with_driver(
            driver,
            clock,
            &reads,
            self.stop_at,
            self.fail_on,
            self.deposit_in_wait,
        )
    }
}

/// Résultat littéral attendu du scénario, quel que soit le pilote.
struct Expected {
    issue: Issue,
    log: Vec<Entry>,
    reads: usize,
    terminal: ProgressState,
}

/// D1 : deux exécutions indépendantes, comparées entre elles (lots et ordre, invocations,
/// échéances demandées, issue, lectures d'horloge) et chacune à des littéraux.
fn assert_equivalence(scenario: &Scenario, expected: &Expected) {
    let blocking = scenario.run(Driver::Blocking);
    let event = scenario.run(Driver::Event);

    // Différentiel.
    assert_eq!(blocking.log, event.log, "lots, invocations et échéances");
    assert_eq!(issue(&blocking.result), issue(&event.result), "issue");
    assert_eq!(blocking.reads, event.reads, "lectures d'horloge");

    // Littéraux : les deux pilotes pourraient partager une même erreur.
    for (nom, outcome) in [("bloquant", &blocking), ("événementiel", &event)] {
        assert_eq!(outcome.log, expected.log, "{nom} : journal");
        assert_eq!(issue(&outcome.result), expected.issue, "{nom} : issue");
        assert_eq!(outcome.reads, expected.reads, "{nom} : lectures");
    }
    assert!(blocking.terminal.is_none());
    assert_eq!(event.terminal, Some(expected.terminal), "état terminal");
}

#[test]
fn d1_equivalence_script_de_reference() {
    let mut scenario = Scenario::ns(&[100, 100, 110, 110, 155, 155, 155, 155, 160], 6);
    scenario.deposit_in_wait = true;
    assert_equivalence(
        &scenario,
        &Expected {
            issue: Issue::Total(6),
            log: vec![
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
            ],
            reads: 9,
            terminal: ProgressState::Stopped,
        },
    );
}

#[test]
fn d1_equivalence_reveil_deja_passe() {
    assert_equivalence(
        &Scenario::ns(&[100, 100, 155, 155, 155], 4),
        &Expected {
            issue: Issue::Total(4),
            log: vec![
                lot(&[]),
                Entry::Wait(110),
                lot(&[]),
                lot(&[]),
                lot(&[]),
                Entry::Wait(140),
            ],
            reads: 5,
            terminal: ProgressState::Stopped,
        },
    );
}

#[test]
fn d1_equivalence_progression_sans_iteration() {
    assert_equivalence(
        &Scenario::ns(&[100, 100, 105, 110, 110, 120], 2),
        &Expected {
            issue: Issue::Total(2),
            log: vec![
                lot(&[]),
                Entry::Wait(110),
                Entry::Wait(110),
                lot(&[]),
                Entry::Wait(120),
            ],
            reads: 6,
            terminal: ProgressState::Stopped,
        },
    );
}

#[test]
fn d1_equivalence_arret_immediat() {
    assert_equivalence(
        &Scenario::ns(&[100], 0),
        &Expected {
            issue: Issue::Total(0),
            log: vec![],
            reads: 1,
            terminal: ProgressState::Stopped,
        },
    );
}

#[test]
fn d1_equivalence_erreur_d_infrastructure() {
    let mut scenario = Scenario::ns(&[100, 100, 110, 110], usize::MAX);
    scenario.fail_on = Some(2);
    assert_equivalence(
        &scenario,
        &Expected {
            issue: Issue::Infrastructure {
                code: 42,
                tag: String::from("panne"),
            },
            log: vec![lot(&[]), Entry::Wait(110), lot(&[])],
            reads: 3,
            terminal: ProgressState::Failed,
        },
    );
}

#[test]
fn d1_equivalence_horloge_hors_plage() {
    let mut scenario = Scenario::ns(&[100, 100], usize::MAX);
    scenario.script.push(hors_plage());
    assert_equivalence(
        &scenario,
        &Expected {
            issue: Issue::Time(String::from("ClockOutOfRange")),
            log: vec![lot(&[]), Entry::Wait(110)],
            reads: 3,
            terminal: ProgressState::Failed,
        },
    );
}

#[test]
fn d1_equivalence_debordement_d_echeancier() {
    let max = u64::MAX;
    assert_equivalence(
        &Scenario::ns(&[max - 25, max - 25, max - 5], usize::MAX),
        &Expected {
            issue: Issue::Time(String::from("ScheduleOverflow")),
            log: vec![lot(&[]), Entry::Wait(max - 15), lot(&[]), lot(&[])],
            reads: 3,
            terminal: ProgressState::Failed,
        },
    );
}

// ---------------------------------------------------------------- T7 : D3

/// Première échéance restituée par un `Progressor` sur horloge scriptée (Σ0 à `t0`).
fn premiere_echeance(t0: u64) -> Deadline {
    let (clock, _reads) = scripted_ns(&[t0, t0]);
    let mut progressor = Progressor::new(
        policy(),
        clock,
        || false,
        Inbox::<u32>::new(1),
        |_lot: &[u32]| Ok::<(), Boom>(()),
    );
    match progressor.progress().wake {
        Wake::At(deadline) => deadline,
        Wake::NoDeadline => panic!("échéance attendue"),
    }
}

/// D3 : `Deadline → Instant` par `checked_add`. Une seule origine est lue (par
/// `MonotonicClock::start`) ; la vérification est arithmétique : aucun `sleep`, aucune mesure
/// de durée réelle, aucune dépendance à la vitesse d'exécution.
#[test]
fn d3_deadline_vers_instant_par_checked_add() {
    let origin = MonotonicClock::start().origin();
    let une_heure = 3_600_000_000_000_u64;

    for (t0, attendu) in [(100, 110), (une_heure, une_heure + 10)] {
        let deadline = premiere_echeance(t0);
        assert_eq!(deadline_ns(deadline), attendu);

        let instant = origin
            .checked_add(deadline.as_duration())
            .expect("échéance représentable en `Instant`");
        assert!(instant > origin);
        assert_eq!(
            instant.duration_since(origin),
            Duration::from_nanos(attendu),
            "décalage depuis l'origine conservé"
        );
    }
}

// ---------------------------------------------------------------- T7 : D6

/// D6 : dépôts depuis un autre fil entre deux progressions. Séquence déterministe :
/// progression, dépôts du fil secondaire, jointure, progression sans rien de dû (le dépôt ne
/// rend pas une itération due), progression due. Le lot est complet, exact et ordonné.
///
/// Horloge : [100, 100] Σ0 ; [105] rien dû (échéance 110) ; [110, 110] une itération.
#[test]
fn d6_depot_depuis_un_autre_fil_entre_deux_progressions() {
    let (clock, reads) = scripted_ns(&[100, 100, 105, 110, 110]);
    let inbox = Inbox::<u32>::new(16);
    let deposit = inbox.handle();
    let lots = Rc::new(RefCell::new(Vec::<Vec<u32>>::new()));
    let lots_infra = Rc::clone(&lots);
    let mut progressor = Progressor::new(
        policy(),
        clock,
        || false,
        inbox,
        move |lot: &[u32]| {
            lots_infra.borrow_mut().push(lot.to_vec());
            Ok::<(), Boom>(())
        },
    );

    // 1. Première progression du fil principal : Σ0.
    let first = progressor.progress();
    assert_eq!(first.iterations, 1);
    match first.wake {
        Wake::At(deadline) => assert_eq!(deadline_ns(deadline), 110),
        Wake::NoDeadline => panic!("échéance attendue"),
    }

    // 2. Dépôts successifs depuis le fil secondaire, 3. jointure.
    let depositor = thread::spawn(move || {
        for value in 1..=5_u32 {
            deposit.push(value).expect("capacité suffisante");
        }
    });
    depositor.join().expect("fil déposant terminé sans panique");

    // Un dépôt ne rend pas une itération due : rien dû à 105, aucune invocation.
    let idle = progressor.progress();
    assert_eq!(idle.iterations, 0);
    assert_eq!(
        lots.borrow().len(),
        1,
        "aucune invocation sans échéance due"
    );

    // 4. Progression suivante : le lot complet, exact et ordonné.
    let second = progressor.progress();
    assert_eq!(second.iterations, 1);
    assert_eq!(second.state, ProgressState::Running);
    assert_eq!(*lots.borrow(), vec![vec![], vec![1, 2, 3, 4, 5]]);
    assert_eq!(reads.count(), 5);
}
