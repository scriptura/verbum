// crates/verbum-schedule/tests/support/mod.rs

//! Components and resources used by the phase 7a tests, defined outside the crates (like the
//! real caller): two components (`Position`, `Velocity`), three resources (`Accumulator` and
//! two `Counter`s, of the same concrete type), the `Parts` assembly (one `Locate` per
//! category) and its deliberately crossed variant `CrossedParts`.
//!
//! `Motion` has a real `MotionRemove` since phase 7b, so that the commands emitted through
//! `Commands` can be resolved by `World::apply` and observed in the world. `Parts` binds a
//! command buffer only when one is given with `Parts::with_buffer`.

use verbum_ecs::{
    CommandBuffer, ComponentSet, EntityId, InsertError, ResourceSlot, SparseSet, ValuesMut, World,
};
use verbum_schedule::access::{COMMANDS_BINDING, Commands, Locate, Produce, Read, Write};
use verbum_schedule::system::Binding;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position(pub i32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Velocity(pub i32);

/// Transient accumulator (resource), neither `Copy` nor `Default`.
#[derive(Debug, PartialEq)]
pub struct Accumulator(pub i32);

/// Two resources of this concrete type exist in `Globals`: the compiler cannot tell them
/// apart, only the wiring test can.
#[derive(Debug, PartialEq)]
pub struct Counter(pub u32);

// Category identifiers, assigned by the caller (plan, §2.2).
pub const POSITION: u8 = 0;
pub const VELOCITY: u8 = 1;
pub const ACCUMULATOR: u8 = 2;
pub const COUNTER_A: u8 = 3;
pub const COUNTER_B: u8 = 4;

#[derive(Default)]
pub struct Motion {
    pub positions: SparseSet<Position, 4, 4>,
    pub velocities: SparseSet<Velocity, 4, 4>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotionAdd {
    Position(Position),
    Velocity(Velocity),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotionRemove {
    Position,
    Velocity,
}

impl ComponentSet for Motion {
    type Add = MotionAdd;
    type Remove = MotionRemove;
    type Counters = [usize; 2];
    type Values<'a> = (ValuesMut<'a, Position, 4, 4>, ValuesMut<'a, Velocity, 4, 4>);

    fn kind(add: &MotionAdd) -> usize {
        match add {
            MotionAdd::Position(_) => 0,
            MotionAdd::Velocity(_) => 1,
        }
    }

    fn remove_kind(remove: &MotionRemove) -> usize {
        match remove {
            MotionRemove::Position => 0,
            MotionRemove::Velocity => 1,
        }
    }

    fn dense_len(&self, kind: usize) -> usize {
        match kind {
            0 => self.positions.len(),
            _ => self.velocities.len(),
        }
    }

    fn dense_capacity(&self, kind: usize) -> usize {
        match kind {
            0 => self.positions.dense_capacity(),
            _ => self.velocities.dense_capacity(),
        }
    }

    fn covers(&self, kind: usize, entity: EntityId) -> bool {
        match kind {
            0 => self.positions.covers(entity),
            _ => self.velocities.covers(entity),
        }
    }

    fn is_present(&self, kind: usize, entity: EntityId) -> bool {
        match kind {
            0 => self.positions.contains(entity),
            _ => self.velocities.contains(entity),
        }
    }

    fn insert(&mut self, entity: EntityId, add: MotionAdd) -> Result<(), InsertError> {
        match add {
            MotionAdd::Position(p) => self.positions.insert(entity, p),
            MotionAdd::Velocity(v) => self.velocities.insert(entity, v),
        }
    }

    fn remove(&mut self, entity: EntityId, remove: MotionRemove) -> bool {
        match remove {
            MotionRemove::Position => self.positions.remove(entity),
            MotionRemove::Velocity => self.velocities.remove(entity),
        }
    }

    fn remove_all(&mut self, entity: EntityId) {
        self.positions.remove(entity);
        self.velocities.remove(entity);
    }

    fn values_mut(&mut self) -> Self::Values<'_> {
        (self.positions.values_mut(), self.velocities.values_mut())
    }
}

/// Resources of the test world: one field per slot, key = static position (ADR-005).
#[derive(Default)]
pub struct Globals {
    pub accumulator: ResourceSlot<Accumulator>,
    pub counter_a: ResourceSlot<Counter>,
    pub counter_b: ResourceSlot<Counter>,
}

pub type MotionWorld = World<4, Motion, Globals>;

/// Command buffer of the test world: 4 commands, so that overflow is easy to reach.
pub type MotionBuffer = CommandBuffer<4, MotionAdd, MotionRemove>;

pub type Positions<'w> = ValuesMut<'w, Position, 4, 4>;
pub type Velocities<'w> = ValuesMut<'w, Velocity, 4, 4>;
pub type Slot<'w, T> = &'w mut ResourceSlot<T>;

/// Assembly of the world: each slot in its own `Option`, taken once. Built anew for each
/// binding.
pub struct Parts<'w> {
    positions: Option<Positions<'w>>,
    velocities: Option<Velocities<'w>>,
    accumulator: Option<Slot<'w, Accumulator>>,
    counter_a: Option<Slot<'w, Counter>>,
    counter_b: Option<Slot<'w, Counter>>,
    commands: Option<&'w mut MotionBuffer>,
}

impl<'w> Parts<'w> {
    pub fn from_world(world: &'w mut MotionWorld) -> Self {
        let ((positions, velocities), globals) = world.split_mut();
        let Globals {
            accumulator,
            counter_a,
            counter_b,
        } = globals;
        Self {
            positions: Some(positions),
            velocities: Some(velocities),
            accumulator: Some(accumulator),
            counter_a: Some(counter_a),
            counter_b: Some(counter_b),
            commands: None,
        }
    }

    /// Gives the assembly a command buffer, bound under `COMMANDS_BINDING`. Without it, the
    /// buffer slot is empty and `Commands` binds to `None`.
    pub fn with_buffer(mut self, buffer: &'w mut MotionBuffer) -> Self {
        self.commands = Some(buffer);
        self
    }
}

impl<'w> Locate<'w, POSITION> for Parts<'w> {
    type Store = Positions<'w>;
    fn take(&mut self) -> Option<Self::Store> {
        self.positions.take()
    }
}

