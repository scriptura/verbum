# Plan de construction — étape 1, Scheduler (`verbum-schedule`) — révision 5

## Statut
Document de planification interne, non normatif, révisable sans ADR. **Soumis à revue finale : aucun code livré, aucune modification d'ADR, aucun commit avant validation explicite.** Détaille l'étape 1 de `ROADMAP.md` en micro-phases. N'introduit aucun invariant : chaque point cite sa source.

Révision 5, passe finale de cohérence (arbitrages A1 à A3, corrections ciblées, revue du document entier) :
- **A1** validé : `Locate`, `Binding`, `Bind` ; limite d'étiquetage conservée ; test de deux Ressources de même type ajouté (phase 7a).
- **A2** autorisé : `World::split_mut`, commit distinct (phase 6) ; signature exacte à valider à l'implémentation.
- **A3** modifié : `RemoveComponent` est corrigé dans un commit distinct de `verbum-ecs` (phase 6bis), **avant** l'intégration des commandes typées (7b) et avant l'acceptance ; il ne bloque pas les phases 0 à 5.
- **ADR-000** : statut « adopté le 10 octobre 2026 » confirmé par l'utilisateur dans le dépôt ; la question n'est pas rouverte.
- **Test 5 de la phase 9 corrigé** (aucune dépendance déduite entre producteur de commandes et lecteur) ; **test d'interchangeabilité des producteurs** ajouté (phase 10a).
- **§5.1 réécrit** : surface d'API, règles de prévalidation (ajout/retrait, rejets) et tests de `RemoveComponent`, sans la formule « dernière commande gagnante ».
- **Portée de l'atomicité** (I10, §6.1) : `World::apply` seul est atomique, sur son lot ; aucune atomicité de phase ni de tick.
- **Contrat d'erreur du tick et du buffer** (§6.2) : variant `TickError::BufferNotEmpty`, garde à l'entrée de `run_tick`, tests ajoutés (phase 9).
- **Phase 8** : le treizième système est exécuté par le dispatch récursif (journal d'exécution), pas seulement compilé.
- **Revue du document entier** : références croisées, effets structurels, graphe de dépendances, erreurs/buffer, statut des sources.
- §11 : phases indépendantes, dépendances, emplacement des correctifs de `verbum-ecs` ; **plan d'exécution par sessions** dans `01-verbum-scheduler-sessions.md` (document joint).

## Règle gouvernante et méthode
Reprise de `00-verbum-ecs.md` : un concept n'est codé que s'il est porté par un invariant adopté ou rendu nécessaire par le test de la phase. Par phase : invariant cité → test → API minimale → un commit par invariant démontré. Quatre contrôles verts à chaque commit ; Miri par l'utilisateur ; alarme « tant que j'y suis ». Exclus : `Plugin`, `Registry`, `Service`, `Factory`, `Builder`, `SystemSet`, `TypeId`, `PhaseKind`, parallélisme, `dyn Trait`, `unsafe`.

Légende : **[Invariant]** texte normatif cité · **[Conséquence]** ce que l'implémentation doit respecter · **[Choix local]** décision de conception non-invariant · **[Limite]** garantie volontairement absente, justifiée · **[Arbitrage]** question pour l'utilisateur.

## Sources et statut de vérification
Je n'ai pas accès au dépôt : tout ce qui suit a été lu dans les **copies jointes à la session**, pas dans les fichiers du dépôt. Cette provenance ne rend pas le statut normatif incertain : l'utilisateur a **confirmé** que ADR-000 est adopté (10 octobre 2026) dans le dépôt, et ADR-000 est donc une source adoptée dont les contraintes (I12 à I15) s'appliquent à la conception du noyau. Les chemins du dépôt sont ceux rapportés par `HANDOFF-Stage-1.md` §3. Les ADR 001 à 006 sont lues avec le statut qu'indiquent leurs copies.

| Statut | Documents |
|---|---|
| Lus en entier (copie) | `ROADMAP.md`, `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`, ADR-000 (adopté, confirmé par l'utilisateur), ADR-001 (copie : « Adopté. Date : 31 juillet 2026 »), ADR-002 à 006, `00-verbum-ecs.md`, `HANDOFF-Stage-0/1.md`, les trois `Cargo.toml` (workspace, `verbum-ecs`, `verbum`), `verbum/src/lib.rs`, `world.rs`, `command.rs`, `query.rs`, `values.rs`, `component_set.rs`, `resource.rs`, `lib.rs`, les six fichiers de `tests/` |
| Lus partiellement | `storage.rs` (signatures publiques, par recherche) |
| Non lus, hors sujet | `entity.rs`, `fixtures.rs` (privé aux tests de `verbum-ecs`), `README.md` |

---

## 1. Invariants normatifs et références

| # | Invariant | Source exacte | Conséquence pour l'étape 1 |
|---|---|---|---|
| I1 | Le contrat d'un système est « entièrement déductible de son interface déclarative » ; une réponse qui exige d'ouvrir l'implémentation est « proscrite » ; la représentation concrète est libre « tant qu'elle permet la déduction statique du contrat » | ADR-003, points 1 et « Ce qui n'est pas tranché » | Le type d'accès est la source unique du contrat |
| I2 | « Un système ne raisonne que sur les données explicitement fournies par son interface déclarative [...] aucun accès direct au `World` en dehors de ce qui est déclaré » | ADR-003, point 5 | Le corps d'un système ne reçoit aucun `World`, aucun stockage général |
| I3 | « Aucune Ressource n'est accessible par un système sans figurer dans son interface déclarative » | ADR-005, point 2 | Les vues de Ressources sont limitées à la catégorie déclarée |
| I4 | Mode d'accès « dans son type » ; syntaxe libre (`&T`/`&mut T` ou marqueurs `Read<T>`/`Write<T>`) | ADR-002, point 3 | `Read`, `Write`, `Produce` |
| I5 | Un système qui écrit dans le Command Buffer le déclare dans son interface ; aucune dépendance déduite entre producteurs de commandes | ADR-006, point 4 | `Commands<..>` typé ; effets hors des dépendances |
| I6 | L'entrée de la construction est un ensemble de contrats, « jamais une liste de fonctions » ; absence d'ordre = erreur de construction ; modèle immuable | ADR-004, points 1, 4, 5 | `ExecutionModel::build` ne reçoit que des contrats |
| I7 | Un système ne décide jamais de sa phase ni de son ordre ; deux systèmes de même contrat sont interchangeables | ADR-003, points 3 et 6 | La phase est portée par la place dans la liste, pas par le système |
| I8 | Toute dépendance est une relation production → consommation, intra-phase ; aucun conflit d'écriture persistante « par construction » | ADR-004, point 3 | Déduction + contrôle d'unicité |
| I9 | « Produire est libre ; résoudre est unique » ; un unique propriétaire d'écriture persistante par phase | `01-runtime-ecs.md`, section 4 | `MultipleWriters` par phase |
| I10 | Chaque instance de phase se termine par un point de validation atomique ; lecture libre de ce qui est mis à jour plus tôt dans la phase ; mutations structurelles différées | `01-runtime-ecs.md`, section 5 ; ADR-006, point 3 | `apply` en fin de phase. **Portée réalisée par l'étape 0 : le lot de commandes** que `World::apply` reçoit est prévalidé et appliqué atomiquement. Les écritures de valeurs faites par les systèmes avant l'appel ne sont pas annulées si le lot est rejeté ; les phases déjà validées ne sont pas restaurées ; **aucune atomicité de phase entière ni de tick n'est promise** (§6.1) |
| I11 | Le runtime raisonne par **catégorie** ; la durée de vie est un axe de la catégorie | `02-concepts.md`, « Conséquence — le runtime ne connaît jamais le contenu individuel » et « Modèle général de donnée » ; ADR-005, point 4 | `DataId` désigne une catégorie ; `MixedNature` global |
| I12 | Le noyau (`World`, stockages, Queries, systèmes, Command Buffer, **modèle d'exécution construit**) est sans allocation ; `no_std` sans `alloc` ; code des systèmes sans `unsafe` | ADR-000, points 1 et 4 (tableau « Instanciation actuelle ») | `verbum-schedule` : `no_std`, aucune dépendance autre que `verbum-ecs`, aucun `unsafe` |
| I13 | Les interdits de contrat (« un système ne lit que ce que son interface déclarative expose ») « ne s'encodent pas par la seule visibilité ; leur vérification relève d'ADR-002 à ADR-004 et de l'outillage associé » | ADR-000, point 3 | Les types vont plus loin que la visibilité ; le résidu relève de l'outillage (§2.4, points 1 et 3) |
| I14 | « Le mécanisme par lequel les stockages sont localisés reste celui laissé ouvert par ADR-005 » ; la résolution dynamique par métadonnées (effacement de type) est exclue | ADR-000, point 5 ; ADR-005, « Ce qui n'est pas tranché » | `Locate` statique, sans `TypeId` ni registre |
| I15 | Capacité fixée avant simulation ; dépassement = « bug de configuration, détecté au build ou par assertion en debug — jamais un cas géré silencieusement en release » | ADR-001, point 5 | Plafond de 64 catégories ; hors-limite = erreur de compilation ou `Err`, jamais silencieux |
| I16 | `World` expose des stockages ; les Queries construisent des vues ; exclut `world.query()` | ADR-001, point 3 ; ADR-002, point 1 | `split_mut` expose des stockages, jamais de Query |
| I17 | `CommandBuffer` : `Spawn`, `Destroy`, `AddComponent`, `RemoveComponent` | ADR-006, point 3 ; `02-concepts.md`, « Commande » ; `01-runtime-ecs.md`, sections 2, 3 et 5 | Écart de l'étape 0 corrigé par la phase 6bis, avant 7b (§5) |

ADR-000 laisse ouverte « la forme exacte de l'interface de franchissement de `world.execute()` » : `run_tick` est une API interne provisoire de l'étape 1 et ne préjuge pas de cette interface.

## Garanties réelles de l'étape 0 (lues dans `world.rs`, `command.rs`, `resource.rs`)
| Garantie | Source |
|---|---|
| `apply` prévalide le lot, puis l'applique ; sur erreur, structure du `World` inchangée **par cet `apply`**, Ressources comprises | `World::apply` |
| Sur erreur, buffer conservé intact ; l'appelant le vide (`clear`) ; sur succès, buffer vidé | `failed_apply_preserves_the_buffer_until_cleared`, `apply_empties_the_buffer_and_keeps_its_capacity` |
| Valeurs écrites via `component_values_mut` non annulées par un `apply` rejeté | `world_values_view_writes_values_while_structure_stays_with_apply` |
| Atomicité limitée au lot reçu par `apply` : ni les valeurs écrites avant l'appel, ni les phases déjà validées ne sont restaurées ; aucune atomicité de phase entière ni de tick | `World::apply`, `world_values_view_writes_values_while_structure_stays_with_apply` |
| `CommandBuffer` : `spawn`, `despawn`, `add_component` ; **pas de retrait** (écart corrigé par la phase 6bis) | `command.rs` |
| `component_values_mut(&mut self)` et `resources_mut(&mut self)` empruntent chacun tout le `World` | `world.rs` |
| `ValuesMut::new` est `pub(crate)` ; `World`, `fixtures`, `as_slice` non utilisables depuis un autre crate (`cfg(test)` ou `pub(crate)`) | `values.rs`, `lib.rs` |

---

## 2. API typée et identité des catégories

### 2.1 Principe (Q-A validé)
Le type de ce que le système reçoit est la source unique : il détermine à la fois l'accès effectif et le contrat. Aucun `Contract` n'est écrit à la main pour un système ; `System` n'a aucun membre `CONTRACT` implémentable.

Signatures **indicatives** (vérifiées par prototype, §10 ; pas du code livré) :

```rust
// Assemblage, écrit UNE fois par monde par l'appelant (code d'infrastructure, jamais un système)
struct MotionParts<'w> { positions: Option<ValuesMut<..>>, velocities: Option<ValuesMut<..>>,
                         accumulator: Option<&'w mut ResourceSlot<Accumulator>>, .. }
impl<'w> Locate<'w, 0> for MotionParts<'w> { type Store = ValuesMut<'w, Position, 8, 8>; fn take(&mut self) -> Option<Self::Store> { .. } }
impl<'w> Locate<'w, 2> for MotionParts<'w> { type Store = &'w mut ResourceSlot<Accumulator>; .. }
struct MotionBinding;
impl Binding for MotionBinding { type World = MotionWorld; type Buffer = MotionBuffer; type Parts<'w> = MotionParts<'w>;
                                 fn parts(world: &mut MotionWorld) -> MotionParts<'_> { let (values, resources) = world.split_mut(); .. } }

// Un système : type unité, aucun état. Le type d'accès EST l'interface déclarative.
struct ResolveMotion;
impl System<MotionBinding> for ResolveMotion {
    type Access<'w> = (Read<ACCUMULATOR, ..>, Write<POSITION, ..>);   // alias par monde : Store déduit de Locate
    fn run((accumulator, positions): Self::Access<'_>) -> Result<(), BufferFull> { .. }
}
```

### 2.2 Source d'identité d'une catégorie
- **Identité = l'entier const-générique `ID: u8` du type de vue** (`Read<ID, V>`, `Write<ID, V>`, `Produce<ID, V>`). Deux catégories distinctes sont deux types distincts, **quel que soit le type de valeur stocké** : `Read<0, ..>` et `Read<1, ..>` ne sont pas interchangeables même si les deux portent un `u32`. Aucun `TypeId`, aucun registre, aucune table dynamique (I14).
- **Représentation numérique du contrat** : `DataSet` (masque `u64`) construit par `const fn` à partir de `ID`. L'identité du type et l'entier du contrat sont **la même valeur** : ils ne peuvent pas diverger.
- **Où l'entier est attribué** : par l'appelant, dans les `impl Locate<'w, ID> for Parts`, un par catégorie et par monde. Des constantes nommées (`const POSITION: u8 = 0;`) portent le sens.
- **Association à un stockage** : `Locate<ID>` est l'unique endroit où une catégorie est reliée à un stockage ou à un emplacement de Ressource. Le type de la vue (`Store`) est **déduit** de cette implémentation, jamais redéclaré par le système.
- **Constructeurs de vues non publics** : une vue ne peut naître que de `Locate::take`, via l'infrastructure. Un système ne peut ni fabriquer une vue, ni choisir le stockage qui l'alimente. Il n'existe **pas de `bind` écrit à la main par système** : la liaison est dérivée du type d'accès.

### 2.3 Traitement des identifiants (observé sur prototype, §10)
| Cas | Traitement | Moment | Code observé |
|---|---|---|---|
| Identifiant **absent** (aucun `Locate<ID>` pour ce monde) | borne `Parts: Locate<ID>` non satisfaite | compilation | E0277 |
| Identifiant **dupliqué** pour deux stockages (deux `impl Locate<ID>` pour le même `Parts`) | cohérence des implémentations de Rust : une seule implémentation par `(Parts, ID)` | compilation | E0119 |
| Même `ID` déclaré **deux fois dans un seul** type d'accès | `merge_disjoint` (`const fn`) panique pendant l'évaluation de `CONTRACT` | compilation | E0080 |
| `ID ≥ 64` (hors capacité) | `DataSet::of` panique en évaluation const (I15) ; en contexte non const, `DataId::new` rend `Err(DataIdOutOfRange)` | compilation / `Err` | E0080 |
| Type de valeur **incohérent** (le système déclare un `Store` différent de celui de `Locate`) | le type de la vue doit égaler `<Parts as Locate<ID>>::Store` : l'`impl Bind` ne s'applique plus | compilation | E0277 |
| Doublon d'effets (deux `Commands` dans un accès) | même mécanisme que ci-dessus | compilation | E0080 |
| Emplacement déjà pris au runtime (accès construit hors règle, `parts()` défectueux) | `Option::take` → `None` → `TickError::Binding`, jamais une panique silencieuse | exécution | — |

La panique d'une `const fn` n'est présentée comme erreur de compilation **que** lorsqu'elle est évaluée en contexte const (cas de `Part::CONTRACT`, `const` associé), ce que le prototype démontre. Hors contexte const, c'est un `Result`.

### 2.4 Ce que les types garantissent, et ce qu'ils ne garantissent pas
**Garanti (accès aux données du `World` exposées par l'interface)** :
- le corps de `run` n'a ni `World`, ni `&mut World`, ni stockage général, ni autre Ressource que celles déclarées (le tuple d'accès n'accepte que des `Part` scellés) ;
- un système ne peut pas élargir un accès : `Read` n'expose qu'un emprunt partagé, `Write`/`Produce` ne se fabriquent pas depuis un `Read` ;
- un système ne peut pas choisir le stockage derrière une catégorie (liaison par `Locate`) ;
- ni deux catégories distinctes confondues par un système ou par le Scheduler, ni deux accès à la même catégorie dans un contrat (compilation).

**Non garanti** :
1. **Étiquetage sémantique dans `Locate::take`.** Le compilateur garantit l'unicité, l'absence et la cohérence de type des identifiants ; il ne peut pas savoir que la catégorie nommée `POSITION` doit renvoyer le champ `positions`. Une erreur d'étiquetage est une erreur de **définition** d'une catégorie, localisée dans la définition de la liaison (`impl Locate` du monde, un point par catégorie et par monde) ; elle ne permet pas à un système de confondre deux catégories qu'il distinguerait. Elle n'est détectée par les types que si les types de valeur diffèrent (cas des composants : un stockage par type, ADR-001, point 1). Pour deux Ressources de même type, elle ne l'est pas. Cette responsabilité est donc **portée par la définition de la liaison et couverte par des tests de câblage** (un test par liaison écrite ; modèle : test des deux Ressources de même type, phase 7a). Ces tests vérifient le câblage effectivement écrit ; ils ne prouvent pas que le compilateur déduit la signification métier des identifiants. **[Limite]**
2. **Composition d'un `Produce`.** Le type n'interdit pas à un producteur de décider d'après la valeur accumulée (agrégation sans arbitrage, `02-concepts.md`, « Composition »). **[Limite]**
3. **Effets externes cachés** (état global par `static` à mutabilité intérieure, atomiques). Ce n'est pas un accès aux données du `World` : les types ne l'empêchent pas, et il ne faut pas confondre les deux. `unsafe_code = "deny"` exclut `static mut` ; ADR-000, point 3, renvoie le reste à « l'outillage associé » (I13). **[Limite]**
4. **Vivacité d'entité** : voir §4.5.

### 2.5 Abstractions supplémentaires exposées (à valider)
La liaison dérivée exige trois éléments que la révision 3 ne portait pas. Aucun n'est un registre ni ne dépend d'un `TypeId` :

| Élément | Rôle | Pourquoi nécessaire | Alternative écartée |
|---|---|---|---|
| `Locate<'w, ID>` (trait, impl par l'appelant) | associe une catégorie à un stockage ou emplacement, une fois par monde | I14 ; seul moyen de garantir unicité (E0119) et absence (E0277) à la compilation | `bind` écrit par système avec vues « étiquetées » : plus léger, mais l'unicité des étiquettes exige un contrôle const supplémentaire et chaque système répète le câblage |
| `Binding` (trait, impl par l'appelant) | fournit `Parts<'w>` à partir du `World` ; seul code qui voit le monde entier | règle d'orphelin : on ne peut pas implémenter un trait de la bibliothèque pour `World<..>` depuis l'appelant ; un type local est requis | — |
| `Bind` (trait **scellé**) | construit l'accès depuis `Parts` par `Locate::take`, y compris les tuples | supprime le `bind` par système | — |

**[Arbitrage A1 — validé]** Ces trois éléments sont validés, avec la liaison définie **au niveau du monde et de ses catégories**, pas répétée par système. Séparation retenue : `ExecutionModel` ne reçoit que des contrats ; `Locate` associe un identifiant de catégorie à un emplacement concret ; `Binding` construit les parties depuis le `World` ; `Bind` construit les vues demandées par le système ; le corps du système ne reçoit que son interface typée. Le compilateur garantit cohérence des types, absence d'identifiant et unicité de `Locate<ID>` ; il ne garantit pas le sens métier donné à chaque emplacement (§2.4, point 1).

### 2.6 Tests de compilation (garantie ↔ test)
Doctests `compile_fail` épinglés (codes indicatifs observés sur prototype ; les doctests réels fixent les codes) :

| Démontre | Test | Code |
|---|---|---|
| Pas d'accès à une catégorie non déclarée | accès sans `Locate` pour l'`ID` | E0277 |
| Pas de contournement par le `World` | `(Read<..>, &mut World)` n'est pas un `Part` | E0277 |
| Pas de capacité plus large | `Read::shared().insert(..)` / affectation | E0596 / E0594 |
| Pas de conversion `Read` → `Write` | affectation d'une `Read` à un `Write` | E0308 |
| Pas de vue fabriquée par un système | constructeur de vue non public | E0423 |
| `Part` non implémentable hors crate | `impl Part for Mine` | E0277 |
| Contrat non implémentable | `impl System { const CONTRACT }` | E0438 |
| Effets déclarés = effets disponibles | `Commands<.., false, ..>::spawn` ; `remove_component` sans `REMOVE` (après 6bis) | E0599 |
| Identifiant dupliqué | deux `impl Locate<ID>` | E0119 |
| Catégorie déclarée deux fois dans un accès | `contract_of` en `const` | E0080 |
| Identifiant hors limites | `ID = 64` en `const` | E0080 |
| Type de valeur incohérent | `Store` déclaré ≠ `Locate::Store` | E0277 |

---

## 3. Dispatch statique (Q-B : S2, sans plafond de 12)

- **[Invariant]** `00-principes.md`, « Modèles écartés » : « Interfaces / vtable [...] Bloque la prédiction de branchement et l'inlining ; remplacé par des tagged unions (`enum` Rust) résolues au compile-time ». ADR-002, point 2 ne vise que les Queries. `01-runtime-ecs.md` section 5 : ordre « fixe, connu à la compilation (ou figé au démarrage) ».
- **Retenu (Q-B)** : liste typée de systèmes, modèle construit des seuls contrats, dispatch statique. Pas de table de pointeurs de fonction, pas de `dyn`.
- **Liste récursive** : `Nil` et `Cons<At<PHASE, S>, Tail>`. Aucune macro d'arité, aucun plafond d'architecture, aucune allocation. La liste fournit à la fois `L::LEN`, `L::fill` (contrats et phases, par position) et `L::run(index, ..)` (récursion : `index == 0` appelle ce système, sinon `Tail::run(index - 1)`).
- **Séparation (I6, I7)** : `ExecutionModel::build` reçoit `[(phase, Contract); N]` : des entiers et des contrats. L'identité des systèmes et leurs fonctions n'y entrent jamais ; la position n'est qu'un indice de correspondance. L'extraction des contrats (`contract_of::<S>()`) se fait en amont, dans la couche Scheduler.
- **Limites techniques inévitables, à assumer comme choix locaux** :
  1. Limite de récursion de résolution de traits du compilateur (`recursion_limit`, 128 par défaut) : relevable par attribut dans le crate appelant ; ce n'est pas une limite du moteur.
  2. Coût de la sélection par indice : O(position) comparaisons par système, soit O(N²) par tick au pire, sans inlining garanti. **Non mesuré**, négligeable pour de petits N, à mesurer avant toute optimisation. Le mécanisme est confiné derrière `SystemList` : révisable sans toucher aux contrats.
- **Cohérence `SYSTEMS` / `LEN`** : un bloc `const` impose `L::LEN == SYSTEMS` (erreur de compilation sinon).
- **Source unique** : la même liste donne contrats et exécution ; ils ne peuvent pas diverger.

---

## 4. Extension minimale de `verbum-ecs` (Q-C)

### 4.1 Pourquoi
La Roadmap (étape 1, test 2) prévoit un accumulateur transitoire **en Ressource**. Aujourd'hui, `component_values_mut(&mut self)` et `resources_mut(&mut self)` empruntent chacun tout le `World` : un système ne peut pas tenir une vue de composants et une Ressource ensemble. Le résolveur (`Read<accumulateur>` + `Write<Position>`) l'exige. Blocage exact : deux emprunts exclusifs de `&mut self` entier.

### 4.2 Surface d'API minimale
Une seule méthode, **additive** :

```rust
impl<const ENTITIES: usize, S: ComponentSet, R: Default> World<ENTITIES, S, R> {
    /// Accès simultané aux valeurs des composants et aux Ressources : deux champs disjoints.
    pub fn split_mut(&mut self) -> (S::Values<'_>, &mut R)
}
```
- Deux champs disjoints de `World` (`components`, `resources`) : emprunts disjoints vérifiés par le compilateur, **aucun `unsafe`**.
- **N'expose jamais** `&mut World`, `&mut S` (`S::Values` reste composé de `ValuesMut`, `ValuesView` scellé), ni la table d'identité. `&mut R` n'est pas plus large que ce qu'offre déjà `resources_mut`.
- Aucun registre de Ressources, aucune abstraction générale : `R` reste la structure définie par l'appelant (ADR-005, clé = position statique). Le code d'assemblage de l'appelant la décompose par motif (`let Globals { acc, .. } = resources;`), ce qui donne des emprunts disjoints par champ.
- Compatible avec I16 : le `World` expose des stockages, sans Query.

### 4.3 Restriction des vues de Ressources à la catégorie déclarée (I3)
Chaque emplacement (`&mut ResourceSlot<T>`) est rangé dans son propre `Option` de `Parts` et ne peut être pris que par `Locate::take` pour son `ID`. Une vue ne détient donc **qu'un** emplacement ; les champs frères de `R` lui sont inaccessibles.
- `Read` : n'expose qu'un `&ResourceSlot<T>` (`get`), aucune écriture.
- `Write`/`Produce` : rendent `&mut ResourceSlot<T>` (`get`, `get_mut`, `insert`). `insert` crée ou remplace la valeur ; il ne crée aucune entité et n'est pas structurel.

### 4.4 Tests et conséquences sur l'étape 0
Tests ajoutés dans `verbum-ecs` (style de l'étape 0) :
1. Valeurs de composants et Ressource modifiables **ensemble** dans une même portée ; les deux effets sont visibles après la portée.
2. Structure inchangée : la vue ne permet ni `insert` ni `remove` (déjà épinglé pour `ValuesMut`) ; les entités vivantes et générations sont inchangées.
3. `compile_fail` : `world.spawn()` (ou tout autre emprunt de `world`) pendant que les deux emprunts sont vivants → E0499.
4. `compile_fail` : le premier élément du résultat ne se laisse pas affecter à `&mut S`.
5. Le résultat est identique à celui de `component_values_mut` + `resources_mut` pris séparément.

Conséquences : **aucun test existant n'est modifié** (méthode additive, aucune signature changée). Les décomptes de l'étape 0 augmentent (tests ajoutés ; à reporter dans `HANDOFF`/`00-verbum-ecs.md` à ce moment). Miri sur `verbum-ecs` à relancer par l'utilisateur ; aucun `unsafe` ajouté.

**[Arbitrage A2 — autorisé]** Extension additive et minimale, en **commit correctif distinct** (phase 6), avec tests de compilation, quatre contrôles et Miri à l'initiative de l'utilisateur. La signature exacte `split_mut(&mut self) -> (S::Values<'_>, &mut R)` est **sous réserve de validation à l'implémentation**. L'autorisation ne vaut pas réouverture générale de l'étape 0. Les phases de construction du modèle (0 à 5) n'en dépendent pas ; l'accès conjoint aux composants et aux Ressources n'entre dans le Scheduler (phase 7a) qu'une fois cette extension validée.

### 4.5 Vivacité des entités (`is_alive`)
- **[Invariant]** `01-runtime-ecs.md`, section 2 : « une entité référencée peut avoir disparu » ; sa validité « se vérifie systématiquement via `(index, génération)` ». Le corpus ne dit pas **qui** la vérifie ni par quelle interface.
- Les deux tests de la Roadmap, les tests de la phase 9 et l'exemple headless n'en ont pas besoin. Les effets structurels s'observent par la présence de composants (un système) et par `world.is_alive` (le harnais de test, qui possède le `World`).
- **Conclusion** : non exigé à l'étape 1. **[Limite]** Un système ne peut pas valider un handle ; il ne peut donc pas porter de relation entre entités (`01-runtime-ecs.md`, section 3). Si un besoin apparaît, la forme conforme à I2 serait une catégorie déclarée en lecture seule sur la table d'identité, par le même mécanisme `Locate` ; elle n'est ni spécifiée ni implémentée ici. Aucun accès général au `World` n'est réintroduit.

---

## 5. Effets structurels et `RemoveComponent` (Q-D, A3)

| Catégorie normative | ADR-006, point 3 | `CommandBuffer` (étape 0, lu) | Contrat exécutable de l'étape 1 |
|---|---|---|---|
| Création | `Spawn` | oui | `spawn` |
| Destruction | `Destroy` | oui (`despawn`) | `despawn` |
| Ajout de composant | `AddComponent` | oui | `add_component` |
| Retrait de composant | `RemoveComponent` | **absent** | **exécutable seulement après la phase 6bis** |

- **Vocabulaire** : les quatre catégories sont reconnues (I17). `StructuralEffects` porte quatre indicateurs (donnée pure du contrat, phase 1). `emits_commands()` est un accesseur dérivé.
- **Capacités exécutables** : un indicateur n'est posé que par la poignée `Commands` (phase 7b). Cette poignée n'expose `remove_component` que si la phase 6bis est achevée : **7b dépend de 6bis**. Aucune capacité inutilisable n'est donc exposée, et la phase 7b ne prétend pas à la conformité structurelle complète tant que 6bis n'est pas faite.
- **Écart de conformité** : l'absence de `RemoveComponent` est un écart de **l'étape 0** vis-à-vis d'ADR-006, point 3, de `02-concepts.md` (« Commande ») et de `01-runtime-ecs.md` (sections 2, 3 et 5, qui nomment `remove_component` parmi les primitives). Aucun test d'invariant de la Roadmap ne l'exige ; il est corrigé quand même (A3), sans élargir le reste du Scheduler.
- **Décision A3** : correctif de `verbum-ecs` en commit distinct (phase 6bis), avant 7b et avant l'acceptance finale ; ne bloque pas les phases 0 à 5.

### 5.1 Phase corrective 6bis — `RemoveComponent` dans `verbum-ecs`
**Invariants** : I17 ; ADR-006, point 3 (mutations structurelles différées, appliquées par le `World` seul) ; ADR-001, points 3 et 4 (isolation des stockages, swap-remove). Périmètre : `component_set.rs`, `world.rs`, `command.rs`, leurs tests et doctests ; **rien d'autre**.

**Difficulté structurelle exposée.** Un retrait doit désigner *quel* composant retirer, comme `Add` le fait par variante. Le trait public `ComponentSet` n'a aujourd'hui que `Add`, `kind(&Add)` et `insert` : il faut y ajouter un pendant pour le retrait. Ce n'est donc **pas purement additif** pour les implémenteurs de `ComponentSet` (au moins les deux doctests de `component_set.rs` et `fixtures.rs` ; recensement exact à l'implémentation) : ils devront fournir le nouveau type et la nouvelle méthode. `SparseSet::remove` (déjà présent et testé) ne change pas. L'extension est limitée au strict nécessaire ci-dessous et soumise à votre revue du commit (point B2).

#### 5.1.1 Surface d'API minimale (signatures indicatives, à confirmer)
| Élément | Ajout | Remarque |
|---|---|---|
| `ComponentSet` | `type Remove: Copy;` ; `fn remove_kind(&Self::Remove) -> usize` ; `fn remove(&mut self, entity: EntityId, remove: Self::Remove) -> bool` | une variante de `Remove` par stockage ; `Infallible` pour `()` et pour les ensembles sans retrait ; `remove` délègue à `SparseSet::remove` (`false` si absent) |
| `World` | `remove_component(&mut self, id: EntityId, remove: S::Remove) -> Result<bool, StaleEntity>` | primitive citée par `01-runtime-ecs.md` ; symétrique de `add_component` ; `Ok(false)` : composant absent |
| `Command` (interne) | variante `Remove(EntityId, R)` | `Command<A, R>` |
| `CommandBuffer` | `remove_component(entity, remove) -> Result<(), BufferFull>` ; paramètres `CommandBuffer<COMMANDS, A = Infallible, Rm = Infallible>` | le défaut garde valide le code qui nomme `CommandBuffer<N, A>` lorsque `Remove = Infallible` |
| `World::apply` | accepte `CommandBuffer<COMMANDS, S::Add, S::Remove>` | aucune nouvelle variante de `ApplyError` |

#### 5.1.2 Règles de prévalidation (`plan`)
`plan` parcourt le lot **dans l'ordre d'émission**. Chaque commande est évaluée contre un état simulé qui ne reflète que les commandes **précédentes** déjà jugées effectives ; la première commande inapplicable met fin à la prévalidation avec son erreur. **Une commande ultérieure ne peut jamais rendre valide rétroactivement une commande précédente** : en particulier un `Remove` placé après un `Add` invalide ne le répare pas.

État simulé, par entité et par stockage (`kind`) : présence = présence dans le stockage avant le lot, puis modifiée dans l'ordre par chaque commande précédente effective (`Add` effectif → présent ; `Remove` effectif → absent ; `Despawn` effectif → absent pour tous les stockages). Longueur dense simulée par stockage, comme pour `Add` aujourd'hui.

| Commande | Condition d'effet | Effet simulé | Erreur éventuelle |
|---|---|---|---|
| `Add(id, a)` (inchangé, mais la présence est celle de l'état simulé) | entité vivante à ce point | présent ; longueur dense + 1 | `IndexOutOfRange`, `AlreadyPresent` (si présent dans l'état simulé), `DenseFull` (si longueur simulée = capacité), dans l'ordre de `SparseSet::insert` |
| `Remove(id, r)` | entité vivante à ce point **et** composant présent dans l'état simulé | absent ; longueur dense − 1 | **aucune** |
| `Remove(id, r)` autrement (handle périmé, détruit plus tôt dans le lot, composant absent, index hors étendue) | — | aucun effet | aucune |
| `Despawn` | inchangé ; décrémente la longueur de chaque stockage où l'entité est présente **dans l'état simulé** | | |

Cas explicites :
- **Composant absent** : `Remove` sans effet, `apply` renvoie `Ok`, rien n'est modifié.
- **Entité périmée ou déjà détruite dans le lot** : `Remove` sans effet (même règle que `Add` et `Despawn` aujourd'hui).
- **`Add` puis `Remove`** (même entité, même stockage) : si l'`Add` est valide, il est effectif puis le `Remove` l'est aussi : composant absent à la fin. Si l'`Add` est invalide (doublon, stockage plein, hors étendue) : **le lot est rejeté à l'`Add`**, le `Remove` n'est pas évalué et ne le rend pas valide.
- **`Remove` puis `Add`** : si le composant était présent, le `Remove` est effectif, puis l'`Add` est évalué contre l'état « absent » : valide, composant présent avec la valeur ajoutée. Si le composant était absent, le `Remove` est sans effet, puis l'`Add` est évalué normalement.
- **`Remove` puis `Add` sur un stockage dense plein** : le `Remove` effectif libère une place simulée ; l'`Add` suivant passe `DenseFull`. L'inverse (`Add` puis `Remove`) sur un stockage plein : `Add` rejeté pour `DenseFull`, le `Remove` ne le sauve pas.
- **`Remove` puis `Remove`** : le second est sans effet (absent dans l'état simulé).
- **`Remove` puis `Despawn`** : le `Remove` est effectif ; le `Despawn` ne décrémente pas ce stockage (absent dans l'état simulé).
- **Lot rejeté** : atomicité de `apply` inchangée : aucun `Remove` valide précédent n'est appliqué, le `World` est intact, le buffer est conservé.

« Dernière commande effective gagnante » n'est donc **pas** la règle : elle ne décrit pas les cas de rejet. La règle est la simulation séquentielle ci-dessus ; l'application n'exécute que les commandes marquées effectives (`applies`), dans l'ordre, comme aujourd'hui.

**Comportement lorsque le composant demandé est absent** : le corpus ne le fixe pas (ADR-006 liste les commandes sans préciser leurs cas d'échec ; `SparseSet::remove` d'une entrée absente est déjà un no-op renvoyant `false`, test `remove_of_absent_entry_is_a_no_op`). **[Choix local, non normatif]** : sans effet, pas une erreur, le lot n'est pas rejeté. Raison : plusieurs producteurs peuvent viser la même entité (ADR-006, point 4) et l'état visé, « composant absent », est déjà atteint ; contrairement à l'`Add` en doublon, aucune valeur n'est perdue. Alternative écartée : `ApplyError::Remove(Absent)`, qui ferait échouer un lot entier sur une intention déjà satisfaite.

#### 5.1.3 Tests (écrits avant le code, style de l'étape 0)
1. Retrait d'un composant présent : disparaît ; valeurs des autres entités intactes ; swap-remove cohérent (l'élément de fin comble la position libérée, index sparse à jour, préfixe dense = `entities()`).
2. Isolation : les autres stockages de la même entité ne sont pas touchés (ADR-001, point 3) ; la position libérée est réutilisable.
3. Composant absent : `apply` = `Ok`, aucun effet ; idem index hors étendue sparse.
4. Handle périmé ; entité détruite plus tôt dans le lot ; `Remove` puis `Despawn` : sans effet / pas de double décrément.
5. `Add` valide puis `Remove` → absent ; `Remove` puis `Add` → présent avec la valeur ajoutée ; composant absent : `Remove` (sans effet) puis `Add` → présent ; `Remove` puis `Remove` → un seul effet.
6. Stockage dense plein : `Remove` puis `Add` → lot accepté ; `Add` puis `Remove` → rejeté `DenseFull`, **aucune commande appliquée**.
7. **Pas de validation rétroactive** : lot `[Add(doublon), Remove(même composant)]` → `Err(Add(AlreadyPresent))`, composant toujours présent ; lot `[Add(hors étendue), Remove(..)]` → `Err(Add(IndexOutOfRange))`.
8. **Rejet atomique** : lot `[Remove(valide), Add(doublon d'une autre entité)]` → `Err`, le retrait n'est pas appliqué, `World` intact, buffer conservé intact, vidé seulement par `clear`.
9. `CommandBuffer::remove_component` : `Err(BufferFull)` au débordement, sans croissance ; `len` cohérent ; `apply` réussi vide le buffer.
10. `ValuesMut` ne permet toujours aucun retrait (`compile_fail` existant conservé).
11. **Non-régression** : les 64 tests unitaires, 11 tests d'intégration et 5 doctests de l'étape 0 passent, après la seule adaptation mécanique des implémenteurs de `ComponentSet` (décomptes à actualiser dans `HANDOFF`/`00-verbum-ecs.md`).

**Commit** : `world: RemoveComponent — retrait de composant différé par le command buffer`. Quatre contrôles verts ; Miri par l'utilisateur ; aucun `unsafe` ajouté.

---

## 6. Arbitrages conservés

- **A — atomicité et arrêt du tick** : voir §6.1 et §6.2.
- **Q3 — `MixedNature` global.** Lecture vérifiée (I11) : `DataId` désigne la catégorie ; la durée de vie est un axe de la catégorie. Aucune phrase ne l'énonce en règle unique : c'est une **conséquence** de `02-concepts.md` (tableau des axes) et d'ADR-005, point 4. `MixedNature` : une catégorie n'est pas à la fois dans des `writes` et des `produces` **où que ce soit dans le modèle**. `MultipleWriters` : **par phase** seulement (I9). Si la lecture est contestée, le contrôle se réduit à la phase.
- **Q5 — transitoire.** Le Scheduler ne remet rien à zéro. Un système explicite `ClearAccumulator`, dans une phase antérieure à celle des producteurs, **initialise ou remet à zéro** la Ressource accumulateur (`Produce` ; `insert` d'une valeur initiale). Aucune généralisation aux autres transitoires. Test sur deux ticks avec témoin négatif (phase 10a).

### 6.1 Portée exacte de l'atomicité
**[Invariant]** `01-runtime-ecs.md`, section 5 ; ADR-006, point 3 : un point de validation en fin de phase. Ce que l'étape 0 réalise, et que l'étape 1 promet **sans rien ajouter** :
1. `World::apply` prévalide et applique **atomiquement le lot de commandes qu'il reçoit**.
2. Les mutations de **valeurs** faites directement par les systèmes avant cet appel ne sont **pas annulées** si le lot est rejeté.
3. Les phases déjà validées ne sont **pas restaurées**.
4. **Aucune atomicité globale de phase ou de tick** n'est promise ; aucun rollback.

Démonstration : le test d'échec d'`apply` en trois phases (phase 9, test 6).

### 6.2 Contrat d'erreur du tick et état du buffer
Le buffer appartient à l'appelant et se passe à `run_tick` (comme à `apply`). Politique retenue :
- aucun rollback des mutations de valeurs déjà réalisées ;
- aucune application du lot structurel d'une phase qui n'a pas atteint son point de validation ;
- le buffer **conserve** les commandes déjà acceptées, jusqu'à son effacement **explicite** (`clear`) par l'appelant ;
- après **toute** erreur de tick, l'appelant vide le buffer avant de réessayer ;
- **garde d'entrée** : si le buffer n'est pas vide, `run_tick` rend `Err(TickError::BufferNotEmpty)` **avant toute exécution** (aucun système lancé, aucun état modifié). Cela empêche qu'une erreur d'usage applique des commandes obsolètes au tick suivant. Aucun mécanisme général de récupération n'est ajouté.

| Échec | Moment | Valeurs déjà écrites | Buffer | Phases suivantes | Erreur |
|---|---|---|---|---|---|
| `BufferFull` pendant un système | dans le corps d'un système, avant le point de validation | conservées (systèmes précédents et système fautif jusqu'à l'échec) | commandes acceptées conservées, **jamais appliquées** | non exécutées ; `apply` non appelé | `TickError::Buffer(BufferFull)` |
| Défaut de liaison | avant le corps du système concerné, après les systèmes précédents de la phase | conservées | commandes déjà émises conservées, jamais appliquées | non exécutées ; `apply` non appelé | `TickError::Binding` |
| Échec d'`apply` | point de validation de la phase | conservées ; phases précédentes conservées | lot **conservé intact** par `apply` (déjà garanti) | non exécutées | `TickError::Apply(ApplyError)` |
| Buffer non vide à l'entrée | avant tout système | inchangées | inchangé | aucune phase | `TickError::BufferNotEmpty` |

Un tick réussi laisse le buffer **vide** (`apply` le vide à chaque phase, y compris phase sans commande). Après une erreur, l'état de la simulation est partiel (aucun rollback) ; la reprise et sa politique relèvent de l'appelant et ne sont pas spécifiées à l'étape 1.

---

## 7. Décisions D1–D10 et frontières

| # | Décision | Statut |
|---|---|---|
| D1 | Crate `verbum-schedule`, `#![no_std]`, sans `alloc`, sans `unsafe`, dépendance unique : `verbum-ecs` (I12). Membre du workspace ; `Cargo.toml` calqué sur `verbum-ecs` (héritage workspace, `[lints] workspace = true`, édition 2024, `rust-version = 1.85`) | **[Choix local]** |
| D2 | `DataSet` masque `u64` ; plafond 64 catégories ; `DataId::new(u8) -> Result` hors contexte const ; `ID` const-générique `u8` dans les vues ; hors-limite en const = E0080 | **[Choix local]** ; I15 |
| D3 | `Contract { reads, writes, produces, effects }` ; `effects` = quatre indicateurs, vocabulaire d'ADR-006 point 3 (§5) ; jamais écrit à la main par un système ; l'indicateur de retrait n'est posable qu'après la phase 6bis | **[Conséquence]** I1 |
| D4 | `MixedNature` (global), `MultipleWriters` (par phase), `UnknownPhase`, `Cycle` ; ordre de contrôle dans cet ordre | **[Choix local]** pour l'ordre |
| D5 | Phase portée par `At<PHASE, S>` ; pas de `PhaseKind` ; `apply` à la fin de chaque phase (atomicité du lot seulement, §6.1) | **[Conséquence]** I7, I10 |
| D6 | Accès typé (§2) ; `Locate`, `Binding`, `Bind` | **[Arbitrage A1]** |
| D7 | Liste récursive, dispatch statique (§3) | **[Choix local]**, limites assumées |
| D8 | `examples/headless.rs` dans `verbum-schedule` : plusieurs ticks, aucune sortie, code retour 0 ; ce n'est pas le Shell (étape 3) | **[Choix local]** |
| D9 | `run -> Result<(), BufferFull>` ; `TickError::{BufferNotEmpty, Binding, Buffer(BufferFull), Apply(ApplyError)}` ; contrat §6.2 | **[Choix local]** ; I15 |
| D10 | `Write` et `Produce` n'ajoutent pas `reads` au contrat (sinon deux producteurs d'une même catégorie formeraient un cycle) | **[Choix local]** |

### Frontières internes de `verbum-schedule`
| Module | Responsabilité | Dépend de | Ne doit jamais dépendre de |
|---|---|---|---|
| `data` | `DataSet`, `DataId` | rien | tout le reste |
| `contract` | `Contract`, effets | `data` | `verbum-ecs`, `access`, `system` |
| `order` | dépendances, contrôles, tri d'une phase | `contract` | `verbum-ecs`, `access`, `system` |
| `model` | `ExecutionModel` immuable | `order`, `contract` | `verbum-ecs`, `access`, `system` |
| `access` | `Locate`, `Bind`, `Read`/`Write`/`Produce`, `Commands`, `Part` scellé | `contract`, `verbum-ecs` | `order`, `model`, `system` |
| `system` | `Binding`, `System`, `At`, `Nil`/`Cons`, `contract_of` | `access`, `contract` | `order`, `model` |
| `scheduler` | `Scheduler`, `run_tick`, erreurs | `model`, `system`, `verbum-ecs` | — |

Contrôle en revue : aucune occurrence de `verbum_ecs`, `System`, `Read`, `Write`, `At` dans `data`, `contract`, `order`, `model`.

### Hors périmètre de l'étape 1
Uniquement des éléments qu'aucun invariant ni test de l'étape 1 n'exige : `PhaseKind` ; parallélisme ; génération de code pour la liaison (O4) ; diagnostic détaillé des cycles ; construction `const` du modèle ; Shell et horloge (étape 3). Les vues de Ressources, `split_mut` et `RemoveComponent` **ne figurent pas** dans cette liste : ils sont requis (acceptance, ADR-006 point 3). La vivacité d'entité côté système reste hors périmètre pour la raison donnée au §4.5.

---

## 8. Micro-phases

### Phase 0 — Crate et workspace
Ajout de `crates/schedule` aux `members`. **Test** : `cargo test --workspace` inchangé. **Commit** : `workspace: crate verbum-schedule vide`.

### Phase 1 — Données et contrat
**Invariant** : I1, I15. **Tests** : `DataId::new(63)` → `Ok`, `new(64)` → `Err` ; bits distincts ; `DataSet` (`with`, `contains`, `intersects`, union) ; `Contract` répond par accesseurs aux cinq questions d'ADR-003 ; `emits_commands()` dérivé ; doctest `compile_fail` : `DataSet::of(64)` en `const` (E0080).
**Commit** : `contract: interface déclarative inspectable, sans fonction de système`.

### Phase 2 — Dépendance déduite
**Invariant** : I8, I5. **Tests** : `a.precedes(b)` ssi `(a.writes ∪ a.produces) ∩ b.reads ≠ ∅` ; écriture seule, lecture seule, deux lectures, deux producteurs : aucune arête ; `effects` n'influence jamais.
**Commit** : `order: dépendance déduite production → consommation`.

### Phase 3 — Ordre d'une phase, cycle = erreur de construction
**Invariant** : I6. **Tests** : chaîne déclarée à l'envers ; sans contrainte, ordre de déclaration ; constructions identiques ; cycles à deux et à trois → `Cycle` ; **exhaustif** 3 systèmes, 2 catégories, `reads × produces` (16³) contre un oracle par permutations. À mesurer sous Miri (réduction documentée si nécessaire, comme à l'étape 0).
**Commit** : `order: tri déterministe d'une phase, cycle rejeté à la construction`.

### Phase 4 — Écrivain unique et nature cohérente
**Invariant** : I9, I11. **Tests** : trois producteurs, un résolveur, un lecteur ; nombre quelconque de producteurs accepté ; deux `writes` d'une catégorie dans une phase → `MultipleWriters` ; dans deux phases → accepté ; `writes` ici et `produces` ailleurs, **y compris dans une autre phase** → `MixedNature` ; deux producteurs `reads`+`produces` du même X → `Cycle` ; ordre de contrôle D4.
**Commit** : `order: écrivain unique par phase, nature de catégorie cohérente`.

### Phase 5 — Programme d'exécution immuable
**Invariant** : I6, I10. **Tests** : `build([(phase, Contract); N])` ; `phase(p)` rend des indices dans l'ordre ; phase vide ; `UnknownPhase` ; contrats cycliques répartis sur deux phases acceptés ; aucune API `&mut` ; deux listes de contrats égaux → modèles `==`.
**Commit** : `model: programme d'exécution immuable, construit depuis les seuls contrats`.

### Phase 6 — Extension additive de `verbum-ecs` : `split_mut` (A2 autorisé)
**Invariant** : I3, I16. **Tests** : §4.4. **Commit correctif distinct** : `world: accès disjoint aux valeurs de composants et aux Ressources`. Quatre contrôles verts ; Miri par l'utilisateur. Signature exacte validée à l'implémentation.

### Phase 6bis — Correctif `RemoveComponent` dans `verbum-ecs` (A3)
Contenu, comportement du composant absent et tests : §5.1. **Placement** : avant 7b, avant 8, avant l'acceptance ; indépendante des phases 0 à 5 et de la phase 6 (même fichier `world.rs` : à enchaîner, ne pas fusionner dans le même commit). **Commit correctif distinct** : `world: RemoveComponent — retrait de composant différé par le command buffer`.

### Phase 7 — Accès typé
**7a — vues et liaison.** **Invariants** : I1 à I4, I14. **Tests** (`tests/access.rs`, jeu de composants et de Ressources défini dans `tests/support`, modèle : `acceptance.rs`) : liaison d'un accès à deux composants et une Ressource depuis un vrai `World` via `split_mut` ; `Read` lit, `Write` et `Produce` écrivent, valeurs visibles dans le `World` ; `ReadWriteQuery` construite depuis `Read` et `Write` ; `contract_of` évalué en `const` donne `reads`/`writes`/`produces` attendus, y compris pour un accès imbriqué ; `Write` et `Produce` sans `reads` (D10) ; emplacement déjà pris → `Binding` (jamais une panique) ; **tous les `compile_fail` de §2.6** hors effets ; **câblage de deux Ressources de même type concret** (`Counter`), associées aux identifiants 3 et 4 par deux `impl Locate` : une écriture via `Write<3>` ne modifie pas la valeur lue via `Read<4>`, et inversement ; chaque Ressource est lue et modifiée indépendamment, valeurs visibles dans les bons champs du `World`. Ce test vérifie **le câblage effectivement écrit** ; il ne prouve pas que le compilateur déduit le sens des identifiants. **Témoin documentaire** : une liaison volontairement croisée (3 → champ de 4) compile et se comporte de façon croisée, ce qui confirme que seul le test de câblage protège de cette erreur (§2.4, point 1).
**API** : `Locate`, `Bind` (scellé), `Read`, `Write`, `Produce`, `Part` (scellé). **Commit** : `access: vues déclaratives, identité par catégorie, contrat calculé sur le type`.
**7b — commandes. Dépend de 6bis.** **Invariants** : I5, I17. **Tests** : `Commands<.., true, false, false, false>` émet `spawn` ; effets du contrat = indicateurs du type, y compris le retrait ; `compile_fail` `despawn` sans `DESPAWN` et `remove_component` sans `REMOVE` (E0599) ; débordement → `Err(BufferFull)` sans croissance ; une commande de retrait émise par `Commands<.., REMOVE = true>` arrive dans le buffer de `verbum-ecs`. **Commit** : `access: poignée de commandes typée par effets déclarés`.

### Phase 8 — Systèmes, liste, construction du Scheduler
**Invariants** : I6, I7, I1. **Tests** : `contract_of::<S>()` = fusion des parties ; `compile_fail` `const CONTRACT` (E0438) ; liste de 1, 2, puis **13 systèmes** (au-delà de l'ancien plafond, aucune erreur de compilation ni limite d'API) ; **exécution du dispatch récursif sur 13 systèmes** (voir ci-dessous) ; `Scheduler::new()` extrait les contrats, construit le modèle, rejette `Cycle`, `MultipleWriters`, `MixedNature` ; ordre correct pour une liste déclarée à l'envers ; deux listes de systèmes distincts de même contrat → modèles égaux ; `SYSTEMS != L::LEN` : erreur de compilation.
**Test d'exécution des 13 systèmes** (la compilation d'une liste de treize éléments ne vérifie pas le dispatch) : treize systèmes `Mark<K>` (`K = 0..=12`), chacun `Produce<LOG>` (journal en Ressource, tableau d'octets à longueur propre, capacité ≥ 13) dont le corps ajoute `K` au journal. Le système `Mark<12>` (le treizième, atteint par douze niveaux de `Tail::run`) produit en plus une catégorie `X` lue par `Mark<0>`, de sorte que **le modèle construit place l'index 12 en tête de la phase** ; le test vérifie (i) `model.phase(0)` contient l'index 12, (ii) après un tick le journal contient chaque `K` **exactement une fois**, (iii) le journal égale la traduction du `model.phase(0)` en `K` (le corps du treizième est donc réellement appelé, à la position que le modèle lui assigne, et non en dernier par construction de la liste). Variante : le même test avec le treizième sans contrainte, qui s'exécute à la fin. Si `recursion_limit` est dépassé par la liste, l'attribut est relevé dans le crate de test (limite du compilateur documentée, §3) ; aucun plafond n'est ajouté au moteur. Le coût O(N²) reste non mesuré.
**Commit** : `scheduler: liste de systèmes typée, modèle construit des seuls contrats`.

### Phase 9 — Exécution d'un tick
**Invariants** : I10 (portée : §6.1), I7, I5, I17 ; contrat d'erreur §6.2. Les états se comparent par lectures publiques ; la vivacité s'observe par le harnais (`world.is_alive`) ; les systèmes l'observent par la présence de composants.
**Commit A** (`scheduler: exécution d'un tick, lecture intra-phase`) :
1. **Roadmap, test 1** : A écrit `Velocity`, B la lit ; B déclaré avant A s'exécute après et observe la valeur de A dans la même phase.
2. **Interchangeabilité** : deux jeux de systèmes de même contrat : modèles égaux, états finaux égaux.
3. **Immutabilité** : `run_tick(&self, ..)` ; `model()` identique après N ticks.
4. **Déterminisme** : deux mondes identiques, mêmes ticks : mêmes lectures.
**Commit B** (`scheduler: validation en fin de phase, effets structurels différés`) :
5. **Différé structurel** (aucune dépendance déduite entre producteurs de commandes et lecteurs : I5, ADR-006 point 4). Phase 0 : A émet `Despawn(e)`, `Add(e2, Position)` et `Remove(e3, Velocity)` ; B lit `Position` et `Velocity` ; A et B n'ont aucune donnée en commun, donc **aucune arête**.
   - **5a — ordre par départage déterministe** : A est déclaré **avant** B ; B s'exécute après A parce que l'ordre sans contrainte est l'ordre de déclaration (phase 3), **pas** parce qu'une dépendance de données serait déduite. Le test l'affirme : `precedes(A, B)` est faux dans les deux sens. B voit encore la `Position` de `e` et la `Velocity` de `e3`, et pas la `Position` de `e2`. Phase 1 : C voit `e` purgé, `e3` sans `Velocity`, et la `Position` de `e2`.
   - **5b — dépendance réelle** : A écrit en plus une catégorie distincte `Flag` (`Write<FLAG>`) que B lit (`Read<FLAG>`) ; B est déclaré **avant** A et s'exécute après lui par la dépendance déduite de `Flag`. Mêmes constats de visibilité : l'ordre n'est pas ce qui expose les mutations structurelles ; seul `World::apply` de fin de phase le fait.
   - Un `spawn` émis se vérifie par la capacité du monde après le tick. Les cas de retrait supposent la phase 6bis.
6. **Échec de validation** (3 phases) : phase 0 réussit (écrit une valeur de `c`, émet `Despawn(a)`) ; phase 1 écrit `v`, émet `Despawn(d)` puis un `Add` doublon sur `c` → échoue ; phase 2 écrit un témoin. Résultat `Err(TickError::Apply(ApplyError::Add(InsertError::AlreadyPresent)))` ; (i) `a` reste détruit (phase 0 validée, non restaurée) ; (ii) la valeur de phase 0 est conservée ; (iii) `v` est **conservée** (valeurs non annulées) ; (iv) `d` est vivant, ses composants présents (lot rejeté, non appliqué) ; (v) `buffer.len()` inchangé et non nul ; (vi) le témoin de la phase 2 n'a pas bougé. Ce test est la démonstration de §6.1 : atomicité du lot, rien d'autre.
7. **`BufferFull` pendant l'exécution** (phase de deux systèmes + une phase suivante) : S1 émet une commande valide ; S2 en émet plus que `COMMANDS` → `Err(TickError::Buffer(BufferFull))`. Constats : `apply` non appelé ; le monde est **structurellement inchangé** (aucune commande appliquée, y compris celle de S1) ; la valeur écrite par S1 est conservée ; `buffer.len() == COMMANDS` (commandes acceptées conservées) ; S3 (même phase) et la phase suivante **ne se sont pas exécutés** (témoins inchangés). Retry sans `clear` → `Err(TickError::BufferNotEmpty)`, aucun système exécuté ; après `clear`, la garde est franchie (le tick atteint de nouveau S2 et rend de nouveau `Err(Buffer)`, comportement déterministe).
8. **Défaut de liaison** : `parts()` défectueux pour le deuxième système d'une phase (S1 émet une commande, S2 : emplacement absent) → `Err(TickError::Binding)`, aucune panique ; S1 a écrit sa valeur (conservée) ; sa commande est **retenue dans le buffer et non appliquée** ; S2 et les phases suivantes non exécutés ; retry sans `clear` → `BufferNotEmpty`.
9. **Garde d'entrée** : buffer rempli par l'appelant avant `run_tick` → `Err(TickError::BufferNotEmpty)` ; aucun système exécuté (compteur témoin inchangé) ; monde et buffer inchangés. Après `apply` rejeté (test 6), le même constat vaut pour un nouveau `run_tick` sans `clear`.
10. **Tick réussi** : le buffer est vide à la sortie, y compris pour une phase sans commande.

### Phase 10 — Acceptance et artefact
**10a — acceptance** (`tests/acceptance.rs`). Pipeline : phase 0 `ClearAccumulator` (`Produce<ACC>`, Ressource) ; phase 1 `ContributeA/B/C` (`Produce<ACC>`) et `ResolveMotion` (`Read<ACC>`, `Write<POSITION>`).
- Roadmap, test 1 : lecture après écriture dans la même phase ; un cycle échoue à `Scheduler::new`, aucun tick exécuté.
- Roadmap, test 2 : producteurs libres, un seul résolveur ; deux résolveurs dans la même phase → `MultipleWriters` ; ordre de la phase 1 : {producteurs} < résolveur quel que soit l'ordre de déclaration.
- **Interchangeabilité des producteurs (ADR-003, point 6)** : les contributions de `ContributeA/B/C` sont des entiers **additionnés** dans l'accumulateur (agrégation commutative et associative). Le test construit le `Scheduler` pour les **six permutations** de l'ordre de déclaration des trois producteurs (la phase 1 contient aussi le résolveur) et vérifie que `Position` après un tick est identique (6) pour toutes. Il vérifie l'exemple d'agrégation retenu ; il ne signifie pas que le type `Produce` garantit seul la commutativité de tous les corps de systèmes. Limite conservée : la conformité d'un corps à l'intention métier de sa production ne se prouve pas par le seul type d'accès (§2.4, point 2).
- **Q5, deux ticks** : tick 1 : `Position = 6` ; tick 2 : `12`, pas `18` : aucune contribution du tick précédent. **Témoin négatif** : sans `ClearAccumulator`, le tick 2 donne `18`, ce qui démontre que le Scheduler ne remet rien à zéro de lui-même.
**Commit** : `scheduler: acceptance test, étape 1`.
**10b — exécutable** : `examples/headless.rs`. **Commit** : `schedule: exemple headless, plusieurs ticks sans sortie`.
**10c — façade** : `crates/verbum/Cargo.toml` dépend de `verbum-ecs.workspace = true` ; ajout de `verbum-schedule` dans `[workspace.dependencies]` (chemin et version, comme `verbum-ecs`), de `verbum-schedule.workspace = true` dans la façade, et de `pub use verbum_schedule as schedule;` (rôle documenté : réexporter, sans logique). **Commit** : `verbum: réexporte verbum-schedule`.

---

## 9. Garanties, tests, limites

| Garantie | Démontrée par | Limite connue |
|---|---|---|
| Modèle construit des seuls contrats | phases 5, 8 (modèles égaux pour systèmes distincts) | — |
| Cycle = erreur de construction | phases 3, 10a | diagnostic non détaillé |
| Écrivain unique par phase ; nature cohérente | phase 4, 10a | nature : lecture de I11 |
| Aucun accès hors interface | `compile_fail` §2.6 | étiquetage dans `Locate`, composition de `Produce`, effets externes cachés, vivacité (§2.4, §4.5) |
| Lecture après écriture intra-phase | phase 9, test 1 | — |
| Effets structurels différés, `apply` en fin de phase | phase 9, tests 5a, 5b, 6 | aucun ordre déduit entre producteur de commandes et lecteur |
| Atomicité du lot seulement ; arrêt du tick ; pas de rollback | phase 9, tests 6, 7, 8 | aucune atomicité de phase ou de tick (§6.1) ; reprise après erreur hors périmètre |
| Buffer : garde d'entrée, vidage explicite | phase 9, tests 7, 8, 9, 10 | pas de mécanisme général de récupération (§6.2) |
| Aucun reset implicite | phase 10a, témoin négatif | — |
| Dispatch statique sans plafond | phase 8 (13 systèmes, **exécutés**) | O(N²) par tick, non mesuré ; `recursion_limit` |
| Interchangeabilité de producteurs indépendants | phase 10a (six permutations) | vaut pour l'agrégation choisie, pas pour tout corps |
| Câblage des catégories (`Locate`) | phase 7a (deux Ressources de même type + témoin croisé) | sens métier non déductible par le compilateur (§2.4) |
| `RemoveComponent` opérationnel | phase 6bis (`verbum-ecs`), 7b, 9 (test 5) | écart de l'étape 0 jusqu'à 6bis ; comportement du composant absent = choix local |

## 10. Faisabilité vérifiée
Pour ne pas figer dans le plan une signature impossible à compiler, j'ai compilé un **prototype jetable** (un crate de test dans mon espace temporaire, hors dépôt, **non livré**, avec des substituts simples à `verbum-ecs`) avec rustc 1.97.0. Il confirme : types associés génériques (GAT) pour `Access<'w>` ; `Locate<'w, const ID: u8>` ; liaison dérivée pour tuples ; contrat dérivé en `const` ; liste récursive ; accès disjoint valeurs + Ressource ; et les codes d'erreur du tableau §2.3 et §2.6. **Non vérifié** : compilation sous `rust-version = 1.85` ; comportement avec le vrai `World` (*annotation du 30 octobre 2026* : la faisabilité n'a été vérifiée que sur des substituts et ne couvrait pas les Ressources empruntant des octets ; `Binding::Parts<'w>` et `System::Access<'w>` ont reçu les bornes `where Self: 'w` / `where B: 'w` et `contract_of` une durée de vie `'a` bornée par `B`, dans `crates/schedule/src/system.rs`, sans effet sur les contrats ni l'ordre) ; effet de la limite de récursion sur de grandes listes.

---

## 11. Décisions fixées, phases et découpage

### 11.1 Décisions fixées
- **Q-A / A1** : vues typées, contrat calculé sur le type, identité par `ID` const-générique, `Locate`/`Binding`/`Bind` validés ; limite d'étiquetage assumée et couverte par des tests de câblage.
- **Q-B** : S2, liste récursive, aucun plafond d'arité, ni `dyn` ni pointeur de fonction ; `ExecutionModel::build` ne reçoit que des contrats.
- **Q-C / A2** : accumulateur en Ressource ; `split_mut` autorisé en commit distinct (signature à valider) ; pas de `&World`/`&mut World` côté système.
- **Q5** : système explicite de remise à zéro, deux ticks avec témoin négatif.
- **Q-D / A3** : quatre catégories dans le vocabulaire ; `RemoveComponent` corrigé en commit distinct (6bis) avant 7b ; comportement du composant absent : choix local « sans effet ».
- Atomicité du lot de phase seulement ; arrêt du tick après un `apply` rejeté ; `MixedNature` global ; ADR-000 adopté (confirmé).
- Tests ajoutés : deux Ressources de même type (7a) ; test 5 corrigé (5a, 5b) ; interchangeabilité des producteurs (10a).

### 11.2 Points à confirmer à l'implémentation (non bloquants pour valider ce plan)
| # | Point | Moment |
|---|---|---|
| B1 | Signature exacte de `split_mut` | phase 6 |
| B2 | Forme du changement de `ComponentSet`/`CommandBuffer` pour `RemoveComponent` : non purement additif pour les implémenteurs du trait (§5.1) | phase 6bis, à la revue du commit |
| B3 | Choix local « retrait d'un composant absent = sans effet » | phase 6bis |

**Blocage normatif** : aucun identifié. Dépendances non vérifiées : compilation sous `rust-version = 1.85`, comportement avec le vrai `World`, coût O(N²) du dispatch (§10).

### 11.3 Dépendances entre phases
Arêtes de dépendance (A → B : B exige A) :
```
0 → 1 → 2 → 3 → 4 → 5 → 8          (piste Scheduler : modèle)
6 → 6bis                             (piste verbum-ecs : correctifs, séquentiels)
1, 6 → 7a ;  1, 7a, 6bis → 7b ;  7a, 7b → 8
8 → 9 → 10a → 10b ;   0, 8 → 10c
```
| Phase | Dépend de | Touche |
|---|---|---|
| 0 | — | workspace (`Cargo.toml` racine, `Cargo.lock`), `crates/schedule` |
| 1 à 5 | 0 ; séquence 1 → 2 → 3 → 4 → 5 | `crates/schedule` seul, aucune dépendance à `verbum-ecs` |
| 6 `split_mut` | aucune | `verbum-ecs` : `world.rs` (+ tests) |
| 6bis `RemoveComponent` | aucune ; **enchaînée après 6** (même `world.rs`, jamais en parallèle ni dans le même commit) | `verbum-ecs` : `component_set.rs`, `world.rs`, `command.rs`, `fixtures.rs`, tests, doctests |
| 7a | 1, 6 ; **ordre pratique : après 6bis** (les implémenteurs de `ComponentSet` des tests du Scheduler seraient sinon à réécrire) | `verbum-schedule` |
| 7b | 1, 7a, **6bis** | `verbum-schedule` |
| 8 | 5, 7a, 7b | `verbum-schedule` |
| 9 | 8 | `verbum-schedule` |
| 10a | 9 | `verbum-schedule` (tests) |
| 10b | 10a | `verbum-schedule` (exemple) |
| 10c | 0, 8 | façade `verbum`, `Cargo.toml` racine |

**Développables indépendamment** : la piste {1 à 5} et la piste {6 puis 6bis}, car leurs ensembles de fichiers sont disjoints (seule la piste Scheduler touche `Cargo.toml` racine et `Cargo.lock`, par la phase 0). Deux sessions ne modifient **jamais** les mêmes fichiers en même temps ; le workspace partagé est traité séquentiellement : phase 0 (racine) d'abord, phase 10c (racine) en dernier. **Emplacement des deux correctifs `verbum-ecs`** : **6** (`split_mut`, avant 7a) et **6bis** (`RemoveComponent`, avant 7b, ordre pratique avant 7a). Aucune autre modification de `verbum-ecs`.

### 11.4 Découpage entre sessions
Le plan d'exécution complet (fichiers, prérequis, commits, commandes, interdits, handoff autonome, règles de nouvelle session) est dans **`01-verbum-scheduler-sessions.md`**. Résumé :

| Session | Phases | Prérequis | Sortie vérifiée |
|---|---|---|---|
| S1 | 0 à 5 | ce plan validé | `verbum-schedule` : contrat, ordre, modèle |
| S2a | 6 | S1 mergée ; autorisation A2 (donnée) | commit `split_mut` ; Miri par l'utilisateur |
| S2b | 6bis | S1 et S2a mergées, Miri vert ; revue B2 | commit `RemoveComponent` ; Miri |
| S3 | 7a, 7b | S1, S2a, S2b mergées | vues typées, liaison, commandes |
| S4 | 8, 9 | S3 | `Scheduler`, exécution d'un tick |
| S5 | 10a à 10c | S4 | acceptance, exemple, façade |
Ordre recommandé : strictement séquentiel S1 → S2a → S2b → S3 → S4 → S5 (le handoff de S1 attend les décomptes de l'étape 0). Le travail de S1 en branche séparée, en parallèle de S2, reste théoriquement possible (fichiers disjoints) mais n'est pas recommandé pour ce démarrage.
