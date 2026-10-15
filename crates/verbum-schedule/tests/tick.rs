// crates/verbum-schedule/tests/tick.rs

//! Phase 9, commit A: execution of a tick, phase after phase, in the order of the model.
//!
//! Real `World`, `Parts` and `Binding` written outside the crates, public API only. The state
//! is compared through public reads: velocities by entity, a counter resource as witness.

// Shared with the other test crates: each one uses its own part of the module.
#[allow(dead_code)]
mod support;

use verbum_ecs::{BufferFull, EntityId};
use verbum_schedule::contract::Contract;
use verbum_schedule::scheduler::{Scheduler, TickError};
use verbum_schedule::system::{At, Cons, Nil, RunError, System, contract_of};

use support::*;

/// Adds one to every velocity.
struct Boost;

impl System<MotionBinding> for Boost {
    type Access<'w> = (WriteOf<'w, VELOCITY>,);

    fn run(access: Self::Access<'_>) -> Result<(), BufferFull> {
        let (mut velocities,) = access;
        for index in 0..velocities.len() {
            let entity = velocities.entities()[index];
            if let Some(velocity) = velocities.get_mut(entity) {
                velocity.0 += 1;
            }
        }
        Ok(())
    }
}

/// Same access as `Boost`, other body: visits the entities backwards.
struct BoostBackwards;

impl System<MotionBinding> for BoostBackwards {
    type Access<'w> = (WriteOf<'w, VELOCITY>,);

    fn run(access: Self::Access<'_>) -> Result<(), BufferFull> {
        let (mut velocities,) = access;
        for index in (0..velocities.len()).rev() {
            let entity = velocities.entities()[index];
            if let Some(velocity) = velocities.get_mut(entity) {
                velocity.0 = velocity.0 + 2 - 1;
            }
        }
        Ok(())
    }
}

/// Stores the sum of the velocities into counter A.
struct SumInto;

impl System<MotionBinding> for SumInto {
    type Access<'w> = (ReadOf<'w, VELOCITY>, WriteOf<'w, COUNTER_A>);

    fn run(access: Self::Access<'_>) -> Result<(), BufferFull> {
        let (velocities, mut counter) = access;
        let sum = velocities
            .entities()
            .iter()
            .filter_map(|&entity| velocities.get(entity))
            .map(|velocity| velocity.0)
            .sum::<i32>();
        counter.get_mut().expect("counter A inserted").0 = sum as u32;
        Ok(())
    }
}

/// Same access as `SumInto`, other body: a loop over the dense order.
struct SumLoop;

impl System<MotionBinding> for SumLoop {
    type Access<'w> = (ReadOf<'w, VELOCITY>, WriteOf<'w, COUNTER_A>);

    fn run(access: Self::Access<'_>) -> Result<(), BufferFull> {
        let (velocities, mut counter) = access;
        let mut sum = 0;
        for index in 0..velocities.len() {
            let entity = velocities.entities()[index];
            sum += velocities.get(entity).map_or(0, |velocity| velocity.0);
        }
        counter.get_mut().expect("counter A inserted").0 = sum as u32;
        Ok(())
    }
}

/// Emits more `Spawn` commands than the buffer holds.
struct Overflow;

impl System<MotionBinding> for Overflow {
    type Access<'w> = (CommandsOf<'w, true, false, false, false>,);

    fn run(access: Self::Access<'_>) -> Result<(), BufferFull> {
        let (mut commands,) = access;
        for _ in 0..5 {
            commands.spawn()?;
        }
        Ok(())
    }
}

fn velocities_of(world: &mut MotionWorld, entities: [EntityId; 3]) -> [Option<i32>; 3] {
    let ((_, velocities), _) = world.split_mut();
    entities.map(|entity| velocities.get(entity).map(|velocity| velocity.0))
}

fn counter_a(world: &MotionWorld) -> u32 {
    world
        .resources()
        .counter_a
        .get()
        .expect("counter A inserted")
        .0
}

// ---------------------------------------------------------------------------
// (1) Read after write inside a phase
// ---------------------------------------------------------------------------

