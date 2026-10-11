//! `World` minimal : identité et cycle de vie des entités, rien d'autre.
//!
//! Seul le `World` crée et détruit des entités (`01-runtime-ecs.md`, section 2).
//! À ce stade, il n'a aucun stockage de composant : le premier `SparseSet` y entre
//! en Phase 4b. `spawn` et `despawn` sont ici des primitives directes ; le Command
//! Buffer (Phase 4a) s'y ajoutera en façade (cf. `docs/PLAN-ETAPE-0-ECS.md`).

use crate::entity::{CapacityExceeded, EntityId, GenerationTable, StaleEntity};

/// Monde de `ENTITIES` entités au plus. La capacité est fixée avant la simulation et la
/// mémoire de la table est portée par le `World` lui-même : aucune allocation.
pub struct World<const ENTITIES: usize> {
    entities: GenerationTable<ENTITIES>,
}

impl<const ENTITIES: usize> World<ENTITIES> {
    pub fn new() -> Self {
        Self {
            entities: GenerationTable::new(),
        }
    }

    /// Crée une entité. Erreur explicite si la capacité est atteinte, jamais une croissance.
    pub fn spawn(&mut self) -> Result<EntityId, CapacityExceeded> {
        self.entities.allocate()
    }

    /// Détruit une entité. Refuse un handle périmé sans toucher à l'occupant actuel de l'index.
    pub fn despawn(&mut self, id: EntityId) -> Result<(), StaleEntity> {
        self.entities.release(id)
    }

    /// Vivacité d'un handle : comparaison `(index, génération)`, en O(1).
    pub fn is_alive(&self, id: EntityId) -> bool {
        self.entities.is_alive(id)
    }
}

impl<const ENTITIES: usize> Default for World<ENTITIES> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `01-runtime-ecs.md`, section 2 : le `World` crée les entités ; un handle émis est valide.
    #[test]
    fn spawn_produces_a_valid_handle() {
        let mut world = World::<4>::new();
        let a = world.spawn().unwrap();
        let b = world.spawn().unwrap();

        assert_ne!(a, b);
        assert!(world.is_alive(a) && world.is_alive(b));
    }

    /// Cas canonique de la roadmap, au niveau du `World` : despawn puis réutilisation de
    /// l'index libéré ; l'ancien handle doit rester invalide après réattribution.
    /// Capacité 1 : la réutilisation de l'index est forcée.
    #[test]
    fn despawn_invalidates_the_handle_even_after_index_reuse() {
        let mut world = World::<1>::new();
        let old = world.spawn().unwrap();

        world.despawn(old).unwrap();
        assert!(!world.is_alive(old));

        let new = world.spawn().unwrap();
        assert_ne!(new, old);
        assert!(world.is_alive(new));
        assert!(!world.is_alive(old), "l'ancien handle doit rester invalide");
    }

    #[test]
    fn despawning_a_stale_handle_is_refused_and_spares_the_new_occupant() {
        let mut world = World::<1>::new();
        let old = world.spawn().unwrap();
        world.despawn(old).unwrap();
        let new = world.spawn().unwrap();

        assert_eq!(world.despawn(old), Err(StaleEntity));
        assert!(world.is_alive(new));
    }

    /// ADR-001, point 5 (étendu à l'identité) : dépassement de capacité = erreur explicite.
    #[test]
    fn spawn_beyond_capacity_is_an_explicit_error() {
        let mut world = World::<2>::new();
        let a = world.spawn().unwrap();
        let b = world.spawn().unwrap();

        assert_eq!(world.spawn(), Err(CapacityExceeded));
        assert!(world.is_alive(a) && world.is_alive(b));
    }
}
