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