impl<'w> Locate<'w, VELOCITY> for Parts<'w> {
    type Store = Velocities<'w>;
    fn take(&mut self) -> Option<Self::Store> {
        self.velocities.take()
    }
}

impl<'w> Locate<'w, ACCUMULATOR> for Parts<'w> {
    type Store = Slot<'w, Accumulator>;
    fn take(&mut self) -> Option<Self::Store> {
        self.accumulator.take()
    }
}

impl<'w> Locate<'w, COUNTER_A> for Parts<'w> {
    type Store = Slot<'w, Counter>;
    fn take(&mut self) -> Option<Self::Store> {
        self.counter_a.take()
    }
}

impl<'w> Locate<'w, COUNTER_B> for Parts<'w> {
    type Store = Slot<'w, Counter>;
    fn take(&mut self) -> Option<Self::Store> {
        self.counter_b.take()
    }
}

impl<'w> Locate<'w, COMMANDS_BINDING> for Parts<'w> {
    type Store = &'w mut MotionBuffer;
    fn take(&mut self) -> Option<Self::Store> {
        self.commands.take()
    }
}

/// Documentary witness (plan, §2.4 point 1): a deliberately crossed wiring, where `COUNTER_A`
/// designates the `counter_b` field and conversely. It compiles and behaves crosswise: only a
/// wiring test protects from this labelling error.
pub struct CrossedParts<'w> {
    counter_a: Option<Slot<'w, Counter>>,
    counter_b: Option<Slot<'w, Counter>>,
}

impl<'w> CrossedParts<'w> {
    pub fn from_world(world: &'w mut MotionWorld) -> Self {
        let (_, globals) = world.split_mut();
        Self {
            counter_a: Some(&mut globals.counter_a),
            counter_b: Some(&mut globals.counter_b),
        }
    }
}

