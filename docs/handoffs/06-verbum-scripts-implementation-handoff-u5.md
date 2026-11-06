# Verbum — Étape 6 (scripts) — Handoff d'implémentation de l'unité U5

**Statut : document de passation, produit à la clôture d'U4. Ce document ne vaut pas autorisation de démarrer.** La session ne code, ne crée aucun fichier et ne lance aucune commande de build ou de test avant l'instruction explicite de l'utilisateur : « démarrez l'unité U5 ».

Étiquettes : **[Vérifié]** lu ou exécuté ; **[Déduit]** conclu d'une lecture ; **[À vérifier]** non établi à la rédaction.

---

## 1. Rôle et posture

Architecte système / engine programmer (ECS, DOD, AOT, Rust/Cargo). Une unité à la fois, une session par unité. L'utilisateur seul déclare une unité validée ; la session ne déclare jamais l'étape close. Vouvoiement, réponses concises, opérationnelles, sans pédagogie scolaire ; tout terme avancé est défini brièvement.

## 2. Conduite au démarrage

1. Accuser réception, lire les documents (§ 3), confronter les faits [Vérifié] du § 9 aux fichiers reçus.
2. Réclamer les pièces manquantes du § 14, **sans élargir** la demande.
3. Attendre « démarrez l'unité U5 ». Aucun code, aucun fichier, aucune commande avant.
4. Ensuite : liste de tâches (dernière tâche = vérification), annonce d'une phrase, travail, rapport (§ 12).

## 3. Documents de référence et hiérarchie

| Rang | Document | Usage |
|---|---|---|
| 1 | `06-verbum-scripts.md` (plan r1.2) | **Prévaut** : § 7.5 (DM-1 à DM-3), § 7.6, § 8 (U5), § 9, § 10 (non démontré, à reproduire **littéralement**), § 14 |
| 2 | `06-verbum-scripts-execution.md` (révision du 3 nov. soir) | § 4 « U5 », § 5 (gabarit de rapport), § 6 (arrêt/reprise), § 7, § 8 (T1 à T15), E-1 à E-14 |
| 3 | `06-verbum-scripts-implementation-handoff-u4.md` | Modèle de forme et de méthode (hérité) |
| 4 | `06-verbum-scripts-report-u3.md`, ADR-000, ADR-003, ADR-004, ADR-Sc01, `01-runtime-ecs.md` | Contexte |

