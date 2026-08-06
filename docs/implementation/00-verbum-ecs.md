# Plan de construction — `verbum-ecs` (détail de l'étape 0)

## Statut
Document de planification interne, non normatif, révisable sans ADR. Détaille l'étape 0 de `ROADMAP.md` en micro-phases. N'introduit aucun invariant nouveau — chaque phase cite l'invariant du corpus qui la justifie.

## Règle gouvernante
> Un concept n'est codé que s'il est porté par un invariant déjà adopté dans le corpus, ou rendu strictement nécessaire par le test de la phase en cours. Aucune API n'est écrite par anticipation de besoins futurs.

Corollaire direct de la règle méta-architecturale de `00-principes.md` (« une ADR ne doit figer que des invariants structurels »), étendu du document au code. Sont donc explicitement exclus tant qu'aucun invariant ne les réclame : `EntityBuilder`, `Bundle`, `Archetype`, `Scene`, `Prefab`, `Registry`, `Plugin`, `Service`, `Factory` — aucun n'apparaît nommément dans le corpus.

Méthode par phase : **invariant cité → test qui l'exprime → API minimale qui fait passer ce test → un commit par invariant démontré**. Le test précède l'écriture de l'API, jamais l'inverse. Si une phase révèle deux invariants indépendants, deux commits sont légitimes ; le découpage des commits suit la logique du corpus, pas la numérotation de ce document.

**Signal d'alarme** : si une pensée du type « tant que j'y suis » ou « ça servira plus tard » apparaît pendant l'écriture, c'est le signe qu'on code hors du périmètre de la phase en cours — la réponse correcte est de s'arrêter, pas de continuer.

**Règle de processus** (invariant du développement, pas du moteur) : chaque commit doit laisser le dépôt dans un état publiable (`cargo test` vert), même si la phase globale n'est pas terminée — dans le même esprit de réversibilité que le reste de la démarche, sans être lui-même un invariant architectural.

---

## Frontières internes de `verbum-ecs`

| Module | Responsabilité | Dépend de | Ne doit jamais dépendre de | Source |
|---|---|---|---|---|
| `entity` | `EntityId`, table de génération | rien | tout le reste | `01-runtime-ecs.md`, section 1 |
| `storage` | Sparse Set, un stockage isolé par type de composant | `entity` | `query` | ADR-001, points 2–4 |
| `resource` | donnée de cardinalité 1, clé unique | rien | `query` | ADR-005, point 1 |
| `command` | Command Buffer, accumulation libre / résolution unique | `entity` | `query` ; `storage` directement (passe toujours par `world`) | ADR-006 |
| `query` | vue passive sur les stockages exposés par `world` | `storage`, `resource` | rien ne doit dépendre de `query` | ADR-002 |
| `world` | identité, cycle de vie, expose les stockages, applique le Command Buffer | `entity`, `storage`, `resource`, `command` | **`query`, sans exception** | ADR-001, point 3 : *« Cette séparation exclut par construction toute méthode `world.query()`, tout cache de requête interne au World, ou toute optimisation couplant `World` et `Query`. »* |

La dernière ligne est la frontière la plus importante du module : c'est elle qui empêche `World` de redevenir un dieu-objet qui connaîtrait les requêtes.

---

## Micro-phases

### Phase 1 — `EntityId`
**Invariant** : identifiant opaque, validité par `(index, génération)` (`01-runtime-ecs.md`, section 1).
**Test** : après despawn d'un index puis réattribution de cet index à une nouvelle entité, l'ancien `EntityId` doit être invalide.
**API minimale** : `EntityId` (opaque, accesseurs privés), table de génération brute.
**Commit** : `entity: EntityId opaque + table de génération`.

### Phase 2 — `SparseSet<T>`
**Invariant** : swap-remove, contiguïté du tableau dense, isolation stricte entre stockages (ADR-001, points 3–4).
**Réduction volontaire** : pas encore de notion de « Composant » ni de trait dédié — un `SparseSet<T>` générique sur n'importe quel `T`, sans dépendance à un registre ou à un mécanisme de dispatch. Le but de cette phase est de démontrer ADR-001, pas de démontrer un ECS complet.
**Test** : insert/remove préserve la contiguïté sur une instance `SparseSet<Position>` ; deux instances indépendantes (`SparseSet<Position>` et `SparseSet<Velocity>`) n'interagissent jamais, même en cas d'erreur dans l'une des deux.
**API minimale** : `SparseSet::<T>::new()`, `insert(EntityId, T)`, `remove(EntityId)`, `get(EntityId) -> Option<&T>`, `contains(EntityId) -> bool`.
**Commit** : `storage: SparseSet<T> générique, sans notion de composant`.

