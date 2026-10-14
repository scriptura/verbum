// crates/verbum-schedule/tests/access.rs

//! Phase 7a: declarative views, identity by category, contract computed from the type.
//!
//! Real `World` (`split_mut`), `Locate` written outside the crates, public API only. The
//! compile-time prohibitions (plan, §2.6) are `compile_fail` doctests of
//! `verbum_schedule::access`.

mod support;

use verbum_ecs::{EntityId, ReadWriteQuery};
use verbum_schedule::access::{Bind, Locate, Part, Produce, Read, Write};
use verbum_schedule::contract::Contract;
use verbum_schedule::data::DataSet;

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
