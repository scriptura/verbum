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
//!
//! # Séparation valeurs / structure, vérifiée par les types
//!
//! `Values` ne peut être qu'un ensemble de `ValuesMut` (le trait `ValuesView` est scellé).
//! Une implémentation conforme compile :
//!
//! ```
//! use core::convert::Infallible;
//! use verbum_ecs::{ComponentSet, EntityId, InsertError, SparseSet, ValuesMut};
//!
//! #[derive(Default)]
//! struct Set { numbers: SparseSet<u8, 1, 1> }
//!
//! impl ComponentSet for Set {
//!     type Add = Infallible;
//!     type Counters = [usize; 1];
//!     type Values<'a> = (ValuesMut<'a, u8, 1, 1>,);
//!     fn kind(add: &Infallible) -> usize { match *add {} }
//!     fn dense_len(&self, _: usize) -> usize { 0 }
//!     fn dense_capacity(&self, _: usize) -> usize { 0 }
//!     fn covers(&self, _: usize, _: EntityId) -> bool { false }
//!     fn is_present(&self, _: usize, _: EntityId) -> bool { false }
//!     fn insert(&mut self, _: EntityId, add: Infallible) -> Result<(), InsertError> { match add {} }
//!     fn remove_all(&mut self, _: EntityId) {}
//!     fn values_mut(&mut self) -> Self::Values<'_> { (self.numbers.values_mut(),) }
//! }
//! ```
//!
//! Rendre `&mut Self` (donc `insert` et `remove`) ne compile pas :
//!
//! ```compile_fail,E0277
//! use core::convert::Infallible;
//! use verbum_ecs::{ComponentSet, EntityId, InsertError, SparseSet, ValuesMut};
//!
//! #[derive(Default)]
//! struct Set { numbers: SparseSet<u8, 1, 1> }
//!
//! impl ComponentSet for Set {
//!     type Add = Infallible;
//!     type Counters = [usize; 1];
//!     type Values<'a> = &'a mut Self;
//!     fn kind(add: &Infallible) -> usize { match *add {} }
//!     fn dense_len(&self, _: usize) -> usize { 0 }
//!     fn dense_capacity(&self, _: usize) -> usize { 0 }
//!     fn covers(&self, _: usize, _: EntityId) -> bool { false }
//!     fn is_present(&self, _: usize, _: EntityId) -> bool { false }
//!     fn insert(&mut self, _: EntityId, add: Infallible) -> Result<(), InsertError> { match add {} }
//!     fn remove_all(&mut self, _: EntityId) {}
//!     fn values_mut(&mut self) -> Self::Values<'_> { self }
//! }
//! ```

use core::convert::Infallible;

use crate::entity::EntityId;
use crate::storage::InsertError;
use crate::values::ValuesView;

/// Ensemble de stockages. `Default` : état initial vide, mémoire portée par la valeur.
///
/// # Contrat
/// - `kind(add)` et tout `kind` passé aux autres méthodes sont `< Counters::len()`.
/// - `insert` applique exactement les contrôles de `SparseSet::insert`, dans cet ordre :
///   index hors étendue, déjà présent, dense plein. `covers`, `is_present`, `dense_len` et
///   `dense_capacity` en sont les prédicats sans mutation, par `kind`.
/// - `Values` est l'accès mutable aux **valeurs** que le `World` expose (`component_values_mut`) :
///   des `ValuesMut` (seuls ou en tuples, une entrée par stockage). `ValuesView` est scellé : un
///   `Values` donnant accès aux opérations structurelles (par exemple `&mut Self`) ne compile
///   pas, donc la séparation tient par les types. Le `World` n'expose jamais `&mut S`.
/// - `remove_all` retire l'entité de **tous** les stockages (purge à la destruction).
pub trait ComponentSet: Default {
    /// Ajout typé : une variante par stockage.
    type Add: Copy;
    /// Compteurs de simulation, un par stockage (typiquement `[usize; N]`). Pile seulement.
    type Counters: Copy + Default + AsMut<[usize]>;
    /// Vues de valeurs mutables, un champ statique par stockage (tuple de `ValuesMut`).
    type Values<'a>: ValuesView
    where
        Self: 'a;

    fn kind(add: &Self::Add) -> usize;
    fn dense_len(&self, kind: usize) -> usize;
    fn dense_capacity(&self, kind: usize) -> usize;
    fn covers(&self, kind: usize, entity: EntityId) -> bool;
    fn is_present(&self, kind: usize, entity: EntityId) -> bool;
    fn insert(&mut self, entity: EntityId, add: Self::Add) -> Result<(), InsertError>;
    fn remove_all(&mut self, entity: EntityId);
    fn values_mut(&mut self) -> Self::Values<'_>;
}

/// Ensemble vide : un `World` sans composant. `Add` est inhabité, donc aucun ajout émissible.
impl ComponentSet for () {
    type Add = Infallible;
    type Counters = [usize; 0];
    type Values<'a> = ();

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

    fn values_mut(&mut self) -> Self::Values<'_> {}
}
