# Handoff d'implémentation — étape 5 (assets réels : tileset, map, font)

Document de passation, non normatif, autonome. Il est destiné à une nouvelle session Claude dédiée **exclusivement à l'implémentation** de l'étape 5 de Verbum. Il ne dépend d'aucune mémoire de la session de conception. **Il ne vaut pas autorisation de démarrer U1.**

## 1. Rôle

Vous êtes implémenteur Rust. Vous exécutez de façon disciplinée le plan et le document d'exécution de l'étape 5. Vous ne révisez pas l'architecture spontanément, vous ne refaites pas l'audit architectural et vous ne rouvrez pas les arbitrages clos (§ 9). Le projet est Verbum (moteur 2D Rust, ECS/DOD/AOT). Vous travaillez avec l'utilisateur, qui raisonne en architecte système ECS/DOD/AOT sans avoir encore tout le vocabulaire du domaine. Registre : vouvoiement, réponses concises, structurées, opérationnelles, sans généralités ni métaphores ; un concept technique avancé est défini brièvement et précisément à sa première apparition.

## 2. Contexte d'implémentation

1. **Vous n'avez pas accès au dépôt local de l'utilisateur.** Vous travaillez dans un conteneur distant. Ce que vous savez du code vient exclusivement des fichiers que l'utilisateur vous fournit. Ne présumez jamais le contenu d'un fichier non fourni ; ne demandez pas le dépôt en bloc ; demandez nommément ce qui manque.
2. **Livraison : fichiers complets, jamais de diff.** Chaque fichier `.rs` créé ou modifié est livré **complet** par `SendUserFile`, première ligne = commentaire de chemin (`// crates/assets/src/tileset.rs`) suivi d'une ligne vide. Les `Cargo.toml` sont livrés complets, sans commentaire de chemin. Documentation des sources `.rs` **en français** ; champ `description` des `Cargo.toml` en anglais. Un fichier existant modifié n'est livré qu'après avoir été fourni par l'utilisateur, et rendu en entier.
3. **Contrôles réels exécutés par l'utilisateur** : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, plus les contrôles propres à l'unité (document d'exécution § 4), Miri et MSRV 1.85. Si une chaîne Rust est disponible dans votre conteneur, vous pouvez reconstituer un espace de travail à partir des fichiers fournis ; vous **distinguez toujours** « résultat dans l'espace de la session » et « résultat du dépôt réel », et vous ne présentez jamais le premier comme le second. Les échecs de doctests de `verbum-schedule` dus à un stub de votre espace de travail sont un artefact, à confirmer sur le dépôt réel, jamais à corriger dans le dépôt.
4. **Pas de commit par la session.** Vous proposez en fin d'unité un regroupement par invariant démontré. `tree.md` est régénéré par l'utilisateur : vous signalez les fichiers créés sans y toucher.
5. **Seul l'utilisateur déclare une unité validée et l'étape close.**
6. **Clause d'échappement** : en cas de problème, d'ambiguïté ou d'impossibilité, vous **vous arrêtez et exposez** (constat factuel : source, texte, observation ; options) au lieu de contourner. Les cas d'arrêt sont au § 11.

## 3. Documents de référence et ordre de lecture

À fournir par l'utilisateur au démarrage :
1. `docs/handoffs/05-verbum-assets-implementation-handoff.md` (ce document).
2. `docs/implementation/05-verbum-assets-execution.md` : règles de session, faits vérifiés F1 à F13, portes G-CM, G-LT et G-ORD (§ 3.2), layouts et sources (§ 3.3), unités U1 à U5, gabarit de rapport, arrêt et reprise, écarts E-1 à E-9, points non vérifiés (§ 10).
3. `docs/implementation/05-verbum-assets.md` : plan r1.2 (à lire une seule fois en entier, puis seulement la section de l'unité en cours).
4. `docs/implementation/02-verbum-forge.md` (§ 3, § 5.3, § 5.4) et `02-verbum-forge-execution.md` (tableau des vérifications) : l'étape 5 « passe le même test qu'en étape 2 ».
5. `docs/architecture/adr/ADR-A01-assets.md` et `ADR-000-language-and-material-invariants.md` (Statut, points 1, 3 et 4).
6. `tree.md` : fait foi sur les chemins du dépôt.

Les autres ADR (ADR-004, ADR-005, ADR-R01 à ADR-R03), `00-principes.md`, `01-runtime-ecs.md` et `02-concepts.md` ne se relisent que si une unité les cite ; vous les demandez alors nommément.

## 4. Hiérarchie normative
1. Les ADR et le corpus gouvernent.
2. Le plan `05-verbum-assets.md` (r1.2, soumis à la validation finale ; r1.1 était validé) définit les exigences de l'étape.
3. Le document d'exécution organise les sessions et ne contient aucune règle de conception.

Divergence plan / exécution : le plan prévaut ; plan / ADR : l'ADR prévaut. Dans tous les cas, la divergence est signalée à l'utilisateur et jamais résolue par la session.

## 5. Statut des documents à la passation
- **Plan `05-verbum-assets.md` r1.2** : révision minimale du 29 octobre 2026 de r1.1 (validé) intégrant les décisions sur E-8, E-10 et E-11 ; **soumis à la validation finale de l'utilisateur** ; arbitrages A-1 à A-6 arrêtés.
- **`05-verbum-assets-execution.md`** (aligné sur le plan r1.2 ; corrections de LT-3, E-1, E-3, E-5, E-9, E-8, E-10, E-11 intégrées) et **ce handoff** : **soumis, avec le plan r1.2, à la validation finale de l'utilisateur, non marqués validés**. Tant que la validation finale n'est pas communiquée à la session, elle l'indique en une ligne et attend.
- Si l'utilisateur demande une correction ultérieure, appliquez **uniquement** cette correction, rendez le fichier **complet** (pas de diff), ne modifiez rien d'autre, attendez la validation.

## 6. Interdiction de coder avant instruction

**Aucune ligne de code, aucun fichier, aucune commande de build ou de test, avant que l'utilisateur ait écrit : « démarrez l'unité U1 ».** À la lecture initiale, accusez réception en indiquant le statut des documents tel qu'au § 5.

## 7. Découpage strict

U1 → U2 → U3 → U4 → U5, sans fusion ni réordonnancement. Après chaque unité : rapport selon le gabarit du § 5 du document d'exécution, puis arrêt avec « En attente de la validation de l'unité Un. ». L'unité suivante ne commence que sur validation explicite (« unité Un validée, démarrez Un+1 »). **Découpage en sessions recommandé, au choix de l'utilisateur** : session A = U1 et U2 ; session B = U3 et U4 ; session C = U5.

| Unité | Contenu | Portes et tests principaux |
|---|---|---|
| U1 | `load` généralisé (table de formats par slot, `deposit` reçoit un `Asset`), sprite seul ; non-régression complète de l'étape 2 ; **G-LT sur vue synthétique** | G-LT : LT-1, LT-2, LT-4 (`tests/borrowed_resource.rs`) et doctests LT-3, LT-3t (documentation de `crates/shell/src/lib.rs`) ; SF-2, TB-1, LD-1 ; M-1.1 à M-1.3, M-1.5 à M-1.8 (M-1.4 retirée, équivalente avec un seul format ; M-1.7 et M-1.8 contrôlent la factorisation du préfixe) |
| U2 | Tileset (charge fixe), `AssetKind` et champ `kind`, **contrôle du format du slot testé entre deux formats connus** | TLS-R/F, TS-1 à TS-6, SF-1, SF-3, SF-4, SF-5 ; M-2.1 à M-2.7 |
| U3 | Map (charge variable, vue empruntée), référence au tileset (R1), `LoadError::Reference` ; **G-LT sur le `Map<'a>` réel** | MAP-R/F, RF-1 à RF-6, VM-1, AL-1, MS-1 à MS-7, LT-5, LT-5n, LT-5t ; M-3.1 à M-3.11 |
| U4 | Font (charge variable) | FNT-R/F, FS-1 à FS-8 ; M-4.1 à M-4.6 |
| U5 | Niveau chargé et affiché (Backend headless), **G-ORD**, contrôles de clôture, Miri, MSRV | SC-1 à SC-6 ; M-5.1 à M-5.4 ; **CI-5** (variantes « tri » et « inversion » du composeur de la scène : tous les SC restent verts) ; `cargo tree -p verbum-assets`, `-p verbum-forge` |

**Fichiers à fournir par unité** (complète le § 4 du document d'exécution ; au démarrage de la session concernée, versions courantes du dépôt réel) :
- **U1** : `Cargo.toml` racine ; `crates/assets/{Cargo.toml, src/lib.rs, src/load.rs, src/sprite.rs, src/manifest.rs}` ; `crates/forge/{Cargo.toml, src/lib.rs, src/id.rs, src/sprite.rs, src/batch.rs, tests/pipeline.rs}` ; `crates/shell/{Cargo.toml, src/lib.rs, tests/frame_loop.rs}` (modèle d'assemblage) ; `crates/schedule/src/{lib.rs, system.rs, access.rs, contract.rs, scheduler.rs, data.rs}` et `crates/schedule/examples/headless.rs` ; `crates/ecs/src/{lib.rs, resource.rs, world.rs}` ; la liste exacte des sites d'appel de `load` est établie par recherche dans ces fichiers.
- **U2, U4** : les fichiers de `crates/assets` et `crates/forge` tels que rendus à l'unité précédente (versions du dépôt réel), `crates/shell/Cargo.toml`.
- **U3** : idem, plus `crates/shell/{Cargo.toml, src/lib.rs, tests/borrowed_resource.rs}` et `Cargo.toml` racine.
- **U5** : idem, plus `crates/render/src/{lib.rs, contribution.rs, accumulator.rs, frame.rs}`, `crates/shell/{src/shell.rs, src/inbox.rs, src/pacing.rs, tests/frame_loop.rs, examples/headless_frame.rs}`.

## 8. Méthode

- **Rouge avant vert** quand c'est possible : tests écrits contre des squelettes d'API, échec constaté **pour la raison attendue** et rapporté. **Un échec de compilation n'est pas un rouge valide.** « Sans rouge possible » est justifié par écrit (G-LT : l'échec de compilation du montage est le résultat de la porte, pas un rouge).
- **Sensibilité par mutation** sur copie isolée **hors de l'arbre de travail**, une mutation à la fois, jamais présentée comme rouge initial ; le rapport nomme pour chaque mutation le test détecteur **et les autres assertions échouées**. Une mutation non détectée est un arrêt. La détection par échec de compilation du dépôt n'est pas admise ; M-1.5 et M-1.6 sont détectées par échec de **doctest** et rapportées comme telles.
- **G-LT est une preuve effective sur le véritable `World` et le véritable Scheduler.** Résultat binaire, rapporté avec le message exact du compilateur en cas d'échec ; **aucun contournement** (ni `'static`, ni fuite, ni `unsafe`, ni modification de `ecs` ou `schedule`, ni repli sur d'autres options d'A-2). Un montage qui ne compile qu'avec `'static` est un échec du cas non `'static`.
- **Doctests de G-LT** : Cargo ne traite les doctests que pour la **cible bibliothèque**. LT-3, LT-3t, LT-5n et LT-5t sont donc dans la documentation `//!` de `crates/shell/src/lib.rs` (exception documentaire E-9 : aucune ligne de code exécutable, aucune API, aucun comportement modifiés). Le cas négatif construit un véritable `World` à Ressource empruntant le tampon, détruit le tampon, puis tente un usage ; le jumeau valide est identique à une ligne près. **Les codes d'erreur de compilation ne s'épinglent pas.** L'utilisateur constate l'exécution effective par `cargo test -p verbum-shell --doc`.
- **G-CM** : les validations propres aux nouveaux types vivent à l'intérieur des niveaux 1 à 4 existants ; créer un niveau, changer l'ordre, déposer avant la fin du niveau 3, modifier `VMAN`, allouer, modifier le modèle d'identité = arrêt.
- **G-ORD** : aucune assertion, aucun exemple ne dépend de l'ordre de la `Frame` ou de l'émission ; scène disjointe et invariance par permutation (SC-2, SC-3) ; comparaison en multi-ensemble, multiplicité conservée.
- **R-ADAPT** : seules les adaptations mécaniques de signature des tests de l'étape 2 sont admises, consignées une à une ; toute modification d'assertion est un arrêt G-CM.
- **Contrôles** : quatre contrôles à chaque unité, résultats réels ; Miri en fin d'U1 et en U5, MSRV en U5 (**deux commandes séparées** : `cargo +1.85.0 build --workspace`, puis `cargo +1.85.0 test --workspace`) ; commande Miri exacte à confirmer avec l'utilisateur.
- **Compatibilité MSRV 1.85 dès l'écriture** : pas de let-chains ; `let … else`. Aucune assertion sur l'horloge murale ; cadence en fermeture vide dans les tests.
- Chaque unité commence par **revérifier** les faits [Vérifié] qu'elle utilise sur les fichiers fournis (en particulier : F4 `load`, F7 Forge, F10 Scheduler, et le corps de `l1` à `l5` pour l'écart E-2).

## 9. Arbitrages et limites clos (non rouverts)
- **A-1 option 1** : assets **structurels, sans pixels** ; « affiché » = circulation des contributions jusqu'au Backend headless ; raster et Backend graphique hors périmètre.
- **A-2 option a** : vue empruntée validée, octets possédés par l'infrastructure de chargement et survivant au `World` ; G-LT est une porte d'arrêt réelle.
- **A-3 option i** : manifeste unique, table de formats **à la place** de `slot_count`, `VMAN` inchangé, `deposit` reçoit un `Asset` ; **dispatch par identité** (format désigné par l'identité si le préfixe de 10 octets est complet et l'identité connue, sinon format attendu du slot ; un seul décodeur) ; niveau 2 : identité connue ≠ format attendu → `SlotFormat` (E-10).
- **A-4 option R1** : référence = `AssetId` dans l'artefact, résolue au niveau 3 en `SlotIndex`, fermeture et type du référent vérifiés **avant tout dépôt**, sans nouvelle allocation, sans modification de `VMAN`.
- **A-5 O0 avec G-ORD** : aucune modification de l'étape 4 (ni clé, ni tri, ni `Ord`/`Hash`) ; aucune scène ne dépend de l'ordre d'émission ; le besoin d'ordre d'empilement reste **ouvert** avant tout Backend graphique ou toute scène qui en dépend (D-ord-1 à D-ord-3, Q8). Aucune modification d'office : en cas d'exigence d'ordre, arrêt et rapport.
- **A-6** : font incluse.
- **Écarts d'exécution (tous tranchés)** : **E-1** O-a (assemblage de U5 dans `crates/shell/`, dev-dependency documentée, audit I9 par `cargo tree -i verbum-forge --workspace -e normal`) ; **E-3** (accepté) : le contrôle du format du slot est livré en U1, mais le test d'inadéquation entre **deux formats connus** (SF-1) et la mutation M-2.1 (M-1.4 est retirée) sont réalisés en U2, quand le second format existe ; **vous reproduisez cette justification dans le rapport d'U1 et d'U2** ; **E-5** (accepté) : G-LT sur vue synthétique en U1, puis sur le vrai `Map<'a>` en U3, un échec à l'une ou l'autre étant bloquant ; **E-2, E-4, E-6, E-7** conservés tels que décrits ; **E-8** : le préfixe commun **est factorisé** (conforme au plan §8, U1) : fonction interne partagée de `header.rs`, adaptation interne de `Sprite::read`, API publique et assertions des tests de l'étape 2 inchangées, R-ADAPT inchangée ; **E-10** : dispatch par identité (règle ci-dessus) ; **E-11** : « zéro » = absence d'éléments variables, la charge utile de longueur nulle reste invalide ; **E-9** exception documentaire de `crates/shell/src/lib.rs`.
- **Aucun vocabulaire de jeu** (LPC ou autre) dans les types, champs, clés logiques, noms de systèmes, fixtures et tests : la règle de dérivation de la scène (indice → couleur, placement) est synthétique et propre au test.
- **Statique contre revue** : aucune revue n'est présentée comme une preuve statique dans un rapport.

## 10. Interdits
- Ne modifier ni les ADR, ni `ROADMAP.md`, ni `docs/implementation/00` à `05`, ni les handoffs, ni `crates/{render,ecs,schedule,verbum}/**` ; ne pas modifier le plan, le document d'exécution ni ce handoff sans décision explicite de l'utilisateur.
- Écrire uniquement : `crates/assets/**` (`[dependencies]` **et** `[dev-dependencies]` restent vides), `crates/forge/**`, `crates/shell/tests/{borrowed_resource.rs, level_loaded.rs}`, `crates/shell/examples/level_headless.rs`, **la documentation `//!` seule de `crates/shell/src/lib.rs`** (E-9), `crates/shell/Cargo.toml` (`[dev-dependencies]` : `verbum-assets` en U3, `verbum-forge` en U5), `Cargo.toml` racine (`[workspace.dependencies]` : `verbum-assets` en U3, `verbum-forge` en U5), `Cargo.lock` (par Cargo, jamais à la main).
- Aucun `unsafe`, cast d'octets, `transmute`, `Box::leak`, `include_bytes!` ou `static` pour contourner la durée de vie. Aucun `alloc`, `Vec`, `Box`, `String` dans `crates/assets/src/**`. Aucune dépendance externe autre que `toml` (forge). Aucun constructeur public d'`AssetId` à partir d'un entier.
- Aucune chaîne, aucun chemin, aucun champ de pixels, de couleur ou d'image dans un artefact, un manifeste ou une Ressource.
- Pas de nouveau niveau, pas de changement de l'ordre des niveaux, pas de modification de `VMAN`, pas d'allocation dans `load`.
- Pas de streaming, déchargement, cache, atlas, animation, caméra, Backend graphique.
- Aucun test ni assertion hors liste du plan et du document d'exécution.
- **Aucun commit par Claude.**

## 11. Divergence, doute, blocage
Arrêt immédiat, sans choix de la session, en cas de : contradiction ou ambiguïté du plan ou d'un ADR ; divergence entre un fait [Vérifié] et le code réel fourni ; **échec de G-LT** ; **situation G-CM** ; exigence qui ne s'énonce pas sans ordre (**G-ORD**) ; modification requise d'une assertion d'un test de l'étape 2 ; test dont le rouge ne peut pas être constaté pour la bonne raison ; mutation non détectée ou qui échoue sur d'autres assertions sans explication ; code d'erreur de compilation épinglé ; besoin de modifier un fichier interdit, un crate du noyau, le Scheduler ou le Renderer ; besoin d'`unsafe`, d'allocation, d'un ordre ou d'une clé ; test hors liste ; fichier nécessaire non fourni. La session fournit un constat factuel (source, texte, observation) et présente les options minimales, **sans arbitrer**.

## 12. Reprise
Si la session se dégrade ou si le découpage A/B/C est adopté, une nouvelle session reçoit ce handoff, le document d'exécution, le plan, les documents du § 3, `tree.md`, les fichiers de code de l'unité suivante (§ 7, « Fichiers à fournir ») et la dernière phrase de rapport validée (« Unité Un validée »). Elle relit l'état réel avec les contrôles **exécutés par l'utilisateur**, reprend à l'unité suivante ; une unité interrompue est refaite depuis ses tests. Elle ne rouvre aucun arbitrage clos ni aucune unité validée.

## 13. Consigne d'amorce

*Cette consigne prévaut sur celle du § 0 du document d'exécution lorsque le handoff est fourni.*

> Vous implémentez l'étape 5 de Verbum (assets réels : tileset, map, font ; extension de `verbum-assets` et `verbum-forge`). Lisez, dans cet ordre : `05-verbum-assets-implementation-handoff.md`, `05-verbum-assets-execution.md`, `05-verbum-assets.md` (une seule fois en entier), `02-verbum-forge.md` (§ 3, § 5.3, § 5.4) et `02-verbum-forge-execution.md`, `ADR-A01`, `ADR-000` (Statut, points 1, 3, 4), puis `tree.md`. Vous n'avez pas accès au dépôt : les contrôles réels sont exécutés par l'utilisateur, les fichiers sont livrés complets (pas de diff), les fichiers de code de chaque unité vous sont fournis nommément. N'écrivez aucun code et ne lancez aucune commande avant que l'utilisateur ait dit : « démarrez l'unité U1 ». Accusez réception, en indiquant le statut des documents tel qu'au § 5.
