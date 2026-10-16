// crates/schedule/tests/access.rs

//! Phase 7a: declarative views, identity by category, contract computed from the type.
//!
//! Real `World` (`split_mut`), `Locate` written outside the crates, public API only. The
//! compile-time prohibitions (plan, §2.6) are `compile_fail` doctests of
//! `verbum_schedule::access`.

mod support;

use verbum_ecs::{BufferFull, CapacityExceeded, EntityId, ReadWriteQuery};
use verbum_schedule::access::{
    Bind, COMMANDS_BINDING, Commands, Locate, Part, Produce, Read, Write,
};
use verbum_schedule::contract::{Contract, StructuralEffects};
use verbum_schedule::data::{DataId, DataSet};
use verbum_schedule::model::ExecutionModel;
use verbum_schedule::order::{has_edge, precedes};

use support::*;

/// Binds an access from any assembly: exactly what the Scheduler will do.
fn bind<'w, P, A: Bind<'w, P>>(parts: &mut P) -> Option<A> {
    A::bind(parts)
}

fn ids(indices: &[u8]) -> DataSet {
    indices
        .iter()
        .fold(DataSet::EMPTY, |set, &index| set.union(DataSet::of(index)))
}

// ---------------------------------------------------------------------------------------------
// Binding from a real World
// ---------------------------------------------------------------------------------------------

/// Access of the test system: two components and one resource.
type Resolve<'w> = (
    ReadOf<'w, ACCUMULATOR>,
    ReadOf<'w, VELOCITY>,
    WriteOf<'w, POSITION>,
);

fn resolve((accumulator, velocities, mut positions): Resolve<'_>, entities: [EntityId; 3]) {
    let bonus = accumulator.get().unwrap().0;
    for entity in entities {
        if let Some(velocity) = velocities.get(entity) {
            positions.get_mut(entity).unwrap().0 += velocity.0 + bonus;
        }
    }
}

#[test]
fn binds_two_components_and_a_resource_from_a_real_world() {
    let (mut world, entities) = populated_world();
    let [a, b, c] = entities;
    {
        let mut parts = Parts::from_world(&mut world);
        let access: Resolve<'_> = bind(&mut parts).expect("all slots are free");
        resolve(access, entities);
    }
    let positions = &world.components().positions;
    assert_eq!(positions.get(a), Some(&Position(1 + 10 + 100)));
    assert_eq!(
        positions.get(b),
        Some(&Position(2)),
        "no velocity: unchanged"
    );
    assert_eq!(positions.get(c), Some(&Position(3 + 30 + 100)));
    assert_eq!(world.resources().accumulator.get(), Some(&Accumulator(100)));
}

#[test]
fn a_fresh_parts_binds_again_once_the_previous_access_is_gone() {
    let (mut world, _) = populated_world();
    for _ in 0..2 {
        let mut parts = Parts::from_world(&mut world);
        let access: Option<Resolve<'_>> = bind(&mut parts);
        assert!(access.is_some());
    }
}

// ---------------------------------------------------------------------------------------------
// Capabilities: Read reads, Write and Produce write
// ---------------------------------------------------------------------------------------------

#[test]
fn read_reads_components_and_resources() {
    let (mut world, [a, b, c]) = populated_world();
    let mut parts = Parts::from_world(&mut world);
    let velocities: ReadOf<'_, VELOCITY> = bind(&mut parts).unwrap();
    let counter: ReadOf<'_, COUNTER_A> = bind(&mut parts).unwrap();

    assert_eq!((velocities.len(), velocities.is_empty()), (2, false));
    assert_eq!(velocities.entities(), [a, c]);
    assert!(velocities.contains(a) && !velocities.contains(b));
    assert_eq!(velocities.get(c), Some(&Velocity(30)));
    assert_eq!(velocities.get(b), None);
    assert_eq!(velocities.shared().get(a), Some(&Velocity(10)));
    assert_eq!(counter.get(), Some(&Counter(10)));
}

