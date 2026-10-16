// crates/ecs/tests/command_buffer.rs

//! Command Buffer vu depuis l'API publique : la création n'existe qu'après `World::apply`,
//! la destruction est différée, le buffer est vide et réutilisable après application.

use verbum_ecs::{ApplyError, BufferFull, CapacityExceeded, CommandBuffer, World};

#[test]
fn deferred_structural_mutations_are_resolved_once_by_the_world() {
    let mut world = World::<2>::new();
    let e = world.spawn().unwrap();
    let mut buffer = CommandBuffer::<2>::new();

    buffer.despawn(e).unwrap();
    buffer.spawn().unwrap();
    assert_eq!(buffer.spawn(), Err(BufferFull)); // borné
    assert!(world.is_alive(e)); // différé

    assert_eq!(world.apply(&mut buffer), Ok(()));
    assert!(!world.is_alive(e));
    assert!(buffer.is_empty());

    // Réutilisable ; le monde plein signale explicitement le dépassement.
    buffer.spawn().unwrap();
    buffer.spawn().unwrap();
    assert_eq!(
        world.apply(&mut buffer),
        Err(ApplyError::Spawn(CapacityExceeded))
    );
    // Échec : le buffer est conservé intact, l'appelant le vide explicitement.
    assert_eq!(buffer.len(), 2);
    buffer.clear();
    assert!(buffer.is_empty());
}
