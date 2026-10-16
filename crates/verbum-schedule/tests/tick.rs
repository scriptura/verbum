// crates/verbum-schedule/tests/tick.rs

//! Phase 9, commit A: execution of a tick, phase after phase, in the order of the model.
//!
//! Real `World`, `Parts` and `Binding` written outside the crates, public API only. The state
//! is compared through public reads: velocities by entity, a counter resource as witness.

// Shared with the other test crates: each one uses its own part of the module.
#[allow(dead_code)]
mod support;

use verbum_ecs::{ApplyError, BufferFull, EntityId, InsertError};
use verbum_schedule::contract::Contract;
use verbum_schedule::order::precedes;
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

// ===========================================================================
// Commit B: validation at the end of each phase, structural effects deferred
// ===========================================================================

/// Implements a system for both bindings of the tick world, with one body.
macro_rules! tick_system {
    ($name:ident<$w:lifetime> => $access:ty, |$arg:ident| $body:block) => {
        struct $name;

        impl System<TickBinding> for $name {
            type Access<$w> = $access;

            fn run($arg: Self::Access<'_>) -> Result<(), BufferFull> $body
        }

        impl System<DefectiveBinding> for $name {
            type Access<$w> = $access;

            fn run($arg: Self::Access<'_>) -> Result<(), BufferFull> $body
        }
    };
}

fn target(targets: &TRead<'_, TARGETS>) -> Targets {
    *targets.get().expect("targets inserted")
}

/// What an observer saw: bit 0 = `a` has a position, bit 1 = `c` has a velocity, bit 2 = `b`
/// has a position.
fn observe(
    targets: &TRead<'_, TARGETS>,
    positions: &TRead<'_, POSITION>,
    velocities: &TRead<'_, VELOCITY>,
) -> i32 {
    let t = target(targets);
    i32::from(positions.contains(t.a))
        | i32::from(velocities.contains(t.c)) << 1
        | i32::from(positions.contains(t.b)) << 2
}

fn set(cell: &mut TWrite<'_, { SEEN_B }>, value: i32) {
    cell.get_mut().expect("cell inserted").0 = value;
}

// Test 5: the emitter despawns `a`, adds a position to `b`, removes the velocity of `c`.
tick_system!(Emit<'w> => (TRead<'w, TARGETS>, TCommands<'w, false, true, true, true>), |access| {
    let (targets, mut commands) = access;
    let t = target(&targets);
    commands.despawn(t.a)?;
    commands.add_component(t.b, MotionAdd::Position(Position(9)))?;
    commands.remove_component(t.c, MotionRemove::Velocity)
});

// Same emitter, which also writes `FLAG`: a real data dependency with its observer.
tick_system!(EmitFlag<'w> => (
    TRead<'w, TARGETS>,
    TWrite<'w, FLAG>,
    TCommands<'w, false, true, true, true>
), |access| {
    let (targets, mut flag, mut commands) = access;
    flag.get_mut().expect("flag inserted").0 = 1;
    let t = target(&targets);
    commands.despawn(t.a)?;
    commands.add_component(t.b, MotionAdd::Position(Position(9)))?;
    commands.remove_component(t.c, MotionRemove::Velocity)
});

tick_system!(ObserveB<'w> => (
    TRead<'w, TARGETS>,
    TRead<'w, POSITION>,
    TRead<'w, VELOCITY>,
    TWrite<'w, SEEN_B>
), |access| {
    let (targets, positions, velocities, mut seen) = access;
    set(&mut seen, observe(&targets, &positions, &velocities));
    Ok(())
});

// Observer that depends on `FLAG`. An access holds at most four parts: the fifth nests.
tick_system!(ObserveFlag<'w> => (
    (TRead<'w, TARGETS>, TRead<'w, POSITION>, TRead<'w, VELOCITY>),
    (TRead<'w, FLAG>, TWrite<'w, SEEN_B>)
), |access| {
    let ((targets, positions, velocities), (_flag, mut seen)) = access;
    set(&mut seen, observe(&targets, &positions, &velocities));
    Ok(())
});

