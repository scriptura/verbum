// crates/schedule/src/system.rs

//! Typed systems and the static list that carries them (ADR-003, ADR-004).
//!
//! A system is a type: it names its [`Access`](System::Access) and runs on it. Its
//! [`Contract`] is derived from the access type alone ([`contract_of`]); nothing else
//! describes the system to the scheduler, so a system cannot declare more or less than
//! it uses (I1).
//!
//! The systems of a schedule form a typed list, [`Cons`] cells ending in [`Nil`], each
//! head being an [`At`] that gives the system its phase. No `dyn`, no function pointer,
//! no allocation: the list is resolved at compile time, and `run` dispatches on the
//! position by recursion over the list. Cost of a dispatch: linear in the position, so
//! quadratic for a full pass over `N` systems; not measured.
//!
//! The contract is never stated by hand. An `impl System` cannot carry one (E0438):
//!
//! ```compile_fail,E0438
//! use verbum_schedule::contract::Contract;
//! use verbum_schedule::system::{Binding, System};
//! use verbum_schedule::access::Part;
//! use verbum_ecs::BufferFull;
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
//!     const CONTRACT: Contract = <() as Part>::CONTRACT;
//!     fn run(_: ()) -> Result<(), BufferFull> {
//!         Ok(())
//!     }
//! }
//! ```
//!
//! Twin without the constant, which compiles and whose contract is derived:
//!
//! ```
//! use verbum_schedule::contract::Contract;
//! use verbum_schedule::data::DataSet;
//! use verbum_schedule::system::{Binding, System, contract_of};
//! use verbum_ecs::BufferFull;
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
//! const CONTRACT: Contract = contract_of::<Nothing, Idle>();
//! assert_eq!(CONTRACT.reads(), DataSet::EMPTY);
//! assert!(!CONTRACT.emits_commands());
//! ```

use core::marker::PhantomData;

use verbum_ecs::BufferFull;

use crate::access::{Bind, Part};
use crate::contract::Contract;

/// How a caller's world and buffer become the `Parts` an access binds from.
///
/// Written by the caller, once per world: it is the only place that knows which field of
/// the world sits under which category (`Locate`). `parts` is called afresh for every
/// system run, so that a slot taken by one access is free for the next.
pub trait Binding {
    /// The caller's world.
    type World;
    /// The caller's command buffer.
    type Buffer;
    /// The assembly of the world and of the buffer, bound by accesses.
    ///
    /// `where Self: 'w` lets an implementor whose world carries a lifetime (a resource
    /// borrowing bytes owned by the caller) name that lifetime in its `Parts<'w>`: such a
    /// `Parts<'w>` holds `&'w mut` slots of that world, hence needs `'a: 'w`. A binding
    /// type without lifetime parameter is `'static` and satisfies the bound at once.
    type Parts<'w>
    where
        Self: 'w;

    /// Splits `world` into its parts and gives them the `buffer` (under
    /// [`COMMANDS_BINDING`](crate::access::COMMANDS_BINDING)).
    fn parts<'w>(world: &'w mut Self::World, buffer: &'w mut Self::Buffer) -> Self::Parts<'w>;
}

/// A system: a type that names its access and runs on it.
///
/// The access type is the contract (see [`contract_of`]). `run` receives the bound
/// access and reports a full command buffer, the only failure a system can raise.
pub trait System<B: Binding> {
    /// What the system reads, writes, produces and emits, as a typed access.
    ///
    /// `where B: 'w` is required to name `B::Parts<'w>`, itself bounded by `B: 'w`.
    type Access<'w>: Bind<'w, B::Parts<'w>>
    where
        B: 'w;

    /// Runs the system on its bound access.
    fn run(access: Self::Access<'_>) -> Result<(), BufferFull>;
}

/// The contract of system `S`, derived from its access type alone.
///
/// Evaluable in a `const` context. The lifetime of the access is irrelevant to its
/// contract: any lifetime `'a` that `B` outlives names the type, so that a binding type
/// carrying a lifetime is accepted (`'static` would require `B: 'static`).
pub const fn contract_of<'a, B: Binding + 'a, S: System<B>>() -> Contract {
    <S::Access<'a> as Part>::CONTRACT
}

/// Why a system run did not complete.
#[derive(Debug, PartialEq, Eq)]
pub enum RunError {
    /// The access could not be bound: a slot it needs was already taken or is not
    /// provided by the `Parts` of the binding.
    Binding,
    /// The system ran and found the command buffer full.
    Buffer(BufferFull),
}

/// A system and the phase it belongs to, in a [`Cons`] cell.
pub struct At<const PHASE: usize, S>(PhantomData<fn() -> S>);

/// The empty list of systems.
pub struct Nil;

/// A list of systems: the head `H` (an [`At`]) and the tail `T`.
pub struct Cons<H, T>(PhantomData<fn() -> (H, T)>);

/// A static list of systems, in declaration order.
pub trait SystemList<B: Binding> {
    /// Number of systems of the list.
    const LEN: usize;

    /// Writes `(phase, contract)` of each system into `out`, from `offset`, in
    /// declaration order. `out` must hold at least `offset + LEN` slots.
    fn fill(out: &mut [(usize, Contract)], offset: usize);

    /// Binds the access of the system at `index` from a fresh `B::parts` and runs it.
    ///
    /// `index` must be below [`LEN`](Self::LEN): a position out of the list is a caller
    /// error and panics.
    fn run(index: usize, world: &mut B::World, buffer: &mut B::Buffer) -> Result<(), RunError>;
}

impl<B: Binding> SystemList<B> for Nil {
    const LEN: usize = 0;

    fn fill(_out: &mut [(usize, Contract)], _offset: usize) {}

    fn run(index: usize, _world: &mut B::World, _buffer: &mut B::Buffer) -> Result<(), RunError> {
        panic!("system index {index} is out of the list");
    }
}

impl<B: Binding, const PHASE: usize, S: System<B>, T: SystemList<B>> SystemList<B>
    for Cons<At<PHASE, S>, T>
{
    const LEN: usize = 1 + T::LEN;

    fn fill(out: &mut [(usize, Contract)], offset: usize) {
        out[offset] = (PHASE, contract_of::<B, S>());
        T::fill(out, offset + 1);
    }

    fn run(index: usize, world: &mut B::World, buffer: &mut B::Buffer) -> Result<(), RunError> {
        if index > 0 {
            return T::run(index - 1, world, buffer);
        }
        let mut parts = B::parts(world, buffer);
        let access =
            <S::Access<'_> as Bind<'_, B::Parts<'_>>>::bind(&mut parts).ok_or(RunError::Binding)?;
        S::run(access).map_err(RunError::Buffer)
    }
}
