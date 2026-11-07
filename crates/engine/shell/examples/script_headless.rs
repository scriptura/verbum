// crates/engine/shell/examples/script_headless.rs

//! Exemple headless de l'étape 6 : compile un script par la Forge, le charge, l'exécute sous
//! `run_loop` (`FixedStepPacing` de 1 ms, 12 itérations) et imprime les émissions de chaque
//! tick. L'assemblage minimal est celui de `tests/script_support/mod.rs`, recopié (un exemple
//! ne peut pas inclure un module de `tests/` ; précédent `level_headless.rs`).
//!
//! Les 12 transcripts sont vérifiés en interne (panique sinon) : code de sortie 0, aucun
//! backend graphique. Le format imprimé est libre et provisoire.

use core::cell::Cell;
use core::marker::PhantomData;
use std::time::Duration;

use verbum_assets::{Asset, Format, SlotIndex, load};
use verbum_ecs::{
    BufferFull, CommandBuffer, ComponentSet, EntityId, InsertError, ResourceSlot, SparseSet,
    ValuesMut, World,
};
use verbum_forge::{AssetKind, AssetSource, compile_assets};
use verbum_schedule::access::{Locate, Produce, Read, Write};
use verbum_schedule::scheduler::{Scheduler, TickError};
use verbum_schedule::system::{At, Binding, Cons, Nil, System};
use verbum_script::{Emissions, ScriptState, ScriptTable, emissions_bound_holds, step};
use verbum_shell::{FixedStepPacing, Inbox, run_loop};

// ---------------------------------------------------------------------------
// Constantes de scène
// ---------------------------------------------------------------------------

const ENTITIES: usize = 4;
const SPARSE: usize = 4;
const STATE_DENSE: usize = 4;
const COMMANDS: usize = 4;
const EMISSIONS_CAP: usize = 64;
const SCRIPT_SLOTS: usize = 6;
const SYSTEMS: usize = 4;
const PHASES: usize = 2;
const SLOT: u32 = 4;
/// Nombre d'itérations.
const N: usize = 12;

const STATE: u8 = 0;
const SCRIPTS: u8 = 1;
const EMISSIONS: u8 = 2;
const TRANSCRIPT: u8 = 3;
const BEAT: u8 = 4;

/// Borne des émissions : chaque instance émet au plus `SCRIPT_MAX_STEPS_PER_TICK` fois.
const _: () = assert!(emissions_bound_holds(STATE_DENSE, EMISSIONS_CAP));

/// Script de référence : `set r0 3 ; loop: emit 1,10 ; wait 3 ; add r0 -1 ; jnz r0 loop ;
/// emit 2,99 ; halt`.
const SCRIPT_TOML: &str = "\
[[step]]
op = \"set\"
register = 0
value = 3

[[step]]
label = \"loop\"
op = \"emit\"
channel = 1
value = 10

[[step]]
op = \"wait\"
ticks = 3

[[step]]
op = \"add\"
register = 0
value = -1

[[step]]
op = \"jnz\"
register = 0
target = \"loop\"

[[step]]
op = \"emit\"
channel = 2
value = 99

[[step]]
op = \"halt\"
";

// ---------------------------------------------------------------------------
// Types de données
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Beat(u32);

/// Ancre de durée de vie (E-12) : voir `tests/script_support/mod.rs`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Anchored<'a, T>(T, PhantomData<&'a ()>);

impl<T> Anchored<'_, T> {
    fn new(value: T) -> Self {
        Self(value, PhantomData)
    }
}

type Acc = Emissions<EMISSIONS_CAP>;

#[derive(Default)]
struct Components<'a> {
    states: SparseSet<ScriptState, SPARSE, STATE_DENSE>,
    beats: SparseSet<Anchored<'a, Beat>, SPARSE, STATE_DENSE>,
}

#[derive(Clone, Copy, Debug)]
enum Add<'a> {
    State(ScriptState),
    Beat(Anchored<'a, Beat>),
}

/// Retraits typés : ensemble fermé de variantes, aucun retrait n'est émis ici.
#[allow(
    dead_code,
    reason = "variantes exigées par `ComponentSet`, jamais construites"
)]
#[derive(Clone, Copy, Debug)]
enum Remove {
    State,
    Beat,
}

const STATE_KIND: usize = 0;
const BEAT_KIND: usize = 1;

