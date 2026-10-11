//! `World` : identité, cycle de vie des entités et stockages de composants.
//!
//! Seul le `World` crée et détruit des entités et mute les stockages (`01-runtime-ecs.md`,
//! sections 2 et 3). Il ignore toute sémantique métier : l'ensemble des stockages est un
//! `ComponentSet` fourni par l'appelant (cf. `component_set`), accessible en lecture par
//! `components()` ; toute mutation passe par `spawn`/`despawn` ou par `apply` (ADR-006).

use crate::command::{Command, CommandBuffer};
use crate::component_set::ComponentSet;
use crate::entity::{CapacityExceeded, EntityId, GenerationTable, StaleEntity};
use crate::storage::InsertError;

/// Échec de `World::apply` : première commande inapplicable dans l'ordre d'émission.
#[derive(Debug, PartialEq, Eq)]
pub enum ApplyError {
    /// Une création dépasse la capacité en entités.
    Spawn(CapacityExceeded),
    /// Un ajout viole un stockage : doublon, dense plein ou index hors de l'étendue sparse.
    Add(InsertError),
}

/// Échec de la primitive d'ajout. Le handle périmé (ignoré par `apply`) se distingue
/// explicitement de l'erreur de stockage (rejet du lot).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum AddError {
    Stale(StaleEntity),
    Insert(InsertError),
}

/// Monde de `ENTITIES` entités au plus, portant les stockages `S` et les Ressources `R`
/// (champs `ResourceSlot`, cardinalité 1, sans lien avec les entités). La capacité est fixée avant
/// la simulation et toute la mémoire est portée par le `World` lui-même : aucune allocation.
#[cfg_attr(test, derive(Clone, Debug, PartialEq))]
pub struct World<const ENTITIES: usize, S: ComponentSet = (), R = ()> {
    entities: GenerationTable<ENTITIES>,
    components: S,
    resources: R,
}

impl<const ENTITIES: usize, S: ComponentSet, R: Default> World<ENTITIES, S, R> {
    pub fn new() -> Self {
        Self {
            entities: GenerationTable::new(),
            components: S::default(),
            resources: R::default(),
        }
    }

    /// Stockages en lecture. Aucun accès mutable : les mutations structurelles passent par le
    /// `World`. Le stockage ne revalide pas la génération (ADR-001, point 2) : l'appelant
    /// vérifie la vivacité du handle avec `is_alive`.
    pub fn components(&self) -> &S {
        &self.components
    }

    /// Ressources en lecture (ADR-005) : accès statique par champ, jamais par clé dynamique.
    pub fn resources(&self) -> &R {
        &self.resources
    }

    /// Ressources en écriture. Ne donne accès qu'à `R` : ni identité, ni stockages. Une
    /// Ressource n'est liée à aucune entité (ni purge, ni lot) ; le propriétaire unique en
    /// écriture (ADR-005, point 5) relève des systèmes, pas du `World`.
    pub fn resources_mut(&mut self) -> &mut R {
        &mut self.resources
    }

