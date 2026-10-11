//! Query vue depuis l'API publique : une vue sur des stockages, sans couplage avec le `World`
//! (ADR-002). Les identifiants viennent du `World` (seule source d'`EntityId`) ; les stockages
//! sont ceux de l'appelant.

use verbum_ecs::{EntityId, ReadWriteQuery, SparseSet, World};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position(i32);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity(i32);

#[test]
fn iterates_exactly_the_entities_holding_both_components() {
    let mut world = World::<4>::new();
    let [a, b, c, d] = [(); 4].map(|()| world.spawn().unwrap());

    let mut positions = SparseSet::<Position, 4, 4>::new();
    let mut velocities = SparseSet::<Velocity, 4, 4>::new();
    for (e, x) in [(a, 1), (b, 2), (d, 4)] {
        positions.insert(e, Position(x)).unwrap();
    }
    for (e, v) in [(a, 10), (c, 30), (d, 40)] {
        velocities.insert(e, Velocity(v)).unwrap();
    }

    // Lecture de `Position`, lecture/écriture de `Velocity` : le mode est dans le type.
    let mut query = ReadWriteQuery::new(&positions, velocities.values_mut());
    let mut seen: [Option<EntityId>; 4] = [None; 4];
    let mut count = 0;
    while let Some((entity, position, velocity)) = query.advance() {
        velocity.0 += position.0;
        seen[count] = Some(entity);
        count += 1;
    }

    assert_eq!(seen, [Some(a), Some(d), None, None]); // ordre du stockage pilote
    assert_eq!(velocities.get(a), Some(&Velocity(11)));
    assert_eq!(velocities.get(d), Some(&Velocity(44)));
    assert_eq!(
        velocities.get(c),
        Some(&Velocity(30)),
        "non visé : inchangé"
    );
    assert_eq!(positions.get(b), Some(&Position(2)));
}

#[test]
fn a_query_leaves_no_state_behind() {
    let mut world = World::<2>::new();
    let [a, b] = [(); 2].map(|()| world.spawn().unwrap());
    let mut positions = SparseSet::<Position, 2, 2>::new();
    let mut velocities = SparseSet::<Velocity, 2, 2>::new();
    positions.insert(a, Position(1)).unwrap();
    velocities.insert(a, Velocity(1)).unwrap();

    let mut first = ReadWriteQuery::new(&positions, velocities.values_mut());
    assert!(first.advance().is_some());
    assert!(first.advance().is_none());
    assert!(first.advance().is_none(), "épuisée : reste vide");

    // Les stockages sont de nouveau pleinement utilisables ; une nouvelle vue reflète l'état
    // courant, sans cache.
    positions.insert(b, Position(2)).unwrap();
    velocities.insert(b, Velocity(2)).unwrap();
    let mut second = ReadWriteQuery::new(&positions, velocities.values_mut());
    assert_eq!(second.advance().map(|(e, ..)| e), Some(a));
    assert_eq!(second.advance().map(|(e, ..)| e), Some(b));
    assert!(second.advance().is_none());
}
