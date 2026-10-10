//! Stockage des composants : Sparse Set (ADR-001).
//!
//! Le Sparse Set est un index de présence, jamais un mécanisme de validation
//! d'identité : il ne lit jamais la génération d'un `EntityId` (ADR-001, point 2).
//!
//! Seul `get` contient du code `unsafe` (lecture d'une entrée dense initialisée) ;
//! le fondement est l'invariant d'initialisation de `SparseSet`.
//!
//! Aucune allocation. La mémoire est fournie à la construction sous forme de deux
//! capacités const-génériques distinctes : `SPARSE` (étendue des index d'entité
//! adressables) et `DENSE` (nombre maximal de composants). Forme la plus petite que
//! les tests de la Phase 2 imposent ; révisable en Phase 4b, cf.
//! `docs/PLAN-ETAPE-0-ECS.md`.

use core::mem::MaybeUninit;

use crate::entity::EntityId;

/// Sentinelle « absent » de l'index sparse. Hors de portée de toute position dense
/// (`DENSE` est borné statiquement en dessous).
const ABSENT: u32 = u32::MAX;

/// Échec d'insertion. Dépassements de capacité et doublons : erreurs explicites,
/// jamais une croissance ni un comportement silencieux (ADR-001, point 5).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InsertError {
    /// L'entité possède déjà un composant dans ce stockage (comportement local,
    /// non spécifié par le corpus).
    AlreadyPresent,
    /// Le tableau dense est plein.
    DenseFull,
    /// L'index d'entité dépasse l'étendue du tableau sparse.
    IndexOutOfRange,
}

/// Stockage d'un type de composant `T`. Ignore tout autre stockage (ADR-001, point 3).
///
/// Trois tableaux : sparse (index d'entité → position dense ou `ABSENT`), dense
/// d'`EntityId` parallèle, dense de données contigu. Le préfixe `[0, len)` des
/// tableaux denses est plein, sans trou (swap-remove, ADR-001, point 4).
///
/// `T: Copy` seulement : aucune exigence de `Default` (aucune restriction anticipée
/// sur les types stockés). `Copy` dispense de tout `Drop` : retirer ou écraser une
/// entrée n'exige aucune libération.
///
/// # Invariant d'initialisation (seul fondement du seul bloc `unsafe`)
/// `dense_data[i]` est initialisé **si et seulement si** `i < len`. Chaque valeur
/// non `ABSENT` de `sparse` est `< len`. Les cases `[len, DENSE)` de `dense_data`
/// ne sont jamais lues comme des `T` : elles ne sont atteignables ni par `get`
/// (qui passe par `sparse`), ni par aucune autre méthode.
pub(crate) struct SparseSet<T, const SPARSE: usize, const DENSE: usize> {
    sparse: [u32; SPARSE],
    dense_entities: [EntityId; DENSE],
    dense_data: [MaybeUninit<T>; DENSE],
    len: usize,
}

impl<T: Copy, const SPARSE: usize, const DENSE: usize> SparseSet<T, SPARSE, DENSE> {
    /// Les positions denses doivent tenir sous la sentinelle `ABSENT`, et l'index
    /// `u32::MAX` (celui de `EntityId::VACANT`) doit rester hors de l'étendue sparse.
    const CAPACITIES_FIT: () = assert!(DENSE < ABSENT as usize && SPARSE < ABSENT as usize);