### Phase 3 — World minimal (identité seule)
**Invariant** : seul le `World` crée/détruit des entités (`01-runtime-ecs.md`, section 2).
**Réduction volontaire** : le test de cette phase ne porte que sur `spawn`/`despawn`/validité de handle — aucun composant n'y est impliqué. Le `World` n'a donc, à ce stade, aucun stockage de composant : seulement l'identité et son cycle de vie (`generations`, `freelist` ou équivalent). Le premier `SparseSet<T>` n'apparaîtra dans `World` qu'en Phase 4, au moment exact où `add_component` l'exigera.
**Précision de séquencement** (au-delà de ce que l'audit avait relevé) : à cette phase, `spawn`/`despawn` sont des primitives directes du `World`, appelées par le test lui-même — pas encore par un Command Buffer. Ce n'est pas une violation : l'invariant *« les systèmes ne créent jamais d'identifiants eux-mêmes »* vise les systèmes, pas un test unitaire du `World` en l'absence de tout système. Le Command Buffer (phase 4) viendra s'ajouter en façade de ces primitives, pas les remplacer.
**Test** : `world.spawn()` produit un `EntityId` valide ; `world.despawn()` invalide l'ancien handle même après réutilisation de l'index (répétition du test de la phase 1, au niveau du `World` cette fois).
**API minimale** : `World::spawn()`, `World::despawn()`, uniquement les structures d'identité (pas de champ de composant).
**Commit** : `world: identité et cycle de vie minimal, aucun stockage de composant`.

### Phase 4 — Command Buffer
**Invariant** : accumulation libre en écriture, résolution unique par le `World` au point de validation (ADR-006) ; le `World` expose des stockages sans connaître les requêtes (ADR-001, point 3) — cette seconde partie de l'invariant n'a de raison d'apparaître qu'ici, puisque c'est `add_component` qui, le premier, exige un stockage.

Cette phase démontre deux invariants indépendants ; elle donne donc lieu à deux commits distincts (cf. règle de granularité ci-dessus) :

**4a — spawn/despawn différés, sans composant**
**Test** : plusieurs producteurs accumulent des commandes `Spawn`/`Despawn` distinctes dans le même buffer ; un seul flush les applique de façon atomique en appelant les primitives déjà posées en Phase 3 ; le buffer est vide après flush.
**API minimale** : `CommandBuffer::spawn(...)`, `::despawn(...)`, `World::apply(CommandBuffer)`.
**Commit** : `command: Command Buffer pour spawn/despawn, aucune donnée composant`.

**4b — add_component, première apparition d'un stockage dans World**
**Test** : un `add_component(Position)` accumulé puis flushé rend la donnée lisible directement depuis le stockage exposé par `World` — première vérification que « le `World` expose des stockages » (ADR-001, point 3) est réellement observable, pas seulement énoncée.
**API minimale** : `CommandBuffer::add_component(...)`, apparition du premier champ direct par type sur `World` (`positions: SparseSet<Position>`) — introduit uniquement parce que ce test l'exige maintenant, pas avant. Aucun registre, aucun `TypeId`, aucune `HashMap` de dispatch : chaque nouveau type de composant s'ajoute délibérément comme un champ, jusqu'à ce qu'un besoin réel force une automatisation (décision d'implémentation différée, jamais anticipée).
**Commit** : `world+command: premier SparseSet<T> intégré à World, add_component via flush`.

### Phase 5 — Ressource
**Invariant** : donnée de cardinalité 1 par `World`, mêmes contrats qu'un composant (ADR-005).
**Test** : une Ressource insérée est lisible et modifiable ; son cardinal reste 1 (une seconde insertion remplace, ne duplique jamais).
**API minimale** : `World::insert_resource<T>()`, `World::get_resource<T>()`.
**Commit** : `resource: cardinalité 1, mêmes contrats qu'un composant`.
**Note** : pas de type `ResourceId` figé à ce stade — ADR-005, point 1, laisse explicitement la clé concrète (« type Rust, `TypeId`, identifiant généré par la Forge… ») comme décision d'implémentation. En figer un maintenant contredirait la règle gouvernante de ce document.

### Phase 6 — Query (lecture et lecture/écriture)
**Invariant** : vue passive, spécialisée statiquement, sans production ni transformation, mode d'accès explicite dans le type (ADR-002, points 1, 2, 3, 7).
**Réduction volontaire** : le contrat porte sur un comportement, jamais sur une syntaxe. La première implémentation n'a pas besoin de la forme ergonomique `Query<(&Position, &mut Velocity)>` — un mécanisme d'itération minimal (même sous une forme aussi rudimentaire qu'une fonction retournant un itérateur manuel sur des paires) suffit tant qu'il démontre le comportement attendu. La syntaxe tuple ou les macros de spécialisation sont un raffinement ultérieur, jamais une condition du contrat (ADR-002, point 2 : la spécialisation statique « reste valable si une partie de la spécialisation est un jour déportée » vers un autre mécanisme).
**Test** : une vue combinant lecture de `Position` et lecture/écriture de `Velocity` sur les mêmes entités itère exactement les entités possédant les deux ; la donnée elle-même n'est jamais transformée par la Query ; aucune Query ne subsiste après la fin de son itération (pas de cache, pas d'état durable) ; le mode d'accès (lecture vs écriture) est visible dans le type dès cette première implémentation, même rudimentaire.
**API minimale** : laissée délibérément ouverte à ce stade — voir « Réduction volontaire » ci-dessus.
**Commit** : `query: vue statique, lecture et lecture/écriture, mode d'accès explicite`.

---

## Critère de sortie de l'étape 0
Le test global déjà posé dans `ROADMAP.md` (« spawn → ajout composant → query → command buffer → resolve → query ») devient l'**acceptance test** de l'ensemble des six phases, pas le test d'une phase isolée — il ne doit passer qu'une fois les phases 1 à 6 toutes vertes.

## Ce que ce document ne tranche pas
Le découpage en crates Rust séparés vs. modules d'un seul crate, les traits concrets (`Component`, `Resource`), la stratégie de parallélisme intra-Sparse-Set — tout cela reste une décision d'implémentation à prendre au moment d'écrire le code, pas ici.

---

_le 5 août 2026_
