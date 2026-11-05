# Rapport de fin d'unité U3 (validé) — avec annexe des résultats réels

Rapport tel que remis à la fin d'U3 (résultats de l'« espace de travail » de la session), suivi d'une annexe qui reproduit les résultats **réels** communiqués par l'utilisateur lors de la validation (5 novembre 2026). Document destiné à la session d'U4 ; il ne modifie aucune exigence.

## 1. Unité et état
U3 (crate `verbum-script`) livrée, **validée par l'utilisateur le 5 novembre 2026**. Aucun commit n'a été fait par la session.

## 2. Fichiers
- Créés : `crates/extensions/script/Cargo.toml` et `src/{lib,state,emissions,table,step}.rs`.
- Modifié : `Cargo.toml` racine, livré complet ; deux lignes ajoutées : le membre `"crates/extensions/script"` après `crates/engine/render`, et `verbum-script = { path = "crates/extensions/script", version = "0.1.0" }` dans `[workspace.dependencies]`.
- Aucun autre fichier touché.

## 3. Faits revérifiés (F1, F2, F4, F6, AMB-6)
Confirmés sans divergence : `Cargo.toml` racine à 7 membres ; `Script<'a>` `Copy` avec `read`, `len`, `is_empty`, `instruction(u16) -> Option<Instruction>`, constantes 4 et 16 de type `usize` ; `SlotIndex::new(u32)` et `get() -> u32` `const` ; `accumulator.rs` et `entity.rs` conformes à ce que le document d'exécution en dit.

## 4. Rouge
Squelettes `unimplemented!()` : 19 tests sur 19 en échec, tous par panique `not implemented` (aucun échec de compilation compté). Tests : ST-1 à ST-4, IN-1 à IN-5, IN-budget, IN-bound, IN-end, IN-miss, IN-full, IN-det, EM-1 à EM-3, TB-1. Les assertions de ST-1 sur les émissions par tick et celles de ST-4 sur `start` figurent dans les tests listés. Aucune assertion n'a été ajoutée après le rouge.

## 5. Contrôles
- **Espace de travail** (chaîne 1.97 stable, `verbum-assets` partiel : vrais `script.rs` et `manifest.rs`, stubs `header.rs`, `sprite.rs` réduit à `ReadError`, `lib.rs`) : `cargo build` sans avertissement ; `cargo test -p verbum-script` : 19 réussis, 0 échoué ; `cargo clippy -p verbum-script --all-targets -- -D warnings` propre ; `cargo fmt --check` propre ; `cargo doc` avec `-D warnings` propre.
- **`cargo tree`** (espace de travail) : `-p verbum-script -e normal` et `-p verbum-script` ne listent que `verbum-assets` ; `-p verbum-assets` sans dépendance ; `-i verbum-script --workspace` sans dépendant.
- Écarts de l'espace réduit : 2 membres sur 7, stubs d'assets, aucun `Cargo.lock`.
- **Dépôt réel** : voir l'annexe.

## 6. Mutations
29 lignes traitées sur copies isolées hors de l'arbre de travail, une à la fois, `cargo test` en profil `dev` ; toutes détectées sauf M-3.28 (équivalente). Mode de détection observé.

