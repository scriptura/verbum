// crates/schedule/src/order.rs

//! Dependencies deduced from contracts (ADR-004, point 3; I5, I8).
//!
//! A dependency is deduced from a *production → consumption* relation
//! between the contracts of two **distinct** systems of one phase. It is
//! never declared by a system, and it never comes from structural effects
//! (ADR-006, point 4). Depends on `contract` only.
//!
//! [`order_phase`] turns the contracts of one phase into a deterministic
//! execution order. No ordering exists when the deduced dependencies close
//! into a cycle: that is a construction error, never a runtime one
//! (ADR-004, point 4).
//!
//! [`check_ownership`] enforces the single-owner rules over the systems of
//! a model: one writer per persistent category within a phase, and a
//! category is either written or produced, never both (I9, I11).

use crate::contract::Contract;
use crate::data::DataSet;

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

/// Why the contracts of a phase could not be ordered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderError {
    /// The deduced dependencies form a cycle: no valid order exists
    /// (ADR-004, point 4).
    Cycle,
    /// The output slice does not have exactly one slot per contract.
    LengthMismatch,
}

/// Orders the systems of one phase from their contracts alone.
///
/// On success, `order` holds the positions `0..contracts.len()` of the
/// systems in execution order: every system comes after all the systems it
/// depends on (see [`has_edge`]). Among the systems that are free to run,
/// the one declared first is chosen, so the result is the smallest valid
/// order in declaration terms. It depends on the contracts and their
/// declaration order only (ADR-004, point 5).
///
/// `order` must have exactly `contracts.len()` slots, otherwise
/// [`OrderError::LengthMismatch`]. On error, the content of `order` is
/// unspecified. There is no allocation and no ceiling on the number of
/// systems: the caller provides the output.
///
/// Cost: at most `n⁴` elementary steps for `n` systems, not measured. It
/// runs once, at construction time, and is confined behind this function.
pub fn order_phase(contracts: &[Contract], order: &mut [usize]) -> Result<(), OrderError> {
    let count = contracts.len();
    if order.len() != count {
        return Err(OrderError::LengthMismatch);
    }
    let mut placed = 0;
    while placed < count {
        let mut next = None;
        for candidate in 0..count {
            if order[..placed].contains(&candidate) {
                continue;
            }
            let blocked = (0..count).any(|other| {
                !order[..placed].contains(&other) && has_edge(contracts, other, candidate)
            });
            if !blocked {
                next = Some(candidate);
                break;
            }
        }
        match next {
            Some(candidate) => {
                order[placed] = candidate;
                placed += 1;
            }
            None => return Err(OrderError::Cycle),
        }
    }
    Ok(())
}

/// A violation of the single-owner rules (I9, I11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnershipError {
    /// A category is written by some system and produced by some system
    /// (possibly the same one), anywhere in the model. A category is
    /// either persistent (written) or transient (produced): its nature is
    /// one of the axes of the category, not a property of a phase.
    MixedNature,
    /// Two distinct systems of the same phase write the same category:
    /// a persistent category has one writer per phase.
    MultipleWriters,
}

