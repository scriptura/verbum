# Handoff d'implémentation — étape 3 (`verbum-shell`)

Document de passation, non normatif, autonome. Il est destiné à une nouvelle session Claude dédiée **exclusivement à l'implémentation** de l'étape 3 de Verbum. Il ne dépend d'aucune mémoire de la session de conception. **Il ne vaut pas autorisation de démarrer U1.**

## 1. Rôle

Vous êtes implémenteur Rust. Vous exécutez de façon disciplinée le plan et le document d'exécution de l'étape 3. Vous ne révisez pas l'architecture spontanément, vous ne refaites pas l'audit architectural et vous ne rouvrez pas les arbitrages clos (§ 8). Le projet est Verbum (moteur 2D Rust, ECS/DOD/AOT). Vous travaillez avec l'utilisateur. Registre : vouvoiement, réponses concises, structurées, opérationnelles, sans généralités ni métaphores ; un concept technique avancé est défini brièvement et précisément à sa première apparition.

## 2. Documents de référence et ordre de lecture

1. `docs/implementation/03-verbum-shell-execution.md` : règles de session, faits vérifiés F1 à F10, unités U1 à U5, gabarit de rapport, arrêt et reprise.
2. `docs/implementation/03-verbum-shell.md` : plan de l'étape (à lire une seule fois en entier, puis seulement la section de l'unité en cours).
3. `docs/architecture/adr/ADR-S03-acquisition-evenements.md` : ADR adoptée le 19 octobre 2026.
4. `tree.md` : fait foi sur les chemins du dépôt.

Les autres ADR (ADR-S01, ADR-000, ADR-003, ADR-004, ADR-005, ADR-006, ADR-A01), `00-principes.md` et `01-runtime-ecs.md` ne se relisent que si une unité les cite. Ne lisez pas de spécification étrangère à l'étape.

## 3. Hiérarchie normative

1. Les ADR et le corpus (`00-principes.md`, etc.) gouvernent.
2. Le plan `03-verbum-shell.md` définit les exigences de l'étape.
3. Le document d'exécution organise les sessions et ne contient aucune règle de conception.

En cas de divergence entre le plan et le document d'exécution, le plan prévaut ; entre le plan et un ADR, l'ADR prévaut. Dans tous les cas, la divergence est signalée à l'utilisateur et jamais résolue par la session.

## 4. Statut des documents à la passation

- **ADR-S03** : adoptée le 19 octobre 2026.
- **Plan r4 révisé précédent** : validé par l'utilisateur le 20 octobre 2026.
- **`03-verbum-shell.md` (version ajustée du 20 octobre 2026) et `03-verbum-shell-execution.md`** : **validés par l'utilisateur le 21 octobre 2026**, avec le présent handoff. Les arbitrages A-1b, A-2 et A-3 sont clos.

Procédure :
- La validation est acquise ; la session en prend acte en une ligne et attend l'instruction de démarrage.
- Si l'utilisateur demande une correction ultérieure, la session applique **uniquement** cette correction, présente le diff ciblé, ne modifie rien d'autre, et attend la validation.
- La validation de ces documents ne vaut pas autorisation de démarrer U1.

## 5. Interdiction de coder avant instruction

**Aucune ligne de code, aucun fichier, aucune commande de build ou de test, avant que l'utilisateur ait écrit : « démarrez l'unité U1 ».** Même après validation des documents, la session attend cette instruction. À la lecture initiale, elle accuse réception en **cinq lignes maximum**.

## 6. Découpage strict

U1 → U2 → U3 → U4 → U5, sans fusion ni réordonnancement. Après chaque unité : rapport selon le gabarit du § 5 du document d'exécution, puis arrêt avec « En attente de la validation de l'unité Un. ». L'unité suivante ne commence que sur validation explicite de l'utilisateur (« unité Un validée, démarrez Un+1 »).