| Mutation | Détecteur et autres échecs | Mode |
|---|---|---|
| 3.1 | ST-1, ST-3, ST-4 | assertion |
| 3.2, 3.3, 3.5 | ST-2 seul | assertion |
| 3.4 a, b, c (une par statut) | ST-2 seul | assertion |
| 3.6 | ST-4 seul | assertion |
| 3.7 | IN-4, IN-bound, IN-miss, ST-1 | assertion |
| 3.8 | IN-4, IN-miss, ST-1 | assertion |
| 3.9 | IN-4, IN-bound, IN-miss, ST-1, ST-4 | assertion |
| 3.10 | IN-4, IN-bound, IN-1, ST-1 | assertion |
| 3.11 | IN-1, IN-full, IN-det, ST-1, ST-4 | panique `sink assez grand: Full` (IN-full : assertion) |
| 3.12 | IN-5, IN-3, IN-4, ST-1 | assertion |
| 3.13 | IN-5 (état `Halted` construit) | assertion |
| 3.13b | IN-5 (états `Faulted` construits) | assertion |
| 3.14 | IN-3, ST-1 | assertion |
| 3.15 | IN-3 seul | assertion |
| 3.16 | IN-budget, IN-5 | assertion |
| 3.17 | IN-bound, IN-end, IN-budget | assertion |
| 3.18 | IN-end seul | assertion |
| 3.19 | IN-budget, IN-5 | assertion |
| 3.20 | IN-2 | panique `attempt to add with overflow` |
| 3.21 | IN-miss, IN-5, ST-4 | panique `script absent` |
| 3.22 | IN-full seul | assertion |
| 3.23 | EM-2 | assertion |
| 3.24 | EM-1, IN-full | assertion |
| 3.25 | TB-1, IN-miss | panique `index out of bounds` (6 sur 6) |
| 3.26 | TB-1 | assertion |
| 3.27 | TB-1, IN-miss | assertion |
| 3.28 | aucun test en échec | équivalente |

