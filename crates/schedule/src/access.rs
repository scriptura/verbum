// crates/schedule/src/access.rs

//! Typed access of a system: what it receives is what it declares (ADR-003,
//! points 1 and 5; I1 to I4, I14).
//!
//! The type of a system's access is the single source of both the data it can
//! reach and its [`Contract`]: no contract is written by hand. Depends on
//! `contract` and `verbum-ecs` only.
//!
//! # Roles
//!
//! - [`Locate`] (implemented by the caller, once per category and per world)
//!   is the only place where a category identifier is tied to a storage or a
//!   resource slot. Its `Store` type is deduced from that implementation and
//!   never restated by a system.
//! - [`Read`], [`Write`] and [`Produce`] are the declarative views. The
//!   identity of a category is the const-generic `ID: u8` of the view, never
//!   the type of the stored value. The field of a view is private: a system
//!   cannot build a view, nor choose the storage behind a category.
//! - [`Commands`] is the typed handle on the command buffer. It is bound through
//!   [`Locate`] under the reserved key [`COMMANDS_BINDING`], which is not a
//!   category. Its contract carries only the structural effects it declares.
//! - [`Part`] (sealed) gives an access type its [`Contract`] as an associated
//!   constant, computed from the type. Tuples of one to four parts, and `()`,
//!   are parts; tuples nest beyond four.
//! - [`Bind`] builds an access from the caller's assembly (`Parts`) through
//!   [`Locate::take`], tuples included. It is sealed by a private supertrait,
//!   so no external code can implement it, whatever the `Parts` type.
//!
//! # Capabilities
//!
//! A view exposes methods only for the store forms this crate supports: a
//! [`ValuesMut`] (components) or a `&mut` [`ResourceSlot`] (resources). They
//! are inherent methods of the view types themselves, constrained to those
//! forms, so an arbitrary `Locate::Store` gives a view with no method at all.
//! `Read` exposes shared borrows only; `Write` and `Produce` expose the value
//! access of their store, and neither adds a read to the contract (D10).
//!
//! # Binding failure
//!
//! [`Bind::bind`] returns `None` when a slot was already taken; it never
//! panics. Tuples bind from left to right, so a failure on a later element
//! leaves the earlier slots consumed: after a failed `bind`, the `Parts` is
//! partially consumed and must be discarded. Binding only moves `Option`s out
//! of the `Parts` and does not touch the world: nothing needs to be rolled
//! back. A fresh `Parts` is built for each binding.
//!
//! # Constant checks
//!
//! `Part::CONTRACT` panics, during constant evaluation, when an identifier is
//! `>= DataSet::CAPACITY` or when a category appears twice in one access (in
//! any mode). Evaluated in a `const` context, that panic is a compile-time
//! error (E0080). [`Bind::bind`] mentions `Self::CONTRACT`, which makes the
//! compiler evaluate it when `bind` is instantiated, so a bad access that is
//! bound fails when the code is built (`cargo build`, `cargo test`), not under
//! `cargo check`: see the doctests of [`Bind`].

use verbum_ecs::{BufferFull, CommandBuffer, EntityId, ResourceSlot, SparseSet, ValuesMut};

use crate::contract::{Contract, StructuralEffects};
use crate::data::DataSet;

mod sealed {
    pub trait Sealed {
        /// Number of command handles ([`super::Commands`]) in this access. Private marker: it
        /// counts handles, not effect bits, so that a second handle is detected whatever its
        /// effects.
        const HANDLES: usize = 0;
    }

    /// Private supertrait of `Bind`: implemented next to each `Bind` impl of this module.
    pub trait Bound<P> {}
}

/// Ties the category `ID` to a storage or a resource slot of the caller's
/// assembly. Implemented by the caller on its `Parts` type, once per category
/// and per world; `'w` is the lifetime of the borrow of the world.
///
/// `take` moves the slot out, once: a second call returns `None`.
///
/// A category with no implementation cannot be declared by a system (the
/// `Bind` bound is not satisfied, E0277):
///
/// ```compile_fail,E0277
/// use verbum_ecs::ResourceSlot;
/// use verbum_schedule::access::{Bind, Locate, Read};
///
/// struct Parts<'w> {
///     slot: Option<&'w mut ResourceSlot<u8>>,
/// }
/// impl<'w> Locate<'w, 0> for Parts<'w> {
///     type Store = &'w mut ResourceSlot<u8>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.slot.take()
///     }
/// }
///
/// let mut slot = ResourceSlot::<u8>::new();
/// let mut parts = Parts { slot: Some(&mut slot) };
/// // Category 5 has no `Locate` for `Parts`.
/// let _: Option<Read<5, &mut ResourceSlot<u8>>> = Bind::bind(&mut parts);
/// ```
///
/// With a declared category it compiles:
///
/// ```
/// use verbum_ecs::ResourceSlot;
/// use verbum_schedule::access::{Bind, Locate, Read};
///
/// struct Parts<'w> {
///     slot: Option<&'w mut ResourceSlot<u8>>,
/// }
/// impl<'w> Locate<'w, 0> for Parts<'w> {
///     type Store = &'w mut ResourceSlot<u8>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.slot.take()
///     }
/// }
///
/// let mut slot = ResourceSlot::<u8>::new();
/// let mut parts = Parts { slot: Some(&mut slot) };
/// let read: Option<Read<0, &mut ResourceSlot<u8>>> = Bind::bind(&mut parts);
/// assert!(read.is_some());
/// ```
///
/// Two implementations for the same `(Parts, ID)` are rejected by coherence
/// (E0119): an identifier cannot designate two storages.
///
/// ```compile_fail,E0119
/// use verbum_ecs::ResourceSlot;
/// use verbum_schedule::access::Locate;
///
/// struct Parts<'w> {
///     first: Option<&'w mut ResourceSlot<u8>>,
///     second: Option<&'w mut ResourceSlot<u8>>,
/// }
/// impl<'w> Locate<'w, 0> for Parts<'w> {
///     type Store = &'w mut ResourceSlot<u8>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.first.take()
///     }
/// }
/// impl<'w> Locate<'w, 0> for Parts<'w> {
///     type Store = &'w mut ResourceSlot<u8>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.second.take()
///     }
/// }
/// ```
///
/// Distinct identifiers coexist:
///
/// ```
/// use verbum_ecs::ResourceSlot;
/// use verbum_schedule::access::Locate;
///
/// struct Parts<'w> {
///     first: Option<&'w mut ResourceSlot<u8>>,
///     second: Option<&'w mut ResourceSlot<u8>>,
/// }
/// impl<'w> Locate<'w, 0> for Parts<'w> {
///     type Store = &'w mut ResourceSlot<u8>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.first.take()
///     }
/// }
/// impl<'w> Locate<'w, 1> for Parts<'w> {
///     type Store = &'w mut ResourceSlot<u8>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.second.take()
///     }
/// }
/// ```
pub trait Locate<'w, const ID: u8> {
    /// What the category designates: a [`ValuesMut`] or a `&mut`
    /// [`ResourceSlot`] in the forms supported by [`Read`], [`Write`] and
    /// [`Produce`].
    type Store;

    /// Moves the slot out of the assembly. `None` if it was already taken.
    fn take(&mut self) -> Option<Self::Store>;
}

