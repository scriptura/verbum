//! Identité des entités, vue depuis l'API publique seule : aucun constructeur
//! d'`EntityId` n'est accessible hors du `World`.

use verbum_ecs::{CapacityExceeded, StaleEntity, World};

#[test]
fn stale_handle_stays_invalid_after_index_reuse() {
    let mut world = World::<1>::new();
    let old = world.spawn().unwrap();
    world.despawn(old).unwrap();
    let new = world.spawn().unwrap();

    assert!(world.is_alive(new));
    assert!(!world.is_alive(old));
    assert_eq!(world.despawn(old), Err(StaleEntity));
    assert_eq!(world.spawn(), Err(CapacityExceeded));
}
