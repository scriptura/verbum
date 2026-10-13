// crates/verbum-schedule/src/contract.rs

//! Declarative interface of a system (ADR-003, point 1; ADR-006, point 3).
//!
//! A [`Contract`] is pure data: it answers what a system reads, writes,
//! produces and which structural commands it may emit, without any
//! reference to the system's code (I1). Depends on `data` only.

use crate::data::DataSet;

/// The structural commands a system may emit (ADR-006, point 3).
///
/// Four independent indicators: `Spawn`, `Destroy`, `AddComponent`,
/// `RemoveComponent`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StructuralEffects {
    spawns: bool,
    destroys: bool,
    adds_component: bool,
    removes_component: bool,
}

impl StructuralEffects {
    /// No structural command.
    pub const NONE: Self = Self::new(false, false, false, false);

    /// Builds the indicators, in the order `Spawn`, `Destroy`,
    /// `AddComponent`, `RemoveComponent`.
    pub const fn new(
        spawns: bool,
        destroys: bool,
        adds_component: bool,
        removes_component: bool,
    ) -> Self {
        Self {
            spawns,
            destroys,
            adds_component,
            removes_component,
        }
    }

    /// May emit `Spawn`.
    pub const fn spawns(self) -> bool {
        self.spawns
    }

    /// May emit `Destroy`.
    pub const fn destroys(self) -> bool {
        self.destroys
    }

    /// May emit `AddComponent`.
    pub const fn adds_component(self) -> bool {
        self.adds_component
    }

    /// May emit `RemoveComponent`.
    pub const fn removes_component(self) -> bool {
        self.removes_component
    }
}

/// What a system reads, writes and produces, and the structural commands
/// it may emit. Never written by hand for a system once typed access
/// exists (plan, phase 7a); here it is plain, inspectable data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Contract {
    reads: DataSet,
    writes: DataSet,
    produces: DataSet,
    effects: StructuralEffects,
}

impl Contract {
    /// Builds a contract from its four components.
    pub const fn new(
        reads: DataSet,
        writes: DataSet,
        produces: DataSet,
        effects: StructuralEffects,
    ) -> Self {
        Self {
            reads,
            writes,
            produces,
            effects,
        }
    }

    /// Categories read.
    pub const fn reads(self) -> DataSet {
        self.reads
    }

    /// Persistent categories written (single owner per phase, I9).
    pub const fn writes(self) -> DataSet {
        self.writes
    }

    /// Transient categories produced (free production, I9).
    pub const fn produces(self) -> DataSet {
        self.produces
    }

    /// Structural commands that may be emitted.
    pub const fn effects(self) -> StructuralEffects {
        self.effects
    }

    /// Whether the system may emit at least one structural command.
    /// Derived from the four indicators, never stored.
    pub const fn emits_commands(self) -> bool {
        let e = self.effects;
        e.spawns || e.destroys || e.adds_component || e.removes_component
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags(effects: StructuralEffects) -> [bool; 4] {
        [
            effects.spawns(),
            effects.destroys(),
            effects.adds_component(),
            effects.removes_component(),
        ]
    }

    fn only_effect(position: usize) -> StructuralEffects {
        let on = |i: usize| i == position;
        StructuralEffects::new(on(0), on(1), on(2), on(3))
    }

    fn with_effects(effects: StructuralEffects) -> Contract {
        Contract::new(DataSet::EMPTY, DataSet::EMPTY, DataSet::EMPTY, effects)
    }

    fn data_only(reads: DataSet, writes: DataSet, produces: DataSet) -> Contract {
        Contract::new(reads, writes, produces, StructuralEffects::NONE)
    }

    #[test]
    fn contract_answers_the_questions_of_adr_003_by_inspection_alone() {
        let contract = Contract::new(
            DataSet::of(0),
            DataSet::of(1),
            DataSet::of(2),
            StructuralEffects::new(true, true, false, false),
        );
        assert_eq!(contract.reads(), DataSet::of(0));
        assert_eq!(contract.writes(), DataSet::of(1));
        assert_eq!(contract.produces(), DataSet::of(2));
        assert!(contract.effects().spawns());
        assert!(contract.effects().destroys());
        assert!(contract.emits_commands());
    }

    #[test]
    fn each_effect_indicator_is_independent() {
        assert_eq!(flags(StructuralEffects::NONE), [false; 4]);
        for position in 0..4 {
            let mut expected = [false; 4];
            expected[position] = true;
            assert_eq!(flags(only_effect(position)), expected);
        }
    }

    #[test]
    fn emits_commands_is_true_for_each_effect_and_false_for_none() {
        for position in 0..4 {
            assert!(with_effects(only_effect(position)).emits_commands());
        }
        let quiet = Contract::new(
            DataSet::of(0),
            DataSet::of(1),
            DataSet::of(2),
            StructuralEffects::NONE,
        );
        assert!(!quiet.emits_commands());
    }

    #[test]
    fn equality_is_structural_and_every_field_counts() {
        let a = DataSet::of(0);
        let e = DataSet::EMPTY;
        assert_eq!(data_only(a, e, e), data_only(a, e, e));
        assert_ne!(data_only(a, e, e), data_only(e, a, e));
        assert_ne!(data_only(e, a, e), data_only(e, e, a));
        assert_ne!(data_only(a, e, e), data_only(e, e, a));
        assert_ne!(
            with_effects(StructuralEffects::NONE),
            with_effects(only_effect(0))
        );
    }

    #[test]
    fn contract_is_buildable_in_const() {
        const CONTRACT: Contract = Contract::new(
            DataSet::of(0),
            DataSet::EMPTY,
            DataSet::of(63),
            StructuralEffects::new(false, false, true, false),
        );
        assert_eq!(CONTRACT.reads(), DataSet::of(0));
        assert_eq!(CONTRACT.writes(), DataSet::EMPTY);
        assert_eq!(CONTRACT.produces(), DataSet::of(63));
        assert!(CONTRACT.emits_commands());
    }
}