/// An access type that has a [`Contract`], computed from the type itself.
///
/// Sealed: only the views of this module and tuples of parts are parts, so
/// no contract can be declared apart from the access it describes.
///
/// ```compile_fail,E0277
/// use verbum_schedule::access::Part;
/// use verbum_schedule::contract::{Contract, StructuralEffects};
/// use verbum_schedule::data::DataSet;
///
/// struct Mine;
/// impl Part for Mine {
///     const CONTRACT: Contract = Contract::new(
///         DataSet::EMPTY,
///         DataSet::EMPTY,
///         DataSet::EMPTY,
///         StructuralEffects::NONE,
///     );
/// }
/// ```
///
/// An access cannot reach the world by being a tuple that holds it: a tuple
/// is a part only if each of its elements is.
///
/// ```compile_fail,E0277
/// use verbum_ecs::World;
/// use verbum_schedule::access::{Part, Read};
///
/// fn declared<A: Part>() {}
/// declared::<(Read<0, ()>, &mut World<1>)>();
/// ```
///
/// Twin without the world:
///
/// ```
/// use verbum_schedule::access::{Part, Read, Write};
///
/// fn declared<A: Part>() {}
/// declared::<(Read<0, ()>, Write<1, ()>)>();
/// ```
///
/// An identifier outside the capacity of a `DataSet` is a compile-time error
/// when the contract is evaluated in a `const` context (E0080):
///
/// ```compile_fail,E0080
/// use verbum_schedule::access::{Part, Read};
/// use verbum_schedule::contract::Contract;
///
/// const _: Contract = <Read<64, ()> as Part>::CONTRACT;
/// ```
///
/// Twin on the last valid identifier:
///
/// ```
/// use verbum_schedule::access::{Part, Read};
/// use verbum_schedule::contract::Contract;
///
/// const _: Contract = <Read<63, ()> as Part>::CONTRACT;
/// ```
///
/// A category declared twice in one access, in any mode, is the same error
/// (E0080), however deeply the access is nested:
///
/// ```compile_fail,E0080
/// use verbum_schedule::access::{Part, Read, Write};
/// use verbum_schedule::contract::Contract;
///
/// const _: Contract = <(Read<1, ()>, Write<1, ()>) as Part>::CONTRACT;
/// ```
///
/// ```compile_fail,E0080
/// use verbum_schedule::access::{Part, Produce, Read};
/// use verbum_schedule::contract::Contract;
///
/// const _: Contract = <((Read<1, ()>,), (Read<2, ()>, Produce<1, ()>)) as Part>::CONTRACT;
/// ```
///
/// Twin with distinct categories:
///
/// ```
/// use verbum_schedule::access::{Part, Produce, Read, Write};
/// use verbum_schedule::contract::Contract;
///
/// const _: Contract = <(Read<1, ()>, Write<2, ()>, ((Produce<3, ()>,),)) as Part>::CONTRACT;
/// ```
pub trait Part: sealed::Sealed {
    /// The contract of this access: pure data, computed in `const`.
    const CONTRACT: Contract;
}

/// Builds an access from the caller's assembly `P`, by [`Locate::take`].
///
/// Sealed: it has a private supertrait, implemented only next to the
/// implementations of this module, which exist exactly where the assembly
/// implements [`Locate`] for each identifier of the access. The private fields
/// of the views are a separate protection and do not replace this one.
///
/// An external implementation for one of our views and a local `Parts`
/// without `Locate` is rejected by the sealing (E0277, reported by the
/// compiler as a sealed trait), so `Locate` stays the only way to bind:
///
/// ```compile_fail,E0277
/// use verbum_ecs::ResourceSlot;
/// use verbum_schedule::access::{Bind, Read};
///
/// struct SonParts<'w> {
///     slot: Option<&'w mut ResourceSlot<u8>>,
/// }
/// impl<'w> Bind<'w, SonParts<'w>> for Read<5, &'w mut ResourceSlot<u8>> {
///     fn bind(_parts: &mut SonParts<'w>) -> Option<Self> {
///         None
///     }
/// }
/// ```
///
/// Twin: the same `Parts` with the same view, bound through `Locate`, which
/// is what the sealing leaves available. Only the `Bind` implementation is
/// removed, so it is that implementation that the sealing rejects above:
///
/// ```
/// use verbum_ecs::ResourceSlot;
/// use verbum_schedule::access::{Bind, Locate, Read};
///
/// struct SonParts<'w> {
///     slot: Option<&'w mut ResourceSlot<u8>>,
/// }
/// impl<'w> Locate<'w, 5> for SonParts<'w> {
///     type Store = &'w mut ResourceSlot<u8>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.slot.take()
///     }
/// }
///
/// let mut slot = ResourceSlot::<u8>::new();
/// let mut parts = SonParts { slot: Some(&mut slot) };
/// let read: Option<Read<5, &mut ResourceSlot<u8>>> = Bind::bind(&mut parts);
/// assert!(read.is_some());
/// ```
///
/// When the assembly does implement `Locate`, an external implementation
/// conflicts with ours by coherence (E0119):
///
/// ```compile_fail,E0119
/// use verbum_ecs::ResourceSlot;
/// use verbum_schedule::access::{Bind, Locate, Read};
///
/// struct SonParts<'w> {
///     slot: Option<&'w mut ResourceSlot<u8>>,
/// }
/// impl<'w> Locate<'w, 5> for SonParts<'w> {
///     type Store = &'w mut ResourceSlot<u8>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.slot.take()
///     }
/// }
/// impl<'w> Bind<'w, SonParts<'w>> for Read<5, &'w mut ResourceSlot<u8>> {
///     fn bind(_parts: &mut SonParts<'w>) -> Option<Self> {
///         None
///     }
/// }
/// ```
///
/// The `Store` of every view of the access must be
/// the one `P` declares for its identifier; otherwise the projection
/// `<P as Locate<ID>>::Store` does not match and the access does not bind
/// (E0271):
///
/// ```compile_fail,E0271
/// use verbum_ecs::ResourceSlot;
/// use verbum_schedule::access::{Bind, Locate, Read};
///
/// struct Parts<'w> {
///     slot: Option<&'w mut ResourceSlot<u8>>,
/// }
/// impl<'w> Locate<'w, 0> for Parts<'w> {
///     type Store = &'w mut ResourceSlot<u8>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.slot.take()
///     }
/// }
///
/// let mut slot = ResourceSlot::<u8>::new();
/// let mut parts = Parts { slot: Some(&mut slot) };
/// // `Locate<0>::Store` holds a `u8` slot, not a `u16` one.
/// let _: Option<Read<0, &mut ResourceSlot<u16>>> = Bind::bind(&mut parts);
/// ```
///
/// Twin with the declared store:
///
/// ```
/// use verbum_ecs::ResourceSlot;
/// use verbum_schedule::access::{Bind, Locate, Read};
///
/// struct Parts<'w> {
///     slot: Option<&'w mut ResourceSlot<u8>>,
/// }
/// impl<'w> Locate<'w, 0> for Parts<'w> {
///     type Store = &'w mut ResourceSlot<u8>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.slot.take()
///     }
/// }
///
/// let mut slot = ResourceSlot::<u8>::new();
/// let mut parts = Parts { slot: Some(&mut slot) };
/// let _: Option<Read<0, &mut ResourceSlot<u8>>> = Bind::bind(&mut parts);
/// ```
///
/// Instantiating `bind` is enough to evaluate the constant checks of the
/// contract, with no explicit `const` item: an out-of-range identifier that
/// is bound is rejected (E0080) when the code is built (`cargo build`,
/// `cargo test`, doctests). `cargo check` does not instantiate `bind` and
/// does not report it.
///
/// ```compile_fail,E0080
/// use verbum_ecs::ResourceSlot;
/// use verbum_schedule::access::{Bind, Locate, Read};
///
/// struct Parts<'w> {
///     slot: Option<&'w mut ResourceSlot<u8>>,
/// }
/// impl<'w> Locate<'w, 64> for Parts<'w> {
///     type Store = &'w mut ResourceSlot<u8>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.slot.take()
///     }
/// }
///
/// let mut slot = ResourceSlot::<u8>::new();
/// let mut parts = Parts { slot: Some(&mut slot) };
/// let _: Option<Read<64, &mut ResourceSlot<u8>>> = Bind::bind(&mut parts);
/// ```
///
/// Twin on the last valid identifier:
///
/// ```
/// use verbum_ecs::ResourceSlot;
/// use verbum_schedule::access::{Bind, Locate, Read};
///
/// struct Parts<'w> {
///     slot: Option<&'w mut ResourceSlot<u8>>,
/// }
/// impl<'w> Locate<'w, 63> for Parts<'w> {
///     type Store = &'w mut ResourceSlot<u8>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.slot.take()
///     }
/// }
///
/// let mut slot = ResourceSlot::<u8>::new();
/// let mut parts = Parts { slot: Some(&mut slot) };
/// let read: Option<Read<63, &mut ResourceSlot<u8>>> = Bind::bind(&mut parts);
/// assert!(read.is_some());
/// ```
pub trait Bind<'w, P>: Part + sealed::Bound<P> + Sized {
    /// Takes every slot of the access out of `parts`, left to right.
    ///
    /// `None` if one of them was already taken. On `None`, `parts` may be
    /// partially consumed and must be discarded; the world is untouched.
    fn bind(parts: &mut P) -> Option<Self>;
}