impl<'w> Locate<'w, COUNTER_A> for CrossedParts<'w> {
    type Store = Slot<'w, Counter>;
    fn take(&mut self) -> Option<Self::Store> {
        self.counter_b.take() // crossed: A -> field B
    }
}

impl<'w> Locate<'w, COUNTER_B> for CrossedParts<'w> {
    type Store = Slot<'w, Counter>;
    fn take(&mut self) -> Option<Self::Store> {
        self.counter_a.take() // crossed: B -> field A
    }
}

/// Per-world aliases: the `Store` is deduced from `Locate`, never restated (plan, §2.2).
pub type ReadOf<'w, const ID: u8> = Read<ID, <Parts<'w> as Locate<'w, ID>>::Store>;
pub type WriteOf<'w, const ID: u8> = Write<ID, <Parts<'w> as Locate<'w, ID>>::Store>;
pub type ProduceOf<'w, const ID: u8> = Produce<ID, <Parts<'w> as Locate<'w, ID>>::Store>;
pub type CommandsOf<
    'w,
    const SPAWN: bool,
    const DESPAWN: bool,
    const ADD: bool,
    const REMOVE: bool,
> = Commands<<Parts<'w> as Locate<'w, COMMANDS_BINDING>>::Store, SPAWN, DESPAWN, ADD, REMOVE>;

/// Populated world: three entities `[a, b, c]`, positions 1, 2, 3, velocities 10 (a) and 30
/// (c), accumulator at 100, counters A and B at 10 and 20.
pub fn populated_world() -> (MotionWorld, [EntityId; 3]) {
    let mut world = MotionWorld::new();
    let [a, b, c] = [(); 3].map(|()| world.spawn().unwrap());
    let mut buffer = CommandBuffer::<8, MotionAdd, MotionRemove>::new();
    for (entity, x) in [(a, 1), (b, 2), (c, 3)] {
        buffer
            .add_component(entity, MotionAdd::Position(Position(x)))
            .unwrap();
    }
    for (entity, x) in [(a, 10), (c, 30)] {
        buffer
            .add_component(entity, MotionAdd::Velocity(Velocity(x)))
            .unwrap();
    }
    world.apply(&mut buffer).unwrap();
    let globals = world.resources_mut();
    globals.accumulator.insert(Accumulator(100));
    globals.counter_a.insert(Counter(10));
    globals.counter_b.insert(Counter(20));
    (world, [a, b, c])
}

// ---------------------------------------------------------------------------
// Phase 8: world, assembly and bindings of the scheduler tests
// ---------------------------------------------------------------------------

/// Transient category produced by every system of the scheduler tests: each one appends its
/// identifier to the `Log` resource, so that the order of execution is observable.
#[allow(dead_code)]
pub const LOG: u8 = 5;
/// Transient category used to create ordering constraints between systems.
#[allow(dead_code)]
pub const X: u8 = 6;
/// Second transient category, used to build a cycle.
#[allow(dead_code)]
pub const Y: u8 = 7;

/// Execution log: identifiers in order of execution. Capacity 16, above the 13 systems of the
/// largest list of the tests.
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct Log {
    pub entries: [u8; 16],
    pub len: usize,
}

#[allow(dead_code)]
impl Log {
    pub fn new() -> Self {
        Self {
            entries: [0; 16],
            len: 0,
        }
    }

    pub fn push(&mut self, id: u8) {
        self.entries[self.len] = id;
        self.len += 1;
    }

    pub fn recorded(&self) -> &[u8] {
        &self.entries[..self.len]
    }
}

/// Payload of the categories `X` and `Y`: never read, only its slot matters.
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct Token;

#[derive(Default)]
#[allow(dead_code)]
pub struct LogGlobals {
    pub log: ResourceSlot<Log>,
    pub x: ResourceSlot<Token>,
    pub y: ResourceSlot<Token>,
}

