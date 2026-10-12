// crates/verbum-ecs/src/fixtures.rs

//! Jeu de composants de test : deux types distincts dans un même `World`, définis hors du
//! noyau (qui ne nomme aucun type métier). `Position` et `Velocity` n'implémentent
//! volontairement ni `Default` ni aucun trait `Component`.
//!
//! Capacités choisies pour distinguer les erreurs : `positions` couvre 3 index d'entité pour
//! 2 composants (la capacité dense se sature avant l'étendue sparse) ; `velocities` ne couvre
//! que 2 index (l'entité d'index 2 est hors étendue).

use crate::{ComponentSet, EntityId, InsertError, SparseSet, ValuesMut};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Velocity {
    pub dx: i32,
    pub dy: i32,
}

pub(crate) const POSITION: usize = 0;
pub(crate) const VELOCITY: usize = 1;

#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct Kinematics {
    pub positions: SparseSet<Position, 3, 2>,
    pub velocities: SparseSet<Velocity, 2, 2>,
}

/// Ensemble fermé des ajouts possibles : une variante typée par stockage.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum KinematicsAdd {
    Position(Position),
    Velocity(Velocity),
}

pub(crate) const fn pos(x: i32) -> KinematicsAdd {
    KinematicsAdd::Position(Position { x, y: -x })
}

pub(crate) const fn vel(dx: i32) -> KinematicsAdd {
    KinematicsAdd::Velocity(Velocity { dx, dy: -dx })
}

impl ComponentSet for Kinematics {
    type Add = KinematicsAdd;
    type Counters = [usize; 2];
    type Values<'a> = (ValuesMut<'a, Position, 3, 2>, ValuesMut<'a, Velocity, 2, 2>);

    fn kind(add: &Self::Add) -> usize {
        match add {
            KinematicsAdd::Position(_) => POSITION,
            KinematicsAdd::Velocity(_) => VELOCITY,
        }
    }

    fn dense_len(&self, kind: usize) -> usize {
        match kind {
            POSITION => self.positions.len(),
            VELOCITY => self.velocities.len(),
            _ => 0,
        }
    }

    fn dense_capacity(&self, kind: usize) -> usize {
        match kind {
            POSITION => self.positions.dense_capacity(),
            VELOCITY => self.velocities.dense_capacity(),
            _ => 0,
        }
    }

    fn covers(&self, kind: usize, entity: EntityId) -> bool {
        match kind {
            POSITION => self.positions.covers(entity),
            VELOCITY => self.velocities.covers(entity),
            _ => false,
        }
    }

    fn is_present(&self, kind: usize, entity: EntityId) -> bool {
        match kind {
            POSITION => self.positions.contains(entity),
            VELOCITY => self.velocities.contains(entity),
            _ => false,
        }
    }

    fn insert(&mut self, entity: EntityId, add: Self::Add) -> Result<(), InsertError> {
        match add {
            KinematicsAdd::Position(position) => self.positions.insert(entity, position),
            KinematicsAdd::Velocity(velocity) => self.velocities.insert(entity, velocity),
        }
    }

    fn remove_all(&mut self, entity: EntityId) {
        self.positions.remove(entity);
        self.velocities.remove(entity);
    }

    fn values_mut(&mut self) -> Self::Values<'_> {
        (self.positions.values_mut(), self.velocities.values_mut())
    }
}