/// Read access to category `ID`. Exposes shared borrows only.
///
/// The field is private: a view is built by [`Bind::bind`] only (E0423):
///
/// ```compile_fail,E0423
/// use verbum_schedule::access::Read;
///
/// let _read: Read<0, ()> = Read(());
/// ```
///
/// A `Read` over components gives no way to change them (E0596, E0594):
///
/// ```compile_fail,E0596
/// use verbum_ecs::{SparseSet, ValuesMut, World};
/// use verbum_schedule::access::{Bind, Locate, Read};
///
/// struct Parts<'w> {
///     values: Option<ValuesMut<'w, u8, 1, 1>>,
/// }
/// impl<'w> Locate<'w, 0> for Parts<'w> {
///     type Store = ValuesMut<'w, u8, 1, 1>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.values.take()
///     }
/// }
///
/// let mut set = SparseSet::<u8, 1, 1>::new();
/// let entity = World::<1>::new().spawn().unwrap();
/// let mut parts = Parts { values: Some(set.values_mut()) };
/// let read: Read<0, ValuesMut<u8, 1, 1>> = Bind::bind(&mut parts).unwrap();
/// let _ = read.shared().insert(entity, 1);
/// ```
///
/// ```compile_fail,E0594
/// use verbum_ecs::{SparseSet, ValuesMut, World};
/// use verbum_schedule::access::{Bind, Locate, Read};
///
/// struct Parts<'w> {
///     values: Option<ValuesMut<'w, u8, 1, 1>>,
/// }
/// impl<'w> Locate<'w, 0> for Parts<'w> {
///     type Store = ValuesMut<'w, u8, 1, 1>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.values.take()
///     }
/// }
///
/// let mut set = SparseSet::<u8, 1, 1>::new();
/// let entity = World::<1>::new().spawn().unwrap();
/// let mut parts = Parts { values: Some(set.values_mut()) };
/// let read: Read<0, ValuesMut<u8, 1, 1>> = Bind::bind(&mut parts).unwrap();
/// *read.get(entity).unwrap() = 1;
/// ```
///
/// Twin that only reads:
///
/// ```
/// use verbum_ecs::{SparseSet, ValuesMut, World};
/// use verbum_schedule::access::{Bind, Locate, Read};
///
/// struct Parts<'w> {
///     values: Option<ValuesMut<'w, u8, 1, 1>>,
/// }
/// impl<'w> Locate<'w, 0> for Parts<'w> {
///     type Store = ValuesMut<'w, u8, 1, 1>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.values.take()
///     }
/// }
///
/// let mut set = SparseSet::<u8, 1, 1>::new();
/// let entity = World::<1>::new().spawn().unwrap();
/// let mut parts = Parts { values: Some(set.values_mut()) };
/// let read: Read<0, ValuesMut<u8, 1, 1>> = Bind::bind(&mut parts).unwrap();
/// assert!(!read.shared().contains(entity));
/// assert_eq!(read.get(entity), None);
/// ```
///
/// A `Read` is not a `Write` (E0308):
///
/// ```compile_fail,E0308
/// use verbum_ecs::{SparseSet, ValuesMut};
/// use verbum_schedule::access::{Bind, Locate, Read, Write};
///
/// struct Parts<'w> {
///     values: Option<ValuesMut<'w, u8, 1, 1>>,
/// }
/// impl<'w> Locate<'w, 0> for Parts<'w> {
///     type Store = ValuesMut<'w, u8, 1, 1>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.values.take()
///     }
/// }
///
/// let mut set = SparseSet::<u8, 1, 1>::new();
/// let mut parts = Parts { values: Some(set.values_mut()) };
/// let read: Read<0, ValuesMut<u8, 1, 1>> = Bind::bind(&mut parts).unwrap();
/// let _write: Write<0, ValuesMut<u8, 1, 1>> = read;
/// ```
///
/// Twin that keeps the mode:
///
/// ```
/// use verbum_ecs::{SparseSet, ValuesMut};
/// use verbum_schedule::access::{Bind, Locate, Read};
///
/// struct Parts<'w> {
///     values: Option<ValuesMut<'w, u8, 1, 1>>,
/// }
/// impl<'w> Locate<'w, 0> for Parts<'w> {
///     type Store = ValuesMut<'w, u8, 1, 1>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.values.take()
///     }
/// }
///
/// let mut set = SparseSet::<u8, 1, 1>::new();
/// let mut parts = Parts { values: Some(set.values_mut()) };
/// let read: Read<0, ValuesMut<u8, 1, 1>> = Bind::bind(&mut parts).unwrap();
/// let _same: Read<0, ValuesMut<u8, 1, 1>> = read;
/// ```
///
/// A store that is neither a [`ValuesMut`] nor a `&mut` [`ResourceSlot`] gets
/// a view with no capability (E0599):
///
/// ```compile_fail,E0599
/// use verbum_schedule::access::{Bind, Locate, Read};
///
/// struct Parts {
///     number: Option<u32>,
/// }
/// impl<'w> Locate<'w, 0> for Parts {
///     type Store = u32;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.number.take()
///     }
/// }
///
/// let mut parts = Parts { number: Some(1) };
/// let read: Read<0, u32> = Bind::bind(&mut parts).unwrap();
/// let _ = read.get();
/// ```
pub struct Read<const ID: u8, V>(V);