#[allow(dead_code)]
pub type LogWorld = World<4, Motion, LogGlobals>;

/// Assembly of `LogWorld`: one `Option` per slot, taken once, built anew for each binding.
#[allow(dead_code)]
pub struct LogParts<'w> {
    log: Option<Slot<'w, Log>>,
    x: Option<Slot<'w, Token>>,
    y: Option<Slot<'w, Token>>,
    commands: Option<&'w mut MotionBuffer>,
}

#[allow(dead_code)]
impl<'w> LogParts<'w> {
    /// Without a buffer: `Commands` then binds to `None`.
    pub fn from_world(world: &'w mut LogWorld) -> Self {
        let (_, LogGlobals { log, x, y }) = world.split_mut();
        Self {
            log: Some(log),
            x: Some(x),
            y: Some(y),
            commands: None,
        }
    }

    pub fn with_buffer(mut self, buffer: &'w mut MotionBuffer) -> Self {
        self.commands = Some(buffer);
        self
    }
}

impl<'w> Locate<'w, LOG> for LogParts<'w> {
    type Store = Slot<'w, Log>;
    fn take(&mut self) -> Option<Self::Store> {
        self.log.take()
    }
}

impl<'w> Locate<'w, X> for LogParts<'w> {
    type Store = Slot<'w, Token>;
    fn take(&mut self) -> Option<Self::Store> {
        self.x.take()
    }
}

impl<'w> Locate<'w, Y> for LogParts<'w> {
    type Store = Slot<'w, Token>;
    fn take(&mut self) -> Option<Self::Store> {
        self.y.take()
    }
}

impl<'w> Locate<'w, COMMANDS_BINDING> for LogParts<'w> {
    type Store = &'w mut MotionBuffer;
    fn take(&mut self) -> Option<Self::Store> {
        self.commands.take()
    }
}

/// The binding of the tests: the caller's buffer is given to the assembly.
#[allow(dead_code)]
pub struct LogBinding;

impl Binding for LogBinding {
    type World = LogWorld;
    type Buffer = MotionBuffer;
    type Parts<'w> = LogParts<'w>;

    fn parts<'w>(world: &'w mut LogWorld, buffer: &'w mut MotionBuffer) -> LogParts<'w> {
        LogParts::from_world(world).with_buffer(buffer)
    }
}

/// A binding that never gives the buffer to the assembly: `Commands` cannot bind.
#[allow(dead_code)]
pub struct NoBufferBinding;

impl Binding for NoBufferBinding {
    type World = LogWorld;
    type Buffer = MotionBuffer;
    type Parts<'w> = LogParts<'w>;

    fn parts<'w>(world: &'w mut LogWorld, _buffer: &'w mut MotionBuffer) -> LogParts<'w> {
        LogParts::from_world(world)
    }
}

/// Per-assembly aliases: the `Store` is deduced from `Locate`, never restated.
#[allow(dead_code)]
pub type LRead<'w, const ID: u8> = Read<ID, <LogParts<'w> as Locate<'w, ID>>::Store>;
#[allow(dead_code)]
pub type LWrite<'w, const ID: u8> = Write<ID, <LogParts<'w> as Locate<'w, ID>>::Store>;
#[allow(dead_code)]
pub type LProduce<'w, const ID: u8> = Produce<ID, <LogParts<'w> as Locate<'w, ID>>::Store>;
#[allow(dead_code)]
pub type LCommands<'w, const SPAWN: bool> =
    Commands<<LogParts<'w> as Locate<'w, COMMANDS_BINDING>>::Store, SPAWN, false, false, false>;

/// Populated log world: empty log, `X` and `Y` slots filled.
#[allow(dead_code)]
pub fn log_world() -> LogWorld {
    let mut world = LogWorld::new();
    let globals = world.resources_mut();
    globals.log.insert(Log::new());
    globals.x.insert(Token);
    globals.y.insert(Token);
    world
}

// ---------------------------------------------------------------------------
// Phase 9: binding of the motion world
// ---------------------------------------------------------------------------

