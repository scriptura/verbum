// crates/shell/tests/frame_loop.rs

//! Acceptation de l'étape 4 sous la boucle du Shell (U4) : RA-1 à RA-5.
//!
//! Le montage ci-dessous est un **montage de test** (arbitrage A-2, forme 2) : l'assemblage lit
//! la `Frame` dans le `World` après `run_tick` et la remet à un instrument de présentation. Il
//! n'existe aucun trait `Backend` public : une présentation est une fermeture `FnMut(&Frame)`
//! qui ne rend rien. Le motif du lot (publication, `ConfigError`) est recopié de l'étape 3
//! (`tests/acceptance.rs`, non modifié) ; aucune catégorie n'est déclarée pour le lot, qui n'a
//! aucun lecteur.
//!
//! Les systèmes (`Clear`, `Compose`, `Witness`, `ProduceA`, `ProduceB`) et les constantes de
//! scénario sont ceux de `crates/render/tests/pipeline.rs` (U3), recopiés ici.
//!
//! Fermeture d'infrastructure, dans cet ordre : (1) publier le lot ; (2) journaliser
//! `TickStart(k)` ; (3) `run_tick` ; (4) journaliser `TickEnd(k)` ; (5) lire la `Frame` ;
//! (6) présenter ; (7) journaliser `Present(k)` et prendre un instantané. Toute erreur est
//! rendue par `?` et ressort inchangée de `run_loop`.
//!
//! **Ce que ces tests n'établissent pas.** L'accumulateur n'est jamais relu après `Compose`,
//! ni par la fermeture, ni par les instruments, ni par l'observateur : c'est une discipline de
//! ce montage, vérifiée par revue ; ni les types ni les contrats du Scheduler ne l'imposent
//! (plan C4, écart E-5). La séquence tick → lecture → présentation est vérifiée sur ce montage
//! seulement : aucune API publique ne l'impose. L'absence de référence des instruments au
//! `World`, au buffer ou à la boîte est vérifiée par revue : le crate peut nommer le noyau.
//! Le compte `N` de présentations est une propriété du montage, non un invariant (plan C6).

use core::convert::Infallible;
use std::cell::Cell;

use verbum_ecs::{BufferFull, CommandBuffer, ResourceSlot, World};
use verbum_render::{Contribution, Contributions, Frame};
use verbum_schedule::access::{Locate, Produce, Read};
use verbum_schedule::scheduler::{Scheduler, TickError};
use verbum_schedule::system::{At, Binding, Cons, Nil, System};
use verbum_shell::{Inbox, run_loop};

// ---------------------------------------------------------------------------
// Constantes de scénario (document d'exécution § 3.2), recopiées de U3
// ---------------------------------------------------------------------------

/// Capacité de l'accumulateur et de la `Frame`.
const CAP: usize = 4;

/// Contribution carrée partagée par `ProduceA` et `ProduceB` : doublon voulu.
const Q1: Contribution = Contribution {
    x: 0,
    y: 0,
    width: 2,
    height: 2,
    color: [255, 0, 0, 255],
};

/// Seconde contribution carrée, émise par `ProduceA` seulement.
const Q2: Contribution = Contribution {
    x: 4,
    y: 1,
    width: 3,
    height: 3,
    color: [0, 255, 0, 255],
};

/// Sortie exacte de `ProduceA` à chaque tick.
const A_SORTIE: [Contribution; 2] = [Q1, Q2];

/// Sortie exacte de `ProduceB` à chaque tick.
const B_SORTIE: [Contribution; 1] = [Q1];

/// Borne de production par tick : somme des sorties des producteurs.
const MAX_PAR_TICK: usize = A_SORTIE.len() + B_SORTIE.len();

// Ne prouve que l'arithmétique des constantes ; le lien avec le code des
// producteurs est vérifié par revue (document d'exécution § 3.2, point 4).
const _: () = assert!(MAX_PAR_TICK <= CAP);

/// Nombre d'itérations des scénarios.
const N: usize = 3;

/// Nombre maximal d'itérations que les enregistrements préalloués peuvent contenir.
const MAX_ITERATIONS: usize = 8;

