# Handoff d'implémentation — étape 4 (`verbum-render`)

Document de passation, non normatif, autonome. Il est destiné à une nouvelle session Claude dédiée **exclusivement à l'implémentation** de l'étape 4 de Verbum. Il ne dépend d'aucune mémoire de la session de conception. **Il ne vaut pas autorisation de démarrer U1.**

## 1. Rôle

Vous êtes implémenteur Rust. Vous exécutez de façon disciplinée le plan et le document d'exécution de l'étape 4. Vous ne révisez pas l'architecture spontanément, vous ne refaites pas l'audit architectural et vous ne rouvrez pas les arbitrages clos (§ 9). Le projet est Verbum (moteur 2D Rust, ECS/DOD/AOT). Vous travaillez avec l'utilisateur, qui raisonne en architecte système ECS/DOD/AOT sans avoir encore tout le vocabulaire du domaine. Registre : vouvoiement, réponses concises, structurées, opérationnelles, sans généralités ni métaphores ; un concept technique avancé est défini brièvement et précisément à sa première apparition.

## 2. Contexte d'implémentation (identique à celui de la session de conception)

1. **Vous n'avez pas accès au dépôt local de l'utilisateur.** Vous travaillez dans un conteneur distant. Ce que vous savez du code vient exclusivement des fichiers que l'utilisateur vous fournit, et de rien d'autre. Ne présumez jamais le contenu d'un fichier non fourni ; ne demandez pas le dépôt en bloc.
2. **Liste des fichiers à fournir** : le § 3 donne les documents de départ ; le document d'exécution (§ 4, champ « À fournir » de chaque unité) donne, unité par unité, les sources de code à vous remettre. Si un fichier nécessaire manque, demandez-le nommément avant d'écrire.
3. **Livraison : fichiers complets, jamais de diff.** Chaque fichier `.rs` créé ou modifié est livré **complet** par `SendUserFile`, première ligne = commentaire de chemin (`// crates/render/src/frame.rs`) suivi d'une ligne vide. Les `Cargo.toml` sont livrés complets, sans commentaire de chemin. Aucun fragment, aucun patch, aucune « modification à appliquer ». Documentation des sources `.rs` **en français** ; champ `description` des `Cargo.toml` en anglais (usage du workspace). Un fichier existant modifié (`Cargo.toml` racine, `crates/shell/Cargo.toml`) n'est livré qu'après avoir été fourni par l'utilisateur, et rendu en entier.
4. **Contrôles réels exécutés par l'utilisateur.** Le dépôt réel n'est pas accessible : les quatre contrôles (`cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`), Miri et MSRV 1.85 sont **exécutés par l'utilisateur** sur le dépôt réel. Si une chaîne Rust est disponible dans votre conteneur, vous pouvez reconstituer un espace de travail à partir des fichiers fournis et y exécuter les contrôles ; vous **distinguez alors toujours** « résultat dans l'espace de la session » et « résultat du dépôt réel », et vous ne présentez jamais le premier comme le second. Les commandes à lancer sont données à l'utilisateur de façon exacte, une commande par ligne. Vous attendez ses résultats ; aucun résultat n'est supposé, déduit ni estimé.
5. **Pas de commit par la session.** Vous proposez en fin d'unité un regroupement par invariant démontré ; l'utilisateur commite. `tree.md` est régénéré par l'utilisateur : vous signalez les fichiers créés sans y toucher.
6. **Seul l'utilisateur déclare une unité validée et l'étape close.** Vous ne le déclarez jamais.
7. **Clause d'échappement** : en cas de problème, d'ambiguïté ou d'impossibilité, vous **vous arrêtez et exposez** (constat factuel : source, texte, observation ; options) au lieu de contourner. Les cas d'arrêt sont au § 11.

## 3. Documents de référence et ordre de lecture

