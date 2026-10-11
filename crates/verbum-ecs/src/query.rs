//! Query : vue passive sur des stockages (ADR-002).
//!
//! Une Query n'est ni possédée par le `World` ni couplée à lui (ADR-001, ADR-002, point 1) : elle
//! emprunte des stockages (`SparseSet`) que l'appelant lui donne, et ne dépend que de `entity` et
//! `storage`. Elle n'a aucune identité ni aucun état durable : deux références et un curseur, qui
//! disparaissent avec elle ; les emprunts rendent les stockages à l'appelant dès qu'elle est
//! lâchée. Elle ne produit, ne transforme, ne calcule et n'alloue rien (point 7) : elle
//! expose des références vers les données existantes.
//!
//! **Spécialisation statique** (point 2) : générique sur les types et les capacités, aucun
//! dispatch dynamique. **Mode d'accès dans le type** (point 3) : `ReadWriteQuery<'a, R, W, ..>`
//! lit `R` (`&SparseSet<R>`) et écrit `W` (`ValuesMut<W>`, vue exclusive sur les valeurs), et chaque élément rend `&R` et
//! `&mut W`. **Source d'itération** (point 4) : le stockage lu, premier argument, pilote la boucle
//! dans son ordre dense ; l'autre est résolu par consultation de son index de présence. Le
//! critère de choix du pilote (ici : la position de l'argument) est une décision
//! d'implémentation. **`EntityId`** (point 6) fait partie de l'élément, sans traitement spécial.
//!
//! Comme tout stockage, la Query ne revalide pas la génération (ADR-001, point 2) : la vivacité
//! relève du `World`. Les filtres `With`/`Without` (point 5) n'apparaissent qu'avec un test qui
//! les exige.

use crate::entity::EntityId;
use crate::storage::SparseSet;
use crate::values::ValuesMut;

/// Vue sur deux stockages : lecture de `R`, lecture/écriture de `W`, sur les entités qui
/// possèdent les deux. Itération « prêteuse » (`advance`) : chaque élément emprunte la Query, ce
/// qui permet de rendre `&mut W` sans `unsafe` ni allocation.
pub struct ReadWriteQuery<
    'a,
    R,
    W,
    const READ_SPARSE: usize,
    const READ_DENSE: usize,
    const WRITE_SPARSE: usize,
    const WRITE_DENSE: usize,
> {
    read: &'a SparseSet<R, READ_SPARSE, READ_DENSE>,
    write: ValuesMut<'a, W, WRITE_SPARSE, WRITE_DENSE>,
    cursor: usize,
}

impl<
    'a,
    R: Copy,
    W: Copy,
    const READ_SPARSE: usize,
    const READ_DENSE: usize,
    const WRITE_SPARSE: usize,
    const WRITE_DENSE: usize,
