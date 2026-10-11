//! Ressource : donnée de cardinalité 1 par `World` (ADR-005).
//!
//! Une Ressource est identifiée par une clé unique plutôt que par `EntityId` ; ici, la clé est
//! la **position statique** : un champ `ResourceSlot<T>` d'un ensemble défini par l'appelant
//! (`World<_, _, R>`). Aucun `TypeId`, aucun registre, aucune allocation, aucun dispatch
//! dynamique, et aucun lien avec le cycle de vie des entités.
//!
//! `ResourceSlot<T>` porte la cardinalité par construction : au plus une valeur, une seconde
//! insertion la remplace. Aucune contrainte sur `T` (ni `Copy`, ni `Default`) : `Option<T>`
//! suffit, sans `unsafe`.

/// Emplacement d'une Ressource de type `T` : vide, ou porteur d'une unique valeur.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceSlot<T> {
    value: Option<T>,
}

impl<T> ResourceSlot<T> {
    /// Emplacement vide : état initial déterministe.
    pub const fn new() -> Self {
        Self { value: None }
    }

    /// Insère la Ressource. Si une valeur existait, elle est **remplacée** et rendue :
    /// le cardinal reste 1, jamais de duplication.
    pub fn insert(&mut self, value: T) -> Option<T> {
        self.value.replace(value)
    }

    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.value.as_mut()
    }
}

// Manuel : `#[derive(Default)]` exigerait `T: Default`, restriction sans nécessité.
impl<T> Default for ResourceSlot<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ni `Copy`, ni `Default`, ni `Clone`.
    #[derive(Debug, PartialEq)]
    struct Opaque(u8);

    #[test]
    fn starts_empty_without_requiring_default() {
        let slot = ResourceSlot::<Opaque>::default();
        assert_eq!(slot.get(), None);
        assert_eq!(ResourceSlot::<Opaque>::new(), slot);
    }

    #[test]
    fn insert_then_read_and_modify() {
        let mut slot = ResourceSlot::new();
        assert_eq!(slot.insert(Opaque(1)), None);
        assert_eq!(slot.get(), Some(&Opaque(1)));

        slot.get_mut().unwrap().0 = 2;
        assert_eq!(slot.get(), Some(&Opaque(2)));
    }

    /// Cardinalité 1 : une seconde insertion remplace et rend l'ancienne valeur.
    #[test]
    fn second_insert_replaces_and_never_duplicates() {
        let mut slot = ResourceSlot::new();
        slot.insert(Opaque(1));

        assert_eq!(slot.insert(Opaque(2)), Some(Opaque(1)));
        assert_eq!(slot.insert(Opaque(3)), Some(Opaque(2)));
        assert_eq!(slot.get(), Some(&Opaque(3)));
    }

    #[test]
    fn get_mut_on_an_empty_slot_is_none() {
        let mut slot = ResourceSlot::<Opaque>::new();
        assert_eq!(slot.get_mut(), None);
    }
}
