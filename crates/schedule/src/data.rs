// crates/schedule/src/data.rs

//! Identity and set of data categories (ADR-003, ADR-005, I11, I15).
//!
//! The runtime reasons about *categories* of data, never about individual
//! items (`02-concepts.md`). A category is designated by a small integer;
//! a [`DataSet`] is a fixed-capacity set of categories held in one `u64`.
//!
//! Capacity is fixed before simulation (ADR-001, point 5): an index at or
//! above [`DataSet::CAPACITY`] is a configuration bug, never handled
//! silently. In a `const` context it is a compile-time error (E0080);
//! elsewhere, [`DataId::new`] returns an error.

/// Index of a data category, guaranteed to be below [`DataSet::CAPACITY`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataId(u8);

/// An index outside the capacity of a [`DataSet`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataIdOutOfRange;

impl DataId {
    /// Validates an index. Use this outside `const` contexts.
    pub const fn new(index: u8) -> Result<Self, DataIdOutOfRange> {
        if index < DataSet::CAPACITY {
            Ok(Self(index))
        } else {
            Err(DataIdOutOfRange)
        }
    }

    /// The validated index.
    pub const fn index(self) -> u8 {
        self.0
    }
}

/// A set of data categories. Plain data, `Copy`, no allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataSet(u64);

const _: () = assert!(DataSet::CAPACITY as u32 == u64::BITS);

impl DataSet {
    /// Number of distinct categories a set can hold.
    pub const CAPACITY: u8 = 64;

    /// The set with no category.
    pub const EMPTY: Self = Self(0);

    /// The set holding only category `index`.
    ///
    /// Panics if `index >= CAPACITY`. Evaluated in a `const` context, the
    /// panic is a compile-time error:
    ///
    /// ```compile_fail,E0080
    /// use verbum_schedule::data::DataSet;
    ///
    /// const _: DataSet = DataSet::of(64);
    /// ```
    pub const fn of(index: u8) -> Self {
        assert!(index < Self::CAPACITY, "data category index out of range");
        Self(1u64 << index)
    }

    /// This set plus category `id`.
    pub const fn with(self, id: DataId) -> Self {
        Self(self.0 | (1u64 << id.0))
    }

    /// Whether category `id` belongs to this set.
    pub const fn contains(self, id: DataId) -> bool {
        self.0 & (1u64 << id.0) != 0
    }

    /// The categories belonging to either set.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Whether the two sets share at least one category.
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(index: u8) -> DataId {
        DataId::new(index).unwrap()
    }

    #[test]
    fn data_id_accepts_the_last_valid_index_and_rejects_the_first_invalid_one() {
        assert_eq!(DataId::new(63).map(DataId::index), Ok(63));
        assert_eq!(DataId::new(64), Err(DataIdOutOfRange));
        assert_eq!(DataId::new(255), Err(DataIdOutOfRange));
    }

    #[test]
    fn every_valid_index_has_its_own_bit() {
        for a in 0..DataSet::CAPACITY {
            let set_a = DataSet::of(a);
            for b in 0..DataSet::CAPACITY {
                assert_eq!(set_a.contains(id(b)), a == b);
                assert_eq!(set_a.intersects(DataSet::of(b)), a == b);
            }
        }
    }

    #[test]
    fn with_adds_one_member_and_is_idempotent() {
        let set = DataSet::EMPTY.with(id(5));
        assert_eq!(set, DataSet::of(5));
        assert!(set.contains(id(5)));
        assert!(!set.contains(id(4)));
        assert!(!set.contains(id(6)));
        assert_eq!(set.with(id(5)), set);
    }

    #[test]
    fn union_holds_both_operands_and_is_commutative() {
        let ab = DataSet::of(1).union(DataSet::of(2));
        assert!(ab.contains(id(1)));
        assert!(ab.contains(id(2)));
        assert!(!ab.contains(id(3)));
        assert_eq!(ab, DataSet::of(2).union(DataSet::of(1)));
        assert_eq!(ab.union(DataSet::EMPTY), ab);
        assert_eq!(ab.union(ab), ab);
    }

    #[test]
    fn intersects_is_symmetric_and_the_empty_set_intersects_nothing() {
        let a = DataSet::of(1).union(DataSet::of(2));
        let b = DataSet::of(2).union(DataSet::of(3));
        let c = DataSet::of(4);
        assert!(a.intersects(b));
        assert!(b.intersects(a));
        assert!(a.intersects(a));
        assert!(!a.intersects(c));
        assert!(!c.intersects(a));
        assert!(!DataSet::EMPTY.intersects(a));
        assert!(!DataSet::EMPTY.intersects(DataSet::EMPTY));
    }

    #[test]
    fn sets_are_buildable_in_const_up_to_the_last_index() {
        const EDGES: DataSet = DataSet::of(0).union(DataSet::of(63));
        assert!(EDGES.contains(id(0)));
        assert!(EDGES.contains(id(63)));
        assert!(!EDGES.contains(id(1)));
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn of_panics_at_runtime_when_the_index_is_out_of_range() {
        let _ = DataSet::of(DataSet::CAPACITY);
    }
}