#[test]
fn write_modifies_components_and_resources_visibly_in_the_world() {
    let (mut world, [a, b, c]) = populated_world();
    let without_position = world.spawn().unwrap();
    {
        let mut parts = Parts::from_world(&mut world);
        let mut positions: WriteOf<'_, POSITION> = bind(&mut parts).unwrap();
        let mut accumulator: WriteOf<'_, ACCUMULATOR> = bind(&mut parts).unwrap();

        positions.get_mut(a).unwrap().0 = 50;
        assert_eq!(positions.get(a), Some(&Position(50)));
        assert_eq!(
            positions.get_mut(without_position),
            None,
            "absent: nothing to write"
        );
        accumulator.get_mut().unwrap().0 += 1;
        assert_eq!(accumulator.insert(Accumulator(7)), Some(Accumulator(101)));
        assert_eq!(accumulator.get(), Some(&Accumulator(7)));
    }
    let positions = &world.components().positions;
    assert_eq!(positions.get(a), Some(&Position(50)));
    assert_eq!(positions.get(b), Some(&Position(2)));
    assert_eq!(positions.get(c), Some(&Position(3)));
    assert_eq!(world.resources().accumulator.get(), Some(&Accumulator(7)));
}

#[test]
fn produce_writes_components_and_resources_visibly_in_the_world() {
    let (mut world, [a, ..]) = populated_world();
    {
        let mut parts = Parts::from_world(&mut world);
        let mut velocities: ProduceOf<'_, VELOCITY> = bind(&mut parts).unwrap();
        let mut accumulator: ProduceOf<'_, ACCUMULATOR> = bind(&mut parts).unwrap();

        velocities.get_mut(a).unwrap().0 = -1;
        assert_eq!(accumulator.insert(Accumulator(5)), Some(Accumulator(100)));
        accumulator.get_mut().unwrap().0 += 1;
    }
    assert_eq!(world.components().velocities.get(a), Some(&Velocity(-1)));
    assert_eq!(world.resources().accumulator.get(), Some(&Accumulator(6)));
}

#[test]
fn a_resource_view_over_an_empty_slot_reads_none_and_write_can_fill_it() {
    let mut world = MotionWorld::new();
    {
        let mut parts = Parts::from_world(&mut world);
        let read: ReadOf<'_, ACCUMULATOR> = bind(&mut parts).unwrap();
        assert_eq!(read.get(), None);
    }
    {
        let mut parts = Parts::from_world(&mut world);
        let mut write: WriteOf<'_, ACCUMULATOR> = bind(&mut parts).unwrap();
        assert_eq!(write.get_mut(), None);
        assert_eq!(write.insert(Accumulator(1)), None);
    }
    assert_eq!(world.resources().accumulator.get(), Some(&Accumulator(1)));
}

#[test]
fn read_write_query_is_built_from_read_and_write() {
    let (mut world, [a, b, c]) = populated_world();
    let mut visited = [None; 3];
    let mut count = 0;
    {
        let mut parts = Parts::from_world(&mut world);
        let (positions, velocities): (ReadOf<'_, POSITION>, WriteOf<'_, VELOCITY>) =
            bind(&mut parts).unwrap();

        let mut query = ReadWriteQuery::new(positions.shared(), velocities.into_values());
        while let Some((entity, position, velocity)) = query.advance() {
            velocity.0 += position.0;
            visited[count] = Some(entity);
            count += 1;
        }
    }
    assert_eq!((visited, count), ([Some(a), Some(c), None], 2));
    let velocities = &world.components().velocities;
    assert_eq!(velocities.get(a), Some(&Velocity(11)));
    assert_eq!(velocities.get(b), None);
    assert_eq!(velocities.get(c), Some(&Velocity(33)));
}

// ---------------------------------------------------------------------------------------------
// Contract computed from the type (evaluated in const)
// ---------------------------------------------------------------------------------------------

const RESOLVE: Contract = <Resolve<'static> as Part>::CONTRACT;

#[test]
fn contract_of_a_real_access_type_is_derived_in_const() {
    assert_eq!(RESOLVE.reads(), ids(&[ACCUMULATOR, VELOCITY]));
    assert_eq!(RESOLVE.writes(), ids(&[POSITION]));
    assert_eq!(RESOLVE.produces(), DataSet::EMPTY);
    assert!(!RESOLVE.emits_commands());
}

