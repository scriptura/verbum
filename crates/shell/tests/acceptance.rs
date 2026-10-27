// crates/shell/tests/acceptance.rs

//! Acceptation de bout en bout : vrai `World`, vrai `Scheduler`, vrai buffer (AC-1 à AC-7).
//!
//! Le câblage ci-dessous est un **montage de test** (arbitrage A-2), non un second point
//! d'assemblage applicatif : l'assemblage livré est `examples/console.rs`.
//!
//! L'adaptateur d'application est écrit dans la fermeture passée à `run_loop` : il publie le
//! lot dans la Ressource dédiée, puis invoque `run_tick`, en une seule séquence.
//!
//! **Instrument de test (dépôt pendant un tick).** `SimulatedArrival` est un système de test
//! qui dépose un événement dans la boîte pendant le tick, par un handle externe (variable
//! locale au fil de test). C'est strictement un instrument : ni comportement de production,
//! ni API publique, ni fil réel, ni `unsafe`.
//!
//! **Portée.** Ces tests ne démontrent aucune sûreté concurrente.

use core::convert::Infallible;
use std::cell::{Cell, RefCell};

use verbum_ecs::{BufferFull, CommandBuffer, ResourceSlot, World};
use verbum_schedule::access::{COMMANDS_BINDING, Commands, Locate, Produce, Read, Write};
use verbum_schedule::contract::{Contract, StructuralEffects};
use verbum_schedule::data::{DataId, DataSet};
use verbum_schedule::scheduler::{Scheduler, TickError};
use verbum_schedule::system::{At, Binding, Cons, Nil, System, SystemList};
use verbum_shell::{DepositHandle, Inbox, run_loop};

// ---------------------------------------------------------------------------
// Données applicatives
// ---------------------------------------------------------------------------

/// Événement : donnée simple, `Copy`, sans type de plateforme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Ev(u32);

/// Capacité du porteur du lot.
const CARRIER_CAP: usize = 4;
/// Nombre maximal de ticks journalisés par un lecteur.
const JOURNAL_CAP: usize = 8;

/// Ressource transitoire dédiée : le lot, tableau de capacité fixe plus longueur. « Lot vide »
/// signifie longueur zéro, jamais un emplacement vide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Batch {
    items: [Ev; CARRIER_CAP],
    len: usize,
}

impl Batch {
    const EMPTY: Self = Self {
        items: [Ev(0); CARRIER_CAP],
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

    fn events(&self) -> &[Ev] {
        &self.items[..self.len]
    }

    fn values(&self) -> Vec<u32> {
        self.events().iter().map(|event| event.0).collect()
    }
}

/// Témoin d'un lecteur : le lot vu à chaque tick, et le nombre de lectures `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Journal {
    seen: [Batch; JOURNAL_CAP],
    count: usize,
    absent: usize,
}

impl Journal {
    const fn new() -> Self {
        Self {
            seen: [Batch::EMPTY; JOURNAL_CAP],
            count: 0,
            absent: 0,
        }
    }

    fn record(&mut self, batch: Option<&Batch>) {
        match batch {
            Some(batch) => {
                self.seen[self.count] = *batch;
                self.count += 1;
            }
            None => self.absent += 1,
        }
    }

    fn history(&self) -> Vec<Vec<u32>> {
        self.seen[..self.count].iter().map(Batch::values).collect()
    }
}

/// État persistant : somme des valeurs reçues et nombre d'événements.
#[derive(Debug, PartialEq, Eq)]
struct Total {
    sum: u32,
    events: u32,
}

#[derive(Debug, PartialEq, Eq)]
enum ConfigError {
    BatchExceedsCarrier { batch: usize, carrier: usize },
}

#[derive(Debug, PartialEq, Eq)]
enum AppError {
    Config(ConfigError),
    Tick(TickError),
}

const BATCH: u8 = 0;
const FIRST: u8 = 1;
const SECOND: u8 = 2;
const TOTAL: u8 = 3;

#[derive(Default)]
struct Globals {
    batch: ResourceSlot<Batch>,
    first: ResourceSlot<Journal>,
    second: ResourceSlot<Journal>,
    total: ResourceSlot<Total>,
}