Écarts entre détecteurs annoncés et échecs observés, expliqués par le code :
- **Ligne ST-4 de M-3.7, 3.8, 3.10, 3.14 et 3.15.** ST-4 vérifie la validité des états atteints, pas leurs valeurs ; ces mutations produisent des états valides. ST-1 les détecte, sauf M-3.15 (`Jump` absent de la trajectoire de référence : seul IN-3 la voit).
- **M-3.12 et AMB-2.** ST-1 échoue aussi, au tick 10 (`pc` 7 au lieu de 6), car ses états viennent de l'exécution réelle de `step` ; IN-5 reste le détecteur principal, avec IN-3 et IN-4.
- **M-3.11.** Le premier échec d'IN-1 est la panique du `expect` du helper (sink de capacité 8 rempli sans avancée de `pc`), avant l'assertion `pc == 4` ; la sensibilité propre de cette assertion n'a pas été isolée, IN-full porte `pc == 1` après `Err(Full)`.
- **M-3.17, 3.16, 3.19.** IN-budget et IN-5 échouent en plus (`pc` et `r0` à la faute).
- **M-3.24.** IN-full échoue en plus (l'écrasement rend `Ok` à la place de `Err(Full)`) ; EM-2 ne détecte rien, comme annoncé.
- **M-3.28, équivalente.** Le test est passé après mutation (observé). `countdown` n'est appelée que sur un état `Waiting` ; `Waiting ⇒ wait ≥ 1` est garanti par `from_bytes` et par `wait_for` (ticks ≥ 1, le lecteur rejette `Wait 0`) ; le `wait - 1` ne peut pas déborder. Aucun test artificiel.

## 7. Portes
G-CM et G-LT sans objet (U3 ne touche ni `load`, ni `verbum-assets`, ni un montage `World`/`Scheduler`) ; G-ORD-S sans objet ; G-ST : revue ci-dessous ; R-ADAPT : aucune adaptation hors des fichiers listés.

## 8. Garanties (revue, sans preuve statique)
- **Revue G-ST.** Recherche de `static`, `thread_local`, `Cell`, `RefCell`, `Atomic`, `async`, `await`, `thread`, `Box`, `Vec`, `unsafe`, `dyn`, `Rc`, `Arc`, `alloc`, `transmute`, `include_bytes` : aucune occurrence en code de production ; deux occurrences dans les tests (lifetime `'static`, borne de ST-3 et garde de TB-1 ; ce ne sont pas des `static` items) ; une occurrence en commentaire (« sans `Vec` »). `step` n'a que `(script, state, sink)` comme paramètres ; `ScriptState` n'a aucune référence ; ni `unsafe`, ni `dyn`, ni trait public.
- **Signature publique.** (a) `registers()` rend `[i32; SCRIPT_REGISTERS]` par copie ; (b) `script()`, `pc()`, `status()`, `wait()` rendent des valeurs `Copy` ; (c) aucun accesseur ne rend de référence ni de `&mut` (champs privés, transitions `pub(crate)`) ; (d) `get(u16)` rend `Option<&Script<'a>>` ; (e) la liste des signatures publiques est celle du § 3.3 du document d'exécution (écarts : point 10).
- **Raisonnement AMB-5 (écrit, aucun test).** Une instruction n'est exécutée que si `instruction(pc)` est `Some`, donc `pc ≤ n − 1 ≤ 65 534` et `pc + 1 ≤ 65 535` tient dans un `u16`. La fin de programme implicite ne fait pas d'incrément : `pc` reste à `n`. Un `pc = 65 535` venu de `from_bytes` donne `instruction(65 535) = None` (puisque `n ≤ 65 535`), donc `Halted` sans incrément. `advance` utilise `+= 1`, protégé par ce raisonnement.

## 9. Natif, Miri, MSRV
Natif : voir point 5. Miri : non exécuté dans l'espace de session (pas de chaîne `nightly`) ; commande : `cargo +nightly miri test -p verbum-script` (résultat réel en annexe). MSRV 1.85 : non exécuté (U5) ; point non vérifié : `[None; CAP]`, `checked_mul` en `const fn` et garde statique sous 1.85 (la chaîne de session est 1.97 et compile) ; let-chains évitées, aucune constante utilisée seulement dans un `const _`.

## 10. Ajustements et réponses aux AMB
- **Noms et formes provisoires [L].** `StateError` et `SlotTooLarge` sont des structures unitaires ; `TableError` : `OutOfRange`, `Occupied` ; `ScriptTable` ne dérive ni `Clone` ni `Debug` ; accesseurs de `ScriptState` en `const fn` ; transitions internes en `pub(crate)` (`register`, `set_register`, `advance`, `jump_to`, `halt`, `fault`, `wait_for`, `countdown`) ; constantes de codes privées ; aucune API publique au-delà du § 3.3.
- **AMB-2** : ST-1 obtient ses états par `step` réel ; M-3.12 fait échouer ST-1 et IN-5. **AMB-3** : `same_capacity` fonction locale de test (EM-3). **AMB-4** : `#[allow(clippy::new_without_default)]` localisé et motivé, aucun `Default` public (traitement de `Contributions`). **AMB-5** : raisonnement au point 8. **AMB-6** : aucun écart. **AMB-7** : « révision du 3 novembre 2026 ». **AMB-8** : fichiers supplémentaires acceptés.
- Les trois documents (plan, document d'exécution, handoff d'U3) ont été corrigés pour `crates/extensions/script` ; le handoff d'U1 garde `crates/script` à sa ligne 188 (mention historique, non modifiée).

## 11. Commit et `tree.md`
Regroupement proposé : (1) manifestes (`Cargo.toml` racine, `Cargo.toml` du crate, `lib.rs`) ; (2) `state.rs` (ST-2, ST-3) ; (3) `emissions.rs` (EM-1 à EM-3) ; (4) `table.rs` (TB-1) ; (5) `step.rs` (ST-1, ST-4, IN-*). `tree.md` à régénérer : `crates/extensions/script/Cargo.toml` et `src/{lib,state,emissions,table,step}.rs`.

## 12.
En attente de la validation de l'unité U3. — **Validée le 5 novembre 2026.**

---

## Annexe — Résultats réels communiqués par l'utilisateur (dépôt réel, 5 novembre 2026)

Reproduction de la confirmation de validation de l'utilisateur ; ces résultats ont été produits par l'utilisateur, non par la session.

- Contrôles Cargo : conformes.
- Dépendances (`cargo tree`) : conformes.
- `cargo +nightly miri test -p verbum-script` : **19 tests réussis, aucun échec, en 3,92 secondes**.
- Décision : « Je confirme la validation de l'unité U3 » ; aucun travail supplémentaire sur U3 n'est demandé.

Les résultats « espace de travail » du corps du rapport et les résultats « dépôt réel » de cette annexe restent distincts ; aucun n'est déduit de l'autre. MSRV 1.85 n'a pas été exécuté (prévu en U5).