impl<'a> ComponentSet for Components<'a> {
    type Add = Add<'a>;
    type Remove = Remove;
    type Counters = [usize; 2];
    type Values<'v>
        = (
        ValuesMut<'v, ScriptState, SPARSE, STATE_DENSE>,
        ValuesMut<'v, Anchored<'a, Beat>, SPARSE, STATE_DENSE>,
    )
    where
        Self: 'v;

    fn kind(add: &Add<'a>) -> usize {
        match add {
            Add::State(_) => STATE_KIND,
            Add::Beat(_) => BEAT_KIND,
        }
    }

    fn remove_kind(remove: &Remove) -> usize {
        match remove {
            Remove::State => STATE_KIND,
            Remove::Beat => BEAT_KIND,
        }
    }

    fn dense_len(&self, kind: usize) -> usize {
        match kind {
            STATE_KIND => self.states.len(),
            _ => self.beats.len(),
        }
    }

    fn dense_capacity(&self, kind: usize) -> usize {
        match kind {
            STATE_KIND => self.states.dense_capacity(),
            _ => self.beats.dense_capacity(),
        }
    }

    fn covers(&self, kind: usize, entity: EntityId) -> bool {
        match kind {
            STATE_KIND => self.states.covers(entity),
            _ => self.beats.covers(entity),
        }
    }

    fn is_present(&self, kind: usize, entity: EntityId) -> bool {
        match kind {
            STATE_KIND => self.states.contains(entity),
            _ => self.beats.contains(entity),
        }
    }

    fn insert(&mut self, entity: EntityId, add: Add<'a>) -> Result<(), InsertError> {
        match add {
            Add::State(value) => self.states.insert(entity, value),
            Add::Beat(value) => self.beats.insert(entity, value),
        }
    }

    fn remove(&mut self, entity: EntityId, remove: Remove) -> bool {
        match remove {
            Remove::State => self.states.remove(entity),
            Remove::Beat => self.beats.remove(entity),
        }
    }

    fn remove_all(&mut self, entity: EntityId) {
        self.states.remove(entity);
        self.beats.remove(entity);
    }

    fn values_mut(&mut self) -> Self::Values<'_> {
        (self.states.values_mut(), self.beats.values_mut())
    }
}

#[derive(Default)]
struct Rs<'a> {
    scripts: ResourceSlot<ScriptTable<'a, SCRIPT_SLOTS>>,
    emissions: ResourceSlot<Anchored<'a, Acc>>,
    transcript: ResourceSlot<Anchored<'a, Acc>>,
}

type W<'a> = World<ENTITIES, Components<'a>, Rs<'a>>;
type Buf<'a> = CommandBuffer<COMMANDS, Add<'a>, Remove>;

// ---------------------------------------------------------------------------
// Montage : `Parts`, `Locate`, `Binding`
// ---------------------------------------------------------------------------

type StateView<'w> = ValuesMut<'w, ScriptState, SPARSE, STATE_DENSE>;
type BeatView<'w, 'a> = ValuesMut<'w, Anchored<'a, Beat>, SPARSE, STATE_DENSE>;

struct Parts<'w, 'a> {
    state: Option<StateView<'w>>,
    beat: Option<BeatView<'w, 'a>>,
    scripts: Option<&'w mut ResourceSlot<ScriptTable<'a, SCRIPT_SLOTS>>>,
    emissions: Option<&'w mut ResourceSlot<Anchored<'a, Acc>>>,
    transcript: Option<&'w mut ResourceSlot<Anchored<'a, Acc>>>,
}

macro_rules! locate {
    ($id:ident, $field:ident, $store:ty) => {
        impl<'w, 'a> Locate<'w, $id> for Parts<'w, 'a> {
            type Store = $store;
            fn take(&mut self) -> Option<Self::Store> {
                self.$field.take()
            }
        }
    };
}

locate!(STATE, state, StateView<'w>);
locate!(BEAT, beat, BeatView<'w, 'a>);
locate!(
    SCRIPTS,
    scripts,
    &'w mut ResourceSlot<ScriptTable<'a, SCRIPT_SLOTS>>
);
locate!(
    EMISSIONS,
    emissions,
    &'w mut ResourceSlot<Anchored<'a, Acc>>
);
locate!(
    TRANSCRIPT,
    transcript,
    &'w mut ResourceSlot<Anchored<'a, Acc>>
);

struct B<'a>(PhantomData<&'a ()>);

