# Handoff — reprise du chantier ADR-S04 à l'unité T6

Document de passation, **non normatif**, pour une nouvelle session Claude. Date : 14 novembre 2026.

## 1. Interdiction

**Aucune modification du code, des tests, des `Cargo.toml` ni des ADR avant l'instruction explicite de l'utilisateur** (par exemple « démarrez l'unité T6 »). La session commence par lire et cadrer. Pas de commit par la session. Aucune unité au-delà de T6 sans validation explicite de T6.

## 2. État

| Unité | État |
|---|---|
| T4 cœur d'itération, T1 `TemporalPolicy`, T2 arithmétique d'échéancier, T3 horloge | validées (porte P-A) |
| T5a `Progressor`, T5b arrêt/erreurs/acquisition, T5c conformité et rejeu | validées (porte P-B) |
| **T6 pilote bloquant à politique** | **prochaine unité, à cadrer** |
| T8 (optionnelle), T7, chantier B (V2, F2, F3), G1 | non démarrés |

## 3. Documents et hiérarchie

1. ADR adoptées (dont ADR-S01 à S04, ADR-W01) : normatif.
2. Spécification `03-verbum-shell-temporal-policy.md` r3 : référence technique (T6 : § 9.2).
3. Roadmap `03-verbum-shell-temporal-policy-roadmap.md` r1 : ordre, unités, portes (fiche T6 au § 8).
4. Le présent handoff : suffit pour les consignes opérationnelles de la session.

Références historiques complémentaires, **non requises en lecture** : `03-verbum-shell-temporal-policy-implementation-handoff.md` (handoff d'implémentation antérieur) et `03-verbum-shell-execution.md` (journal de la première implémentation du Shell).

En cas de divergence : signaler et soumettre à arbitrage, ne pas trancher.

## 4. API en place (`crates/engine/shell`)

- `TemporalPolicy::from_nanos(u64, u32, u32)` (q, K, M) ; accesseurs `quantum_ns()`, `budget()`, `threshold()`.
- `MonotonicClock::{start, origin, elapsed}`.
- `Progressor<Ev: Copy, E, C, S, I>::{new(policy, clock, stop, inbox, infrastructure), progress(), take_error()}` ; il possède l'`Inbox`, sans accesseur.
- `Progress { iterations: u32, wake: Wake, state: ProgressState }` ; `ProgressState { Running, Stopped, Failed }` (nom définitif) ; `Wake { At(Deadline), NoDeadline }` ; `Deadline::as_duration()` = décalage depuis l'origine de l'horloge, pas une durée restante.
- `ProgressError<E> { Infrastructure(E), Time(TimeError) }` ; `take_error` rend l'erreur une seule fois, sans `Clone`.
- `iterate()` (`pub(crate)`) : cœur unique ; `run_loop` inchangé (option B), hors politique S04.
- Tests existants : `lifecycle`, `acquisition`, `frame_loop` (historiques, `run_loop`), `stop_before_cut`, `temporal_progress`, `temporal_equivalence`, `temporal_structure`, harnais `temporal_support`.

## 5. Fiche T6 (roadmap § 8, à relire en entier)

Fonction distincte de `run_loop` : `progress()` puis attente jusqu'à `wake` par une fermeture **injectée** ; rend le total sur arrêt, l'erreur sur échec. Fichiers : `src/driver.rs`, `tests/temporal_drivers.rs` créés, `src/lib.rs` (export). Tests D2 et moitié « pilote bloquant » de D1 ; aucun `sleep` ni horloge murale dans les tests ; aucune copie de la logique K/M ; `run_loop` intact. 1 commit.

## 6. Contraintes structurelles et points à instruire au cadrage de T6

**Contrainte D7 existante (à respecter)** : `tests/temporal_structure.rs` vérifie notamment que les utilisations de `Instant` restent cantonnées à `clock.rs` et `pacing.rs` (en plus de : `[dependencies]` vide, aucun `dyn`, `Box`, `unsafe`, flottant, type de fenêtrage, trait public). Le pilote T6 doit la tenir. Ne pas modifier ce test ni élargir l'usage de `Instant` sans arbitrage préalable.

Points non tranchés :

- Forme exacte du pilote (spéc. § 12.2) et de la fermeture d'attente par défaut (dormir `deadline − now`, jamais négatif).
- **PO-3** reste ouvert : à examiner si le choix d'implémentation de T6 nécessite une conversion `Deadline` → `Instant` (cas non représentable). Toute conversion utilise `checked_add`, jamais d'addition qui panique, et reste soumise à la contrainte D7 ci-dessus.
- Distinction `SCRIPT_MAX_STEPS_PER_TICK` / K (RK-8).

## 7. Règles de session (rappel)

- Une unité à la fois ; rapport selon le format du § 8 ; arrêt en attente de validation explicite.
- Fichiers `.rs` livrés complets, commentaire de chemin en première ligne ; documentation en français ; vouvoiement, concision.
- Séparer fichiers finaux et états intermédiaires ; ne jamais livrer un état intermédiaire sous le nom d'un fichier final.
- Un échec de compilation n'est pas un rouge ; sans rouge possible, sensibilité par mutation, une à la fois, sur copie isolée ; mutation non équivalente survivante = arrêt.
- Lints :
  - suppression locale et justifiée (`reason = "…"`), jamais globale ;
  - `#[expect(lint)]` lorsque le lint est effectivement attendu dans une configuration donnée ; un `expect` insatisfait (`unfulfilled_lint_expectations`) est un défaut à corriger, non à masquer ;
  - `#[allow(lint, reason = "…")]` ponctuel lorsque c'est le choix approprié, notamment dans les tests et leurs modules de support (précédent : `dead_code` sur `temporal_support` en T5c), avec justification ;
  - aucune neutralisation générale de `unfulfilled_lint_expectations` ;
  - ne pas remplacer un `expect` existant par un `allow` pour faire taire un avertissement.
- Distinguer systématiquement résultats de l'environnement réduit et résultats du dépôt réel ; aucun résultat déduit.
- Contraintes : dispatch statique, aucun trait public, `dyn`, `Box`, `unsafe`, flottant ; `[dependencies]` de `verbum-shell` vide ; MSRV 1.85 ; assertions des tests historiques, doctests et `Cargo.toml` non modifiés sans arbitrage.
- Contrôles utilisateur à chaque unité : `cargo build/test --workspace`, clippy `-D warnings`, fmt, `cargo tree -p verbum-shell -e normal`, Miri, MSRV 1.85.0.

## 8. Format du rapport de fin d'unité

1. **Fichiers** : finaux (chemin, SHA-256 abrégé) séparés des éventuels états intermédiaires.
2. **Changements** : par fichier, en une ou deux lignes.
3. **Tests** : ajoutés, modifiés, résultats ; rouge contre squelette quand possible.
4. **Mutations** : liste, une par une, tuée ou survivante ; survivante non équivalente = arrêt.
5. **Contrôles** : environnement réduit (exécuté par Claude) et dépôt réel (exécuté par l'utilisateur), toujours distingués ; aucun résultat déduit.
6. **Écarts** par rapport à la roadmap et à la spécification.
7. **Arbitrages** demandés ; **non démontré** ; regroupement de commit proposé.

## 9. Consigne d'amorce

> Vous reprenez le chantier ADR-S04 du Shell de Verbum. Lisez ce handoff, puis la roadmap (fiche T6), puis la spécification § 9.2 ; les documents historiques du § 3 ne sont pas requis. T4, T1, T2, T3, T5a, T5b, T5c sont validées. Cadrez T6 (plan, questions, risques) sans modifier aucun fichier, et attendez l'instruction explicite « démarrez l'unité T6 ».