/// Write access to category `ID`: a persistent category with a single owner
/// per phase (I9). Exposes the value access of its store; it adds no read to
/// the contract (D10).
///
/// `into_values` hands over the existing [`ValuesMut`] without copying, to
/// build a [`verbum_ecs::ReadWriteQuery`].
pub struct Write<const ID: u8, V>(V);

/// Production into category `ID`: a transient category, freely produced (I9).
/// Exposes the value access of its store; it adds no read to the contract
/// (D10).
pub struct Produce<const ID: u8, V>(V);

// ---------------------------------------------------------------------------
// Contracts
// ---------------------------------------------------------------------------

/// Every category used by `contract`, in any mode.
const fn categories(contract: Contract) -> DataSet {
    contract
        .reads()
        .union(contract.writes())
        .union(contract.produces())
}

/// The structural commands of either contract.
const fn either_effects(a: StructuralEffects, b: StructuralEffects) -> StructuralEffects {
    StructuralEffects::new(
        a.spawns() || b.spawns(),
        a.destroys() || b.destroys(),
        a.adds_component() || b.adds_component(),
        a.removes_component() || b.removes_component(),
    )
}

/// Merges the contracts of two accesses that share no category, in any mode.
///
/// Panics, in the evaluation of an associated constant, when a category
/// appears in both: a compile-time error (E0080) in a `const` context.
/// Structural effects are or-ed; the rule on duplicated effects belongs to
/// the command handle.
const fn merge_disjoint(a: Contract, b: Contract) -> Contract {
    assert!(
        !categories(a).intersects(categories(b)),
        "a data category is declared twice in one access"
    );
    Contract::new(
        a.reads().union(b.reads()),
        a.writes().union(b.writes()),
        a.produces().union(b.produces()),
        either_effects(a.effects(), b.effects()),
    )
}

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

macro_rules! view {
    ($view:ident, reads: $reads:expr, writes: $writes:expr, produces: $produces:expr) => {
        impl<const ID: u8, V> sealed::Sealed for $view<ID, V> {}

        impl<const ID: u8, V> Part for $view<ID, V> {
            const CONTRACT: Contract =
                Contract::new($reads, $writes, $produces, StructuralEffects::NONE);
        }

        impl<'w, P, const ID: u8> sealed::Bound<P> for $view<ID, <P as Locate<'w, ID>>::Store> where
            P: Locate<'w, ID>
        {
        }

        impl<'w, P, const ID: u8> Bind<'w, P> for $view<ID, <P as Locate<'w, ID>>::Store>
        where
            P: Locate<'w, ID>,
        {
            fn bind(parts: &mut P) -> Option<Self> {
                // Mentioned so that the constant checks of `CONTRACT` are evaluated
                // when this function is instantiated.
                let _ = Self::CONTRACT;
                parts.take().map($view)
            }
        }
    };
}

view!(Read, reads: DataSet::of(ID), writes: DataSet::EMPTY, produces: DataSet::EMPTY);
view!(Write, reads: DataSet::EMPTY, writes: DataSet::of(ID), produces: DataSet::EMPTY);
view!(Produce, reads: DataSet::EMPTY, writes: DataSet::EMPTY, produces: DataSet::of(ID));

impl<'w, const ID: u8, T: Copy, const SPARSE: usize, const DENSE: usize>
    Read<ID, ValuesMut<'w, T, SPARSE, DENSE>>
{
    /// Number of entities holding the component.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether no entity holds the component.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Whether `entity` holds the component.
    pub fn contains(&self, entity: EntityId) -> bool {
        self.0.contains(entity)
    }

    /// The component of `entity`, if present.
    pub fn get(&self, entity: EntityId) -> Option<&T> {
        self.0.get(entity)
    }

    /// The entities holding the component, in dense order.
    pub fn entities(&self) -> &[EntityId] {
        self.0.entities()
    }

    /// The storage, as a shared borrow: what a query reads.
    pub fn shared(&self) -> &SparseSet<T, SPARSE, DENSE> {
        self.0.shared()
    }
}

impl<const ID: u8, T> Read<ID, &mut ResourceSlot<T>> {
    /// The resource, if the slot holds one.
    pub fn get(&self) -> Option<&T> {
        self.0.get()
    }
}

macro_rules! writable_view {
    ($view:ident) => {
        impl<'w, const ID: u8, T: Copy, const SPARSE: usize, const DENSE: usize>
            $view<ID, ValuesMut<'w, T, SPARSE, DENSE>>
        {
            /// Number of entities holding the component.
            pub fn len(&self) -> usize {
                self.0.len()
            }

            /// Whether no entity holds the component.
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            /// Whether `entity` holds the component.
            pub fn contains(&self, entity: EntityId) -> bool {
                self.0.contains(entity)
            }

            /// The component of `entity`, if present.
            pub fn get(&self, entity: EntityId) -> Option<&T> {
                self.0.get(entity)
            }

            /// The component of `entity` for modification, if present. The
            /// value changes; presence and order do not.
            pub fn get_mut(&mut self, entity: EntityId) -> Option<&mut T> {
                self.0.get_mut(entity)
            }

            /// The entities holding the component, in dense order.
            pub fn entities(&self) -> &[EntityId] {
                self.0.entities()
            }
        }

        impl<const ID: u8, T> $view<ID, &mut ResourceSlot<T>> {
            /// The resource, if the slot holds one.
            pub fn get(&self) -> Option<&T> {
                self.0.get()
            }

            /// The resource for modification, if the slot holds one.
            pub fn get_mut(&mut self) -> Option<&mut T> {
                self.0.get_mut()
            }

            /// Creates the resource, or replaces it and returns the previous
            /// one. Not a structural operation.
            pub fn insert(&mut self, value: T) -> Option<T> {
                self.0.insert(value)
            }
        }
    };
}

