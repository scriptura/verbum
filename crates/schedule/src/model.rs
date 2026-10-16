// crates/schedule/src/model.rs

//! Immutable execution model, built from contracts alone (ADR-004,
//! points 1, 4, 5; I6, I10).
//!
//! [`ExecutionModel::build`] receives `(phase, contract)` pairs: integers
//! and contracts, never a function or a system identity. The position in the
//! input is only the index under which a system is reported back. The
//! result owns its storage inline and has no mutating method: once built, it
//! can only be read. Depends on `order` and `contract` only.
//!
//! **Local choice (capacities).** The model is sized by two const generics,
//! `SYSTEMS` and `PHASES`, fixed by the caller. No ceiling is added by this
//! crate beyond the one imposed by the type system, and nothing is
//! allocated: the construction uses stack arrays of `SYSTEMS` entries, and
//! a system whose phase is `>= PHASES` is rejected, never dropped.
//!
//! ```compile_fail,E0594
//! use verbum_schedule::model::ExecutionModel;
//!
//! let model = ExecutionModel::<0, 1>::build([]).unwrap();
//! // The model exposes shared slices only: an order cannot be rewritten.
//! model.phase(0).unwrap()[0] = 1;
//! ```
//!
//! ```compile_fail,E0596
//! use verbum_schedule::model::ExecutionModel;
//!
//! let model = ExecutionModel::<0, 1>::build([]).unwrap();
//! // A built model is not borrowed mutably by any operation.
//! let _ = model.phase(0).unwrap().first_mut();
//! ```

use crate::contract::{Contract, StructuralEffects};
use crate::data::DataSet;
use crate::order::{OrderError, OwnershipError, check_ownership, order_phase};

/// A violation detected while building an [`ExecutionModel`].
///
/// Checks are made in this order, and the first violation is reported:
/// [`MixedNature`](Self::MixedNature), [`MultipleWriters`](Self::MultipleWriters),
/// [`UnknownPhase`](Self::UnknownPhase), [`Cycle`](Self::Cycle).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelError {
    /// A category is both written and produced somewhere in the model.
    MixedNature,
    /// Two systems of one phase write the same category.
    MultipleWriters,
    /// A system names a phase `>= PHASES`.
    UnknownPhase,
    /// The dependencies deduced inside one phase form a cycle.
    Cycle,
}

/// The execution order of every phase, fixed at construction.
///
/// `SYSTEMS` is the number of systems and `PHASES` the number of phases.
/// Two models built from equal contract lists are equal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutionModel<const SYSTEMS: usize, const PHASES: usize> {
    /// System indices, phase after phase, each phase in execution order.
    order: [usize; SYSTEMS],
    /// Offset of the first system of each phase inside `order`.
    starts: [usize; PHASES],
    /// Number of systems of each phase.
    lens: [usize; PHASES],
}

impl<const SYSTEMS: usize, const PHASES: usize> ExecutionModel<SYSTEMS, PHASES> {
    /// Builds the model from `(phase, contract)` pairs, one per system, in
    /// declaration order.
    ///
    /// Controls, in this order: [`ModelError::MixedNature`] (whole model),
    /// [`ModelError::MultipleWriters`] (per phase),
    /// [`ModelError::UnknownPhase`], then [`ModelError::Cycle`] (per phase).
    /// A cycle spread over two phases is not a cycle: dependencies exist
    /// inside a phase only.
    ///
    /// Within a phase, systems with no constraint between them keep their
    /// declaration order (see [`order_phase`]). No allocation.
    pub fn build(systems: [(usize, Contract); SYSTEMS]) -> Result<Self, ModelError> {
        match check_ownership(&systems) {
            Ok(()) => {}
            Err(OwnershipError::MixedNature) => return Err(ModelError::MixedNature),
            Err(OwnershipError::MultipleWriters) => return Err(ModelError::MultipleWriters),
        }
        if systems.iter().any(|(phase, _)| *phase >= PHASES) {
            return Err(ModelError::UnknownPhase);
        }

        let filler = Contract::new(
            DataSet::EMPTY,
            DataSet::EMPTY,
            DataSet::EMPTY,
            StructuralEffects::NONE,
        );
        let mut order = [0; SYSTEMS];
        let mut starts = [0; PHASES];
        let mut lens = [0; PHASES];
        let mut next = 0;

        for phase in 0..PHASES {
            // Contracts of this phase, in declaration order, with the index
            // of the system each one belongs to.
            let mut contracts = [filler; SYSTEMS];
            let mut indices = [0; SYSTEMS];
            let mut local_order = [0; SYSTEMS];
            let mut count = 0;
            for (index, (system_phase, contract)) in systems.iter().enumerate() {
                if *system_phase == phase {
                    contracts[count] = *contract;
                    indices[count] = index;
                    count += 1;
                }
            }

            match order_phase(&contracts[..count], &mut local_order[..count]) {
                Ok(()) => {}
                Err(OrderError::Cycle) => return Err(ModelError::Cycle),
                // Both slices are cut to `count` just above, so their
                // lengths are equal by construction.
                Err(OrderError::LengthMismatch) => {
                    unreachable!("slices of one phase have the same length")
                }
            }

            starts[phase] = next;
            lens[phase] = count;
            for (slot, local) in local_order[..count].iter().enumerate() {
                order[next + slot] = indices[*local];
            }
            next += count;
        }

        Ok(Self {
            order,
            starts,
            lens,
        })
    }

