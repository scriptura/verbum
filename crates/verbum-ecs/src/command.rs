//! Command Buffer : support transitoire des mutations structurelles différées (ADR-006).
//!
//! Accumulation libre en écriture ; la résolution est unique et appartient au `World`
//! (`World::apply`). Le buffer n'interprète jamais ses commandes (ADR-006, point 5) et ne
//! dépend d'aucun autre module que `entity`.
//!
//! Aucune allocation : la capacité est fixée à la construction (const-générique, forme la
//! plus petite que le test impose ; révisable, cf. `docs/PLAN-ETAPE-0-ECS.md`).

use core::convert::Infallible;

use crate::entity::EntityId;

/// Mutation structurelle différée : création, destruction, ajout de composant. `Spawn` ne
/// porte pas d'identifiant : seul le `World` en crée. `A` est l'ajout typé du `ComponentSet`
/// du monde cible (ensemble fermé de variantes, une par stockage).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Command<A> {
    Spawn,
    Despawn(EntityId),
    Add(EntityId, A),
}

/// Le buffer est plein : erreur explicite, jamais une croissance.
#[derive(Debug, PartialEq, Eq)]
pub struct BufferFull;

/// Buffer de `COMMANDS` commandes au plus. État initial déterministe : vide. `A` est le type
/// d'ajout du `ComponentSet` du `World` qui le résoudra ; par défaut inhabité (aucun ajout).
pub struct CommandBuffer<const COMMANDS: usize, A = Infallible> {
    commands: [Command<A>; COMMANDS],
    len: usize,
}

impl<const COMMANDS: usize, A: Copy> CommandBuffer<COMMANDS, A> {
    pub fn new() -> Self {
        Self {
            commands: [Command::Spawn; COMMANDS], // remplissage ; seul `[0, len)` est lisible
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Accumule la création d'une entité. L'identifiant n'existe qu'après `World::apply`.
    pub fn spawn(&mut self) -> Result<(), BufferFull> {
        self.push(Command::Spawn)
    }

    /// Accumule la destruction d'une entité.
    pub fn despawn(&mut self, entity: EntityId) -> Result<(), BufferFull> {
        self.push(Command::Despawn(entity))
    }

    /// Accumule l'ajout d'un composant à une entité. Le `World` l'ignore si le handle est
    /// périmé ou détruit plus tôt dans le lot ; un doublon ou un dépassement de capacité
    /// rejette le lot entier (`World::apply`).
    pub fn add_component(&mut self, entity: EntityId, add: A) -> Result<(), BufferFull> {
        self.push(Command::Add(entity, add))
    }

    fn push(&mut self, command: Command<A>) -> Result<(), BufferFull> {
        let slot = self.commands.get_mut(self.len).ok_or(BufferFull)?;
        *slot = command;
        self.len += 1;
        Ok(())
    }

    /// Commandes accumulées, dans l'ordre d'émission.
    pub(crate) fn as_slice(&self) -> &[Command<A>] {
        &self.commands[..self.len]
    }

    /// Remet le buffer à son état initial (vide), mémoire conservée. `World::apply` le fait
    /// après un succès ; après un échec, le buffer est conservé et c'est à l'appelant de le
    /// vider.
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl<const COMMANDS: usize, A: Copy> Default for CommandBuffer<COMMANDS, A> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{KinematicsAdd, pos, vel};

    /// Les ajouts s'accumulent dans l'ordre d'émission avec les autres commandes, sans effet
    /// sur le monde ; la capacité borne aussi les ajouts.
    #[test]
    fn accumulates_component_adds_in_emission_order() {
        let mut world = crate::World::<2>::new();
        let a = world.spawn().unwrap();
        let mut buffer = CommandBuffer::<3, KinematicsAdd>::new();

        buffer.add_component(a, pos(1)).unwrap();
        buffer.despawn(a).unwrap();
        buffer.add_component(a, vel(2)).unwrap();

        assert_eq!(
            buffer.as_slice(),
            [
                Command::Add(a, pos(1)),
                Command::Despawn(a),
                Command::Add(a, vel(2))
            ]
        );
        assert_eq!(buffer.add_component(a, pos(3)), Err(BufferFull));
        assert_eq!(buffer.len(), 3);
    }

    /// ADR-006, point 3 : accumulation libre en écriture. Les commandes s'empilent dans
    /// l'ordre d'émission, quel que soit le producteur.
    #[test]
    fn accumulates_commands_in_emission_order() {
        let mut world = crate::World::<2>::new();
        let a = world.spawn().unwrap();
        let mut buffer = CommandBuffer::<4>::new();
        assert!(buffer.is_empty());

        buffer.spawn().unwrap();
        buffer.despawn(a).unwrap();
        buffer.spawn().unwrap();

        assert_eq!(buffer.len(), 3);
        assert_eq!(
            buffer.as_slice(),
            [
                Command::<Infallible>::Spawn,
                Command::Despawn(a),
                Command::Spawn
            ]
        );
    }

    /// Capacité fixée avant simulation (ADR-001, point 5, étendu au buffer) : accumuler au-delà
    /// est une erreur explicite, le contenu déjà accumulé est préservé.
    #[test]
    fn accumulating_beyond_capacity_is_an_explicit_error() {
        let mut buffer = CommandBuffer::<2>::new();
        buffer.spawn().unwrap();
        buffer.spawn().unwrap();

        assert_eq!(buffer.spawn(), Err(BufferFull));
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.as_slice(), [Command::Spawn, Command::Spawn]);
    }

    #[test]
    fn clear_restores_the_empty_initial_state() {
        let mut buffer = CommandBuffer::<2>::new();
        buffer.spawn().unwrap();
        buffer.clear();

        assert!(buffer.is_empty());
        assert_eq!(buffer.as_slice(), []);
        // Pleine capacité de nouveau disponible.
        buffer.spawn().unwrap();
        buffer.spawn().unwrap();
        assert_eq!(buffer.len(), 2);
    }
}