// ---------------------------------------------------------------------------
// Lot (motif de l'étape 3), catégories, monde, assemblage
// ---------------------------------------------------------------------------

/// Événement du Shell : donnée simple, `Copy`, sans type de plateforme.
type Ev = u8;

/// Capacité du porteur du lot.
const CARRIER_CAP: usize = 4;

/// Ressource transitoire dédiée : tableau de capacité fixe plus longueur. « Lot vide » signifie
/// longueur zéro, jamais un emplacement vide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Batch {
    items: [Ev; CARRIER_CAP],
    len: usize,
}

impl Batch {
    const EMPTY: Self = Self {
        items: [0; CARRIER_CAP],
        len: 0,
    };

    /// Copie complète ; erreur de configuration si le lot ne tient pas, jamais de troncature.
    fn from_events(events: &[Ev]) -> Result<Self, ConfigError> {
        if events.len() > CARRIER_CAP {
            return Err(ConfigError::BatchExceedsCarrier {
                batch: events.len(),
                carrier: CARRIER_CAP,
            });
        }
        let mut batch = Self::EMPTY;
        batch.items[..events.len()].copy_from_slice(events);
        batch.len = events.len();
        Ok(batch)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ConfigError {
    BatchExceedsCarrier { batch: usize, carrier: usize },
}

/// Erreur de l'assemblage de test, rendue inchangée par `run_loop`.
#[derive(Debug, PartialEq, Eq)]
enum AppError {
    Config(ConfigError),
    Tick(TickError),
    /// Erreur d'assemblage : la `Frame` est absente du `World` après le tick.
    FrameAbsent,
    /// Erreur d'assemblage : le témoin est absent du `World` après le tick.
    WitnessAbsent,
}

type Acc = Contributions<CAP>;
type Fr = Frame<CAP>;

const ACC: u8 = 0;
const FRAME: u8 = 1;
const WITNESS: u8 = 2;

#[derive(Default)]
struct Globals {
    acc: ResourceSlot<Acc>,
    frame: ResourceSlot<Fr>,
    witness: ResourceSlot<Fr>,
    /// Lot de l'itération : aucune catégorie, aucun lecteur.
    batch: ResourceSlot<Batch>,
}

type TestWorld = World<1, (), Globals>;
type Buffer = CommandBuffer<1, Infallible, Infallible>;

struct Parts<'w> {
    acc: Option<&'w mut ResourceSlot<Acc>>,
    frame: Option<&'w mut ResourceSlot<Fr>>,
    witness: Option<&'w mut ResourceSlot<Fr>>,
}

impl<'w> Locate<'w, ACC> for Parts<'w> {
    type Store = &'w mut ResourceSlot<Acc>;
    fn take(&mut self) -> Option<Self::Store> {
        self.acc.take()
    }
}

impl<'w> Locate<'w, FRAME> for Parts<'w> {
    type Store = &'w mut ResourceSlot<Fr>;
    fn take(&mut self) -> Option<Self::Store> {
        self.frame.take()
    }
}

impl<'w> Locate<'w, WITNESS> for Parts<'w> {
    type Store = &'w mut ResourceSlot<Fr>;
    fn take(&mut self) -> Option<Self::Store> {
        self.witness.take()
    }
}

struct TestBinding;

impl Binding for TestBinding {
    type World = TestWorld;
    type Buffer = Buffer;
    type Parts<'w> = Parts<'w>;

    fn parts<'w>(world: &'w mut TestWorld, _buffer: &'w mut Buffer) -> Parts<'w> {
        let ((), globals) = world.split_mut();
        Parts {
            acc: Some(&mut globals.acc),
            frame: Some(&mut globals.frame),
            witness: Some(&mut globals.witness),
        }
    }
}

type ProduceAcc<'w> = Produce<ACC, <Parts<'w> as Locate<'w, ACC>>::Store>;
type ReadAcc<'w> = Read<ACC, <Parts<'w> as Locate<'w, ACC>>::Store>;
type ProduceFrame<'w> = Produce<FRAME, <Parts<'w> as Locate<'w, FRAME>>::Store>;
type ReadFrame<'w> = Read<FRAME, <Parts<'w> as Locate<'w, FRAME>>::Store>;
type ProduceWitness<'w> = Produce<WITNESS, <Parts<'w> as Locate<'w, WITNESS>>::Store>;