type AppWorld = World<1, (), Globals>;
/// Un seul emplacement : un système qui émet deux commandes déborde.
type AppBuffer = CommandBuffer<1, Infallible, Infallible>;
type Slot<'w, T> = &'w mut ResourceSlot<T>;

struct Parts<'w> {
    batch: Option<Slot<'w, Batch>>,
    first: Option<Slot<'w, Journal>>,
    second: Option<Slot<'w, Journal>>,
    total: Option<Slot<'w, Total>>,
    commands: Option<&'w mut AppBuffer>,
}

macro_rules! locate_slot {
    ($($id:ident => $field:ident: $store:ty;)*) => {$(
        impl<'w> Locate<'w, $id> for Parts<'w> {
            type Store = $store;
            fn take(&mut self) -> Option<Self::Store> {
                self.$field.take()
            }
        }
    )*};
}

locate_slot! {
    BATCH => batch: Slot<'w, Batch>;
    FIRST => first: Slot<'w, Journal>;
    SECOND => second: Slot<'w, Journal>;
    TOTAL => total: Slot<'w, Total>;
    COMMANDS_BINDING => commands: &'w mut AppBuffer;
}

struct AppBinding;

impl Binding for AppBinding {
    type World = AppWorld;
    type Buffer = AppBuffer;
    type Parts<'w> = Parts<'w>;

    fn parts<'w>(world: &'w mut AppWorld, buffer: &'w mut AppBuffer) -> Parts<'w> {
        let ((), globals) = world.split_mut();
        let Globals {
            batch,
            first,
            second,
            total,
        } = globals;
        Parts {
            batch: Some(batch),
            first: Some(first),
            second: Some(second),
            total: Some(total),
            commands: Some(buffer),
        }
    }
}

type ReadOf<'w, const ID: u8> = Read<ID, <Parts<'w> as Locate<'w, ID>>::Store>;
type WriteOf<'w, const ID: u8> = Write<ID, <Parts<'w> as Locate<'w, ID>>::Store>;
type ProduceOf<'w, const ID: u8> = Produce<ID, <Parts<'w> as Locate<'w, ID>>::Store>;
type SpawnOf<'w> =
    Commands<<Parts<'w> as Locate<'w, COMMANDS_BINDING>>::Store, true, false, false, false>;

// ---------------------------------------------------------------------------
// Systèmes
// ---------------------------------------------------------------------------

/// Lecteur de la phase 0 : journalise le lot observé.
struct ReadFirst;

impl System<AppBinding> for ReadFirst {
    type Access<'w> = (ReadOf<'w, BATCH>, WriteOf<'w, FIRST>);

    fn run((batch, mut journal): Self::Access<'_>) -> Result<(), BufferFull> {
        if let Some(journal) = journal.get_mut() {
            journal.record(batch.get());
        }
        Ok(())
    }
}

/// Lecteur de la phase 1 : journalise le lot observé.
struct ReadSecond;

impl System<AppBinding> for ReadSecond {
    type Access<'w> = (ReadOf<'w, BATCH>, WriteOf<'w, SECOND>);

    fn run((batch, mut journal): Self::Access<'_>) -> Result<(), BufferFull> {
        if let Some(journal) = journal.get_mut() {
            journal.record(batch.get());
        }
        Ok(())
    }
}

/// Phase 1 : accumule les valeurs du lot dans l'état persistant.
struct Accumulate;

impl System<AppBinding> for Accumulate {
    type Access<'w> = (ReadOf<'w, BATCH>, WriteOf<'w, TOTAL>);

    fn run((batch, mut total): Self::Access<'_>) -> Result<(), BufferFull> {
        if let (Some(batch), Some(total)) = (batch.get(), total.get_mut()) {
            for event in batch.events() {
                total.sum += event.0;
                total.events += 1;
            }
        }
        Ok(())
    }
}

/// Instrument de test : dépôt simulé pendant le tick, par un handle externe.
struct SimulatedArrival;

struct Arrival {
    handle: DepositHandle<Ev>,
    next: u32,
    remaining: u32,
}

thread_local! {
    static ARRIVAL: RefCell<Option<Arrival>> = const { RefCell::new(None) };
}

impl System<AppBinding> for SimulatedArrival {
    type Access<'w> = ();