/// `SumInto` is declared before `Boost` and reads what `Boost` writes: the model puts it
/// after, and it observes the new values (10 + 1, 30 + 1) instead of the old ones (40).
#[test]
fn a_reader_declared_first_observes_the_write_of_its_phase() {
    type List = Cons<At<0, SumInto>, Cons<At<0, Boost>, Nil>>;
    let scheduler = Scheduler::<MotionBinding, List, 2, 1>::new().unwrap();
    assert_eq!(scheduler.model().phase(0), Some(&[1, 0][..]));

    let (mut world, [a, b, c]) = populated_world();
    let mut buffer = MotionBuffer::new();
    assert_eq!(counter_a(&world), 10);

    scheduler.run_tick(&mut world, &mut buffer).unwrap();

    assert_eq!(counter_a(&world), 42, "the sum is taken after the write");
    assert_eq!(
        velocities_of(&mut world, [a, b, c]),
        [Some(11), None, Some(31)]
    );
}

/// Declaration order does not decide: with the writer declared first, the result is the same.
#[test]
fn the_order_of_declaration_does_not_change_the_observation() {
    type List = Cons<At<0, Boost>, Cons<At<0, SumInto>, Nil>>;
    let scheduler = Scheduler::<MotionBinding, List, 2, 1>::new().unwrap();
    assert_eq!(scheduler.model().phase(0), Some(&[0, 1][..]));

    let (mut world, _) = populated_world();
    scheduler
        .run_tick(&mut world, &mut MotionBuffer::new())
        .unwrap();
    assert_eq!(counter_a(&world), 42);
}

/// Phases run in order, whatever the declaration order: the reader of phase 1 declared first
/// sees the write of phase 0; a reader in phase 0 does not see the write of phase 1.
#[test]
fn phases_run_in_order_whatever_the_declaration_order() {
    type Later = Cons<At<1, SumInto>, Cons<At<0, Boost>, Nil>>;
    let later = Scheduler::<MotionBinding, Later, 2, 2>::new().unwrap();
    assert_eq!(later.model().phase(0), Some(&[1][..]));
    assert_eq!(later.model().phase(1), Some(&[0][..]));
    let (mut world, _) = populated_world();
    later
        .run_tick(&mut world, &mut MotionBuffer::new())
        .unwrap();
    assert_eq!(
        counter_a(&world),
        42,
        "read in phase 1, after the write of phase 0"
    );

    type Earlier = Cons<At<0, SumInto>, Cons<At<1, Boost>, Nil>>;
    let earlier = Scheduler::<MotionBinding, Earlier, 2, 2>::new().unwrap();
    let (mut world, entities) = populated_world();
    earlier
        .run_tick(&mut world, &mut MotionBuffer::new())
        .unwrap();
    assert_eq!(
        counter_a(&world),
        40,
        "read in phase 0, before the write of phase 1"
    );
    assert_eq!(
        velocities_of(&mut world, entities),
        [Some(11), None, Some(31)]
    );
}

#[test]
fn a_tick_of_an_empty_list_changes_nothing() {
    let scheduler = Scheduler::<MotionBinding, Nil, 0, 1>::new().unwrap();
    let (mut world, entities) = populated_world();
    scheduler
        .run_tick(&mut world, &mut MotionBuffer::new())
        .unwrap();
    assert_eq!(counter_a(&world), 10);
    assert_eq!(
        velocities_of(&mut world, entities),
        [Some(10), None, Some(30)]
    );
}

// ---------------------------------------------------------------------------
// (2) Interchangeability: same contracts, same model, same states
// ---------------------------------------------------------------------------

#[test]
fn systems_with_the_same_contracts_give_equal_models_and_equal_states() {
    assert_eq!(
        contract_of::<MotionBinding, Boost>(),
        contract_of::<MotionBinding, BoostBackwards>()
    );
    assert_eq!(
        contract_of::<MotionBinding, SumInto>(),
        contract_of::<MotionBinding, SumLoop>()
    );
    let _: Contract = contract_of::<MotionBinding, Boost>();

    type First = Cons<At<0, SumInto>, Cons<At<0, Boost>, Nil>>;
    type Second = Cons<At<0, SumLoop>, Cons<At<0, BoostBackwards>, Nil>>;
    let first = Scheduler::<MotionBinding, First, 2, 1>::new().unwrap();
    let second = Scheduler::<MotionBinding, Second, 2, 1>::new().unwrap();
    assert_eq!(first.model(), second.model());

    let (mut world_first, entities) = populated_world();
    let (mut world_second, _) = populated_world();
    for _ in 0..3 {
        first
            .run_tick(&mut world_first, &mut MotionBuffer::new())
            .unwrap();
        second
            .run_tick(&mut world_second, &mut MotionBuffer::new())
            .unwrap();
    }
    assert_eq!(counter_a(&world_first), 46);
    assert_eq!(counter_a(&world_first), counter_a(&world_second));
    assert_eq!(
        velocities_of(&mut world_first, entities),
        velocities_of(&mut world_second, entities)
    );
}