// ---------------------------------------------------------------------------
// Systèmes (recopiés de U3 ; types distincts, aucun ne nomme un autre)
// ---------------------------------------------------------------------------

/// Phase 0 : remet l'accumulateur à l'état initial.
struct Clear;

impl System<TestBinding> for Clear {
    type Access<'w> = (ProduceAcc<'w>,);

    fn run((mut acc,): Self::Access<'_>) -> Result<(), BufferFull> {
        acc.insert(Acc::new());
        Ok(())
    }
}

/// Phase 1 : unique lecteur de l'accumulateur, unique producteur de la `Frame`.
struct Compose;

impl System<TestBinding> for Compose {
    type Access<'w> = (ReadAcc<'w>, ProduceFrame<'w>);

    fn run((acc, mut frame): Self::Access<'_>) -> Result<(), BufferFull> {
        let Some(source) = acc.get() else {
            panic!("violation d'assemblage : accumulateur absent");
        };
        frame.insert(Fr::compose(source));
        Ok(())
    }
}

/// Phase 1 : lecture seule de la `Frame`, copie dans une catégorie de test.
struct Witness;

impl System<TestBinding> for Witness {
    type Access<'w> = (ReadFrame<'w>, ProduceWitness<'w>);

    fn run((frame, mut witness): Self::Access<'_>) -> Result<(), BufferFull> {
        let Some(frame) = frame.get() else {
            panic!("violation d'assemblage : frame absente");
        };
        witness.insert(frame.clone());
        Ok(())
    }
}

/// Phase 1 : producteur indépendant, émet exactement `A_SORTIE`.
struct ProduceA;

impl System<TestBinding> for ProduceA {
    type Access<'w> = (ProduceAcc<'w>,);

    fn run((mut acc,): Self::Access<'_>) -> Result<(), BufferFull> {
        let Some(acc) = acc.get_mut() else {
            panic!("violation d'assemblage : accumulateur absent");
        };
        for contribution in A_SORTIE {
            if acc.push(contribution).is_err() {
                panic!("borne de production dépassée : accumulateur plein (ProduceA)");
            }
        }
        Ok(())
    }
}

/// Phase 1 : producteur indépendant, émet exactement `B_SORTIE`.
struct ProduceB;

impl System<TestBinding> for ProduceB {
    type Access<'w> = (ProduceAcc<'w>,);

    fn run((mut acc,): Self::Access<'_>) -> Result<(), BufferFull> {
        let Some(acc) = acc.get_mut() else {
            panic!("violation d'assemblage : accumulateur absent");
        };
        for contribution in B_SORTIE {
            if acc.push(contribution).is_err() {
                panic!("borne de production dépassée : accumulateur plein (ProduceB)");
            }
        }
        Ok(())
    }
}

const SYSTEMS: usize = 5;
const PHASES: usize = 2;

type Systems = Cons<
    At<0, Clear>,
    Cons<At<1, Compose>, Cons<At<1, Witness>, Cons<At<1, ProduceA>, Cons<At<1, ProduceB>, Nil>>>>,
>;

type TestScheduler = Scheduler<TestBinding, Systems, SYSTEMS, PHASES>;

/// Monde assemblé : accumulateur, `Frame`, témoin et lot initialisés à l'état initial.
fn assembled_world() -> TestWorld {
    let mut world = TestWorld::new();
    let globals = world.resources_mut();
    globals.acc.insert(Acc::new());
    globals.frame.insert(Fr::empty());
    globals.witness.insert(Fr::empty());
    globals.batch.insert(Batch::EMPTY);
    world
}

// ---------------------------------------------------------------------------
// Instruments : journal, instantanés, présentations (tableaux préalloués)
// ---------------------------------------------------------------------------

/// Entrée du journal de la fermeture d'infrastructure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entry {
    TickStart(usize),
    TickEnd(usize),
    Present(usize),
}