tick_system!(ObserveC<'w> => (
    TRead<'w, TARGETS>,
    TRead<'w, POSITION>,
    TRead<'w, VELOCITY>,
    TWrite<'w, SEEN_C>
), |access| {
    let (targets, positions, velocities, mut seen) = access;
    let value = observe(&targets, &positions, &velocities);
    seen.get_mut().expect("cell inserted").0 = value;
    Ok(())
});

// Test 6, phase 0: writes a value of `c`, emits `Despawn(a)`.
tick_system!(ValidPhase<'w> => (
    TRead<'w, TARGETS>,
    TWrite<'w, VELOCITY>,
    TCommands<'w, false, true, false, false>
), |access| {
    let (targets, mut velocities, mut commands) = access;
    let t = target(&targets);
    velocities.get_mut(t.c).expect("c has a velocity").0 += 5;
    commands.despawn(t.a)
});

// Test 6, phase 1: writes `VALUE`, emits `Despawn(d)` then a duplicate `Add` on `c`.
tick_system!(FailingPhase<'w> => (
    TRead<'w, TARGETS>,
    TWrite<'w, VALUE>,
    TCommands<'w, false, true, true, false>
), |access| {
    let (targets, mut value, mut commands) = access;
    value.get_mut().expect("value inserted").0 = 7;
    let t = target(&targets);
    commands.despawn(t.d)?;
    commands.add_component(t.c, MotionAdd::Position(Position(99)))
});

// A system that must not run: it writes `WITNESS`.
tick_system!(Witness<'w> => (TWrite<'w, WITNESS>,), |access| {
    let (mut witness,) = access;
    witness.get_mut().expect("witness inserted").0 = 1;
    Ok(())
});

// A system of a following phase that must not run: it writes `SEEN_C`.
tick_system!(LaterWitness<'w> => (TWrite<'w, SEEN_C>,), |access| {
    let (mut seen,) = access;
    seen.get_mut().expect("cell inserted").0 = 1;
    Ok(())
});

// Test 7, S1: a valid command, a value, one more run.
tick_system!(First<'w> => (
    TRead<'w, TARGETS>,
    TWrite<'w, VALUE>,
    TWrite<'w, TALLY>,
    TCommands<'w, false, true, false, false>
), |access| {
    let (targets, mut value, mut tally, mut commands) = access;
    value.get_mut().expect("value inserted").0 = 11;
    tally.get_mut().expect("tally inserted").0 += 1;
    commands.despawn(target(&targets).a)
});

// Test 7, S2: emits more commands than the buffer holds (4 despawns after one already there).
tick_system!(Overfill<'w> => (
    TRead<'w, TARGETS>,
    TCommands<'w, false, true, false, false>
), |access| {
    let (targets, mut commands) = access;
    let t = target(&targets);
    for entity in [t.b, t.c, t.d, t.d] {
        commands.despawn(entity)?;
    }
    Ok(())
});

// Test 8, S1: a value and a command, then the next system cannot bind.
tick_system!(EmitThenFault<'w> => (
    TRead<'w, TARGETS>,
    TWrite<'w, VALUE>,
    TCommands<'w, false, true, false, false>
), |access| {
    let (targets, mut value, mut commands) = access;
    value.get_mut().expect("value inserted").0 = 11;
    commands.despawn(target(&targets).a)
});

// Counts its runs and emits nothing.
tick_system!(Tally<'w> => (TWrite<'w, TALLY>,), |access| {
    let (mut tally,) = access;
    tally.get_mut().expect("tally inserted").0 += 1;
    Ok(())
});

fn cell(world: &TickWorld, pick: fn(&TickGlobals) -> &verbum_ecs::ResourceSlot<Cell>) -> i32 {
    pick(world.resources()).get().expect("cell inserted").0
}

fn seen_b(world: &TickWorld) -> i32 {
    cell(world, |globals| &globals.seen_b)
}
fn seen_c(world: &TickWorld) -> i32 {
    cell(world, |globals| &globals.seen_c)
}
fn value(world: &TickWorld) -> i32 {
    cell(world, |globals| &globals.value)
}
fn witness(world: &TickWorld) -> i32 {
    cell(world, |globals| &globals.witness)
}
fn tally(world: &TickWorld) -> i32 {
    cell(world, |globals| &globals.tally)
}

fn has_position(world: &mut TickWorld, entity: EntityId) -> bool {
    let ((positions, _), _) = world.split_mut();
    positions.contains(entity)
}