> ReadWriteQuery<'a, R, W, READ_SPARSE, READ_DENSE, WRITE_SPARSE, WRITE_DENSE>
{
    pub fn new(
        read: &'a SparseSet<R, READ_SPARSE, READ_DENSE>,
        write: ValuesMut<'a, W, WRITE_SPARSE, WRITE_DENSE>,
    ) -> Self {
        Self {
            read,
            write,
            cursor: 0,
        }
    }

    /// Élément suivant : une entité présente dans les deux stockages, avec sa donnée lue et sa
    /// donnée modifiable. `None` une fois la source épuisée (et ensuite).
    pub fn advance(&mut self) -> Option<(EntityId, &R, &mut W)> {
        let read = self.read;
        let entities = read.entities();
        while let Some(&entity) = entities.get(self.cursor) {
            self.cursor += 1;
            // L'appartenance se décide par consultation de l'index de présence de l'autre
            // stockage (emprunt partagé) ; l'emprunt exclusif n'est pris que pour rendre
            // l'élément. L'entité pilote est présente dans le stockage lu, par construction.
            if self.write.contains(entity) {
                let value = read.get(entity)?;
                let target = self.write.get_mut(entity)?;
                return Some((entity, value, target));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::GenerationTable;

    fn ids<const N: usize>(table: &mut GenerationTable<N>) -> [EntityId; N] {
        [(); N].map(|()| table.allocate().unwrap())
    }

    /// Collecte les entités visitées, sans écrire (la Query ne transforme rien d'elle-même).
    fn visited<
        R: Copy,
        W: Copy,
        const RS: usize,
        const RD: usize,
        const WS: usize,
        const WD: usize,
    >(
        read: &SparseSet<R, RS, RD>,
        write: &mut SparseSet<W, WS, WD>,
    ) -> ([Option<EntityId>; 8], usize) {
        let mut out = [None; 8];
        let mut n = 0;
        let mut query = ReadWriteQuery::new(read, write.values_mut());
        while let Some((entity, ..)) = query.advance() {
            out[n] = Some(entity);
            n += 1;
        }
        (out, n)
    }

    /// Intersection exacte, dans l'ordre dense du stockage lu (pilote), pour tous les
    /// sous-ensembles de présence de 4 entités dans chacun des deux stockages.
    #[test]
    fn yields_exactly_the_intersection_for_all_presence_patterns() {
        let mut table = GenerationTable::<4>::new();
        let entities = ids(&mut table);
        for read_mask in 0u8..16 {
            for write_mask in 0u8..16 {
                let mut read = SparseSet::<u8, 4, 4>::new();
                let mut write = SparseSet::<u8, 4, 4>::new();
                for (i, &e) in entities.iter().enumerate() {
                    if read_mask >> i & 1 == 1 {
                        read.insert(e, i as u8).unwrap();
                    }
                    if write_mask >> i & 1 == 1 {
                        write.insert(e, 100 + i as u8).unwrap();
                    }
                }
                let mut want = [None; 4];
                let mut wanted = 0;
                for &e in read.entities() {
                    if write.contains(e) {
                        want[wanted] = Some(e);
                        wanted += 1;
                    }
                }

                let (seen, n) = visited(&read, &mut write);
                assert_eq!(n, wanted, "{read_mask:04b}/{write_mask:04b}");
                assert_eq!(seen[..4], want, "{read_mask:04b}/{write_mask:04b}");
            }
        }
    }

    /// Lecture seule des données : parcourir sans écrire laisse les deux stockages identiques.
    #[test]
    fn iterating_without_writing_changes_nothing() {
        let mut table = GenerationTable::<3>::new();
        let [a, b, c] = ids(&mut table);
        let mut read = SparseSet::<u8, 3, 3>::new();
        let mut write = SparseSet::<u8, 3, 3>::new();
        for (e, v) in [(a, 1), (b, 2), (c, 3)] {
            read.insert(e, v).unwrap();
            write.insert(e, v + 10).unwrap();
        }
        let (read_before, write_before) = (read.clone(), write.clone());

        let (_, n) = visited(&read, &mut write);

        assert_eq!(n, 3);
        assert_eq!(read, read_before);
        assert_eq!(write, write_before);
    }

    /// Les écritures atteignent la donnée du stockage écrit pour l'entité visée, et elle seule.
    #[test]
    fn writes_reach_the_matching_entity_only() {
        let mut table = GenerationTable::<3>::new();
        let [a, b, c] = ids(&mut table);
        let mut read = SparseSet::<u8, 3, 3>::new();
        let mut write = SparseSet::<u8, 3, 3>::new();
        read.insert(a, 1).unwrap();
        read.insert(c, 3).unwrap();
        for (e, v) in [(a, 10), (b, 20), (c, 30)] {
            write.insert(e, v).unwrap();
        }

        let mut query = ReadWriteQuery::new(&read, write.values_mut());
        while let Some((_, &r, w)) = query.advance() {
            *w += r;
        }

        assert_eq!(write.get(a), Some(&11));
        assert_eq!(write.get(b), Some(&20));
        assert_eq!(write.get(c), Some(&33));
    }

    /// Une source vide ou une intersection vide : `None` immédiat, et la Query reste épuisée.
    #[test]
    fn empty_source_or_empty_intersection_yields_nothing() {
        let mut table = GenerationTable::<2>::new();
        let [a, b] = ids(&mut table);
        let mut read = SparseSet::<u8, 2, 2>::new();
        let mut write = SparseSet::<u8, 2, 2>::new();
        assert_eq!(visited(&read, &mut write).1, 0);

        read.insert(a, 1).unwrap();
        write.insert(b, 2).unwrap();
        let mut query = ReadWriteQuery::new(&read, write.values_mut());
        assert!(query.advance().is_none());
        assert!(query.advance().is_none());
    }

    /// Ni `Default`, ni autre exigence que `Copy` sur les types lus et écrits.
    #[test]
    fn accepts_copy_types_without_default() {
        #[derive(Clone, Copy, Debug, PartialEq)]
        struct Opaque(u8);
        let mut table = GenerationTable::<1>::new();
        let [a] = ids(&mut table);
        let mut read = SparseSet::<Opaque, 1, 1>::new();
        let mut write = SparseSet::<Opaque, 1, 1>::new();
        read.insert(a, Opaque(1)).unwrap();
        write.insert(a, Opaque(2)).unwrap();

        let mut query = ReadWriteQuery::new(&read, write.values_mut());
        let (_, r, w) = query.advance().unwrap();
        assert_eq!((*r, *w), (Opaque(1), Opaque(2)));
    }
}
