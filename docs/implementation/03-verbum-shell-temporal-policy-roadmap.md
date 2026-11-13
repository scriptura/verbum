# Verbum — Roadmap d'implémentation de la politique temporelle du Shell (ADR-S04) — r1, 13 novembre 2026

## 1. Statut et périmètre

**Non normative. Non implémentée. Révision 1.** Document de planification opérationnelle, subordonné :
- aux ADR adoptées (ADR-000, ADR-004, ADR-A01, ADR-S01, ADR-S02, ADR-S03, **ADR-S04 adoptée le 11 novembre 2026**, ADR-W01) ;
- à la spécification d'implémentation `03-verbum-shell-temporal-policy.md`, r3 consolidée ;
- aux portes de gouvernance des étapes Forge et Assets (G-CM, R-ADAPT).

La spécification dit **ce qui est construit** ; cette roadmap dit **dans quel ordre, avec quels prérequis et quelles preuves**. Elle ne recopie aucun algorithme.

**Aucune implémentation n'est autorisée par ce document.** Aucun code, test, `Cargo.toml`, ADR, format d'artefact ou commit n'est créé par la mission qui l'a produit. La validation, l'exécution des commandes, la revue et les commits appartiennent à l'utilisateur.

| Marqueur | Sens |
|---|---|
| **[Contrat]** | Posé par une ADR adoptée ou par la spécification r3 (référence donnée) |
| **[Fait]** | Constaté dans un fichier fourni |
| **[Arbitré]** | Décision de l'utilisateur (12 et 13 novembre 2026) |
| **[Proposition]** | Choix de cette roadmap, à valider |
| **[Divergence]** | Écart avec la spécification r3, signalé en § 10.1 |
| **[Ouvert]** | Non déterminé ; énoncé sans choix |

**Hors périmètre** : modification des ADR ; modification du code, des tests ou des documents de l'étape 6 (close, décision utilisateur) ; résolution de R03/S05 ; intégration native winit/wgpu (dépendance de validation plateforme, pas une unité de ce chantier) ; politique de pause, suspension, reprise ; montée de MSRV.

## 2. Références normatives et documentaires

