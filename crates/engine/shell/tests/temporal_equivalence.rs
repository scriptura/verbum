// crates/engine/shell/tests/temporal_equivalence.rs

//! ADR-S04 (unité T5c) : rejeu du contrat historique par le `Progressor` (D8) et équivalence
//! avec `run_loop`.
//!
//! Trois ensembles de tests explicitement distincts :
//! 1. **contrat historique** : `lifecycle.rs`, `acquisition.rs`, `frame_loop.rs`, etc., qui
//!    pilotent `run_loop` et ne sont pas modifiés ;
//! 2. **`Progressor`** : `temporal_progress.rs` ;
//! 3. **équivalence entre pilotes** : ce fichier.
//!
//! Dans ce fichier, « une échéance due par progression » : l'horloge scriptée place chaque
//! progression exactement sur l'échéance (`t_i = 100 + q · i`, lue deux fois), de sorte que
//! chaque progression exécute une seule itération, comme une itération de `run_loop`.
//!
//! - LC-1 à LC-4 et AQ-1, 1b, 2, 3, 5, 6, 6b, 7, 7b : scénarios historiques rejoués via le
//!   `Progressor`, avec les mêmes fermetures et les mêmes attendus littéraux ;
//! - RA-1 et RA-5 : **forme de boucle** seulement (journal, nombre d'itérations, première
//!   erreur inchangée), avec une infrastructure à fermetures ; le vrai `World`, le vrai
//!   `Scheduler` et la `Frame` ne sont pas rejoués ici (voir « Non démontré ») ;
//! - équivalence : pour chaque script d'une énumération déterministe (dépôts avant et pendant
//!   l'itération, arrêt, erreur d'infrastructure), `run_loop` et le `Progressor` donnent
//!   les mêmes lots, le même nombre d'itérations achevées et la même première erreur.
//!
//! PC-1 et PC-3 restent rattachés à `run_loop` (`pacing.rs`) : non migrés.
//!
//! **Non démontré** : RA avec le vrai `World`/`Scheduler` (`frame_loop.rs` reste sur `run_loop`) ;
//! la capacité de l'`Inbox` après prélèvement (AQ-6) : le `Progressor` possède la boîte et
//! n'expose aucun accesseur ; ce volet reste couvert par `acquisition.rs`.

#[path = "temporal_support/mod.rs"]
#[allow(dead_code, reason = "harnais partagé avec les unités de pilotes")]
mod temporal_support;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use temporal_support::{Reads, scripted_ns};
use verbum_shell::{
    Inbox, InboxFull, ProgressError, ProgressState, Progressor, TemporalPolicy, run_loop,
};

type Lots = Vec<Vec<u32>>;

/// Nombre maximal de progressions d'un scénario (garde contre une boucle sans fin).
const MAX_PROGRESSIONS: usize = 64;

// ---------------------------------------------------------------- pilotage du Progressor

fn policy(quantum: u64) -> TemporalPolicy {
    TemporalPolicy::from_nanos(quantum, 3, 5).expect("politique de test valide")
}

/// Horloge scriptée : la progression `i` lit deux fois `100 + quantum · i`.
fn horloge(quantum: u64) -> (impl FnMut() -> Duration, Reads) {
    let mut sequence = Vec::new();
    for i in 0..MAX_PROGRESSIONS as u64 {
        let t = 100 + quantum * i;
        sequence.push(t);
        sequence.push(t);
    }
    scripted_ns(&sequence)
}

/// Résultat d'un pilotage jusqu'à l'état terminal.
struct Driven<E> {
    total: u64,
    state: ProgressState,
    error: Option<ProgressError<E>>,
}

