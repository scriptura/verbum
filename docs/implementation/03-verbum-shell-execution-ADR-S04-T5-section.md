# Section « ADR-S04 » — clôture de T5a, T5b, T5c (à ajouter à `03-verbum-shell-execution.md`)

Non normatif. Cette section est à **ajouter** à la section distincte « ADR-S04 » de `03-verbum-shell-execution.md` (handoff, décision 3). Le fichier d'exécution lui-même n'a pas été fourni à la session : il n'a donc pas été modifié et le point d'insertion est à choisir par l'utilisateur.

Convention : **[ER]** = exécuté par Claude sur son environnement réduit (crate `verbum-shell` isolé, sans les dev-dependencies du dépôt) ; **[DR]** = exécuté par Olivier sur le dépôt réel. Aucun résultat n'est attribué à l'autre acteur.

## T5a — `Progressor` nominal (validée)

- **Responsabilité** : objet persistant `Progressor<Ev, E, C, S, I>` (champs privés, dispatch statique) ; `progress()` selon l'algorithme de la spéc. § 5.4 (initialisation paresseuse I-3, plafond K, conservation du résiduel, abandon R1 par réancrage) ; compte rendu `Progress { iterations, wake, state }` ; `take_error()`.
- **Nommage (décision définitive)** : `ProgressState { Running, Stopped, Failed }` et champ `Progress.state`, en remplacement du nom provisoire `Phase` (collision avec la « phase » du `World`).
- **Livrables** : `src/progressor.rs` (créé), `src/lib.rs` (exports), `tests/temporal_progress.rs` (créé), `tests/temporal_support/mod.rs` (harnais d'horloge scriptée, refuse toute suite décroissante, compte les lectures) ; `Deadline::as_duration()` documenté comme décalage depuis l'origine de l'horloge. Livraison en quatre commits intermédiaires ; l'état final seul fait foi.
- **Résultats** : matrice P0..P8, S1..S7, E1, E3, E7, E8, E9 verte. **[ER]** et **[DR]** : contrôles de l'unité verts (validation de l'utilisateur).
- **Écart** : les attributs temporaires `expect(dead_code)` des états intermédiaires ont été retirés des fichiers finaux.

## T5b — Arrêt, erreurs, acquisition (validée)

- **Responsabilité** : garanties de S02 pt 4, 7 et S03 pt 9.1 sur le `Progressor`.
- **Livrables** : `tests/temporal_progress.rs` étendu (E2, E4, E5, E6, E10, D9 ; 33 tests au total). Aucune modification de `src/` par cette unité.
- **Résultats** : **[ER]** 33 tests verts en debug et en release, clippy, fmt, doc, `cargo tree -e normal` sans dépendance. **[DR]** après correction ci-dessous : « tous les contrôles verts ou résultat attendu » (communication de l'utilisateur).
- **Incident** : sur le dépôt réel, clippy a rapporté `unfulfilled lint expectation` (`schedule.rs:21`) : un état intermédiaire de T5a était resté dans le dépôt. Les fichiers finaux `schedule.rs` et `clock.rs` ne portent aucun attribut temporaire ; l'erreur a été reproduite **[ER]** par injection de l'attribut. Aucun `allow` de remplacement, `unfulfilled_lint_expectations` non neutralisé. Fichiers finaux relivrés, puis contrôles **[DR]** verts.
- **Règle retenue** : les livraisons séparent systématiquement fichiers finaux et états intermédiaires ; ces derniers ne sont plus réintégrés.

## T5c — Conformité structurelle et rejeu du contrat historique (validée)

- **Responsabilité** : D8 (LC, AQ, RA rejoués via `Progressor`, mêmes lots, arrêts et première erreur que `run_loop`) ; D7 (`[dependencies]` vide, aucun trait public, `dyn`, `Box`, `unsafe`, flottant ni type de fenêtrage, `Instant` confiné à `clock.rs` et `pacing.rs`).
- **Livrables** : `tests/temporal_equivalence.rs` (16 tests) et `tests/temporal_structure.rs` (3 tests), créés. Aucun fichier de `src/` ni `Cargo.toml` modifié. Trois ensembles distincts : contrat historique (`run_loop`), `Progressor` (`temporal_progress.rs`), équivalence entre pilotes (`temporal_equivalence.rs`). PC reste sur `run_loop`.
- **Équivalence différentielle** : `run_loop` contre `Progressor`, ticks 0..=4, dépôts avant/pendant, échec à chaque rang ou jamais : 1 + 18 + 243 + 2 916 + 32 805 cas, total vérifié par le test.
- **Résultats [ER]** : suite complète verte en debug et en release (lib 23, `stop_before_cut` 1, `temporal_equivalence` 16, `temporal_progress` 33, `temporal_structure` 3) ; clippy `--all-targets -D warnings`, fmt, doc `-D warnings`, `cargo tree -e normal` sans dépendance. Mutations (une à la fois, copie isolée) : 9 tuées sur 9 (`Progressor` : comptage, erreur, état ; `run_loop` : comptage, erreur ; `iterate` : arrêt inversé, lot vide, échec compté en succès) ; `if !stop()` tuée par la cible lib, délai de 200 s atteint avant les autres cibles. D7 : injection de `dyn`, `Box`, `pub trait`, `f64`, `Instant` hors module et d'une dépendance fait échouer le test attendu.
- **Résultats [DR]** : Olivier confirme que tous les contrôles prévus ont été effectués sur le dépôt réel et sont verts (confirmation de l'utilisateur ; détail par commande non transmis à la session).
- **Non démontré** : RA rejoué en forme de boucle seulement (`World`, `Scheduler`, `Frame` réels absents de l'environnement réduit ; `frame_loop.rs` reste sur `run_loop`) ; AQ-6 (capacité après coupure) non observable via `Progressor`, couvert par `acquisition.rs` ; D5 (coupure unique dans `iterate`) par revue ; G-REG, Miri, MSRV 1.85.0 et doctests non exécutés par Claude.
- **Arbitrage** : deux fichiers dédiés plutôt qu'une extension de `temporal_progress.rs` ; `#[allow(dead_code, reason = …)]` localisé à l'inclusion de `temporal_support`.
- **Commit** : 1.

## Porte P-B

T5a, T5b, T5c validées. Porte P-B satisfaite. Prochaine unité : T6 (à cadrer, non autorisée).