fn has_velocity(world: &mut TickWorld, entity: EntityId) -> bool {
    let ((_, velocities), _) = world.split_mut();
    velocities.contains(entity)
}

fn velocity_of(world: &mut TickWorld, entity: EntityId) -> Option<i32> {
    let ((_, velocities), _) = world.split_mut();
    velocities.get(entity).map(|velocity| velocity.0)
}

/// Observers see `a` and `c` unchanged, `b` without position: bits 0 and 1.
const BEFORE: i32 = 0b011;
/// After the validation: `a` purged, `c` without velocity, `b` with a position: bit 2.
const AFTER: i32 = 0b100;

// ---------------------------------------------------------------------------
// (5a) Deferred structural effects: no edge, order by declaration
// ---------------------------------------------------------------------------

#[test]
fn structural_effects_are_invisible_before_the_validation_and_visible_after() {
    // The emitter is declared before its observer, and the two have no data dependency:
    // the observer runs second because of the declaration order, not of an edge.
    let emitter = contract_of::<TickBinding, Emit>();
    let observer = contract_of::<TickBinding, ObserveB>();
    assert!(!precedes(emitter, observer), "no edge emitter -> observer");
    assert!(!precedes(observer, emitter), "no edge observer -> emitter");

    type List = Cons<At<0, Emit>, Cons<At<0, ObserveB>, Cons<At<1, ObserveC>, Nil>>>;
    let scheduler = Scheduler::<TickBinding, List, 3, 2>::new().unwrap();
    assert_eq!(scheduler.model().phase(0), Some(&[0, 1][..]));
    assert_eq!(scheduler.model().phase(1), Some(&[2][..]));

    let (mut world, t) = tick_world();
    let mut buffer = MotionBuffer::new();
    scheduler.run_tick(&mut world, &mut buffer).unwrap();

    assert_eq!(seen_b(&world), BEFORE, "same phase: nothing applied yet");
    assert_eq!(seen_c(&world), AFTER, "next phase: the validation has run");
    assert!(!world.is_alive(t.a));
    assert!(has_position(&mut world, t.b));
    assert!(!has_velocity(&mut world, t.c));
    assert!(buffer.is_empty());
}

#[test]
fn the_observer_declared_first_sees_the_same_state() {
    // Reversing the declaration changes the order, and changes nothing of what is seen: the
    // structural effects are exposed by the validation only.
    type List = Cons<At<0, ObserveB>, Cons<At<0, Emit>, Cons<At<1, ObserveC>, Nil>>>;
    let scheduler = Scheduler::<TickBinding, List, 3, 2>::new().unwrap();
    assert_eq!(scheduler.model().phase(0), Some(&[0, 1][..]));

    let (mut world, _) = tick_world();
    scheduler
        .run_tick(&mut world, &mut MotionBuffer::new())
        .unwrap();
    assert_eq!(seen_b(&world), BEFORE);
    assert_eq!(seen_c(&world), AFTER);
}

// ---------------------------------------------------------------------------
// (5b) Real dependency through `FLAG`: still invisible in the phase
// ---------------------------------------------------------------------------

#[test]
fn a_real_dependency_orders_the_observer_after_the_emitter_but_exposes_nothing() {
    let emitter = contract_of::<TickBinding, EmitFlag>();
    let observer = contract_of::<TickBinding, ObserveFlag>();
    assert!(precedes(emitter, observer), "edge through FLAG");
    assert!(!precedes(observer, emitter));

    // The observer is declared first and runs second, by the deduced dependency.
    type List = Cons<At<0, ObserveFlag>, Cons<At<0, EmitFlag>, Cons<At<1, ObserveC>, Nil>>>;
    let scheduler = Scheduler::<TickBinding, List, 3, 2>::new().unwrap();
    assert_eq!(scheduler.model().phase(0), Some(&[1, 0][..]));

    let (mut world, t) = tick_world();
    let mut buffer = MotionBuffer::new();
    scheduler.run_tick(&mut world, &mut buffer).unwrap();

    assert_eq!(cell(&world, |g| &g.flag), 1, "the emitter ran");
    assert_eq!(
        seen_b(&world),
        BEFORE,
        "ran after the emitter, saw nothing of it"
    );
    assert_eq!(seen_c(&world), AFTER);
    assert!(!world.is_alive(t.a));
    assert!(buffer.is_empty());
}