    /// System indices of `phase`, in execution order; empty when no system
    /// belongs to it, `None` when `phase >= PHASES`.
    pub fn phase(&self, phase: usize) -> Option<&[usize]> {
        let start = *self.starts.get(phase)?;
        let len = *self.lens.get(phase)?;
        self.order.get(start..start + len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const E: DataSet = DataSet::EMPTY;
    const A: DataSet = DataSet::of(0);
    const B: DataSet = DataSet::of(1);
    const C: DataSet = DataSet::of(2);
    const D: DataSet = DataSet::of(3);

    fn contract(reads: DataSet, writes: DataSet, produces: DataSet) -> Contract {
        Contract::new(reads, writes, produces, StructuralEffects::NONE)
    }

    fn free() -> Contract {
        contract(E, E, E)
    }

    #[test]
    fn a_chain_declared_backwards_is_reordered_inside_its_phase() {
        let model = ExecutionModel::<3, 1>::build([
            (0, contract(B, E, E)),
            (0, contract(A, E, B)),
            (0, contract(E, E, A)),
        ])
        .unwrap();
        assert_eq!(model.phase(0), Some(&[2, 1, 0][..]));
    }

    #[test]
    fn indices_are_declaration_indices_and_phases_do_not_mix() {
        // Phase 0: systems 1, 3, 4 (3 -> 4 -> 1 once ordered);
        // phase 1: systems 0, 2 (free, declaration order).
        let model = ExecutionModel::<5, 2>::build([
            (1, free()),
            (0, contract(B, E, E)),
            (1, free()),
            (0, contract(E, E, A)),
            (0, contract(A, E, B)),
        ])
        .unwrap();
        assert_eq!(model.phase(0), Some(&[3, 4, 1][..]));
        assert_eq!(model.phase(1), Some(&[0, 2][..]));
    }

    #[test]
    fn an_empty_phase_is_known_and_empty() {
        let model = ExecutionModel::<2, 3>::build([(0, free()), (2, free())]).unwrap();
        assert_eq!(model.phase(0), Some(&[0][..]));
        assert_eq!(model.phase(1), Some(&[][..]));
        assert_eq!(model.phase(2), Some(&[1][..]));
    }

    #[test]
    fn a_model_without_system_has_empty_phases() {
        let model = ExecutionModel::<0, 2>::build([]).unwrap();
        assert_eq!(model.phase(0), Some(&[][..]));
        assert_eq!(model.phase(1), Some(&[][..]));
    }

    #[test]
    fn a_phase_beyond_the_model_has_no_order() {
        let model = ExecutionModel::<1, 2>::build([(0, free())]).unwrap();
        assert_eq!(model.phase(2), None);
        assert_eq!(model.phase(usize::MAX), None);
    }

    #[test]
    fn a_system_in_an_unknown_phase_is_rejected_not_dropped() {
        let result = ExecutionModel::<2, 2>::build([(0, free()), (2, free())]);
        assert_eq!(result, Err(ModelError::UnknownPhase));
        let result = ExecutionModel::<1, 0>::build([(0, free())]);
        assert_eq!(result, Err(ModelError::UnknownPhase));
    }

    #[test]
    fn a_cycle_inside_one_phase_is_rejected() {
        let result =
            ExecutionModel::<2, 1>::build([(0, contract(A, E, B)), (0, contract(B, E, A))]);
        assert_eq!(result, Err(ModelError::Cycle));
    }

    #[test]
    fn a_cycle_spread_over_two_phases_is_accepted() {
        let model = ExecutionModel::<2, 2>::build([(0, contract(A, E, B)), (1, contract(B, E, A))])
            .unwrap();
        assert_eq!(model.phase(0), Some(&[0][..]));
        assert_eq!(model.phase(1), Some(&[1][..]));
    }

    #[test]
    fn equal_contract_lists_give_equal_models() {
        let list = [
            (1, contract(B, E, E)),
            (0, contract(A, E, B)),
            (1, contract(E, E, A)),
        ];
        let first = ExecutionModel::<3, 2>::build(list).unwrap();
        let second = ExecutionModel::<3, 2>::build(list).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.phase(0), second.phase(0));
        assert_eq!(first.phase(1), second.phase(1));
    }

    #[test]
    fn different_contract_lists_give_different_models() {
        let first = ExecutionModel::<2, 2>::build([(0, free()), (1, free())]).unwrap();
        let moved = ExecutionModel::<2, 2>::build([(1, free()), (0, free())]).unwrap();
        assert_ne!(first, moved);
        let chained =
            ExecutionModel::<2, 1>::build([(0, contract(A, E, E)), (0, contract(E, E, A))])
                .unwrap();
        let free_pair = ExecutionModel::<2, 1>::build([(0, free()), (0, free())]).unwrap();
        assert_ne!(chained, free_pair);
    }

    #[test]
    fn controls_follow_the_normative_order() {
        // Systems 0, 1: two writers of A in phase 0 (MultipleWriters).
        // Systems 2, 3: B written in phase 0 and produced in phase 1
        //               (MixedNature, across phases).
        // System 4: phase 9 (UnknownPhase), the model has two phases.
        // Systems 5, 6: a cycle in phase 1 (Cycle).
        let mut list = [
            (0, contract(E, A, E)),
            (0, contract(E, A, E)),
            (0, contract(E, B, E)),
            (1, contract(E, E, B)),
            (9, free()),
            (1, contract(C, E, D)),
            (1, contract(D, E, C)),
        ];
        assert_eq!(
            ExecutionModel::<7, 2>::build(list),
            Err(ModelError::MixedNature)
        );
        list[3] = (1, free());
        assert_eq!(
            ExecutionModel::<7, 2>::build(list),
            Err(ModelError::MultipleWriters)
        );
        list[1] = (0, free());
        assert_eq!(
            ExecutionModel::<7, 2>::build(list),
            Err(ModelError::UnknownPhase)
        );
        list[4] = (0, free());
        assert_eq!(ExecutionModel::<7, 2>::build(list), Err(ModelError::Cycle));
        list[6] = (1, free());
        assert!(ExecutionModel::<7, 2>::build(list).is_ok());
    }

    #[test]
    fn mixed_nature_is_reported_before_an_unknown_phase_it_hides() {
        let result =
            ExecutionModel::<2, 1>::build([(0, contract(E, A, E)), (5, contract(E, E, A))]);
        assert_eq!(result, Err(ModelError::MixedNature));
    }

    #[test]
    fn the_number_of_systems_is_not_capped() {
        // Volumes: 200 systems in ordinary runs, 65 under Miri (more than
        // the 64 categories, cheap configuration, not a stress test).
        // The ordinary volume must never be reduced.
        const LEN: usize = if cfg!(miri) { 65 } else { 200 };
        const _: () = assert!(LEN > DataSet::CAPACITY as usize);
        let model = ExecutionModel::<LEN, 2>::build([(1, free()); LEN]).unwrap();
        assert_eq!(model.phase(0), Some(&[][..]));
        let phase = model.phase(1).unwrap();
        assert_eq!(phase.len(), LEN);
        assert!(phase.iter().enumerate().all(|(slot, index)| slot == *index));
    }
}