/// Binding of `MotionWorld`: the caller's buffer is given to the assembly, under
/// `COMMANDS_BINDING`.
#[allow(dead_code)]
pub struct MotionBinding;

impl Binding for MotionBinding {
    type World = MotionWorld;
    type Buffer = MotionBuffer;
    type Parts<'w> = Parts<'w>;

    fn parts<'w>(world: &'w mut MotionWorld, buffer: &'w mut MotionBuffer) -> Parts<'w> {
        Parts::from_world(world).with_buffer(buffer)
    }
}

// ---------------------------------------------------------------------------
// Phase 9, commit B: world, assembly and bindings of the tick tests
// ---------------------------------------------------------------------------

/// Entities the systems act on: systems are zero-sized types and cannot hold a handle, so
/// the handles reach them through a resource (read-only: it creates no ordering edge).
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Targets {
    pub a: EntityId,
    pub b: EntityId,
    pub c: EntityId,
    pub d: EntityId,
}

/// A witness cell: a value written by a system and read back by the test.
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Cell(pub i32);

// Category identifiers of the tick world. `POSITION` and `VELOCITY` are those of `Motion`.
#[allow(dead_code)]
pub const TARGETS: u8 = 20;
/// Written by the emitter of the dependency test, read by its observer.
#[allow(dead_code)]
pub const FLAG: u8 = 21;
/// What the observer of the first phase saw.
#[allow(dead_code)]
pub const SEEN_B: u8 = 22;
/// What the observer of the following phase saw.
#[allow(dead_code)]
pub const SEEN_C: u8 = 23;
/// A value written by a system, to be found again after an error.
#[allow(dead_code)]
pub const VALUE: u8 = 24;
/// Written by a system that must not run: stays at 0 when it did not.
#[allow(dead_code)]
pub const WITNESS: u8 = 25;
/// Counts the runs of a system.
#[allow(dead_code)]
pub const TALLY: u8 = 26;

#[allow(dead_code)]
#[derive(Default)]
pub struct TickGlobals {
    pub targets: ResourceSlot<Targets>,
    pub flag: ResourceSlot<Cell>,
    pub seen_b: ResourceSlot<Cell>,
    pub seen_c: ResourceSlot<Cell>,
    pub value: ResourceSlot<Cell>,
    pub witness: ResourceSlot<Cell>,
    pub tally: ResourceSlot<Cell>,
}

#[allow(dead_code)]
pub type TickWorld = World<4, Motion, TickGlobals>;

/// Assembly of `TickWorld`: one `Option` per slot, taken once, built anew for each binding.
#[allow(dead_code)]
pub struct TickParts<'w> {
    positions: Option<Positions<'w>>,
    velocities: Option<Velocities<'w>>,
    targets: Option<Slot<'w, Targets>>,
    flag: Option<Slot<'w, Cell>>,
    seen_b: Option<Slot<'w, Cell>>,
    seen_c: Option<Slot<'w, Cell>>,
    value: Option<Slot<'w, Cell>>,
    witness: Option<Slot<'w, Cell>>,
    tally: Option<Slot<'w, Cell>>,
    commands: Option<&'w mut MotionBuffer>,
}

#[allow(dead_code)]
impl<'w> TickParts<'w> {
    pub fn from_world(world: &'w mut TickWorld, buffer: &'w mut MotionBuffer) -> Self {
        let ((positions, velocities), globals) = world.split_mut();
        let TickGlobals {
            targets,
            flag,
            seen_b,
            seen_c,
            value,
            witness,
            tally,
        } = globals;
        Self {
            positions: Some(positions),
            velocities: Some(velocities),
            targets: Some(targets),
            flag: Some(flag),
            seen_b: Some(seen_b),
            seen_c: Some(seen_c),
            value: Some(value),
            witness: Some(witness),
            tally: Some(tally),
            commands: Some(buffer),
        }
    }

    /// A deliberately faulty assembly: the `WITNESS` slot is missing.
    pub fn without_witness(mut self) -> Self {
        self.witness = None;
        self
    }
}

