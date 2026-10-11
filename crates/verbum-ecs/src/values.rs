//! Accès mutable limité aux valeurs : la séparation entre mutation de valeur et mutation
//! structurelle, tenue par les types et non par convention.
//!
//! `SparseSet::insert` et `remove` sont publics (les implémentations externes de
//! `ComponentSet` en ont besoin) ; le `World` ne peut donc pas exposer `&mut S`. Il expose à la
//! place `ComponentSet::Values`, composé **exclusivement** de [`ValuesMut`] (voir
//! [`ValuesView`], scellé) : une vue exclusive qui lit, parcourt et modifie les valeurs, sans
//! aucune opération structurelle. La résolution des mutations structurelles reste ainsi au
//! `World` (ADR-006).
//!
//! # Garanties vérifiées à la compilation
//!
//! Une vue de valeurs n'a ni `insert` ni `remove` :
//!
//! ```compile_fail,E0599
//! use verbum_ecs::SparseSet;
//! let mut set = SparseSet::<u8, 1, 1>::new();
//! let mut view = set.values_mut();
//! let e = verbum_ecs::World::<1>::new().spawn().unwrap();
//! let _ = view.insert(e, 1);
//! ```
//!
//! ```compile_fail,E0599
//! use verbum_ecs::SparseSet;
//! let mut set = SparseSet::<u8, 1, 1>::new();
//! let mut view = set.values_mut();
//! let e = verbum_ecs::World::<1>::new().spawn().unwrap();
//! let _ = view.remove(e);
//! ```
//!
//! Le trait [`ValuesView`] est scellé : aucun type de l'appelant ne peut en relever, donc aucune
//! implémentation de `ComponentSet` ne peut rendre `&mut Self` (ni un type donnant accès aux
//! opérations structurelles) comme `Values` :
//!
//! ```compile_fail,E0277
//! use verbum_ecs::ValuesView;
//! struct Mine;
//! impl ValuesView for Mine {}
//! ```

use crate::entity::EntityId;
use crate::storage::SparseSet;

mod sealed {
    pub trait Sealed {}
}

/// Ce qu'un `ComponentSet` peut rendre comme accès mutable : des [`ValuesMut`], seuls ou
/// groupés en tuples (qui s'imbriquent). Scellé : non implémentable hors du crate.
pub trait ValuesView: sealed::Sealed {}

/// Vue exclusive sur les **valeurs** d'un `SparseSet` : lecture, parcours, modification des
/// valeurs présentes. Ni `insert`, ni `remove`, ni autre mutation de présence ou d'ordre.
pub struct ValuesMut<'a, T, const SPARSE: usize, const DENSE: usize> {
    set: &'a mut SparseSet<T, SPARSE, DENSE>,
}

impl<'a, T, const SPARSE: usize, const DENSE: usize> ValuesMut<'a, T, SPARSE, DENSE> {
    pub(crate) fn new(set: &'a mut SparseSet<T, SPARSE, DENSE>) -> Self {
        Self { set }
    }
}

impl<T: Copy, const SPARSE: usize, const DENSE: usize> ValuesMut<'_, T, SPARSE, DENSE> {
    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    pub fn contains(&self, entity: EntityId) -> bool {
        self.set.contains(entity)
    }

    pub fn get(&self, entity: EntityId) -> Option<&T> {
        self.set.get(entity)
    }

    /// Modification de la valeur d'une entité présente. Non structurelle.
    pub fn get_mut(&mut self, entity: EntityId) -> Option<&mut T> {
        self.set.get_mut(entity)
    }

    pub fn entities(&self) -> &[EntityId] {
        self.set.entities()
    }

    /// Le stockage en lecture seule (emprunt partagé : aucune mutation possible). Permet à une
    /// Query de lire un stockage tout en écrivant un autre, depuis un même accès au `World`.
    pub fn shared(&self) -> &SparseSet<T, SPARSE, DENSE> {
        self.set
    }
}

impl<T, const SPARSE: usize, const DENSE: usize> sealed::Sealed
    for ValuesMut<'_, T, SPARSE, DENSE>
{
}
impl<T, const SPARSE: usize, const DENSE: usize> ValuesView for ValuesMut<'_, T, SPARSE, DENSE> {}

impl sealed::Sealed for () {}
impl ValuesView for () {}

macro_rules! tuple_views {
    ($($view:ident),+) => {
        impl<$($view: ValuesView),+> sealed::Sealed for ($($view,)+) {}
        impl<$($view: ValuesView),+> ValuesView for ($($view,)+) {}
    };
}
tuple_views!(A);
tuple_views!(A, B);
tuple_views!(A, B, C);
tuple_views!(A, B, C, D);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::GenerationTable;

    /// La vue lit et modifie les valeurs ; présence, ordre dense et longueur sont inchangés.
    #[test]
    fn view_writes_values_without_changing_structure() {
        let mut table = GenerationTable::<3>::new();
        let [a, b, c] = [(); 3].map(|()| table.allocate().unwrap());
        let mut set = SparseSet::<u32, 3, 3>::new();
        for (e, v) in [(a, 1), (b, 2)] {
            set.insert(e, v).unwrap();
        }
        let order = [a, b];

        let mut view = set.values_mut();
        *view.get_mut(a).unwrap() = 10;
        *view.get_mut(b).unwrap() += 5;
        assert_eq!(view.get_mut(c), None, "absente : rien à écrire");
        assert_eq!((view.len(), view.is_empty()), (2, false));
        assert!(view.contains(a) && !view.contains(c));
        assert_eq!(view.entities(), order);
        assert_eq!(view.get(b), Some(&7));
        assert_eq!(view.shared().get(a), Some(&10));

        assert_eq!(set.entities(), order);
        assert_eq!(set.get(a), Some(&10));
        assert_eq!(set.get(b), Some(&7));
    }

    /// Une vue sur un stockage vide est vide.
    #[test]
    fn view_of_an_empty_set_is_empty() {
        let mut set = SparseSet::<u8, 1, 1>::new();
        let view = set.values_mut();
        assert!(view.is_empty());
        assert_eq!(view.entities(), []);
    }
}