// ---------------------------------------------------------------------------
// (3) Immutability
// ---------------------------------------------------------------------------

fn tick_through_a_shared_reference<const S: usize, const P: usize, L>(
    scheduler: &Scheduler<MotionBinding, L, S, P>,
    world: &mut MotionWorld,
) where
    L: verbum_schedule::system::SystemList<MotionBinding>,
{
    scheduler.run_tick(world, &mut MotionBuffer::new()).unwrap();
}

#[test]
fn the_model_is_identical_after_n_ticks() {
    type List = Cons<At<0, SumInto>, Cons<At<0, Boost>, Nil>>;
    // Not `mut`: `run_tick` takes `&self`.
    let scheduler = Scheduler::<MotionBinding, List, 2, 1>::new().unwrap();
    let before = *scheduler.model();

    let (mut world, _) = populated_world();
    for _ in 0..5 {
        tick_through_a_shared_reference(&scheduler, &mut world);
    }
    assert_eq!(*scheduler.model(), before);
    assert_eq!(counter_a(&world), 10 + 30 + 2 * 5, "five ticks did run");
}

// ---------------------------------------------------------------------------
// (4) Determinism
// ---------------------------------------------------------------------------

#[test]
fn two_identical_worlds_give_the_same_readings_tick_after_tick() {
    type List = Cons<At<0, SumInto>, Cons<At<0, Boost>, Nil>>;
    let scheduler = Scheduler::<MotionBinding, List, 2, 1>::new().unwrap();
    let ((mut left, entities), (mut right, _)) = (populated_world(), populated_world());

    let mut readings = [(0, 0); 3];
    for reading in &mut readings {
        scheduler
            .run_tick(&mut left, &mut MotionBuffer::new())
            .unwrap();
        scheduler
            .run_tick(&mut right, &mut MotionBuffer::new())
            .unwrap();
        *reading = (counter_a(&left), counter_a(&right));
    }
    assert_eq!(readings, [(42, 42), (44, 44), (46, 46)]);
    assert_eq!(
        velocities_of(&mut left, entities),
        velocities_of(&mut right, entities)
    );
}

// ---------------------------------------------------------------------------
// Errors of a tick: the conversion keeps the distinction
// ---------------------------------------------------------------------------

#[test]
fn a_run_error_converts_without_losing_its_kind() {
    assert_eq!(TickError::from(RunError::Binding), TickError::Binding);
    assert_eq!(
        TickError::from(RunError::Buffer(BufferFull)),
        TickError::Buffer(BufferFull)
    );
    assert_ne!(
        TickError::from(RunError::Binding),
        TickError::from(RunError::Buffer(BufferFull))
    );
}

/// The error of a system is returned, and the tick stops there: neither the following system
/// of the phase nor the following phase runs. (The full contract of the errors comes with the
/// structural validation.)
#[test]
fn the_error_of_a_system_stops_the_tick() {
    type List = Cons<At<0, Overflow>, Cons<At<0, SumInto>, Cons<At<1, Boost>, Nil>>>;
    let scheduler = Scheduler::<MotionBinding, List, 3, 2>::new().unwrap();
    assert_eq!(scheduler.model().phase(0), Some(&[0, 1][..]));

    let (mut world, entities) = populated_world();
    let result = scheduler.run_tick(&mut world, &mut MotionBuffer::new());

    assert_eq!(result, Err(TickError::Buffer(BufferFull)));
    assert_eq!(
        counter_a(&world),
        10,
        "the next system of the phase did not run"
    );
    assert_eq!(
        velocities_of(&mut world, entities),
        [Some(10), None, Some(30)],
        "the next phase did not run"
    );
}
