//! Identité des entités : `EntityId` et table de génération.
//!
//! Invariant : un `EntityId` est un identifiant opaque dont la validité repose sur
//! la comparaison `(index, génération)` (`01-runtime-ecs.md`, section 1).
//!
//! Aucune allocation : la mémoire de la table est fournie à la construction
//! (ici, un tableau const-générique ; forme la plus petite qui n'expose aucun
//! type de slot — révisable en Phase 3, cf. `docs/PLAN-ETAPE-0-ECS.md`).

/// Fin de la liste libre. Vaut `u32::MAX`, hors de portée de tout index valide
/// (la capacité est bornée statiquement en dessous).
const END_OF_FREE_LIST: u32 = u32::MAX;

/// Identifiant d'entité opaque : génération (32 bits hauts), index (32 bits bas).
///
/// Le découpage interne est une décision d'implémentation. Aucun constructeur
/// public : seul le `World` crée des entités (`01-runtime-ecs.md`, section 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityId(u64);

impl EntityId {
    fn new(index: u32, generation: u32) -> Self {
        Self((u64::from(generation) << 32) | u64::from(index))
    }

    fn index(self) -> u32 {
        // Troncature voulue : les 32 bits bas.
        self.0 as u32
    }

    fn generation(self) -> u32 {
        (self.0 >> 32) as u32
    }
}

/// Le dépassement de capacité de la table : erreur explicite, jamais une croissance.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CapacityExceeded;

/// Le handle ne désigne plus d'entité vivante.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct StaleEntity;

#[derive(Clone, Copy)]
struct Slot {
    /// Génération courante du slot. `u32::MAX` n'est jamais attribuée : elle
    /// marque un slot retiré.
    generation: u32,
    /// Slot libre suivant (liste libre intrusive, LIFO) ; sans objet si occupé.
    next_free: u32,
}

/// Table de génération sur `N` slots : `is_alive(id) == (slots[id.index].generation == id.generation)`.
///
/// Interne au crate jusqu'à la Phase 3 : le `World` la possédera.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "consommée par le World à partir de la Phase 3")
)]
pub(crate) struct GenerationTable<const N: usize> {
    slots: [Slot; N],
    free_head: u32,
}

#[cfg_attr(not(test), expect(dead_code, reason = "cf. GenerationTable"))]
impl<const N: usize> GenerationTable<N> {
    /// Les index doivent tenir sous la sentinelle de fin de liste.
    const CAPACITY_FITS_INDEX: () = assert!(N < END_OF_FREE_LIST as usize);

    pub(crate) fn new() -> Self {
        let () = Self::CAPACITY_FITS_INDEX;

        let mut slots = [Slot {
            generation: 0,
            next_free: END_OF_FREE_LIST,
        }; N];
        for (index, slot) in slots.iter_mut().enumerate() {
            let next = index + 1;
            if next < N {
                slot.next_free = next as u32; // next < N < u32::MAX
            }
        }
        Self {
            slots,
            free_head: if N > 0 { 0 } else { END_OF_FREE_LIST },
        }
    }

    pub(crate) fn allocate(&mut self) -> Result<EntityId, CapacityExceeded> {
        let index = self.free_head;
        // `END_OF_FREE_LIST` dépasse toute capacité : `get_mut` échoue alors.
        let slot = self.slots.get_mut(index as usize).ok_or(CapacityExceeded)?;
        self.free_head = slot.next_free;
        Ok(EntityId::new(index, slot.generation))
    }

    pub(crate) fn release(&mut self, id: EntityId) -> Result<(), StaleEntity> {
        if !self.is_alive(id) {
            return Err(StaleEntity);
        }
        let index = id.index();
        let slot = &mut self.slots[index as usize];
        // Un handle vivant porte une génération < `u32::MAX` : pas de débordement.
        slot.generation += 1;
        // Génération épuisée : le slot est retiré, jamais réattribué.
        if slot.generation != u32::MAX {
            slot.next_free = self.free_head;
            self.free_head = index;
        }
        Ok(())
    }

    pub(crate) fn is_alive(&self, id: EntityId) -> bool {
        self.slots
            .get(id.index() as usize)
            .is_some_and(|slot| slot.generation == id.generation())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `01-runtime-ecs.md`, section 1 : après despawn d'un index puis réattribution
    /// de cet index à une nouvelle entité, l'ancien `EntityId` doit être invalide.
    /// Capacité 1 : la réutilisation de l'index est forcée, quelle que soit la politique.
    #[test]
    fn stale_id_stays_invalid_after_index_reuse() {
        let mut table = GenerationTable::<1>::new();
        let old = table.allocate().unwrap();
        assert!(table.is_alive(old));

        table.release(old).unwrap();
        assert!(!table.is_alive(old));

        let new = table.allocate().unwrap();
        assert_eq!(
            new.index(),
            old.index(),
            "l'index libéré doit être réattribué"
        );
        assert_ne!(new, old);
        assert!(table.is_alive(new));
        assert!(!table.is_alive(old), "l'ancien handle doit rester invalide");
    }

    /// Libérer un handle périmé est refusé et ne libère pas le nouvel occupant.
    #[test]
    fn releasing_a_stale_id_does_not_free_the_new_occupant() {
        let mut table = GenerationTable::<1>::new();
        let old = table.allocate().unwrap();
        table.release(old).unwrap();
        let new = table.allocate().unwrap();

        assert_eq!(table.release(old), Err(StaleEntity));
        assert!(table.is_alive(new));
    }

    /// Un dépassement de capacité est une erreur explicite, jamais une croissance.
    #[test]
    fn exceeding_capacity_is_an_error() {
        let mut table = GenerationTable::<2>::new();
        let a = table.allocate().unwrap();
        let b = table.allocate().unwrap();
        assert_ne!(a.index(), b.index());

        assert_eq!(table.allocate(), Err(CapacityExceeded));
        assert!(table.is_alive(a) && table.is_alive(b));
    }

    /// Une table de capacité nulle est valide : toute allocation échoue.
    #[test]
    fn zero_capacity_table_rejects_allocation() {
        let mut table = GenerationTable::<0>::new();
        assert_eq!(table.allocate(), Err(CapacityExceeded));
    }

    /// Une génération épuisée retire le slot au lieu de reboucler : sinon un
    /// ancien handle redeviendrait valide.
    #[test]
    fn exhausted_generation_retires_the_slot() {
        let mut table = GenerationTable::<1>::new();
        table.slots[0].generation = u32::MAX - 1;
        let last = table.allocate().unwrap();

        table.release(last).unwrap();
        assert!(!table.is_alive(last));
        assert_eq!(table.allocate(), Err(CapacityExceeded));
        assert!(!table.is_alive(last));
    }
}
