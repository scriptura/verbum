# Exécution — étape 3 : Shell minimal (`verbum-shell`)

Document d'exécution, non normatif. **Il ne contient aucune règle de conception.** Source de vérité unique : `docs/implementation/03-verbum-shell.md`, dans sa version **ajustée du 20 octobre 2026**. Cette version ne diffère du plan r4 révisé (validé par l'utilisateur le 20 octobre 2026) que par les ajustements issus des arbitrages A-1b et A-3 ; **elle a été validée par l'utilisateur le 21 octobre 2026**. Seul le plan r4 révisé précédent est validé. En cas de divergence entre ce document et le plan, **le plan prévaut** ; la divergence est signalée à l'utilisateur, jamais résolue par la session. Ce document n'est modifié que sur décision explicite de l'utilisateur. Le normatif est `ADR-S01`, `ADR-S03` (adoptée le 19 octobre 2026), `ADR-000` et `00-principes.md`.

**Statut du plan de référence.** Le plan `03-verbum-shell.md` (version ajustée du 20 octobre 2026) diffère du plan r4 révisé validé par : (a) les ajustements issus des arbitrages A-1b et A-3 ; (b) la clôture de C11 comme choix local de l'étape 3 ; (c) des harmonisations éditoriales de version. **A-2 clarifie la portée de S6 dans le présent document seulement, sans modifier le plan ni le corpus normatif.** Les arbitrages A-1b, A-2 et A-3 sont clos (§ 9). Le plan et le présent document ont été validés par l'utilisateur le 21 octobre 2026.

**Précision de nomenclature.** Les arbitrages « A1, A2, A3, A5 (i et ii), A6 », « C7 », « L4/C12 » et « A4 » (plan § 4) sont les arbitrages antérieurs, clos le 18 et le 19 octobre 2026. « A-1b, A-2, A-3 » désignent trois arbitrages distincts du 20 octobre 2026, portant sur le séquencement de la boîte, la portée de S6 et M-S13. Dans les interdits (§ 2) et les instructions de reprise (§ 6), « arbitrage clos » les vise tous, sans les confondre.

**État : validé par l'utilisateur le 21 octobre 2026 (avec le plan et le handoff). Aucune unité ne démarre avant que l'utilisateur ait dit : « démarrez l'unité U1 ».**

Légende, appliquée dans chaque unité :
- **[Vérifié]** : fait constaté dans un document lu ou dans le code réel, avec sa source. Revérifié au démarrage de l'unité ; une divergence est un arrêt (§ 6).
- **[Plan]** : décision du plan r4 révisé (ajusté le 20 octobre 2026). Ce n'est pas un fait.
- **[À démontrer]** : point que l'unité doit établir, ou point qu'elle ne peut pas établir et qui reste signalé au rapport.

## 0. Consigne d'amorce (à coller au début de la session)

> Vous implémentez l'étape 3 de Verbum (Shell minimal, crate `verbum-shell`). Lisez, dans cet ordre : `docs/implementation/03-verbum-shell-execution.md` (règles de session), puis `docs/implementation/03-verbum-shell.md` (version ajustée du 20 octobre 2026, plan de référence, validé le 21 octobre 2026 ; à lire une seule fois en entier), puis `docs/architecture/adr/ADR-S03-acquisition-evenements.md`, puis `tree.md` (fait foi sur les chemins). Les autres ADR ne se relisent que si une unité les cite. N'écrivez aucun code avant que l'utilisateur ait dit : « démarrez l'unité U1 ». Accusez réception en cinq lignes maximum.

## 1. Règles de session

