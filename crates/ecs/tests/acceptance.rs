// crates/ecs/tests/acceptance.rs

//! Test d'acceptation de l'étape 0 (`ROADMAP.md`) :
//! spawn → ajout composant → query → command buffer → resolve → query.
//!
//! API publique seule, `ComponentSet` défini hors du crate. Démontre que les composants sont
//! modifiés par Query (valeurs seulement), que toute mutation structurelle passe par le Command
//! Buffer résolu une fois par le `World`, que la seconde Query observe l'état résolu, et qu'un
//! handle périmé échoue toujours via `(index, génération)`.

use core::convert::Infallible;

use verbum_ecs::{
    ApplyError, CommandBuffer, ComponentSet, EntityId, InsertError, ReadWriteQuery, SparseSet,
    ValuesMut, World,
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position(i32);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity(i32);

#[derive(Default)]
struct Motion {
    positions: SparseSet<Position, 4, 4>,
    velocities: SparseSet<Velocity, 4, 4>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MotionAdd {
    Position(Position),
    Velocity(Velocity),
}

impl ComponentSet for Motion {
    type Add = MotionAdd;
    type Remove = Infallible;
    type Counters = [usize; 2];
    type Values<'a> = (ValuesMut<'a, Position, 4, 4>, ValuesMut<'a, Velocity, 4, 4>);

    fn kind(add: &MotionAdd) -> usize {
        match add {
            MotionAdd::Position(_) => 0,
            MotionAdd::Velocity(_) => 1,
        }
    }

    fn remove_kind(remove: &Infallible) -> usize {
        match *remove {}
    }

    fn dense_len(&self, kind: usize) -> usize {
        match kind {
            0 => self.positions.len(),
            _ => self.velocities.len(),
        }
    }

    fn dense_capacity(&self, kind: usize) -> usize {
        match kind {
            0 => self.positions.dense_capacity(),
            _ => self.velocities.dense_capacity(),
        }
    }

    fn covers(&self, kind: usize, entity: EntityId) -> bool {
        match kind {
            0 => self.positions.covers(entity),
            _ => self.velocities.covers(entity),
        }
    }

    fn is_present(&self, kind: usize, entity: EntityId) -> bool {
        match kind {
            0 => self.positions.contains(entity),
            _ => self.velocities.contains(entity),
        }
    }

    fn insert(&mut self, entity: EntityId, add: MotionAdd) -> Result<(), InsertError> {
        match add {
            MotionAdd::Position(p) => self.positions.insert(entity, p),
            MotionAdd::Velocity(v) => self.velocities.insert(entity, v),
        }
    }

    fn remove(&mut self, _entity: EntityId, remove: Infallible) -> bool {
        match remove {}
    }

    fn remove_all(&mut self, entity: EntityId) {
        self.positions.remove(entity);
        self.velocities.remove(entity);
    }

    fn values_mut(&mut self) -> Self::Values<'_> {
        (self.positions.values_mut(), self.velocities.values_mut())
    }
}

type MotionWorld = World<4, Motion>;
type MotionBuffer = CommandBuffer<8, MotionAdd>;

/// Passe de calcul : `velocity += position` sur les entités qui ont les deux. Lecture de
/// `Position`, lecture/écriture de `Velocity` ; la vue vient du `World`, sans `world.query()`.
/// Rend les entités visitées, dans l'ordre du stockage pilote.
fn integrate(world: &mut MotionWorld) -> ([Option<EntityId>; 4], usize) {
    let (positions, velocities) = world.component_values_mut();
    let mut query = ReadWriteQuery::new(positions.shared(), velocities);
    let mut visited = [None; 4];
    let mut count = 0;
    while let Some((entity, position, velocity)) = query.advance() {
        velocity.0 += position.0;
        visited[count] = Some(entity);
        count += 1;
    }
    (visited, count)
}

#[test]
fn stage_zero_acceptance() {
    let mut world = MotionWorld::new();

    // spawn
    let e1 = world.spawn().unwrap();
    let e2 = world.spawn().unwrap();
    let e3 = world.spawn().unwrap();

    // ajout de composants (différé : un seul `apply`)
    let mut buffer = MotionBuffer::new();
    for (entity, x) in [(e1, 1), (e2, 2), (e3, 3)] {
        buffer
            .add_component(entity, MotionAdd::Position(Position(x)))
            .unwrap();
    }
    buffer
        .add_component(e1, MotionAdd::Velocity(Velocity(10)))
        .unwrap();
    buffer
        .add_component(e3, MotionAdd::Velocity(Velocity(30)))
        .unwrap();
    assert_eq!(world.components().positions.len(), 0, "rien avant apply");
    assert_eq!(world.apply(&mut buffer), Ok(()));
    assert!(buffer.is_empty());

    // query : exactement les entités qui ont les deux composants, valeurs modifiées par la Query
    let (visited, count) = integrate(&mut world);
    assert_eq!((visited, count), ([Some(e1), Some(e3), None, None], 2));
    assert_eq!(world.components().velocities.get(e1), Some(&Velocity(11)));
    assert_eq!(world.components().velocities.get(e3), Some(&Velocity(33)));
    assert_eq!(world.components().positions.len(), 3, "structure inchangée");

    // command buffer : plusieurs producteurs accumulent, rien n'est visible avant la résolution
    buffer.despawn(e1).unwrap(); // producteur A
    buffer
        .add_component(e2, MotionAdd::Velocity(Velocity(5)))
        .unwrap(); // producteur B
    buffer.spawn().unwrap(); // producteur C
    buffer.despawn(e1).unwrap(); // producteur D : doublon, sans effet
    assert!(world.is_alive(e1));
    assert_eq!(world.components().velocities.len(), 2);

    // resolve : résolution unique par le World
    assert_eq!(world.apply(&mut buffer), Ok(()));
    assert!(buffer.is_empty());
    assert!(!world.is_alive(e1), "détruit");
    assert_eq!(world.components().positions.get(e1), None, "purgé");
    assert_eq!(world.components().velocities.get(e1), None, "purgé");

    // la seconde query observe l'état résolu : e1 disparu, e2 désormais éligible ;
    // le stockage pilote a subi un swap-remove ([e1, e2, e3] -> [e3, e2])
    let (visited, count) = integrate(&mut world);
    assert_eq!((visited, count), ([Some(e3), Some(e2), None, None], 2));
    assert_eq!(world.components().velocities.get(e3), Some(&Velocity(36)));
    assert_eq!(world.components().velocities.get(e2), Some(&Velocity(7)));

    // un handle périmé échoue toujours via (index, génération), après réutilisation du slot
    let reused = world.spawn().unwrap();
    assert!(world.is_alive(reused));
    assert!(
        !world.is_alive(e1),
        "ancien handle invalide après réattribution"
    );
    let mut buffer = MotionBuffer::new();
    buffer
        .add_component(e1, MotionAdd::Position(Position(99)))
        .unwrap(); // handle périmé : ignoré
    assert_eq!(world.apply(&mut buffer), Ok(()));
    assert_eq!(world.components().positions.get(reused), None);
}

#[test]
fn a_rejected_batch_is_invisible_to_queries() {
    let mut world = MotionWorld::new();
    let e = world.spawn().unwrap();
    let mut buffer = MotionBuffer::new();
    buffer
        .add_component(e, MotionAdd::Position(Position(1)))
        .unwrap();
    buffer
        .add_component(e, MotionAdd::Velocity(Velocity(1)))
        .unwrap();
    world.apply(&mut buffer).unwrap();
    assert_eq!(integrate(&mut world).1, 1);

    // Lot rejeté : la seconde création déborde (monde de 4, 1 vivante, 4 créations demandées).
    for _ in 0..4 {
        buffer.spawn().unwrap();
    }
    buffer.despawn(e).unwrap();
    assert_eq!(
        world.apply(&mut buffer),
        Err(ApplyError::Spawn(verbum_ecs::CapacityExceeded)),
    );
    buffer.clear();

    // La Query voit le monde d'avant le lot : e est vivante, ses composants présents.
    assert!(world.is_alive(e));
    let (visited, count) = integrate(&mut world);
    assert_eq!((visited, count), ([Some(e), None, None, None], 1));
    // Deux passes d'intégration au total (1 + 1 + 1) : la valeur reflète les seules écritures
    // des Queries, jamais le lot rejeté.
    assert_eq!(world.components().velocities.get(e), Some(&Velocity(3)));
}
