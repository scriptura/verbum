// crates/schedule/src/scheduler.rs

//! The scheduler: a typed system list and the model built from its contracts alone
//! (ADR-004).
//!
//! [`Scheduler::new`] derives one `(phase, contract)` pair per system from the list, in
//! declaration order, and hands them to [`ExecutionModel::build`]. The model never sees a
//! system, only contracts: two lists whose systems differ but whose contracts are equal
//! give equal models.
//!
//! The number of systems is stated twice, by the list and by the `SYSTEMS` parameter of
//! the model; they must agree. The mismatch is a compile-time error (E0080):
//!
//! ```compile_fail,E0080
//! use verbum_ecs::BufferFull;
//! use verbum_schedule::scheduler::Scheduler;
//! use verbum_schedule::system::{At, Binding, Cons, Nil, System};
//!
//! struct Nothing;
//! impl Binding for Nothing {
//!     type World = ();
//!     type Buffer = ();
//!     type Parts<'w> = ();
//!     fn parts<'w>(_: &'w mut (), _: &'w mut ()) {}
//! }
//!
//! struct Idle;
//! impl System<Nothing> for Idle {
//!     type Access<'w> = ();
//!     fn run(_: ()) -> Result<(), BufferFull> {
//!         Ok(())
//!     }
//! }
//!
//! // One system in the list, two declared.
//! let _ = Scheduler::<Nothing, Cons<At<0, Idle>, Nil>, 2, 1>::new();
//! ```
//!
//! Nor can the list hold more systems than declared (E0080):
//!
//! ```compile_fail,E0080
//! use verbum_ecs::BufferFull;
//! use verbum_schedule::scheduler::Scheduler;
//! use verbum_schedule::system::{At, Binding, Cons, Nil, System};
//!
//! struct Nothing;
//! impl Binding for Nothing {
//!     type World = ();
//!     type Buffer = ();
//!     type Parts<'w> = ();
//!     fn parts<'w>(_: &'w mut (), _: &'w mut ()) {}
//! }
//!
//! struct Idle;
//! impl System<Nothing> for Idle {
//!     type Access<'w> = ();
//!     fn run(_: ()) -> Result<(), BufferFull> {
//!         Ok(())
//!     }
//! }
//!
//! // Two systems in the list, one declared.
//! let _ = Scheduler::<Nothing, Cons<At<0, Idle>, Cons<At<0, Idle>, Nil>>, 1, 1>::new();
//! ```
//!
//! The error appears when the constant is evaluated, that is at build or test time, not
//! under `cargo check`.
//!
//! Twin with the right count:
//!
//! ```
//! use verbum_ecs::BufferFull;
//! use verbum_schedule::scheduler::Scheduler;
//! use verbum_schedule::system::{At, Binding, Cons, Nil, System};
//!
//! struct Nothing;
//! impl Binding for Nothing {
//!     type World = ();
//!     type Buffer = ();
//!     type Parts<'w> = ();
//!     fn parts<'w>(_: &'w mut (), _: &'w mut ()) {}
//! }
//!
//! struct Idle;
//! impl System<Nothing> for Idle {
//!     type Access<'w> = ();
//!     fn run(_: ()) -> Result<(), BufferFull> {
//!         Ok(())
//!     }
//! }
//!
//! let scheduler = Scheduler::<Nothing, Cons<At<0, Idle>, Nil>, 1, 1>::new().unwrap();
//! assert_eq!(scheduler.model().phase(0), Some(&[0][..]));
//! ```

use core::fmt;
use core::marker::PhantomData;

use verbum_ecs::{ApplyError, BufferFull, CommandBuffer, ComponentSet, World};

use crate::contract::{Contract, StructuralEffects};
use crate::data::DataSet;
use crate::model::{ExecutionModel, ModelError};
use crate::system::{Binding, RunError, SystemList};

