# Passation — fin de l'étape 0 (`verbum-ecs`), ouverture de l'étape 1

Document de travail, non normatif. Rédigé le 12 octobre 2026 à partir de l'état réel du dépôt. Aucune ADR n'est modifiée ni à modifier.

## 1. Historique de l'étape 0

Du plus récent au plus ancien :
- vue mutable + acceptation
- query
- borne Miri du test 4a
- resource
- borne Miri du test 4b
- ComponentSet / add_component / purge / atomicité sur les stockages
- atomicité stricte de `apply`
- Command Buffer spawn/despawn
- World minimal
- SparseSet sans `Default`
- SparseSet
- entity
- workspace
- plan

Pour plus de détail sur étape 0, voir `docs/implementation/00-verbum-ecs.md`.

## 2. Validation

Commandes (depuis la racine du workspace) :

```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo +nightly miri test -p verbum-ecs    # exécuté par l'utilisateur
```

Résultats relevés le 12 octobre 2026 (rustc/cargo 1.97.0, édition 2024, `rust-version = 1.85`) :

- build, clippy `-D warnings`, fmt : sans avertissement ; fmt `--check` code 0.
- `cargo test --workspace` : **64 tests unitaires** (`verbum-ecs`), **11 tests d'intégration** (`acceptance` 2, `command_buffer` 1, `components` 3, `query` 2, `resources` 2, `world_identity` 1), **5 doctests** (1 conforme + 4 `compile_fail` épinglés). Le crate `verbum` (façade) n'a aucun test.
- **Miri : vert, vérifié par l'utilisateur**, en effet, l'environnement de Claude ne permet pas de télécharger le toolchain nightly (les doctests `compile_fail` ne s'exécutent pas sous Miri).
- Mutation testing manuel (non automatisé, non rejouable par une commande) : les tests exhaustifs de la prévalidation, de la Query et de `ResourceSlot` ont été éprouvés par des mutants ; un seul survivant, équivalent.

## 3. Documents : emplacement et rôle

```
verbum/docs$ tree
.
├── architecture
│   ├── adr
│   │   ├── ADR-000-language-and-material-invariants.md
│   │   ├── ADR-001-sparse-set.md
│   │   ├── ADR-002-query-model.md
│   │   ├── ADR-003-contrat-systeme.md
│   │   ├── ADR-004-modele-execution.md
│   │   ├── ADR-005-ressources.md
│   │   ├── ADR-006-command-buffer.md
│   │   ├── ADR-A01-assets.md
│   │   ├── ADR-AU01-audio.md
│   │   ├── ADR-N01-reseau.md
│   │   ├── ADR-R01-modele-donnee-rendu.md
│   │   ├── ADR-R02-pipeline-production.md
│   │   ├── ADR-R03-backend.md
│   │   ├── ADR-S01-shell-responsabilites.md
│   │   ├── ADR-Sc01-scripts.md
│   │   └── ADR-SV01-sauvegarde.md
│   ├── concepts
│   │   ├── 00-principes.md
│   │   ├── 01-runtime-ecs.md
│   │   └── 02-concepts.md
│   ├── README.md
│   └── ROADMAP.md
├── drafts
│   ├── bow_recurve.md
│   ├── grid_64.svg
│   ├── LPC-Paradigm.md
│   └── LPC_YAML
│       ├── action_resolution.yaml
│       ├── base_character.contract.yaml
│       ├── bow_recurve.contract.yaml
│       ├── build_manifest.yaml
│       ├── canonical_animations.yaml
│       ├── gnarled_staff.contract.yaml
│       ├── hammer.contract.yaml
│       ├── longsword.contract.yaml
│       └── trident.contract.yaml
├── handoffs
│   ├── HANDOFF-Stage-0.md
│   └── HANDOFF-Stage-1.md
├── implementation
│   └── 00-verbum-ecs.md
└── reflexivity
    ├── filiation.md
    ├── methodological-lineage-from-marius-to-verbum.md
    ├── scribe-conservateur.md
    ├── when-context-leaves-the-conversation-to-enter-the-corpus.md
    └── why-the-name-verbum.md
```

- Rôle de ce document (`docs/handoffs/HANDOFF-Stage-1.md`) : **journal de construction de l'étape 0** (non normatif, révisable sans ADR). Il contient, par phase, l'invariant cité, le test, l'API, le journal des décisions locales et le commit. La section « Critère de sortie » porte le statut de l'étape 0. Le journal de la Phase 4b et celui de la Phase 6 sont les plus denses. C'est le modèle à suivre pour le plan de l'étape 1.
- `docs/architecture/ROADMAP.md` : séquence des étapes ; l'étape 0 est terminée.
- `docs/handoffs/HANDOFF-Stage-0.md` (session précédente) : discipline de travail, toujours valable (voir §7).

## 4. Décisions structurantes de l'étape 0

Toutes sont consignées dans le journal du plan ; les choix locaux n'y sont pas des invariants du corpus.

**Mémoire (ADR-000, option A)** : `verbum-ecs` est `#![no_std]` sans `alloc`. Mémoire fournie à la construction, capacités const-génériques, dépassement = erreur explicite, jamais de croissance. `unsafe_code = "deny"` au niveau du workspace ; deux blocs `unsafe` seulement, dans `storage.rs` (`SparseSet::get`, `get_mut`), fondés sur l'invariant « `dense_data[i]` initialisé ssi `i < len` ; toute position sparse non absente est `< len` ». `T: Copy`, sans `Default`.

**Identité** : `EntityId(u64)` opaque (index bas, génération haute), sans constructeur public. Table de génération avec liste libre intrusive LIFO et compteur de slots libres ; un slot dont la génération atteint `u32::MAX` est retiré, jamais réattribué ; `EntityId::VACANT` ne peut jamais être émis.

**`ComponentSet`** : trait implémenté par l'appelant, pas un trait `Component`, ni un registre, ni un plugin. Types associés `Add` (enum fermé de variantes typées, une par stockage), `Counters` (`[usize; N]` de simulation sur la pile), `Values<'a>`. Méthodes : `kind`, `dense_len`, `dense_capacity`, `covers`, `is_present` (prévalidation, sans mutation), `insert`, `remove_all` (application, purge), `values_mut`. Les stockages sont des champs `SparseSet` accessibles par `world.components()` (lecture seule). `impl ComponentSet for ()` : monde sans composant (`Add = Infallible`). Aucun `TypeId`. Types de test (`Position`, …) uniquement dans les tests.

**`ValuesMut` et `ValuesView`** : le `World` n'expose **jamais** `&mut S`. `World::component_values_mut()` rend `S::Values<'_>`, composé exclusivement de `ValuesMut` (vue exclusive sur un `SparseSet` : `get`, `get_mut`, `contains`, `entities`, `len`, `shared() -> &SparseSet` ; ni `insert` ni `remove`). `ValuesView` est **scellé** (implémenté seulement pour `ValuesMut`, `()` et les tuples d'arité 1 à 4) : un `Values = &mut Self` ne compile pas. Vérifié par des doctests `compile_fail` épinglés (E0599, E0277).