    pub(crate) fn new() -> Self {
        let () = Self::CAPACITIES_FIT;
        Self {
            sparse: [ABSENT; SPARSE],
            dense_entities: [EntityId::VACANT; DENSE],
            dense_data: [MaybeUninit::uninit(); DENSE],
            len: 0,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    /// Position dense de l'entité, si présente. Compare l'index seul, jamais la génération.
    fn position(&self, entity: EntityId) -> Option<usize> {
        let position = *self.sparse.get(entity.index() as usize)?;
        (position != ABSENT).then_some(position as usize)
    }

    pub(crate) fn contains(&self, entity: EntityId) -> bool {
        self.position(entity).is_some()
    }

    #[allow(
        unsafe_code,
        reason = "seul bloc unsafe du stockage ; cf. invariant d'initialisation"
    )]
    pub(crate) fn get(&self, entity: EntityId) -> Option<&T> {
        let position = self.position(entity)?;
        debug_assert!(position < self.len);
        // SAFETY: `position` provient de `sparse` et est donc `< len` (invariant de
        // la structure) ; `dense_data[position]` est alors initialisé.
        Some(unsafe { self.dense_data[position].assume_init_ref() })
    }

    pub(crate) fn insert(&mut self, entity: EntityId, value: T) -> Result<(), InsertError> {
        let slot = self
            .sparse
            .get_mut(entity.index() as usize)
            .ok_or(InsertError::IndexOutOfRange)?;
        if *slot != ABSENT {
            return Err(InsertError::AlreadyPresent);
        }
        if self.len == DENSE {
            return Err(InsertError::DenseFull);
        }
        let position = self.len;
        *slot = position as u32; // position < DENSE < ABSENT
        self.dense_entities[position] = entity;
        self.dense_data[position].write(value);
        self.len += 1;
        Ok(())
    }