    /// Accès mutable aux **valeurs** des composants, jamais à la structure : `S::Values` est
    /// composé de `ValuesMut` (type scellé), sans `insert` ni `remove`. Seul le `World` résout
    /// les mutations structurelles (ADR-006). Une Query en lecture/écriture se construit
    /// depuis ces vues (`ValuesMut::shared` pour la lecture), sans `world.query()`.
    pub fn component_values_mut(&mut self) -> S::Values<'_> {
        self.components.values_mut()
    }

    /// Crée une entité. Erreur explicite si la capacité est atteinte, jamais une croissance.
    pub fn spawn(&mut self) -> Result<EntityId, CapacityExceeded> {
        self.entities.allocate()
    }

    /// Détruit une entité et purge ses composants de tous les stockages, avant que son slot
    /// puisse être réutilisé. Refuse un handle périmé sans toucher à l'occupant actuel de
    /// l'index : ni identité, ni composants.
    pub fn despawn(&mut self, id: EntityId) -> Result<(), StaleEntity> {
        self.entities.release(id)?;
        self.components.remove_all(id);
        Ok(())
    }

    /// Ajoute un composant à une entité vivante. Primitive de référence de `apply`.
    pub(crate) fn add_component(&mut self, id: EntityId, add: S::Add) -> Result<(), AddError> {
        if !self.entities.is_alive(id) {
            return Err(AddError::Stale(StaleEntity));
        }
        self.components.insert(id, add).map_err(AddError::Insert)
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
    /// générations, liste libre, stockages) et le buffer est **conservé intact** : transaction
    /// non consommée, qu'un appelant qui ignore l'erreur retrouvera à l'identique à la phase
    /// suivante, jamais en silence. L'appelant le vide alors avec [`CommandBuffer::clear`].
    /// En cas de succès, le lot est appliqué en une seule passe, dans l'ordre d'accumulation,
    /// exactement selon la séquence prévalidée, puis le buffer est vidé. Le buffer n'est pas
    /// possédé par le `World` : sa mémoire reste à l'appelant.
    ///
    /// Sémantique du lot, dans l'ordre d'émission (choix locaux, non spécifiés par le corpus) :
    /// - un handle périmé (ou détruit plus tôt dans le lot) rend la destruction ou l'ajout
    ///   **sans effet** : plusieurs producteurs peuvent viser la même entité ;
    /// - un ajout avant la destruction de la cible est appliqué, puis purgé avec elle ;
    /// - ajout d'un composant déjà présent, stockage dense plein, index hors de l'étendue
    ///   sparse, création au-delà de la capacité : erreur, **lot entier rejeté** (un
    ///   dépassement de capacité est un bug de configuration, ADR-001, point 5) ;
    /// - les handles sont évalués contre le monde avant le lot : aucune commande ne peut
    ///   désigner une entité créée par ce lot.
    pub fn apply<const COMMANDS: usize>(
        &mut self,
        commands: &mut CommandBuffer<COMMANDS, S::Add>,
    ) -> Result<(), ApplyError> {
        let planned = self.plan::<COMMANDS>(commands.as_slice())?;

        for (command, &applies) in commands.as_slice().iter().zip(&planned) {
            if !applies {
                continue;
            }
            match *command {
                Command::Spawn => {
                    let spawned = self.spawn();
                    debug_assert!(spawned.is_ok(), "lot prévalidé");
                }
                Command::Despawn(id) => {
                    let despawned = self.despawn(id);
                    debug_assert!(despawned.is_ok(), "lot prévalidé");
                }
                Command::Add(id, add) => {
                    let added = self.add_component(id, add);
                    debug_assert!(added.is_ok(), "lot prévalidé");
                }
            }
        }
        commands.clear();
        Ok(())
    }

    /// Prévalidation d'un lot : simule dans l'ordre d'émission les effets logiques de chaque
    /// commande, sans mutation ni allocation (compteurs et drapeaux sur la pile). Renvoie, par
    /// commande, si elle a un effet ; `apply` n'exécute que celles-ci, de sorte que
    /// l'application coïncide exactement avec la séquence prévalidée.
    ///
    /// Simulé : le décompte de slots libres (créations, destructions effectives), et, par
    /// stockage, la longueur dense (ajouts effectifs, purges des destructions effectives). Une
    /// destruction est effective si le handle est vivant avant le lot et pas déjà détruit par
    /// lui ; un ajout, si sa cible l'est aussi. La présence d'un composant se lit dans le
    /// stockage avant le lot, complétée par les ajouts effectifs précédents (aucune purge
    /// n'intervient sur une cible encore vivante). Erreurs dans l'ordre de `SparseSet::insert` :
    /// hors étendue, doublon, dense plein.
    ///
    /// Coût O(n²) (parcours de préfixe par commande) : borné par la capacité du buffer, fixée
    /// avant la simulation ; optimisable sans changer le contrat (mesure d'abord).
    fn plan<const COMMANDS: usize>(
        &self,
        batch: &[Command<S::Add>],
    ) -> Result<[bool; COMMANDS], ApplyError> {
        let mut applies = [false; COMMANDS];
        let mut free = self.entities.free_slots();
        let mut lengths = S::Counters::default();
        for (kind, length) in lengths.as_mut().iter_mut().enumerate() {
            *length = self.components.dense_len(kind);
        }
        let kinds = lengths.as_mut().len();

        for (position, command) in batch.iter().enumerate() {
            let before = &batch[..position];
            match *command {
                Command::Spawn => {
                    free = free
                        .checked_sub(1)
                        .ok_or(ApplyError::Spawn(CapacityExceeded))?;
                    applies[position] = true;
                }
                Command::Despawn(id) => {
                    if self.is_live_at(id, before) {
                        applies[position] = true;
                        if self.entities.frees_slot_on_release(id) {
                            free += 1;
                        }
                        for kind in 0..kinds {
                            if self.holds(kind, id, before, &applies) {
                                lengths.as_mut()[kind] -= 1;
                            }
                        }
                    }
                }
                Command::Add(id, add) => {
                    if self.is_live_at(id, before) {
                        let kind = S::kind(&add);
                        if !self.components.covers(kind, id) {
                            return Err(ApplyError::Add(InsertError::IndexOutOfRange));
                        }
                        if self.holds(kind, id, before, &applies) {
                            return Err(ApplyError::Add(InsertError::AlreadyPresent));
                        }
                        if lengths.as_mut()[kind] >= self.components.dense_capacity(kind) {
                            return Err(ApplyError::Add(InsertError::DenseFull));
                        }
                        lengths.as_mut()[kind] += 1;
                        applies[position] = true;
                    }
                }
            }
        }
        Ok(applies)
    }

    /// Le handle est vivant avant le lot et aucune destruction qui le vise ne le précède dans
    /// le lot (la première destruction d'un handle vivant est toujours effective).
    fn is_live_at(&self, id: EntityId, before: &[Command<S::Add>]) -> bool {
        self.entities.is_alive(id)
            && !before
                .iter()
                .any(|command| matches!(*command, Command::Despawn(d) if d == id))
    }

    /// L'entité (vivante à ce point du lot) possède un composant de ce `kind` : présent avant le
    /// lot, ou ajouté par une commande précédente effective.
    fn holds(
        &self,
        kind: usize,
        id: EntityId,
        before: &[Command<S::Add>],
        applies: &[bool],
    ) -> bool {
        self.components.is_present(kind, id)
            || before.iter().zip(applies).any(|(command, &applied)| {
                applied
                    && matches!(*command, Command::Add(target, add)
                        if target == id && S::kind(&add) == kind)
            })
    }
}