const LOG_CAP: usize = 3 * MAX_ITERATIONS;

/// Journal à capacité fixe, alloué avant la boucle.
struct Journal {
    entries: [Entry; LOG_CAP],
    len: usize,
}

impl Journal {
    const fn new() -> Self {
        Self {
            entries: [Entry::TickStart(0); LOG_CAP],
            len: 0,
        }
    }

    fn record(&mut self, entry: Entry) {
        assert!(self.len < LOG_CAP, "journal plein");
        self.entries[self.len] = entry;
        self.len += 1;
    }

    fn as_slice(&self) -> &[Entry] {
        &self.entries[..self.len]
    }
}

/// Instantané pris à l'étape (7) : aucune `Contributions` (l'accumulateur n'est pas relu).
#[derive(Clone, Debug, PartialEq, Eq)]
struct Snapshot {
    /// `Frame` lue à l'étape (5), après le tick.
    read: Fr,
    /// `Frame` relue dans le `World` après la présentation.
    reread: Fr,
    /// Copie du témoin, lue dans le `World` après le tick.
    witness: Fr,
}

impl Snapshot {
    const EMPTY: Self = Self {
        read: Fr::empty(),
        reread: Fr::empty(),
        witness: Fr::empty(),
    };
}

/// Instantanés à capacité fixe, alloués avant la boucle.
struct Snapshots {
    items: [Snapshot; MAX_ITERATIONS],
    len: usize,
}

impl Snapshots {
    const fn new() -> Self {
        Self {
            items: [Snapshot::EMPTY; MAX_ITERATIONS],
            len: 0,
        }
    }

    fn record(&mut self, snapshot: Snapshot) {
        assert!(self.len < MAX_ITERATIONS, "instantanés pleins");
        self.items[self.len] = snapshot;
        self.len += 1;
    }

    fn as_slice(&self) -> &[Snapshot] {
        &self.items[..self.len]
    }
}

/// Instrument de présentation qui conserve chaque `Frame` reçue, dans un enregistrement
/// préalloué ; aucune référence au `World`, au buffer, à la boîte ni à une poignée de dépôt.
struct RecordingBackend {
    frames: [Fr; MAX_ITERATIONS],
    len: usize,
}

impl RecordingBackend {
    const fn new() -> Self {
        Self {
            frames: [const { Fr::empty() }; MAX_ITERATIONS],
            len: 0,
        }
    }

    fn present(&mut self, frame: &Fr) {
        assert!(self.len < self.frames.len(), "enregistrement plein");
        self.frames[self.len] = frame.clone();
        self.len += 1;
    }

    fn received(&self) -> &[Fr] {
        &self.frames[..self.len]
    }
}

/// Instrument de présentation qui compte les présentations et ne conserve aucune `Frame`.
struct CountingBackend {
    count: usize,
}

impl CountingBackend {
    const fn new() -> Self {
        Self { count: 0 }
    }

    fn present(&mut self, _frame: &Fr) {
        self.count += 1;
    }
}

// ---------------------------------------------------------------------------
// Exécution sous la boucle du Shell
// ---------------------------------------------------------------------------

/// État observé après le retour de `run_loop`, côté Plateforme.
struct Execution {
    world: TestWorld,
    journal: Journal,
    snapshots: Snapshots,
    result: Result<u64, AppError>,
}

