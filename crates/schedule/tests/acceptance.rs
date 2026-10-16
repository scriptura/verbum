// crates/schedule/tests/acceptance.rs

//! Stage 1, acceptance (ROADMAP, step 1): the invariants observed through the public API of
//! the scheduler, with a real `World`, `Binding` and `Parts` written outside the crates.
//!
//! Two cases, not a copy of the unit coverage:
//!
//! 1. an access that reads a datum observes what the producers wrote in the same phase, and a
//!    cycle is refused at construction, before any system can run;
//! 2. several systems produce freely to one transient datum, a single system resolves it into
//!    the persistent datum; a second resolver is refused at construction.
//!
//! The datum: `ACCUMULATOR` is transient (produced), `COUNTER_A` is persistent (written).
//! Each producer adds its own constant to the accumulator; the resolver copies the
//! accumulator into the counter. The aggregation is additive, hence commutative: the
//! permutation test checks that the declaration order does not change it, nothing more.

// Shared with the other test crates: each one uses its own part of the module.
#[allow(dead_code)]
mod support;

use core::sync::atomic::{AtomicBool, Ordering};

use verbum_ecs::BufferFull;
use verbum_schedule::model::ModelError;
use verbum_schedule::scheduler::Scheduler;
use verbum_schedule::system::{At, Cons, Nil, System, SystemList};

use support::*;

/// Free production: adds `K` to the accumulator. Constrained by nothing.
struct Contribute<const K: i32>;

impl<const K: i32> System<MotionBinding> for Contribute<K> {
    type Access<'w> = (ProduceOf<'w, ACCUMULATOR>,);

    fn run(access: Self::Access<'_>) -> Result<(), BufferFull> {
        let (mut accumulator,) = access;
        accumulator
            .get_mut()
            .expect("accumulator inserted by the fixture")
            .0 += K;
        Ok(())
    }
}

/// Resolution: reads the accumulator, writes the persistent counter. `K` only tells the
/// resolvers apart.
struct Resolve<const K: u8>;

impl<const K: u8> System<MotionBinding> for Resolve<K> {
    type Access<'w> = (ReadOf<'w, ACCUMULATOR>, WriteOf<'w, COUNTER_A>);

    fn run(access: Self::Access<'_>) -> Result<(), BufferFull> {
        let (accumulator, mut counter) = access;
        let total = accumulator.get().expect("accumulator present").0;
        counter.insert(Counter(
            u32::try_from(total).expect("total is not negative"),
        ));
        Ok(())
    }
}

/// Set when a system of a refused cycle runs: it must stay unset.
static CYCLE_RAN: AtomicBool = AtomicBool::new(false);

/// Reads `ACCUMULATOR`, produces `COUNTER_B`.
struct ToB;

impl System<MotionBinding> for ToB {
    type Access<'w> = (ReadOf<'w, ACCUMULATOR>, ProduceOf<'w, COUNTER_B>);

    fn run(_: Self::Access<'_>) -> Result<(), BufferFull> {
        CYCLE_RAN.store(true, Ordering::SeqCst);
        Ok(())
    }
}

/// Reads `COUNTER_B`, produces `ACCUMULATOR`: with `ToB`, a cycle.
struct ToAccumulator;

impl System<MotionBinding> for ToAccumulator {
    type Access<'w> = (ReadOf<'w, COUNTER_B>, ProduceOf<'w, ACCUMULATOR>);

    fn run(_: Self::Access<'_>) -> Result<(), BufferFull> {
        CYCLE_RAN.store(true, Ordering::SeqCst);
        Ok(())
    }
}

/// A list of systems, all in phase 0, in the order written.
macro_rules! list {
    () => { Nil };
    ($head:ty $(, $tail:ty)*) => { Cons<At<0, $head>, list!($($tail),*)> };
}

type P1 = Contribute<1>;
type P2 = Contribute<2>;
type P3 = Contribute<3>;
type R = Resolve<0>;

/// The accumulator is zero and the counter absent: nothing is inherited from an earlier tick.
fn fixture() -> (MotionWorld, MotionBuffer) {
    let mut world = MotionWorld::new();
    world.resources_mut().accumulator.insert(Accumulator(0));
    (world, MotionBuffer::new())
}

fn counter(world: &MotionWorld) -> Option<u32> {
    world.resources().counter_a.get().map(|c| c.0)
}

fn accumulator(world: &MotionWorld) -> Option<i32> {
    world.resources().accumulator.get().map(|a| a.0)
}