#[test]
fn contract_of_each_leaf_view() {
    const READ: Contract = <Read<5, ()> as Part>::CONTRACT;
    const WRITE: Contract = <Write<6, ()> as Part>::CONTRACT;
    const PRODUCE: Contract = <Produce<7, ()> as Part>::CONTRACT;

    assert_eq!(
        (READ.reads(), READ.writes(), READ.produces()),
        (ids(&[5]), DataSet::EMPTY, DataSet::EMPTY)
    );
    assert_eq!(
        (WRITE.reads(), WRITE.writes(), WRITE.produces()),
        (DataSet::EMPTY, ids(&[6]), DataSet::EMPTY)
    );
    assert_eq!(
        (PRODUCE.reads(), PRODUCE.writes(), PRODUCE.produces()),
        (DataSet::EMPTY, DataSet::EMPTY, ids(&[7]))
    );
}

/// D10: `Write` and `Produce` add no `reads` to the contract.
#[test]
fn write_and_produce_add_no_reads() {
    const WRITE_ONLY: Contract = <(Write<1, ()>, Produce<2, ()>) as Part>::CONTRACT;
    assert_eq!(WRITE_ONLY.reads(), DataSet::EMPTY);
    assert_eq!(WRITE_ONLY.writes(), ids(&[1]));
    assert_eq!(WRITE_ONLY.produces(), ids(&[2]));
}

#[test]
fn tuples_of_every_arity_and_the_unit_derive_their_contract() {
    const UNIT: Contract = <() as Part>::CONTRACT;
    const ONE: Contract = <(Read<0, ()>,) as Part>::CONTRACT;
    const TWO: Contract = <(Read<0, ()>, Write<1, ()>) as Part>::CONTRACT;
    const THREE: Contract = <(Read<0, ()>, Write<1, ()>, Produce<2, ()>) as Part>::CONTRACT;
    const FOUR: Contract =
        <(Read<0, ()>, Write<1, ()>, Produce<2, ()>, Read<3, ()>) as Part>::CONTRACT;

    assert_eq!(
        UNIT,
        Contract::new(
            DataSet::EMPTY,
            DataSet::EMPTY,
            DataSet::EMPTY,
            UNIT.effects()
        )
    );
    assert!(!UNIT.emits_commands());
    assert_eq!(ONE.reads(), ids(&[0]));
    assert_eq!((TWO.reads(), TWO.writes()), (ids(&[0]), ids(&[1])));
    assert_eq!(
        (THREE.reads(), THREE.writes(), THREE.produces()),
        (ids(&[0]), ids(&[1]), ids(&[2]))
    );
    assert_eq!(
        (FOUR.reads(), FOUR.writes(), FOUR.produces()),
        (ids(&[0, 3]), ids(&[1]), ids(&[2]))
    );
}

#[test]
fn nested_accesses_beyond_four_elements_derive_their_contract() {
    type Nested = (
        (Read<0, ()>, Write<1, ()>),
        (Produce<2, ()>, (Read<3, ()>, Write<4, ()>)),
        Read<63, ()>,
    );
    const NESTED: Contract = <Nested as Part>::CONTRACT;

    assert_eq!(NESTED.reads(), ids(&[0, 3, 63]));
    assert_eq!(NESTED.writes(), ids(&[1, 4]));
    assert_eq!(NESTED.produces(), ids(&[2]));
}

/// The identity is the integer of the view, never the stored value type (plan, §2.2).
#[test]
fn contract_follows_the_id_and_ignores_the_stored_type() {
    const A: Contract = <Read<0, u32> as Part>::CONTRACT;
    const B: Contract = <Read<1, u32> as Part>::CONTRACT;
    const C: Contract = <Read<0, &'static str> as Part>::CONTRACT;
    assert_ne!(A, B, "same value type, distinct categories");
    assert_eq!(A, C, "same category, distinct value types");
}

#[test]
fn the_last_valid_id_is_accepted_in_const() {
    const LAST: Contract = <Write<63, ()> as Part>::CONTRACT;
    assert_eq!(LAST.writes(), ids(&[63]));
}

// ---------------------------------------------------------------------------------------------
// Slot already taken: `None`, never a panic
// ---------------------------------------------------------------------------------------------

