//! `World` minimal : identité et cycle de vie des entités, rien d'autre.
//!
//! Seul le `World` crée et détruit des entités (`01-runtime-ecs.md`, section 2).
//! À ce stade, il n'a aucun stockage de composant : le premier `SparseSet` y entre
//! en Phase 4b. `spawn` et `despawn` sont ici des primitives directes ; le Command
//! Buffer (Phase 4a) s'y ajoutera en façade (cf. `docs/PLAN-ETAPE-0-ECS.md`).

use crate::command::{Command, CommandBuffer};
use crate::entity::{CapacityExceeded, EntityId, GenerationTable, StaleEntity};

/// Monde de `ENTITIES` entités au plus. La capacité est fixée avant la simulation et la
/// mémoire de la table est portée par le `World` lui-même : aucune allocation.
#[cfg_attr(test, derive(Clone, Debug, PartialEq))]
pub struct World<const ENTITIES: usize> {
    entities: GenerationTable<ENTITIES>,
}

impl<const ENTITIES: usize> World<ENTITIES> {
    pub fn new() -> Self {
        Self {
            entities: GenerationTable::new(),
        }
    }

    /// Crée une entité. Erreur explicite si la capacité est atteinte, jamais une croissance.
    pub fn spawn(&mut self) -> Result<EntityId, CapacityExceeded> {
        self.entities.allocate()
    }

    /// Détruit une entité. Refuse un handle périmé sans toucher à l'occupant actuel de l'index.
    pub fn despawn(&mut self, id: EntityId) -> Result<(), StaleEntity> {
        self.entities.release(id)
    }

    /// Vivacité d'un handle : comparaison `(index, génération)`, en O(1).
    pub fn is_alive(&self, id: EntityId) -> bool {
        self.entities.is_alive(id)
    }

    /// Résout le buffer : seule autorité habilitée à appliquer les mutations structurelles
    /// (ADR-006, point 3).
    ///
    /// **Atomique** : le lot est d'abord prévalidé, dans son ordre d'émission, sans mutation ni
    /// allocation ; il n'est appliqué que s'il est intégralement applicable. Si `apply`
    /// renvoie une erreur, le `World` est resté intégralement inchangé (entités vivantes,
    /// générations, liste libre) et le buffer est **conservé intact** : transaction non
    /// consommée, qu'un appelant qui ignore l'erreur retrouvera à l'identique à la phase
    /// suivante, jamais en silence. L'appelant le vide alors avec [`CommandBuffer::clear`].
    /// En cas de succès, le lot est appliqué en une seule passe, dans l'ordre d'accumulation,
    /// via les primitives `spawn` et `despawn`, puis le buffer est vidé. Le buffer n'est pas
    /// possédé par le `World` : sa mémoire reste à l'appelant.
    ///
    /// Un dépassement de capacité est un bug de configuration (ADR-001, point 5). Choix local
    /// (non spécifié par le corpus) : une destruction dont le handle est périmé est sans effet,
    /// plusieurs producteurs pouvant viser la même entité.
    pub fn apply<const COMMANDS: usize>(
        &mut self,
        commands: &mut CommandBuffer<COMMANDS>,
    ) -> Result<(), CapacityExceeded> {
        self.validate(commands.as_slice())?;

        for command in commands.as_slice() {
            match *command {
                Command::Spawn => {
                    let spawned = self.spawn();
                    debug_assert!(spawned.is_ok(), "lot prévalidé");
                }
                Command::Despawn(id) => {
                    // Handle périmé : sans effet.
                    let _ = self.despawn(id);
                }
            }
        }
        commands.clear();
        Ok(())
    }