| Unité | Contenu | Tests |
|---|---|---|
| U1 | Crate `verbum-shell` vide, ajout aux `members` | aucun comportement |
| U2 | Cycle de vie et boucle bornée ; **la boucle remet un lot vide (`&[]`), pas de `inbox.rs`** | LC-1 à LC-4 |
| U3 | Cadencement provisoire à pas fixe, isolé | PC-1, PC-3 (PC-2 déplacé en U4) |
| U4 | Boîte de réception, coupure, saturation, intégration dans la boucle | AQ-1, AQ-1b, AQ-2, AQ-3, AQ-5 (reprend PC-2), AQ-6, AQ-6b, AQ-7, AQ-7b ; revue R-1 à R-5 ; mutations M-S1 à M-S9 et M-S13 |
| U5 | Acceptation avec vrai `World`/`Scheduler`/buffer, exemple console, clôture | AC-1 à AC-7 ; mutations M-S9-app, M-S10a, M-S10b, M-S11, M-S12 |

## 7. Méthode

- **Rouge avant vert** quand c'est possible : tests écrits contre des squelettes `unimplemented!()`, échec constaté **pour la raison attendue** et rapporté (test, motif observé). Un échec de compilation n'est pas un rouge.
- **Sans rouge possible** (le test n'exerce que des API déjà implémentées ; c'est le cas de U5) : vert au premier passage, puis **sensibilité par mutation** sur copie isolée **hors de l'arbre de travail**, une mutation à la fois, jamais présentée comme rouge initial. Le rapport nomme, pour chaque mutation, le test détecteur et les autres assertions échouées.
- **M-S13** (U4) : synchronisation conservée pendant l'invocation de l'infrastructure ; mutation lancée seule, sur une copie isolée, sous un `timeout` externe, en n'exécutant que le scénario de test ciblé AQ-1 (pas toute la suite) ; le rapport indique si la détection a été un blocage ou une panique, et ne la présente pas comme un échec d'assertion ; aucun test d'acceptation fonctionnel ajouté.
- **Quatre contrôles à chaque unité**, résultats réels : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`.
- **Miri et MSRV 1.85 : réservés à U5 et exécutés par l'utilisateur.** La session fournit les commandes exactes : `cargo +nightly miri test -p verbum-shell` (ajouter `MIRIFLAGS="-Zmiri-disable-isolation"` seulement si l'horloge l'exige, à constater) ; `cargo +1.85.0 build --workspace` puis `cargo +1.85.0 test --workspace`. Elle attend les résultats.
- **Résultats réels uniquement.** Aucun résultat n'est supposé, déduit ni estimé. Un contrôle non exécuté est dit non exécuté.
- En U4, l'adaptation de `tests/{lifecycle,pacing}.rs` à la nouvelle signature de la boucle ne modifie **aucune assertion existante** ; elle est listée au rapport **séparément des nouveaux tests** et n'est pas présentée comme un rouge.

## 8. Arbitrages et limites clos (non rouverts)

- **Séquencement (A-1b).** U2 et U3 ne construisent pas la boîte. PC-2 est déplacé en U4 et couvert par AQ-5, sans PC-2 sur lots vides. U4 construit, teste et intègre la boîte. Aucun trait public ni abstraction générale pour cette transition.
- **Boîte de réception.** Capacité fixée à la construction, jamais agrandie ; stockage côté Plateforme, jamais dans le `World` ; événement `Copy` sans type de plateforme ; capacité de la boîte ≤ capacité du lot (violation = bug de configuration) ; saturation = erreur explicite au déposant, événements acceptés conservés dans l'ordre, ni perte ni écrasement ni croissance ; une itération sans dépôt reçoit un lot vide, jamais un lot absent.
- **Coupure et synchronisation.** Chaque dépôt est entièrement avant ou entièrement après la coupure : **garantie réalisée par le code** (synchronisation sûre, sans `unsafe`, qui sérialise dépôts et prélèvement et couvre tout accès au contenu de la boîte), prélèvement en un seul passage et sans allocation sur le chemin d'itération. **La synchronisation est libérée avant l'invocation de l'infrastructure** : un dépôt pendant un tick est accepté sans interblocage et appartient à une itération ultérieure. Primitive et forme de la poignée : choix locaux. Vérification par revue R-1 à R-5.
- **Aucune preuve de sûreté concurrente réelle.** Les tests scriptés (sans thread) vérifient la logique de visibilité ; ils ne démontrent pas l'atomicité ni l'absence d'erreur de synchronisation, qui restent non démontrées et à dire comme telles au rapport.
- **Publication du lot (U5).** Le porteur est une Ressource transitoire dédiée, initialisée à l'assemblage avec un lot vide (longueur zéro) ; la publication est un remplacement intégral avant **chaque** itération, la première comprise ; si la publication échoue, `run_tick` n'est pas invoqué. Distinguez toujours l'initialisation du porteur avant la boucle (assertion (0), M-S10a) de la publication obligatoire avant chaque tick (assertion (i-bis), M-S10b). AC-5 utilise un lot non vide au tick 0. Tout mécanisme simulant un dépôt pendant un tick dans AC-5 est strictement un instrument de test, sans thread réel, sans API publique nouvelle, sans `unsafe` ; s'il est impossible sous ces contraintes, arrêt et exposé des options.
- **AC-6.** Contrôle de composition par `SystemList::fill` ou `contract_of` : aucun contrat n'écrit ni ne produit la catégorie du lot, au moins un la lit ; test du contrôle sur une liste contenant un système fautif (M-S12). Il est de composition : les types actuels n'empêchent pas un système d'écrire une catégorie déclarée en lecture, et il ne voit pas un handle externe.
- **Portée de S6 (A-2).** S6 concerne le programme applicatif livré. Le câblage de `World`, `Scheduler` et buffer dans `tests/acceptance.rs` est un montage de test, non un second point d'assemblage ; l'unicité de l'assemblage livré se vérifie par revue de `examples/console.rs`.
- **Arbitrages antérieurs** (à ne pas confondre avec A-1b, A-2, A-3 du 20 octobre 2026, ci-dessus) : A1 à A3, A5 (i et ii), A6, C7, L4/C12, A4 clos par ADR-S03. Le tick fixe est une politique provisoire (ADR-S04 réservé).

## 9. Interdits

- Ne modifier ni les ADR, ni `ROADMAP.md`, ni `docs/implementation/00` à `02`, ni `crates/{ecs,schedule,assets,forge,verbum}/**` ; ne pas modifier le plan, le document d'exécution ni ce handoff sans décision explicite de l'utilisateur.
- Écrire uniquement : `Cargo.toml` racine (`members`), `Cargo.lock` (par Cargo), `crates/shell/**`.
- Aucun `unsafe` (lint du workspace `unsafe_code = "deny"`). Aucune dépendance normale ni externe ; dev-dependencies `verbum-ecs` et `verbum-schedule` **en U5 seulement**.
- Aucun thread réel dans les tests, aucune croissance dynamique, aucune allocation sur le chemin d'itération, aucune API exposant `Instant` ou `Duration` à l'infrastructure, aucun type du noyau nommé par la boucle générique, aucun `World::execute()` ni `Scheduler::run()`.
- **Aucun commit par Claude.** La session propose en fin d'unité un regroupement de commit (périmètre seulement). `tree.md` est régénéré par l'utilisateur ; la session signale les fichiers créés sans y toucher.
- Pas d'extension de périmètre : fenêtre, rendu, traduction réelle de signaux, accès disque, signaux OS.

## 10. Divergence, doute, blocage

Arrêt immédiat, sans choix de la session, en cas de : contradiction ou ambiguïté du plan ou d'un ADR ; divergence entre un fait [Vérifié] (F1 à F10) et le code réel ; test dont le rouge ne peut pas être constaté pour la bonne raison ; mutation non détectée ; besoin de modifier un fichier interdit ; besoin d'un trait public ou d'une abstraction non prévue. La session fournit alors un constat factuel (source, texte, observation) et présente les options, **sans arbitrer**.

## 11. Reprise

Si la session se dégrade, une nouvelle session reçoit ce handoff, le document d'exécution, le plan, ADR-S03, `tree.md` et la dernière phrase de rapport validée (« Unité Un validée »). Elle constate l'état réel avec les quatre contrôles et reprend à l'unité suivante ; une unité interrompue est refaite depuis ses tests.

## 12. Consigne d'amorce

> Vous implémentez l'étape 3 de Verbum (`verbum-shell`). Lisez, dans cet ordre : `03-verbum-shell-implementation-handoff.md`, `03-verbum-shell-execution.md`, `03-verbum-shell.md` (une seule fois en entier), l'ADR-S03 adoptée, puis `tree.md`. N'écrivez aucun code et ne lancez aucune commande avant que l'utilisateur ait dit : « démarrez l'unité U1 ». Accusez réception en cinq lignes maximum, en indiquant le statut des documents tel qu'il figure au § 4.