À fournir par l'utilisateur au démarrage :
1. `docs/handoffs/04-verbum-renderer-implementation-handoff.md` (ce document).
2. `docs/implementation/04-verbum-renderer-execution.md` : règles de session, faits vérifiés F1 à F11, borne de production (§ 3.2), garanties statiques et revues (§ 3.3), unités U1 à U5, gabarit de rapport, arrêt et reprise, écarts E-1 à E-5.
3. `docs/implementation/04-verbum-renderer.md` : plan r3.2 (à lire une seule fois en entier, puis seulement la section de l'unité en cours).
4. `docs/architecture/adr/ADR-R02-pipeline-production.md`, `ADR-R03-backend.md` et `ADR-000-language-and-material-invariants.md` (Statut, points 1, 3 et 4).
5. `tree.md` : fait foi sur les chemins du dépôt.

Les autres ADR (ADR-R01, ADR-S01, ADR-S03, ADR-003, ADR-004, ADR-005), `00-principes.md`, `01-runtime-ecs.md` et `02-concepts.md` ne se relisent que si une unité les cite ; vous les demandez alors nommément. Ne lisez pas de spécification étrangère à l'étape.

## 4. Hiérarchie normative

1. Les ADR et le corpus gouvernent.
2. Le plan `04-verbum-renderer.md` (r3.2) définit les exigences de l'étape.
3. Le document d'exécution organise les sessions et ne contient aucune règle de conception.

Divergence plan / exécution : le plan prévaut ; plan / ADR : l'ADR prévaut. Dans tous les cas, la divergence est signalée à l'utilisateur et jamais résolue par la session.

## 5. Statut des documents à la passation

- **Plan `04-verbum-renderer.md` r3.2**, **`04-verbum-renderer-execution.md`** (aligné sur r3.2) et **ce handoff** : livraison corrective du 28 octobre 2026 issue d'un audit de cohérence (dix corrections rédactionnelles, aucune modification de conception ni d'arbitrage), **soumise à la validation finale de l'utilisateur, non marquée validée**. Les arbitrages sont ceux validés le 28 octobre 2026 (plan r2, puis r3 pour C4). Tant que la validation finale n'est pas communiquée à la session, elle l'indique en une ligne et attend. **Révision r3.2 (28 octobre 2026)** : minimale, limitée à RP-5 (écart E-7 du document d'exécution : codes d'erreur des doctests `compile_fail` non épinglables sur Rust stable) ; soumise à validation.
- **Ce handoff** : soumis à validation avec la livraison ; la validation de l'ensemble ne vaut pas autorisation de démarrer U1.
- Si l'utilisateur demande une correction ultérieure, appliquez **uniquement** cette correction, rendez le fichier **complet** (pas de diff), ne modifiez rien d'autre, attendez la validation.

## 6. Interdiction de coder avant instruction

**Aucune ligne de code, aucun fichier, aucune commande de build ou de test, avant que l'utilisateur ait écrit : « démarrez l'unité U1 ».** À la lecture initiale, accusez réception en indiquant le statut des documents tel qu'au § 5.

## 7. Découpage strict

U1 → U2 → U3 → U4 → U5, sans fusion ni réordonnancement. Après chaque unité : rapport selon le gabarit du § 5 du document d'exécution, puis arrêt avec « En attente de la validation de l'unité Un. ». L'unité suivante ne commence que sur validation explicite (« unité Un validée, démarrez Un+1 »). **Découpage en sessions recommandé, au choix de l'utilisateur** : session A = U1 à U3 (types et pipeline, sans Shell) ; session B = U4 et U5.

| Unité | Contenu | Tests |
|---|---|---|
| U1 | Crate `verbum-render` vide ; racine : `members` + `[workspace.dependencies]` | aucun comportement ; `cargo tree` |
| U2 | `Contribution`, `Contributions<CAP>`, `Full`, `Frame<CAP>` | RP-1 à RP-7 ; M-R3, M-R4, M-R1, M-R11, mutateur ajouté à `Frame` |
| U3 | Pipeline sous Scheduler (`Clear`, `ProduceA`, `ProduceB`, `Compose`, `Witness`) et contrôle de composition | RN-1 à RN-6 ; M-R8, M-R9, M-R2, M-R1 et M-R11 réexercées, mutations de contrôle de composition |
| U4 | Acceptation sous `run_loop` du Shell, instruments Backend headless, dev-dépendance | RA-1 à RA-5 ; M-R5, M-R6, M-R7, M-R10 |
| U5 | Exemple `headless_frame`, contrôles de clôture, Miri, MSRV | exemple (code 0) ; contrôles `cargo tree` |

## 8. Méthode