impl<'a> Binding for B<'a> {
    type World = W<'a>;
    type Buffer = Buf<'a>;
    type Parts<'w>
        = Parts<'w, 'a>
    where
        Self: 'w;

    fn parts<'w>(world: &'w mut W<'a>, _buffer: &'w mut Buf<'a>) -> Parts<'w, 'a> {
        let ((state, beat), rs) = world.split_mut();
        Parts {
            state: Some(state),
            beat: Some(beat),
            scripts: Some(&mut rs.scripts),
            emissions: Some(&mut rs.emissions),
            transcript: Some(&mut rs.transcript),
        }
    }
}

type StateStore<'w, 'a> = <Parts<'w, 'a> as Locate<'w, STATE>>::Store;
type BeatStore<'w, 'a> = <Parts<'w, 'a> as Locate<'w, BEAT>>::Store;
type ScriptsStore<'w, 'a> = <Parts<'w, 'a> as Locate<'w, SCRIPTS>>::Store;
type EmissionsStore<'w, 'a> = <Parts<'w, 'a> as Locate<'w, EMISSIONS>>::Store;
type TranscriptStore<'w, 'a> = <Parts<'w, 'a> as Locate<'w, TRANSCRIPT>>::Store;

// ---------------------------------------------------------------------------
// Systèmes
// ---------------------------------------------------------------------------

/// Phase 0 : remet l'accumulateur d'émissions à l'état vide.
struct Clear;

impl<'a> System<B<'a>> for Clear {
    type Access<'w>
        = (Produce<EMISSIONS, EmissionsStore<'w, 'a>>,)
    where
        B<'a>: 'w;

    fn run((mut emissions,): Self::Access<'_>) -> Result<(), BufferFull> {
        emissions.insert(Anchored::new(Acc::new()));
        Ok(())
    }
}

/// Phase 1 : unique lecteur de `EMISSIONS`, unique producteur de `TRANSCRIPT` (remplacement
/// intégral, y compris vide).
struct Collect;

impl<'a> System<B<'a>> for Collect {
    type Access<'w>
        = (
        Read<EMISSIONS, EmissionsStore<'w, 'a>>,
        Produce<TRANSCRIPT, TranscriptStore<'w, 'a>>,
    )
    where
        B<'a>: 'w;

    fn run((emissions, mut transcript): Self::Access<'_>) -> Result<(), BufferFull> {
        let Some(source) = emissions.get() else {
            panic!("violation d'assemblage : EMISSIONS absente (Collect)");
        };
        transcript.insert(Anchored::new(source.0.clone()));
        Ok(())
    }
}

/// Un `step` par instance. `Err(Full)` viole la borne
/// `EMISSIONS_CAP >= STATE_DENSE x SCRIPT_MAX_STEPS_PER_TICK`.
fn interpret<'a>(
    state: &mut Write<STATE, StateView<'_>>,
    table: &ScriptTable<'a, SCRIPT_SLOTS>,
    sink: &mut Acc,
) {
    for position in 0..state.len() {
        let Some(&id) = state.entities().get(position) else {
            unreachable!("position {position} sous len()");
        };
        let Some(current) = state.get_mut(id) else {
            unreachable!("entité listée sans composant d'état");
        };
        let script = table.get(current.script());
        let Ok(()) = step(script, current, sink) else {
            unreachable!(
                "contrat violé : EMISSIONS_CAP >= STATE_DENSE x SCRIPT_MAX_STEPS_PER_TICK"
            );
        };
    }
}

/// Phase 1 : interpréteur.
struct Interpreter;

impl<'a> System<B<'a>> for Interpreter {
    type Access<'w>
        = (
        Write<STATE, StateStore<'w, 'a>>,
        Read<SCRIPTS, ScriptsStore<'w, 'a>>,
        Produce<EMISSIONS, EmissionsStore<'w, 'a>>,
    )
    where
        B<'a>: 'w;

    fn run((mut state, table, mut sink): Self::Access<'_>) -> Result<(), BufferFull> {
        let (Some(table), Some(sink)) = (table.get(), sink.get_mut()) else {
            panic!("violation d'assemblage : ressource absente (Interpreter)");
        };
        interpret(&mut state, table, &mut sink.0);
        Ok(())
    }
}

/// Phase 1 : incrémente le `Beat` de chaque instance.
struct Drift;