/// Appelle `progress()` jusqu'à un état terminal ; `before(i)` précède la progression `i`.
fn drive<E, C, S, I>(
    progressor: &mut Progressor<u32, E, C, S, I>,
    mut before: impl FnMut(usize),
) -> Driven<E>
where
    C: FnMut() -> Duration,
    S: FnMut() -> bool,
    I: FnMut(&[u32]) -> Result<(), E>,
{
    let mut total = 0u64;
    for index in 0..MAX_PROGRESSIONS {
        before(index);
        let report = progressor.progress();
        total += u64::from(report.iterations);
        match report.state {
            ProgressState::Running => {}
            ProgressState::Stopped => {
                return Driven {
                    total,
                    state: ProgressState::Stopped,
                    error: None,
                };
            }
            ProgressState::Failed => {
                return Driven {
                    total,
                    state: ProgressState::Failed,
                    error: progressor.take_error(),
                };
            }
        }
    }
    panic!("le pilotage ne s'est pas achevé");
}

// ---------------------------------------------------------------- LC rejoué

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Trace {
    Debut(u32),
    Fin(u32),
    Libere,
}

type Journal = Rc<RefCell<Vec<Trace>>>;

/// Valeur capturée par l'infrastructure : sa libération est journalisée.
struct Garde {
    journal: Journal,
}