Une contradiction entre rangs 1 et 2 est un arrêt (§ 13) ; le plan prévaut (arbitrage antérieur de l'utilisateur).

## 4. Objectif et périmètre d'U5

Premier comportement scripté **de bout en bout, headless, sous la boucle du Shell** (`run_loop`), contrôles de clôture, rapport « non démontré ». Chaîne : source TOML (scénario de référence) → `compile_assets` (un script, slot 4, capacité 6) → `CompiledSet` → `load` (table `[Format::Script; SCRIPT_SLOTS]`) → dépôt dans `ScriptTable` → lancement par `apply` → `run_loop` (cadence vide, arrêt après N = 12 itérations, `Inbox::<u8>::new(4)` vide) → fermeture d'infrastructure : journal `TickStart(k)`, `run_tick`, `TickEnd(k)`, lecture **en lecture seule** de `TRANSCRIPT`, copie dans l'observateur de la Plateforme (un `Vec` de l'appelant), `Present(k)`.

**Livrables (uniques)** : `crates/engine/shell/tests/script_loaded.rs` et `crates/engine/shell/examples/script_headless.rs`. **Aucune autre modification** : `shell/Cargo.toml` est complet depuis U4, la racine est inchangée, `script_support/mod.rs` et `script_suspend.rs` ne sont pas touchés.

## 5. Tests (liste fermée : aucun test hors liste)

| Test | Énoncé |
|---|---|
| DM-1 | 12 itérations ; l'observateur reçoit 12 transcripts égaux au littéral : ticks 1, 4, 7 → `{(1,10)}` ; tick 10 → `{(2,99)}` ; vides ailleurs ; état final égal à la ligne « 10 » (26 octets : `04 00 06 00 02 00` puis 20 octets nuls) ; `Beat` = 12 |
| DM-2 | Quatre lots corrompus, chacun : (a) artefact tronqué d'un octet ; (b) `Wait` de zéro tick (octet de ticks de l'enregistrement 2 à zéro) ; (c) registre hors bornes (octet de registre de l'enregistrement 0 mis à `SCRIPT_REGISTERS`) ; (d) table de formats attendant `Format::Sprite` au slot 4. `load` échoue (attendu : `Artifact { Truncated }`, `Artifact { Instruction }` deux fois, `SlotFormat` du slot 4), **aucun tick, aucune présentation** (journal vide, observateur vide) |
| DM-3 | Journal exactement `[TickStart(k), TickEnd(k), Present(k)]`, k de 1 à 12 ; la présentation reçoit un `&Emissions` en lecture seule ; aucune lecture de `EMISSIONS` hors des systèmes (revue) ; l'historique est tenu par l'observateur, jamais par le `World` (après le tick 12, `TRANSCRIPT` du `World` ne contient que le transcript du tick 12, vide) |

Aucun rouge possible (API déjà implémentées) : vert au premier passage, sensibilité par mutation. Le `CompiledSet` est déclaré **avant** le `World`. Cadence = fermeture vide dans les tests ; seul l'exemple utilise `FixedStepPacing` (1 ms, 12 itérations).

**Exemple** : `examples/script_headless.rs` **recopie** l'assemblage minimal (il ne peut pas inclure un module de `tests/` ; précédent `level_headless.rs`). Imprime la liste des émissions de chaque tick (format libre, provisoire), vérifie en interne les 12 transcripts (panique sinon), code de sortie 0, aucun backend graphique.

## 6. Mutations d'U5 (copies isolées, une à la fois)

| # | Mutation | Détecteur |
|---|---|---|
| M-5.1 | résultat de `load` ignoré | DM-2 (journal non vide) |
| M-5.2 | présentation avant `run_tick` | DM-3 (ordre) ; DM-1 attendu aussi |
| M-5.3 | l'observateur lit `TRANSCRIPT` après le tick suivant | DM-1 ; DM-3 attendu aussi |
| M-5.4 | valeur d'émission modifiée dans la source (10 → 11) | DM-1 |
| M-5.5 | `Collect` désactivé dans l'assemblage de l'exemple | exemple : panique interne, code ≠ 0 |
| M-5.6 | l'exemple ne vérifie pas ses transcripts | **équivalente côté tests** (propriété portée par DM-1) ; revue, pas de test artificiel |

Rapporter le **mode de détection observé** (assertion, panique, erreur), jamais supposé. Une mutation non détectée et non établie équivalente, ou qui fait échouer d'autres assertions que celles annoncées sans explication par le code : arrêt.

## 7. Contrôles de clôture et rapport

- **Utilisateur, dépôt réel** : `cargo fmt --all -- --check` ; `cargo build --workspace` ; `cargo test --workspace` (doctests compris) ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo run -p verbum-shell --example script_headless` (code 0) ; `cargo tree` (assets sans dépendance, `[dev-dependencies]` compris ; `verbum-script -e normal` : seule `verbum-assets` ; `-i toml --workspace` : seul `verbum-forge` ; `-i verbum-forge --workspace -e normal` : aucun dépendant ; `-i verbum-script --workspace` : seul `verbum-shell`, en dev) ; diff vide sur `crates/{ecs,schedule,render,verbum}/**` et `crates/engine/shell/src/**` ; assertions de F14 inchangées.
- **Miri (utilisateur)** : `cargo +nightly miri test -p verbum-assets` ; `-p verbum-script` ; `-p verbum-forge -- --skip p3_1 --skip p3_2 --skip t6_1 --skip t6_3 --skip t6_6` (commande du document d'exécution, exclusions de l'étape 5 déjà décidées) ; `-p verbum-shell`. L'exemple n'est pas exécuté sous Miri. Aucun `--skip` ni `ignore` nouveau sans décision.
- **MSRV 1.85 (utilisateur)**, deux commandes : `cargo +1.85.0 build --workspace`, puis `cargo +1.85.0 test --workspace`. Pas de let-chains ; aucune assertion sur l'horloge ; aucune constante utilisée seulement dans une `const _` assert.
- **Rapport** : gabarit du document d'exécution § 5 (12 rubriques), tableau **T1 à T15** (plan § 1.3, critères § 14 : identifiants de tests et revues, résultats réels), **§ 10 du plan r1.2 reproduit littéralement** avec mention que 4 et 16 sont provisoires, plus un complément distinct « Compléments d'exécution » (§ 7 du document d'exécution). Revue de l'absence d'état caché (plan § 7.5, T5) : recherche textuelle de `static`, `thread_local`, `Cell`, `RefCell`, atomiques, `async`, `await`, `thread`, `Box`, `Vec`, `unsafe` dans `crates/extensions/script/**` et dans le corps des systèmes (les `Vec` du journal de l'observateur, hors systèmes, sont admis). Résultats de l'espace de travail et du dépôt réel **toujours séparés**. Fin : « En attente de la validation de l'unité U5. »

## 8. Environnement de la session (acquis d'U4)

[Vérifié] Session sans `nightly` (Miri et MSRV à la charge de l'utilisateur) ; chaîne stable ≈ 1.97 ; crates.io accessible par le proxy (`toml` 1.x, versions figées par `Cargo.lock`). `cargo fmt --all` échoue si `ecs/src/fixtures.rs` est absent : employer `rustfmt --edition 2024 <fichiers>`. Les modules étrangers au périmètre peuvent être stubés dans un espace isolé, à trois conditions : jamais sur les API réelles utilisées, recensés au rapport, aucun résultat présenté comme validation du dépôt réel. **Exception pour U5 : `shell/src/{lib,shell,inbox,pacing}.rs` doivent être les sources réelles** (`run_loop` est l'objet de la démonstration) ; ne pas les stuber.

## 9. État réel des unités (ne pas rouvrir)

| Unité | Contenu | Validation |
|---|---|---|
| U1 | lecteur `script.bin` dans `verbum-assets` | validée par l'utilisateur, dépôt réel vert (build, test, clippy, fmt, Miri intégral) |
| U2 | Forge : `compile_script`, `AssetKind::Script`, `compile_assets`, budget/cycles | validée ; Miri forge partiel |
| U3 | crate `verbum-script` (`ScriptState`, `Emissions`, `ScriptTable`, `step`) | validée ; Miri 19 tests verts |
| **U4** | `shell/tests/script_support/mod.rs`, `script_suspend.rs`, dev-dépendance `verbum-script` | **validée par l'utilisateur le 6 novembre 2026** |

**U4, ce qui a été testé où.**
- **Espace isolé de la session** [Vérifié] : 12 tests verts (CT-1 à CT-3, SY-0, SR-1 à SR-7 dont SR-5 en deux variantes), debug et release ; clippy `-D warnings` et rustfmt propres sur les deux fichiers ; 16 mutations sur 17 détectées, M-4.14 (ordre inversé) survit **par conception** (G-ORD-S), M-4.12 détectée à la compilation (`E0080`), M-4.15 non rejouée (équivalente à M-3.2). Rouge constaté : 7 paniques `not implemented` ; CT-1 à CT-3, SR-6 et SR-7 verts d'emblée (ils ne passent pas par les corps des systèmes). Stubs de session (assets `header/sprite/font/map/tileset`, forge `id/font/map/tileset`, ecs `query`, shell `inbox/pacing/shell`, `render` et `verbum` vides, manifestes reconstitués) : **ne pas les reprendre**.
- **Dépôt réel de l'utilisateur** : build, tests, clippy, fmt, Miri, MSRV et documentation verts, **sauf** `p3_1_full_chain_on_disk_with_the_source_deleted_before_reading` (test Forge, hors U4) sous Miri isolé (`mkdir` refusé). Voir § 15.

## 10. Assemblage U4 sur lequel U5 s'appuie (fichiers tels que rendus)

`crates/engine/shell/tests/script_support/mod.rs`, inclus par `#[path = "script_support/mod.rs"] mod support;` — **disposition conforme au plan et au document d'exécution (E-1)** ; `script_loaded.rs` l'inclut de la même façon, avec le même attribut `#[allow(dead_code, reason = …)]` **si et seulement si** `clippy -D warnings` l'exige (en U4, seules les variantes `Remove::{State, Beat}` le déclenchent) ; le dire au rapport. **`mod.rs` est en lecture seule** : tout besoin d'y ajouter ou modifier quelque chose est un arrêt à arbitrer (U5 ne crée que deux fichiers) ; les éléments propres à U5 (journal, observateur, fermeture d'infrastructure) vivent dans `script_loaded.rs`.

API publique du module [Vérifié] :
- **Constantes** : `ENTITIES=4`, `SPARSE=4`, `STATE_DENSE=4`, `COMMANDS=4`, `EMISSIONS_CAP=64`, `SCRIPT_SLOTS=6`, `SYSTEMS=4`, `PHASES=2`, `RIVAL_SYSTEMS=5`, catégories `STATE=0, SCRIPTS=1, EMISSIONS=2, TRANSCRIPT=3, BEAT=4`, `SLOT_A=4`, `SLOT_B=1`, `SCRIPT_FORMATS`, `MAX_STEPS`.
- **Types** : `Beat(u32)`, `Anchored<'a,T>`, `Acc = Emissions<EMISSIONS_CAP>`, `Components<'a>`, `Add<'a>`, `Remove`, `Rs<'a>` (`scripts: ResourceSlot<ScriptTable<'a,SCRIPT_SLOTS>>`, `emissions` et `transcript`: `ResourceSlot<Anchored<'a, Acc>>`), `W<'a> = World<ENTITIES, Components<'a>, Rs<'a>>`, `Buf<'a> = CommandBuffer<COMMANDS, Add<'a>, Remove>`, `Parts<'w,'a>`, `B<'a>`.
- **Systèmes** : `Clear` (phase 0), `Collect`, `Interpreter`, `Drift` (phase 1) ; listes `Systems`, `RivalSystems` ; `Sched<'a> = Scheduler<B<'a>, Systems, SYSTEMS, PHASES>` ; contrôles `WideInterpreter`, `RivalWriter`. Ordre exécuté déduit des contrats : Clear < Interpreter < Collect ; Drift départagé par l'ordre de déclaration.
- **Sources et lots** : `SCRIPT_A_TOML`, `SCRIPT_B_TOML`, `ARTIFACT_A` (littéral de 68 octets), `compile_scripts(&[(clé, source, slot)]) -> CompiledSet`, `reference_set()` (contrôle l'artefact contre le littéral), `pair_set()`, `slices(&CompiledSet) -> Vec<&[u8]>`.
- **Fonctions** : `build_world()` (table vide, `EMISSIONS` et `TRANSCRIPT` présents et vides), `load_scripts(&mut W, manifest, artifacts) -> Result<(), LoadError>` (vrai `load`, dépôt des seuls `Asset::Script`), `launch(world, buffer, slot, beat) -> EntityId` (spawn, `add_component` ×2, vérification d'absence, `apply`), `launch_observed(…, observe)`, `capture(&W, id) -> [u8; 26]`, `reconstruct(manifest, artifacts, &capture, beat) -> (W, Buf, EntityId)`, `tick(&Sched, &mut W, &mut Buf)` (panique si `run_tick` échoue), `transcript(&W) -> Option<Vec<(u16,i32)>>`, `multiset`, `beat_of`.

**Écarts de la livraison U4 par rapport au document d'exécution** (validés avec U4, à connaître) :
1. `RivalWriter` porte un `Read<SCRIPTS>` en plus de `Write<STATE>` : `Write<STATE>` seul échoue en E0477 sous `B<'a>` (aucun type portant `'a`) ; l'ancre n'affecte pas la propriété de `STATE`.
2. L'`Interpreter` produit dans `EMISSIONS`, dont le slot est `Anchored<'a, Acc>` (conforme à `Rs<'a>`) ; son accès nomme aussi `ScriptTable<'a>`. L'hypothèse « compile sans `Anchored` » n'a pas été isolée.
3. CT-1 avec `B<'static>` a compilé (aucun arrêt G-LT).
4. `Scheduler` n'implémente pas `Debug` : `Sched::new()` se traite par `.expect`/`.err()`.
5. `ModelError` est publique à `verbum_schedule::model::ModelError`.

## 11. Invariants à ne pas remettre en discussion

- Crates `no_std` sans `alloc` (ADR-000) ; `unsafe_code = "deny"` ; aucun état hors `World` (pas de `static`, cellule, atomique) ; les systèmes sont des types sans donnée.
- `verbum-script` (U3) : **aucune modification**, aucune API publique ajoutée, aucun `Default` ajouté.
- Contrat de chaque système **déduit du type d'accès** ; interface de l'interpréteur fixe (CT-1) ; `STATE` : un seul écrivain.
- `TRANSCRIPT` : remplacé en entier à chaque tick par `Collect`, y compris vide ; `EMISSIONS` lu seulement par `Collect` ; l'historique appartient à l'observateur de la Plateforme.
- Aucun ordre d'émission affirmé entre instances (G-ORD-S) : comparaisons en multiensemble ; **un seul script, une seule instance en U5**.
- Lancement par `CommandBuffer::add_component` + `World::apply` seulement ; reconstruction depuis les 26 octets seuls.
- Les comparaisons de transcripts portent sur les `Emissions` internes, jamais sur les enveloppes `Anchored`.
- Mécanismes de l'étape 5 (niveaux de `load`, `VMAN`, identité) intacts ; tests de F14 inchangés.
- Dépendance `verbum-script` en `[dev-dependencies]` de `verbum-shell` : conservée telle quelle (§ 15 A).

## 12. Interdits

Modifier `crates/{ecs,schedule,render,verbum}/**` ; `crates/engine/shell/src/**` (documentation `//!` comprise, aucun doctest nouveau) ; `crates/extensions/script/**` ; `crates/engine/assets/src/**` et `crates/engine/forge/src/**` ; `Cargo.toml` (racine ou crates) et `Cargo.lock` ; `script_support/mod.rs`, `script_suspend.rs` ; tout test d'une étape antérieure. Aucune clé de tri, aucun trait public, aucun `unsafe`, aucune allocation dans le moteur. Aucun commit par la session ; elle propose un regroupement. Aucun `--skip` ou `ignore` sans décision. Aucun travail de visualisation. Aucun test hors liste ; aucune extension de périmètre (lancement depuis un système, retrait d'instance, deuxième script, lot mixte : **non démontrés**, voir plan § 10).

## 13. Conditions d'arrêt (rapport immédiat, aucun contournement)

Fait [Vérifié] qui diverge ; contradiction plan / document d'exécution ; besoin de modifier un fichier interdit ou `mod.rs` ; besoin d'`unsafe`, d'allocation, d'état hors `World`, d'un ordre ou d'une clé (G-ST, G-ORD-S) ; échec de compilation de durée de vie de la fermeture d'infrastructure ou du montage (G-LT : message exact, aucune substitution de forme) ; situation G-CM ; `match` exhaustif cassé ; mutation non détectée non équivalente, ou échouant sur d'autres assertions sans explication par le code ; code d'erreur de compilation épinglé ; octet de `ARTIFACT_A` divergent ; durée Miri excessive ; erreur de `load` de DM-2 différente de celle annoncée (exposer la variante réelle avant de l'épingler). À l'arrêt : état exact, cause, options minimales, aucune modification d'office.

## 14. Pièces à fournir à la session

**Nécessaires** : ce handoff ; plan r1.2 ; document d'exécution ; rapport U3 ; `tree.md` à jour ; `crates/engine/shell/tests/{script_support/mod.rs,script_suspend.rs}` **tels que rendus** (copies dépôt réel) ; `crates/engine/shell/src/{lib,shell,inbox,pacing}.rs` (réels) ; `crates/engine/shell/tests/level_loaded.rs` et `examples/level_headless.rs` (modèle d'`execute` sous `run_loop`, journal, présentation) ; `crates/engine/shell/Cargo.toml`, `Cargo.toml` et `Cargo.lock` racine. **Sources déjà utilisées en U4 (à redonner, sans les relire en entier)** : `ecs/src/{entity,world,command,component_set,resource,storage,values,lib}.rs` + manifeste, `schedule/src/**` (access, system, scheduler, contract, data, model) + manifeste, `assets/src/{load,manifest,format,script,lib}.rs`, `forge/src/{lib,batch,script,sprite}.rs`, `extensions/script/**`. Les modules étrangers au périmètre peuvent rester stubés selon le § 8. [À vérifier] : formes exactes de `run_loop`, `Inbox`, `FixedStepPacing` (connues seulement par leurs usages ; `shell/src/{shell,inbox,pacing}.rs` jamais lus par la session d'U4).

## 15. Points de suivi hors périmètre U5 (aucune modification)

**A. Stratification Engine / Extensions.** `verbum-script` en `[dev-dependencies]` de `verbum-shell` est provisoirement conservée. L'ADR à venir examinera : le graphe de production (qui doit respecter la stratification) ; le graphe de développement (dont les dépendances peuvent éventuellement traverser les strates) ; la responsabilité et l'emplacement des tests d'intégration moteur ↔ extension. Constat établi en U4 : `script_support/mod.rs` et `script_suspend.rs` n'importent **aucun item de `verbum-shell`** ; ils composent `World`, `Scheduler`, assets, Forge et `verbum-script` ; leur emplacement `engine/shell/tests` tient au plan, au précédent (`borrowed_resource.rs`, `level_loaded.rs`) et aux dev-dépendances déjà présentes. À l'inverse, **`script_loaded.rs` (U5) utilisera réellement l'API du Shell** (`run_loop`, `Inbox`). Aucun déplacement de test, aucune modification de manifeste en U5.

**B. Miri et `p3_1_full_chain_on_disk_with_the_source_deleted_before_reading`.** Ce test Forge échoue sous Miri isolé (`mkdir` interdit par l'isolation ; aucun signal de problème mémoire). Piste conservée, décision et traitement **reportés** : exclusion explicite `#[cfg_attr(miri, ignore = "accès disque (mkdir) refusé par l'isolation Miri")]` limitée à ce test (option A), ou exécution séparée sous `MIRIFLAGS=-Zmiri-disable-isolation` distinguée de la certification isolée (option B). En U5 : ne pas modifier ce test ; appliquer la commande Miri forge du § 7 (qui exclut déjà `p3_1`). Le test p3_1 figure déjà parmi les exclusions de la commande Miri Forge prévue par le document d'exécution. L'échec rapporté correspond à son exécution sous isolation, mais ne nécessite donc pas de modifier la commande de certification prévue.
