// crates/ecs/src/entity.rs

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
    const fn new(index: u32, generation: u32) -> Self {
        Self(((generation as u64) << 32) | index as u64)
    }

    /// Valeur de remplissage des tampons denses hors de leur préfixe occupé.
    ///
    /// Ne peut jamais être émise par une table : son index (`u32::MAX`) dépasse toute
    /// capacité (bornée sous `END_OF_FREE_LIST`) et sa génération (`u32::MAX`) n'est
    /// jamais attribuée (marqueur de slot retiré).
    pub(crate) const VACANT: Self = Self::new(u32::MAX, u32::MAX);

    /// Index d'entité. Seule composante accessible aux stockages : ils n'ont pas à
    /// lire la génération (ADR-001, point 2).
    pub(crate) fn index(self) -> u32 {
        // Troncature voulue : les 32 bits bas.
        self.0 as u32
    }

    fn generation(self) -> u32 {
        (self.0 >> 32) as u32
    }
}

/// Le dépassement de capacité de la table : erreur explicite, jamais une croissance.
#[derive(Debug, PartialEq, Eq)]
pub struct CapacityExceeded;

/// Le handle ne désigne plus d'entité vivante.
#[derive(Debug, PartialEq, Eq)]
pub struct StaleEntity;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GenerationTable<const N: usize> {
    slots: [Slot; N],
    free_head: u32,
    /// Nombre de slots dans la liste libre (les slots retirés n'y figurent pas). Permet de
    /// prévalider un lot de créations en O(1), sans parcourir la liste.
    free: usize,
}

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
            free: N,
        }
    }

    pub(crate) fn allocate(&mut self) -> Result<EntityId, CapacityExceeded> {
        let index = self.free_head;
        // `END_OF_FREE_LIST` dépasse toute capacité : `get_mut` échoue alors.
        let slot = self.slots.get_mut(index as usize).ok_or(CapacityExceeded)?;
        self.free_head = slot.next_free;
        self.free -= 1;
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
            self.free += 1;
        }
        Ok(())
    }

    /// Test seulement : amène un slot libre à une génération donnée (ex. une avant l'épuisement).
    #[cfg(test)]
    pub(crate) fn set_generation_for_test(&mut self, index: usize, generation: u32) {
        self.slots[index].generation = generation;
    }

    /// Nombre de slots que `allocate` peut encore attribuer.
    pub(crate) fn free_slots(&self) -> usize {
        self.free
    }

    /// Prédit, sans muter, si `release(id)` rendrait un slot à la liste libre : le handle doit
    /// être vivant, et son slot ne doit pas être retiré par épuisement de génération.
    pub(crate) fn frees_slot_on_release(&self, id: EntityId) -> bool {
        self.slots.get(id.index() as usize).is_some_and(|slot| {
            slot.generation == id.generation()
                && slot
                    .generation
                    .checked_add(1)
                    .is_some_and(|next| next != u32::MAX)
        })
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

    /// Longueur réelle de la liste libre, par parcours.
    fn walk_free_list<const N: usize>(table: &GenerationTable<N>) -> usize {
        let (mut count, mut index) = (0, table.free_head);
        while (index as usize) < N {
            count += 1;
            index = table.slots[index as usize].next_free;
        }
        count
    }

    /// Le compteur de slots libres suit `allocate`/`release` et coïncide avec la liste libre.
    #[test]
    fn free_slots_tracks_the_free_list() {
        let mut table = GenerationTable::<3>::new();
        assert_eq!((table.free_slots(), walk_free_list(&table)), (3, 3));
        let a = table.allocate().unwrap();
        let _b = table.allocate().unwrap();
        assert_eq!((table.free_slots(), walk_free_list(&table)), (1, 1));
        table.release(a).unwrap();
        assert_eq!((table.free_slots(), walk_free_list(&table)), (2, 2));
        assert_eq!(table.release(a), Err(StaleEntity));
        assert_eq!((table.free_slots(), walk_free_list(&table)), (2, 2));
    }

    /// Un slot retiré (génération épuisée) ne revient pas dans le décompte des slots libres.
    #[test]
    fn retired_slot_is_not_counted_as_free() {
        let mut table = GenerationTable::<1>::new();
        table.slots[0].generation = u32::MAX - 1;
        let last = table.allocate().unwrap();
        assert!(!table.frees_slot_on_release(last));

        table.release(last).unwrap();
        assert_eq!((table.free_slots(), walk_free_list(&table)), (0, 0));
    }

    /// `frees_slot_on_release` prédit sans muter ce que `release` fera : un handle vivant
    /// libère un slot ; un handle périmé ou hors de portée n'en libère aucun.
    #[test]
    fn frees_slot_on_release_predicts_release_without_mutating() {
        let mut table = GenerationTable::<2>::new();
        let a = table.allocate().unwrap();
        let before = table.clone();

        assert!(table.frees_slot_on_release(a));
        assert!(!table.frees_slot_on_release(EntityId::VACANT));
        assert_eq!(table, before);

        table.release(a).unwrap();
        assert!(!table.frees_slot_on_release(a), "périmé : rien à libérer");
    }

    /// `VACANT` ne peut correspondre à aucun handle émis par une table : son index
    /// (`u32::MAX`) dépasse toute capacité, et sa génération (`u32::MAX`) n'est jamais attribuée.
    #[test]
    fn vacant_is_never_issued_nor_alive() {
        let mut table = GenerationTable::<3>::new();
        for _ in 0..3 {
            let id = table.allocate().unwrap();
            assert_ne!(id, EntityId::VACANT);
            assert!(!table.is_alive(EntityId::VACANT));
        }
        assert_eq!(EntityId::VACANT.index(), u32::MAX);
        assert_eq!(EntityId::VACANT.generation(), u32::MAX);
    }
}