// ---------------------------------------------------------------------------
// (6) Failure of the validation: the batch is atomic, nothing else is
// ---------------------------------------------------------------------------

type ThreePhases = Cons<At<0, ValidPhase>, Cons<At<1, FailingPhase>, Cons<At<2, Witness>, Nil>>>;

#[test]
fn a_rejected_batch_leaves_the_previous_phases_and_the_values_in_place() {
    let scheduler = Scheduler::<TickBinding, ThreePhases, 3, 3>::new().unwrap();
    let (mut world, t) = tick_world();
    let mut buffer = MotionBuffer::new();

    let result = scheduler.run_tick(&mut world, &mut buffer);

    assert_eq!(
        result,
        Err(TickError::Apply(ApplyError::Add(
            InsertError::AlreadyPresent
        )))
    );
    assert!(
        !world.is_alive(t.a),
        "(i) phase 0 was validated, not restored"
    );
    assert_eq!(
        velocity_of(&mut world, t.c),
        Some(35),
        "(ii) value of phase 0 kept"
    );
    assert_eq!(value(&world), 7, "(iii) value of phase 1 kept, not undone");
    assert!(
        world.is_alive(t.d),
        "(iv) the rejected batch applied nothing"
    );
    assert!(
        has_position(&mut world, t.d),
        "(iv) and the components of `d` are there"
    );
    assert_eq!(buffer.len(), 2, "(v) the batch is kept, not cleared");
    assert_eq!(witness(&world), 0, "(vi) phase 2 did not run");

    // The same observation holds for the next tick: the guard refuses it.
    assert_eq!(
        scheduler.run_tick(&mut world, &mut buffer),
        Err(TickError::BufferNotEmpty)
    );
    assert_eq!(buffer.len(), 2);
    assert_eq!(velocity_of(&mut world, t.c), Some(35), "nothing ran again");
}

// ---------------------------------------------------------------------------
// (7) `BufferFull` during the execution
// ---------------------------------------------------------------------------

#[test]
fn a_full_buffer_stops_the_tick_without_applying_anything() {
    type List = Cons<
        At<0, First>,
        Cons<At<0, Overfill>, Cons<At<0, Witness>, Cons<At<1, LaterWitness>, Nil>>>,
    >;
    let scheduler = Scheduler::<TickBinding, List, 4, 2>::new().unwrap();
    assert_eq!(scheduler.model().phase(0), Some(&[0, 1, 2][..]));

    let (mut world, t) = tick_world();
    let mut buffer = MotionBuffer::new();

    assert_eq!(
        scheduler.run_tick(&mut world, &mut buffer),
        Err(TickError::Buffer(BufferFull))
    );
    for entity in [t.a, t.b, t.c, t.d] {
        assert!(world.is_alive(entity), "apply was not called");
    }
    assert_eq!(value(&world), 11, "the value written by S1 is kept");
    assert_eq!(
        buffer.len(),
        4,
        "accepted commands are kept: the buffer is full"
    );
    assert_eq!(witness(&world), 0, "S3 of the same phase did not run");
    assert_eq!(seen_c(&world), 0, "the next phase did not run");
    assert_eq!(tally(&world), 1);

    // Retry without `clear`: refused before any system.
    assert_eq!(
        scheduler.run_tick(&mut world, &mut buffer),
        Err(TickError::BufferNotEmpty)
    );
    assert_eq!(tally(&world), 1, "no system ran");
    assert_eq!(buffer.len(), 4);

    // After `clear`, the guard is passed and the tick fails again, the same way.
    buffer.clear();
    assert_eq!(
        scheduler.run_tick(&mut world, &mut buffer),
        Err(TickError::Buffer(BufferFull))
    );
    assert_eq!(tally(&world), 2, "S1 ran again");
    assert_eq!(buffer.len(), 4);
}

// ---------------------------------------------------------------------------
// (8) Binding fault on the second system of a phase
// ---------------------------------------------------------------------------