/// Why a tick did not complete.
///
/// A tick that fails stops at once: the systems already run keep their effects, nothing is
/// rolled back, and the systems and phases that follow are not run.
#[derive(Debug, PartialEq, Eq)]
pub enum TickError {
    /// An access could not be bound: a slot it needs was already taken or is not provided by
    /// the `Parts` of the binding.
    Binding,
    /// A system found the command buffer full.
    Buffer(BufferFull),
    /// The buffer was not empty when the tick started.
    BufferNotEmpty,
    /// The validation of a phase rejected its batch of commands.
    Apply(ApplyError),
}

impl From<RunError> for TickError {
    fn from(error: RunError) -> Self {
        match error {
            RunError::Binding => Self::Binding,
            RunError::Buffer(full) => Self::Buffer(full),
        }
    }
}

/// A typed list `L` of `SYSTEMS` systems over `PHASES` phases, with its execution model.
pub struct Scheduler<B, L, const SYSTEMS: usize, const PHASES: usize> {
    model: ExecutionModel<SYSTEMS, PHASES>,
    list: PhantomData<fn() -> (B, L)>,
}

impl<B: Binding, L: SystemList<B>, const SYSTEMS: usize, const PHASES: usize>
    Scheduler<B, L, SYSTEMS, PHASES>
{
    /// The list and the declared count agree. Evaluated by [`new`](Self::new).
    const LEN_MATCHES: () = assert!(
        L::LEN == SYSTEMS,
        "the number of systems of the list differs from SYSTEMS"
    );

    /// Builds the model from the contracts of the list, in declaration order.
    ///
    /// Fails with the [`ModelError`] of [`ExecutionModel::build`].
    pub fn new() -> Result<Self, ModelError> {
        let () = Self::LEN_MATCHES;

        let filler = Contract::new(
            DataSet::EMPTY,
            DataSet::EMPTY,
            DataSet::EMPTY,
            StructuralEffects::NONE,
        );
        let mut systems = [(0, filler); SYSTEMS];
        L::fill(&mut systems, 0);
        Ok(Self {
            model: ExecutionModel::build(systems)?,
            list: PhantomData,
        })
    }

    /// Runs one tick: the phases in order, and in each phase the systems in the order of the
    /// model. Each system binds its access from a fresh `B::parts` (see
    /// [`SystemList::run`]). At the end of every phase, including a phase that emitted no
    /// command, the commands of the buffer are validated and applied by `World::apply`: the
    /// structural effects of a phase are visible from the next phase on, never inside it.
    ///
    /// The scheduler is only read: the model is the same after any number of ticks.
    ///
    /// A buffer that is not empty on entry is refused with [`TickError::BufferNotEmpty`]
    /// before any system runs. A tick that succeeds leaves the buffer empty.
    ///
    /// The first error stops the tick (see [`TickError`]). There is no rollback: the values
    /// written by the systems already run and the phases already validated stay as they are;
    /// the commands accepted so far stay in `buffer`, unapplied, until the caller empties it
    /// with `clear`. A batch rejected by `World::apply` is kept intact. The systems and
    /// phases that follow are not run.
    pub fn run_tick<const ENTITIES: usize, const COMMANDS: usize, S, R>(
        &self,
        world: &mut B::World,
        buffer: &mut B::Buffer,
    ) -> Result<(), TickError>
    where
        B: Binding<
                World = World<ENTITIES, S, R>,
                Buffer = CommandBuffer<COMMANDS, S::Add, S::Remove>,
            >,
        S: ComponentSet,
        R: Default,
    {
        if !buffer.is_empty() {
            return Err(TickError::BufferNotEmpty);
        }
        for phase in 0..PHASES {
            let Some(order) = self.model.phase(phase) else {
                unreachable!("phase {phase} is below PHASES")
            };
            for &index in order {
                L::run(index, world, buffer)?;
            }
            world.apply(buffer).map_err(TickError::Apply)?;
        }
        Ok(())
    }

    /// The execution model, immutable.
    pub const fn model(&self) -> &ExecutionModel<SYSTEMS, PHASES> {
        &self.model
    }
}

impl<B, L, const SYSTEMS: usize, const PHASES: usize> fmt::Debug
    for Scheduler<B, L, SYSTEMS, PHASES>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scheduler")
            .field("model", &self.model)
            .finish()
    }
}