- **Rouge avant vert** quand c'est possible : tests écrits contre des squelettes d'API (signatures, corps `unimplemented!()`), échec constaté **pour la raison attendue** (panique) et rapporté (test, motif observé). **Un échec de compilation n'est pas un rouge valide.**
- **Sans rouge possible** : RP-5 (doctests `compile_fail`), RN-2 (contrôle de contrats déjà typés), RA-1 à RA-5 (API déjà implémentées) : vert au premier passage, puis **sensibilité par mutation** sur copie isolée **hors de l'arbre de travail**, une mutation à la fois, jamais présentée comme rouge initial. Le rapport nomme, pour chaque mutation, le test détecteur **et les autres assertions échouées**. Une mutation non détectée est un arrêt. M-R4 est détectée par échec de compilation de RP-7 et rapportée comme telle ; M-R2 est détectée par RN-5 via le `panic!` du producteur (§ 3.2 du document d'exécution), rapportée comme telle.
- **Quatre contrôles à chaque unité**, résultats réels (§ 2, point 4).
- **Miri et MSRV 1.85 : réservés à U5, exécutés par l'utilisateur.** Commandes exactes : `cargo +nightly miri test -p verbum-render`, puis `cargo +nightly miri test -p verbum-shell` ; **deux commandes séparées pour MSRV** : `cargo +1.85.0 build --workspace`, puis `cargo +1.85.0 test --workspace`. Une seule ligne combinant les deux est une erreur déjà commise à l'étape 3.
- **Compatibilité MSRV 1.85 dès l'écriture** : pas de chaînes `if let … && …` (let-chains, stables à partir de 1.88) ; `let … else` à la place. **Aucune assertion sur l'horloge murale** dans les tests (échec sous Miri à l'étape 3) : les tests RA-* utilisent une cadence en fermeture vide ; seul l'exemple utilise `FixedStepPacing`.
- **Doctests `compile_fail`** : blocs **sans code d'erreur** (non contrôlé par rustdoc sur Rust stable) ; la sensibilité est établie par mutation (RP-5 (b) sur une `Frame` possédée et mutable). Les codes observés par compilation directe sont un constat ponctuel, non une assertion de test.
- Chaque unité commence par **revérifier** les faits [Vérifié] qu'elle utilise sur les fichiers fournis. En U4, **F10 (Shell) est à revérifier** : il provient de l'étape 3 et n'a pas été relu pendant la conception.

## 9. Arbitrages et limites clos (non rouverts)

- **A-1 = Y** : deux producteurs indépendants ; assertions sur la Frame comme **multiensemble**, **multiplicité conservée** (un doublon `Q1` est voulu), **aucune assertion d'ordre de séquence** (C8). Pas de tri, de clé, de couche, de regroupement. **Ni `Ord`, ni `PartialOrd`, ni `Hash`** pour `Contribution`, `Contributions`, `Frame` ; les tests trient avec une fonction de clé locale.
- **A-2 = forme 2** (choix d'implémentation propre à l'étape 4, ADR-000 pt 6, sans amendement d'ADR) : le code d'assemblage lit la Ressource `Frame` après le retour de `run_tick` et la transmet au Backend par `&Frame`. **Aucun trait public `Backend`** ni autre trait d'interface ; le Backend est une fermeture `FnMut(&Frame<CAP>)` ou une méthode d'instrument de test. Aucune dépendance de `verbum-render` vers le Shell, même en dev.
- **A-3 = (i)** : types dans le crate `crates/render` (`verbum-render`, `#![no_std]`, sans `alloc`, **aucune dépendance normale**, dev-dépendances `verbum-ecs` et `verbum-schedule`) ; acceptation et exemple dans `crates/shell`. Motif de dépendance : entrée `verbum-render` dans `[workspace.dependencies]` puis `verbum-render.workspace = true` en dev-dépendance de `verbum-shell`.
- **C3** : Frame déclarée `produces` ; unicité du composeur **non garantie par les types** ; contrôle de composition RN-2 et revue.
- **C4 (corrigé en r3, conservé en r3.1 et r3.2 ; E-1 et E-5)** : le composeur **lit** l'accumulateur et produit la Frame ; il ne le vide pas (lire et produire la même catégorie dans un accès est une erreur E0080). Un système `Clear` en **phase 0** remet l'accumulateur à l'état initial ; phase 1 : `ProduceA`, `ProduceB`, `Compose`, `Witness`. **Ordre de déclaration obligatoire** : `Clear`, `Compose`, `Witness`, `ProduceA`, `ProduceB` (indices 0 à 4) : `Compose` est déclaré **avant** ses producteurs pour que l'ordre exécuté ne puisse venir que de la déduction (cet ordre de déclaration n'est pas l'ordre d'exécution). **Interprétation de durée de vie propre à l'étape 4, sans amendement d'ADR-R01** : la durée de vie logique d'une contribution prend fin à sa consommation par `Compose` ; l'ancienne valeur physique n'est pas une contribution vivante si aucun système autorisé ne peut la relire et si l'état est remis à l'état initial avant les lectures déclarées. **Deux limites à rappeler aux rapports de U3, U4, U5** : le contrôle de composition ne porte que sur les systèmes assemblés ; l'absence de lecture de `ACC` hors systèmes (assemblage, fermeture, instruments, observateurs) après `Compose` relève de la **revue de l'assemblage de référence** ; **les instantanés de U4 ne contiennent pas `Contributions`**. Aucune de ces garanties n'est imposée par les types ni par les contrats.
- **C5** : « fin de la phase d'Extraction » ≡ « fin du tick », contrôlé par la position de `Compose` en dernière phase (RN-2 (d)).
- **C7 (ouvert)** : la réaction générale d'un système à la saturation reste non résolue. Pour le scénario seulement : `CAP = 4`, 3 contributions par tick, `const` assert `MAX_PAR_TICK <= CAP`, `panic!` explicite des producteurs sur `Err(Full)` ou emplacement absent, **jamais** `BufferFull`, **jamais** ignoré. Ce `panic!` n'est pas déclenché par le scénario nominal ; il n'est exercé que par la mutation M-R2 (tick 2 : 3 contributions déjà présentes, premier `push` de `ProduceA` à `CAP`, second `Full`). Ne modifiez pas le Scheduler ; n'inventez pas de canal d'erreur général.
- **C9** : lot publié avant chaque tick, même sans lecteur, selon le motif de l'étape 3. **C12** : contribution de test carrée, pour exercer le parcours.
- **Écarts d'exécution validés** : E-2 (doctest `compile_fail` du Backend écrit en U2 avec RP-5 (c), RA-2 ajoute un test de typage en U4) ; E-3 (cadence vide dans les tests RA-*, désormais inscrite au plan § 7.3) ; E-4 (politique des producteurs ci-dessus) ; E-6 (`is_empty` ajouté pour `clippy::len_without_is_empty`, sans autre API).
- **Statique contre revue** : appliquez la table du § 3.3 du document d'exécution. Aucune revue n'est présentée comme une preuve statique dans un rapport.

## 10. Interdits

- Ne modifier ni les ADR, ni `ROADMAP.md`, ni `docs/implementation/00` à `03`, ni les handoffs, ni `crates/{ecs,schedule,assets,forge,verbum}/**`, ni **aucun fichier existant** de `crates/shell/src/**`, `crates/shell/tests/**`, `crates/shell/examples/**` ; ne pas modifier le plan, le document d'exécution ni ce handoff sans décision explicite de l'utilisateur.
- Écrire uniquement : `Cargo.toml` racine (U1 : `members` et `[workspace.dependencies]`), `crates/shell/Cargo.toml` (U4 : `[dev-dependencies]` seulement), `Cargo.lock` (par Cargo, jamais à la main), `crates/render/**`, `crates/shell/tests/frame_loop.rs`, `crates/shell/examples/headless_frame.rs`.
- Aucun `unsafe` (lint `unsafe_code = "deny"`). Aucune dépendance normale ni externe pour `verbum-render` ; aucun `extern crate alloc`, `Vec`, `Box`, `String` dans `crates/render/src/**`.
- Aucun flottant, référence, `EntityId` ni type de la Plateforme dans `Contribution` ou `Frame` ; aucune croissance de capacité ; aucune allocation sur le chemin d'une itération.
- Aucun clear-color, caméra, tri, clé de couche, règle de regroupement ; aucun test ni assertion hors liste du plan ; aucun `World::execute()`, `Scheduler::run()` ni `renderer.compose()`.
- **Aucun commit par Claude.**

## 11. Divergence, doute, blocage

Arrêt immédiat, sans choix de la session, en cas de : contradiction ou ambiguïté du plan ou d'un ADR ; divergence entre un fait [Vérifié] et le code réel fourni ; test dont le rouge ne peut pas être constaté pour la bonne raison ; mutation non détectée ou qui échoue sur d'autres assertions sans explication ; doctest `compile_fail` qui réussit alors que la propriété gardée a été supprimée (mutation non détectée) ; besoin de modifier un fichier interdit, un crate du noyau ou le Scheduler ; besoin d'`unsafe`, d'allocation, d'un trait public, d'un ordre ou d'une clé ; test hors liste ; fichier nécessaire non fourni. La session fournit un constat factuel (source, texte, observation) et présente les options, **sans arbitrer**.

## 12. Reprise

Si la session se dégrade ou si le découpage A/B est adopté, une nouvelle session reçoit ce handoff, le document d'exécution, le plan, les ADR du § 3, `tree.md`, les fichiers de code de l'unité suivante (selon « À fournir ») et la dernière phrase de rapport validée (« Unité Un validée »). Elle relit l'état réel avec les quatre contrôles **exécutés par l'utilisateur**, reprend à l'unité suivante ; une unité interrompue est refaite depuis ses tests. Elle ne rouvre aucun arbitrage clos ni aucune unité validée.

## 13. Consigne d'amorce

> Vous implémentez l'étape 4 de Verbum (Renderer minimal, crate `verbum-render`). Lisez, dans cet ordre : `04-verbum-renderer-implementation-handoff.md`, `04-verbum-renderer-execution.md`, `04-verbum-renderer.md` (une seule fois en entier), `ADR-R02`, `ADR-R03`, `ADR-000` (Statut, points 1, 3, 4), puis `tree.md`. Vous n'avez pas accès au dépôt : les contrôles réels sont exécutés par l'utilisateur, les fichiers sont livrés complets (pas de diff), les fichiers de code de chaque unité vous sont fournis nommément. N'écrivez aucun code et ne lancez aucune commande avant que l'utilisateur ait dit : « démarrez l'unité U1 ». Accusez réception, en indiquant le statut des documents tel qu'au § 5.
