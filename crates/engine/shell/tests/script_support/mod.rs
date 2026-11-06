// crates/engine/shell/tests/script_support/mod.rs

//! Assemblage partagé des tests de l'étape 6 : `World`, `Scheduler`, systèmes `Clear`,
//! `Collect`, `Interpreter` et `Drift`, montage emprunté (`B<'a>`), fonctions d'assemblage.
//!
//! Inclus par `#[path = "script_support/mod.rs"] mod support;` (`script_suspend.rs` en U4,
//! `script_loaded.rs` en U5), comme `tests/support/mod.rs` de `crates/engine/schedule`.
//!
//! Types retenus :
//!
//! - `Beat(u32)` : composant `Copy` sans durée de vie ; stocké sous `Anchored<'a, Beat>` ;
//! - `Anchored<'a, T>` : enveloppe qui porte `'a` (E-12). Elle sert `Clear`, `Collect`,
//!   `Drift`, le stockage de `Beat` et les deux ressources `Emissions` ; `ScriptTable<'a, _>`
//!   et `ScriptState` ne sont jamais enveloppés ;
//! - `Rs<'a>` : un `ResourceSlot` par ressource, donc `Default` sans exiger `Default` de
//!   `ScriptTable` ni d'`Emissions` ;
//! - catégories `STATE`, `SCRIPTS`, `EMISSIONS`, `TRANSCRIPT`, `BEAT` : entiers attribués ici.
//!
//! Aucun état hors du `World`, aucun `static`, aucune cellule : les systèmes sont des types
//! sans donnée.

use core::marker::PhantomData;

use verbum_assets::{Asset, Format, LoadError, SlotIndex, load};
use verbum_ecs::{
    BufferFull, CommandBuffer, ComponentSet, EntityId, InsertError, ResourceSlot, SparseSet,
    ValuesMut, World,
};
use verbum_forge::{AssetKind, AssetSource, CompiledSet, compile_assets};
use verbum_schedule::access::{Locate, Produce, Read, Write};
use verbum_schedule::scheduler::Scheduler;
use verbum_schedule::system::{At, Binding, Cons, Nil, System};
use verbum_script::{
    ENCODED_LEN, Emissions, SCRIPT_MAX_STEPS_PER_TICK, ScriptState, ScriptTable,
    emissions_bound_holds, step,
};

// ---------------------------------------------------------------------------
// Constantes de scène
// ---------------------------------------------------------------------------

/// Capacité en entités du `World`.
pub const ENTITIES: usize = 4;
/// Étendue sparse des deux stockages (ordre des paramètres : `SPARSE`, `DENSE`).
pub const SPARSE: usize = 4;
/// Capacité dense des deux stockages : nombre maximal d'instances de script.
pub const STATE_DENSE: usize = 4;
/// Capacité du tampon de commandes.
pub const COMMANDS: usize = 4;
/// Capacité des accumulateurs d'émissions (`EMISSIONS` et `TRANSCRIPT`).
pub const EMISSIONS_CAP: usize = 64;
/// Nombre de slots de la table de scripts et de la table de formats.
pub const SCRIPT_SLOTS: usize = 6;
/// Nombre de systèmes de l'assemblage et nombre de phases.
pub const SYSTEMS: usize = 4;
pub const PHASES: usize = 2;
/// Nombre de systèmes de l'assemblage de contrôle de CT-2 (celui de l'assemblage plus `RivalWriter`).
pub const RIVAL_SYSTEMS: usize = 5;

/// Catégories de données (entiers `< 64`, attribués par l'assemblage).
pub const STATE: u8 = 0;
pub const SCRIPTS: u8 = 1;
pub const EMISSIONS: u8 = 2;
pub const TRANSCRIPT: u8 = 3;
pub const BEAT: u8 = 4;

/// Borne des émissions : chaque instance émet au plus `SCRIPT_MAX_STEPS_PER_TICK` fois par tick.
const _: () = assert!(emissions_bound_holds(STATE_DENSE, EMISSIONS_CAP));

/// Table des formats attendus : tous les slots désignent un script.
pub const SCRIPT_FORMATS: [Format; SCRIPT_SLOTS] = [Format::Script; SCRIPT_SLOTS];

// ---------------------------------------------------------------------------
// Types de données
// ---------------------------------------------------------------------------

/// Compteur de ticks de l'instance : composant `Copy`, sans durée de vie.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Beat(pub u32);

/// Ancre de durée de vie (E-12) : enveloppe qui porte `'a` sans changer ni catégorie ni
/// contrat. `Copy` quand `T` l'est : le stockage de `Beat` l'exige (`SparseSet<T: Copy>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Anchored<'a, T>(pub T, PhantomData<&'a ()>);

