# Handoff — implémentation de la politique temporelle du Shell (ADR-S04)

Document de passation, **non normatif**. Ce n'est ni une spécification ni un ADR. Il prépare l'ouverture d'une session Claude dédiée à l'implémentation. Il est cohérent avec `03-verbum-shell-temporal-policy-roadmap.md` (r1, 13 novembre 2026) et `03-verbum-shell-temporal-policy.md` (r3).

**Hiérarchie documentaire** :
1. ADR adoptées : références normatives.
2. Spécification d'implémentation validée (`03-verbum-shell-temporal-policy.md`, r3) : référence technique du chantier pour les comportements et algorithmes. Elle peut rester non normative au regard des ADR.
3. Roadmap validée : référence pour l'ordre, les dépendances, les unités et les portes de validation.
4. Handoff : document de passation non normatif.

La roadmap ne peut pas modifier un algorithme de la spécification. En cas de divergence entre documents, la session la signale et la soumet à arbitrage ; elle ne la tranche pas.

**Aucune implémentation n'est autorisée par ce document.** Elle ne démarre que sur l'instruction explicite de l'utilisateur : « démarrez l'unité T4 » (ou toute autre unité nommée), après validation de la roadmap.

## 0. Consigne d'amorce (à coller au début de la session)

> Vous implémentez la politique temporelle du Shell de Verbum (ADR-S04), dans `crates/engine/shell`. Lisez, dans cet ordre : ce handoff, puis `docs/implementation/03-verbum-shell-temporal-policy-roadmap.md` (roadmap, à lire une seule fois en entier), puis `docs/implementation/03-verbum-shell-temporal-policy.md` (spécification r3 : § 4, 5, 6, 9, 10 et 11 pour l'unité en cours), puis `docs/architecture/adr/ADR-S04-shell-s-temporal-policy.md`, `ADR-S02-loop-pattern-and-shell-control.md` (points 2, 3, 4, 6, 7), `ADR-S03-event-acquisition.md` (points 3, 6, 7, 12) et `tree.md`. Les autres ADR ne se relisent que si une unité les cite. Ne demandez pas le dépôt en entier : demandez par nom de fichier ce que l'unité liste (§ 4). N'écrivez aucun code avant que l'utilisateur ait dit : « démarrez l'unité Tn ». Accusez réception en cinq lignes maximum.

## 1. Références

**Normatif** : `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`, ADR-000, ADR-004, ADR-A01, ADR-S01, ADR-S02, ADR-S03, **ADR-S04 (adoptée le 11 novembre 2026)**, ADR-W01.

**Planification** : roadmap r1 ; spécification r3 ; plans et comptes rendus `03-verbum-shell.md` et `03-verbum-shell-execution.md` (suivi du chantier A), `05-verbum-assets*.md` (G-CM, R-ADAPT, suivi Assets), `02-verbum-forge*.md` (suivi Forge). L'étape 6 (`06-verbum-scripts*.md`) est **close et hors périmètre de modification**.

## 2. Décisions définitivement arbitrées

| # | Décision | Origine |
|---|---|---|
| 1 | Étape 6 close ; aucune modification de ses documents ni sources ; ses consommateurs (`script_loaded.rs`, `level_loaded.rs`, etc.) sont en non-régression | utilisateur, 13 nov. |
| 2 | L'assemblage F3 vit dans `crates/games/demo` (ADR-W01), promu au workspace par une unité explicite F3a ; les tests et exemples du Shell utilisent des valeurs synthétiques | utilisateur, 13 nov. |
| 3 | Pas de document d'exécution supplémentaire ; T1 à T8 suivis dans une **section distincte « ADR-S04 »** de `03-verbum-shell-execution.md` ; V2 consigné dans les comptes rendus Assets et Forge ; véhicule transversal éventuel décidé en V2-0 | utilisateur, 13 nov. |
| 4 | Unité de conception préalable V2-C (layout du paquet, accès par index, offsets, absence d'allocation, réutilisation des niveaux 1 à 4) **avant** tout code du chargeur v2 ; aucune abstraction publique ni trait présumé | utilisateur |
| 5 | `TimeError` défini minimalement en T2, étendu en T3 | utilisateur |
| 6 | `script_loaded.rs` et `level_loaded.rs` dans l'inventaire de non-régression ; les tests PC restent sur `run_loop` ; trois ensembles distincts : contrat historique, `Progressor`, équivalence des pilotes | utilisateur |
| 7 | MSRV Rust 1.85 conservé ; aucune montée implicite | utilisateur |
| 8 | q de test (T1 à T8) distinct de q du paquet (V2, F3) | spéc. D-1, R-1 |
| 9 | Provenance de q : `VMAN` v2 avec `q_ns: u64` après la version, avant `entry_count`, en-tête de 18 octets ; paquet unique (M-1) ; compatibilité V-a (v1 intact, mêmes octets pour un ensemble donné) ; chantier V2 sous G-CM | spéc. D-1, D-1b, D-1c, D-1d, R-1 |
| 10 | `Progressor` concret, public, générique, à champs privés, dispatch statique ; erreurs typées terminales ; `run_loop` conservé à cadence externe (option B) ; initialisation paresseuse I-3 ; arrêt non observé quand aucune échéance n'est due | spéc. D-2, D-3, I-3, § 9 |

## 3. Roadmap retenue

### 3.1 Unités et ordre

| Ordre | Unité | Dépend de | Statut |
|---:|---|---|---|
| 1 | **T4** extraction du cœur d'itération | — | Démarrable |
| 2 | **T1** `TemporalPolicy` | — | Démarrable |
| 3 | **T2** arithmétique d'échéancier (+ `TimeError` minimal) | — | Démarrable |
| 4 | **T3** horloge (+ extension de `TimeError`) | T2 | séquence interne |
| 5–7 | **T5a**, **T5b**, **T5c** `Progressor` | T1–T4 ; T5a ; T5a | séquence interne |
| 8 | **T6** pilote bloquant à politique | T5a, T5b | séquence interne |
| 9 | T8 exemple conforme S04 (optionnelle) | T5, T6 | séquence interne |
| 10 | **T7** harnais événementiel | T5, T6 | séquence interne |
| ∥ | **V2-0** plan V2 (documentaire) | — | Démarrable |
| ∥ | **V2-C** conception du paquet | V2-0 | Conditionnel |
| 11–14 | V2-L, V2-P, V2-F, V2-Z | P-V2 et P-CONC | **Bloqués (G-CM)** |
| 15 | F2 durées non entières | V2-F, arbitrage D-N2 | **Bloqué** |
| 16–18 | F3a, F3b, F3c | D-N5 ; T1, V2-Z, F3a ; F3b, T5, T6, D-N1 | Conditionnel / Bloqués |
| 19 | G1 validation intégrée | tout | — |

### 3.2 Divergences avec la spécification r3 (résumé ; détail : roadmap § 10.1)

T2 indépendante de T1 ; `TimeError` défini en T2 ; T5 découpée en trois unités ; D1 répartie sur T6 et T7 ; non-régression étendue ; F3 découpée et localisée dans `games/demo` ; VQ-8 déplacé en F3b ; lecteur v2 additif (le test `m4` épingle la version `0x0002`) ; F1 découpée en six unités ; PO-2 résolu (`SCRIPT_MAX_STEPS_PER_TICK = 16`, provisoire).

### 3.3 Complément T4 : test de conformité S02 « arrêt avant coupure »

La fiche T4 de la roadmap inclut un test ciblé de l'invariant S02 : un arrêt observé avant une itération empêche toute coupure de l'`Inbox`. Il vérifie : arrêt observé avant la première itération ; aucune coupure de l'`Inbox` ; aucune invocation de l'infrastructure ; aucune itération comptabilisée ; contenu de l'`Inbox` conservé après l'arrêt. C'est un ajout de conformité : aucune assertion historique n'est modifiée. Il ferme la mutation « arrêt observé après la coupure », jusque-là possiblement non détectée.

## 4. Fichiers à fournir, par unité

La session demande par nom de fichier ; elle ne reçoit jamais le dépôt entier.

| Unité | Fichiers |
|---|---|
| **T4** | `crates/engine/shell/src/{shell,lib,pacing,inbox}.rs`, `Cargo.toml` du Shell ; `tests/{lifecycle,pacing,acquisition,acceptance,frame_loop,script_loaded,level_loaded,borrowed_resource}.rs` ; `examples/{console,headless_frame,script_headless,level_headless}.rs` (lecture, pour la non-régression) |
| **T1** | `src/lib.rs`, `Cargo.toml` du Shell |
| **T2, T3** | idem + fichiers créés par les unités précédentes tels que rendus |
| **T5a–T5c, T6, T7, T8** | fichiers de l'unité précédente tels que rendus ; `tests/script_support/mod.rs` (modèle de module de test partagé) |
| **V2-0** | roadmap, spéc. § 7, `02-verbum-forge*.md`, `05-verbum-assets*.md` (G-CM, R-ADAPT), `manifest.rs`, `load.rs`, `batch.rs`, `tests/pipeline.rs` de la Forge, tests d'Assets et de la Forge assertant des octets de manifeste |
| **V2-C** | plan V2 validé ; `manifest.rs`, `load.rs`, `format.rs`, `header.rs`, `lib.rs` d'Assets ; `batch.rs` de la Forge |
| **F3a** | `Cargo.toml` racine, `crates/games/demo/README.md`, ADR-W01 |

## 5. Dépendances et portes

- **G-REG** : suites historiques du Shell inchangées et vertes, signature de `run_loop` inchangée, à chaque unité du chantier A.
- **G-DEP** : `[dependencies]` de `verbum-shell` vide ; aucun trait public, `dyn`, `Box` ; pas d'`unsafe` ; aucun type de fenêtrage.
- **G-MSRV** : `cargo +1.85.0 build --workspace` puis `cargo +1.85.0 test --workspace`.
- **P-V2** et **P-CONC** : avant tout code d'Assets ou de la Forge pour V2 ; **G-CM** et **R-ADAPT** (arrêt, rapport, aucun contournement).
- **G-ARB-F2**, **G-W01** (graphe Engine/Extension/Application), **G-S6** (étape 6 intacte).
- Portes de phase : P-A (T4, T1, T2, T3), P-B (T5), P-C (T6), P-D (T7), P-E (V2-Z), P-F (F3), P-G (G1).

Contrôles à chaque unité, **exécutés par l'utilisateur** : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check` ; `cargo tree -p verbum-shell -e normal` ; `cargo +nightly miri test -p verbum-shell` ; MSRV en deux commandes. La session distingue ses résultats d'espace de travail des résultats du dépôt réel.

## 6. Ordre de démarrage

1. **T4** : seule unité modifiant du code existant ; établit G-REG avant que d'autres modules s'ajoutent.
2. **T1**, puis **T2**, puis **T3**.
3. En parallèle, sur décision de l'utilisateur : **V2-0** (documentaire), à lancer tôt car sa validation est de plus longue échéance.
4. **T5a, T5b, T5c**, puis **T6**, **T7** (T8 optionnelle).

Rien d'autre ne démarre avant sa porte. Une session par unité est la pratique par défaut recommandée, **non une obligation adoptée** (D-N6, ouvert, **ne bloque pas T4**) : de petites unités peuvent être regroupées sur décision de l'utilisateur si leurs dépendances et leur contexte le permettent. Dans une même session, l'unité suivante ne commence que sur « unité Tn validée, démarrez Tn+1 ».

## 7. Règles de session

1. Une unité à la fois ; rapport de fin d'unité au gabarit de `03-verbum-shell-execution.md` § 5 ; arrêt en attente de validation.
2. Tests d'abord contre des squelettes ; **un échec de compilation n'est pas un rouge** ; sans rouge possible : sensibilité par mutation sur copie isolée hors de l'arbre de travail, une mutation à la fois. Une mutation non détectée et non établie équivalente est un arrêt.
3. Aucun résultat déduit ou estimé ; pas de commit par la session (elle propose un regroupement) ; `tree.md` régénéré par l'utilisateur.
4. Fichiers `.rs` livrés complets avec commentaire de chemin en première ligne ; documentation en français ; style : vouvoiement, concis.
5. Rust 1.85 dès l'écriture : pas de let-chains ; aucune constante utilisée seulement dans un `const _ assert!` ; `is_empty` avec tout `len`.
6. Valeurs de test : q = 10 ns, K = 3, M = 5, extrêmes q = 1 et `u64::MAX`, K = M = 1 et `u32::MAX`. Aucune valeur de jeu.
7. Tests sans horloge murale ni `sleep` ; harnais d'horloge refusant toute suite décroissante.

## 8. Exclusions et interdictions

- Ne pas rouvrir un arbitrage de § 2 ; ne pas modifier un ADR, `ROADMAP.md`, la spécification ou la roadmap.
- Ne pas modifier `crates/engine/{ecs,schedule}`, `crates/engine/render`, `crates/engine/verbum`, ni les documents ou sources de l'étape 6.
- Ne pas modifier une assertion d'un test existant ; ne pas migrer PC vers `Progressor` ; laisser intacts les doctests LT-3 à LT-5t.
- Ne pas modifier `Manifest::parse` pour v1 ; ne pas toucher `VMAN`, Assets ou la Forge avant P-V2 et P-CONC ; ne pas fixer le layout du paquet hors V2-C.
- Ne pas choisir la règle des durées non entières ; ne pas choisir q, K, M de composition.
- Pas de flottant, d'`Instant` dans le `Progressor`, de durée ou d'instant dans le `World`, d'`unsafe`, de trait public, de `dyn`, de `Box`.
- Ne pas implémenter l'intégration native (`winit`/`wgpu`), ni R03/S05, ni pause, suspension, reprise.
- Ne pas ajouter test ou API hors liste de la roadmap sans arbitrage (le test de conformité S02 de T4 est dans la liste).

## 9. Points ouverts à ne pas trancher

D-N1 (q, K, M de `games/demo`) ; D-N2 (durées non entières, périmètre de F2, sources de l'étape 6) ; D-N3 (véhicule d'enregistrement de V2) ; D-N4 (layout du paquet) ; D-N5 (portée et production du paquet de `games/demo`) ; D-N6 (granularité des sessions ; défaut recommandé : une unité par session, regroupement possible sur décision de l'utilisateur ; ne bloque pas T4). Questions d'exécution : PO-1 (avancement de `next` après erreur ; proposition : non avancé), PO-3 (`Deadline` non représentable en `Instant` : adaptateur natif), nom `Phase` (collision de vocabulaire). La mutation T4 « arrêt avant coupure » est fermée par le test de conformité du § 3.3.

## 10. Confirmation

**Aucune implémentation n'est encore autorisée.** Aucun code, test, `Cargo.toml`, ADR, format d'artefact ni commit n'a été produit par la mission de planification. La session d'implémentation attend la validation de la roadmap et l'instruction explicite de l'utilisateur.
