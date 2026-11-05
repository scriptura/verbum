// crates/extensions/script/src/table.rs

//! Table de scripts indexée directement par le slot.
//!
//! La table porte `CAP` cases ; le script du slot `i` occupe la case `i`, sans
//! compactage ni table de correspondance. Une case vide désigne un slot sans
//! script (slot vacant ou slot occupé par un asset d'un autre format : cette
//! couche ne connaît aucun autre format).

use verbum_assets::{Script, SlotIndex};

/// Refus d'un dépôt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableError {
    /// Le slot est hors de la capacité de la table.
    OutOfRange,
    /// Le slot porte déjà un script.
    Occupied,
}

/// Table de `CAP` scripts empruntés, indexée par slot.
pub struct ScriptTable<'a, const CAP: usize> {
    slots: [Option<Script<'a>>; CAP],
}

impl<'a, const CAP: usize> ScriptTable<'a, CAP> {
    /// Les slots doivent tenir dans un `u16` : `ScriptState::script` est un
    /// `u16`, donc au plus `65 536` cases (indices `0..=65 535`).
    const CAPACITY_FITS_INDEX: () = assert!(CAP <= 65_536);

    /// Table vide.
    // `clippy::new_without_default` : voir `Emissions::new` (même motif, aucun
    // `Default` public prévu).
    #[allow(clippy::new_without_default)]
    #[must_use]
    pub const fn new() -> Self {
        let () = Self::CAPACITY_FITS_INDEX;
        Self { slots: [None; CAP] }
    }

    /// Dépose `script` au slot `slot`.
    ///
    /// # Errors
    ///
    /// `TableError::OutOfRange` si `slot` est hors de `CAP` ;
    /// `TableError::Occupied` si le slot porte déjà un script. La table reste
    /// alors inchangée.
    pub fn deposit(&mut self, slot: SlotIndex, script: Script<'a>) -> Result<(), TableError> {
        let Ok(index) = usize::try_from(slot.get()) else {
            return Err(TableError::OutOfRange);
        };
        match self.slots.get_mut(index) {
            None => Err(TableError::OutOfRange),
            Some(Some(_)) => Err(TableError::Occupied),
            Some(entry @ None) => {
                *entry = Some(script);
                Ok(())
            }
        }
    }

    /// Script du slot `index`, ou `None` (slot vacant ou hors de `CAP`).
    #[must_use]
    pub fn get(&self, index: u16) -> Option<&Script<'a>> {
        self.slots.get(usize::from(index))?.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `VSCR`, version 1, charge utile 10, `n = 1` : `Halt`.
    const HALT_ONLY: [u8; 20] = [
        0x56, 0x53, 0x43, 0x52, 0x01, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x00, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    /// `VSCR`, version 1, charge utile 18, `n = 2` : `Set r0, 1` puis `Halt`.
    const SET_HALT: [u8; 28] = [
        0x56, 0x53, 0x43, 0x52, 0x01, 0x00, 0x12, 0x00, 0x00, 0x00, 0x02, 0x00, 0x03, 0, 0, 0, 1,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    // Évaluation de la garde statique à la borne exacte : acceptée.
    const _: () = ScriptTable::<'static, 65_536>::CAPACITY_FITS_INDEX;

    // TB-1 : table indexée par slot.
    #[test]
    fn tb1_table_indexed_by_slot() {
        let script_a = Script::read(&HALT_ONLY).expect("script A");
        let script_b = Script::read(&SET_HALT).expect("script B");
        assert_ne!(script_a, script_b);

        let mut table = ScriptTable::<6>::new();
        assert_eq!(table.deposit(SlotIndex::new(4), script_a), Ok(()));
        assert_eq!(table.deposit(SlotIndex::new(1), script_b), Ok(()));

        assert_eq!(table.get(4), Some(&script_a));
        assert_eq!(table.get(1), Some(&script_b));
        for vacant in [0u16, 2, 3, 5] {
            assert_eq!(table.get(vacant), None, "slot {vacant} vacant");
        }

        assert_eq!(
            table.deposit(SlotIndex::new(6), script_a),
            Err(TableError::OutOfRange)
        );
        assert_eq!(
            table.deposit(SlotIndex::new(u32::MAX), script_a),
            Err(TableError::OutOfRange)
        );
        assert_eq!(
            table.deposit(SlotIndex::new(4), script_b),
            Err(TableError::Occupied)
        );
        // Dépôts refusés : table inchangée.
        assert_eq!(table.get(4), Some(&script_a));
        assert_eq!(table.get(1), Some(&script_b));
        assert_eq!(table.get(6), None);
        assert_eq!(table.get(u16::MAX), None);
    }
}