#[test]
fn a_binding_fault_keeps_the_command_of_the_first_system_and_applies_nothing() {
    type List = Cons<At<0, EmitThenFault>, Cons<At<0, Witness>, Cons<At<1, LaterWitness>, Nil>>>;
    let scheduler = Scheduler::<DefectiveBinding, List, 3, 2>::new().unwrap();
    assert_eq!(scheduler.model().phase(0), Some(&[0, 1][..]));

    let (mut world, t) = tick_world();
    let mut buffer = MotionBuffer::new();

    assert_eq!(
        scheduler.run_tick(&mut world, &mut buffer),
        Err(TickError::Binding)
    );
    assert_eq!(value(&world), 11, "S1 ran and its value is kept");
    assert_eq!(buffer.len(), 1, "its command is retained in the buffer");
    assert!(world.is_alive(t.a), "and was not applied");
    assert_eq!(witness(&world), 0, "S2 did not run");
    assert_eq!(seen_c(&world), 0, "the next phase did not run");

    assert_eq!(
        scheduler.run_tick(&mut world, &mut buffer),
        Err(TickError::BufferNotEmpty)
    );
    assert_eq!(buffer.len(), 1);
}

// ---------------------------------------------------------------------------
// (9) Entry guard
// ---------------------------------------------------------------------------

#[test]
fn a_buffer_filled_by_the_caller_is_refused_before_any_system() {
    type List = Cons<At<0, Tally>, Nil>;
    let scheduler = Scheduler::<TickBinding, List, 1, 1>::new().unwrap();
    let (mut world, t) = tick_world();
    let mut buffer = MotionBuffer::new();
    buffer.despawn(t.a).unwrap();

    assert_eq!(
        scheduler.run_tick(&mut world, &mut buffer),
        Err(TickError::BufferNotEmpty)
    );
    assert_eq!(tally(&world), 0, "no system ran");
    assert!(world.is_alive(t.a), "the world is unchanged");
    assert_eq!(buffer.len(), 1, "the buffer is unchanged");

    // Once emptied, the same tick goes through.
    buffer.clear();
    scheduler.run_tick(&mut world, &mut buffer).unwrap();
    assert_eq!(tally(&world), 1);
}

// ---------------------------------------------------------------------------
// (10) A successful tick leaves the buffer empty
// ---------------------------------------------------------------------------

#[test]
fn a_successful_tick_empties_the_buffer_even_with_phases_without_commands() {
    // Phase 0 emits, phase 1 has no system at all, phase 2 has a system that emits nothing.
    type List = Cons<At<0, Emit>, Cons<At<2, Tally>, Nil>>;
    let scheduler = Scheduler::<TickBinding, List, 2, 3>::new().unwrap();
    assert_eq!(scheduler.model().phase(1), Some(&[][..]));

    let (mut world, t) = tick_world();
    let mut buffer = MotionBuffer::new();
    scheduler.run_tick(&mut world, &mut buffer).unwrap();
    assert!(buffer.is_empty());
    assert!(!world.is_alive(t.a), "the commands of phase 0 were applied");
    assert_eq!(tally(&world), 1);

    // A tick with no command at all also leaves it empty, and can be repeated.
    type Quiet = Cons<At<0, Tally>, Nil>;
    let quiet = Scheduler::<TickBinding, Quiet, 1, 1>::new().unwrap();
    for expected in 2..=3 {
        quiet.run_tick(&mut world, &mut buffer).unwrap();
        assert!(buffer.is_empty());
        assert_eq!(tally(&world), expected);
    }
}

/// The validation of the last phase is part of the tick: its commands are applied, and the
/// buffer is empty, before `run_tick` returns.
#[test]
fn the_commands_of_the_last_phase_are_applied_before_the_tick_returns() {
    type List = Cons<At<0, Tally>, Cons<At<1, Emit>, Nil>>;
    let scheduler = Scheduler::<TickBinding, List, 2, 2>::new().unwrap();
    let (mut world, t) = tick_world();
    let mut buffer = MotionBuffer::new();

    scheduler.run_tick(&mut world, &mut buffer).unwrap();

    assert!(!world.is_alive(t.a), "despawn of the last phase applied");
    assert!(
        has_position(&mut world, t.b),
        "add of the last phase applied"
    );
    assert!(
        !has_velocity(&mut world, t.c),
        "remove of the last phase applied"
    );
    assert!(buffer.is_empty());
}