impl<T> Anchored<'_, T> {
    pub fn new(value: T) -> Self {
        Self(value, PhantomData)
    }
}

/// Accumulateur ou transcript : même type, capacité fixe.
pub type Acc = Emissions<EMISSIONS_CAP>;

/// Stockages : un `ScriptState` et un `Beat` par instance. Le stockage de `Beat` est ancré, donc
/// `Components<'a>` porte la durée de vie.
#[derive(Default)]
pub struct Components<'a> {
    pub states: SparseSet<ScriptState, SPARSE, STATE_DENSE>,
    pub beats: SparseSet<Anchored<'a, Beat>, SPARSE, STATE_DENSE>,
}

/// Ajouts typés : une variante par stockage.
#[derive(Clone, Copy, Debug)]
pub enum Add<'a> {
    State(ScriptState),
    Beat(Anchored<'a, Beat>),
}

/// Retraits typés : deux variantes sans donnée.
#[derive(Clone, Copy, Debug)]
pub enum Remove {
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

/// Ressources : la table (non enveloppée : elle porte déjà `'a`) et les deux accumulateurs.
#[derive(Default)]
pub struct Rs<'a> {
    pub scripts: ResourceSlot<ScriptTable<'a, SCRIPT_SLOTS>>,
    pub emissions: ResourceSlot<Anchored<'a, Acc>>,
    pub transcript: ResourceSlot<Anchored<'a, Acc>>,
}

pub type W<'a> = World<ENTITIES, Components<'a>, Rs<'a>>;
pub type Buf<'a> = CommandBuffer<COMMANDS, Add<'a>, Remove>;

// ---------------------------------------------------------------------------
// Montage : `Parts`, `Locate`, `Binding`
// ---------------------------------------------------------------------------

type StateView<'w> = ValuesMut<'w, ScriptState, SPARSE, STATE_DENSE>;
type BeatView<'w, 'a> = ValuesMut<'w, Anchored<'a, Beat>, SPARSE, STATE_DENSE>;

pub struct Parts<'w, 'a> {
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

pub struct B<'a>(PhantomData<&'a ()>);

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
pub struct Clear;

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

/// Phase 1 : unique lecteur de `EMISSIONS`, unique producteur de `TRANSCRIPT`. Remplace le
/// transcript en entier, sans condition, y compris quand l'accumulateur est vide.
pub struct Collect;

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

/// Exécute un `step` par instance, dans l'ordre dense du stockage (aucun ordre n'est affirmé).
/// `Err(Full)` est une violation de la borne `EMISSIONS_CAP ≥ STATE_DENSE × SCRIPT_MAX_STEPS_PER_TICK`,
/// jamais une politique de saturation.
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

/// Phase 1 : une exécution de `step` par instance.
pub struct Interpreter;

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
pub struct Drift;

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

/// Contrôle de CT-1 : même corps que `Interpreter`, un accès `Read<TRANSCRIPT>` de plus.
pub struct WideInterpreter;

impl<'a> System<B<'a>> for WideInterpreter {
    type Access<'w>
        = (
        Write<STATE, StateStore<'w, 'a>>,
        Read<SCRIPTS, ScriptsStore<'w, 'a>>,
        Produce<EMISSIONS, EmissionsStore<'w, 'a>>,
        Read<TRANSCRIPT, TranscriptStore<'w, 'a>>,
    )
    where
        B<'a>: 'w;

    fn run((mut state, table, mut sink, _transcript): Self::Access<'_>) -> Result<(), BufferFull> {
        let (Some(table), Some(sink)) = (table.get(), sink.get_mut()) else {
            panic!("violation d'assemblage : ressource absente (WideInterpreter)");
        };
        interpret(&mut state, table, &mut sink.0);
        Ok(())
    }
}

/// Contrôle de CT-2 : second écrivain de `STATE` dans la phase 1. Ne s'exécute jamais.
///
/// `Write<STATE>` seul ne compile pas sous `B<'a>` (E0477 : l'accès ne mentionne aucun type qui
/// porte `'a`). Le `Read<SCRIPTS>` en est l'ancre ; il n'affecte pas la propriété de `STATE`.
pub struct RivalWriter;

impl<'a> System<B<'a>> for RivalWriter {
    type Access<'w>
        = (
        Write<STATE, StateStore<'w, 'a>>,
        Read<SCRIPTS, ScriptsStore<'w, 'a>>,
    )
    where
        B<'a>: 'w;

    fn run(_access: Self::Access<'_>) -> Result<(), BufferFull> {
        Ok(())
    }
}

/// Liste de l'assemblage. L'ordre de déclaration ne fixe aucune dépendance : l'ordre exécuté
/// se déduit des contrats ; il ne départage que les systèmes indépendants.
pub type Systems =
    Cons<At<0, Clear>, Cons<At<1, Collect>, Cons<At<1, Interpreter>, Cons<At<1, Drift>, Nil>>>>;