    /// Prévalidation d'un lot : sans mutation ni allocation. Seule une création peut échouer
    /// (capacité) ; une destruction peut libérer un slot pour les créations suivantes, une
    /// seule fois par handle (les doublons et les handles périmés sont sans effet).
    ///
    /// Les handles de destruction sont évalués contre l'état du `World` avant le lot :
    /// l'état de référence d'un producteur est le monde stable de sa phase, et aucune
    /// commande ne peut désigner une entité créée par le même lot. Pour tout handle émis par ce
    /// `World`, cela coïncide avec l'application séquentielle des primitives (test exhaustif) ;
    /// pour un handle étranger, la prévalidation est au pire plus stricte, jamais plus
    /// permissive : prévalidé implique applicable.
    fn validate(&self, batch: &[Command]) -> Result<(), CapacityExceeded> {
        // Chemin rapide, O(n) : les destructions ne peuvent qu'ajouter de la capacité.
        let spawns = batch
            .iter()
            .filter(|command| matches!(command, Command::Spawn))
            .count();
        if spawns <= self.entities.free_slots() {
            return Ok(());
        }
        // Simulation ordonnée du seul décompte de slots libres.
        let mut available = self.entities.free_slots();
        for (position, command) in batch.iter().enumerate() {
            match *command {
                Command::Spawn => {
                    available = available.checked_sub(1).ok_or(CapacityExceeded)?;
                }
                Command::Despawn(id) => {
                    let first_occurrence = !batch[..position].contains(&Command::Despawn(id));
                    if first_occurrence && self.entities.frees_slot_on_release(id) {
                        available += 1;
                    }
                }
            }
        }
        Ok(())
    }
}

