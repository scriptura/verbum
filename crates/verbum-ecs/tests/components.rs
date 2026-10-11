//! Composants vus depuis l'API publique, avec un `ComponentSet` défini hors du crate : le noyau
//! ne nomme aucun type métier (`01-runtime-ecs.md`, section 3). Deux types distincts dans un
//! même `World`, ajouts différés, purge à la destruction, atomicité sur les stockages.

use verbum_ecs::{
    ApplyError, CapacityExceeded, CommandBuffer, ComponentSet, EntityId, InsertError, SparseSet,
    World,
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Health(u32);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Mana(u32);

/// Ensemble concret : deux stockages, champs accessibles statiquement. Capacités : `health`
/// couvre 4 index et 2 composants ; `mana` couvre 2 index (l'index 2 est hors étendue).
#[derive(Default)]
struct Stats {
    health: SparseSet<Health, 4, 2>,
    mana: SparseSet<Mana, 2, 2>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum StatsAdd {
    Health(Health),
    Mana(Mana),
}

impl ComponentSet for Stats {
    type Add = StatsAdd;
    type Counters = [usize; 2];

    fn kind(add: &StatsAdd) -> usize {
        match add {
            StatsAdd::Health(_) => 0,
            StatsAdd::Mana(_) => 1,
        }
    }

    fn dense_len(&self, kind: usize) -> usize {
        match kind {
            0 => self.health.len(),
            _ => self.mana.len(),
        }
    }

    fn dense_capacity(&self, kind: usize) -> usize {
        match kind {
            0 => self.health.dense_capacity(),
            _ => self.mana.dense_capacity(),
        }
    }

    fn covers(&self, kind: usize, entity: EntityId) -> bool {
        match kind {
            0 => self.health.covers(entity),
            _ => self.mana.covers(entity),
        }
    }

    fn is_present(&self, kind: usize, entity: EntityId) -> bool {
        match kind {
            0 => self.health.contains(entity),
            _ => self.mana.contains(entity),
        }
    }

    fn insert(&mut self, entity: EntityId, add: StatsAdd) -> Result<(), InsertError> {
        match add {
            StatsAdd::Health(value) => self.health.insert(entity, value),
            StatsAdd::Mana(value) => self.mana.insert(entity, value),
        }
    }

    fn remove_all(&mut self, entity: EntityId) {
        self.health.remove(entity);
        self.mana.remove(entity);
    }
}

type StatsWorld = World<3, Stats>;
type StatsBuffer = CommandBuffer<4, StatsAdd>;

#[test]
fn two_component_types_coexist_in_one_world() {
    let mut world = StatsWorld::new();
    let e = world.spawn().unwrap();
    let mut buffer = StatsBuffer::new();
    buffer
        .add_component(e, StatsAdd::Health(Health(10)))
        .unwrap();
    buffer.add_component(e, StatsAdd::Mana(Mana(5))).unwrap();
    assert!(!world.components().health.contains(e)); // différé

    assert_eq!(world.apply(&mut buffer), Ok(()));
    assert_eq!(world.components().health.get(e), Some(&Health(10)));
    assert_eq!(world.components().mana.get(e), Some(&Mana(5)));
    assert!(buffer.is_empty());
}

#[test]
fn despawn_purges_components_before_the_slot_is_reused() {
    let mut world = World::<1, Stats>::new();
    let old = world.spawn().unwrap();
    let mut buffer = StatsBuffer::new();
    buffer
        .add_component(old, StatsAdd::Health(Health(1)))
        .unwrap();
    buffer.add_component(old, StatsAdd::Mana(Mana(2))).unwrap();
    world.apply(&mut buffer).unwrap();

    buffer.despawn(old).unwrap();
    buffer.spawn().unwrap();
    world.apply(&mut buffer).unwrap();

    // Le nouvel occupant du même index n'hérite d'aucun composant.
    assert_eq!(world.components().health.len(), 0);
    assert_eq!(world.components().mana.len(), 0);
    assert!(!world.is_alive(old));
}

#[test]
fn rejected_batch_leaves_world_and_buffer_untouched() {
    let mut world = StatsWorld::new();
    let a = world.spawn().unwrap();
    let b = world.spawn().unwrap();
    let c = world.spawn().unwrap();
    let mut buffer = StatsBuffer::new();
    buffer
        .add_component(a, StatsAdd::Health(Health(1)))
        .unwrap(); // applicable
    buffer
        .add_component(b, StatsAdd::Health(Health(2)))
        .unwrap(); // applicable
    buffer
        .add_component(c, StatsAdd::Health(Health(3)))
        .unwrap(); // dense plein
    assert_eq!(
        world.apply(&mut buffer),
        Err(ApplyError::Add(InsertError::DenseFull))
    );

    assert_eq!(world.components().health.len(), 0);
    assert_eq!(buffer.len(), 3);
    buffer.clear();

    // Étendue sparse dépassée : erreur, alors qu'un handle périmé du même index serait ignoré.
    buffer.add_component(c, StatsAdd::Mana(Mana(4))).unwrap();
    assert_eq!(
        world.apply(&mut buffer),
        Err(ApplyError::Add(InsertError::IndexOutOfRange))
    );
    buffer.clear();

    // Création impossible : monde plein.
    buffer.spawn().unwrap();
    assert_eq!(
        world.apply(&mut buffer),
        Err(ApplyError::Spawn(CapacityExceeded))
    );
}