/// Liste de contrôle de CT-2 : l'assemblage plus `RivalWriter`.
pub type RivalSystems = Cons<
    At<0, Clear>,
    Cons<
        At<1, Collect>,
        Cons<At<1, Interpreter>, Cons<At<1, Drift>, Cons<At<1, RivalWriter>, Nil>>>,
    >,
>;

pub type Sched<'a> = Scheduler<B<'a>, Systems, SYSTEMS, PHASES>;

// ---------------------------------------------------------------------------
// Sources et artefacts de référence
// ---------------------------------------------------------------------------

/// Slot de référence : artefact 0 d'un lot d'un seul script.
pub const SLOT_A: u32 = 4;
/// Slot du second script de scène.
pub const SLOT_B: u32 = 1;

/// Script de référence (A).
pub const SCRIPT_A_TOML: &str = "\
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

/// Second script de scène (B) : `emit 3,7 ; wait 1 ; emit 3,8 ; halt`.
pub const SCRIPT_B_TOML: &str = "\
[[step]]
op = \"emit\"
channel = 3
value = 7

[[step]]
op = \"wait\"
ticks = 1

[[step]]
op = \"emit\"
channel = 3
value = 8

[[step]]
op = \"halt\"
";

/// Artefact attendu de A (68 octets), écrit à la main, indépendant des constantes.
pub const ARTIFACT_A: [u8; 68] = [
    0x56, 0x53, 0x43, 0x52, // identité VSCR
    0x01, 0x00, // version 1
    0x3A, 0x00, 0x00, 0x00, // charge utile 58
    0x07, 0x00, // n = 7
    0x03, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, // 0 : Set r0, 3
    0x01, 0x00, 0x01, 0x00, 0x0A, 0x00, 0x00, 0x00, // 1 : Emit canal 1, valeur 10
    0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, // 2 : Wait 3
    0x04, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, // 3 : Add r0, -1
    0x06, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, // 4 : JumpIfNonZero r0, cible 1
    0x01, 0x00, 0x02, 0x00, 0x63, 0x00, 0x00, 0x00, // 5 : Emit canal 2, valeur 99
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 6 : Halt
];

/// Compile des scripts `(clé, source, slot)` par la Forge, dans l'ordre donné
/// (`artifact_index` = position).
pub fn compile_scripts(sources: &[(&str, &str, u32)]) -> CompiledSet {
    let sources: Vec<AssetSource<'_>> = sources
        .iter()
        .map(|&(key, toml, slot)| AssetSource {
            kind: AssetKind::Script,
            key,
            toml,
            slot: SlotIndex::new(slot),
        })
        .collect();
    match compile_assets(&sources, SCRIPT_SLOTS) {
        Ok(set) => set,
        Err(error) => panic!("compilation du lot refusée : {error:?}"),
    }
}

/// Lot de référence : A seul, au slot 4. Contrôle l'artefact contre le littéral de 68 octets.
pub fn reference_set() -> CompiledSet {
    let set = compile_scripts(&[("a", SCRIPT_A_TOML, SLOT_A)]);
    assert_eq!(set.artifacts().len(), 1);
    assert_eq!(set.artifacts()[0].as_slice(), ARTIFACT_A.as_slice());
    set
}

/// Lot de SR-5 : A au slot 4 (artefact 0), B au slot 1 (artefact 1).
pub fn pair_set() -> CompiledSet {
    compile_scripts(&[("a", SCRIPT_A_TOML, SLOT_A), ("b", SCRIPT_B_TOML, SLOT_B)])
}

pub fn slices(set: &CompiledSet) -> Vec<&[u8]> {
    set.artifacts().iter().map(Vec::as_slice).collect()
}

// ---------------------------------------------------------------------------
// Fonctions d'assemblage
// ---------------------------------------------------------------------------

/// Monde de départ : table vide, `EMISSIONS` et `TRANSCRIPT` vides et présents.
pub fn build_world<'a>() -> W<'a> {
    let mut world = W::new();
    let rs = world.resources_mut();
    rs.scripts.insert(ScriptTable::new());
    rs.emissions.insert(Anchored::new(Acc::new()));
    rs.transcript.insert(Anchored::new(Acc::new()));
    world
}

/// Dépose dans la table, par le vrai `load`, les seuls artefacts de format script. Le dépôt
/// se fait au `SlotIndex` reçu.
pub fn load_scripts<'a>(
    world: &mut W<'a>,
    manifest: &[u8],
    artifacts: &[&'a [u8]],
) -> Result<(), LoadError> {
    load(manifest, artifacts, &SCRIPT_FORMATS, |slot, asset| {
        if let Asset::Script(script) = asset {
            let Some(table) = world.resources_mut().scripts.get_mut() else {
                panic!("violation d'assemblage : table absente");
            };
            if table.deposit(slot, script).is_err() {
                panic!("dépôt refusé au slot {}", slot.get());
            }
        }
    })
}