    fn run((): Self::Access<'_>) -> Result<(), BufferFull> {
        ARRIVAL.with(|arrival| {
            let mut guard = arrival.borrow_mut();
            let Some(arrival) = guard.as_mut() else {
                return;
            };
            if arrival.remaining == 0 {
                return;
            }
            arrival
                .handle
                .push(Ev(arrival.next))
                .expect("boîte non saturée");
            arrival.next += 1;
            arrival.remaining -= 1;
        });
        Ok(())
    }
}

/// Émet deux créations dans un buffer d'un emplacement : `BufferFull` au second.
struct Overflow;

impl System<AppBinding> for Overflow {
    type Access<'w> = (SpawnOf<'w>,);

    fn run((mut commands,): Self::Access<'_>) -> Result<(), BufferFull> {
        commands.spawn()?;
        commands.spawn()
    }
}

/// Système fautif (liste auxiliaire d'AC-6 seulement) : déclare l'écriture du lot.
struct RogueWriter;

impl System<AppBinding> for RogueWriter {
    type Access<'w> = (WriteOf<'w, BATCH>,);

    fn run(_: Self::Access<'_>) -> Result<(), BufferFull> {
        Ok(())
    }
}

/// Système fautif (liste auxiliaire d'AC-6 seulement) : déclare la production du lot.
struct RogueProducer;

impl System<AppBinding> for RogueProducer {
    type Access<'w> = (ProduceOf<'w, BATCH>,);

    fn run(_: Self::Access<'_>) -> Result<(), BufferFull> {
        Ok(())
    }
}

type AppSystems = Cons<
    At<0, SimulatedArrival>,
    Cons<At<0, ReadFirst>, Cons<At<1, ReadSecond>, Cons<At<1, Accumulate>, Nil>>>,
>;
const APP_SYSTEMS: usize = 4;
const APP_PHASES: usize = 2;
type AppScheduler = Scheduler<AppBinding, AppSystems, APP_SYSTEMS, APP_PHASES>;

// ---------------------------------------------------------------------------
// Assemblage et exécution
// ---------------------------------------------------------------------------

/// Assemblage : le porteur est initialisé avec le lot vide, avant toute publication.
fn assemble() -> (AppWorld, AppBuffer) {
    let mut world = AppWorld::new();
    let globals = world.resources_mut();
    globals.batch.insert(Batch::EMPTY);
    globals.first.insert(Journal::new());
    globals.second.insert(Journal::new());
    globals.total.insert(Total { sum: 0, events: 0 });
    (world, AppBuffer::new())
}

/// Publication : remplace intégralement le lot du porteur. Erreur de configuration (sans
/// effet sur le porteur) si le lot ne tient pas.
fn publish(world: &mut AppWorld, lot: &[Ev]) -> Result<(), ConfigError> {
    let batch = Batch::from_events(lot)?;
    world.resources_mut().batch.insert(batch);
    Ok(())
}

/// Boucle du Shell sur `ticks` itérations. `before_cut(n)` simule les dépôts effectués
/// entre la fin du tick `n − 1` et la coupure du tick `n`.
fn run_app<L: SystemList<AppBinding>, const N: usize, const P: usize>(
    scheduler: &Scheduler<AppBinding, L, N, P>,
    world: &mut AppWorld,
    buffer: &mut AppBuffer,
    inbox: &mut Inbox<Ev>,
    ticks: usize,
    mut before_cut: impl FnMut(usize),
) -> Result<u64, AppError> {
    let cadences = Cell::new(0usize);
    run_loop(
        || {
            before_cut(cadences.get());
            cadences.set(cadences.get() + 1);
        },
        || cadences.get() > ticks,
        inbox,
        |lot: &[Ev]| {
            publish(world, lot).map_err(AppError::Config)?;
            scheduler.run_tick(world, buffer).map_err(AppError::Tick)?;
            assert!(buffer.is_empty(), "buffer vide après un tick réussi");
            Ok(())
        },
    )
}

/// Dépôts scriptés avant la coupure de chaque tick (tick 0 vide).
const ENTRE: [&[u32]; 6] = [&[], &[4], &[5, 6, 7], &[], &[], &[]];
/// Valeur du premier événement déposé pendant un tick ; le suivant vaut +1.
const ARRIVAL_BASE: u32 = 100;

