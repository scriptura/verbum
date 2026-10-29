// crates/shell/examples/headless_frame.rs

//! Étape 4 de bout en bout, sans fenêtre ni GPU : le Shell pilote un vrai `World`, un vrai
//! `Scheduler` et un instrument de présentation headless.
//!
//! Assemblage : `FixedStepPacing` **provisoire** de 1 ms, 5 itérations, `RecordingBackend`,
//! mêmes systèmes et mêmes constantes que `crates/render/tests/pipeline.rs`. À chaque
//! itération, la fermeture d'infrastructure publie le lot, exécute `run_tick`, lit la `Frame`
//! dans le `World` puis la remet à la présentation (arbitrage A-2, forme 2 : aucun trait
//! `Backend` public ; la présentation prend `&Frame` et ne rend rien).
//!
//! Le programme n'écrit rien (ni sortie standard ni sortie d'erreur) et se termine avec le
//! code 0 lorsque `run_loop` rend `Ok(5)` et que chacune des cinq `Frame` présentées a le
//! multiensemble `{Q1, Q1, Q2}` ; un écart panique, donc sort avec un code non nul.
//!
//! L'accumulateur n'est jamais relu après `Compose` : discipline de cet assemblage, vérifiée
//! par revue ; ni les types ni les contrats du Scheduler ne l'imposent (plan C4, écart E-5).
//!
//! ```text
//! cargo run -p verbum-shell --example headless_frame
//! ```

use core::convert::Infallible;
use std::cell::Cell;
use std::time::Duration;

use verbum_ecs::{BufferFull, CommandBuffer, ResourceSlot, World};
use verbum_render::{Contribution, Contributions, Frame};
use verbum_schedule::access::{Locate, Produce, Read};
use verbum_schedule::scheduler::{Scheduler, TickError};
use verbum_schedule::system::{At, Binding, Cons, Nil, System};
use verbum_shell::{FixedStepPacing, Inbox, run_loop};

// ---------------------------------------------------------------------------
// Constantes de scénario (document d'exécution § 3.2)
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

/// Nombre d'itérations.
const ITERATIONS: usize = 5;

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

/// Erreur de l'assemblage, rendue inchangée par `run_loop`.
#[derive(Debug, PartialEq, Eq)]
enum AppError {
    Config(ConfigError),
    Tick(TickError),
    /// Erreur d'assemblage : la `Frame` est absente du `World` après le tick.
    FrameAbsent,
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

type AppWorld = World<1, (), Globals>;
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

struct AppBinding;

impl Binding for AppBinding {
    type World = AppWorld;
    type Buffer = Buffer;
    type Parts<'w> = Parts<'w>;

    fn parts<'w>(world: &'w mut AppWorld, _buffer: &'w mut Buffer) -> Parts<'w> {
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
// Systèmes (mêmes que U3 ; types distincts, aucun ne nomme un autre)
// ---------------------------------------------------------------------------

/// Phase 0 : remet l'accumulateur à l'état initial.
struct Clear;

impl System<AppBinding> for Clear {
    type Access<'w> = (ProduceAcc<'w>,);

    fn run((mut acc,): Self::Access<'_>) -> Result<(), BufferFull> {
        acc.insert(Acc::new());
        Ok(())
    }
}

/// Phase 1 : unique lecteur de l'accumulateur, unique producteur de la `Frame`.
struct Compose;

impl System<AppBinding> for Compose {
    type Access<'w> = (ReadAcc<'w>, ProduceFrame<'w>);

    fn run((acc, mut frame): Self::Access<'_>) -> Result<(), BufferFull> {
        let Some(source) = acc.get() else {
            panic!("violation d'assemblage : accumulateur absent");
        };
        frame.insert(Fr::compose(source));
        Ok(())
    }
}

/// Phase 1 : lecture seule de la `Frame`, copie dans une catégorie de témoin.
struct Witness;

impl System<AppBinding> for Witness {
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

impl System<AppBinding> for ProduceA {
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

impl System<AppBinding> for ProduceB {
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

type Systems = Cons<
    At<0, Clear>,
    Cons<At<1, Compose>, Cons<At<1, Witness>, Cons<At<1, ProduceA>, Cons<At<1, ProduceB>, Nil>>>>,
>;

// ---------------------------------------------------------------------------
// Instrument de présentation headless
// ---------------------------------------------------------------------------

/// Conserve chaque `Frame` reçue dans un enregistrement préalloué ; aucune référence au
/// `World`, au buffer, à la boîte ni à une poignée de dépôt.
struct RecordingBackend {
    frames: [Fr; ITERATIONS],
    len: usize,
}

impl RecordingBackend {
    const fn new() -> Self {
        Self {
            frames: [const { Fr::empty() }; ITERATIONS],
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

/// Clé de tri locale : `Contribution` n'implémente ni `Ord` ni `Hash`.
fn key(c: &Contribution) -> (i32, i32, u32, u32, [u8; 4]) {
    (c.x, c.y, c.width, c.height, c.color)
}

/// Multiensemble : tri par clé locale, multiplicité conservée.
fn multiset(contributions: &[Contribution]) -> Vec<Contribution> {
    let mut copy = contributions.to_vec();
    copy.sort_by_key(key);
    copy
}

fn main() {
    // Assemblage : tout est initialisé à l'état initial avant la première itération.
    let scheduler =
        Scheduler::<AppBinding, Systems, 5, 2>::new().expect("un ordonnancement valide");
    let mut world = AppWorld::new();
    let mut buffer = Buffer::new();
    {
        let globals = world.resources_mut();
        globals.acc.insert(Acc::new());
        globals.frame.insert(Fr::empty());
        globals.witness.insert(Fr::empty());
        globals.batch.insert(Batch::EMPTY);
    }

    let mut inbox = Inbox::<Ev>::new(CARRIER_CAP);
    let mut pacing = FixedStepPacing::new(Duration::from_millis(1));
    let mut backend = RecordingBackend::new();
    let stops = Cell::new(0usize);

    let completed = run_loop(
        || pacing.wait(),
        || {
            stops.set(stops.get() + 1);
            stops.get() > ITERATIONS
        },
        &mut inbox,
        |lot: &[Ev]| -> Result<(), AppError> {
            // Publication puis tick ; échec de publication : pas de tick.
            let batch = Batch::from_events(lot).map_err(AppError::Config)?;
            world.resources_mut().batch.insert(batch);
            scheduler
                .run_tick(&mut world, &mut buffer)
                .map_err(AppError::Tick)?;
            // Lecture de la `Frame` puis présentation ; l'accumulateur n'est pas relu.
            let frame = world.resources().frame.get().ok_or(AppError::FrameAbsent)?;
            backend.present(frame);
            Ok(())
        },
    )
    .expect("la boucle s'achève sans erreur");

    assert_eq!(completed, ITERATIONS as u64);
    assert_eq!(backend.received().len(), ITERATIONS);
    let expected = multiset(&[Q1, Q1, Q2]);
    for frame in backend.received() {
        assert_eq!(multiset(frame.as_slice()), expected);
    }
}