/// Exécute `iterations` itérations sous `run_loop`. Cadence : fermeture vide. Arrêt : après
/// `iterations` itérations. `fault_at = Some(k)` injecte une faute à l'itération `k` : une
/// commande est émise dans le buffer avant `run_tick`, d'où `TickError::BufferNotEmpty`.
fn execute<P: FnMut(&Fr)>(iterations: usize, fault_at: Option<usize>, mut present: P) -> Execution {
    let scheduler = TestScheduler::new().expect("un modèle valide");
    let mut world = assembled_world();
    let mut buffer = Buffer::new();
    let mut inbox = Inbox::<Ev>::new(CARRIER_CAP);
    let mut journal = Journal::new();
    let mut snapshots = Snapshots::new();
    let stops = Cell::new(0usize);
    let mut k = 0usize;

    let result = run_loop(
        || {},
        || {
            stops.set(stops.get() + 1);
            stops.get() > iterations
        },
        &mut inbox,
        |lot: &[Ev]| -> Result<(), AppError> {
            k += 1;
            // (1) publication du lot ; `ConfigError` sans `run_tick` s'il ne tient pas.
            let batch = Batch::from_events(lot).map_err(AppError::Config)?;
            world.resources_mut().batch.insert(batch);
            // (2)
            journal.record(Entry::TickStart(k));
            if fault_at == Some(k) {
                buffer.spawn().expect("tampon d'un emplacement, libre");
            }
            // (3)
            scheduler
                .run_tick(&mut world, &mut buffer)
                .map_err(AppError::Tick)?;
            // (4)
            journal.record(Entry::TickEnd(k));
            // (5) l'accumulateur n'est pas relu : seule la `Frame` l'est.
            let frame = world.resources().frame.get().ok_or(AppError::FrameAbsent)?;
            let read = frame.clone();
            // (6)
            present(frame);
            // (7)
            journal.record(Entry::Present(k));
            let reread = world
                .resources()
                .frame
                .get()
                .ok_or(AppError::FrameAbsent)?
                .clone();
            let witness = world
                .resources()
                .witness
                .get()
                .ok_or(AppError::WitnessAbsent)?
                .clone();
            snapshots.record(Snapshot {
                read,
                reread,
                witness,
            });
            Ok(())
        },
    );

    Execution {
        world,
        journal,
        snapshots,
        result,
    }
}

/// Clé de tri locale au test : `Contribution` n'implémente ni `Ord` ni `Hash`.
fn key(c: &Contribution) -> (i32, i32, u32, u32, [u8; 4]) {
    (c.x, c.y, c.width, c.height, c.color)
}

/// Multiensemble : tri par clé locale, multiplicité conservée.
fn multiset(contributions: &[Contribution]) -> Vec<Contribution> {
    let mut copy = contributions.to_vec();
    copy.sort_by_key(key);
    copy
}