#[test]
fn a_slot_already_taken_binds_to_none() {
    let (mut world, _) = populated_world();
    let mut parts = Parts::from_world(&mut world);
    let first: Option<ReadOf<'_, ACCUMULATOR>> = bind(&mut parts);
    let second: Option<ReadOf<'_, ACCUMULATOR>> = bind(&mut parts);
    assert!(first.is_some());
    assert!(second.is_none());
    let third: Option<ReadOf<'_, ACCUMULATOR>> = bind(&mut parts);
    assert!(third.is_none(), "and it stays so");
}

#[test]
fn the_unit_access_binds_without_touching_the_parts() {
    let (mut world, _) = populated_world();
    let mut parts = Parts::from_world(&mut world);
    let unit: Option<()> = bind(&mut parts);
    let mixed: Option<(ReadOf<'_, ACCUMULATOR>, ())> = bind(&mut parts);
    assert_eq!(unit, Some(()));
    assert!(mixed.is_some(), "the unit consumed no slot");
}

/// Failure after a first successful extraction: the assembly stays partially consumed
/// (state not to be reused), the world is untouched, no panic.
#[test]
fn a_failed_composite_bind_leaves_earlier_slots_consumed_and_the_world_untouched() {
    let (mut world, [a, b, c]) = populated_world();
    {
        let mut parts = Parts::from_world(&mut world);
        let _held: WriteOf<'_, POSITION> = bind(&mut parts).unwrap();

        // (ACCUMULATOR free, POSITION already taken): the first is extracted, the second fails.
        let failing: Option<(ReadOf<'_, ACCUMULATOR>, WriteOf<'_, POSITION>)> = bind(&mut parts);
        assert!(failing.is_none());

        let accumulator: Option<ReadOf<'_, ACCUMULATOR>> = bind(&mut parts);
        assert!(accumulator.is_none(), "consumed by the partial extraction");
        let velocities: Option<ReadOf<'_, VELOCITY>> = bind(&mut parts);
        assert!(velocities.is_some(), "the other slots are intact");
    }
    let positions = &world.components().positions;
    assert_eq!(
        [positions.get(a), positions.get(b), positions.get(c)],
        [Some(&Position(1)), Some(&Position(2)), Some(&Position(3))]
    );
    assert_eq!(world.components().positions.len(), 3, "structure unchanged");
    assert_eq!(world.components().velocities.len(), 2);
    assert_eq!(world.resources().accumulator.get(), Some(&Accumulator(100)));
}

// ---------------------------------------------------------------------------------------------
// Wiring: two resources of the same concrete type, identifiers 3 and 4
// ---------------------------------------------------------------------------------------------

#[test]
fn writing_through_counter_a_does_not_change_what_counter_b_reads() {
    let (mut world, _) = populated_world();
    {
        let mut parts = Parts::from_world(&mut world);
        let mut a: WriteOf<'_, COUNTER_A> = bind(&mut parts).unwrap();
        let b: ReadOf<'_, COUNTER_B> = bind(&mut parts).unwrap();
        a.get_mut().unwrap().0 += 1;
        assert_eq!(b.get(), Some(&Counter(20)));
    }
    assert_eq!(world.resources().counter_a.get(), Some(&Counter(11)));
    assert_eq!(world.resources().counter_b.get(), Some(&Counter(20)));
}

#[test]
fn writing_through_counter_b_does_not_change_what_counter_a_reads() {
    let (mut world, _) = populated_world();
    {
        let mut parts = Parts::from_world(&mut world);
        let a: ReadOf<'_, COUNTER_A> = bind(&mut parts).unwrap();
        let mut b: WriteOf<'_, COUNTER_B> = bind(&mut parts).unwrap();
        b.get_mut().unwrap().0 += 5;
        assert_eq!(a.get(), Some(&Counter(10)));
    }
    assert_eq!(world.resources().counter_a.get(), Some(&Counter(10)));
    assert_eq!(world.resources().counter_b.get(), Some(&Counter(25)));
}

#[test]
fn each_counter_is_read_and_modified_independently() {
    let (mut world, _) = populated_world();
    {
        let mut parts = Parts::from_world(&mut world);
        let (mut a, mut b): (WriteOf<'_, COUNTER_A>, WriteOf<'_, COUNTER_B>) =
            bind(&mut parts).unwrap();
        a.get_mut().unwrap().0 *= 2;
        b.get_mut().unwrap().0 += 1;
        assert_eq!((a.get(), b.get()), (Some(&Counter(20)), Some(&Counter(21))));
    }
    assert_eq!(world.resources().counter_a.get(), Some(&Counter(20)));
    assert_eq!(world.resources().counter_b.get(), Some(&Counter(21)));
}

type Counters<'w> = Slot<'w, Counter>;

/// Wiring probe: increments `COUNTER_A` (and it alone) through `Write`, whatever the
/// assembly it is given.
fn increment_counter_a<'w, P>(parts: &mut P)
where
    P: Locate<'w, COUNTER_A, Store = Counters<'w>> + Locate<'w, COUNTER_B, Store = Counters<'w>>,
{
    let mut a: Write<COUNTER_A, Counters<'w>> = bind(parts).unwrap();
    a.get_mut().unwrap().0 += 1;
}

fn counters(world: &MotionWorld) -> (u32, u32) {
    let globals = world.resources();
    (
        globals.counter_a.get().unwrap().0,
        globals.counter_b.get().unwrap().0,
    )
}

/// Documentary witness (plan, §2.4 point 1): a crossed wiring compiles and behaves crosswise.
/// The same probe, run on the correct wiring, observes the expected wiring: the wiring test
/// therefore detects the error, and it alone.
#[test]
fn a_crossed_wiring_compiles_and_is_observable_by_the_wiring_probe() {
    let (mut correct, _) = populated_world();
    let (mut crossed, _) = populated_world();
    increment_counter_a(&mut Parts::from_world(&mut correct));
    increment_counter_a(&mut CrossedParts::from_world(&mut crossed));

    assert_eq!(counters(&correct), (11, 20), "A -> field A");
    assert_eq!(counters(&crossed), (10, 21), "crossed: A -> field B");
    assert_ne!(counters(&correct), counters(&crossed));
}

// ---------------------------------------------------------------------------------------------
// Phase 7b: typed command handle
// ---------------------------------------------------------------------------------------------

fn flags(contract: Contract) -> [bool; 4] {
    let effects = contract.effects();
    [
        effects.spawns(),
        effects.destroys(),
        effects.adds_component(),
        effects.removes_component(),
    ]
}

/// A contract made of one command handle, over a store irrelevant to the contract.
macro_rules! commands_contract {
    ($spawn:literal, $despawn:literal, $add:literal, $remove:literal) => {{
        const CONTRACT: Contract =
            <Commands<(), $spawn, $despawn, $add, $remove> as Part>::CONTRACT;
        CONTRACT
    }};
}

#[test]
fn the_effects_of_the_contract_are_the_indicators_of_the_type() {
    assert_eq!(
        flags(commands_contract!(false, false, false, false)),
        [false; 4]
    );
    assert_eq!(
        flags(commands_contract!(true, false, false, false)),
        [true, false, false, false]
    );
    assert_eq!(
        flags(commands_contract!(false, true, false, false)),
        [false, true, false, false]
    );
    assert_eq!(
        flags(commands_contract!(false, false, true, false)),
        [false, false, true, false]
    );
    assert_eq!(
        flags(commands_contract!(false, false, false, true)),
        [false, false, false, true]
    );
    assert_eq!(flags(commands_contract!(true, true, true, true)), [true; 4]);
    assert_eq!(
        flags(commands_contract!(true, false, true, true)),
        [true, false, true, true]
    );
}

#[test]
fn a_command_handle_adds_no_category_to_the_contract() {
    for contract in [
        commands_contract!(false, false, false, false),
        commands_contract!(true, true, true, true),
    ] {
        assert_eq!(contract.reads(), DataSet::EMPTY);
        assert_eq!(contract.writes(), DataSet::EMPTY);
        assert_eq!(contract.produces(), DataSet::EMPTY);
    }
    assert!(!commands_contract!(false, false, false, false).emits_commands());
    assert!(commands_contract!(false, false, false, true).emits_commands());
}

#[test]
fn a_command_handle_merges_with_data_views() {
    type Access = (
        Read<0, ()>,
        Write<1, ()>,
        Commands<(), true, false, true, false>,
        (Produce<2, ()>,),
    );
    const CONTRACT: Contract = <Access as Part>::CONTRACT;

    assert_eq!(CONTRACT.reads(), ids(&[0]));
    assert_eq!(CONTRACT.writes(), ids(&[1]));
    assert_eq!(CONTRACT.produces(), ids(&[2]));
    assert_eq!(flags(CONTRACT), [true, false, true, false]);
}

/// The binding key is not a category: outside `0..64`, never a `DataId`.
#[test]
fn the_binding_key_is_outside_the_data_id_space() {
    assert_eq!(COMMANDS_BINDING, u8::MAX);
    assert!(DataId::new(COMMANDS_BINDING).is_err());
    assert!((0..DataSet::CAPACITY).all(|category| category != COMMANDS_BINDING));
}

/// Commands produce no ordering edge, whatever the effects and the data of the other system.
#[test]
fn command_handles_create_no_edge_between_systems() {
    const SPAWNER: Contract = <Commands<(), true, false, false, false> as Part>::CONTRACT;
    const REMOVER: Contract = <Commands<(), false, false, false, true> as Part>::CONTRACT;
    const ALL: Contract = <Commands<(), true, true, true, true> as Part>::CONTRACT;
    const WRITER: Contract =
        <(Write<1, ()>, Commands<(), true, true, true, true>) as Part>::CONTRACT;
    const READER: Contract = <Read<1, ()> as Part>::CONTRACT;

    let contracts = [SPAWNER, REMOVER, ALL, WRITER];
    for from in 0..contracts.len() {
        for to in 0..contracts.len() {
            assert!(!has_edge(&contracts, from, to), "{from} -> {to}");
            assert!(!precedes(contracts[from], contracts[to]), "{from} -> {to}");
        }
    }
    // The only edge is the one of the data: the writer's category read by the reader.
    let mixed = [WRITER, READER];
    assert!(has_edge(&mixed, 0, 1) && !has_edge(&mixed, 1, 0));
}

#[test]
fn command_handles_leave_the_execution_order_untouched() {
    const A: Contract = <Commands<(), true, false, false, false> as Part>::CONTRACT;
    const B: Contract = <Commands<(), false, true, false, false> as Part>::CONTRACT;
    const C: Contract = <Commands<(), false, false, true, true> as Part>::CONTRACT;

    for (first, second, third) in [(A, B, C), (C, B, A), (B, A, C)] {
        let model = ExecutionModel::<3, 1>::build([(0, first), (0, second), (0, third)]).unwrap();
        assert_eq!(model.phase(0), Some([0, 1, 2].as_slice()));
    }
}

// --- emission, observed through `World::apply` ------------------------------------------------

type Everything<'w> = CommandsOf<'w, true, true, true, true>;

#[test]
fn a_spawn_emitted_through_commands_is_resolved_by_apply() {
    let (mut world, _) = populated_world(); // 3 of 4 entities
    let mut buffer = MotionBuffer::new();
    {
        let mut parts = Parts::from_world(&mut world).with_buffer(&mut buffer);
        let mut commands: Everything<'_> = bind(&mut parts).expect("buffer available");
        assert_eq!(commands.spawn(), Ok(()));
    }
    assert_eq!(buffer.len(), 1);
    assert_eq!(world.apply(&mut buffer), Ok(()));
    assert!(buffer.is_empty());
    assert_eq!(
        world.spawn(),
        Err(CapacityExceeded),
        "the fourth place was taken"
    );
}

#[test]
fn a_despawn_emitted_through_commands_is_resolved_by_apply() {
    let (mut world, [a, b, _]) = populated_world();
    let mut buffer = MotionBuffer::new();
    {
        let mut parts = Parts::from_world(&mut world).with_buffer(&mut buffer);
        let mut commands: Everything<'_> = bind(&mut parts).unwrap();
        commands.despawn(a).unwrap();
    }
    assert!(world.is_alive(a), "nothing before apply");
    world.apply(&mut buffer).unwrap();
    assert!(!world.is_alive(a) && world.is_alive(b));
    assert_eq!(world.components().positions.len(), 2, "purged");
}

#[test]
fn an_add_emitted_through_commands_is_resolved_by_apply() {
    let (mut world, [_, b, _]) = populated_world();
    let mut buffer = MotionBuffer::new();
    {
        let mut parts = Parts::from_world(&mut world).with_buffer(&mut buffer);
        let mut commands: Everything<'_> = bind(&mut parts).unwrap();
        commands
            .add_component(b, MotionAdd::Velocity(Velocity(7)))
            .unwrap();
    }
    assert_eq!(
        world.components().velocities.get(b),
        None,
        "nothing before apply"
    );
    world.apply(&mut buffer).unwrap();
    assert_eq!(world.components().velocities.get(b), Some(&Velocity(7)));
}

#[test]
fn a_removal_emitted_through_commands_reaches_the_buffer_of_the_ecs() {
    let (mut world, [a, _, c]) = populated_world();
    let mut buffer = MotionBuffer::new();
    {
        let mut parts = Parts::from_world(&mut world).with_buffer(&mut buffer);
        let mut commands: Everything<'_> = bind(&mut parts).unwrap();
        commands
            .remove_component(a, MotionRemove::Velocity)
            .unwrap();
    }
    assert_eq!(buffer.len(), 1);
    assert_eq!(
        world.components().velocities.get(a),
        Some(&Velocity(10)),
        "nothing before apply"
    );
    world.apply(&mut buffer).unwrap();
    let components = world.components();
    assert_eq!(components.velocities.get(a), None);
    assert_eq!(
        components.velocities.get(c),
        Some(&Velocity(30)),
        "only the target"
    );
    assert_eq!(
        components.positions.len(),
        3,
        "the other storage is untouched"
    );
}

#[test]
fn commands_bind_next_to_data_views_in_one_access() {
    let (mut world, [a, ..]) = populated_world();
    let mut buffer = MotionBuffer::new();
    {
        let mut parts = Parts::from_world(&mut world).with_buffer(&mut buffer);
        let (mut positions, mut commands): (
            WriteOf<'_, POSITION>,
            CommandsOf<'_, false, true, false, false>,
        ) = bind(&mut parts).unwrap();
        positions.get_mut(a).unwrap().0 = 99;
        commands.despawn(a).unwrap();
    }
    assert_eq!(world.components().positions.get(a), Some(&Position(99)));
    world.apply(&mut buffer).unwrap();
    assert!(!world.is_alive(a));
}

#[test]
fn overflow_is_reported_and_the_buffer_does_not_grow() {
    let (mut world, [a, ..]) = populated_world();
    let mut buffer = MotionBuffer::new(); // 4 commands
    {
        let mut parts = Parts::from_world(&mut world).with_buffer(&mut buffer);
        let mut commands: Everything<'_> = bind(&mut parts).unwrap();
        for _ in 0..4 {
            assert_eq!(commands.spawn(), Ok(()));
        }
        assert_eq!(commands.spawn(), Err(BufferFull));
        assert_eq!(commands.despawn(a), Err(BufferFull));
        assert_eq!(
            commands.add_component(a, MotionAdd::Position(Position(1))),
            Err(BufferFull)
        );
        assert_eq!(
            commands.remove_component(a, MotionRemove::Position),
            Err(BufferFull)
        );
    }
    assert_eq!(buffer.len(), 4);
}

#[test]
fn without_a_buffer_commands_bind_to_none() {
    let (mut world, _) = populated_world();
    let mut parts = Parts::from_world(&mut world);
    let commands: Option<Everything<'_>> = bind(&mut parts);
    assert!(commands.is_none());
}

#[test]
fn the_buffer_can_be_taken_once() {
    let (mut world, _) = populated_world();
    let mut buffer = MotionBuffer::new();
    let mut parts = Parts::from_world(&mut world).with_buffer(&mut buffer);
    let first: Option<Everything<'_>> = bind(&mut parts);
    let second: Option<Everything<'_>> = bind(&mut parts);
    assert!(first.is_some());
    assert!(second.is_none());
}

/// A handle with no declared effect is legal: it binds, and its contract has no effect.
#[test]
fn a_handle_without_effects_binds_and_declares_nothing() {
    let (mut world, _) = populated_world();
    let mut buffer = MotionBuffer::new();
    let mut parts = Parts::from_world(&mut world).with_buffer(&mut buffer);
    let handle: Option<CommandsOf<'_, false, false, false, false>> = bind(&mut parts);
    assert!(handle.is_some());
    assert_eq!(
        commands_contract!(false, false, false, false).effects(),
        StructuralEffects::NONE
    );
}