**Command Buffer et atomicité (ADR-006)** : `CommandBuffer<COMMANDS, A = Infallible>` accumule librement (`spawn`, `despawn`, `add_component`) ; `World::apply(&mut buffer) -> Result<(), ApplyError>` est la **seule** résolution. Prévalidation du lot dans l'ordre d'émission, sans mutation ni allocation (`plan` : simulation des slots libres, des longueurs denses par stockage, de la présence ; drapeau par commande sur la pile), puis application de la seule séquence prévalidée. En cas d'erreur : `World` entièrement inchangé (entités, générations, liste libre, stockages, Ressources) et buffer **conservé intact** ; l'appelant le vide avec `clear`. `ApplyError` = `Spawn(CapacityExceeded)` | `Add(InsertError)`, première erreur dans l'ordre d'émission. Coût O(n²) borné par la capacité du buffer : mesurable, optimisable sans changer le contrat.

**Règles de cycle de vie (choix locaux, non spécifiés par le corpus)** : les handles d'un lot sont évalués contre le monde avant le lot (aucune commande ne désigne une entité créée par le même lot) ; une destruction ou un ajout sur un handle périmé ou déjà détruit dans le lot est **sans effet** (ignoré) ; ajout avant destruction : appliqué puis purgé ; destruction avant ajout : ajout ignoré ; composant déjà présent, stockage dense plein, index hors de l'étendue sparse, création au-delà de la capacité : **erreur, lot entier rejeté** ; handle périmé (ignoré) et index vivant hors étendue (erreur) sont distincts. `World::despawn` purge tous les stockages avant la réutilisation du slot ; une destruction périmée ne touche jamais le nouvel occupant ; un slot retiré purge quand même. Le stockage ne revalide jamais la génération (ADR-001, point 2) : la vivacité relève du `World` (`is_alive`).

**Ressources (ADR-005)** : `ResourceSlot<T>` (`Option<T>`, aucun `unsafe`, aucune contrainte sur `T`) ; `World<ENTITIES, S = (), R = ()>` avec `resources()` / `resources_mut()`. La clé d'une Ressource est sa position statique (champ d'un type `R` défini par l'appelant, `R: Default`). Insertion directe, sans commande différée ; indépendante du cycle de vie des entités et de `apply`.

**Query (ADR-002)** : `ReadWriteQuery<'a, R, W, ..>::new(&SparseSet<R>, ValuesMut<W>)`, itération prêteuse `advance()` → `(EntityId, &R, &mut W)`. Mode d'accès dans le type, stockage pilote = le stockage lu (premier argument), ppas de `world.query()`, aucune dépendance à `world`, deux références et un curseur, aucun cache. Depuis un monde : `let (p, v) = world.component_values_mut(); ReadWriteQuery::new(p.shared(), v)`.

**Dépendances de modules réelles** (conformes à la table du plan) : `entity` ← rien ; `storage`, `command` ← `entity` ; `values` ← `entity`, `storage` ; `query` ← `entity`, `storage`, `values` ; `component_set` ← `entity`, `storage`, `values` ; `world` ← `entity`, `storage`, `command`, `component_set` et **jamais `query`**.

