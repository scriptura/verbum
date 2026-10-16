// crates/ecs/tests/resources.rs

//! Ressources vues depuis l'API publique : cardinalité 1 par `World`, ensemble défini hors du
//! crate (le noyau ne nomme aucun type métier), indépendance du cycle de vie des entités.

use verbum_ecs::{CommandBuffer, ResourceSlot, World};

/// Volontairement ni `Copy` ni `Default` : aucune restriction anticipée sur les types stockés.
#[derive(Debug, PartialEq)]
struct Config {
    gravity: i32,
    name: [u8; 4],
}

#[derive(Debug, PartialEq)]
struct Clock(u64);

/// Ensemble concret : un champ par ressource, accès statique.
#[derive(Default)]
struct Globals {
    config: ResourceSlot<Config>,
    clock: ResourceSlot<Clock>,
}

#[test]
fn a_resource_is_readable_and_writable_with_cardinality_one() {
    let mut world = World::<2, (), Globals>::new();
    assert!(world.resources().config.get().is_none());

    let cfg = Config {
        gravity: -9,
        name: *b"main",
    };
    assert_eq!(world.resources_mut().config.insert(cfg), None);
    assert_eq!(world.resources().config.get().map(|c| c.gravity), Some(-9));

    world.resources_mut().config.get_mut().unwrap().gravity = -10;
    assert_eq!(world.resources().config.get().map(|c| c.gravity), Some(-10));

    // Seconde insertion : remplace, rend l'ancienne valeur, ne duplique jamais.
    let previous = world.resources_mut().config.insert(Config {
        gravity: 1,
        name: *b"next",
    });
    assert_eq!(previous.map(|c| c.gravity), Some(-10));
    assert_eq!(
        world.resources().config.get().map(|c| c.name),
        Some(*b"next")
    );
}

#[test]
fn two_resource_types_coexist_and_survive_the_entity_lifecycle() {
    let mut world = World::<1, (), Globals>::new();
    world.resources_mut().clock.insert(Clock(7));
    world.resources_mut().config.insert(Config {
        gravity: 3,
        name: *b"keep",
    });

    // Cycle de vie des entités (primitives et lot) : les ressources n'y sont pas liées.
    let e = world.spawn().unwrap();
    let mut buffer = CommandBuffer::<2>::new();
    buffer.despawn(e).unwrap();
    buffer.spawn().unwrap();
    world.apply(&mut buffer).unwrap();

    assert_eq!(world.resources().clock.get(), Some(&Clock(7)));
    assert_eq!(world.resources().config.get().map(|c| c.gravity), Some(3));
}
