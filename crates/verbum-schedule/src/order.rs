// crates/verbum-schedule/src/order.rs

//! Dependencies deduced from contracts (ADR-004, point 3; I5, I8).
//!
//! A dependency is deduced from a *production → consumption* relation
//! between the contracts of two **distinct** systems of one phase. It is
//! never declared by a system, and it never comes from structural effects
//! (ADR-006, point 4). Depends on `contract` only.

use crate::contract::Contract;

/// Whether `producer` produces or writes a category that `consumer` reads.
///
/// This is a relation between two contracts, not between two systems: it
/// says nothing about identity. `precedes(c, c)` is true when `c` reads what
/// it also writes or produces; that is *not* a dependency, because a system
/// does not constrain its own order. Use [`has_edge`] to decide whether an
/// ordering constraint exists between two positions of a phase.
///
/// Structural effects are ignored by construction (ADR-006, point 4).
pub const fn precedes(producer: Contract, consumer: Contract) -> bool {
    producer
        .writes()
        .union(producer.produces())
        .intersects(consumer.reads())
}

/// Whether the system at `from` must run before the system at `to`, among
/// the contracts of one phase.
///
/// An edge only exists between two distinct positions: `from == to` is
/// never an edge, even when the system reads what it writes or produces.
///
/// Panics if `from` or `to` is out of range.
pub fn has_edge(contracts: &[Contract], from: usize, to: usize) -> bool {
    let producer = contracts[from];
    let consumer = contracts[to];
    from != to && precedes(producer, consumer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::StructuralEffects;
    use crate::data::DataSet;

    const E: DataSet = DataSet::EMPTY;

    fn data(reads: DataSet, writes: DataSet, produces: DataSet) -> Contract {
        Contract::new(reads, writes, produces, StructuralEffects::NONE)
    }

    fn reads(set: DataSet) -> Contract {
        data(set, E, E)
    }

    fn writes(set: DataSet) -> Contract {
        data(E, set, E)
    }

    fn produces(set: DataSet) -> Contract {
        data(E, E, set)
    }

    #[test]
    fn a_write_then_a_read_of_the_same_category_is_a_dependency() {
        let x = DataSet::of(0);
        assert!(precedes(writes(x), reads(x)));
        assert!(!precedes(reads(x), writes(x)));
    }

    #[test]
    fn a_production_then_a_read_of_the_same_category_is_a_dependency() {
        let x = DataSet::of(0);
        assert!(precedes(produces(x), reads(x)));
        assert!(!precedes(reads(x), produces(x)));
    }

    #[test]
    fn categories_must_match_for_a_dependency() {
        assert!(!precedes(writes(DataSet::of(0)), reads(DataSet::of(1))));
        assert!(!precedes(produces(DataSet::of(0)), reads(DataSet::of(1))));
    }

    #[test]
    fn a_shared_category_among_several_is_enough() {
        let producer = writes(DataSet::of(0).union(DataSet::of(5)));
        let consumer = reads(DataSet::of(5).union(DataSet::of(9)));
        assert!(precedes(producer, consumer));
    }

    #[test]
    fn write_only_read_only_two_readers_and_two_producers_create_no_edge() {
        let x = DataSet::of(0);
        // Write-only against write-only, produce-only against produce-only,
        // write against produce: no consumer, no dependency (ADR-004, point 3).
        for (a, b) in [
            (writes(x), writes(x)),
            (produces(x), produces(x)),
            (writes(x), produces(x)),
            (produces(x), writes(x)),
            (reads(x), reads(x)),
        ] {
            assert!(!precedes(a, b));
            assert!(!precedes(b, a));
        }
    }

    #[test]
    fn structural_effects_never_influence_a_dependency() {
        let x = DataSet::of(0);
        for bits_a in 0..16u8 {
            for bits_b in 0..16u8 {
                let effects = |bits: u8| {
                    StructuralEffects::new(
                        bits & 1 != 0,
                        bits & 2 != 0,
                        bits & 4 != 0,
                        bits & 8 != 0,
                    )
                };
                let producer = Contract::new(E, x, E, effects(bits_a));
                let consumer = Contract::new(x, E, E, effects(bits_b));
                let unrelated = Contract::new(E, E, E, effects(bits_b));
                assert!(precedes(producer, consumer));
                assert!(!precedes(producer, unrelated));
                assert!(!precedes(unrelated, producer));
            }
        }
    }

    #[test]
    fn precedes_matches_its_definition_over_all_small_contracts() {
        // Two categories; every combination of reads, writes and produces.
        let subsets = [
            E,
            DataSet::of(0),
            DataSet::of(1),
            DataSet::of(0).union(DataSet::of(1)),
        ];
        let mut all = [reads(E); 64];
        let mut n = 0;
        for r in subsets {
            for w in subsets {
                for p in subsets {
                    all[n] = data(r, w, p);
                    n += 1;
                }
            }
        }
        assert_eq!(n, 64);
        for a in all {
            for b in all {
                let mut expected = false;
                for category in 0..2u8 {
                    let c = DataSet::of(category);
                    let written = a.writes().intersects(c) || a.produces().intersects(c);
                    expected |= written && b.reads().intersects(c);
                }
                assert_eq!(precedes(a, b), expected);
            }
        }
    }

    #[test]
    fn a_system_is_never_its_own_dependency() {
        let x = DataSet::of(0);
        // Reads what it writes, reads what it produces, both: the contract
        // relation holds, the edge does not.
        for contract in [data(x, x, E), data(x, E, x), data(x, x, x)] {
            assert!(precedes(contract, contract));
            assert!(!has_edge(&[contract], 0, 0));
        }
    }

    #[test]
    fn a_self_reading_system_alone_has_no_edge_and_adds_none_to_others() {
        let x = DataSet::of(0);
        let y = DataSet::of(1);
        let contracts = [data(x, x, E), reads(y), writes(y)];
        for from in 0..3 {
            for to in 0..3 {
                // Only the independent pair (1 reads y, 2 writes y) is related.
                assert_eq!(has_edge(&contracts, from, to), (from, to) == (2, 1));
            }
        }
    }

    #[test]
    fn edges_between_distinct_systems_follow_the_contract_relation() {
        let x = DataSet::of(0);
        let contracts = [writes(x), reads(x), reads(x)];
        assert!(has_edge(&contracts, 0, 1));
        assert!(has_edge(&contracts, 0, 2));
        assert!(!has_edge(&contracts, 1, 0));
        assert!(!has_edge(&contracts, 2, 0));
        assert!(!has_edge(&contracts, 1, 2));
        assert!(!has_edge(&contracts, 2, 1));
    }

    #[test]
    fn two_distinct_systems_that_read_and_produce_the_same_category_depend_on_each_other() {
        // The mutual dependency is deduced here; rejecting it as a cycle is
        // the job of the ordering, not of the edge relation.
        let x = DataSet::of(0);
        let contracts = [data(x, E, x), data(x, E, x)];
        assert!(has_edge(&contracts, 0, 1));
        assert!(has_edge(&contracts, 1, 0));
        assert!(!has_edge(&contracts, 0, 0));
        assert!(!has_edge(&contracts, 1, 1));
    }

    #[test]
    fn identical_contracts_at_distinct_positions_are_distinct_systems() {
        let x = DataSet::of(0);
        let contracts = [data(x, x, E), data(x, x, E)];
        assert!(has_edge(&contracts, 0, 1));
        assert!(has_edge(&contracts, 1, 0));
    }

    #[test]
    #[should_panic]
    fn an_out_of_range_position_panics_even_for_a_self_pair() {
        let contracts = [reads(DataSet::of(0))];
        let _ = has_edge(&contracts, 1, 1);
    }

    // Compile-time proof that `precedes` is usable in a `const` context.
    const _: () = {
        const X: DataSet = DataSet::of(0);
        let producer = Contract::new(E, X, E, StructuralEffects::NONE);
        let consumer = Contract::new(X, E, E, StructuralEffects::NONE);
        assert!(precedes(producer, consumer));
        assert!(!precedes(consumer, producer));
    };
}