/// Builds the scheduler of `L` (4 systems, 1 phase) and runs `ticks` ticks on a fresh
/// fixture, returning the counter observed after each one.
fn totals_over<L: SystemList<MotionBinding>, const TICKS: usize>() -> [Option<u32>; TICKS] {
    let scheduler = Scheduler::<MotionBinding, L, 4, 1>::new().expect("a valid schedule");
    let (mut world, mut buffer) = fixture();
    let mut seen = [None; TICKS];
    for slot in &mut seen {
        scheduler
            .run_tick(&mut world, &mut buffer)
            .expect("the tick completes");
        *slot = counter(&world);
    }
    seen
}

// ---------------------------------------------------------------------------
// Case 1: read after write in a phase, cycle refused at construction
// ---------------------------------------------------------------------------

#[test]
fn a_reader_observes_the_writes_of_the_same_phase_whatever_its_declaration_rank() {
    // The resolver is declared first: only the order of the model puts it after the
    // producers. Run before them, it would copy the initial 0.
    type L = list!(R, P1, P2, P3);
    let scheduler = Scheduler::<MotionBinding, L, 4, 1>::new().expect("a valid schedule");
    assert_eq!(scheduler.model().phase(0), Some(&[1, 2, 3, 0][..]));

    let (mut world, mut buffer) = fixture();
    scheduler.run_tick(&mut world, &mut buffer).unwrap();

    assert_eq!(counter(&world), Some(1 + 2 + 3));
}

#[test]
fn a_cycle_is_refused_at_construction_and_no_system_ran() {
    type Cyclic = list!(ToB, ToAccumulator);
    CYCLE_RAN.store(false, Ordering::SeqCst);

    let built = Scheduler::<MotionBinding, Cyclic, 2, 1>::new();

    assert_eq!(built.map(|_| ()), Err(ModelError::Cycle));
    assert!(
        !CYCLE_RAN.load(Ordering::SeqCst),
        "construction runs no system: the cycle is refused before any execution"
    );
}

// ---------------------------------------------------------------------------
// Case 2: free production, a single resolution
// ---------------------------------------------------------------------------

#[test]
fn three_producers_and_one_resolver_resolve_into_the_persistent_datum() {
    type L = list!(P1, P2, P3, R);
    let scheduler = Scheduler::<MotionBinding, L, 4, 1>::new().expect("a valid schedule");
    assert_eq!(
        scheduler.model().phase(0),
        Some(&[0, 1, 2, 3][..]),
        "the producers constrain each other by nothing, the resolver comes last"
    );

    let (mut world, mut buffer) = fixture();
    assert_eq!(counter(&world), None, "the persistent datum starts unset");
    scheduler.run_tick(&mut world, &mut buffer).unwrap();

    assert_eq!(
        accumulator(&world),
        Some(6),
        "the three contributions are aggregated"
    );
    assert_eq!(counter(&world), Some(6), "and resolved once");
}

#[test]
fn two_resolvers_of_the_same_persistent_datum_are_refused_at_construction() {
    type Two = list!(P1, P2, R, Resolve<1>);

    let built = Scheduler::<MotionBinding, Two, 4, 1>::new();

    assert_eq!(built.map(|_| ()), Err(ModelError::MultipleWriters));
}

#[test]
fn the_declaration_order_of_the_producers_does_not_change_an_additive_aggregation() {
    // The six orders of the three producers, the resolver declared first in each. Additive
    // aggregation only: this says nothing about a non-commutative operation.
    let observed = [
        totals_over::<list!(R, P1, P2, P3), 1>(),
        totals_over::<list!(R, P1, P3, P2), 1>(),
        totals_over::<list!(R, P2, P1, P3), 1>(),
        totals_over::<list!(R, P2, P3, P1), 1>(),
        totals_over::<list!(R, P3, P1, P2), 1>(),
        totals_over::<list!(R, P3, P2, P1), 1>(),
    ];
    assert_eq!(observed, [[Some(6)]; 6]);
}

// ---------------------------------------------------------------------------
// The scheduler resets nothing between ticks
// ---------------------------------------------------------------------------

#[test]
fn the_transient_datum_is_not_reset_implicitly_between_ticks() {
    // Accumulator at zero in the fixture, no system clears it: the same scheduler is run
    // three times and each tick adds its contributions to what the previous one left.
    type L = list!(P1, P2, P3, R);

    let totals = totals_over::<L, 3>();

    assert_eq!(totals, [Some(6), Some(12), Some(18)]);
}