1. **Une unité à la fois.** Le § 4 reprend U1 à U5 du plan, sans les fusionner ni les réordonner. À la fin d'une unité, la session s'arrête et rédige le rapport (§ 5). Elle ne commence l'unité suivante que sur validation explicite de l'utilisateur (« unité Un validée, démarrez Un+1 »).
2. **Rouge avant vert quand un rouge est possible.** Écrire les tests listés contre des squelettes d'API (signatures, corps `unimplemented!()`), constater qu'ils échouent pour la raison attendue, puis implémenter. Un échec de compilation n'est pas un rouge valide. Le rouge est rapporté (test, motif d'échec observé).
3. **Sensibilité par mutation quand aucun rouge n'est possible** (le test n'exerce que des API déjà implémentées) : vert au premier passage, puis mutations sur copie isolée **hors de l'arbre de travail**, une seule mutation à la fois, jamais présentée comme rouge initial. Pour chaque mutation, le rapport nomme **le test qui la détecte et les autres assertions qui échouent**. Une mutation non détectée est un arrêt (§ 6). Une mutation qui fait échouer plusieurs assertions alors que le plan annonce une détection séparée (M-S10a, M-S10b) est un arrêt.
4. **Contrôles réels.** À chaque unité : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`. Le rapport donne les lignes de résultat réelles (récapitulatif des tests, avertissements, codes de sortie) ; la sortie complète n'est fournie qu'en cas d'échec. Aucun résultat n'est déduit, estimé ni rapporté sans avoir été produit.
5. **Miri et MSRV 1.85 sont exécutés par l'utilisateur, en U5** (plan, U5). La session fournit les commandes exactes et attend les résultats avant de déclarer l'étape prête à clôture. L'utilisateur peut les exécuter plus tôt ; la session ne l'exige pas.
6. **Pas de commit par la session.** Elle propose, en fin d'unité, un regroupement par invariant démontré ; messages et commits restent à la charge de l'utilisateur. `tree.md` est régénéré par le script de l'utilisateur : la session signale les fichiers créés, sans y toucher.
7. **Noms provisoires.** Les noms du plan marqués provisoires (modules, signatures de la boucle, poignée de dépôt, primitive de synchronisation) peuvent être ajustés ; tout ajustement est listé au rapport, sans changement de sémantique. **Aucun trait public supplémentaire** n'est ajouté pour la couture (A1 = O2).
8. **Économie de contexte.** Lire le plan une fois en entier, puis seulement la section de l'unité en cours ; ne pas relire les fichiers que la session vient d'écrire ; rapports concis.

## 2. Interdits sans arbitrage explicite de l'utilisateur

- Rouvrir un arbitrage clos (A1, A2, A3, A5 i et ii, A6, C7, L4/C12, A4/ADR-S03, ainsi que A-1b, A-2, A-3 du 20 octobre 2026) ou modifier le plan ou l'ADR-S03.
- Modifier les ADR, `ROADMAP.md`, `docs/implementation/00` à `02`, `crates/{ecs,schedule,assets,forge,verbum}/**`.
- Ajouter une dépendance normale à `verbum-shell`, ou toute dépendance externe. Les seules dépendances admises sont les dev-dependencies `verbum-ecs` et `verbum-schedule`, **en U5 seulement**.
- Utiliser `unsafe` (le lint du workspace l'interdit). Faire croître dynamiquement une capacité. Introduire une API qui expose `Instant` ou `Duration` à l'infrastructure.
- Faire dépendre un crate du noyau de `verbum-shell`. Donner au Shell la détention du `World` ou le nom d'un type du noyau.
- Implémenter `World::execute()` ou `Scheduler::run()`. Étendre le périmètre (plan § 10) : fenêtre, rendu, Backend, traduction réelle de signaux, accès disque, gestion de signaux OS, threads réels dans les tests.
- Étendre la portée d'un test ou d'une API au-delà de ce que le plan liste.

## 3. Faits vérifiés communs à toutes les unités

| # | Fait [Vérifié] | Source |
|---|---|---|
| F1 | `Scheduler::<B, L, SYSTEMS, PHASES>::new() -> Result<Self, ModelError>` ; `run_tick<const ENTITIES, const COMMANDS, S, R>(&self, &mut B::World, &mut B::Buffer) -> Result<(), TickError>` avec `R: Default` | `crates/schedule/src/scheduler.rs` |
| F2 | `TickError` = `Binding`, `Buffer(BufferFull)`, `BufferNotEmpty`, `Apply(ApplyError)`. Un buffer non vide à l'entrée est refusé. `world.apply(buffer)` est appelé en fin de chaque phase. La première erreur arrête le tick, sans retour arrière, le buffer garde ses commandes | `scheduler.rs` |
| F3 | `ModelError` = `MixedNature`, `MultipleWriters`, `UnknownPhase`, `Cycle` : aucune variante pour une lecture sans producteur interne. Les tests d'`order.rs` acceptent des modèles composés de lecteurs seuls | `model.rs`, `order.rs` |
| F4 | `ResourceSlot<T>` : `new`, `insert` (rend l'ancienne valeur), `get`, `get_mut` ; **aucune remise à vide**. `Read<ID, &mut ResourceSlot<T>>::get` rend `Option<&T>` | `ecs/resource.rs`, `schedule/access.rs` |
| F5 | L'accès en lecture, en écriture et en production reçoit le même `&mut ResourceSlot<T>` (`Locate::Store`) : **les types n'interdisent pas l'écriture d'une catégorie déclarée en lecture par un autre système** | `access.rs`, `examples/headless.rs` |
| F6 | API publique de contrôle des contrats : `SystemList::fill(&mut [(usize, Contract)], offset)`, `SystemList::LEN`, `contract_of::<B, S>() -> Contract`, `Contract::{reads, writes, produces}`, `DataSet::contains(DataId)`, `DataId::new(u8) -> Result<_, DataIdOutOfRange>` | `system.rs`, `contract.rs`, `data.rs` |
| F7 | `World<ENTITIES, S, R>` : `new`, `resources`, `resources_mut`, `split_mut` ; `CommandBuffer` : `new`, `len`, `is_empty`, `clear` | `world.rs`, `command.rs`, `headless.rs`, tests d'acceptation du Scheduler |
| F8 | Façade `verbum` : `#![no_std]`, réexporte `ecs` et `schedule`. Workspace : édition 2024, `rust-version = 1.85`, résolveur 3, `unsafe_code = "deny"` | `crates/verbum/src/lib.rs`, `Cargo.toml` racine |
| F9 | Expérience U5 exécutée dans une copie du dépôt, hors arbre de travail : un lecteur sans producteur interne est accepté ; une Ressource remplie hors modèle entre deux ticks est lue ; un lot vide et des lots successifs sont lus ; un emplacement jamais rempli se lit `None`. **Portée** : un lot réduit à deux entiers ; ni plusieurs phases, ni plusieurs lecteurs, ni Miri, ni MSRV | plan § 0 |
| F10 | ADR-S03 est adoptée (19 octobre 2026) ; `tree.md` du 18 octobre liste les crates `assets`, `ecs`, `forge`, `schedule`, `verbum`, et `ADR-S01` seul pour le Shell. `World::execute()` et `Scheduler::run()` n'existent pas | ADR-S03 ; `tree.md` ; plan C1 |

Ce que ces faits **ne** disent **pas** : aucun test concurrent réel n'existe ; la sûreté concurrente de la boîte n'est pas établie ; l'absence d'allocation sur le chemin d'itération n'est vérifiable que par revue.

## 4. Unités

Contrôles communs : les quatre commandes du § 1, point 4, sont exécutées à chaque unité, avec le résultat attendu **« code de sortie 0, zéro avertissement clippy, `fmt --check` sans différence, aucun test existant en échec »**. Les nombres de tests du workspace avant l'unité ne sont pas anticipés : le rapport donne le récapitulatif réel et le nombre de tests ajoutés.

### U1 — Crate `verbum-shell` vide

**Fichiers.** Créés : `crates/shell/Cargo.toml`, `crates/shell/src/lib.rs` (documentation de crate, aucune logique). Modifiés : `Cargo.toml` racine (ajout à `members`) ; `Cargo.lock` par Cargo seulement.

**Invariants.** ADR-S01 pt 4 (dépendances de plateforme confinées au Shell) ; ADR-000 pt 3 (le noyau ne nomme pas le Shell) ; A1 (aucune dépendance normale).

| Rubrique | Contenu |
|---|---|
| [Vérifié] | F8 (héritage du workspace). Les crates du noyau ne dépendent pas du Shell (il n'existe pas) |
| [Plan] | Package `verbum-shell`, dans `crates/shell` ; `std` autorisé (ADR-000 pts 2 et 5) ; héritage de `edition`, `rust-version`, `[lints] workspace = true` ; aucune dépendance |
| [À démontrer] | Le crate compile dans le workspace, sans dépendance et sans dépendant ; `cargo tree` des crates du noyau inchangés par rapport à l'état de clôture de l'étape 2 (constaté au démarrage, pas supposé) |

**Tests d'acceptation.** Aucun comportement. **Stratégie** : pas de rouge applicable ni de mutation.

**Contrôles propres.** `cargo tree -p verbum-shell` (aucune dépendance) ; `cargo tree -i verbum-shell --workspace` (aucun dépendant) ; `cargo tree -p verbum-ecs` et `-p verbum-schedule` identiques à l'état de clôture de l'étape 2.

**Résultats attendus.** Quatre contrôles verts. `cargo test --workspace` rapporte `running 0 tests` pour `verbum-shell`. Les deux `cargo tree` du Shell ne montrent aucune dépendance.

**Critère de sortie.** Résultats ci-dessus constatés ; seuls `Cargo.toml` racine, `Cargo.lock` et `crates/shell/**` touchés. Rapport, puis « En attente de la validation de l'unité U1. »

### U2 — Cycle de vie et boucle bornée

**Fichiers.** `crates/shell/src/{lib.rs, shell.rs}`, `crates/shell/tests/lifecycle.rs`. `inbox.rs` n'est **pas** créé en U2 : la boucle remet un lot vide (`&[]`) à l'infrastructure ; aucune boîte, aucun trait public ni abstraction de lot pour cette transition (arbitrage A-1b).

**Invariants.** ADR-S01 pts 1 et 3 ; ADR-000 pt 2 ; ADR-S03 pt 6 (appel unique de l'infrastructure par itération, sans point d'accroche entre la publication et l'invocation) ; A1, A2, A5 (i). Une itération commencée est menée à son terme.

| Rubrique | Contenu |
|---|---|
| [Vérifié] | F1, F2 (le Shell ne les appelle pas : l'infrastructure est une fermeture) ; F10 (`world.execute()` n'existe pas) |
| [Plan] | Forme indicative de la boucle (plan § 5) : fermeture de cadencement `FnMut()`, fermeture d'arrêt `FnMut() -> bool` observée au début de chaque itération, fermeture d'infrastructure `FnMut(&[Ev]) -> Result<(), E>` appelée une fois par itération, **avec un lot vide (`&[]`) en U2 et U3** (la boîte entre en U4) ; `Err(E)` rendue inchangée à la première erreur, sinon `Ok` avec le nombre d'itérations achevées ; un tick engagé n'est jamais interrompu ; aucune allocation dans le corps d'itération (revue) |
| [À démontrer] | LC-1 à LC-4 ; l'API publique ne nomme aucun type du noyau ; aucune dépendance normale |

**Tests d'acceptation** (infrastructure factice qui compte ses exécutions et ses libérations par `Drop` ; arrêt demandé par fermeture scriptée).
- **LC-1** : N itérations puis arrêt demandé → exactement N exécutions, dans l'ordre ; une seule libération, après la dernière.
- **LC-2** : arrêt demandé avant la première itération → 0 exécution ; libération effectuée.
- **LC-3** : arrêt demandé **pendant** une exécution → cette exécution s'achève (témoin de fin écrit) ; aucune suivante.
- **LC-4** : une exécution échoue → la boucle s'arrête ; l'erreur d'origine est rendue **sans transformation** (égalité avec la valeur émise) ; aucune réexécution ; libération effectuée.

**Stratégie.** Rouge applicable : squelette `unimplemented!()` de la fonction d'exécution ; motif attendu : panique du squelette pour LC-1 à LC-4. Les signatures retenues au squelette sont listées au rapport.

**Contrôles propres.** `cargo tree -p verbum-shell -e normal` (aucune dépendance) ; revue : l'API publique ne nomme aucun type du noyau ; revue : aucune allocation dans le corps d'itération (C7) ; revue : aucun `unsafe`.

**Résultats attendus.** Rouge des 4 tests constaté avant implémentation ; après : 4 tests verts, quatre contrôles verts, contrôles propres conformes.

**Critère de sortie.** LC-1 à LC-4 verts ; contrôles propres conformes ; écarts de noms listés. « En attente de la validation de l'unité U2. »

### U3 — Cadencement provisoire

**Fichiers.** `crates/shell/src/{pacing.rs, shell.rs}`, `crates/shell/tests/pacing.rs`. PC-2 est déplacé en U4 (arbitrage A-1b).

**Invariants.** ADR-S01 pt 5 ; `00-principes.md` inv. 3  ; Roadmap (politique provisoire). La boucle remet toujours un lot vide : aucune assertion sur des lots n'est possible en U3.

| Rubrique | Contenu |
|---|---|
| [Vérifié] | ADR-S01 pt 5 : aucune politique temporelle n'est imposée ; ADR-S04 n'existe pas (F10) |
| [Plan] | Cadencement à pas fixe **provisoire**, isolé dans `pacing.rs`, remplaçable ; le mot « provisoire » figure dans la documentation de chaque élément public ; l'API publique n'expose ni `Instant` ni `Duration` à l'infrastructure ; le `World` ne reçoit jamais de durée ni d'instant |
| [À démontrer] | PC-1 et PC-3 ; la politique à pas fixe n'est pas figée (PC-3) |

**Tests d'acceptation.**
- **PC-1** : intervalle nul → N itérations sans attente.
- **PC-2** : **déplacé en U4 et couvert par AQ-5** (comparer des suites de lots exige la boîte ; un PC-2 sur lots vides serait vacu). Le numéro n'est pas réattribué.
- **PC-3** : la boucle accepte une seconde politique de cadencement scriptée (déclenchement à la demande) sans modification du reste.

**Stratégie.** Rouge applicable : squelette `unimplemented!()` du cadencement ; motif attendu : panique du squelette.

**Contrôles propres.** Revue de nommage (« provisoire » dans la documentation publique) ; l'API publique n'expose ni `Instant` ni `Duration` à l'infrastructure.

**Résultats attendus.** Rouge de PC-1 et PC-3 constaté ; puis 2 tests verts, quatre contrôles verts, revue de nommage acceptée.

**Critère de sortie.** PC-1 et PC-3 verts ; revue de nommage acceptée. « En attente de la validation de l'unité U3. »

### U4 — Boîte de réception et test d'invariant de la Roadmap

**Fichiers.** `crates/shell/src/{inbox.rs, shell.rs}`, `crates/shell/tests/acquisition.rs`. U4 construit et teste la boîte, puis achève son intégration dans la boucle : la signature de la boucle reçoit la boîte et le prélèvement remplace le lot vide de U2. Les tests de `tests/{lifecycle,pacing}.rs` sont adaptés à cette seule signature, sans modifier leurs assertions ; l'adaptation est listée au rapport. Aucun trait public ni abstraction générale (arbitrage A-1b).

**Invariants.** ADR-S01 pt 5 ; `00-principes.md` inv. 3 ; ADR-S03 pt 11 (le déterminisme porte sur une même séquence de lots publiés, non sur le découpage de l'arrivée réelle en lots) ; ADR-S01 pt 6 ; `02-concepts.md` (principe de circulation) ; ADR-000 pts 3 et 4 ; ADR-S03 pts 3, 7, 10 et 12 ; A3 ; L4.

| Rubrique | Contenu |
|---|---|
| [Vérifié] | La Roadmap exige que l'invariant d'ADR-S01 pt 6 soit vérifiable « quitte à le simuler avec une fausse source d'entrée asynchrone ». F2 est sans objet ici : le Shell ne touche pas au `World` |
| [Plan] | Boîte de réception à capacité fixée à la construction, jamais agrandie ; stockage côté Plateforme ; poignée de dépôt clonable ; saturation : erreur explicite, rien n'est modifié ; prélèvement de l'ensemble du contenu à la coupure, dans l'ordre d'acceptation, dans un lot de même capacité alloué à la construction ; une itération sans dépôt reçoit un lot vide ; `Ev: Copy` sans type de plateforme ; source scriptée sans thread (A3) |
| [Plan] — atomicité | **Garantie réalisée par le code** : dépôts et prélèvement sont sérialisés par une synchronisation sûre, de sorte que chaque dépôt soit entièrement avant ou entièrement après la coupure (ADR-S03 pt 7). Primitive et forme de la poignée : choix locaux, non imposés |
| [Plan] — libération | La synchronisation est **libérée avant l'invocation de l'infrastructure** (plan, U4, ajusté le 20 octobre 2026) : le lot détenu par la boucle est distinct du contenu de la boîte ; un dépôt effectué pendant un tick est accepté sans interblocage et appartient à une itération ultérieure |
| [À démontrer] | AQ-1 à AQ-7b ; les mutations M-S1 à M-S9 et M-S13 ; la revue R-1 à R-5 ci-dessous. **Non démontrée : la sûreté concurrente réelle** |

**Source scriptée (A3).** Un dépôt effectué par une fermeture du test, appelée depuis l'infrastructure factice (dépôt pendant un tick) ou depuis la fermeture de cadencement scriptée (dépôt entre deux ticks). Aucun thread. **Ces tests vérifient la logique de visibilité (quel lot voit quel tick) ; ils ne démontrent aucune sûreté concurrente.**

**Tests d'acceptation.**
- **AQ-1** (Roadmap ; coupure) : un événement déposé **pendant** l'exécution du tick N (donc après la coupure) n'est observé par aucun système du tick N, **le dépôt est accepté et le tick se termine** (aucun interblocage) ; l'événement reste en attente ; le lot du tick N+1 le contient, avec les événements acceptés avant la coupure N+1.
- **AQ-1b** : un événement déposé entre la fin du tick N−1 et la coupure du tick N est dans le lot du tick N.
- **AQ-2** : ordre d'acceptation par la boîte conservé dans le lot d'un tick.
- **AQ-3** : sur K ticks, aucun événement perdu ni dupliqué.
- **AQ-5** (reprend PC-2 ; S4, ADR-S03 pt 11) : une même séquence scriptée (événements attachés à des ticks) rejouée avec un intervalle nul et un intervalle non nul de l'ordre de la milliseconde donne la même suite de lots observés par l'infrastructure, non vides. Aucune assertion sur la durée réelle.
- **AQ-6** (saturation, L4) : capacité C ; C dépôts réussis ; le dépôt C+1 rend une erreur explicite ; le lot suivant contient exactement les C premiers événements, dans l'ordre ; l'événement refusé n'y figure pas ; la capacité n'a pas changé.
- **AQ-6b** : après le prélèvement, la capacité est de nouveau disponible : un nouveau dépôt réussit.
- **AQ-7** (publication complète) : boîte pleine ; le lot contient les C événements, sans troncature ; la boîte est vide après le prélèvement.
- **AQ-7b** : une itération sans aucun dépôt reçoit un lot vide ; l'infrastructure est invoquée avec zéro événement, elle n'est ni sautée ni appelée sans lot.

**Revue de la synchronisation (plan, U4 ; à rapporter point par point).**
- **R-1** : la synchronisation protège **tous** les accès au contenu de la boîte (dépôt, prélèvement, et toute lecture de longueur ou d'état) ; aucun chemin d'accès au contenu n'existe hors d'elle.
- **R-2** : le prélèvement est un **seul passage** sous la synchronisation : copie complète dans le lot préalloué de la boucle, puis vidage de la boîte ; jamais de lecture partielle.
- **R-3** : la synchronisation est **libérée avant** l'invocation de l'infrastructure.
- **R-4** : aucune allocation dynamique sur le chemin d'itération : ni le dépôt ni le prélèvement n'allouent ni ne font croître une collection ; les allocations n'existent qu'à la construction.
- **R-5** : aucun `unsafe` ; tout mode d'échec de la synchronisation (par exemple verrou empoisonné) est traité explicitement, et le choix est rapporté (choix local).

**Stratégie.** Rouge applicable pour les comportements nouveaux (squelette `unimplemented!()` de la boîte et du prélèvement). La boîte et l'intégration étant nouvelles en U4, le rouge est constaté pour chaque comportement ; l'adaptation des tests de U2 et U3 à la nouvelle signature n'est pas un rouge et n'est pas présentée comme tel. **Sensibilité par mutation**, après vert : M-S1 prélèvement après l'exécution (AQ-1b doit échouer) ; M-S2 dépôt direct dans le lot en cours d'exécution (AQ-1 doit échouer) ; M-S3 perte d'un événement au prélèvement (AQ-3) ; M-S4 inversion d'ordre (AQ-2) ; M-S5 saturation par écrasement du plus ancien (AQ-6) ; M-S6 saturation silencieuse (AQ-6) ; M-S7 croissance de la capacité (AQ-6) ; M-S8 itération sautée quand la boîte est vide (AQ-7b) ; M-S9 troncature du prélèvement au-delà d'un seuil (AQ-7) ; **M-S13** synchronisation conservée pendant l'invocation de l'infrastructure : le scénario d'AQ-1 (dépôt pendant le tick) ne se termine pas ; la mutation est lancée **seule**, sur une copie isolée hors de l'arbre de travail, sous un `timeout` externe, en n'exécutant que le scénario de test ciblé AQ-1 (pas toute la suite) ; le rapport indique si la détection a été un **blocage** ou une **panique**, et ne la présente pas comme un échec d'assertion ; elle n'ajoute aucun test d'acceptation fonctionnel (arbitrage A-3 ; ajout du plan ajusté, § 6 U4).

**Contrôles propres.** Revue R-1 à R-5 ; revue : `Ev` est `Copy`, sans type de plateforme ; revue : aucune allocation hors construction (C7).

**Résultats attendus.** Rouge constaté pour les comportements nouveaux ; 9 tests verts (AQ-1, AQ-1b, AQ-2, AQ-3, AQ-5, AQ-6, AQ-6b, AQ-7, AQ-7b) ; dix mutations (M-S1 à M-S9, M-S13) détectées, chacune par le test nommé ; M-S13 par blocage ou panique sous `timeout` externe ; quatre contrôles verts ; revue R-1 à R-5 rapportée.

**Limites assumées, à reproduire au rapport.** Un système qui capturerait un handle externe n'est pas empêché par les types (`01-verbum-scheduler.md` § 2.4, point 3). Les tests scriptés satisfont trivialement « chaque dépôt entièrement avant ou après la coupure » : **l'atomicité est une garantie du code vérifiée par revue ; elle n'est pas éprouvée par un test concurrent réel** ; l'absence d'erreur de synchronisation reste non démontrée.

**Critère de sortie.** AQ-1 à AQ-3, AQ-5, AQ-6, AQ-6b, AQ-7, AQ-7b verts ; dix mutations détectées ; revue R-1 à R-5 acceptée par l'utilisateur ; limites reproduites. « En attente de la validation de l'unité U4. »

### U5 — Acceptation, exemple console, clôture

**Fichiers.** `crates/shell/Cargo.toml` (dev-dependencies `verbum-ecs` et `verbum-schedule`, ajoutées ici seulement), `crates/shell/tests/acceptance.rs`, `crates/shell/examples/console.rs`.

**Invariants.** Ensemble S1 à S15 du plan ; ADR-S03 pts 4 à 8 et 12.

| Rubrique | Contenu |
|---|---|
| [Vérifié] | F1 à F7, F9 (voir § 3). L'API de lecture d'une Ressource rend `Option` (F4) ; les types n'interdisent pas l'écriture du lot par un autre système (F5) |
| [Plan] | Porteur : Ressource transitoire dédiée de type « lot » (tableau de capacité fixe plus longueur) dans l'ensemble de Ressources du `World` ; « lot vide » = longueur zéro, jamais un emplacement vide ; porteur **initialisé à l'assemblage** avec le lot vide, **avant** la première itération ; publication par `insert` qui remplace intégralement, **avant chaque tick** ; capacité du porteur ≥ celle du lot ; la fermeture publie puis invoque `run_tick` en une seule séquence ; si la publication échoue, `run_tick` n'est pas invoqué et la fermeture rend `Err` (A5 i) |
| [À démontrer] | AC-1 à AC-7 ; les mutations M-S9-app, M-S10a, M-S10b, M-S11, M-S12 ; S1 à S15 test par test ; revue (assemblage, allocation) |

**Distinction maintenue.** L'**initialisation du porteur avant la boucle** (assertion (0)) et la **publication obligatoire avant chaque tick** (assertion (i-bis) et suivantes) sont deux propriétés distinctes, détectées par deux mutations distinctes.

**Tests d'acceptation** (vrai `World`, vrai `Scheduler`, vrai buffer ; l'adaptateur d'application est écrit dans la fermeture).
- **AC-1** : plusieurs ticks, source scriptée ; l'état final égale la valeur attendue, avec le décalage d'un tick pour les événements déposés pendant un tick.
- **AC-2** : `Scheduler::model()` égal avant et après N ticks ; buffer vide après chaque tick réussi ; le Shell n'a fourni que le lot.
- **AC-3** : `cargo run -p verbum-shell --example console` : plusieurs ticks, aucune sortie, code retour 0.
- **AC-4** : une erreur de tick réelle (par exemple `BufferFull`, ou un `apply` rejeté) arrête la boucle ; l'erreur `TickError` d'origine est rendue inchangée ; le buffer n'est pas vidé (`len()` inchangé) ; un nouvel appel sans `clear` rend `BufferNotEmpty`.
- **AC-5** (acceptation : lecture sans producteur interne ; initialisation ; renouvellement). Le modèle contient au moins **deux phases**, avec **au moins un système lecteur du lot dans chacune**, sans producteur interne de la catégorie du lot (le `Scheduler` se construit sans erreur).
  - **Instrument de test (dépôt pendant un tick).** Tout mécanisme de déclenchement servant à simuler, dans ce test avec le vrai `Scheduler`, un dépôt pendant un tick est **strictement un instrument de test** (par exemple une fermeture appelée depuis un système ou l'adaptateur de test) ; il n'est pas un comportement de production à reproduire dans les systèmes livrés. Aucun thread réel, aucune nouvelle API publique, aucun `unsafe`, aucune extension architecturale. **Si ce scénario ne peut pas être vérifié sous ces contraintes, la session s'arrête et expose les options (§ 6) au lieu de contourner les règles.**
  - **(0)** avant le lancement de la boucle, à la fin de l'assemblage et avant toute publication, le porteur est présent (`get()` rend `Some`) et contient un lot vide (longueur zéro).
  - Puis le lot est publié avant chaque tick, sur une séquence scriptée : tick 0 avec un lot **non vide**, puis des lots successifs non vides de longueurs différentes, puis un lot vide.
  - **(i)** à chaque tick, première itération comprise, aucun système n'observe `None` (un système témoin enregistre toute lecture `None`).
  - **(i-bis)** le lot observé au tick 0 est exactement le lot non vide attendu.
  - **(ii)** les lecteurs des deux phases observent le même lot publié pendant leur itération respective (le même lot dans un tick), y compris lorsqu'un dépôt est simulé pendant ce tick.
  - **(iii)** le lot vide suivant un lot non vide est observé vide : aucune donnée d'une itération précédente ne survit.
  - **(iv)** un événement déposé pendant le tick N est observé au tick N+1 seulement.
- **AC-6** (contrôle des contrats des systèmes assemblés). À partir de la liste de systèmes de l'exemple (`SystemList::fill` ou `contract_of`, F6), une fonction de contrôle vérifie qu'**aucun** contrat n'inclut la catégorie du lot dans `writes()` ni dans `produces()`, et qu'au moins un l'inclut dans `reads()` (le contrôle n'est pas vide). Un test de la fonction elle-même, sur une liste auxiliaire contenant un système qui écrit la catégorie, doit la faire échouer. **Ce contrôle est de composition : les types actuels n'empêchent pas un tel système (F5)** ; il ne voit pas un système qui atteindrait le lot par un handle externe ; la revue en complète la portée.
- **AC-7** (publication complète et échec de publication). Un lot de la taille de la capacité du porteur est publié intégralement et lu en entier ; si la capacité du porteur est volontairement inférieure à celle du lot, la fermeture rend une erreur de configuration, **`run_tick` n'est pas invoqué** (état du `World` inchangé, témoin de non-exécution), et la boucle s'arrête avec l'erreur d'origine. Aucune troncature silencieuse.

**Stratégie.** **Aucun rouge n'est possible** : les tests n'exercent que des API déjà implémentées (U2 à U4 et crates des étapes 1 et 2) ; AC-3 n'a pas de rouge valide non plus, un échec de compilation de l'exemple absent n'en étant pas un. Vert au premier passage, puis **sensibilité par mutation** sur copies isolées, jamais présentée comme rouge initial :
- **M-S10a** : aucune initialisation du porteur à l'assemblage, publication avant chaque tick conservée → détectée par l'assertion **(0) seule** ; (i), (i-bis), (ii) à (iv) restent verts.
- **M-S10b** : initialisation conservée, publication omise avant le **premier** tick seulement → détectée par **(i-bis) seule** (le tick 0 observe le lot vide initial au lieu du lot non vide) ; (0) et (i) restent verts.
- **M-S11** : publication omise quand le lot est vide, laissant survivre le lot précédent → AC-5 (iii).
- **M-S12** : ajout d'un système déclarant l'écriture de la catégorie du lot → AC-6 doit échouer.
- **M-S9-app** : troncature à la publication dans la Ressource → AC-7. Défaut distinct de M-S9 (U4, troncature du prélèvement de la boîte, détectée par AQ-7) : code différent, détecteur différent.

**Contrôles propres.** `cargo tree -i verbum-shell --workspace` ; `cargo tree -p verbum-shell -e normal` (toujours aucune dépendance normale) ; `cargo tree -p verbum-ecs`, `-p verbum-schedule`, `-p verbum-assets`, `-p verbum-forge` inchangés ; aucune modification de `crates/{ecs,schedule,assets,forge,verbum}`, des ADR, de `ROADMAP.md`, des plans `00` à `02` ; revue : aucune allocation sur le chemin d'itération, publication comprise (S15) ; revue : l'exemple contient seul le code d'assemblage qui connaît le porteur, la boucle générique n'en connaît pas les types ; **interprétation de S6 (arbitrage A-2)** : S6 concerne le programme applicatif livré ; le câblage de `World`, `Scheduler` et buffer dans `tests/acceptance.rs` est un montage de test, non un second point d'assemblage applicatif ; l'unicité de l'assemblage livré se vérifie par revue de `examples/console.rs` ; **Miri et MSRV 1.85 par l'utilisateur** (commandes ci-dessous).

**Commandes pour l'utilisateur** (à adapter à l'installation) : `cargo +nightly miri test -p verbum-shell` (ajouter `MIRIFLAGS="-Zmiri-disable-isolation"` seulement si l'horloge l'exige, à constater) ; `cargo +1.85.0 build --workspace` puis `cargo +1.85.0 test --workspace`.

**Résultats attendus.** 7 tests d'acceptation verts (AC-1 à AC-7) ; l'exemple console termine avec le code 0 sans sortie ; cinq mutations détectées, M-S10a par (0) seul et M-S10b par (i-bis) seul ; quatre contrôles verts ; Miri et MSRV verts (résultats de l'utilisateur).

**Critère de sortie.** S1 à S15 renseignés test par test dans le rapport (tableau) ; éléments non démontrés énumérés (§ 7) ; l'étape est déclarée close **par l'utilisateur seulement**. « En attente de la validation de l'unité U5. »

## 5. Rapport de fin d'unité (gabarit, rien de narratif)

1. **Unité** et fichiers créés ou modifiés (liste).
2. **Faits [Vérifié] revérifiés** : confirmés, ou divergence (arrêt).
3. **Rouges observés** : test → motif d'échec constaté ; ou « aucun rouge possible » avec la raison.
4. **Mutations** : mutation → test détecteur → autres assertions échouées → détection par échec, blocage ou panique.
5. **Contrôles** : quatre commandes, résultat réel de chacune (lignes pertinentes), nombre de tests ajoutés. **En U4, les adaptations de `tests/{lifecycle,pacing}.rs` à la nouvelle signature (assertions inchangées) sont listées séparément des nouveaux tests.**
6. **Contrôles propres** : résultat, ou « à exécuter par l'utilisateur » avec la commande exacte ; revue R-1 à R-5 point par point pour U4.
7. **Écarts par rapport au plan** : aucun, ou liste (noms ajustés, divergence constatée, choix locaux).
8. **Points restant à démontrer** de l'unité, reproduits.
9. **Questions d'arbitrage ouvertes** : aucune, ou liste avec les options, sans choix.
10. **Regroupement de commit proposé** (périmètre seulement).
11. Fin : « En attente de la validation de l'unité Un. »

## 6. Arrêt, reprise

**Arrêt immédiat, sans choix de la session**, avec exposé des options : contradiction ou ambiguïté entre le plan, ADR-S03 et ce document ; divergence entre un fait [Vérifié] et le code réel ; test dont le rouge ne peut pas être constaté pour la bonne raison, alors que le plan en prévoit un ; mutation non détectée, ou détectée par plusieurs assertions alors que le plan annonce une détection séparée ; besoin d'une dépendance, d'un trait public, d'`unsafe` ou de la détention du `World` par le Shell ; besoin de modifier un fichier interdit (§ 2) ; constat qu'une garantie du plan ne peut être réalisée sans modifier un invariant.

**Reprise dans une nouvelle session** : fournir ce document, le plan, ADR-S03, `tree.md` et la dernière phrase de rapport validée (« Unité Un validée »). La nouvelle session relit le code déjà écrit seulement pour l'unité qu'elle reprend, constate l'état réel avec les quatre contrôles, et reprend à l'unité suivante. Une unité interrompue en cours est refaite depuis ses tests.

## 7. Non démontré à la clôture (à reproduire tel quel au rapport de U5)

- **Atomicité de la coupure sous concurrence réelle** : non éprouvée par un test concurrent réel (A3). C'est une garantie que le code doit réaliser (synchronisation sûre, sans allocation sur le chemin d'itération) et que la revue vérifie ; l'absence d'erreur de synchronisation reste non démontrée.
- Traduction sans interprétation des signaux de plateforme ; catalogue des sortes d'événements et instant temps réel dans une donnée (non tranchés par ADR-S03) ; politique temporelle réelle (ADR-S04) ; gestion des signaux de l'OS.
- **Interdiction d'écriture et de production du lot par les systèmes** : contrôle de composition (AC-6) et revue seulement, non imposée par les types actuels.
- Absence d'état « absent » observable : assurée par l'assemblage et vérifiée par AC-5, non garantie par les types.
- Absence d'allocation sur le chemin d'une itération : revue seulement.
- Absence d'effets externes cachés dans les systèmes ; exclusivité de l'assemblage livré (S6, interprété comme le programme applicatif ; montage de test exclu) : revue de `examples/console.rs` seulement.
- Chargement d'artefacts et accès disque (étape 5) ; moitié « sortie » de l'interface de franchissement (étape 4).
- Reproductibilité de l'ordre d'acceptation d'événements concurrents et de leur répartition entre lots : hors garantie (ADR-S03 pt 11).

## 8. Clôture de l'étape (U5)

Le rapport de U5 contient en plus un tableau S1 à S15 : pour chacun, l'identifiant du test ou de la revue qui l'établit et son résultat réel. La session ne déclare pas l'étape close : l'utilisateur la déclare. Aucun travail de l'étape 4 n'est entrepris.

## 9. Arbitrages rendus par l'utilisateur le 20 octobre 2026 (consignés, non réouverts)

- **A-1 → A-1b.** U2 (cycle de vie et boucle) n'implémente pas `inbox.rs` : la boucle remet un lot vide. U3 reste centré sur le cadencement (PC-1, PC-3). PC-2, qui compare des suites de lots observés sous deux cadencements, est déplacé en U4 et couvert par AQ-5 ; aucun PC-2 sur lots vides. U4 construit et teste la boîte, puis achève son intégration dans la boucle. Aucun trait public ni abstraction générale pour cette transition. Plan ajusté sur les seules sections nécessaires (S4, §5, U2, U3, U4).
- **A-2.** S6 concerne le programme applicatif livré. Le câblage de `World`, `Scheduler` et buffer dans `tests/acceptance.rs` est un montage de test, non un second point d'assemblage applicatif. L'unicité de l'assemblage livré se vérifie par revue de `examples/console.rs`. Interprétation consignée ici : § 4, U5, « Contrôles propres », revue de l'exemple, et § 7 (dernier point sur S6) ; corpus normatif et plan non modifiés sur ce point.
- **A-3.** M-S13 conservée : elle vérifie le scénario d'AQ-1 si la synchronisation est conservée pendant l'invocation ; copie isolée, limite de temps, blocage ou panique rapporté comme détection ; aucun nouveau test d'acceptation fonctionnel. Le plan (U4) porte désormais la précision « synchronisation libérée avant l'invocation » et M-S13.

Aucun point d'arbitrage ne reste ouvert dans ce document.