/// Checks the single-owner rules over `systems`, given as
/// `(phase, contract)` pairs.
///
/// Checks are made in a fixed order, and the first violation is reported:
/// 1. [`OwnershipError::MixedNature`], over the whole model;
/// 2. [`OwnershipError::MultipleWriters`], within each phase only: the same
///    category may be written by systems of different phases.
///
/// Any number of systems may produce the same category (free production),
/// and any number may read it. No allocation.
///
/// Cost: quadratic in the number of systems, not measured; it runs once, at
/// construction time.
pub fn check_ownership(systems: &[(usize, Contract)]) -> Result<(), OwnershipError> {
    let mut written = DataSet::EMPTY;
    let mut produced = DataSet::EMPTY;
    for (_, contract) in systems {
        written = written.union(contract.writes());
        produced = produced.union(contract.produces());
    }
    if written.intersects(produced) {
        return Err(OwnershipError::MixedNature);
    }
    for (index, (phase, contract)) in systems.iter().enumerate() {
        for (other_phase, other) in &systems[index + 1..] {
            if phase == other_phase && contract.writes().intersects(other.writes()) {
                return Err(OwnershipError::MultipleWriters);
            }
        }
    }
    Ok(())
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

    // ---- order_phase -----------------------------------------------------

    fn ordered<const N: usize>(contracts: &[Contract; N]) -> Result<[usize; N], OrderError> {
        let mut order = [usize::MAX; N];
        order_phase(contracts, &mut order).map(|()| order)
    }

    #[test]
    fn a_chain_declared_backwards_runs_in_dependency_order() {
        let (x, y) = (DataSet::of(0), DataSet::of(1));
        let first = produces(x);
        let middle = data(x, E, y);
        let last = reads(y);
        assert_eq!(ordered(&[last, middle, first]), Ok([2, 1, 0]));
        assert_eq!(ordered(&[first, middle, last]), Ok([0, 1, 2]));
        assert_eq!(ordered(&[middle, last, first]), Ok([2, 0, 1]));
    }

    #[test]
    fn without_any_constraint_the_order_is_the_declaration_order() {
        let (x, y) = (DataSet::of(0), DataSet::of(1));
        assert_eq!(ordered(&[reads(x), reads(x), reads(y)]), Ok([0, 1, 2]));
        assert_eq!(ordered(&[writes(x), produces(x), writes(y)]), Ok([0, 1, 2]));
        assert_eq!(ordered::<0>(&[]), Ok([]));
        assert_eq!(ordered(&[reads(x)]), Ok([0]));
    }

    #[test]
    fn among_free_systems_the_first_declared_is_chosen() {
        let x = DataSet::of(0);
        // 0 must wait for 1; 1 and 2 are free, 1 is declared first.
        assert_eq!(
            ordered(&[reads(x), writes(x), reads(DataSet::of(1))]),
            Ok([1, 0, 2])
        );
        // 0 must wait for 2; 0 is blocked, so 1 runs first, then 2, then 0.
        assert_eq!(
            ordered(&[reads(x), reads(DataSet::of(1)), writes(x)]),
            Ok([1, 2, 0])
        );
    }

    #[test]
    fn the_order_depends_on_contracts_alone_never_on_effects() {
        let x = DataSet::of(0);
        let plain = [reads(x), writes(x), reads(DataSet::of(1))];
        let noisy = plain.map(|c| {
            Contract::new(
                c.reads(),
                c.writes(),
                c.produces(),
                StructuralEffects::new(true, true, true, true),
            )
        });
        assert_eq!(ordered(&plain), ordered(&noisy));
        assert_eq!(ordered(&plain), ordered(&plain));
    }

    #[test]
    fn a_two_system_cycle_is_a_construction_error() {
        let x = DataSet::of(0);
        assert_eq!(
            ordered(&[data(x, E, x), data(x, E, x)]),
            Err(OrderError::Cycle)
        );
        let (a, b) = (DataSet::of(0), DataSet::of(1));
        assert_eq!(
            ordered(&[data(a, E, b), data(b, E, a)]),
            Err(OrderError::Cycle)
        );
    }

    #[test]
    fn a_three_system_cycle_is_a_construction_error() {
        let (x, y, z) = (DataSet::of(0), DataSet::of(1), DataSet::of(2));
        let cycle = [data(z, E, x), data(x, E, y), data(y, E, z)];
        assert_eq!(ordered(&cycle), Err(OrderError::Cycle));
        // Order of declaration does not matter.
        assert_eq!(
            ordered(&[cycle[2], cycle[0], cycle[1]]),
            Err(OrderError::Cycle)
        );
    }

    #[test]
    fn a_cycle_is_detected_among_otherwise_orderable_systems() {
        let (x, y) = (DataSet::of(0), DataSet::of(1));
        let free = reads(DataSet::of(5));
        let cycle = [data(x, E, y), data(y, E, x)];
        assert_eq!(ordered(&[free, cycle[0], cycle[1]]), Err(OrderError::Cycle));
        assert_eq!(ordered(&[cycle[0], free, cycle[1]]), Err(OrderError::Cycle));
        assert_eq!(ordered(&[cycle[0], cycle[1], free]), Err(OrderError::Cycle));
    }

    #[test]
    fn a_system_reading_what_it_writes_or_produces_is_orderable() {
        // Not a cycle: a system does not constrain its own order.
        let x = DataSet::of(0);
        assert_eq!(ordered(&[data(x, x, E)]), Ok([0]));
        assert_eq!(ordered(&[data(x, E, x)]), Ok([0]));
        assert_eq!(ordered(&[data(x, x, x), reads(DataSet::of(1))]), Ok([0, 1]));
    }

    #[test]
    fn the_output_must_have_one_slot_per_contract() {
        let contracts = [reads(DataSet::of(0)), reads(DataSet::of(1))];
        assert_eq!(
            order_phase(&contracts, &mut [0; 1]),
            Err(OrderError::LengthMismatch)
        );
        assert_eq!(
            order_phase(&contracts, &mut [0; 3]),
            Err(OrderError::LengthMismatch)
        );
        assert_eq!(
            order_phase(&[], &mut [0; 1]),
            Err(OrderError::LengthMismatch)
        );
        assert_eq!(order_phase(&contracts, &mut [9; 2]), Ok(()));
    }

    /// A chain over the categories: system `k` reads category `k - 1` and
    /// produces category `k`. Only one order is valid, the chain order,
    /// whatever the declaration order.
    ///
    /// Volumes. Ordinary suite: 64 systems on the 64 categories, the full
    /// capacity of a `DataSet`. Under Miri: 8 systems on 8 categories. The
    /// code path is the same (a chain of dependencies resolved against four
    /// declaration orders); only the scale is smaller, because interpreting
    /// the ordering of 64 chained systems is disproportionately slow. The
    /// ordinary suite must never be reduced.
    #[test]
    fn a_chain_over_every_category_is_ordered_whatever_its_declaration() {
        const LEN: usize = if cfg!(miri) {
            8
        } else {
            DataSet::CAPACITY as usize
        };
        // `chain[k]` is system `k`.
        let mut chain = [reads(E); LEN];
        for (k, slot) in chain.iter_mut().enumerate() {
            let reads_previous = if k == 0 { E } else { DataSet::of(k as u8 - 1) };
            *slot = data(reads_previous, E, DataSet::of(k as u8));
        }
        // `declarations[d][j]` is the system declared at position `j`.
        let declarations: [[usize; LEN]; 4] = [
            // forward
            core::array::from_fn(|j| j),
            // backward
            core::array::from_fn(|j| LEN - 1 - j),
            // rotated by a third of the chain
            core::array::from_fn(|j| (j + LEN / 3) % LEN),
            // even systems first, then odd ones
            core::array::from_fn(|j| {
                if j < LEN / 2 {
                    2 * j
                } else {
                    2 * (j - LEN / 2) + 1
                }
            }),
        ];
        for declaration in declarations {
            for system in 0..LEN {
                assert_eq!(declaration.iter().filter(|&&s| s == system).count(), 1);
            }
            let declared: [Contract; LEN] = core::array::from_fn(|j| chain[declaration[j]]);
            // The k-th system to run is system k, found where it was declared.
            let expected: [usize; LEN] = core::array::from_fn(|k| {
                declaration.iter().position(|&system| system == k).unwrap()
            });
            assert_eq!(ordered(&declared), Ok(expected), "{declaration:?}");
        }
    }

    /// The number of systems of a phase is not bounded by the 64 categories.
    ///
    /// Volumes. Ordinary suite: 200 independent systems. Under Miri: 65,
    /// still above the 64 categories, which is all this test has to show;
    /// the systems are independent, so no dependency is exercised at scale
    /// (this is a sizing test, not a stress test of the ordering). The
    /// ordinary suite must never be reduced.
    #[test]
    fn the_number_of_systems_is_not_capped_by_the_number_of_categories() {
        const LEN: usize = if cfg!(miri) { 65 } else { 200 };
        const _: () = assert!(LEN > DataSet::CAPACITY as usize);
        let contracts = [reads(DataSet::of(0)); LEN];
        let order = ordered(&contracts).unwrap();
        for (position, system) in order.iter().enumerate() {
            assert_eq!(position, *system);
        }
    }

    /// Exhaustive comparison with an oracle over every phase of three
    /// systems, two categories, every `reads` x `produces` combination
    /// (16 contracts, so 16^3 = 4096 phases). The oracle enumerates the six
    /// permutations in lexicographic order and takes the first that respects
    /// every dependency; no such permutation means a cycle.
    ///
    /// Under Miri, only every 7th phase is explored (about 586 of 4096; 7 is
    /// coprime with 4096, so the sample spreads over all three contracts).
    /// The ordinary suite explores all of them and must never be reduced.
    #[test]
    fn ordering_matches_an_oracle_over_all_small_phases() {
        const PERMUTATIONS: [[usize; 3]; 6] = [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ];
        let set = |bits: u8| {
            let mut set = E;
            for category in 0..2u8 {
                if bits & (1 << category) != 0 {
                    set = set.union(DataSet::of(category));
                }
            }
            set
        };
        let stride = if cfg!(miri) { 7 } else { 1 };
        let (mut orderable, mut cyclic) = (0, 0);
        for combination in (0..16 * 16 * 16).step_by(stride) {
            let choice = [combination % 16, combination / 16 % 16, combination / 256];
            let reads_bits = choice.map(|c| (c % 4) as u8);
            let produces_bits = choice.map(|c| (c / 4) as u8);
            let contracts: [Contract; 3] =
                core::array::from_fn(|i| data(set(reads_bits[i]), E, set(produces_bits[i])));

            let depends =
                |from: usize, to: usize| from != to && produces_bits[from] & reads_bits[to] != 0;
            let expected = PERMUTATIONS.iter().find(|permutation| {
                let position = |system: usize| permutation.iter().position(|&s| s == system);
                (0..3).all(|from| {
                    (0..3).all(|to| !depends(from, to) || position(from) < position(to))
                })
            });
            match expected {
                Some(permutation) => {
                    orderable += 1;
                    assert_eq!(ordered(&contracts), Ok(*permutation), "{choice:?}");
                }
                None => {
                    cyclic += 1;
                    assert_eq!(ordered(&contracts), Err(OrderError::Cycle), "{choice:?}");
                }
            }
        }
        assert!(orderable > 0 && cyclic > 0);
    }

    // ---- phase 2 complement: legitimate edges of a self-reading system ----

    #[test]
    fn a_self_reading_system_keeps_its_edges_towards_other_systems() {
        let x = DataSet::of(0);
        // A reads and writes X (or reads and produces X); B reads X.
        for a in [data(x, x, E), data(x, E, x)] {
            let contracts = [a, reads(x)];
            assert!(has_edge(&contracts, 0, 1), "A -> B must exist");
            assert!(!has_edge(&contracts, 0, 0), "A -> A must not exist");
            assert!(!has_edge(&contracts, 1, 1));
            assert!(!has_edge(&contracts, 1, 0), "B produces nothing");
            // Declared backwards, A still runs first.
            assert_eq!(ordered(&[reads(x), a]), Ok([1, 0]));
            assert_eq!(ordered(&contracts), Ok([0, 1]));
        }
    }

    // ---- check_ownership -------------------------------------------------

    const POSITION: u8 = 0;
    const ACCUMULATOR: u8 = 1;

    #[test]
    fn three_producers_one_resolver_and_one_reader_are_accepted_and_ordered() {
        let (position, accumulator) = (DataSet::of(POSITION), DataSet::of(ACCUMULATOR));
        let producer = produces(accumulator);
        let resolver = data(accumulator, position, E);
        let reader = reads(position);
        let systems = [
            (0, reader),
            (0, resolver),
            (0, producer),
            (0, producer),
            (0, producer),
        ];
        assert_eq!(check_ownership(&systems), Ok(()));
        // Producers (2, 3, 4) first, then the resolver (1), then the reader (0).
        let contracts = systems.map(|(_, contract)| contract);
        assert_eq!(ordered(&contracts), Ok([2, 3, 4, 1, 0]));
    }

    #[test]
    fn any_number_of_producers_of_one_category_is_accepted() {
        // More than the 64 categories: production is free.
        const LEN: usize = if cfg!(miri) { 70 } else { 200 };
        let systems = [(0, produces(DataSet::of(0))); LEN];
        assert_eq!(check_ownership(&systems), Ok(()));
    }

    #[test]
    fn empty_and_single_system_models_are_accepted() {
        assert_eq!(check_ownership(&[]), Ok(()));
        assert_eq!(check_ownership(&[(0, writes(DataSet::of(0)))]), Ok(()));
        assert_eq!(
            check_ownership(&[(3, data(DataSet::of(0), DataSet::of(0), E))]),
            Ok(())
        );
    }

    #[test]
    fn two_writers_of_one_category_in_one_phase_are_rejected() {
        let x = DataSet::of(0);
        assert_eq!(
            check_ownership(&[(0, writes(x)), (0, writes(x))]),
            Err(OwnershipError::MultipleWriters)
        );
        // Whatever else they declare, and wherever they sit in the list.
        let both = writes(x.union(DataSet::of(1)));
        assert_eq!(
            check_ownership(&[(2, both), (2, writes(DataSet::of(5))), (2, writes(x))]),
            Err(OwnershipError::MultipleWriters)
        );
    }

    #[test]
    fn writers_of_one_category_in_different_phases_are_accepted() {
        let x = DataSet::of(0);
        assert_eq!(check_ownership(&[(0, writes(x)), (1, writes(x))]), Ok(()));
        // Distinct categories in one phase are accepted too.
        assert_eq!(
            check_ownership(&[(0, writes(x)), (0, writes(DataSet::of(1)))]),
            Ok(())
        );
        // A third writer in a phase already holding one is rejected.
        assert_eq!(
            check_ownership(&[(0, writes(x)), (1, writes(x)), (1, writes(x))]),
            Err(OwnershipError::MultipleWriters)
        );
    }

    #[test]
    fn a_category_written_and_produced_is_rejected_wherever_they_sit() {
        let x = DataSet::of(0);
        // Same phase, different phases, and one system doing both.
        assert_eq!(
            check_ownership(&[(0, writes(x)), (0, produces(x))]),
            Err(OwnershipError::MixedNature)
        );
        assert_eq!(
            check_ownership(&[(0, writes(x)), (1, produces(x))]),
            Err(OwnershipError::MixedNature)
        );
        assert_eq!(
            check_ownership(&[(4, produces(x)), (0, writes(x))]),
            Err(OwnershipError::MixedNature)
        );
        assert_eq!(
            check_ownership(&[(0, data(E, x, x))]),
            Err(OwnershipError::MixedNature)
        );
    }

    #[test]
    fn reading_a_category_never_changes_its_nature() {
        let x = DataSet::of(0);
        // Reads of a written category, reads of a produced category, and a
        // system that reads one category while writing another.
        assert_eq!(
            check_ownership(&[(0, writes(x)), (0, reads(x)), (1, reads(x))]),
            Ok(())
        );
        assert_eq!(
            check_ownership(&[(0, produces(x)), (0, reads(x)), (1, reads(x))]),
            Ok(())
        );
        assert_eq!(
            check_ownership(&[(0, data(x, DataSet::of(1), E)), (0, produces(x))]),
            Ok(())
        );
    }

    #[test]
    fn mixed_nature_is_reported_before_multiple_writers() {
        let (x, y) = (DataSet::of(0), DataSet::of(1));
        // Both violations present, whatever the order of declaration.
        let both = [
            (0, writes(x)),
            (0, writes(x)),
            (0, writes(y)),
            (1, produces(y)),
        ];
        assert_eq!(check_ownership(&both), Err(OwnershipError::MixedNature));
        let reversed = [both[3], both[2], both[1], both[0]];
        assert_eq!(check_ownership(&reversed), Err(OwnershipError::MixedNature));
        // Remove the mixed nature: the other violation remains.
        assert_eq!(
            check_ownership(&both[..3]),
            Err(OwnershipError::MultipleWriters)
        );
    }

    #[test]
    fn two_producers_reading_what_they_produce_pass_ownership_but_form_a_cycle() {
        // Ownership and ordering are separate checks: free production is
        // accepted here, the mutual dependency is rejected as a cycle.
        let x = DataSet::of(0);
        let contract = data(x, E, x);
        assert_eq!(check_ownership(&[(0, contract), (0, contract)]), Ok(()));
        assert_eq!(ordered(&[contract, contract]), Err(OrderError::Cycle));
    }

    /// Exhaustive comparison with an oracle over every model of three
    /// systems: each in phase 0 or 1, writing a subset of two categories and
    /// producing a subset of two categories (16 contracts, 8 phase
    /// assignments, so 16^3 x 8 = 32 768 models). The oracle works category
    /// by category: mixed nature if some system writes it and some system
    /// produces it; otherwise multiple writers if two systems of one phase
    /// write it; otherwise accepted.
    ///
    /// Under Miri, only every 37th model is explored (about 886 of 32 768;
    /// 37 is coprime with 32 768, so the sample spreads over all fields).
    /// The ordinary suite explores all of them and must never be reduced.
    #[test]
    fn ownership_matches_an_oracle_over_all_small_models() {
        let set = |bits: u8| {
            let mut set = E;
            for category in 0..2u8 {
                if bits & (1 << category) != 0 {
                    set = set.union(DataSet::of(category));
                }
            }
            set
        };
        let stride = if cfg!(miri) { 37 } else { 1 };
        let (mut accepted, mut mixed, mut multiple) = (0, 0, 0);
        for model in (0..16 * 16 * 16 * 8).step_by(stride) {
            let choice = [model % 16, model / 16 % 16, model / 256 % 16];
            let phases = [model / 4096 % 2, model / 8192 % 2, model / 16384 % 2];
            let writes_bits = choice.map(|c| (c % 4) as u8);
            let produces_bits = choice.map(|c| (c / 4) as u8);
            let systems: [(usize, Contract); 3] = core::array::from_fn(|i| {
                (
                    phases[i],
                    data(E, set(writes_bits[i]), set(produces_bits[i])),
                )
            });

            let in_system =
                |bits: &[u8; 3], system: usize, category: u8| bits[system] & (1 << category) != 0;
            let mut mixed_nature = false;
            let mut multiple_writers = false;
            for category in 0..2u8 {
                let written = (0..3).any(|s| in_system(&writes_bits, s, category));
                let produced = (0..3).any(|s| in_system(&produces_bits, s, category));
                mixed_nature |= written && produced;
                for a in 0..3 {
                    for b in a + 1..3 {
                        multiple_writers |= phases[a] == phases[b]
                            && in_system(&writes_bits, a, category)
                            && in_system(&writes_bits, b, category);
                    }
                }
            }
            let expected = if mixed_nature {
                mixed += 1;
                Err(OwnershipError::MixedNature)
            } else if multiple_writers {
                multiple += 1;
                Err(OwnershipError::MultipleWriters)
            } else {
                accepted += 1;
                Ok(())
            };
            assert_eq!(check_ownership(&systems), expected, "{choice:?} {phases:?}");
        }
        assert!(accepted > 0 && mixed > 0 && multiple > 0);
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