impl Drop for Garde {
    fn drop(&mut self) {
        self.journal.borrow_mut().push(Trace::Libere);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ErreurTest(u32);

/// LC-1 rejoué : N itérations puis arrêt ; une seule libération, après la dernière.
#[test]
fn lc1_n_iterations_puis_arret() {
    const N: u32 = 3;
    let journal: Journal = Rc::default();
    let executions = Rc::new(Cell::new(0u32));
    let garde = Garde {
        journal: journal.clone(),
    };
    let compteur = executions.clone();
    let infrastructure = move |_: &[u32]| -> Result<(), ErreurTest> {
        let n = compteur.get();
        garde.journal.borrow_mut().push(Trace::Debut(n));
        garde.journal.borrow_mut().push(Trace::Fin(n));
        compteur.set(n + 1);
        Ok(())
    };
    let compteur = executions.clone();
    let (clock, _reads) = horloge(10);
    let mut progressor = Progressor::new(
        policy(10),
        clock,
        move || compteur.get() >= N,
        Inbox::new(0),
        infrastructure,
    );
    let driven = drive(&mut progressor, |_| {});
    drop(progressor);

    assert_eq!(driven.total, u64::from(N));
    assert_eq!(driven.state, ProgressState::Stopped);
    assert_eq!(
        *journal.borrow(),
        [
            Trace::Debut(0),
            Trace::Fin(0),
            Trace::Debut(1),
            Trace::Fin(1),
            Trace::Debut(2),
            Trace::Fin(2),
            Trace::Libere,
        ]
    );
}

/// LC-2 rejoué : arrêt demandé avant la première itération.
#[test]
fn lc2_arret_avant_la_premiere_iteration() {
    let journal: Journal = Rc::default();
    let garde = Garde {
        journal: journal.clone(),
    };
    let infrastructure = move |_: &[u32]| -> Result<(), ErreurTest> {
        garde.journal.borrow_mut().push(Trace::Debut(0));
        Ok(())
    };
    let (clock, _reads) = horloge(10);
    let mut progressor = Progressor::new(policy(10), clock, || true, Inbox::new(0), infrastructure);
    let driven = drive(&mut progressor, |_| {});
    drop(progressor);

    assert_eq!(driven.total, 0);
    assert_eq!(driven.state, ProgressState::Stopped);
    assert_eq!(*journal.borrow(), [Trace::Libere]);
}

/// LC-3 rejoué : arrêt demandé pendant une exécution ; elle s'achève, aucune suivante.
#[test]
fn lc3_arret_pendant_une_execution() {
    let journal: Journal = Rc::default();
    let arret = Rc::new(Cell::new(false));
    let garde = Garde {
        journal: journal.clone(),
    };
    let demande = arret.clone();
    let infrastructure = move |_: &[u32]| -> Result<(), ErreurTest> {
        garde.journal.borrow_mut().push(Trace::Debut(0));
        demande.set(true);
        garde.journal.borrow_mut().push(Trace::Fin(0));
        Ok(())
    };
    let (clock, _reads) = horloge(10);
    let mut progressor = Progressor::new(
        policy(10),
        clock,
        move || arret.get(),
        Inbox::new(0),
        infrastructure,
    );
    let driven = drive(&mut progressor, |_| {});
    drop(progressor);

    assert_eq!(driven.total, 1);
    assert_eq!(driven.state, ProgressState::Stopped);
    assert_eq!(
        *journal.borrow(),
        [Trace::Debut(0), Trace::Fin(0), Trace::Libere]
    );
}

/// LC-4 rejoué : première erreur rendue sans transformation, aucune réexécution.
#[test]
fn lc4_erreur_rendue_sans_transformation() {
    let journal: Journal = Rc::default();
    let executions = Rc::new(Cell::new(0u32));
    let garde = Garde {
        journal: journal.clone(),
    };
    let compteur = executions.clone();
    let infrastructure = move |_: &[u32]| -> Result<(), ErreurTest> {
        let n = compteur.get();
        garde.journal.borrow_mut().push(Trace::Debut(n));
        compteur.set(n + 1);
        if n == 2 {
            return Err(ErreurTest(42));
        }
        garde.journal.borrow_mut().push(Trace::Fin(n));
        Ok(())
    };
    let (clock, _reads) = horloge(10);
    let mut progressor =
        Progressor::new(policy(10), clock, || false, Inbox::new(0), infrastructure);
    let driven = drive(&mut progressor, |_| {});
    drop(progressor);

    assert_eq!(driven.state, ProgressState::Failed);
    assert_eq!(driven.total, 2, "les deux exécutions réussies seulement");
    match driven.error {
        Some(ProgressError::Infrastructure(erreur)) => assert_eq!(erreur, ErreurTest(42)),
        autre => panic!("cause inattendue : {autre:?}"),
    }
    assert_eq!(
        *journal.borrow(),
        [
            Trace::Debut(0),
            Trace::Fin(0),
            Trace::Debut(1),
            Trace::Fin(1),
            Trace::Debut(2),
            Trace::Libere,
        ]
    );
}

// ---------------------------------------------------------------- AQ rejoué

/// Rejoue un script de dépôts via le `Progressor` (équivalent de `rejouer` d'`acquisition.rs`).
///
/// `entre[n]` : déposé avant la progression `n`, donc avant sa coupure ; `pendant[n]` :
/// déposé par l'infrastructure pendant l'itération `n`. L'arrêt est demandé à la progression
/// `ticks`.
fn rejouer(
    inbox: Inbox<u32>,
    entre: &[&[u32]],
    pendant: &[&[u32]],
    ticks: usize,
    quantum: u64,
) -> Lots {
    let depot_cadence = inbox.handle();
    let depot_infra = inbox.handle();
    let courant = Rc::new(Cell::new(0usize));
    let executions = Cell::new(0usize);
    let lots = Rc::new(RefCell::new(Lots::new()));
    let (pour_arret, pour_lots) = (Rc::clone(&courant), Rc::clone(&lots));
    let (clock, _reads) = horloge(quantum);
    let mut progressor = Progressor::new(
        policy(quantum),
        clock,
        move || pour_arret.get() >= ticks,
        inbox,
        move |lot: &[u32]| {
            pour_lots.borrow_mut().push(lot.to_vec());
            for &e in pendant.get(executions.get()).copied().unwrap_or(&[]) {
                depot_infra.push(e).unwrap();
            }
            executions.set(executions.get() + 1);
            Ok::<(), ()>(())
        },
    );
    let driven = drive(&mut progressor, |index| {
        courant.set(index);
        for &e in entre.get(index).copied().unwrap_or(&[]) {
            depot_cadence.push(e).unwrap();
        }
    });
    assert_eq!(driven.total, ticks as u64, "la boucle doit s'achever");
    assert_eq!(driven.state, ProgressState::Stopped);
    drop(progressor);
    Rc::try_unwrap(lots).expect("seul détenteur").into_inner()
}

fn rejouer_neuf(
    capacity: usize,
    entre: &[&[u32]],
    pendant: &[&[u32]],
    ticks: usize,
    quantum: u64,
) -> Lots {
    rejouer(Inbox::new(capacity), entre, pendant, ticks, quantum)
}

#[test]
fn aq1_depot_pendant_un_tick_va_au_tick_suivant() {
    let lots = rejouer_neuf(8, &[&[1, 2], &[4]], &[&[3]], 2, 10);
    assert_eq!(lots, [vec![1, 2], vec![3, 4]]);
}

#[test]
fn aq1b_depot_avant_la_coupure_est_dans_le_lot() {
    let lots = rejouer_neuf(8, &[&[1], &[9]], &[], 2, 10);
    assert_eq!(lots, [vec![1], vec![9]]);
}

#[test]
fn aq2_ordre_d_acceptation_conserve() {
    let lots = rejouer_neuf(8, &[&[5, 3, 9, 1]], &[], 1, 10);
    assert_eq!(lots, [vec![5, 3, 9, 1]]);
}

#[test]
fn aq3_aucun_evenement_perdu_ni_duplique() {
    let entre: &[&[u32]] = &[&[1, 2], &[], &[5], &[7, 8, 9], &[], &[12]];
    let pendant: &[&[u32]] = &[&[3, 4], &[], &[6], &[], &[10, 11], &[]];
    let lots = rejouer_neuf(16, entre, pendant, 6, 10);

    assert_eq!(lots.len(), 6);
    let concatenation: Vec<u32> = lots.into_iter().flatten().collect();
    assert_eq!(concatenation, (1..=12).collect::<Vec<u32>>());
}

/// AQ-5 : la même séquence rejouée avec deux quanta très différents (10 ns et 1 ms) donne la
/// même suite de lots, non vides. Aucune assertion sur la durée réelle : l'horloge est scriptée.
#[test]
fn aq5_meme_sequence_meme_suite_de_lots() {
    let entre: &[&[u32]] = &[&[1, 2], &[3], &[4, 5, 6], &[7]];
    let pendant: &[&[u32]] = &[&[], &[], &[], &[]];

    let court = rejouer_neuf(8, entre, pendant, 4, 10);
    let long = rejouer_neuf(8, entre, pendant, 4, 1_000_000);

    assert_eq!(court, long);
    assert_eq!(court.len(), 4);
    assert!(court.iter().all(|lot| !lot.is_empty()));
}

#[test]
fn aq6_saturation_erreur_explicite() {
    const C: usize = 3;
    let inbox = Inbox::<u32>::new(C);
    let depot = inbox.handle();
    for e in [1, 2, 3] {
        assert_eq!(depot.push(e), Ok(()));
    }
    assert_eq!(depot.push(4), Err(InboxFull));
    assert_eq!(inbox.capacity(), C);

    let lots = rejouer(inbox, &[], &[], 1, 10);
    assert_eq!(lots, [vec![1, 2, 3]], "l'événement refusé n'y figure pas");
}

/// AQ-6b : la capacité est rendue après le prélèvement ; un seul `Progressor`, deux progressions.
#[test]
fn aq6b_capacite_rendue_apres_prelevement() {
    let inbox = Inbox::<u32>::new(2);
    let depot = inbox.handle();
    assert_eq!(depot.push(1), Ok(()));
    assert_eq!(depot.push(2), Ok(()));
    assert_eq!(depot.push(3), Err(InboxFull));

    let lots = Rc::new(RefCell::new(Lots::new()));
    let pour_lots = Rc::clone(&lots);
    let (clock, _reads) = horloge(10);
    let mut progressor = Progressor::new(
        policy(10),
        clock,
        || false,
        inbox,
        move |lot: &[u32]| {
            pour_lots.borrow_mut().push(lot.to_vec());
            Ok::<(), ()>(())
        },
    );
    assert_eq!(progressor.progress().iterations, 1);
    assert_eq!(*lots.borrow(), [vec![1, 2]]);

    assert_eq!(depot.push(5), Ok(()));
    assert_eq!(progressor.progress().iterations, 1);
    assert_eq!(*lots.borrow(), [vec![1, 2], vec![5]]);
}

#[test]
fn aq7_lot_complet_et_boite_videe() {
    let inbox = Inbox::<u32>::new(4);
    let depot = inbox.handle();
    for e in 1..=4 {
        assert_eq!(depot.push(e), Ok(()));
    }
    let lots = rejouer(inbox, &[], &[], 2, 10);
    assert_eq!(lots, [vec![1, 2, 3, 4], vec![]]);
}

#[test]
fn aq7b_iteration_sans_depot_recoit_un_lot_vide() {
    let lots = rejouer_neuf(4, &[], &[], 3, 10);
    assert_eq!(lots, [vec![], vec![], vec![]]);
}

// ---------------------------------------------------------------- RA : forme de boucle

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Entry {
    TickStart(usize),
    TickEnd(usize),
    Present(usize),
}

#[derive(Debug, PartialEq, Eq)]
struct TickFault;

/// Observation d'une exécution de forme RA.
#[derive(Debug, PartialEq, Eq)]
struct Execution {
    result: Result<u64, TickFault>,
    journal: Vec<Entry>,
    presented: usize,
}

/// Infrastructure de forme RA : démarrage, éventuelle faute avant la fin du tick, fin,
/// présentation. `fault_at = Some(k)` échoue à l'itération `k` (à partir de 1).
fn tick_infrastructure(
    journal: Rc<RefCell<Vec<Entry>>>,
    presented: Rc<Cell<usize>>,
    fault_at: Option<usize>,
) -> impl FnMut(&[u32]) -> Result<(), TickFault> {
    let mut k = 0usize;
    move |_lot: &[u32]| {
        k += 1;
        journal.borrow_mut().push(Entry::TickStart(k));
        if fault_at == Some(k) {
            return Err(TickFault);
        }
        journal.borrow_mut().push(Entry::TickEnd(k));
        presented.set(presented.get() + 1);
        journal.borrow_mut().push(Entry::Present(k));
        Ok(())
    }
}

/// Arrêt demandé après `iterations` observations réussies, comme `execute` de `frame_loop.rs`.
fn stop_after(iterations: usize) -> impl FnMut() -> bool {
    let stops = Cell::new(0usize);
    move || {
        stops.set(stops.get() + 1);
        stops.get() > iterations
    }
}

fn ra_via_run_loop(iterations: usize, fault_at: Option<usize>) -> Execution {
    let journal = Rc::new(RefCell::new(Vec::new()));
    let presented = Rc::new(Cell::new(0usize));
    let result = run_loop(
        || {},
        stop_after(iterations),
        &mut Inbox::<u32>::new(4),
        tick_infrastructure(Rc::clone(&journal), Rc::clone(&presented), fault_at),
    );
    Execution {
        result,
        journal: journal.borrow().clone(),
        presented: presented.get(),
    }
}

fn ra_via_progressor(iterations: usize, fault_at: Option<usize>) -> Execution {
    let journal = Rc::new(RefCell::new(Vec::new()));
    let presented = Rc::new(Cell::new(0usize));
    let (clock, _reads) = horloge(10);
    let mut progressor = Progressor::new(
        policy(10),
        clock,
        stop_after(iterations),
        Inbox::<u32>::new(4),
        tick_infrastructure(Rc::clone(&journal), Rc::clone(&presented), fault_at),
    );
    let driven = drive(&mut progressor, |_| {});
    let result = match driven.error {
        None => Ok(driven.total),
        Some(ProgressError::Infrastructure(fault)) => Err(fault),
        Some(ProgressError::Time(error)) => panic!("erreur temporelle inattendue : {error:?}"),
    };
    Execution {
        result,
        journal: journal.borrow().clone(),
        presented: presented.get(),
    }
}

/// RA-1 (forme) : `Ok(N)`, N présentations, journal `[TickStart(k), TickEnd(k), Present(k)]`.
#[test]
fn ra1_forme_n_iterations_dans_l_ordre_du_journal() {
    const N: usize = 3;
    let via_progressor = ra_via_progressor(N, None);

    assert_eq!(via_progressor.result, Ok(N as u64));
    assert_eq!(via_progressor.presented, N);
    let attendu: Vec<Entry> = (1..=N)
        .flat_map(|k| [Entry::TickStart(k), Entry::TickEnd(k), Entry::Present(k)])
        .collect();
    assert_eq!(via_progressor.journal, attendu);
    assert_eq!(via_progressor, ra_via_run_loop(N, None));
}

/// RA-5 (forme) : faute à l'itération 2 : erreur d'origine inchangée, une seule présentation,
/// ni `TickEnd(2)` ni `Present(2)` dans le journal.
#[test]
fn ra5_forme_une_erreur_de_tick_arrete_la_boucle() {
    let via_progressor = ra_via_progressor(3, Some(2));

    assert_eq!(via_progressor.result, Err(TickFault));
    assert_eq!(via_progressor.presented, 1, "un seul present, itération 1");
    assert_eq!(
        via_progressor.journal,
        [
            Entry::TickStart(1),
            Entry::TickEnd(1),
            Entry::Present(1),
            Entry::TickStart(2),
        ]
    );
    assert_eq!(via_progressor, ra_via_run_loop(3, Some(2)));
}

// ---------------------------------------------------------------- équivalence énumérée

/// Script d'observation commun aux deux pilotes.
struct Scenario {
    ticks: usize,
    /// Nombre d'événements déposés avant la coupure de l'itération `n`.
    entre: Vec<usize>,
    /// Nombre d'événements déposés par l'infrastructure pendant l'itération `n`.
    pendant: Vec<usize>,
    /// Itération (à partir de 0) dont l'exécution rend `Err(42)` après ses dépôts.
    fail_at: Option<usize>,
}

#[derive(Debug, PartialEq, Eq)]
struct Observed {
    lots: Lots,
    completed: u64,
    error: Option<u32>,
}

/// Valeurs d'événements : numérotées dans l'ordre exact de dépôt, donc uniques.
fn valeurs(scenario: &Scenario) -> (Vec<Vec<u32>>, Vec<Vec<u32>>) {
    let mut suivant = 1u32;
    let mut prendre = |n: usize| {
        let lot: Vec<u32> = (suivant..suivant + n as u32).collect();
        suivant += n as u32;
        lot
    };
    let (mut entre, mut pendant) = (Vec::new(), Vec::new());
    for n in 0..scenario.ticks {
        entre.push(prendre(scenario.entre[n]));
        pendant.push(prendre(scenario.pendant[n]));
    }
    (entre, pendant)
}

fn observe_run_loop(scenario: &Scenario) -> Observed {
    let (entre, pendant) = valeurs(scenario);
    let mut inbox = Inbox::<u32>::new(64);
    let depot_cadence = inbox.handle();
    let depot_infra = inbox.handle();
    let cadences = Cell::new(0usize);
    let executions = Cell::new(0usize);
    let reussies = Cell::new(0u64);
    let lots = RefCell::new(Lots::new());

    let result = run_loop(
        || {
            for &e in entre.get(cadences.get()).map_or(&[][..], Vec::as_slice) {
                depot_cadence.push(e).unwrap();
            }
            cadences.set(cadences.get() + 1);
        },
        || cadences.get() > scenario.ticks,
        &mut inbox,
        |lot: &[u32]| {
            lots.borrow_mut().push(lot.to_vec());
            let n = executions.get();
            for &e in pendant.get(n).map_or(&[][..], Vec::as_slice) {
                depot_infra.push(e).unwrap();
            }
            executions.set(n + 1);
            if scenario.fail_at == Some(n) {
                return Err(42u32);
            }
            reussies.set(reussies.get() + 1);
            Ok(())
        },
    );
    match result {
        Ok(n) => Observed {
            lots: lots.into_inner(),
            completed: n,
            error: None,
        },
        Err(code) => Observed {
            lots: lots.into_inner(),
            completed: reussies.get(),
            error: Some(code),
        },
    }
}

fn observe_progressor(scenario: &Scenario) -> Observed {
    let (entre, pendant) = valeurs(scenario);
    let inbox = Inbox::<u32>::new(64);
    let depot_cadence = inbox.handle();
    let depot_infra = inbox.handle();
    let courant = Rc::new(Cell::new(0usize));
    let pour_arret = Rc::clone(&courant);
    let executions = Cell::new(0usize);
    let lots = Rc::new(RefCell::new(Lots::new()));
    let pour_lots = Rc::clone(&lots);
    let ticks = scenario.ticks;
    let fail_at = scenario.fail_at;
    let (clock, _reads) = horloge(10);
    let mut progressor = Progressor::new(
        policy(10),
        clock,
        move || pour_arret.get() >= ticks,
        inbox,
        move |lot: &[u32]| {
            pour_lots.borrow_mut().push(lot.to_vec());
            let n = executions.get();
            for &e in pendant.get(n).map_or(&[][..], Vec::as_slice) {
                depot_infra.push(e).unwrap();
            }
            executions.set(n + 1);
            if fail_at == Some(n) {
                return Err(42u32);
            }
            Ok(())
        },
    );
    let driven = drive(&mut progressor, |index| {
        courant.set(index);
        for &e in entre.get(index).map_or(&[][..], Vec::as_slice) {
            depot_cadence.push(e).unwrap();
        }
    });
    drop(progressor);
    let error = match driven.error {
        None => None,
        Some(ProgressError::Infrastructure(code)) => Some(code),
        Some(ProgressError::Time(error)) => panic!("erreur temporelle inattendue : {error:?}"),
    };
    Observed {
        lots: Rc::try_unwrap(lots).expect("seul détenteur").into_inner(),
        completed: driven.total,
        error,
    }
}

/// Décode `code` en `n` chiffres en base 3 (0, 1 ou 2 événements par itération).
fn chiffres(mut code: usize, n: usize) -> Vec<usize> {
    (0..n)
        .map(|_| {
            let chiffre = code % 3;
            code /= 3;
            chiffre
        })
        .collect()
}

/// Pour chaque script d'une énumération déterministe, `run_loop` et le `Progressor` (une
/// échéance due par progression) donnent mêmes lots, mêmes itérations achevées, même première
/// erreur.
#[test]
fn equivalence_run_loop_progressor_sur_enumeration() {
    let mut cas = 0u32;
    let mut avec_erreur = 0u32;
    for ticks in 0..=4usize {
        let combinaisons = 3usize.pow(ticks as u32);
        for code_entre in 0..combinaisons {
            for code_pendant in 0..combinaisons {
                let echecs = std::iter::once(None).chain((0..ticks).map(Some));
                for fail_at in echecs {
                    let scenario = Scenario {
                        ticks,
                        entre: chiffres(code_entre, ticks),
                        pendant: chiffres(code_pendant, ticks),
                        fail_at,
                    };
                    let historique = observe_run_loop(&scenario);
                    let progressor = observe_progressor(&scenario);
                    assert_eq!(
                        progressor, historique,
                        "ticks={ticks} entre={code_entre} pendant={code_pendant} fail_at={fail_at:?}"
                    );
                    cas += 1;
                    avec_erreur += u32::from(historique.error.is_some());
                }
            }
        }
    }
    // Garde-fou : l'énumération couvre bien le domaine annoncé.
    assert_eq!(cas, 1 + 18 + 243 + 2916 + 32_805);
    assert!(avec_erreur > 0);
}