fn install_arrival(handle: DepositHandle<Ev>, remaining: u32) {
    ARRIVAL.with(|arrival| {
        *arrival.borrow_mut() = Some(Arrival {
            handle,
            next: ARRIVAL_BASE,
            remaining,
        });
    });
}

struct Outcome {
    world: AppWorld,
    /// Contenu restant de la boîte après la boucle (événements déposés trop tard).
    left_in_inbox: Vec<u32>,
    result: Result<u64, AppError>,
}

/// Scénario commun : boîte de capacité `CARRIER_CAP`, dépôts scriptés avant coupure
/// (`entre`), et un événement déposé pendant chacun des `arrivals` premiers ticks.
fn scenario(ticks: usize, entre: &[&[u32]], arrivals: u32) -> Outcome {
    let scheduler = AppScheduler::new().expect("un ordonnancement valide");
    let (mut world, mut buffer) = assemble();
    let mut inbox = Inbox::<Ev>::new(CARRIER_CAP);
    let deposit = inbox.handle();
    install_arrival(inbox.handle(), arrivals);

    let result = run_app(
        &scheduler,
        &mut world,
        &mut buffer,
        &mut inbox,
        ticks,
        |tick| {
            for &value in entre.get(tick).copied().unwrap_or(&[]) {
                deposit.push(Ev(value)).expect("boîte non saturée");
            }
        },
    );
    let left_in_inbox = inbox.cut().iter().map(|event| event.0).collect();
    Outcome {
        world,
        left_in_inbox,
        result,
    }
}

fn journals(world: &AppWorld) -> (Journal, Journal) {
    let globals = world.resources();
    (
        *globals.first.get().expect("journal du lecteur 1"),
        *globals.second.get().expect("journal du lecteur 2"),
    )
}

// ---------------------------------------------------------------------------
// Tests d'acceptation
// ---------------------------------------------------------------------------

/// AC-1 : plusieurs ticks, source scriptée ; l'état final égale la valeur attendue, avec le
/// décalage d'un tick pour les événements déposés pendant un tick.
///
/// Ticks 0 à 3. Lots : t0 `[]`, t1 `[100, 4]`, t2 `[101, 5, 6, 7]`, t3 `[102]`. L'événement 103,
/// déposé pendant le tick 3, n'est pas observé : il reste en attente dans la boîte.
#[test]
fn ac1_etat_final_avec_decalage_d_un_tick() {
    let outcome = scenario(4, &ENTRE, 4);

    assert_eq!(outcome.result, Ok(4));
    let total = outcome
        .world
        .resources()
        .total
        .get()
        .expect("état persistant");
    assert_eq!(total.sum, 100 + 4 + 101 + 5 + 6 + 7 + 102);
    assert_eq!(total.events, 7);
    assert_eq!(outcome.left_in_inbox, [103]);
}

/// AC-2 : `Scheduler::model()` égal avant et après N ticks ; buffer vide après chaque tick
/// réussi (assertion dans `run_app`) ; le Shell n'a fourni que le lot (la boucle ne reçoit ni
/// `World`, ni buffer, ni `Scheduler`).
#[test]
fn ac2_modele_inchange_et_buffer_vide() {
    let scheduler = AppScheduler::new().expect("un ordonnancement valide");
    let before = *scheduler.model();
    let (mut world, mut buffer) = assemble();
    let mut inbox = Inbox::<Ev>::new(CARRIER_CAP);

    let result = run_app(&scheduler, &mut world, &mut buffer, &mut inbox, 5, |_| {});

    assert_eq!(result, Ok(5));
    assert_eq!(*scheduler.model(), before);
    assert!(buffer.is_empty());
}

