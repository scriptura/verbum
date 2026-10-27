// crates/shell/examples/console.rs

//! Programme applicatif minimal : le Shell pilote un vrai `World` et un vrai `Scheduler`.
//!
//! C'est l'unique point d'assemblage applicatif livré (arbitrage A-2) : lui seul connaît la
//! Ressource « lot » ; la boucle du Shell n'en connaît ni le type ni celui du noyau.
//!
//! Source d'événements scriptée, sans fil réel. Cadencement **provisoire** à pas fixe de
//! 1 ms. Le programme n'écrit rien et se termine avec le code 0 lorsque l'état observé est
//! l'état attendu ; un écart panique, donc sort avec un code non nul.
//!
//! ```text
//! cargo run -p verbum-shell --example console
//! ```

use core::convert::Infallible;
use std::cell::Cell;
use std::time::Duration;

use verbum_ecs::{BufferFull, CommandBuffer, ResourceSlot, World};
use verbum_schedule::access::{Locate, Read, Write};
use verbum_schedule::scheduler::{Scheduler, TickError};
use verbum_schedule::system::{At, Binding, Cons, Nil, System};
use verbum_shell::{FixedStepPacing, Inbox, run_loop};

/// Événement : donnée simple, `Copy`, sans type de plateforme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Ev(u32);

/// Capacité du porteur du lot. Elle est au moins celle de la boîte de réception.
const CARRIER_CAP: usize = 4;

/// Ressource transitoire dédiée : le lot de l'itération, tableau de capacité fixe plus
/// longueur. « Lot vide » signifie longueur zéro, jamais un emplacement vide.
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

    /// Copie complète du lot ; erreur de configuration s'il ne tient pas (jamais de troncature).
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
}

/// État persistant : somme des valeurs reçues.
#[derive(Debug, PartialEq, Eq)]
struct Total {
    sum: u32,
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
const TOTAL: u8 = 1;

#[derive(Default)]
struct Globals {
    batch: ResourceSlot<Batch>,
    total: ResourceSlot<Total>,
}

type AppWorld = World<1, (), Globals>;
type AppBuffer = CommandBuffer<1, Infallible, Infallible>;

struct Parts<'w> {
    batch: Option<&'w mut ResourceSlot<Batch>>,
    total: Option<&'w mut ResourceSlot<Total>>,
}

impl<'w> Locate<'w, BATCH> for Parts<'w> {
    type Store = &'w mut ResourceSlot<Batch>;
    fn take(&mut self) -> Option<Self::Store> {
        self.batch.take()
    }
}

impl<'w> Locate<'w, TOTAL> for Parts<'w> {
    type Store = &'w mut ResourceSlot<Total>;
    fn take(&mut self) -> Option<Self::Store> {
        self.total.take()
    }
}

struct AppBinding;

impl Binding for AppBinding {
    type World = AppWorld;
    type Buffer = AppBuffer;
    type Parts<'w> = Parts<'w>;

    fn parts<'w>(world: &'w mut AppWorld, _buffer: &'w mut AppBuffer) -> Parts<'w> {
        let ((), globals) = world.split_mut();
        Parts {
            batch: Some(&mut globals.batch),
            total: Some(&mut globals.total),
        }
    }
}

type ReadBatch<'w> = Read<BATCH, <Parts<'w> as Locate<'w, BATCH>>::Store>;
type WriteTotal<'w> = Write<TOTAL, <Parts<'w> as Locate<'w, TOTAL>>::Store>;

/// Lit le lot (lecture seule) et accumule les valeurs dans l'état persistant.
struct Accumulate;

impl System<AppBinding> for Accumulate {
    type Access<'w> = (ReadBatch<'w>, WriteTotal<'w>);

    fn run((batch, mut total): Self::Access<'_>) -> Result<(), BufferFull> {
        if let (Some(batch), Some(total)) = (batch.get(), total.get_mut()) {
            for event in batch.events() {
                total.sum += event.0;
            }
        }
        Ok(())
    }
}

type Systems = Cons<At<0, Accumulate>, Nil>;

const TICKS: usize = 5;

/// Valeurs déposées avant la coupure de chaque tick (lot vide au tick 0, puis non vides).
const SCRIPT: [&[u32]; TICKS] = [&[], &[1, 2], &[3], &[], &[4, 5, 6]];

fn main() {
    // Assemblage : le porteur est initialisé avec le lot vide avant la première itération.
    let scheduler =
        Scheduler::<AppBinding, Systems, 1, 1>::new().expect("un ordonnancement valide");
    let mut world = AppWorld::new();
    let mut buffer = AppBuffer::new();
    {
        let globals = world.resources_mut();
        globals.batch.insert(Batch::EMPTY);
        globals.total.insert(Total { sum: 0 });
    }

    let mut inbox = Inbox::<Ev>::new(CARRIER_CAP);
    let deposit = inbox.handle();
    let mut pacing = FixedStepPacing::new(Duration::from_millis(1));
    let cadences = Cell::new(0usize);

    let completed = run_loop(
        || {
            pacing.wait();
            // Source scriptée : dépôts avant la coupure du tick courant.
            for &value in SCRIPT.get(cadences.get()).copied().unwrap_or(&[]) {
                deposit.push(Ev(value)).expect("boîte non saturée");
            }
            cadences.set(cadences.get() + 1);
        },
        || cadences.get() > TICKS,
        &mut inbox,
        |lot: &[Ev]| {
            // Publication puis tick, en une seule séquence ; échec de publication : pas de tick.
            let batch = Batch::from_events(lot).map_err(AppError::Config)?;
            world.resources_mut().batch.insert(batch);
            scheduler
                .run_tick(&mut world, &mut buffer)
                .map_err(AppError::Tick)
        },
    )
    .expect("la boucle s'achève sans erreur");

    assert_eq!(completed, TICKS as u64);
    let total = world.resources().total.get().map(|total| total.sum);
    assert_eq!(total, Some(1 + 2 + 3 + 4 + 5 + 6));
}