**Non implémenté faute de test qui l'exige** : filtres `With`/`Without`, Query à plus de deux stockages ou en lecture seule, syntaxe tuple, critère « plus petit stockage » pour le pilote, commande différée d'insertion de Ressource.

## 5. Limites de couverture sous Miri

- Deux tests exhaustifs comparent `apply` à l'application séquentielle des primitives : `prevalidation_is_exact_and_atomic_over_all_small_batches` (Phase 4a) et `prevalidation_with_components_is_exact_and_atomic_over_all_small_batches` (Phase 4b ; alphabet de 10 commandes, 5 scénarios).
- **Suite ordinaire** : exploration **complète, jusqu'à 5 commandes par lot** (`LEN = 5`), mêmes scénarios, mêmes assertions. C'est la couverture de référence ; elle ne doit jamais être réduite.
- **Sous `cfg(miri)`** : `MAX_LENGTH = 3` (`world.rs`, deux occurrences de `cfg!(miri)`), mêmes scénarios, mêmes assertions, test non désactivé. Les lots de 4 et 5 commandes ne sont donc couverts que par les tests ordinaires. Arbitrages de l'utilisateur du 11 octobre 2026, documentés dans le plan.
- Aucun autre test n'est réduit ou exclu sous Miri. Les 256 combinaisons de présence du test de Query tournent entièrement.

## 6. Étape suivante (ROADMAP.md) : étape 1 — Scheduler

- **Contenu** : contrat de système (interface déclarative), construction du graphe de dépendances à partir des contrats, phases, point de validation, exécution. **Dépendance** : étape 0 (terminée).
- **Tests d'invariant de la Roadmap** : (1) deux systèmes d'une même phase, l'un lisant ce que l'autre a écrit plus tôt dans la même phase, observent la même valeur ; un cycle de dépendance provoque un **échec de construction**, jamais une erreur runtime (ADR-004, point 4). (2) Plusieurs systèmes produisant librement vers une même donnée transitoire, un seul système en aval en assure la résolution vers la donnée persistante : « produire est libre, résoudre est unique » respecté au niveau du Scheduler.
- **Artefact attendu** : premier exécutable headless exécutant plusieurs ticks sans sortie visible.
- **À relire avant de rédiger le plan** (corpus, dans cet ordre) : `ROADMAP.md` (étape 1) ; `00-principes.md` (grille de classification, règle méta) ; `01-runtime-ecs.md` (phases, propriétaire unique, produire/résoudre, mutations différées) ; `02-concepts.md` (quatre axes, critère de validité) ; **ADR-003** (contrat système, interface déclarative, localité) ; **ADR-004** (modèle d'exécution : dépendances déduites, cycles, phases) ; ADR-002, ADR-005 et ADR-006 pour la façon dont lecture/écriture, Ressources et Command Buffer entrent dans les contrats ; `rust-vs-zig.md` / ADR-000 pour la contrainte mémoire (no_std sans alloc : graphe et ordonnancement devront aussi tenir sur mémoire fournie) ; `docs/PLAN-ETAPE-0-ECS.md` (ou son emplacement actuel) comme modèle de plan.
- **Points d'attention issus de l'étape 0** : le mode d'accès (lecture/écriture) est déjà dans les types des Queries, ce que la déclaration de contrat devra pouvoir inspecter ; l'accès mutable aux valeurs passe par `component_values_mut`, et la structure par `apply` seulement ; le point de validation est `World::apply` ; décider (en revue, pas seul) dans quel crate vit le Scheduler (le noyau `verbum-ecs` ne dépend d'aucun système, d'après la Roadmap).

## 7. Discipline de travail à reprendre

- La nouvelle session commence par **lire** la Roadmap et les documents ci-dessus, puis **prépare le plan d'implémentation de l'étape 1, soumis à revue avant toute implémentation**. Aucune implémentation, aucune nouvelle abstraction, aucune modification d'ADR avant validation du plan.
- Rôle : implémenteur, pas architecte. Tests d'abord, API minimale, un commit par invariant, quatre contrôles verts à chaque commit, Miri par l'utilisateur. Rien sans invariant ; alarme « tant que j'y suis ». S'il manque une information : s'arrêter et demander plutôt que deviner la documentation. Contradiction dans les instructions : s'arrêter et faire un rapport.
- Toujours demander à l'utilisateur les ressources manquantes, notamment `.rs`, plutôt que de tenter de deviner le comportement d'un programme manquant dans le contexte.
- Clause d'échappement : si une difficulté structurelle apparaît, **l'exposer avant de contourner le contrat** (c'est ainsi qu'ont été arbitrés `ComponentSet`, l'atomicité de `apply` et `ValuesMut`).
- Préférences de l'utilisateur : réponses concises, structurées, opérationnelles ; pas de généralités ni de métaphores ; définir brièvement tout concept avancé ; distinguer dans les rapports les problèmes de représentation mémoire des conflits réels avec un ADR ; signaler les choix locaux comme non-invariants.
- Livraison : fichiers entiers sur disque ; archive `tar --exclude=verbum/target -czf verbum-<nom>.tar.gz verbum` après chaque phase.