/// AC-4 : une erreur de tick réelle arrête la boucle ; l'erreur d'origine est rendue
/// inchangée ; le buffer n'est pas vidé ; un nouvel appel sans `clear` rend `BufferNotEmpty`.
#[test]
fn ac4_erreur_de_tick_rendue_inchangee() {
    type FailingSystems = Cons<At<0, Overflow>, Nil>;
    let scheduler = Scheduler::<AppBinding, FailingSystems, 1, 1>::new().expect("valide");
    let (mut world, mut buffer) = assemble();
    let mut inbox = Inbox::<Ev>::new(CARRIER_CAP);
    let cadences = Cell::new(0usize);

    let result = run_app(&scheduler, &mut world, &mut buffer, &mut inbox, 10, |_| {
        cadences.set(cadences.get() + 1);
    });

    assert_eq!(result, Err(AppError::Tick(TickError::Buffer(BufferFull))));
    assert_eq!(cadences.get(), 1, "aucune réexécution après l'erreur");
    assert_eq!(buffer.len(), 1, "le buffer garde ses commandes");
    assert_eq!(
        scheduler.run_tick(&mut world, &mut buffer),
        Err(TickError::BufferNotEmpty)
    );
    assert_eq!(buffer.len(), 1);
}

/// Dépôts scriptés d'AC-5 : le tick 0 est un tick à lot non vide.
const ENTRE_AC5: [&[u32]; 6] = [&[1, 2, 3], &[4], &[5, 6, 7], &[], &[], &[]];

/// AC-5 : lecture sans producteur interne, initialisation, renouvellement. Deux phases, un
/// lecteur du lot dans chacune, aucun producteur interne de la catégorie du lot.
///
/// Séquence publiée : t0 `[1, 2, 3]`, t1 `[100, 4]`, t2 `[101, 5, 6, 7]`, t3 `[102]`,
/// t4 `[103]`, t5 `[]`. Les valeurs 100 et suivantes sont déposées pendant le tick précédent.
#[test]
fn ac5_initialisation_lecture_renouvellement() {
    // (0) avant la boucle, à la fin de l'assemblage et avant toute publication : le porteur
    // est présent et contient un lot vide.
    let (world, _) = assemble();
    let carrier = world.resources().batch.get();
    assert_eq!(
        carrier.map(|batch| batch.len),
        Some(0),
        "(0) porteur initialisé, lot vide"
    );

    let outcome = scenario(6, &ENTRE_AC5, 4);
    assert_eq!(outcome.result, Ok(6));
    let (first, second) = journals(&outcome.world);

    // (i) aucune lecture `None`, première itération comprise.
    assert_eq!(
        (first.absent, second.absent),
        (0, 0),
        "(i) aucun état absent"
    );
    assert_eq!((first.count, second.count), (6, 6));

    // (i-bis) le lot observé au tick 0 est exactement le lot non vide attendu.
    assert_eq!(first.history()[0], [1, 2, 3], "(i-bis) lot du tick 0");

    // (ii) les lecteurs des deux phases observent le même lot dans un tick, y compris avec
    // un dépôt simulé pendant ce tick.
    assert_eq!(
        first.history(),
        second.history(),
        "(ii) même lot dans les deux phases"
    );

    // (iii) le lot vide suivant un lot non vide est observé vide.
    let history = first.history();
    assert_eq!(history[4], [103], "(iii) lot non vide");
    assert_eq!(
        history[5],
        Vec::<u32>::new(),
        "(iii) lot vide : rien ne survit"
    );

    // (iv) un événement déposé pendant le tick N n'est observé qu'au tick N + 1.
    assert!(!history[0].contains(&100), "(iv) 100 absent du tick 0");
    assert_eq!(
        history[1],
        [100, 4],
        "(iv) 100 observé au tick 1, en premier"
    );
    assert_eq!(history[2], [101, 5, 6, 7], "(iv) 101 observé au tick 2");
}

/// Contrôle de composition : aucun système de la liste ne déclare l'écriture ni la production
/// de la catégorie du lot, et au moins un la lit.
#[derive(Debug, PartialEq, Eq)]
enum CompositionError {
    Writes(usize),
    Produces(usize),
    NoReader,
}

fn check_composition<L: SystemList<AppBinding>, const N: usize>() -> Result<(), CompositionError> {
    assert_eq!(L::LEN, N, "N décrit la liste");
    let filler = Contract::new(
        DataSet::EMPTY,
        DataSet::EMPTY,
        DataSet::EMPTY,
        StructuralEffects::NONE,
    );
    let mut contracts = [(0usize, filler); N];
    L::fill(&mut contracts, 0);

    let batch = DataId::new(BATCH).expect("catégorie dans la capacité");
    let mut readers = 0;
    for (index, (_, contract)) in contracts.iter().enumerate() {
        if contract.writes().contains(batch) {
            return Err(CompositionError::Writes(index));
        }
        if contract.produces().contains(batch) {
            return Err(CompositionError::Produces(index));
        }
        if contract.reads().contains(batch) {
            readers += 1;
        }
    }
    if readers == 0 {
        return Err(CompositionError::NoReader);
    }
    Ok(())
}