    /// Retire le composant de l'entité par swap-remove. `false` si elle n'en avait pas.
    pub(crate) fn remove(&mut self, entity: EntityId) -> bool {
        let Some(position) = self.position(entity) else {
            return false;
        };
        let last = self.len - 1;
        let moved = self.dense_entities[last];
        self.dense_entities[position] = moved;
        self.dense_data[position] = self.dense_data[last];
        // `moved` peut être `entity` (retrait du dernier) : l'ordre des deux écritures
        // sparse garantit que l'entrée de l'entité retirée finit `ABSENT`.
        self.sparse[moved.index() as usize] = position as u32;
        self.sparse[entity.index() as usize] = ABSENT;
        self.len = last;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::GenerationTable;

    /// Fabrique `n` identifiants distincts via la table de génération (seule source d'`EntityId`).
    fn ids<const N: usize>(table: &mut GenerationTable<N>, n: usize) -> [EntityId; N] {
        let mut out = [EntityId::VACANT; N];
        for slot in out.iter_mut().take(n) {
            *slot = table.allocate().unwrap();
        }
        out
    }

    #[test]
    fn insert_then_get_and_contains() {
        let mut table = GenerationTable::<4>::new();
        let [a, b, ..] = ids(&mut table, 2);
        let mut set = SparseSet::<u32, 4, 4>::new();

        assert!(!set.contains(a));
        assert_eq!(set.insert(a, 10), Ok(()));
        assert_eq!(set.insert(b, 20), Ok(()));

        assert!(set.contains(a) && set.contains(b));
        assert_eq!(set.get(a), Some(&10));
        assert_eq!(set.get(b), Some(&20));
        assert_eq!(set.len(), 2);
    }

    /// ADR-001, point 4 : swap-remove. L'élément de fin comble la position libérée ;
    /// sa correspondance sparse/dense est mise à jour ; l'ancien emplacement est absent.
    #[test]
    fn swap_remove_moves_last_element_and_updates_sparse() {
        let mut table = GenerationTable::<4>::new();
        let [a, b, c, _] = ids(&mut table, 3);
        let mut set = SparseSet::<u32, 4, 4>::new();
        set.insert(a, 10).unwrap();
        set.insert(b, 20).unwrap();
        set.insert(c, 30).unwrap();

        assert!(set.remove(a));

        // Contiguïté : le préfixe dense [0, len) est plein, sans trou.
        assert_eq!(set.len(), 2);
        assert_eq!(set.dense_entities[..2], [c, b]);
        // Les données suivent leurs entités : lecture via sparse -> dense (seule voie de lecture).
        assert_eq!(set.get(set.dense_entities[0]), Some(&30));
        assert_eq!(set.get(set.dense_entities[1]), Some(&20));
        // L'élément déplacé conserve une correspondance sparse -> dense correcte.
        assert_eq!(set.sparse[c.index() as usize], 0);
        assert_eq!(set.get(c), Some(&30));
        assert_eq!(set.get(b), Some(&20));
        // L'ancien emplacement logique de l'élément supprimé est absent.
        assert!(!set.contains(a));
        assert_eq!(set.get(a), None);
        assert_eq!(set.sparse[a.index() as usize], ABSENT);
    }

    #[test]
    fn removing_the_last_element_moves_nothing() {
        let mut table = GenerationTable::<4>::new();
        let [a, b, ..] = ids(&mut table, 2);
        let mut set = SparseSet::<u32, 4, 4>::new();
        set.insert(a, 10).unwrap();
        set.insert(b, 20).unwrap();

        assert!(set.remove(b));

        assert_eq!(set.len(), 1);
        assert_eq!(set.get(a), Some(&10));
        assert!(!set.contains(b));
    }

    #[test]
    fn remove_of_absent_entry_is_a_no_op() {
        let mut table = GenerationTable::<4>::new();
        let [a, b, ..] = ids(&mut table, 2);
        let mut set = SparseSet::<u32, 4, 4>::new();
        set.insert(a, 10).unwrap();

        assert!(!set.remove(b));
        assert_eq!(set.len(), 1);
        assert_eq!(set.get(a), Some(&10));
    }

    /// Un index remplissable à nouveau après suppression : le stockage reste cohérent.
    #[test]
    fn slot_is_reusable_after_remove() {
        let mut table = GenerationTable::<4>::new();
        let [a, b, ..] = ids(&mut table, 2);
        let mut set = SparseSet::<u32, 4, 4>::new();
        set.insert(a, 10).unwrap();
        set.remove(a);
        set.insert(b, 20).unwrap();
        set.insert(a, 11).unwrap();

        assert_eq!(set.get(a), Some(&11));
        assert_eq!(set.get(b), Some(&20));
        assert_eq!(set.len(), 2);
    }

    /// ADR-001, point 2 : le Sparse Set est un index de présence, jamais une validation
    /// d'identité. Un handle périmé qui partage l'index d'une entité vivante atteint
    /// la même entrée : la vivacité est vérifiée en amont, par le `World`.
    #[test]
    fn storage_does_not_revalidate_generation() {
        let mut table = GenerationTable::<1>::new();
        let old = table.allocate().unwrap();
        table.release(old).unwrap();
        let new = table.allocate().unwrap();
        assert_ne!(old, new);
        assert_eq!(old.index(), new.index());

        let mut set = SparseSet::<u32, 1, 1>::new();
        set.insert(new, 7).unwrap();

        assert_eq!(set.get(old), Some(&7));
        assert!(set.contains(old));
    }

    /// ADR-001, point 3 : deux stockages sont indépendants, y compris en cas d'erreur
    /// dans l'un des deux.
    #[test]
    fn storages_are_isolated() {
        let mut table = GenerationTable::<4>::new();
        let [a, b, c, _] = ids(&mut table, 3);
        let mut positions = SparseSet::<[i32; 2], 4, 1>::new();
        let mut velocities = SparseSet::<[i32; 2], 4, 4>::new();

        positions.insert(a, [1, 1]).unwrap();
        velocities.insert(b, [2, 2]).unwrap();
        // Erreur dans `positions` (capacité dense atteinte)…
        assert_eq!(positions.insert(c, [3, 3]), Err(InsertError::DenseFull));

        // …sans effet sur `velocities`, et aucune présence croisée.
        assert_eq!(velocities.get(b), Some(&[2, 2]));
        assert_eq!(velocities.len(), 1);
        assert!(!velocities.contains(a));
        assert!(!positions.contains(b));
        assert_eq!(positions.get(a), Some(&[1, 1]));
    }

    /// ADR-001, point 5 : dépassement de capacité = erreur explicite, jamais croissance.
    #[test]
    fn capacity_overflow_is_an_explicit_error() {
        let mut table = GenerationTable::<4>::new();
        let [a, b, c, _] = ids(&mut table, 3);
        let mut set = SparseSet::<u32, 4, 2>::new();
        set.insert(a, 1).unwrap();
        set.insert(b, 2).unwrap();

        assert_eq!(set.insert(c, 3), Err(InsertError::DenseFull));
        assert_eq!(set.len(), 2);
        assert!(!set.contains(c));
    }

    #[test]
    fn entity_index_beyond_sparse_range_is_an_explicit_error() {
        let mut table = GenerationTable::<4>::new();
        let [_, _, _, d] = ids(&mut table, 4);
        let mut set = SparseSet::<u32, 3, 3>::new(); // index 3 hors de portée

        assert_eq!(set.insert(d, 1), Err(InsertError::IndexOutOfRange));
        assert!(!set.contains(d));
        assert_eq!(set.get(d), None);
        assert!(!set.remove(d));
        assert_eq!(set.len(), 0);
    }

    /// Comportement local (non spécifié par le corpus) : l'insertion en doublon est
    /// refusée et laisse le stockage intact.
    #[test]
    fn duplicate_insert_is_rejected_and_leaves_the_set_unchanged() {
        let mut table = GenerationTable::<4>::new();
        let [a, ..] = ids(&mut table, 1);
        let mut set = SparseSet::<u32, 4, 4>::new();
        set.insert(a, 10).unwrap();

        assert_eq!(set.insert(a, 99), Err(InsertError::AlreadyPresent));
        assert_eq!(set.get(a), Some(&10));
        assert_eq!(set.len(), 1);
    }

    /// Le stockage n'impose pas `Default` : un type `Copy` quelconque est accepté
    /// (`01-runtime-ecs.md` ne l'exige pas ; le plan interdit toute restriction anticipée).
    #[test]
    fn accepts_a_copy_type_without_default() {
        #[derive(Clone, Copy, Debug, PartialEq)]
        struct NoDefault(u8); // volontairement sans `Default`

        let mut table = GenerationTable::<4>::new();
        let [a, b, ..] = ids(&mut table, 2);
        let mut set = SparseSet::<NoDefault, 4, 4>::new();

        set.insert(a, NoDefault(1)).unwrap();
        set.insert(b, NoDefault(2)).unwrap();
        assert_eq!(set.get(a), Some(&NoDefault(1)));

        assert!(set.remove(a));
        assert_eq!(set.get(a), None);
        assert_eq!(set.get(b), Some(&NoDefault(2)));
        assert_eq!(set.len(), 1);
    }

    /// Les cases hors du préfixe dense ne sont jamais exposées comme présentes :
    /// toute position sparse est `ABSENT` ou `< len`, et désigne l'entité qui la porte.
    #[test]
    fn unused_cells_are_never_exposed_as_present() {
        let mut table = GenerationTable::<6>::new();
        let [a, b, c, d, e, f] = ids(&mut table, 6);
        let mut set = SparseSet::<u32, 6, 4>::new();
        for (id, v) in [(a, 1), (b, 2), (c, 3), (d, 4)] {
            set.insert(id, v).unwrap();
        }
        set.remove(b);
        set.remove(d);
        set.insert(e, 5).unwrap();
        set.remove(a);

        let mut present = 0;
        for id in [a, b, c, d, e, f] {
            match set.position(id) {
                Some(position) => {
                    present += 1;
                    assert!(position < set.len());
                    assert_eq!(set.dense_entities[position], id);
                }
                None => assert!(!set.contains(id) && set.get(id).is_none()),
            }
        }
        assert_eq!(present, set.len());
        // La valeur de remplissage n'est jamais une entrée présente.
        assert!(!set.contains(EntityId::VACANT));
        assert_eq!(set.get(EntityId::VACANT), None);
        assert!(!set.remove(EntityId::VACANT));
    }
}