fn expected() -> Vec<Contribution> {
    multiset(&[Q1, Q1, Q2])
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// RA-1 : `run_loop` rend `Ok(N)` ; le Backend a reçu `N` `Frame`, chacune de multiensemble
/// `{Q1, Q1, Q2}` ; le journal est exactement `[TickStart(k), TickEnd(k), Present(k)]` pour
/// `k` de 1 à `N` (donc aucune présentation entre `TickStart` et `TickEnd`).
#[test]
fn ra1_n_frames_presentees_dans_l_ordre_du_journal() {
    let mut backend = RecordingBackend::new();

    let execution = execute(N, None, |frame| backend.present(frame));

    assert_eq!(execution.result, Ok(N as u64));
    assert_eq!(backend.received().len(), N);
    for frame in backend.received() {
        assert_eq!(frame.len(), 3);
        assert_eq!(multiset(frame.as_slice()), expected());
    }
    let expected_journal: Vec<Entry> = (1..=N)
        .flat_map(|k| [Entry::TickStart(k), Entry::TickEnd(k), Entry::Present(k)])
        .collect();
    assert_eq!(execution.journal.as_slice(), expected_journal);
}

/// Accepte toute présentation de la forme `FnMut(&Frame)` : la signature impose une `Frame`
/// lue en lecture seule et aucune valeur rendue.
fn accept<F: FnMut(&Fr)>(mut presentation: F, frame: &Fr) {
    presentation(frame);
}

/// RA-2 : test de typage. La présentation de chaque instrument est une `FnMut(&Frame<CAP>)`
/// qui ne rend rien ; ses signatures exactes sont fixées par des pointeurs de fonction. Le
/// doctest `compile_fail` du Backend est celui de RP-5 (c), dans `verbum-render` (écart E-2).
#[test]
fn ra2_la_presentation_prend_une_frame_et_ne_rend_rien() {
    let _: fn(&mut RecordingBackend, &Fr) = RecordingBackend::present;
    let _: fn(&mut CountingBackend, &Fr) = CountingBackend::present;

    let frame = Fr::empty();
    let mut recording = RecordingBackend::new();
    let mut counting = CountingBackend::new();

    accept(|f| recording.present(f), &frame);
    accept(|f| counting.present(f), &frame);

    assert_eq!(recording.received(), &[frame]);
    assert_eq!(counting.count, 1);
}

/// RA-3 : même scénario avec `RecordingBackend` puis `CountingBackend` : suites d'instantanés
/// `(Frame, témoin)` égales (l'accumulateur n'est pas relu), `Frame` composées égales, journaux
/// égaux ; les deux instruments diffèrent observablement de l'extérieur.
#[test]
fn ra3_le_resultat_ne_depend_pas_de_l_instrument() {
    let mut recording = RecordingBackend::new();
    let with_recording = execute(N, None, |frame| recording.present(frame));
    let mut counting = CountingBackend::new();
    let with_counting = execute(N, None, |frame| counting.present(frame));

    assert_eq!(with_recording.result, Ok(N as u64));
    assert_eq!(with_counting.result, Ok(N as u64));
    assert_eq!(
        with_recording.snapshots.as_slice(),
        with_counting.snapshots.as_slice()
    );
    assert_eq!(with_recording.snapshots.as_slice().len(), N);
    assert_eq!(
        with_recording.world.resources().frame.get(),
        with_counting.world.resources().frame.get()
    );
    assert_eq!(
        with_recording.journal.as_slice(),
        with_counting.journal.as_slice()
    );

    // Les instruments diffèrent de l'extérieur : l'un conserve les `Frame`, l'autre un compteur.
    assert_eq!(recording.received().len(), N);
    assert_eq!(counting.count, N);
    for (frame, snapshot) in recording
        .received()
        .iter()
        .zip(with_recording.snapshots.as_slice())
    {
        assert_eq!(frame, &snapshot.read);
    }
}

/// RA-4 : pour chaque `k`, copie du témoin = `Frame` lue après le tick = `Frame` reçue par le
/// Backend = `Frame` relue dans le `World` après `present`.
#[test]
fn ra4_temoin_lecture_presentation_et_relecture_concordent() {
    let mut backend = RecordingBackend::new();

    let execution = execute(N, None, |frame| backend.present(frame));

    assert_eq!(execution.result, Ok(N as u64));
    let snapshots = execution.snapshots.as_slice();
    assert_eq!(snapshots.len(), N);
    assert_eq!(backend.received().len(), N);
    for (k, snapshot) in snapshots.iter().enumerate() {
        let received = &backend.received()[k];
        assert_eq!(snapshot.witness, snapshot.read, "témoin = lecture, k = {k}");
        assert_eq!(&snapshot.read, received, "lecture = reçue, k = {k}");
        assert_eq!(&snapshot.reread, received, "relue = reçue, k = {k}");
    }
}

/// RA-5 : faute injectée à l'itération 2 (`buffer.spawn()` avant `run_tick`, d'où
/// `TickError::BufferNotEmpty`) : `run_loop` rend `Err` portant cette erreur, inchangée ; un
/// seul `present` a eu lieu (itération 1) ; le journal ne contient ni `TickEnd(2)` ni
/// `Present(2)` ; la `Frame` du `World` reste celle de l'itération 1.
#[test]
fn ra5_une_erreur_de_tick_arrete_la_boucle_sans_presentation() {
    let mut backend = RecordingBackend::new();

    let execution = execute(N, Some(2), |frame| backend.present(frame));

    assert_eq!(
        execution.result,
        Err(AppError::Tick(TickError::BufferNotEmpty))
    );
    assert_eq!(backend.received().len(), 1, "un seul present, itération 1");
    assert_eq!(
        execution.journal.as_slice(),
        [
            Entry::TickStart(1),
            Entry::TickEnd(1),
            Entry::Present(1),
            Entry::TickStart(2),
        ]
    );
    assert_eq!(execution.snapshots.as_slice().len(), 1);
    let frame = execution.world.resources().frame.get().expect("frame");
    assert_eq!(frame, &backend.received()[0]);
    assert_eq!(multiset(frame.as_slice()), expected());
}