impl<const ENTITIES: usize> Default for World<ENTITIES> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `01-runtime-ecs.md`, section 2 : le `World` crée les entités ; un handle émis est valide.
    #[test]
    fn spawn_produces_a_valid_handle() {
        let mut world = World::<4>::new();
        let a = world.spawn().unwrap();
        let b = world.spawn().unwrap();

        assert_ne!(a, b);
        assert!(world.is_alive(a) && world.is_alive(b));
    }

    /// Cas canonique de la roadmap, au niveau du `World` : despawn puis réutilisation de
    /// l'index libéré ; l'ancien handle doit rester invalide après réattribution.
    /// Capacité 1 : la réutilisation de l'index est forcée.
    #[test]
    fn despawn_invalidates_the_handle_even_after_index_reuse() {
        let mut world = World::<1>::new();
        let old = world.spawn().unwrap();

        world.despawn(old).unwrap();
        assert!(!world.is_alive(old));

        let new = world.spawn().unwrap();
        assert_ne!(new, old);
        assert!(world.is_alive(new));
        assert!(!world.is_alive(old), "l'ancien handle doit rester invalide");
    }

    #[test]
    fn despawning_a_stale_handle_is_refused_and_spares_the_new_occupant() {
        let mut world = World::<1>::new();
        let old = world.spawn().unwrap();
        world.despawn(old).unwrap();
        let new = world.spawn().unwrap();

        assert_eq!(world.despawn(old), Err(StaleEntity));
        assert!(world.is_alive(new));
    }

    /// ADR-001, point 5 (étendu à l'identité) : dépassement de capacité = erreur explicite.
    #[test]
    fn spawn_beyond_capacity_is_an_explicit_error() {
        let mut world = World::<2>::new();
        let a = world.spawn().unwrap();
        let b = world.spawn().unwrap();

        assert_eq!(world.spawn(), Err(CapacityExceeded));
        assert!(world.is_alive(a) && world.is_alive(b));
    }

    /// ADR-006, point 3 : plusieurs producteurs accumulent dans un même buffer ; un seul
    /// `apply` les résout, dans l'ordre d'accumulation, exactement comme les primitives
    /// directes de la Phase 3 appelées dans le même ordre.
    #[test]
    fn apply_matches_direct_primitives_in_accumulation_order() {
        fn populate(world: &mut World<4>) -> [EntityId; 3] {
            [
                world.spawn().unwrap(),
                world.spawn().unwrap(),
                world.spawn().unwrap(),
            ]
        }
        let mut world = World::<4>::new();
        let mut reference = World::<4>::new();
        let [e1, e2, e3] = populate(&mut world);
        assert_eq!(
            populate(&mut reference),
            [e1, e2, e3],
            "construction déterministe"
        );

        // Producteur A : destructions. Producteur B : créations. Même buffer.
        let mut buffer = CommandBuffer::<8>::new();
        buffer.despawn(e1).unwrap(); // A
        buffer.spawn().unwrap(); // B
        buffer.despawn(e3).unwrap(); // A
        buffer.spawn().unwrap(); // B
        assert_eq!(world.apply(&mut buffer), Ok(()));

        reference.despawn(e1).unwrap();
        let n1 = reference.spawn().unwrap();
        reference.despawn(e3).unwrap();
        let n2 = reference.spawn().unwrap();

        for id in [e1, e2, e3, n1, n2] {
            assert_eq!(world.is_alive(id), reference.is_alive(id));
        }
        assert!(world.is_alive(e2) && world.is_alive(n1) && world.is_alive(n2));
        assert!(!world.is_alive(e1) && !world.is_alive(e3));
    }

    /// `01-runtime-ecs.md`, section 2 : aucune mutation structurelle directe pendant la phase.
    /// Une destruction accumulée n'a aucun effet avant `apply`.
    #[test]
    fn despawn_is_deferred_until_apply() {
        let mut world = World::<2>::new();
        let e = world.spawn().unwrap();
        let mut buffer = CommandBuffer::<2>::new();
        buffer.despawn(e).unwrap();

        assert!(world.is_alive(e));
        world.apply(&mut buffer).unwrap();
        assert!(!world.is_alive(e));
    }

    /// Une création accumulée n'occupe aucune capacité avant `apply` ; le dépassement n'est
    /// détecté — et signalé explicitement — qu'à l'application.
    #[test]
    fn spawn_is_deferred_until_apply() {
        let mut world = World::<1>::new();
        let mut buffer = CommandBuffer::<2>::new();
        buffer.spawn().unwrap();

        let direct = world.spawn(); // la capacité est encore libre
        assert!(direct.is_ok());
        assert_eq!(world.apply(&mut buffer), Err(CapacityExceeded));
    }

    /// Le buffer est transitoire : vide après application, mémoire réutilisable.
    #[test]
    fn apply_empties_the_buffer_and_keeps_its_capacity() {
        let mut world = World::<4>::new();
        let mut buffer = CommandBuffer::<2>::new();
        buffer.spawn().unwrap();
        buffer.spawn().unwrap();
        world.apply(&mut buffer).unwrap();
        assert!(buffer.is_empty());

        buffer.spawn().unwrap();
        buffer.spawn().unwrap();
        world.apply(&mut buffer).unwrap();
        assert!(buffer.is_empty());
    }

    /// Deux producteurs détruisent la même entité (« une entité référencée peut avoir
    /// disparu », `01-runtime-ecs.md`, section 1) : la seconde destruction est sans effet et
    /// ne touche pas le nouvel occupant de l'index. Choix local, non spécifié par le corpus.
    #[test]
    fn duplicate_despawn_is_ignored_and_spares_the_new_occupant() {
        let mut world = World::<1>::new();
        let mut reference = World::<1>::new();
        let e = world.spawn().unwrap();
        reference.spawn().unwrap();

        let mut buffer = CommandBuffer::<4>::new();
        buffer.despawn(e).unwrap();
        buffer.spawn().unwrap();
        buffer.despawn(e).unwrap(); // handle désormais périmé
        assert_eq!(world.apply(&mut buffer), Ok(()));

        reference.despawn(e).unwrap();
        let new = reference.spawn().unwrap();
        assert!(world.is_alive(new));
        assert!(!world.is_alive(e));
    }

    /// ADR-006, point 3 : application atomique. Un échec après des commandes applicables ne
    /// modifie pas le `World` : ni les entités vivantes, ni les générations, ni la liste libre.
    #[test]
    fn failed_apply_leaves_the_world_entirely_unchanged() {
        let mut world = World::<2>::new();
        let e1 = world.spawn().unwrap();
        let before = world.clone();

        let mut buffer = CommandBuffer::<4>::new();
        buffer.despawn(e1).unwrap(); // applicable
        buffer.spawn().unwrap(); // applicable
        buffer.spawn().unwrap(); // applicable
        buffer.spawn().unwrap(); // échoue : 3 créations pour 2 slots
        assert_eq!(world.apply(&mut buffer), Err(CapacityExceeded));

        assert_eq!(world, before, "état complet inchangé");
        assert!(
            world.is_alive(e1),
            "la destruction applicable n'a pas eu lieu"
        );
        // Le cycle de vie ultérieur est celui du monde non touché.
        let mut reference = before.clone();
        assert_eq!(world.spawn(), reference.spawn());
    }

    /// Comportement du buffer en cas d'échec : transaction non consommée. Il est conservé
    /// intact (contenu et ordre), pour diagnostic ; l'appelant le vide explicitement
    /// avec `clear`. Un échec ignoré se répète à l'identique à la phase suivante, jamais
    /// en silence. Après un succès, le buffer est vide.
    #[test]
    fn failed_apply_preserves_the_buffer_until_cleared() {
        let mut world = World::<1>::new();
        let e = world.spawn().unwrap();
        let mut buffer = CommandBuffer::<3>::new();
        buffer.spawn().unwrap(); // échoue : monde plein
        buffer.despawn(e).unwrap();

        assert_eq!(world.apply(&mut buffer), Err(CapacityExceeded));
        assert_eq!(buffer.as_slice(), [Command::Spawn, Command::Despawn(e)]);
        assert_eq!(world.apply(&mut buffer), Err(CapacityExceeded), "répétable");
        assert_eq!(buffer.len(), 2);

        buffer.clear();
        assert!(buffer.is_empty());
        assert_eq!(world.apply(&mut buffer), Ok(()), "lot vide : sans effet");
    }

    /// Exactitude de la prévalidation : pour tout petit lot, `apply` échoue si et seulement
    /// si l'application séquentielle des primitives échouerait ; en cas d'échec le `World` et
    /// le buffer sont inchangés, en cas de succès le `World` est identique à celui des
    /// primitives appliquées dans l'ordre. Couvre doublons, handles périmés et slot retiré.
    #[test]
    fn prevalidation_is_exact_and_atomic_over_all_small_batches() {
        const LEN: usize = 5;

        // Scénarios : (monde, [a, b, périmé]).
        fn scenarios() -> [(World<3>, [EntityId; 3]); 5] {
            let empty = World::<3>::new();
            let stale_of = |mut w: World<3>| {
                let s = w.spawn().unwrap();
                w.despawn(s).unwrap();
                (w, s)
            };
            let (one, s1) = stale_of(World::<3>::new());
            let mut one = one;
            let a1 = one.spawn().unwrap();
            let mut two = World::<3>::new();
            let (a2, b2) = (two.spawn().unwrap(), two.spawn().unwrap());
            let mut full = World::<3>::new();
            let (a3, b3, _c3) = (
                full.spawn().unwrap(),
                full.spawn().unwrap(),
                full.spawn().unwrap(),
            );
            // Slot 0 à une génération de l'épuisement : sa destruction le retire.
            let mut retiring = World::<3>::new();
            retiring.entities.set_generation_for_test(0, u32::MAX - 1);
            let (a4, b4) = (retiring.spawn().unwrap(), retiring.spawn().unwrap());
            [
                (empty, [EntityId::VACANT; 3]),
                (one, [a1, EntityId::VACANT, s1]),
                (two, [a2, b2, EntityId::VACANT]),
                (full, [a3, b3, EntityId::VACANT]),
                (retiring, [a4, b4, EntityId::VACANT]),
            ]
        }

        for (world0, [a, b, stale]) in scenarios() {
            // Alphabet : Spawn et Despawn des trois handles.
            let alphabet = [
                Command::Spawn,
                Command::Despawn(a),
                Command::Despawn(b),
                Command::Despawn(stale),
            ];
            for length in 0..=LEN {
                for code in 0..alphabet.len().pow(length as u32) {
                    let mut sequence = [Command::Spawn; LEN];
                    let mut rest = code;
                    for slot in sequence.iter_mut().take(length) {
                        *slot = alphabet[rest % alphabet.len()];
                        rest /= alphabet.len();
                    }
                    let sequence = &sequence[..length];

                    // Oracle : primitives appliquées dans l'ordre sur une copie.
                    let mut oracle = world0.clone();
                    let mut oracle_failed = false;
                    for command in sequence {
                        match *command {
                            Command::Spawn => oracle_failed |= oracle.spawn().is_err(),
                            Command::Despawn(id) => {
                                let _ = oracle.despawn(id);
                            }
                        }
                        if oracle_failed {
                            break;
                        }
                    }

                    let mut world = world0.clone();
                    let mut buffer = CommandBuffer::<LEN>::new();
                    for command in sequence {
                        match *command {
                            Command::Spawn => buffer.spawn().unwrap(),
                            Command::Despawn(id) => buffer.despawn(id).unwrap(),
                        }
                    }
                    let result = world.apply(&mut buffer);

                    if oracle_failed {
                        assert_eq!(result, Err(CapacityExceeded), "{sequence:?}");
                        assert_eq!(world, world0, "{sequence:?}");
                        assert_eq!(buffer.as_slice(), sequence, "{sequence:?}");
                    } else {
                        assert_eq!(result, Ok(()), "{sequence:?}");
                        assert_eq!(world, oracle, "{sequence:?}");
                        assert!(buffer.is_empty(), "{sequence:?}");
                    }
                }
            }
        }
    }
}