impl<const ENTITIES: usize, S: ComponentSet, R: Default> Default for World<ENTITIES, S, R> {
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
        assert_eq!(
            world.apply(&mut buffer),
            Err(ApplyError::Spawn(CapacityExceeded))
        );
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
        assert_eq!(
            world.apply(&mut buffer),
            Err(ApplyError::Spawn(CapacityExceeded))
        );

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

        assert_eq!(
            world.apply(&mut buffer),
            Err(ApplyError::Spawn(CapacityExceeded))
        );
        assert_eq!(buffer.as_slice(), [Command::Spawn, Command::Despawn(e)]);
        assert_eq!(
            world.apply(&mut buffer),
            Err(ApplyError::Spawn(CapacityExceeded)),
            "répétable"
        );
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
        // Capacité du buffer : longueur maximale des lots, donc couverture complète hors Miri.
        const LEN: usize = 5;
        // Sous Miri, l'exploration est limitée à 3 commandes (mêmes scénarios, mêmes assertions) :
        // le coût de l'interpréteur rend les 5 commandes prohibitives. Les chemins de
        // prévalidation et d'application restent exercés ; les lots de 4 et 5 commandes ne le
        // sont que par les tests ordinaires.
        const MAX_LENGTH: usize = if cfg!(miri) { 3 } else { LEN };

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
            for length in 0..=MAX_LENGTH {
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
                            Command::Add(_, add) => match add {}, // inhabité
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
                            Command::Add(_, add) => match add {},
                        }
                    }
                    let result = world.apply(&mut buffer);

                    if oracle_failed {
                        assert_eq!(
                            result,
                            Err(ApplyError::Spawn(CapacityExceeded)),
                            "{sequence:?}"
                        );
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

    // ---- Phase 4b : composants ----------------------------------------------------------

    use crate::fixtures::{Kinematics, KinematicsAdd, Position, Velocity, pos, vel};

    type KWorld<const E: usize> = World<E, Kinematics>;
    type KBuffer<const C: usize> = CommandBuffer<C, KinematicsAdd>;

    fn position(x: i32) -> Position {
        Position { x, y: -x }
    }

    fn velocity(dx: i32) -> Velocity {
        Velocity { dx, dy: -dx }
    }

    /// Deux types distincts dans un même `World`, stockages indépendants, ajouts différés.
    #[test]
    fn two_distinct_component_types_live_in_one_world() {
        let mut world = KWorld::<3>::new();
        let (a, b) = (world.spawn().unwrap(), world.spawn().unwrap());
        let mut buffer = KBuffer::<3>::new();
        buffer.add_component(a, pos(1)).unwrap();
        buffer.add_component(a, vel(2)).unwrap();
        buffer.add_component(b, pos(3)).unwrap();
        assert_eq!(world.components().positions.len(), 0, "différé");

        assert_eq!(world.apply(&mut buffer), Ok(()));
        let set = world.components();
        assert_eq!(set.positions.get(a), Some(&position(1)));
        assert_eq!(set.velocities.get(a), Some(&velocity(2)));
        assert_eq!(set.positions.get(b), Some(&position(3)));
        assert_eq!(set.velocities.get(b), None);
        assert!(buffer.is_empty());
    }

    /// Une destruction valide purge tous les stockages avant la réutilisation du slot.
    #[test]
    fn despawn_purges_every_storage_before_slot_reuse() {
        let mut world = KWorld::<1>::new();
        let old = world.spawn().unwrap();
        world.add_component(old, pos(1)).unwrap();
        world.add_component(old, vel(2)).unwrap();

        world.despawn(old).unwrap();
        assert_eq!(world.components().positions.len(), 0);
        assert_eq!(world.components().velocities.len(), 0);

        let new = world.spawn().unwrap();
        assert_eq!(world.components().positions.get(new), None);
        assert_eq!(world.components().velocities.get(new), None);
    }

    /// Une destruction périmée ne retire jamais les composants du nouvel occupant (primitive
    /// et lot).
    #[test]
    fn stale_despawn_never_removes_the_new_occupants_components() {
        let mut world = KWorld::<1>::new();
        let old = world.spawn().unwrap();
        world.despawn(old).unwrap();
        let new = world.spawn().unwrap();
        world.add_component(new, pos(7)).unwrap();
        world.add_component(new, vel(8)).unwrap();

        assert_eq!(world.despawn(old), Err(StaleEntity));
        let mut buffer = KBuffer::<1>::new();
        buffer.despawn(old).unwrap();
        assert_eq!(world.apply(&mut buffer), Ok(()));

        assert!(world.is_alive(new));
        assert_eq!(world.components().positions.get(new), Some(&position(7)));
        assert_eq!(world.components().velocities.get(new), Some(&velocity(8)));
    }

    /// Un ajout sur un handle périmé est ignoré (pas une erreur), y compris quand le nouvel
    /// occupant du slot est vivant : il ne reçoit rien.
    #[test]
    fn add_on_a_stale_handle_is_ignored() {
        let mut world = KWorld::<1>::new();
        let old = world.spawn().unwrap();
        world.despawn(old).unwrap();
        let new = world.spawn().unwrap();

        let mut buffer = KBuffer::<2>::new();
        buffer.add_component(old, pos(1)).unwrap();
        buffer.add_component(EntityId::VACANT, vel(2)).unwrap();
        assert_eq!(world.apply(&mut buffer), Ok(()));

        assert_eq!(world.components().positions.len(), 0);
        assert_eq!(world.components().velocities.len(), 0);
        assert!(world.is_alive(new));
    }

    /// Ajout avant destruction : appliqué puis purgé. Destruction avant ajout : ajout ignoré.
    /// Le slot libéré par le lot est réutilisable sans hériter d'un composant.
    #[test]
    fn adds_before_and_after_destruction_in_one_batch() {
        let mut world = KWorld::<3>::new();
        let (a, b) = (world.spawn().unwrap(), world.spawn().unwrap());

        let mut buffer = KBuffer::<5>::new();
        buffer.add_component(a, pos(1)).unwrap(); // appliqué…
        buffer.despawn(a).unwrap(); // …puis purgé
        buffer.add_component(a, vel(2)).unwrap(); // cible détruite : ignoré
        buffer.add_component(b, pos(3)).unwrap();
        buffer.spawn().unwrap();
        assert_eq!(world.apply(&mut buffer), Ok(()));

        assert!(!world.is_alive(a));
        assert_eq!(world.components().positions.len(), 1);
        assert_eq!(world.components().positions.get(b), Some(&position(3)));
        assert_eq!(world.components().velocities.len(), 0);
    }

    /// La purge libère de la capacité dense au sein du lot : l'ajout suivant tient.
    #[test]
    fn purge_frees_dense_capacity_within_the_batch() {
        // `positions` : 2 composants au plus.
        let mut world = KWorld::<3>::new();
        let (a, b, c) = (
            world.spawn().unwrap(),
            world.spawn().unwrap(),
            world.spawn().unwrap(),
        );
        world.add_component(a, pos(1)).unwrap();
        world.add_component(b, pos(2)).unwrap(); // dense plein

        let mut buffer = KBuffer::<2>::new();
        buffer.add_component(c, pos(3)).unwrap();
        assert_eq!(
            world.apply(&mut buffer),
            Err(ApplyError::Add(InsertError::DenseFull))
        );
        buffer.clear();

        buffer.despawn(a).unwrap();
        buffer.add_component(c, pos(3)).unwrap();
        assert_eq!(world.apply(&mut buffer), Ok(()));
        assert_eq!(world.components().positions.get(c), Some(&position(3)));
        assert_eq!(world.components().positions.get(b), Some(&position(2)));
    }

    /// Une destruction dont le slot est retiré (génération épuisée) purge quand même.
    #[test]
    fn despawn_of_a_retiring_slot_still_purges() {
        let mut world = KWorld::<1>::new();
        world.entities.set_generation_for_test(0, u32::MAX - 1);
        let last = world.spawn().unwrap();
        world.add_component(last, pos(1)).unwrap();

        let mut buffer = KBuffer::<1>::new();
        buffer.despawn(last).unwrap();
        assert_eq!(world.apply(&mut buffer), Ok(()));
        assert_eq!(world.components().positions.len(), 0);
    }

    /// Doublons : déjà présent dans le stockage, ou ajouté plus tôt dans le lot, rejette le lot
    /// entier ; deux stockages distincts pour une même entité ne sont pas un doublon.
    #[test]
    fn duplicate_add_rejects_the_whole_batch() {
        let mut world = KWorld::<3>::new();
        let (a, b) = (world.spawn().unwrap(), world.spawn().unwrap());
        world.add_component(a, pos(1)).unwrap();
        let before = world.clone();

        let mut buffer = KBuffer::<3>::new();
        buffer.add_component(b, pos(2)).unwrap(); // applicable
        buffer.add_component(a, pos(3)).unwrap(); // déjà présent
        assert_eq!(
            world.apply(&mut buffer),
            Err(ApplyError::Add(InsertError::AlreadyPresent))
        );
        assert_eq!(world, before);
        assert_eq!(buffer.len(), 2);
        buffer.clear();

        buffer.add_component(b, vel(1)).unwrap();
        buffer.add_component(b, vel(2)).unwrap(); // doublon dans le lot
        assert_eq!(
            world.apply(&mut buffer),
            Err(ApplyError::Add(InsertError::AlreadyPresent))
        );
        assert_eq!(world, before);
        buffer.clear();

        buffer.add_component(a, vel(1)).unwrap(); // autre stockage : valide
        assert_eq!(world.apply(&mut buffer), Ok(()));
    }

    /// Handle périmé (ignoré) et index vivant hors étendue (erreur) : distinction explicite.
    /// `velocities` ne couvre que les index 0 et 1.
    #[test]
    fn stale_handle_is_ignored_while_live_out_of_range_index_is_an_error() {
        let mut world = KWorld::<3>::new();
        let _a = world.spawn().unwrap();
        let _b = world.spawn().unwrap();
        let c = world.spawn().unwrap(); // index 2

        let mut buffer = KBuffer::<1>::new();
        buffer.add_component(c, vel(1)).unwrap();
        assert_eq!(
            world.apply(&mut buffer),
            Err(ApplyError::Add(InsertError::IndexOutOfRange))
        );
        buffer.clear();

        world.despawn(c).unwrap(); // `c` est maintenant périmé, d'index inchangé
        buffer.add_component(c, vel(1)).unwrap();
        assert_eq!(world.apply(&mut buffer), Ok(()));
        assert_eq!(world.components().velocities.len(), 0);
    }

    /// Erreur = première commande inapplicable dans l'ordre d'émission, quelle que soit sa nature.
    #[test]
    fn first_failure_in_emission_order_is_reported() {
        let mut world = KWorld::<2>::new();
        let a = world.spawn().unwrap();
        let _b = world.spawn().unwrap();
        world.add_component(a, pos(1)).unwrap();

        let mut buffer = KBuffer::<2>::new();
        buffer.spawn().unwrap(); // monde plein
        buffer.add_component(a, pos(2)).unwrap(); // doublon
        assert_eq!(
            world.apply(&mut buffer),
            Err(ApplyError::Spawn(CapacityExceeded))
        );
        buffer.clear();
        buffer.add_component(a, pos(2)).unwrap();
        buffer.spawn().unwrap();
        assert_eq!(
            world.apply(&mut buffer),
            Err(ApplyError::Add(InsertError::AlreadyPresent))
        );
    }

    /// Primitive de référence : application séquentielle, la première erreur arrête.
    fn oracle<const E: usize, S: ComponentSet>(
        world: &mut World<E, S>,
        sequence: &[Command<S::Add>],
    ) -> Result<(), ApplyError> {
        for command in sequence {
            match *command {
                Command::Spawn => {
                    world.spawn().map_err(ApplyError::Spawn)?;
                }
                Command::Despawn(id) => {
                    let _ = world.despawn(id); // périmé : sans effet
                }
                Command::Add(id, add) => match world.add_component(id, add) {
                    Ok(()) | Err(AddError::Stale(_)) => {}
                    Err(AddError::Insert(error)) => return Err(ApplyError::Add(error)),
                },
            }
        }
        Ok(())
    }

    /// Exactitude et atomicité sur les stockages, sur tous les petits lots : même résultat
    /// que les primitives appliquées dans l'ordre (succès : monde identique, buffer vide ;
    /// échec : même erreur, monde et buffer inchangés). Couvre doublons, handles périmés,
    /// index hors étendue, dense plein, purge, slot retiré, monde vide.
    #[test]
    fn prevalidation_with_components_is_exact_and_atomic_over_all_small_batches() {
        // Capacité du buffer : longueur maximale des lots, donc couverture complète hors Miri.
        const LEN: usize = 5;
        // Sous Miri, l'exploration est limitée à 3 commandes (même alphabet, mêmes 5 scénarios,
        // mêmes assertions) : le coût de l'interpréteur rend les 5 commandes prohibitives.
        // Les chemins de prévalidation et d'application sur les stockages restent exercés ;
        // seuls les lots de 4 et 5 commandes ne le sont que par les tests ordinaires.
        const MAX_LENGTH: usize = if cfg!(miri) { 3 } else { LEN };
        type Seq = Command<KinematicsAdd>;

        // Scénarios : (monde, [x0, x1, x2]). `positions` : dense 2, 3 index ; `velocities` :
        // dense 2, 2 index (l'index 2 est hors étendue).
        fn scenarios() -> [(KWorld<3>, [EntityId; 3]); 5] {
            let spawn3 = |w: &mut KWorld<3>| [(); 3].map(|()| w.spawn().unwrap());

            // 1. Tout vivant, aucun composant (index 2 vivant : hors étendue de `velocities`).
            let mut w1 = KWorld::<3>::new();
            let h1 = spawn3(&mut w1);

            // 2. x2 périmé (index 2 libre, réattribuable), x0 porte une position.
            let mut w2 = KWorld::<3>::new();
            let h2 = spawn3(&mut w2);
            w2.despawn(h2[2]).unwrap();
            w2.add_component(h2[0], pos(9)).unwrap();

            // 3. Positions saturées (x0, x1) ; x1 porte aussi une vitesse.
            let mut w3 = KWorld::<3>::new();
            let h3 = spawn3(&mut w3);
            w3.add_component(h3[0], pos(1)).unwrap();
            w3.add_component(h3[1], pos(2)).unwrap();
            w3.add_component(h3[1], vel(3)).unwrap();

            // 4. Slot 0 à une génération de l'épuisement : sa destruction le retire.
            let mut w4 = KWorld::<3>::new();
            w4.entities.set_generation_for_test(0, u32::MAX - 1);
            let h4 = spawn3(&mut w4);
            w4.add_component(h4[0], pos(4)).unwrap();
            w4.add_component(h4[0], vel(5)).unwrap();

            // 5. Monde vide, handles étrangers : tout ajout et toute destruction sont ignorés.
            let w5 = KWorld::<3>::new();

            [
                (w1, h1),
                (w2, h2),
                (w3, h3),
                (w4, h4),
                (w5, [EntityId::VACANT; 3]),
            ]
        }

        for (world0, [x0, x1, x2]) in scenarios() {
            let alphabet: [Seq; 10] = [
                Command::Spawn,
                Command::Despawn(x0),
                Command::Despawn(x1),
                Command::Despawn(x2),
                Command::Add(x0, pos(10)),
                Command::Add(x0, vel(11)),
                Command::Add(x1, pos(12)),
                Command::Add(x1, vel(13)),
                Command::Add(x2, pos(14)),
                Command::Add(x2, vel(15)),
            ];
            for length in 0..=MAX_LENGTH {
                for code in 0..alphabet.len().pow(length as u32) {
                    let mut sequence = [Command::Spawn; LEN];
                    let mut rest = code;
                    for slot in sequence.iter_mut().take(length) {
                        *slot = alphabet[rest % alphabet.len()];
                        rest /= alphabet.len();
                    }
                    let sequence = &sequence[..length];

                    let mut expected = world0.clone();
                    let expected_result = oracle(&mut expected, sequence);

                    let mut world = world0.clone();
                    let mut buffer = KBuffer::<LEN>::new();
                    for command in sequence {
                        match *command {
                            Command::Spawn => buffer.spawn().unwrap(),
                            Command::Despawn(id) => buffer.despawn(id).unwrap(),
                            Command::Add(id, add) => buffer.add_component(id, add).unwrap(),
                        }
                    }
                    let result = world.apply(&mut buffer);

                    assert_eq!(result, expected_result, "{sequence:?}");
                    if result.is_ok() {
                        assert_eq!(world, expected, "{sequence:?}");
                        assert!(buffer.is_empty(), "{sequence:?}");
                    } else {
                        assert_eq!(world, world0, "{sequence:?}");
                        assert_eq!(buffer.as_slice(), sequence, "{sequence:?}");
                    }
                }
            }
        }
    }

    // ---- Phase 5 : Ressources -------------------------------------------------------------

    use crate::resource::ResourceSlot;

    #[derive(Clone, Debug, PartialEq, Default)]
    struct Globals {
        score: ResourceSlot<u32>,
        label: ResourceSlot<i8>,
    }

    /// Deux types de Ressource dans un monde, indépendants ; le monde vide n'en a aucune.
    #[test]
    fn world_holds_distinct_resources_by_static_field() {
        let mut world = World::<1, Kinematics, Globals>::new();
        assert_eq!(world.resources().score.get(), None);

        world.resources_mut().score.insert(10);
        world.resources_mut().label.insert(-3);
        *world.resources_mut().score.get_mut().unwrap() += 1;

        assert_eq!(world.resources().score.get(), Some(&11));
        assert_eq!(world.resources().label.get(), Some(&-3));
    }

    /// Une Ressource n'est liée à aucune entité : ni la destruction, ni un lot appliqué, ni un
    /// lot rejeté ne la modifient (atomicité : le monde rejeté est identique, Ressources
    /// comprises).
    #[test]
    fn resources_are_independent_of_entity_lifecycle_and_batches() {
        let mut world = World::<1, Kinematics, Globals>::new();
        world.resources_mut().score.insert(5);
        let e = world.spawn().unwrap();
        world.add_component(e, pos(1)).unwrap();

        let mut buffer = CommandBuffer::<2, KinematicsAdd>::new();
        buffer.spawn().unwrap(); // monde plein : lot rejeté
        let before = world.clone();
        assert_eq!(
            world.apply(&mut buffer),
            Err(ApplyError::Spawn(CapacityExceeded))
        );
        assert_eq!(world, before);
        buffer.clear();

        buffer.despawn(e).unwrap();
        assert_eq!(world.apply(&mut buffer), Ok(()));
        assert_eq!(world.resources().score.get(), Some(&5));
        assert_eq!(world.components().positions.len(), 0);
    }

    // ---- Intégration : accès mutable limité aux valeurs --------------------------------------

    /// La vue du `World` écrit les valeurs sans toucher à la structure ; la structure ne change
    /// que par `apply`, qui purge ensuite normalement ; un lot rejeté laisse intactes les valeurs
    /// déjà écrites (atomicité avec composants présents).
    #[test]
    fn world_values_view_writes_values_while_structure_stays_with_apply() {
        let mut world = KWorld::<3>::new();
        let (a, b) = (world.spawn().unwrap(), world.spawn().unwrap());
        world.add_component(a, pos(1)).unwrap();
        world.add_component(b, pos(2)).unwrap();
        world.add_component(a, vel(3)).unwrap();

        {
            let (mut positions, velocities) = world.component_values_mut();
            *positions.get_mut(a).unwrap() = position(100);
            assert_eq!(positions.entities(), [a, b], "ordre dense inchangé");
            assert_eq!((positions.len(), velocities.len()), (2, 1));
            assert!(!velocities.contains(b));
        }
        assert_eq!(world.components().positions.get(a), Some(&position(100)));
        assert!(world.is_alive(a) && world.is_alive(b));

        // Lot rejeté (doublon) : valeurs écrites et structure inchangées, buffer intact.
        let before = world.clone();
        let mut buffer = KBuffer::<2>::new();
        buffer.add_component(b, vel(4)).unwrap(); // applicable
        buffer.add_component(a, pos(5)).unwrap(); // déjà présent
        assert_eq!(
            world.apply(&mut buffer),
            Err(ApplyError::Add(InsertError::AlreadyPresent))
        );
        assert_eq!(world, before);
        assert_eq!(buffer.len(), 2);
        buffer.clear();

        // La destruction passe par `apply` et purge la valeur modifiée.
        buffer.despawn(a).unwrap();
        world.apply(&mut buffer).unwrap();
        assert_eq!(world.components().positions.entities(), [b]);
        assert_eq!(world.components().velocities.len(), 0);
    }
}
