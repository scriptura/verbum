//! `ComponentSet` : l'ensemble concret des stockages d'un `World`.
//!
//! Le noyau ne nomme aucun type de composant (`01-runtime-ecs.md`, section 3) : l'ensemble est
//! fourni par l'appelant, avec des champs `SparseSet` accessibles statiquement. Ce n'est ni un
//! trait `Component`, ni un registre, ni un mécanisme de plugin : aucun `TypeId`, aucune
//! allocation. Les ajouts sont un **ensemble fermé de variantes typées** (`Add`), une par
//! stockage, définies par l'appelant.
//!
//! Chaque stockage est désigné par un `kind` : entier dense `0..n`, où `n` est la longueur de
//! `Counters`. Le `World` n'a besoin que de ces opérations pour prévalider un lot (sans
//! muter) puis l'appliquer.

use core::convert::Infallible;

use crate::entity::EntityId;
use crate::storage::InsertError;

/// Ensemble de stockages. `Default` : état initial vide, mémoire portée par la valeur.
///
/// # Contrat
/// - `kind(add)` et tout `kind` passé aux autres méthodes sont `< Counters::len()`.
/// - `insert` applique exactement les contrôles de `SparseSet::insert`, dans cet ordre :
///   index hors étendue, déjà présent, dense plein. `covers`, `is_present`, `dense_len` et
///   `dense_capacity` en sont les prédicats sans mutation, par `kind`.
/// - `remove_all` retire l'entité de **tous** les stockages (purge à la destruction).
pub trait ComponentSet: Default {
    /// Ajout typé : une variante par stockage.
    type Add: Copy;
    /// Compteurs de simulation, un par stockage (typiquement `[usize; N]`). Pile seulement.
    type Counters: Copy + Default + AsMut<[usize]>;

    fn kind(add: &Self::Add) -> usize;
    fn dense_len(&self, kind: usize) -> usize;
    fn dense_capacity(&self, kind: usize) -> usize;
    fn covers(&self, kind: usize, entity: EntityId) -> bool;
    fn is_present(&self, kind: usize, entity: EntityId) -> bool;
    fn insert(&mut self, entity: EntityId, add: Self::Add) -> Result<(), InsertError>;
    fn remove_all(&mut self, entity: EntityId);
}

/// Ensemble vide : un `World` sans composant. `Add` est inhabité, donc aucun ajout émissible.
impl ComponentSet for () {
    type Add = Infallible;
    type Counters = [usize; 0];

    fn kind(add: &Infallible) -> usize {
        match *add {}
    }

    fn dense_len(&self, _kind: usize) -> usize {
        0
    }

    fn dense_capacity(&self, _kind: usize) -> usize {
        0
    }

    fn covers(&self, _kind: usize, _entity: EntityId) -> bool {
        false
    }

    fn is_present(&self, _kind: usize, _entity: EntityId) -> bool {
        false
    }

    fn insert(&mut self, _entity: EntityId, add: Infallible) -> Result<(), InsertError> {
        match add {}
    }

    fn remove_all(&mut self, _entity: EntityId) {}
}