macro_rules! locate_slot {
    ($($id:ident => $field:ident: $store:ty;)*) => {$(
        impl<'w> Locate<'w, $id> for TickParts<'w> {
            type Store = $store;
            fn take(&mut self) -> Option<Self::Store> {
                self.$field.take()
            }
        }
    )*};
}

locate_slot! {
    POSITION => positions: Positions<'w>;
    VELOCITY => velocities: Velocities<'w>;
    TARGETS => targets: Slot<'w, Targets>;
    FLAG => flag: Slot<'w, Cell>;
    SEEN_B => seen_b: Slot<'w, Cell>;
    SEEN_C => seen_c: Slot<'w, Cell>;
    VALUE => value: Slot<'w, Cell>;
    WITNESS => witness: Slot<'w, Cell>;
    TALLY => tally: Slot<'w, Cell>;
    COMMANDS_BINDING => commands: &'w mut MotionBuffer;
}

/// The binding of the tick tests.
#[allow(dead_code)]
pub struct TickBinding;

impl Binding for TickBinding {
    type World = TickWorld;
    type Buffer = MotionBuffer;
    type Parts<'w> = TickParts<'w>;

    fn parts<'w>(world: &'w mut TickWorld, buffer: &'w mut MotionBuffer) -> TickParts<'w> {
        TickParts::from_world(world, buffer)
    }
}

/// A binding whose `parts` forgets the `WITNESS` slot: an access that needs it cannot bind.
#[allow(dead_code)]
pub struct DefectiveBinding;

impl Binding for DefectiveBinding {
    type World = TickWorld;
    type Buffer = MotionBuffer;
    type Parts<'w> = TickParts<'w>;

    fn parts<'w>(world: &'w mut TickWorld, buffer: &'w mut MotionBuffer) -> TickParts<'w> {
        TickParts::from_world(world, buffer).without_witness()
    }
}

/// Per-assembly aliases: the `Store` is deduced from `Locate`, never restated.
#[allow(dead_code)]
pub type TRead<'w, const ID: u8> = Read<ID, <TickParts<'w> as Locate<'w, ID>>::Store>;
#[allow(dead_code)]
pub type TWrite<'w, const ID: u8> = Write<ID, <TickParts<'w> as Locate<'w, ID>>::Store>;
#[allow(dead_code)]
pub type TCommands<
    'w,
    const SPAWN: bool,
    const DESPAWN: bool,
    const ADD: bool,
    const REMOVE: bool,
> = Commands<<TickParts<'w> as Locate<'w, COMMANDS_BINDING>>::Store, SPAWN, DESPAWN, ADD, REMOVE>;

/// Tick world with four entities, `[a, b, c, d]`:
/// `a` has a position (1) and a velocity (10); `b` has nothing; `c` has a position (3) and a
/// velocity (30); `d` has a position (4). Every witness cell is at 0.
#[allow(dead_code)]
pub fn tick_world() -> (TickWorld, Targets) {
    let mut world = TickWorld::new();
    let [a, b, c, d] = [(); 4].map(|()| world.spawn().unwrap());
    let mut buffer = CommandBuffer::<8, MotionAdd, MotionRemove>::new();
    for (entity, x) in [(a, 1), (c, 3), (d, 4)] {
        buffer
            .add_component(entity, MotionAdd::Position(Position(x)))
            .unwrap();
    }
    for (entity, x) in [(a, 10), (c, 30)] {
        buffer
            .add_component(entity, MotionAdd::Velocity(Velocity(x)))
            .unwrap();
    }
    world.apply(&mut buffer).unwrap();
    let targets = Targets { a, b, c, d };
    let globals = world.resources_mut();
    globals.targets.insert(targets);
    for slot in [
        &mut globals.flag,
        &mut globals.seen_b,
        &mut globals.seen_c,
        &mut globals.value,
        &mut globals.witness,
        &mut globals.tally,
    ] {
        slot.insert(Cell(0));
    }
    (world, targets)
}