writable_view!(Write);
writable_view!(Produce);

impl<'w, const ID: u8, T, const SPARSE: usize, const DENSE: usize>
    Write<ID, ValuesMut<'w, T, SPARSE, DENSE>>
{
    /// Hands over the existing view, to build a [`verbum_ecs::ReadWriteQuery`].
    /// No copy, no reconstruction.
    pub fn into_values(self) -> ValuesMut<'w, T, SPARSE, DENSE> {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Command handle
// ---------------------------------------------------------------------------

/// Binding key of the command buffer under [`Locate`]: `Locate<'w,
/// COMMANDS_BINDING>` is implemented by the caller on its `Parts` to give the
/// buffer to [`Commands`].
///
/// **This is a binding key only.** It is not a data category: it is outside
/// the identifiers of a `DataSet` (`0..DataSet::CAPACITY`), it is never turned
/// into a `DataId`, it never appears in `reads`, `writes` or `produces`, and it
/// creates no ordering dependency. An ordinary view using it as a category is
/// rejected by the constant check of its contract (E0080):
///
/// ```compile_fail,E0080
/// use verbum_schedule::access::{COMMANDS_BINDING, Part, Read};
/// use verbum_schedule::contract::Contract;
///
/// const _: Contract = <Read<COMMANDS_BINDING, ()> as Part>::CONTRACT;
/// ```
///
/// Twin on the last valid category:
///
/// ```
/// use verbum_schedule::access::{Part, Read};
/// use verbum_schedule::contract::Contract;
///
/// const _: Contract = <Read<63, ()> as Part>::CONTRACT;
/// ```
pub const COMMANDS_BINDING: u8 = u8::MAX;

/// Typed handle on the command buffer: it emits the structural commands the
/// four indicators declare, and nothing else.
///
/// The indicators (`SPAWN`, `DESPAWN`, `ADD`, `REMOVE`) are the `effects` of
/// the contract of the access; the contract carries no data category, and
/// commands produce no ordering edge between systems (ADR-006, point 4). The
/// field is private: a handle is built by [`Bind::bind`] only. A handle with no
/// indicator is legal: it has no effect and no method, but it still counts as
/// the one command handle an access may hold.
///
/// Only a store that is a `&mut` [`CommandBuffer`] exposes methods, each one
/// available only when its indicator is `true`. Emitting never grows the
/// buffer: a full buffer is `Err(BufferFull)`.
///
/// Without the indicator, the method does not exist (E0599): `spawn` needs `SPAWN`.
///
/// ```compile_fail,E0599
/// # use verbum_ecs::{CommandBuffer, World};
/// # use verbum_schedule::access::{Bind, COMMANDS_BINDING, Commands, Locate};
/// # struct Parts<'w> {
/// #     buffer: Option<&'w mut CommandBuffer<2, u8, u8>>,
/// # }
/// # impl<'w> Locate<'w, COMMANDS_BINDING> for Parts<'w> {
/// #     type Store = &'w mut CommandBuffer<2, u8, u8>;
/// #     fn take(&mut self) -> Option<Self::Store> {
/// #         self.buffer.take()
/// #     }
/// # }
/// # let mut buffer = CommandBuffer::<2, u8, u8>::new();
/// # let entity = World::<1>::new().spawn().unwrap();
/// # let mut parts = Parts { buffer: Some(&mut buffer) };
/// let mut commands: Commands<&mut CommandBuffer<2, u8, u8>, false, true, true, true> =
///     Bind::bind(&mut parts).unwrap();
/// commands.spawn().unwrap();
/// ```
///
/// Twin with the indicator:
///
/// ```
/// # use verbum_ecs::{CommandBuffer, World};
/// # use verbum_schedule::access::{Bind, COMMANDS_BINDING, Commands, Locate};
/// # struct Parts<'w> {
/// #     buffer: Option<&'w mut CommandBuffer<2, u8, u8>>,
/// # }
/// # impl<'w> Locate<'w, COMMANDS_BINDING> for Parts<'w> {
/// #     type Store = &'w mut CommandBuffer<2, u8, u8>;
/// #     fn take(&mut self) -> Option<Self::Store> {
/// #         self.buffer.take()
/// #     }
/// # }
/// # let mut buffer = CommandBuffer::<2, u8, u8>::new();
/// # let entity = World::<1>::new().spawn().unwrap();
/// # let mut parts = Parts { buffer: Some(&mut buffer) };
/// let mut commands: Commands<&mut CommandBuffer<2, u8, u8>, true, true, true, true> =
///     Bind::bind(&mut parts).unwrap();
/// commands.spawn().unwrap();
/// ```
///
/// Without the indicator, the method does not exist (E0599): `despawn` needs `DESPAWN`.
///
/// ```compile_fail,E0599
/// # use verbum_ecs::{CommandBuffer, World};
/// # use verbum_schedule::access::{Bind, COMMANDS_BINDING, Commands, Locate};
/// # struct Parts<'w> {
/// #     buffer: Option<&'w mut CommandBuffer<2, u8, u8>>,
/// # }
/// # impl<'w> Locate<'w, COMMANDS_BINDING> for Parts<'w> {
/// #     type Store = &'w mut CommandBuffer<2, u8, u8>;
/// #     fn take(&mut self) -> Option<Self::Store> {
/// #         self.buffer.take()
/// #     }
/// # }
/// # let mut buffer = CommandBuffer::<2, u8, u8>::new();
/// # let entity = World::<1>::new().spawn().unwrap();
/// # let mut parts = Parts { buffer: Some(&mut buffer) };
/// let mut commands: Commands<&mut CommandBuffer<2, u8, u8>, true, false, true, true> =
///     Bind::bind(&mut parts).unwrap();
/// commands.despawn(entity).unwrap();
/// ```
///
/// Twin with the indicator:
///
/// ```
/// # use verbum_ecs::{CommandBuffer, World};
/// # use verbum_schedule::access::{Bind, COMMANDS_BINDING, Commands, Locate};
/// # struct Parts<'w> {
/// #     buffer: Option<&'w mut CommandBuffer<2, u8, u8>>,
/// # }
/// # impl<'w> Locate<'w, COMMANDS_BINDING> for Parts<'w> {
/// #     type Store = &'w mut CommandBuffer<2, u8, u8>;
/// #     fn take(&mut self) -> Option<Self::Store> {
/// #         self.buffer.take()
/// #     }
/// # }
/// # let mut buffer = CommandBuffer::<2, u8, u8>::new();
/// # let entity = World::<1>::new().spawn().unwrap();
/// # let mut parts = Parts { buffer: Some(&mut buffer) };
/// let mut commands: Commands<&mut CommandBuffer<2, u8, u8>, true, true, true, true> =
///     Bind::bind(&mut parts).unwrap();
/// commands.despawn(entity).unwrap();
/// ```
///
/// Without the indicator, the method does not exist (E0599): `add_component` needs `ADD`.
///
/// ```compile_fail,E0599
/// # use verbum_ecs::{CommandBuffer, World};
/// # use verbum_schedule::access::{Bind, COMMANDS_BINDING, Commands, Locate};
/// # struct Parts<'w> {
/// #     buffer: Option<&'w mut CommandBuffer<2, u8, u8>>,
/// # }
/// # impl<'w> Locate<'w, COMMANDS_BINDING> for Parts<'w> {
/// #     type Store = &'w mut CommandBuffer<2, u8, u8>;
/// #     fn take(&mut self) -> Option<Self::Store> {
/// #         self.buffer.take()
/// #     }
/// # }
/// # let mut buffer = CommandBuffer::<2, u8, u8>::new();
/// # let entity = World::<1>::new().spawn().unwrap();
/// # let mut parts = Parts { buffer: Some(&mut buffer) };
/// let mut commands: Commands<&mut CommandBuffer<2, u8, u8>, true, true, false, true> =
///     Bind::bind(&mut parts).unwrap();
/// commands.add_component(entity, 1).unwrap();
/// ```
///
/// Twin with the indicator:
///
/// ```
/// # use verbum_ecs::{CommandBuffer, World};
/// # use verbum_schedule::access::{Bind, COMMANDS_BINDING, Commands, Locate};
/// # struct Parts<'w> {
/// #     buffer: Option<&'w mut CommandBuffer<2, u8, u8>>,
/// # }
/// # impl<'w> Locate<'w, COMMANDS_BINDING> for Parts<'w> {
/// #     type Store = &'w mut CommandBuffer<2, u8, u8>;
/// #     fn take(&mut self) -> Option<Self::Store> {
/// #         self.buffer.take()
/// #     }
/// # }
/// # let mut buffer = CommandBuffer::<2, u8, u8>::new();
/// # let entity = World::<1>::new().spawn().unwrap();
/// # let mut parts = Parts { buffer: Some(&mut buffer) };
/// let mut commands: Commands<&mut CommandBuffer<2, u8, u8>, true, true, true, true> =
///     Bind::bind(&mut parts).unwrap();
/// commands.add_component(entity, 1).unwrap();
/// ```
///
/// Without the indicator, the method does not exist (E0599): `remove_component` needs `REMOVE`.
///
/// ```compile_fail,E0599
/// # use verbum_ecs::{CommandBuffer, World};
/// # use verbum_schedule::access::{Bind, COMMANDS_BINDING, Commands, Locate};
/// # struct Parts<'w> {
/// #     buffer: Option<&'w mut CommandBuffer<2, u8, u8>>,
/// # }
/// # impl<'w> Locate<'w, COMMANDS_BINDING> for Parts<'w> {
/// #     type Store = &'w mut CommandBuffer<2, u8, u8>;
/// #     fn take(&mut self) -> Option<Self::Store> {
/// #         self.buffer.take()
/// #     }
/// # }
/// # let mut buffer = CommandBuffer::<2, u8, u8>::new();
/// # let entity = World::<1>::new().spawn().unwrap();
/// # let mut parts = Parts { buffer: Some(&mut buffer) };
/// let mut commands: Commands<&mut CommandBuffer<2, u8, u8>, true, true, true, false> =
///     Bind::bind(&mut parts).unwrap();
/// commands.remove_component(entity, 1).unwrap();
/// ```
///
/// Twin with the indicator:
///
/// ```
/// # use verbum_ecs::{CommandBuffer, World};
/// # use verbum_schedule::access::{Bind, COMMANDS_BINDING, Commands, Locate};
/// # struct Parts<'w> {
/// #     buffer: Option<&'w mut CommandBuffer<2, u8, u8>>,
/// # }
/// # impl<'w> Locate<'w, COMMANDS_BINDING> for Parts<'w> {
/// #     type Store = &'w mut CommandBuffer<2, u8, u8>;
/// #     fn take(&mut self) -> Option<Self::Store> {
/// #         self.buffer.take()
/// #     }
/// # }
/// # let mut buffer = CommandBuffer::<2, u8, u8>::new();
/// # let entity = World::<1>::new().spawn().unwrap();
/// # let mut parts = Parts { buffer: Some(&mut buffer) };
/// let mut commands: Commands<&mut CommandBuffer<2, u8, u8>, true, true, true, true> =
///     Bind::bind(&mut parts).unwrap();
/// commands.remove_component(entity, 1).unwrap();
/// ```
///
/// An access holds at most one command handle, whatever the effects of each
/// one (E0080); the buffer is unique:
///
/// ```compile_fail,E0080
/// use verbum_schedule::access::{Commands, Part};
/// use verbum_schedule::contract::Contract;
///
/// const _: Contract = <(
///     Commands<(), true, false, false, false>,
///     Commands<(), false, true, false, false>,
/// ) as Part>::CONTRACT;
/// ```
///
/// ```compile_fail,E0080
/// use verbum_schedule::access::{Commands, Part};
/// use verbum_schedule::contract::Contract;
///
/// const _: Contract = <(
///     Commands<(), true, true, false, false>,
///     Commands<(), true, false, false, false>,
/// ) as Part>::CONTRACT;
/// ```
///
/// ```compile_fail,E0080
/// use verbum_schedule::access::{Commands, Part, Read};
/// use verbum_schedule::contract::Contract;
///
/// const _: Contract = <(
///     (Commands<(), true, false, false, false>,),
///     (Read<1, ()>, Commands<(), false, false, false, true>),
/// ) as Part>::CONTRACT;
/// ```
///
/// ```compile_fail,E0080
/// use verbum_schedule::access::{Commands, Part};
/// use verbum_schedule::contract::Contract;
///
/// const _: Contract = <(
///     Commands<(), false, false, false, false>,
///     Commands<(), false, false, false, false>,
/// ) as Part>::CONTRACT;
/// ```
///
/// Twin with a single handle among data views:
///
/// ```
/// use verbum_schedule::access::{Commands, Part, Read, Write};
/// use verbum_schedule::contract::Contract;
///
/// const _: Contract = <(
///     Read<1, ()>,
///     (Commands<(), true, false, false, true>,),
///     Write<2, ()>,
/// ) as Part>::CONTRACT;
/// ```
///
/// `Bind` is sealed for the handle too: an external implementation with a
/// local `Parts` that has no `Locate` for the key is rejected (E0277):
///
/// ```compile_fail,E0277
/// use verbum_ecs::CommandBuffer;
/// use verbum_schedule::access::{Bind, Commands};
///
/// struct SonParts<'w> {
///     buffer: Option<&'w mut CommandBuffer<1>>,
/// }
/// impl<'w> Bind<'w, SonParts<'w>>
///     for Commands<&'w mut CommandBuffer<1>, true, false, false, false>
/// {
///     fn bind(_parts: &mut SonParts<'w>) -> Option<Self> {
///         None
///     }
/// }
/// ```
///
/// Twin: the same `Parts`, bound through `Locate` under the key:
///
/// ```
/// use verbum_ecs::CommandBuffer;
/// use verbum_schedule::access::{Bind, COMMANDS_BINDING, Commands, Locate};
///
/// struct SonParts<'w> {
///     buffer: Option<&'w mut CommandBuffer<1>>,
/// }
/// impl<'w> Locate<'w, COMMANDS_BINDING> for SonParts<'w> {
///     type Store = &'w mut CommandBuffer<1>;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.buffer.take()
///     }
/// }
///
/// let mut buffer = CommandBuffer::<1>::new();
/// let mut parts = SonParts { buffer: Some(&mut buffer) };
/// let commands: Option<Commands<&mut CommandBuffer<1>, true, false, false, false>> =
///     Bind::bind(&mut parts);
/// assert!(commands.is_some());
/// ```
///
/// A store that is not a command buffer gives a handle with no method (E0599):
///
/// ```compile_fail,E0599
/// use verbum_schedule::access::{Bind, COMMANDS_BINDING, Commands, Locate};
///
/// struct Parts {
///     number: Option<u32>,
/// }
/// impl<'w> Locate<'w, COMMANDS_BINDING> for Parts {
///     type Store = u32;
///     fn take(&mut self) -> Option<Self::Store> {
///         self.number.take()
///     }
/// }
///
/// let mut parts = Parts { number: Some(1) };
/// let mut commands: Commands<u32, true, true, true, true> = Bind::bind(&mut parts).unwrap();
/// let _ = commands.spawn();
/// ```
pub struct Commands<V, const SPAWN: bool, const DESPAWN: bool, const ADD: bool, const REMOVE: bool>(
    V,
);

impl<V, const SPAWN: bool, const DESPAWN: bool, const ADD: bool, const REMOVE: bool> sealed::Sealed
    for Commands<V, SPAWN, DESPAWN, ADD, REMOVE>
{
    const HANDLES: usize = 1;
}

impl<V, const SPAWN: bool, const DESPAWN: bool, const ADD: bool, const REMOVE: bool> Part
    for Commands<V, SPAWN, DESPAWN, ADD, REMOVE>
{
    const CONTRACT: Contract = Contract::new(
        DataSet::EMPTY,
        DataSet::EMPTY,
        DataSet::EMPTY,
        StructuralEffects::new(SPAWN, DESPAWN, ADD, REMOVE),
    );
}

impl<'w, P, const SPAWN: bool, const DESPAWN: bool, const ADD: bool, const REMOVE: bool>
    sealed::Bound<P>
    for Commands<<P as Locate<'w, COMMANDS_BINDING>>::Store, SPAWN, DESPAWN, ADD, REMOVE>
where
    P: Locate<'w, COMMANDS_BINDING>,
{
}

impl<'w, P, const SPAWN: bool, const DESPAWN: bool, const ADD: bool, const REMOVE: bool> Bind<'w, P>
    for Commands<<P as Locate<'w, COMMANDS_BINDING>>::Store, SPAWN, DESPAWN, ADD, REMOVE>
where
    P: Locate<'w, COMMANDS_BINDING>,
{
    fn bind(parts: &mut P) -> Option<Self> {
        // See `view!`: evaluates the constant checks of the contract.
        let _ = Self::CONTRACT;
        parts.take().map(Commands)
    }
}

impl<
    const COMMANDS: usize,
    Add: Copy,
    Remove: Copy,
    const DESPAWN: bool,
    const ADD: bool,
    const REMOVE: bool,
> Commands<&mut CommandBuffer<COMMANDS, Add, Remove>, true, DESPAWN, ADD, REMOVE>
{
    /// Emits the creation of an entity. `Err(BufferFull)` if the buffer is full.
    pub fn spawn(&mut self) -> Result<(), BufferFull> {
        self.0.spawn()
    }
}

impl<
    const COMMANDS: usize,
    Add: Copy,
    Remove: Copy,
    const SPAWN: bool,
    const ADD: bool,
    const REMOVE: bool,
> Commands<&mut CommandBuffer<COMMANDS, Add, Remove>, SPAWN, true, ADD, REMOVE>
{
    /// Emits the destruction of an entity. `Err(BufferFull)` if the buffer is full.
    pub fn despawn(&mut self, entity: EntityId) -> Result<(), BufferFull> {
        self.0.despawn(entity)
    }
}

impl<
    const COMMANDS: usize,
    Add: Copy,
    Remove: Copy,
    const SPAWN: bool,
    const DESPAWN: bool,
    const REMOVE: bool,
> Commands<&mut CommandBuffer<COMMANDS, Add, Remove>, SPAWN, DESPAWN, true, REMOVE>
{
    /// Emits the addition of a component to an entity. `Err(BufferFull)` if the
    /// buffer is full.
    pub fn add_component(&mut self, entity: EntityId, add: Add) -> Result<(), BufferFull> {
        self.0.add_component(entity, add)
    }
}

impl<
    const COMMANDS: usize,
    Add: Copy,
    Remove: Copy,
    const SPAWN: bool,
    const DESPAWN: bool,
    const ADD: bool,
> Commands<&mut CommandBuffer<COMMANDS, Add, Remove>, SPAWN, DESPAWN, ADD, true>
{
    /// Emits the removal of a component from an entity. `Err(BufferFull)` if the
    /// buffer is full.
    pub fn remove_component(&mut self, entity: EntityId, remove: Remove) -> Result<(), BufferFull> {
        self.0.remove_component(entity, remove)
    }
}

// ---------------------------------------------------------------------------
// Unit and tuples
// ---------------------------------------------------------------------------

impl sealed::Sealed for () {}

impl Part for () {
    const CONTRACT: Contract = Contract::new(
        DataSet::EMPTY,
        DataSet::EMPTY,
        DataSet::EMPTY,
        StructuralEffects::NONE,
    );
}

impl<P> sealed::Bound<P> for () {}

impl<'w, P> Bind<'w, P> for () {
    fn bind(_parts: &mut P) -> Option<Self> {
        Some(())
    }
}

macro_rules! tuple_parts {
    ($first:ident $(, $rest:ident)*) => {
        impl<$first: Part $(, $rest: Part)*> sealed::Sealed for ($first, $($rest,)*) {
            const HANDLES: usize =
                <$first as sealed::Sealed>::HANDLES $(+ <$rest as sealed::Sealed>::HANDLES)*;
        }

        impl<$first: Part $(, $rest: Part)*> Part for ($first, $($rest,)*) {
            const CONTRACT: Contract = {
                assert!(
                    <Self as sealed::Sealed>::HANDLES <= 1,
                    "a command handle is declared twice in one access"
                );
                let contract = $first::CONTRACT;
                $(let contract = merge_disjoint(contract, $rest::CONTRACT);)*
                contract
            };
        }

        impl<'w, P, $first: Bind<'w, P> $(, $rest: Bind<'w, P>)*> sealed::Bound<P>
            for ($first, $($rest,)*)
        {
        }

        impl<'w, P, $first: Bind<'w, P> $(, $rest: Bind<'w, P>)*> Bind<'w, P>
            for ($first, $($rest,)*)
        {
            fn bind(parts: &mut P) -> Option<Self> {
                // See `view!`: evaluates the constant checks of the whole access.
                let _ = Self::CONTRACT;
                Some(($first::bind(parts)?, $($rest::bind(parts)?,)*))
            }
        }
    };
}

tuple_parts!(A);
tuple_parts!(A, B);
tuple_parts!(A, B, C);
tuple_parts!(A, B, C, D);

#[cfg(test)]
mod tests {
    use super::*;

    fn data(reads: u8, writes: u8, produces: u8) -> Contract {
        let set = |index: u8| DataSet::of(index);
        Contract::new(
            set(reads),
            set(writes),
            set(produces),
            StructuralEffects::NONE,
        )
    }

    /// A part with no category and the structural effects given by `BITS`
    /// (spawn, destroy, add, remove): it exists to test the effect merge, which
    /// no view of this phase can exercise.
    struct Effects<const BITS: u8>;

    impl<const BITS: u8> sealed::Sealed for Effects<BITS> {}

    impl<const BITS: u8> Part for Effects<BITS> {
        const CONTRACT: Contract = Contract::new(
            DataSet::EMPTY,
            DataSet::EMPTY,
            DataSet::EMPTY,
            StructuralEffects::new(
                BITS & 0b0001 != 0,
                BITS & 0b0010 != 0,
                BITS & 0b0100 != 0,
                BITS & 0b1000 != 0,
            ),
        );
    }

    fn flags(contract: Contract) -> [bool; 4] {
        let effects = contract.effects();
        [
            effects.spawns(),
            effects.destroys(),
            effects.adds_component(),
            effects.removes_component(),
        ]
    }

    fn handles<T: sealed::Sealed>() -> usize {
        T::HANDLES
    }

    /// The marker counts handles, not effect bits, through nesting.
    #[test]
    fn handles_are_counted_through_tuples_and_nesting() {
        type None = Commands<(), false, false, false, false>;
        type Spawn = Commands<(), true, false, false, false>;

        assert_eq!(handles::<()>(), 0);
        assert_eq!(handles::<Read<0, ()>>(), 0);
        assert_eq!(handles::<(Read<0, ()>, Write<1, ()>)>(), 0);
        assert_eq!(handles::<None>(), 1);
        assert_eq!(handles::<Spawn>(), 1);
        assert_eq!(handles::<(Spawn, Read<0, ()>)>(), 1);
        assert_eq!(handles::<(Spawn, None)>(), 2);
        assert_eq!(handles::<((Spawn,), (Read<0, ()>, (None,)))>(), 2);
        assert_eq!(handles::<(Spawn, Spawn, Spawn, Spawn)>(), 4);
    }

    #[test]
    fn merge_unites_every_component_of_disjoint_contracts() {
        let merged = merge_disjoint(data(0, 1, 2), data(3, 4, 5));
        let set = |indices: [u8; 2]| DataSet::of(indices[0]).union(DataSet::of(indices[1]));
        assert_eq!(merged.reads(), set([0, 3]));
        assert_eq!(merged.writes(), set([1, 4]));
        assert_eq!(merged.produces(), set([2, 5]));
    }

    #[test]
    fn merge_with_the_empty_contract_changes_nothing() {
        let contract = data(0, 1, 2);
        assert_eq!(merge_disjoint(contract, <() as Part>::CONTRACT), contract);
        assert_eq!(merge_disjoint(<() as Part>::CONTRACT, contract), contract);
    }

    /// A contract using category `index` in one mode only: 0 reads, 1 writes, 2 produces.
    fn in_mode(mode: usize, index: u8) -> Contract {
        let on = |m: usize| {
            if m == mode {
                DataSet::of(index)
            } else {
                DataSet::EMPTY
            }
        };
        Contract::new(on(0), on(1), on(2), StructuralEffects::NONE)
    }

    /// Sharing a category is refused whatever the mode on each side: the nine pairs.
    macro_rules! shared_category_is_refused {
        ($($name:ident: $mode_a:literal, $mode_b:literal;)*) => {$(
            #[test]
            #[should_panic(expected = "declared twice")]
            fn $name() {
                let _ = merge_disjoint(in_mode($mode_a, 7), in_mode($mode_b, 7));
            }
        )*};
    }

    shared_category_is_refused! {
        merge_refuses_read_and_read: 0, 0;
        merge_refuses_read_and_write: 0, 1;
        merge_refuses_read_and_produce: 0, 2;
        merge_refuses_write_and_read: 1, 0;
        merge_refuses_write_and_write: 1, 1;
        merge_refuses_write_and_produce: 1, 2;
        merge_refuses_produce_and_read: 2, 0;
        merge_refuses_produce_and_write: 2, 1;
        merge_refuses_produce_and_produce: 2, 2;
    }

    #[test]
    #[should_panic(expected = "declared twice")]
    fn merge_panics_at_runtime_on_a_shared_category() {
        let _ = merge_disjoint(data(1, 2, 3), data(4, 2, 5));
    }

    #[test]
    fn merge_does_not_panic_on_adjacent_categories() {
        let _ = merge_disjoint(data(1, 2, 3), data(4, 5, 6));
    }

    #[test]
    fn structural_effects_are_or_ed_independently_by_indicator() {
        const NONE: Contract = <(Effects<0>, Effects<0>) as Part>::CONTRACT;
        const SPAWN_DESTROY: Contract = <(Effects<0b0001>, Effects<0b0010>) as Part>::CONTRACT;
        const ADD_REMOVE_ADD: Contract =
            <(Effects<0b0100>, Effects<0b1000>, Effects<0b0100>) as Part>::CONTRACT;
        const ALL: Contract = <(
            Effects<0b0001>,
            Effects<0b0010>,
            Effects<0b0100>,
            Effects<0b1000>,
        ) as Part>::CONTRACT;

        assert_eq!(flags(NONE), [false; 4]);
        assert_eq!(flags(SPAWN_DESTROY), [true, true, false, false]);
        assert_eq!(flags(ADD_REMOVE_ADD), [false, false, true, true]);
        assert_eq!(flags(ALL), [true; 4]);
        assert!(ALL.emits_commands() && !NONE.emits_commands());
    }
}