/// Lance une instance : `spawn`, commandes `add_component` (état puis `Beat`) ajoutées au
/// tampon, `observe` (le monde n'a pas encore reçu les composants ; le tampon les porte), puis
/// `World::apply`.
/// Aucune écriture directe d'un composant.
pub fn launch_observed<'a>(
    world: &mut W<'a>,
    buffer: &mut Buf<'a>,
    slot: u32,
    beat: u32,
    observe: impl FnOnce(&W<'a>, &Buf<'a>, EntityId),
) -> EntityId {
    let Ok(id) = world.spawn() else {
        panic!("capacité en entités dépassée");
    };
    let Ok(state) = ScriptState::start(SlotIndex::new(slot)) else {
        panic!("slot hors de la borne de ScriptState");
    };
    if buffer.add_component(id, Add::State(state)).is_err()
        || buffer
            .add_component(id, Add::Beat(Anchored::new(Beat(beat))))
            .is_err()
    {
        panic!("tampon de commandes plein");
    }
    observe(world, buffer, id);
    if world.apply(buffer).is_err() {
        panic!("lot de commandes refusé");
    }
    id
}

/// Lancement ordinaire : vérifie l'absence des composants avant `apply`.
pub fn launch<'a>(world: &mut W<'a>, buffer: &mut Buf<'a>, slot: u32, beat: u32) -> EntityId {
    launch_observed(world, buffer, slot, beat, |world, _buffer, id| {
        let components = world.components();
        assert!(
            components.states.get(id).is_none() && components.beats.get(id).is_none(),
            "composant présent avant World::apply"
        );
    })
}

/// Les 26 octets du composant d'état de l'instance.
pub fn capture(world: &W<'_>, id: EntityId) -> [u8; ENCODED_LEN] {
    let Some(state) = world.components().states.get(id) else {
        panic!("composant d'état absent");
    };
    state.to_bytes()
}

/// Monde neuf, table rechargée depuis les mêmes artefacts, instance recréée depuis la capture
/// seule (`from_bytes`, `add_component`, `apply`), `Beat` initial `beat`.
pub fn reconstruct<'a>(
    manifest: &[u8],
    artifacts: &[&'a [u8]],
    capture: &[u8; ENCODED_LEN],
    beat: u32,
) -> (W<'a>, Buf<'a>, EntityId) {
    let mut world = build_world();
    if load_scripts(&mut world, manifest, artifacts).is_err() {
        panic!("rechargement de la table refusé");
    }
    let Ok(state) = ScriptState::from_bytes(capture) else {
        panic!("capture invalide");
    };
    let Ok(id) = world.spawn() else {
        panic!("capacité en entités dépassée");
    };
    let mut buffer = Buf::new();
    if buffer.add_component(id, Add::State(state)).is_err()
        || buffer
            .add_component(id, Add::Beat(Anchored::new(Beat(beat))))
            .is_err()
    {
        panic!("tampon de commandes plein");
    }
    if world.apply(&mut buffer).is_err() {
        panic!("lot de commandes refusé");
    }
    (world, buffer, id)
}

/// Un tick sous l'ordonnanceur `scheduler`.
pub fn tick<'a>(scheduler: &Sched<'a>, world: &mut W<'a>, buffer: &mut Buf<'a>) {
    if let Err(error) = scheduler.run_tick(world, buffer) {
        panic!("tick refusé : {error:?}");
    }
}

/// Émissions du transcript courant, dans l'ordre du tampon, ou `None` si la ressource est absente.
pub fn transcript(world: &W<'_>) -> Option<Vec<(u16, i32)>> {
    let slot = world.resources().transcript.get()?;
    Some(
        slot.0
            .as_slice()
            .iter()
            .map(|emission| (emission.channel, emission.value))
            .collect(),
    )
}

/// Forme normale d'un multiensemble : tri, multiplicité conservée.
pub fn multiset(mut pairs: Vec<(u16, i32)>) -> Vec<(u16, i32)> {
    pairs.sort_unstable();
    pairs
}

/// `Beat` de l'instance.
pub fn beat_of(world: &W<'_>, id: EntityId) -> u32 {
    let Some(beat) = world.components().beats.get(id) else {
        panic!("composant Beat absent");
    };
    beat.0.0
}

/// Borne rappelée aux tests de budget : nombre maximal de pas par tick.
pub const MAX_STEPS: usize = SCRIPT_MAX_STEPS_PER_TICK;