impl<'a> System<B<'a>> for Drift {
    type Access<'w>
        = (Write<BEAT, BeatStore<'w, 'a>>,)
    where
        B<'a>: 'w;

    fn run((mut beats,): Self::Access<'_>) -> Result<(), BufferFull> {
        for position in 0..beats.len() {
            let Some(&id) = beats.entities().get(position) else {
                unreachable!("position {position} sous len()");
            };
            let Some(beat) = beats.get_mut(id) else {
                unreachable!("entité listée sans composant Beat");
            };
            beat.0.0 = beat.0.0.wrapping_add(1);
        }
        Ok(())
    }
}

type Systems =
    Cons<At<0, Clear>, Cons<At<1, Collect>, Cons<At<1, Interpreter>, Cons<At<1, Drift>, Nil>>>>;

type Sched<'a> = Scheduler<B<'a>, Systems, SYSTEMS, PHASES>;

// ---------------------------------------------------------------------------
// Exécution
// ---------------------------------------------------------------------------

/// Transcript attendu au tick `k` (littéral du scénario de référence).
fn expected(k: usize) -> Vec<(u16, i32)> {
    match k {
        1 | 4 | 7 => vec![(1, 10)],
        10 => vec![(2, 99)],
        _ => Vec::new(),
    }
}

fn main() {
    // Lot compilé : déclaré avant le `World` (les vues empruntent ses octets).
    let sources = [AssetSource {
        kind: AssetKind::Script,
        key: "reference",
        toml: SCRIPT_TOML,
        slot: SlotIndex::new(SLOT),
    }];
    let set = compile_assets(&sources, SCRIPT_SLOTS).expect("lot valide");
    let artifacts: Vec<&[u8]> = set.artifacts().iter().map(Vec::as_slice).collect();

    let scheduler = Sched::new().expect("modèle valide");
    let mut world: W<'_> = World::new();
    {
        let rs = world.resources_mut();
        rs.scripts.insert(ScriptTable::new());
        rs.emissions.insert(Anchored::new(Acc::new()));
        rs.transcript.insert(Anchored::new(Acc::new()));
    }
    let formats = [Format::Script; SCRIPT_SLOTS];
    load(set.manifest(), &artifacts, &formats, |slot, asset| {
        if let Asset::Script(script) = asset {
            let Some(table) = world.resources_mut().scripts.get_mut() else {
                panic!("violation d'assemblage : table absente");
            };
            if table.deposit(slot, script).is_err() {
                panic!("dépôt refusé au slot {}", slot.get());
            }
        }
    })
    .expect("chargement");

    // Lancement par commande différée puis `apply`.
    let mut buffer = Buf::new();
    let id = world.spawn().expect("entité");
    let state = ScriptState::start(SlotIndex::new(SLOT)).expect("slot");
    buffer.add_component(id, Add::State(state)).expect("tampon");
    buffer
        .add_component(id, Add::Beat(Anchored::new(Beat(0))))
        .expect("tampon");
    world.apply(&mut buffer).expect("lot de commandes");

    let mut inbox = Inbox::<u8>::new(4);
    let mut pacing = FixedStepPacing::new(Duration::from_millis(1));
    let stops = Cell::new(0usize);
    let mut k = 0usize;
    let mut observed: Vec<Vec<(u16, i32)>> = Vec::new();
    let ran = run_loop(
        || pacing.wait(),
        || {
            stops.set(stops.get() + 1);
            stops.get() > N
        },
        &mut inbox,
        |_lot: &[u8]| -> Result<(), TickError> {
            k += 1;
            scheduler.run_tick(&mut world, &mut buffer)?;
            let Some(slot) = world.resources().transcript.get() else {
                panic!("violation d'assemblage : transcript absent");
            };
            let pairs: Vec<(u16, i32)> = slot
                .0
                .as_slice()
                .iter()
                .map(|emission| (emission.channel, emission.value))
                .collect();
            println!("tick {k} : {pairs:?}");
            observed.push(pairs);
            Ok(())
        },
    );
    assert_eq!(ran, Ok(N as u64), "boucle");

    // Vérification interne des 12 transcripts.
    assert_eq!(observed.len(), N, "nombre de transcripts");
    for (index, pairs) in observed.iter().enumerate() {
        assert_eq!(*pairs, expected(index + 1), "tick {}", index + 1);
    }
    println!("{N} transcripts conformes");
}