/// AC-6 : contrôle des contrats des systèmes assemblés. Contrôle de composition : les types
/// actuels n'empêchent pas un système qui écrit le lot (F5) ; il ne voit pas un système qui
/// atteindrait le lot par un handle externe.
#[test]
fn ac6_controle_des_contrats() {
    // La liste assemblée est conforme, et le contrôle n'est pas vide (au moins un lecteur).
    assert_eq!(check_composition::<AppSystems, APP_SYSTEMS>(), Ok(()));

    // Le contrôle échoue sur une liste contenant un système qui écrit la catégorie…
    type WithWriter = Cons<At<0, ReadFirst>, Cons<At<0, RogueWriter>, Nil>>;
    assert_eq!(
        check_composition::<WithWriter, 2>(),
        Err(CompositionError::Writes(1))
    );

    // …ou qui la produit…
    type WithProducer = Cons<At<0, ReadFirst>, Cons<At<0, RogueProducer>, Nil>>;
    assert_eq!(
        check_composition::<WithProducer, 2>(),
        Err(CompositionError::Produces(1))
    );

    // …et sur une liste sans lecteur (contrôle vide).
    type WithoutReader = Cons<At<0, SimulatedArrival>, Nil>;
    assert_eq!(
        check_composition::<WithoutReader, 1>(),
        Err(CompositionError::NoReader)
    );
}

/// AC-7 : publication complète et échec de publication.
#[test]
fn ac7_publication_complete_et_echec_de_publication() {
    // (a) Un lot de la taille du porteur est publié intégralement et lu en entier. Il est
    // déposé au tick 1 ; le tick 0 est vide.
    let full: [&[u32]; 2] = [&[], &[1, 2, 3, 4]];
    let outcome = scenario(2, &full, 0);
    assert_eq!(outcome.result, Ok(2));
    let (first, second) = journals(&outcome.world);
    assert_eq!(
        first.history()[1],
        [1, 2, 3, 4],
        "lot complet, sans troncature"
    );
    assert_eq!(second.history()[1], [1, 2, 3, 4]);
    assert_eq!(
        outcome.world.resources().total.get().map(|total| total.sum),
        Some(10)
    );
    assert!(
        outcome.left_in_inbox.is_empty(),
        "la boîte est vide après le prélèvement"
    );

    // (b) Porteur plus petit que le lot : la fermeture rend une erreur de configuration,
    // `run_tick` n'est pas invoqué, la boucle s'arrête avec l'erreur d'origine.
    let scheduler = AppScheduler::new().expect("un ordonnancement valide");
    let (mut world, mut buffer) = assemble();
    let mut inbox = Inbox::<Ev>::new(8); // capacité de la boîte > capacité du porteur
    let deposit = inbox.handle();
    // Le lot trop grand est déposé au tick 1 ; le tick 0, vide, s'exécute normalement.
    let too_big: [&[u32]; 2] = [&[], &[1, 2, 3, 4, 5]];

    let result = run_app(&scheduler, &mut world, &mut buffer, &mut inbox, 3, |tick| {
        for &value in too_big.get(tick).copied().unwrap_or(&[]) {
            deposit.push(Ev(value)).expect("boîte non saturée");
        }
    });

    assert_eq!(
        result,
        Err(AppError::Config(ConfigError::BatchExceedsCarrier {
            batch: 5,
            carrier: CARRIER_CAP
        }))
    );
    // Témoin de non-exécution : seul le tick 0 (vide) a tourné ; le tick 1 n'a invoqué aucun
    // système, l'état du `World` est inchangé.
    let (first, second) = journals(&world);
    assert_eq!(
        (first.count, second.count),
        (1, 1),
        "run_tick non invoqué au tick 1"
    );
    assert_eq!(
        world.resources().total.get().map(|total| total.events),
        Some(0)
    );
    assert_eq!(
        world.resources().batch.get().map(|batch| batch.len),
        Some(0),
        "porteur inchangé"
    );
}