**Normatif** : `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`, ADR-000, ADR-004, ADR-A01, ADR-S01, ADR-S02 (adopté 10 novembre), ADR-S03 (adopté 19 octobre), ADR-S04 (adopté 11 novembre), ADR-W01 (adopté 7 novembre), ADR-Sc01 (pour la frontière avec l'étape 6).

**Spécification** : `03-verbum-shell-temporal-policy.md` r3 — contrat C-1 à C-16, arithmétique (§ 4), `Progressor` (§ 5), initialisation I-3 (§ 6), provenance de q (§ 7, D-1 à D-1d, R-1, VQ-1 à VQ-9), pilotes (§ 9), matrice de tests (§ 10), unités T1 à T8 et F1 à F3 (§ 11).

**Plans et comptes rendus lus** : `02-verbum-forge.md` et `-execution`, `03-verbum-shell.md` et `-execution`, `05-verbum-assets.md` et `-execution`, `06-verbum-scripts.md` et `-execution` (lecture seule : étape 6 close).

**Sources de code lues** : Shell (`shell.rs`, `pacing.rs`, `inbox.rs`, `lib.rs`, `tests/{lifecycle,pacing,acquisition,acceptance,frame_loop,script_loaded,script_suspend}.rs`, `examples/{console,headless_frame,level_headless,script_headless}.rs`) ; Assets (`manifest.rs`, `header.rs`, `format.rs`, `load.rs`, `lib.rs`, `script.rs`) ; Forge (`lib.rs`, `batch.rs`, `script.rs`) ; `Cargo.toml` (racine, Shell, Forge, Assets) ; `tree.md` du 13 novembre 2026, 04:01.

**Non fournis dans la conversation** (liste exhaustive, aucun contenu supposé) : `crates/engine/shell/tests/{level_loaded,borrowed_resource}.rs` ; `crates/engine/forge/tests/pipeline.rs` ; `crates/engine/{assets,forge}/src/{map,font,tileset}.rs` ; ADR-001 à ADR-003, ADR-005, ADR-006, ADR-R01 à R03 ; `ROADMAP.md`. L'usage de `run_loop` par `level_loaded.rs` est établi par `05-verbum-assets-execution.md` (U5) et par `level_headless.rs`, qui en recopie la scène.

## 3. État initial observé

| # | Fait | Source |
|---|---|---|
| R-1 | `crates/engine/shell/src/` ne contient que `inbox.rs`, `lib.rs`, `pacing.rs`, `shell.rs`. Aucun module de politique, d'échéancier, d'horloge, de `Progressor` ni de pilote n'existe | `tree.md` |
| R-2 | `run_loop(cadence, stop, inbox: &mut Inbox<Ev>, infrastructure) -> Result<u64, E>` : `cadence()` → `stop()` → `infrastructure(inbox.cut())?` → comptage. `cadence` ne rend aucun instant | `shell.rs` |
| R-3 | `FixedStepPacing` lit `Instant::now()` (non substituable), dort par `thread::sleep`, se déclare provisoire ; son commentaire dit « ADR-S04, non rédigée » (obsolète) | `pacing.rs` |
| R-4 | `Inbox::cut(&mut self) -> &[Ev]` emprunte la boîte ; verrou libéré avant le retour ; `DepositHandle` clonable sans durée de vie liée | `inbox.rs` |
| R-5 | `[dependencies]` du Shell vide ; `lib.rs` annonce « État actuel (unité U4) » (obsolète) et héberge les doctests LT-3, LT-3t, LT-5n, LT-5t, dont deux portent un littéral de manifeste v1 de 42 octets | `lib.rs`, `shell_Cargo.toml` |
| R-6 | `VMAN` v1 : `MANIFEST_VERSION = 1`, en-tête de 10 octets, entrées de 16 octets. `Manifest::parse` rend `ManifestError::Version` pour toute autre version ; le test `m4` épingle les versions `0x0000`, `0x0002`, `0x0100`, `0xFFFF` | `manifest.rs` |
| R-7 | `load<'a>(manifest: &[u8], artifacts: &[&'a [u8]], slot_formats: &[Format], deposit)` ; niveaux 1 à 4 ; `no_std` sans `alloc` ; cinq formats connus, `Format::Script` compris | `load.rs`, `format.rs`, `assets_Cargo.toml` |
| R-8 | `compile_assets(&[AssetSource], usize)` : signature épinglée par doctest ; `compile_assets_with` `pub(crate)` ; `CompiledSet` expose `manifest()` et `artifacts()` séparément ; `AssetKind::Script` existe ; `compile_script` ne connaît que `wait.ticks` | `forge_src_batch.rs`, `forge_src_script.rs` |
| R-9 | `SCRIPT_MAX_STEPS_PER_TICK = 16`, valeur provisoire, indépendante de K. **Résout PO-2 de la spécification** | `assets_src_script.rs` |
| R-10 | `Cargo.toml` racine : `members` = `ecs, schedule, shell, render, assets, forge, verbum, extensions/script`. `verbum-shell` n'est pas dans `[workspace.dependencies]`. `crates/games/demo` ne contient qu'un `README.md` et n'est pas membre | `Cargo.toml`, `tree.md` |
| R-11 | Les dev-dependencies du Shell incluent `verbum-assets`, `verbum-forge`, `verbum-script` : l'infrastructure de test pour un assemblage de bout en bout existe déjà | `shell_Cargo.toml` |

### 3.1 Inventaire des consommateurs historiques (périmètre de non-régression)

Ces éléments sont **conservés, non modifiés** ; ils bornent la non-régression, pas le périmètre de modification.

| Contrat protégé | Consommateurs |
|---|---|
| **`run_loop` historique** (signature et comportement observable) | `tests/lifecycle.rs` (LC-1 à 4) ; `tests/pacing.rs` (PC-1, PC-3) ; `tests/acquisition.rs` (AQ-1 à 7b) ; `tests/acceptance.rs` (AC-1 à 7) ; `tests/frame_loop.rs` (RA-1 à 5) ; `tests/script_loaded.rs` (DM-1 à 3) ; `tests/level_loaded.rs` (SC-1 à 6, non fourni) ; `examples/{console,headless_frame,script_headless,level_headless}.rs` |
| **Shell hors `run_loop`** | `tests/script_suspend.rs` (n'appelle pas `run_loop`) ; `tests/borrowed_resource.rs` (non fourni) ; doctests LT-3 à LT-5t de `lib.rs` |
| **`VMAN` v1 et `load` historique** | Assets : tests `m1` à `m8` de `manifest.rs`, `l1` à `l5`, `sf*`, `rf*`, `tb1`, `ld1` de `load.rs`, `scr_l*` de `script.rs`. Forge : `f2`, `f3`, `f7`, `f8` de `batch.rs` (littéraux de manifeste), `scr_f5`, `tests/pipeline.rs` (non fourni). Shell : doctests LT-5n/LT-5t, `script_loaded`, `script_suspend`, `level_loaded`, `level_headless`, `script_headless` |

Le recensement des tests qui assertent des octets de manifeste est **partiel** (§ 7.3 de la spécification le disait déjà) : il est complété par l'unité V2-0.

## 4. Résultat final attendu

À la clôture complète du chantier :
- `verbum-shell` offre un `Progressor` persistant et deux pilotes conformes à S04 (bloquant à politique, événementiel), qui partagent **le même** `progress()` et **le même** cœur d'itération ; `run_loop` reste, sans modification de contrat, une boucle bloquante à cadence externe hors politique S04 ;
- le quantum q provient d'un **paquet unique** (manifeste `VMAN` v2 portant `q_ns` une seule fois + artefacts), produit par une voie de compilation additive de la Forge, chargé sans allocation par `verbum-assets` ; `VMAN` v1 et `compile_assets` sont inchangés octet pour octet ;
- la règle des durées non entières est arbitrée et appliquée par la Forge ;
- `crates/games/demo` est promu au workspace (niveau Application, ADR-W01) et constitue l'assemblage de référence : paquet v2 chargé → q → `TemporalPolicy` → `World`, `Scheduler`, Shell sous pilote bloquant à politique ;
- le cœur temporel est prouvé avec des valeurs de test (q, K, M synthétiques) **indépendamment** de V2.

## 5. Organisation des chantiers

| Chantier | Niveau (ADR-W01) | Crates | Unités | Compte rendu d'exécution [Arbitré] |
|---|---|---|---|---|
| **A — Shell temporel** | Engine | `crates/engine/shell` | T1, T2, T3, T4, T5a, T5b, T5c, T6, T7, T8 | section distincte « ADR-S04 » de `03-verbum-shell-execution.md` |
| **B — Provenance de q** | Engine | `crates/engine/assets`, `crates/engine/forge` | V2-0, V2-C, V2-L, V2-P, V2-F, V2-Z, F2 | Assets : compte rendu de l'étape Assets ; Forge : compte rendu de l'étape Forge ; véhicule transversal éventuel décidé en V2-0 |
| **C — Composition applicative** | Application | `crates/games/demo` | F3a, F3b, F3c | à fixer en V2-0 / F3a |
| **G — Validation intégrée** | — | transversal | G1 | rapport de clôture |

**Trois régimes de q** (à ne pas confondre) :
1. **q de test** : valeurs synthétiques (spéc. § 8 : q = 10 ns, K = 3, M = 5 ; extrêmes). Suffit pour T1 à T8 et pour les tests de `verbum-shell`.
2. **q du paquet** : seule provenance admise pour une composition conforme à S04 ; chantier B.
3. **q de composition de jeu** : choix de l'application (`games/demo`) ; **[Ouvert]** (D-N1).

Les 16 666 667 ns / 4 / 8 de la spécification restent des illustrations, non une configuration.

Les ADR-S01 pt 7 et ADR-W01 pt 2 situent l'assemblage dans l'Application : F3 ne relève pas de `verbum-shell` [Arbitré]. Les tests et exemples de `verbum-shell` utilisent des valeurs synthétiques et ne sont pas l'assemblage de référence.

## 6. Graphe des dépendances

```
Chantier A (code)                       Chantier B (documentaire puis code)

T4 ───────────────┐                     V2-0 plan V2 ──► [P-V2] ─┐
T1 ───────┐       │                       │                      │
T2 ──► T3 ┤       ├─► T5a ► T5b ► T5c     ▼                      ▼
          └───────┘      │                V2-C conception ► [P-CONC] ► V2-L ► V2-P ► V2-F ► V2-Z
                         ▼                                                              │
                        T6 ──► T7                                                       │
                         └──► T8 (opt.)                       F2 (arbitrage D-N2) ◄─────┤
                                                                                        │
Chantier C :  F3a ──► F3b (T1, V2-Z) ──► F3c (T5, T6, D-N1) ◄──────────────────────────┘
Validation :  G1 (tout ce qui précède)
```

### 6.1 Dépendances réelles (code, documentaire, arbitrage)

| Unité | Dépend de | Nature | Remarque |
|---|---|---|---|
| T1 | — | — | |
| T2 | — | — | Fonctions sur `NonZeroU64`/`NonZeroU32` natifs : **aucune dépendance de code vers T1** [Divergence D-1]. Ordre T1→T2 conservé pour la revue |
| T3 | T2 | code | Étend `TimeError` défini en T2 [Arbitré] |
| T4 | — | — | Seule unité modifiant du code existant |
| T5a | T1, T2, T3, T4 | code | |
| T5b | T5a | code | |
| T5c | T5a (T5b recommandé validé) | code | |
| T6 | T5a, T5b | code | |
| T7 | T5, T6 | code | |
| T8 | T5, T6 | code | optionnelle |
| V2-0 | — | documentaire | démarrable en parallèle du chantier A |
| V2-C | V2-0 | documentaire | porte de conception [Arbitré] |
| V2-L, V2-P, V2-F, V2-Z | **P-V2** et **P-CONC** | **G-CM** | pas de code avant fermeture des deux portes |
| F2 | V2-F, arbitrage D-N2 | doc. + code | touche probablement des sources de l'étape 6 (§ 10.2) |
| F3a | décision D-N5 | décision | aucune dépendance de code |
| F3b | T1, V2-Z, F3a | code | q lu **du paquet chargé uniquement** |
| F3c | F3b, T5, T6, D-N1 | code + décision | |
| G1 | tout | vérification | |

### 6.2 Classement des travaux

| Catégorie | Travaux |
|---|---|
| **Démarrables** (après validation de cette roadmap ; aucun prérequis externe) | **T4, T1, T2, V2-0** ; puis, dans l'ordre interne du chantier A : T3, T5a, T5b, T5c, T6, T7, T8 |
| **Conditionnels** (après une validation ou un document précis) | V2-C (V2-0 validé) ; F3a (décision D-N5) |
| **Bloqués** | V2-L, V2-P, V2-F, V2-Z (portes P-V2 et P-CONC, G-CM) ; F2 (V2-F + arbitrage D-N2) ; F3b (V2-Z) ; F3c (F3b + D-N1) ; intégration native (micro-PoC, R03/S05 : hors chantier) |

Distinctions demandées : **développement du Shell avec q de test** = chantier A, démarrable ; **assemblage de production avec q du paquet** = F3b/F3c, bloqué ; **évolution VMAN** = V2-L à V2-Z, bloquée par G-CM ; **conversion Forge des durées humaines** = F2, bloquée par un arbitrage ; **intégration événementielle native** = hors chantier.

## 7. Phases et jalons

### 7.1 Tableau synthétique

| Ordre | Phase | Unité | Chantier | Dépend de | Statut | Jalon / porte |
|---:|---|---|---|---|---|---|
| 1 | A | **T4** cœur d'itération | A | — | Démarrable | G-REG |
| 2 | A | **T1** `TemporalPolicy` | A | — | Démarrable | |
| 3 | A | **T2** arithmétique d'échéancier | A | — | Démarrable | |
| 4 | A | **T3** horloge | A | T2 | séquence interne | **P-A** |
| 5 | B | **T5a** `Progressor` nominal | A | T1–T4 | séquence interne | |
| 6 | B | **T5b** arrêt, erreurs, acquisition | A | T5a | séquence interne | |
| 7 | B | **T5c** conformité structurelle, rejeu historique | A | T5a | séquence interne | **P-B** |
| 8 | C | **T6** pilote bloquant à politique | A | T5a, T5b | séquence interne | |
| 9 | C | T8 exemple conforme S04 (opt.) | A | T5, T6 | séquence interne | **P-C** |
| 10 | D | **T7** harnais événementiel | A | T5, T6 | séquence interne | **P-D** (équivalence des pilotes) |
| ∥ | E | **V2-0** plan V2 | B | — | Démarrable | **P-V2** (validation) |
| ∥ | E | **V2-C** conception du paquet | B | V2-0 | Conditionnel | **P-CONC** |
| 11 | E | V2-L lecteur v2 du manifeste | B | P-V2, P-CONC | Bloqué | |
| 12 | E | V2-P paquet et chargeur v2 | B | V2-L | Bloqué | |
| 13 | E | V2-F voie Forge additive | B | V2-P | Bloqué | |
| 14 | E | V2-Z clôture V2 | B | V2-F | Bloqué | **P-E** |
| 15 | E′ | F2 durées non entières | B | V2-F, D-N2 | Bloqué | **P-E′** |
| 16 | F | F3a promotion de `games/demo` | C | D-N5 | Conditionnel | |
| 17 | F | F3b provenance de q | C | T1, V2-Z, F3a | Bloqué | |
| 18 | F | F3c assemblage sous pilote à politique | C | F3b, T5, T6, D-N1 | Bloqué | **P-F** |
| 19 | G | G1 validation intégrée | G | tout | — | **P-G** |

`∥` : documentaire, sans dépendance de code, à lancer tôt car la validation de V2 est l'élément de plus longue échéance. Les sessions étant séquentielles, l'entrelacement des voies A et B relève de l'utilisateur ; la recommandation est de lancer V2-0 immédiatement après validation de cette roadmap.

### 7.2 Portes de sortie de phase

| Phase | Entrée | Sortie |
|---|---|---|
| **A** | roadmap validée | T4, T1, T2, T3 validées ; G-REG vert ; `cargo tree -p verbum-shell -e normal` sans dépendance |
| **B** | P-A | T5a, T5b, T5c validées ; matrice TP-P, TP-S, TP-E complète ; D7, D8, D9 verts |
| **C** | P-B | T6 validée ; D2 et moitié bloquante de D1 verts ; aucun `sleep` dans les tests |
| **D** | P-C | T7 validée ; D1 complet, D3, D6 verts : équivalence des deux pilotes établie |
| **E** | P-V2 + P-CONC | V2-Z validée ; VQ-1 à VQ-7 et VQ-9 verts ; tests de l'étape 2, 5, 6 inchangés |
| **E′** | P-E + arbitrage D-N2 | règle appliquée, tests de la Forge, aucune régression |
| **F** | P-D, P-E, F3a | F3b et F3c validées ; VQ-8 vert ; exemple de `games/demo` de code 0 |
| **G** | P-F (et P-E′ si F2 retenu) | G1 validée |

## 8. Fiches des unités de travail

Gabarit de compte rendu de **toutes** les unités : celui de `03-verbum-shell-execution.md` § 5 (unité, fichiers, faits revérifiés, rouges, mutations, contrôles, écarts, non démontré, arbitrages, regroupement de commit). Quatre contrôles à chaque unité : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, **exécutés par l'utilisateur**. Méthode : invariant → test → API minimale → implémentation ; rouge avant vert quand il est possible ; sinon vert au premier passage puis sensibilité par mutation sur copie isolée. Contrainte transverse : **Rust 1.85** (pas de let-chains ; aucune constante utilisée seulement dans un `const _ assert!` ; `is_empty` avec tout `len` public).

### T4 — Extraction du cœur d'itération

- **Objectif** : la sémantique d'itération (observation de l'arrêt → coupure → invocation unique → comptage) n'existe qu'**une fois** ; `run_loop` est réécrit au-dessus, signature et contrat observable inchangés (S02 pt 2, 3, 13).
- **Prérequis** : aucun. **Statut** : démarrable.
- **Fichiers** : modifié `src/shell.rs`. **Documentation seule** : `src/lib.rs` (texte `//!` « État actuel », sans toucher aux doctests LT-3 à LT-5t ni aux `mod`/`pub use`) et `src/pacing.rs` (commentaire obsolète sur ADR-S04). Aucun `Cargo.toml`.
- **Invariants** : cœur unique ; coupure unique à la frontière ; verrou libéré avant l'invocation (S03 pts 6, 7, 12) ; arrêt observé **avant** la coupure ; première erreur rendue inchangée.
- **Tests** : D4 (suites historiques de § 3.1 **inchangées et vertes**, aucune adaptation) ; D5 (revue structurelle : coupure, publication, invocation dans une seule fonction) ; **ajout ciblé de conformité** (invariant S02 : un arrêt observé avant une itération empêche toute coupure de l'`Inbox`), sans modification des assertions historiques. Ce test vérifie : arrêt observé avant la première itération ; aucune coupure de l'`Inbox` ; aucune invocation de l'infrastructure ; aucune itération comptabilisée ; contenu de l'`Inbox` conservé après l'arrêt. Refactorisation : sensibilité par mutation pour D4 et D5. Familles : arrêt observé après la coupure ; verrou conservé pendant l'invocation (AQ-1) ; deux invocations par itération ; erreur transformée (LC-4).
- **API minimale** : une fonction interne `pub(crate)` rendant trois issues (arrêt observé / échec, erreur inchangée / itération achevée). Aucun trait public, aucun `dyn`, aucune allocation.
- **Non-régression** : tout § 3.1 ; `[dependencies]` vide ; diff de `lib.rs` limité à des lignes de commentaire de documentation.
- **Acceptation** : quatre contrôles ; Miri `verbum-shell` ; MSRV ; `cargo tree -p verbum-shell -e normal` vide.
- **Point d'attention** : la mutation « arrêt observé après la coupure » pouvait survivre faute de test historique vérifiant l'`Inbox` intacte ; le test de conformité S02 ci-dessus la ferme explicitement. Ce test est autorisé et inscrit à la liste de T4 ; il peut être rouge contre une mutation et n'altère aucune assertion historique. Toute mutation qui survivrait malgré lui reste un arrêt.
- **Commits** : (1) extraction + suites inchangées ; (2) documentation.

### T1 — `TemporalPolicy`

- **Objectif** : représenter (q, K, M) ; C-1 et C-2 portés par les types ; aucune politique invalide constructible.
- **Prérequis** : aucun. **Statut** : démarrable.
- **Fichiers** : créé `src/policy.rs` ; `src/lib.rs` (module, exports `TemporalPolicy`, `PolicyError`).
- **Tests** : A7 (q = 0, K = 0, M = 0 rejetés ; K = M = `u32::MAX` acceptés ; (1, 1000) et (1000, 1) acceptés ; q hors plage) ; E11 **au niveau de la politique** (« aucun `Progressor` constructible » est reconfirmé en T5a par la signature). Familles de mutation : zéro accepté, bornes décalées, K et M permutés.
- **API minimale** : formes d'entrée de q selon spéc. § 4.2 (nanosecondes `u64` : forme issue du paquet ; `Duration` : forme de test) ; accesseurs en lecture.
- **Non-régression** : aucune dépendance ; aucune allocation ; ni flottant ni `unsafe`.
- **Acceptation** : A7, E11 verts ; quatre contrôles ; Miri ; MSRV.
- **Commit** : 1.

### T2 — Arithmétique d'échéancier

- **Objectif** : fonctions pures de la spéc. § 4.3 (itérations exécutées, test « R > M », avancement, réancrage) en temps constant, `checked`, sans flottant ni parcours proportionnel au retard ; `TimeError` **minimal** (`ScheduleOverflow`) défini ici, première unité qui en a besoin [Arbitré].
- **Prérequis** : aucun de code (D-1). **Statut** : démarrable.
- **Fichiers** : créé `src/schedule.rs` ; `src/lib.rs` (`mod`, sans export public encore).
- **Tests** : A1 à A5, A8 ; A5 porte sur `ScheduleOverflow`. A8 compare à une définition naïve codée dans le test, par énumération exhaustive ; q = 1, `t = u64::MAX` rend `e = K` sans débordement. Familles de mutation : `s + 1` non évité, `wrapping_add` ou `saturating_add` (saturation silencieuse interdite), `>=` au lieu de `>` pour M.
- **API minimale** : fonctions `pub(crate)` ; `TimeError` à variante unique.
- **Acceptation** : égalité avec la définition naïve ; extrêmes sans panique ; revue : aucune boucle bornée par le retard.
- **Commits** : (1) arithmétique ; (2) `TimeError` minimal avec son test A5.

### T3 — Horloge

- **Objectif** : conversion `Duration` → `u64` ns vérifiée ; **extension** de `TimeError` par `ClockOutOfRange` ; `MonotonicClock` (origine `Instant`, `Copy`) ; précondition de monotonie documentée (C-14).
- **Prérequis** : T2. **Statut** : séquence interne.
- **Fichiers** : créé `src/clock.rs` ; `src/schedule.rs` (ajout d'une variante) ; `src/lib.rs` (exports `MonotonicClock`, `TimeError`).
- **Tests** : A6. Aucune lecture d'horloge réelle hors de ce module et du pilote par défaut (T6).
- **Acceptation** : `Duration` > `u64::MAX` ns → `ClockOutOfRange` ; aucune panique ; revue : l'`Instant` ne franchit jamais `Progressor`.
- **Commit** : 1.
- **Porte P-A** : T4, T1, T2, T3 validées.

### T5a — `Progressor` : progression nominale

- **Objectif** : `Progressor`, `Progress`, `Wake`, `Deadline`, `ProgressError`, `take_error` ; état persistant ; initialisation paresseuse I-3 ; algorithme complet de la spéc. § 5.4 (conservation, plafond K, abandon R1) ; transitions terminales présentes, couvertes par un cas par bras.
- **Prérequis** : T1, T2, T3, T4. **Statut** : séquence interne.
- **Fichiers** : créés `src/progressor.rs`, `tests/temporal_progress.rs`, `tests/temporal_support/mod.rs` (harnais d'horloge scriptée **refusant toute suite décroissante à sa construction** ; inclus par `#[path]`, précédent `script_support`, réutilisé par T6 et T7) ; `src/lib.rs` (exports).
- **Invariants** : C-3 à C-14 ; dispatch statique (aucun trait public, `dyn`, `Box`) ; aucune allocation dans `progress()` ni `take_error()` ; `progress()` ne rend jamais un `Result`.
- **Tests** : TP-P (P0, P0b, P1 à P8), TP-S (S1 à S5, S6a, S6b, S7), TP-E : E1, E3 (un cas par bras terminal), **E7, E8, E9**. Lignes de la spéc. § 10 : chacune repart d'un `Progressor` neuf suivi de Σ0. Rouge contre squelette, puis vert. Familles de mutation : `>` / `>=` sur M, réancrage sur la grille initiale, échéances nées pendant l'exécution comptées dans l'ensemble exécuté, deuxième lecture d'horloge omise, état terminal non stable.
- **Décisions d'exécution** : PO-1 (avancement de `next` après une erreur postérieure à l'invocation ; proposition de la spéc. : non avancé, sans effet observable car terminal).
- **Acceptation** : toutes les lignes attendues ; compteur d'itérations conservé sur erreur (E7) ; quatre contrôles ; Miri ; MSRV.
- **Commits** : (1) types, construction, initialisation I-3 ; (2) boucle, plafond K, conservation ; (3) abandon R1 ; (4) bras terminaux et E7/E8/E9.
- **Alerte de nommage** : `Phase` entre en collision avec le vocabulaire normatif (« phase » = unité de cohérence du `World`, ignorée du Shell par ADR-S01 pt 3). Nom provisoire selon la spéc. § 12.2 ; renommage à proposer à l'exécution.

### T5b — `Progressor` : arrêt, erreur, acquisition

- **Objectif** : garanties de S02 pt 4, 7 et S03 pt 9.1 sur le `Progressor` déjà implémenté.
- **Prérequis** : T5a.
- **Fichiers** : `tests/temporal_progress.rs` (extension) ; correction minimale de `src/progressor.rs` **seulement** si un test révèle un écart.
- **Tests** : E2 (arrêt après j itérations, une seule lecture, aucune décision), E4 (échec de publication), E5 (dépôt pendant une itération d'une progression de 3 ; lots vides comptés, aucun lot rejoué), E6, E10 (`take_error` deux fois), **D9** (arrêt demandé quand D est vide : non observé, observé à la progression suivante qui exécute une itération). Les tests passent probablement dès l'écriture : **sensibilité par mutation** (omission de la coupure, lot rejoué, `take_error` clonant, arrêt observé quand D vide).
- **Acceptation** : E2, E4, E5, E6, E10, D9 verts ; toute mutation détectée ou établie équivalente.
- **Commit** : 1.

### T5c — Conformité structurelle et rejeu du contrat historique

- **Objectif** : prouver que le `Progressor` reproduit, par une échéance due à chaque progression, les mêmes lots, arrêts et première erreur que `run_loop`, et que le contrat statique est tenu.
- **Prérequis** : T5a (T5b validé de préférence).
- **Fichiers** : `tests/temporal_progress.rs` (extension) ou fichier dédié (à fixer à l'exécution).
- **Tests** : **D8** (scénarios LC, AQ, RA rejoués via `Progressor` ; **PC reste rattaché à `run_loop`**, non migré [Arbitré]) ; **D7** (`[dependencies]` vide, aucun trait public, aucun `dyn`/`Box`, aucun type de fenêtrage) par revue et `cargo tree`.
- **Acceptation** : D7, D8 verts ; trois ensembles de tests **explicitement distincts** : contrat historique (`run_loop`), `Progressor`, équivalence entre pilotes.
- **Commit** : 1.
- **Porte P-B** : T5a, T5b, T5c validées.

### T6 — Pilote bloquant à politique

- **Objectif** : fonction distincte de `run_loop` : `progress()` puis attente jusqu'à `wake` par une fermeture **injectée** ; rend le total sur arrêt, l'erreur (`take_error`) sur échec (spéc. § 9.2).
- **Prérequis** : T5a, T5b.
- **Fichiers** : créés `src/driver.rs`, `tests/temporal_drivers.rs` ; `src/lib.rs` (export).
- **Tests** : D2 (réveil déjà passé : attente nulle, aucune boucle active supplémentaire) ; moitié « pilote bloquant » de D1 (résultats comparés à des littéraux). Une fermeture d'attente par défaut (dormir `deadline − now`, jamais en négatif) est fournie ; son calcul est testé avec horloge scriptée ; le sommeil réel n'est exercé que par les exemples.
- **Acceptation** : aucun `sleep` ni horloge murale dans les tests ; aucune copie de la logique K/M ; `run_loop` intact (D4 rejoué) ; MSRV.
- **Commit** : 1.

### T8 — Exemple conforme S04 (optionnelle)

- **Objectif** : exemple headless distinct des exemples historiques, qui **ne démontrent pas** la conformité S04.
- **Prérequis** : T5, T6. **Fichiers** : créé `examples/policy_headless.rs`.
- **Acceptation** : code de sortie 0 ; horloge et attenteur scriptés ; valeurs de test (q = 10, K = 3, M = 5) ; aucune valeur de jeu.
- **Commit** : 1. **Porte P-C** : T6 (et T8 si retenue) validées.

### T7 — Harnais du pilote événementiel

- **Objectif** : simuler le `about_to_wait` d'une boucle de plateforme sans `winit` ; établir l'équivalence des pilotes (S02 pt 3).
- **Prérequis** : T5, T6.
- **Fichiers** : `tests/temporal_drivers.rs` (extension) ; aucun module de production attendu.
- **Tests** : D1 **complet** (même script d'observations, mêmes lots, mêmes invocations, même issue, même `progress()`) ; D3 (`Deadline → Instant` par `checked_add`) ; D6 (dépôt depuis un autre fil entre deux progressions, jointure, sans `sleep`).
- **Points d'attention** : D6 est le **premier test à fil réel** du Shell (les étapes 3 à 6 s'en abstenaient) : comportement sous Miri à constater ; PO-3 (échéance non représentable en `Instant`) est une question d'adaptateur natif, hors de ce harnais.
- **Acceptation** : équivalence établie ; aucun type de plateforme.
- **Commit** : 1. **Porte P-D**.

### V2-0 — Plan V2 (véhicule documentaire)

- **Objectif** : produire le plan du chantier V2, **pleinement soumis à G-CM**, validé avant toute modification de `VMAN`, d'Assets ou de la Forge (D-1c).
- **Prérequis** : aucun. **Statut** : démarrable. **Nature** : documentaire.
- **Contenu requis** : périmètre documentaire explicite ; **recensement complet** des fichiers et tests assertant des octets de manifeste (compléter § 3.1, y compris `tests/pipeline.rs`) ; critères d'acceptation ; vérifications G-CM ; reprise de VQ-1 à VQ-9 ; contraintes issues de V2-C ; traitement de R-ADAPT ; décision sur l'**enregistrement** des travaux (additifs aux comptes rendus Assets et Forge [Arbitré], ou document transversal) ; correspondance avec la chaîne plan → exécution → handoff de D-1c.
- **Acceptation** : validation par l'utilisateur = **P-V2**. Aucun code, aucun fichier d'artefact.
- **Non-décisions** : cette unité ne tranche ni le layout du paquet (V2-C) ni la règle des durées (D-N2).

### V2-C — Conception du paquet (porte de conception) [Arbitré]

- **Objectif** : établir, **avant tout code du chargeur v2**, les points suivants. Aucune abstraction publique ni trait n'est présumé : rechercher d'abord la représentation et les fonctions internes les plus simples.
  1. layout du paquet (en-tête, table d'accès aux artefacts, ordre d'octets, largeurs) ;
  2. accès aux artefacts **par index** ;
  3. limites de représentation et contrôles d'offset (débordement, chevauchement, longueur exacte, conversions vérifiées) ;
  4. **absence d'allocation** dans le chargement `no_std` : `load` attend un `&[&[u8]]` qu'un chargeur à entrée unique ne peut pas construire sans allocation ;
  5. **réutilisation des niveaux 1 à 4 sans duplication** : une seule écriture de leur logique, ordre, tout-ou-rien et absence de dépôt avant la fin du niveau 3 préservés ;
  6. refus croisés : manifeste v2 par le chemin historique (VQ-9b), manifeste v1 par le chemin v2 (VQ-9d), absence de chemin public v2 à deux entrées (VQ-9a) ;
  7. forme de lecture de q et variantes d'erreur additives ;
  8. garantie que le constructeur de paquet est l'unique producteur (VQ-9c).
- **Contraintes déjà établies à fermer** : `Manifest::parse` doit rester **strictement v1** (le test `m4` épingle la version `0x0002` → `Version` ; l'étendre en place serait un arrêt R-ADAPT) : le lecteur v2 est additif. Le chemin historique refuse déjà v2 par `ManifestError::Version`. Les littéraux v1 (tests d'Assets, de la Forge, doctests LT-5n/5t) restent intacts (V-a).
- **Sortie** : note de conception annexée au plan V2, validée = **P-CONC**.
- **Prérequis** : V2-0. **Statut** : conditionnel.

### V2-L — Lecteur v2 du manifeste (`verbum-assets`)

- **Objectif** : lire `VMAN` v2 (`[signature][version][q_ns u64][entry_count][entrées]`, en-tête de 18 octets, longueur 18 + 16 n) ; q porté une seule fois ; q = 0 refusé au niveau 1.
- **Prérequis** : P-V2, P-CONC. **Statut** : bloqué (G-CM).
- **Fichiers** (prévisions, à confirmer en V2-0) : `crates/engine/assets/src/manifest.rs` (constantes et lecture **additives**), `lib.rs` (exports).
- **Tests** : VQ-1 (littéral v2 écrit à la main), VQ-2 (niveau 1), VQ-3 (longueur exacte, troncature, surplus, `n` incohérent, ordre d'octets explicite), VQ-4 (un v1 ne produit jamais de q) ; tests `m1` à `m8` **inchangés** (VQ-7). Familles de mutation : endianness de `q_ns`, longueur d'en-tête, q nul accepté, v1 accepté comme v2.
- **Acceptation** : VQ-1 à 4 et 7 verts ; `verbum-assets` sans dépendance ; aucune allocation.

### V2-P — Paquet et chargeur v2 (`verbum-assets`)

- **Objectif** : format du paquet, accès par index, chargeur v2 réutilisant les niveaux 1 à 4 (V2-C) ; refus croisés.
- **Prérequis** : V2-L. **Statut** : bloqué (G-CM).
- **Fichiers** (prévisions) : nouveau module de paquet, `load.rs` (**refactorisation interne seulement si V2-C la retient** ; niveaux, ordre, atomicité inchangés ; toute modification d'assertion = arrêt), `lib.rs`.
- **Tests** : VQ-9 (a, b, d), VQ-6 (ordre et nombre des niveaux, tout-ou-rien, absence d'allocation, `cargo tree` inchangé) ; `l1` à `l5`, `sf*`, `rf*` **inchangés**. Familles de mutation : dépôt avant le niveau 3, niveaux inversés, offset hors limite accepté, chevauchement accepté, entrée unique contournée.
- **Acceptation** : VQ-6 et VQ-9 verts ; Miri `verbum-assets` intégral.

### V2-F — Voie de compilation additive (`verbum-forge`)

- **Objectif** : recevoir q explicitement ; produire le paquet (unique producteur) ; refuser q = 0 ; **laisser `compile_assets` et son doctest inchangés** ; production v1 octet pour octet identique.
- **Prérequis** : V2-P. **Statut** : bloqué.
- **Fichiers** (prévisions) : `batch.rs` ou nouveau module (additif), `lib.rs`, `sprite.rs` (variantes de `ForgeError`), `tests/pipeline.rs` (additif).
- **Tests** : VQ-2 (frontière Forge), VQ-5 (aller-retour Forge → paquet → lecture ; v1 identique à `compile_assets` ; invariance par permutation ; q différent → manifeste différent, artefacts identiques), VQ-9c. Mutations : q non écrit, q écrit deux fois, v1 altéré, paquet recomposable hors constructeur.
- **Acceptation** : VQ-2, 5, 9c verts ; `cargo tree -i toml --workspace` inchangé ; Miri Forge partiel (exclusions connues de l'étape 5, à rapporter comme tel).

### V2-Z — Clôture V2

- **Objectif** : régression globale et consignation.
- **Contenu** : VQ-6 et VQ-7 sur l'ensemble ; consommateurs de § 3.1 verts ; `cargo tree` (Assets vide, Forge, `toml` isolé) ; Miri et MSRV ; consignation additive dans les comptes rendus Assets et Forge ; liste « non démontré ».
- **Porte P-E** : V2-Z validée.

### F2 — Conversion des durées humaines en ticks (conditionnelle)

- **Statut** : bloqué par l'**arbitrage D-N2** (rejet, quantification déterministe ou autre) et par V2-F.
- **Ce que la roadmap fixe sans choisir** : S04 pt 2 exige une règle déterministe explicite ; aucune grammaire de durée humaine n'existe aujourd'hui (les scripts n'ont que `wait.ticks`) ; F2 introduit donc **à la fois** la règle et son premier consommateur. F2 touchera très probablement `forge/src/script.rs`, l'aiguillage de `batch.rs` et les tests `SCR-F*`, **sources issues de l'étape 6** : modifications additives uniquement, assertions existantes intactes, sous arbitrage explicite (§ 10.2).
- **Acceptation** (à préciser après arbitrage) : règle documentée et testée sur des cas entiers, non entiers et limites ; déterminisme.

### F3a — Promotion de `games/demo` au workspace

- **Objectif** : promouvoir `crates/games/demo` au niveau Application, avec les responsabilités de l'ADR-W01 ; unité explicite [Arbitré].
- **Prérequis** : décision D-N5 (nom, contenu, production du paquet). **Statut** : conditionnel.
- **Fichiers** (prévisions) : `Cargo.toml` racine (`members`, et `[workspace.dependencies]` pour `verbum-shell`, absent aujourd'hui) ; créés `crates/games/demo/Cargo.toml` (héritage du workspace, `[lints] workspace = true`) et un squelette minimal ; `README.md` mis à jour.
- **Tests / contrôles** : graphe ADR-W01 : `cargo tree -i verbum-demo --workspace` sans dépendant ; aucun crate Engine ni Extension ne dépend de l'Application ; `cargo tree -i verbum-shell --workspace` ne montre plus que l'Application ; quatre contrôles ; MSRV.
- **Acceptation** : le crate se construit, sans logique ; aucun changement de comportement ailleurs.

### F3b — Provenance de q

- **Objectif** : l'application charge son paquet v2 et en obtient q **uniquement** par ce chemin ; construit `TemporalPolicy` ; erreur typée (v1, q absent) **avant toute progression**.
- **Prérequis** : T1, V2-Z, F3a.
- **Tests** : **VQ-8** (Forge avec q → paquet → chargement → politique, `TemporalPolicy` porte le q du paquet ; v1 → erreur typée) ; revue : q n'est jamais fourni ailleurs que par le paquet. La règle « jamais silencieux » repose sur le chemin d'assemblage et ces tests, pas sur un verrou de type (spéc. § 7.2).
- **Acceptation** : VQ-8 vert ; aucune valeur de q par défaut ni constante d'application.

### F3c — Assemblage sous pilote à politique

- **Objectif** : assembler `World`, `Scheduler`, `Inbox`, `Progressor`, pilote bloquant à politique ; exemple headless de code de sortie 0.
- **Prérequis** : F3b, T5, T6, décision D-N1 (q, K, M de composition).
- **Tests** : acceptation de bout en bout (q du paquet → itérations dues selon l'horloge scriptée) ; l'exemple distingue valeurs de test et valeurs de composition.
- **Acceptation** : P-F. Aucun type de fenêtrage ; aucune résolution de R03/S05.

### G1 — Validation intégrée

- **Contenu** : matrice ADR-S04 (C-1 à C-16) → tests ; non-régression de tous les consommateurs de § 3.1 ; graphe de dépendances (Shell sans dépendance normale ; Application → Engine uniquement) ; Miri et MSRV ; liste « non démontré » ; **liste des exigences transmises à l'intégration native** (S02 pts 8 et 9, S04 pt 11, PO-3, expériences E1, E2, E8, E10, E14, E15 non exécutées).
- **Acceptation** : rapport de clôture validé = **P-G**.

## 9. Portes de validation

| Porte | Objet | Condition |
|---|---|---|
| **G-REG** | non-régression de `run_loop` | suites historiques de § 3.1 vertes, **aucune assertion modifiée**, signature inchangée, à chaque unité du chantier A |
| **G-DEP** | pureté du Shell | `[dependencies]` vide ; aucun trait public, `dyn` ni `Box` ajouté ; pas d'`unsafe` ; pas de type de fenêtrage |
| **G-MSRV** | Rust 1.85 | `cargo +1.85.0 build --workspace` puis `cargo +1.85.0 test --workspace` (deux commandes) ; pas de montée de version |
| **P-V2** | autorisation de toucher `VMAN`, Assets, Forge | plan V2 validé (et chaîne de D-1c : exécution et handoff V2 validés avant le code) |
| **P-CONC** | conception du paquet | note de V2-C validée |
| **G-CM** | mécanismes de l'étape 5 | arrêt avec rapport si : niveau créé ou supprimé, ordre modifié, dépôt avant la fin du niveau 3, layout `VMAN` v1 modifié, octets v1 changés, allocation dans `load`, modèle d'identité modifié |
| **R-ADAPT** | tests des étapes antérieures | seules des modifications mécaniques de signature, consignées une à une ; toute assertion modifiée = arrêt |
| **G-ARB-F2** | durées non entières | arbitrage D-N2 rendu avant tout code de F2 |
| **G-W01** | graphe Engine/Extension/Application | `cargo tree -i` : aucun crate Engine ni Extension ne dépend de l'Application |
| **G-S6** | étape 6 close | aucune modification de ses documents ou sources ; toute incompatibilité concrète est signalée comme dépendance et soumise à arbitrage |

Contrôles propres au chantier A à chaque unité : `cargo tree -p verbum-shell -e normal` (aucune dépendance) ; Miri : `cargo +nightly miri test -p verbum-shell` ; diff de `lib.rs` limité à la documentation et aux exports ; doctests LT-3 à LT-5t octet pour octet.

**Interdictions communes** : implémenter avant la validation de la roadmap et l'instruction explicite « démarrez l'unité … » ; rouvrir un arbitrage clos ; modifier un ADR ; ajouter une dépendance à `verbum-shell` ; introduire un flottant, un `Instant` dans le `Progressor`, une durée ou un instant dans le `World` ; `sleep` ou horloge murale dans un test ; q, K ou M fournis par appel de progression.

## 10. Risques et dépendances non résolus

### 10.1 Divergences avec la spécification r3

| # | Écart | Justification |
|---|---|---|
| D-1 | T2 n'a pas de dépendance de code vers T1 | fonctions sur types `NonZero` natifs |
| D-2 | `TimeError` défini en T2 (variante `ScheduleOverflow`), étendu en T3 ; la spéc. le place dans `clock.rs` | décision utilisateur : dépendance réelle plutôt que localisation initiale |
| D-3 | T5 découpée en T5a, T5b, T5c | unité d'examen isolé ; le code de T5a couvre ses propres branches, T5b et T5c apportent des preuves complémentaires |
| D-4 | D1 répartie sur T6 (moitié bloquante) et T7 (équivalence) | l'équivalence exige les deux pilotes |
| D-5 | Périmètre de non-régression étendu à `script_loaded.rs`, `level_loaded.rs`, `script_suspend.rs`, `borrowed_resource.rs` ; PC reste sur `run_loop` | spéc. F-4 les omettait ; décision utilisateur |
| D-6 | F3 devient F3a/F3b/F3c, localisée dans `crates/games/demo` ; F3b dépend de T1 et V2-Z, F3c de T5 et T6 (la spéc. ne citait que T1 et F1) | ADR-W01 ; l'exécution sous politique exige le `Progressor` et le pilote |
| D-7 | VQ-8 déplacé en F3b | il traverse Forge, Assets et `TemporalPolicy` |
| D-8 | Le lecteur v2 est **additif** ; `parse` reste v1 | le test `m4` épingle `0x0002` → `Version` (R-ADAPT) |
| D-9 | F1 découpée en V2-0, V2-C, V2-L, V2-P, V2-F, V2-Z | décision utilisateur (porte de conception) et petites unités sous G-CM |
| D-10 | PO-2 résolu : `SCRIPT_MAX_STEPS_PER_TICK = 16` (provisoire, indépendante de K) | lecture de `script.rs` |
| D-11 | E11 vérifié en T1 au niveau politique, reconfirmé par la signature en T5a | `Progressor` n'existe pas en T1 |
| D-12 | Harnais d'horloge scriptée explicite (`temporal_support`) | partagé par T5 à T7 ; la spéc. ne le localisait pas |

### 10.2 Risques

| # | Risque | Atténuation |
|---|---|---|
| RK-1 | T4 modifie du code existant : régression silencieuse de `run_loop` | G-REG, unité isolée et première, sensibilité par mutation ; mutation possiblement non détectée faute de test historique (§ 8, T4) |
| RK-2 | Chargement v2 sans allocation incompatible avec la signature de `load` | porte de conception V2-C ; aucune abstraction publique présumée |
| RK-3 | Refactorisation de `load.rs` jugée « changement de mécanisme » | G-CM, tests `l*`/`sf*`/`rf*` inchangés comme preuve |
| RK-4 | F2 touche des sources de l'étape 6 (`forge/src/script.rs`, tests `SCR-F*`) | modifications additives sous arbitrage explicite ; sinon F2 reste bloquée |
| RK-5 | q, K, M de composition non choisis | D-N1 avant F3c ; les tests restent sur valeurs de test |
| RK-6 | Premier test à fil réel (D6) | jointure sans `sleep` ; comportement sous Miri constaté |
| RK-7 | Collision de nom `Phase` | renommage proposé à l'exécution de T5a |
| RK-8 | Valeur de `SCRIPT_MAX_STEPS_PER_TICK` confondue avec K | distinction rappelée (spéc. § 8) |
| RK-9 | Comptes rendus additifs dans des documents d'étapes closes | additif seulement, sections datées, sur décision explicite de l'utilisateur déjà consignée |
| RK-10 | PO-3 (`Deadline` non représentable en `Instant`) | traité à l'adaptateur natif ; hors chantier |

### 10.3 Décisions à soumettre (aucune n'est tranchée ici)

| # | Décision | Bloque |
|---|---|---|
| **D-N1** | valeurs q, K, M de composition de `games/demo` | F3c |
| **D-N2** | règle des durées non entières, périmètre de F2, autorisation de modifier des sources de l'étape 6 ; mise à jour documentaire probable (plan V2 ou note Forge), ADR non nécessaire a priori, à confirmer | F2 |
| **D-N3** | véhicule d'enregistrement de V2 ; nom et numéro du plan V2 | V2-0 |
| **D-N4** | validation du layout du paquet (sortie de V2-C) | V2-L, V2-P |
| **D-N5** | portée de `games/demo` : consommateur (script ou système minimal), production du paquet dans l'application (dev-dependency de la Forge, exemple, autre), nom du crate | F3a |
| **D-N6** | granularité des sessions d'implémentation (défaut proposé : une unité par session) | handoff |

**Nouvelle décision architecturale indispensable : aucune identifiée.** La dépendance R03/S05 (« 0..N itérations pour 0..1 frame ») n'est pas incontournable pour ce chantier : F3c est headless.

## 11. Critères de fin de chantier

| Jalon | Condition |
|---|---|
| **J-A** — cœur temporel conforme (valeurs de test) | P-A, P-B, P-C, P-D franchies ; matrice C-1 à C-16 couverte ; consommateurs historiques verts ; Miri et MSRV exécutés par l'utilisateur |
| **J-V2** — provenance de q | P-V2, P-CONC, P-E ; VQ-1 à VQ-7 et VQ-9 verts ; octets v1 identiques ; aucune assertion des étapes 2, 5, 6 modifiée |
| **J-F3** — composition applicative | P-F ; `games/demo` membre du workspace, graphe ADR-W01 conforme ; q lu du paquet uniquement ; VQ-8 vert |
| **J-F2** — durées non entières | P-E′ (ou décision explicite de livrer F2 séparément) |
| **Fin complète** | J-A, J-V2, J-F3, J-F2 et P-G ; liste « non démontré » consignée ; **l'intégration native reste hors chantier** |

Une clôture partielle « conforme S04 sans F2 » n'est possible que sur décision de l'utilisateur, car ADR-S04 pt 2 exige une règle explicite pour les durées non entières.
