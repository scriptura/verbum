# Plan d'exécution par sessions — étape 1 (`verbum-schedule`)

## Statut
Document d'organisation, non normatif, révisable sans ADR. Complète `01-verbum-scheduler.md` (révision 5), qui reste la **référence** des invariants, de l'API et des tests. **Aucun code, aucune modification d'ADR, aucun commit avant validation explicite du plan.** Sources lues sur des copies jointes, pas sur le dépôt (ADR-000 : adopté, confirmé par l'utilisateur).

Les numéros de phase, de section (§) et de test renvoient à `01-verbum-scheduler.md`.

---

## 1. Règles communes à toutes les sessions

### 1.1 Discipline (reprise de `HANDOFF-Stage-1.md` et de l'étape 0)
Une phase à la fois, dans l'ordre. Le test précède le code. Un commit par invariant démontré. Quatre contrôles verts à chaque commit. Alarme « tant que j'y suis » : on s'arrête. Aucune modification d'ADR. Si une information manque : s'arrêter et demander. Si une difficulté est structurelle : l'exposer avant de contourner. Les ADR sont traitées comme une spécification : on n'en remet une en cause que si plusieurs implémentations indépendantes échouent pour la même raison structurelle.

### 1.2 Commandes de vérification (indicatives ; à adapter à votre commande habituelle)
Les quatre contrôles, à chaque commit, par la session :
```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```
Miri, **lancé par l'utilisateur** (la session fournit les commandes et signale les tests lourds) :
```
cargo +nightly miri test -p verbum-ecs
cargo +nightly miri test -p verbum-schedule
cargo +nightly miri test -p verbum-schedule --doc      # doctests exécutables seulement ; ne couvre pas les compile_fail
```
Le test exhaustif de la phase 3 (16³ combinaisons) est à réduire sous Miri via `cfg!(miri)`, avec la réduction documentée dans le test.

**Doctests : deux familles à distinguer.**
- Les doctests **`compile_fail`** démontrent que du code **ne compile pas** (ce sont eux qui prouvent les garanties de typage du plan §2.6 et §4.4). Ils sont vérifiés par `cargo test --doc` (inclus dans `cargo test --workspace`), qui contrôle que la compilation échoue. Miri n'exécute rien pour eux : **une commande Miri n'est pas une preuve de leur couverture**, et `cargo miri test --doc` ne doit pas être présentée comme telle.
- Les doctests **exécutables** (compilent et s'exécutent) sont vérifiés par `cargo test --doc` et peuvent en plus passer sous Miri.
Le rapport de sortie distingue les deux décomptes.

### 1.3 Fichiers interdits à toutes les sessions
- tout ADR et tout document de `docs/architecture/` (corpus figé) ;
- `ROADMAP.md`, `00-verbum-ecs.md`, `HANDOFF-Stage-0.md`, `HANDOFF-Stage-1.md`, `01-verbum-scheduler.md` : **non modifiés** par les sessions d'implémentation. Une session consigne ses constats dans son rapport de sortie (§1.5) ; l'utilisateur décide de les reporter dans le plan ;
- `crates/ecs/**` pour toute session autre que S2a et S2b ;
- `Cargo.toml` racine et `Cargo.lock` pour toute session autre que S1 (phase 0) et S5 (phase 10c).

### 1.4 Quand démarrer une nouvelle session
Une session **se termine** sur un état vérifié, jamais au milieu d'une phase. Démarrer une nouvelle session :
1. à la fin d'une session du tableau §2, une fois le rapport de sortie rédigé ;
2. à tout point de contrôle prévu (Miri en attente, revue B2) : la session s'arrête et attend l'utilisateur ;
3. si, au milieu d'une session, une phase n'est pas terminable proprement dans le contexte restant (estimation prudente : au-delà d'environ 70 % de 200 000 tokens consommés) : terminer le commit en cours, produire le rapport de sortie, s'arrêter ; la phase suivante démarre dans une session neuve ;
4. après un blocage structurel (deux implémentations indépendantes échouant pour la même raison) : rapport, arrêt, décision de l'utilisateur.

Une session ne commence jamais par reconstituer le raisonnement architectural : elle lit le **handoff d'entrée** (§1.6) et les sections citées du plan.

### 1.5 Rapport de sortie (modèle, rempli par chaque session)
```
SESSION : <id>             DATE : <date>
Commits réalisés : <hash> <message> (un par ligne)
État des tests : build / test / clippy / fmt : <vert|rouge> ; nombres de tests par crate ; doctests : <nombre compile_fail> / <nombre exécutables> ; Miri : <à lancer par l'utilisateur | vert>
Écarts au plan : <aucun | liste, avec justification>
Limites connues et points ouverts : <liste>
Décisions prises en cours de route (choix locaux) : <liste>
Fichiers modifiés : <liste> ; fichiers interdits touchés : <aucun>
Commandes Miri à lancer : <liste> ; tests lourds : <liste>
Prochaine session : <id> ; prérequis satisfaits : <oui|non, lesquels>
```

### 1.6 Socle de décisions (à inclure tel quel dans chaque handoff d'entrée)
- **Rôle** : implémenteur, pas architecte. Plan : `01-verbum-scheduler.md` r5, validé par l'utilisateur.
- **Contrat** : `Contract { reads, writes, produces, effects }`, `effects` = quatre indicateurs (création, destruction, ajout, retrait) ; jamais écrit à la main par un système ; dérivé du type d'accès. Dépendance : arête a→b ssi `(a.writes ∪ a.produces) ∩ b.reads ≠ ∅` ; `Write`/`Produce` n'ajoutent pas `reads` (D10) ; aucune dépendance déduite des commandes.
- **Erreurs de construction** : `MixedNature` (global), `MultipleWriters` (par phase), `UnknownPhase`, `Cycle`, contrôlées dans cet ordre (D4).
- **Identité** : `ID: u8` const-générique dans `Read`/`Write`/`Produce` ; `Locate<'w, ID>` par monde (E0119 doublon, E0277 absent) ; `Binding` ; `Bind` scellé ; limite : sens métier de l'étiquetage non déductible, couvert par des tests de câblage.
- **Dispatch** : liste récursive `Nil`/`Cons<At<PHASE, S>, Tail>`, aucun plafond d'arité, ni `dyn` ni pointeur de fonction ; `ExecutionModel::build` ne reçoit que des contrats.
- **Atomicité** : seul le lot de commandes de `World::apply` est atomique. Pas de rollback des valeurs, des phases validées, du tick. Tick arrêté après toute erreur.
- **Erreurs de tick** : `TickError::{BufferNotEmpty, Binding, Buffer(BufferFull), Apply(ApplyError)}` ; garde d'entrée (buffer non vide → refus avant tout système) ; buffer vidé explicitement par l'appelant après une erreur (plan §6.2).
- **Accumulateur** : Ressource ; système explicite de remise à zéro, pas de reset du Scheduler ; test sur deux ticks (12, témoin négatif 18).
- **`verbum-ecs`** : deux correctifs seulement, `split_mut` (6, autorisé) et `RemoveComponent` (6bis, autorisé ; composant absent = sans effet, choix local) ; commits distincts ; Miri par l'utilisateur.
- **Hors périmètre** : `PhaseKind`, parallélisme, génération de code pour la liaison, diagnostic détaillé des cycles, construction `const` du modèle, Shell, horloge, vivacité d'entité côté système.
- **Limites assumées** : coût O(N²) du dispatch non mesuré ; `recursion_limit` ; effets externes cachés non empêchés par les types ; composition d'un `Produce` non vérifiable par le type ; Rust 1.85 non vérifié ; prototype de faisabilité hors dépôt, non livré.
- **Attributions de commit** : fin de message : `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>` puis `Claude-Session: <URL de la session>`.

---

## 2. Vue d'ensemble

| Session | Phases | Pourquoi cette frontière | Prérequis exacts | Durée de contexte visée |
|---|---|---|---|---|
| **S1** | 0 à 5 | Piste purement Scheduler, sans `verbum-ecs`, tests de logique pure ; se termine sur un modèle immuable vérifié | plan validé | moyenne |
| **S2a** | 6 | Petit commit isolé sur `verbum-ecs` ; Miri à passer avant d'aller plus loin | S1 mergée ; A2 (donné) | courte |
| **S2b** | 6bis | Touche une API publique de l'étape 0 (trait `ComponentSet`) : revue B2 de l'utilisateur avant d'écrire, Miri après | S1 et S2a mergées, Miri vert ; B2/B3 revus | moyenne |
| **S3** | 7a, 7b | Cœur du typage (GAT, `Locate`, `Commands`) ; exige un `verbum-ecs` stable ; point d'arrêt interne après 7a | S1, S2a, S2b mergées | longue |
| **S4** | 8, 9 | Assemblage (liste, `Scheduler`) puis exécution et contrat d'erreur ; dépend de tous les types | S3 mergée | longue |
| **S5** | 10a à 10c | Acceptance, exemple, façade ; touche le workspace racine en dernier | S4 mergée | courte |

**Ordre recommandé : strictement séquentiel, S1 → S2a → S2b → S3 → S4 → S5.** S1 précède les modifications de `verbum-ecs` parce que son handoff prérempli attend encore les décomptes de l'étape 0 (64 unitaires, 11 d'intégration, 5 doctests) : exécuter S2a/S2b avant S1 changerait ces décomptes. Le parallélisme S1 ∥ (S2a → S2b) reste **théoriquement possible** sur des branches séparées (ensembles de fichiers disjoints : S1 : `Cargo.toml` racine, `Cargo.lock`, `crates/schedule/**` ; S2 : `crates/ecs/**`), à condition d'adapter le handoff de S1 aux décomptes modifiés ; **il n'est pas recommandé pour ce démarrage**.

---

## 3. Sessions

### S1 — Phases 0 à 5 : modèle d'exécution
**Fichiers à traiter** : `Cargo.toml` (racine, `members`), `crates/schedule/Cargo.toml`, `src/lib.rs`, `src/data.rs`, `src/contract.rs`, `src/order.rs`, `src/model.rs`, tests unitaires et doctests de ces modules. Le crate suit les frontières du plan §7 : aucune occurrence de `verbum_ecs`, `System`, `Read`, `Write`, `At` dans `data`, `contract`, `order`, `model`.
**Fichiers interdits** : `crates/ecs/**`, `crates/verbum/**`, tout ce qui figure en §1.3.
**Prérequis** : plan validé ; workspace avec `verbum-ecs` et `verbum` à l'état d'origine (étape 0 close : 64 tests unitaires, 11 d'intégration, 5 doctests).
**Livrable** : `verbum-schedule` : `DataId`/`DataSet`, `Contract` (quatre indicateurs d'effets), dépendance déduite, tri déterministe, cycle = erreur de construction, `MultipleWriters`, `MixedNature` (global), `ExecutionModel` immuable construit des seuls contrats.
**Commits attendus (un par invariant)** :
1. `workspace: crate verbum-schedule vide` (phase 0)
2. `contract: interface déclarative inspectable, sans fonction de système` (1)
3. `order: dépendance déduite production → consommation` (2)
4. `order: tri déterministe d'une phase, cycle rejeté à la construction` (3)
5. `order: écrivain unique par phase, nature de catégorie cohérente` (4)
6. `model: programme d'exécution immuable, construit depuis les seuls contrats` (5)

**Vérifications** : les quatre contrôles à chaque commit. **À transmettre à l'utilisateur pour Miri** : `cargo +nightly miri test -p verbum-schedule` ; signaler la réduction `cfg!(miri)` du test exhaustif de la phase 3 et sa durée.
**Attention** : `verbum-schedule` est `#![no_std]`, sans `alloc`, `unsafe_code = "deny"` hérité du workspace (`[lints] workspace = true`) ; dépendance unique `verbum-ecs` déclarée dès la phase 0 mais **non utilisée** avant 7a (si cela déclenche un avertissement, la dépendance n'est ajoutée qu'en 7a ; consigner le choix).
**Arrêt** : fin de la phase 5, état vert. Rapport §1.5.

### S2a — Phase 6 : `World::split_mut`
**Fichiers à traiter** : `crates/ecs/src/world.rs` (méthode, tests), éventuellement un fichier de test d'intégration dédié dans `crates/ecs/tests/`.
**Fichiers interdits** : tout le reste, notamment `component_set.rs`, `command.rs`, `storage.rs`, `values.rs`, `crates/schedule/**`, `Cargo.toml` racine.
**Prérequis** : S1 mergée (état vert, rapport de sortie reçu) ; A2 (donné) ; `verbum-ecs` à l'état de la clôture de l'étape 0 (S1 ne le modifie pas).
**Livrable** : `pub fn split_mut(&mut self) -> (S::Values<'_>, &mut R)`, sans `unsafe`, avec les tests du plan §4.4 (accès conjoint, structure inchangée, `compile_fail` E0499, `compile_fail` sur l'affectation à `&mut S`, équivalence avec les deux accessoires séparés). Signature exacte validée à l'implémentation (point B1) : si elle doit changer, la session s'arrête et expose pourquoi.
**Commit attendu** : `world: accès disjoint aux valeurs de composants et aux Ressources`.
**Vérifications** : quatre contrôles ; non-régression : tous les tests de l'étape 0 inchangés (aucun test existant modifié). **Miri** : `cargo +nightly miri test -p verbum-ecs`.
**Arrêt** : **point de contrôle obligatoire**, attente du Miri de l'utilisateur et de la fusion avant S2b. Rapport §1.5 avec les nouveaux décomptes de tests.

### S2b — Phase 6bis : `RemoveComponent`
**Fichiers à traiter** : `crates/ecs/src/component_set.rs`, `world.rs`, `command.rs`, `fixtures.rs` (implémenteur de test), tests et doctests qui implémentent `ComponentSet` ; nouveaux tests du plan §5.1.3.
**Fichiers interdits** : `storage.rs` (sauf lecture : `SparseSet::remove` existe et n'est pas modifié), `values.rs`, `query.rs`, `resource.rs`, `entity.rs`, `crates/schedule/**`, `Cargo.toml` racine.
**Prérequis** : S1 et S2a mergées, Miri vert ; l'utilisateur a pris connaissance du point **B2** (le trait `ComponentSet` n'est pas purement additif) et de **B3** (composant absent = sans effet). Si l'utilisateur modifie B2/B3 avant la session, le handoff d'entrée le consigne.
**Livrable** : surface du plan §5.1.1 ; règles de prévalidation §5.1.2 (simulation séquentielle, pas de validation rétroactive, rejet atomique) ; tests §5.1.3 écrits avant le code. Aucune nouvelle variante de `ApplyError`.
**Commit attendu** : `world: RemoveComponent — retrait de composant différé par le command buffer`. Les adaptations mécaniques des implémenteurs de `ComponentSet` font partie de ce commit ; elles sont listées dans le rapport.
**Vérifications** : quatre contrôles ; les 64 tests unitaires, 11 d'intégration et 5 doctests de l'étape 0 passent ; décomptes actualisés. **Miri** : `cargo +nightly miri test -p verbum-ecs` (signaler que `plan` simule des lots : tests de lots volumineux à réduire sous Miri si nécessaire).
**Arrêt** : fin du commit, Miri en attente. Rapport §1.5 incluant la liste exacte des implémenteurs de `ComponentSet` adaptés.

### S3 — Phases 7a, 7b : accès typé et commandes
**Fichiers à traiter** : `crates/schedule/src/access.rs` (`Locate`, `Bind` scellé, `Read`/`Write`/`Produce`, `Part` scellé, `Commands`), éventuellement `system.rs` pour `Binding` ; `crates/schedule/tests/access.rs`, `tests/support/`, doctests `compile_fail` du plan §2.6 ; `crates/schedule/Cargo.toml` (dépendance `verbum-ecs` si non encore ajoutée).
**Fichiers interdits** : `crates/ecs/**` (tout besoin découvert est un blocage à signaler, pas une modification), `data`/`contract`/`order`/`model` hors ajout strictement nécessaire à l'accès (consigner), `crates/verbum/**`.
**Prérequis** : S1, S2a, S2b mergées ; `split_mut` et `RemoveComponent` disponibles ; états verts. Le jeu de composants et de Ressources de `tests/support` implémente le `ComponentSet` **de l'état post-6bis**.
**Livrable** : 7a (vues, liaison dérivée, contrat calculé en `const`, test de câblage de deux Ressources de même type avec témoin croisé, `compile_fail` §2.6 hors effets) ; 7b (`Commands<.., SPAWN, DESPAWN, ADD, REMOVE>`, effets du contrat = indicateurs du type, `compile_fail` des effets non déclarés, débordement → `BufferFull`).
**Commits attendus** : `access: vues déclaratives, identité par catégorie, contrat calculé sur le type` (7a) ; `access: poignée de commandes typée par effets déclarés` (7b).
**Vérifications** : quatre contrôles ; doctests `compile_fail` ; **Miri** : `cargo +nightly miri test -p verbum-schedule`.
**Point d'arrêt interne** : après le commit 7a, état vert, rapport intermédiaire ; poursuivre en 7b dans la même session si le contexte le permet (§1.4, règle 3), sinon nouvelle session S3b avec le rapport intermédiaire comme handoff.
**Si la liaison ne compile pas telle que prévue** : le prototype de faisabilité (plan §10) n'est pas dans le dépôt ; se rapporter au plan §2 et au tableau §2.3 ; exposer toute divergence (rustc 1.85, vrai `World`) avant de contourner.

### S4 — Phases 8, 9 : Scheduler et exécution d'un tick
**Fichiers à traiter** : `crates/schedule/src/system.rs` (`System`, `At`, `Nil`/`Cons`, `contract_of`), `scheduler.rs` (`Scheduler`, `run_tick`, `TickError`), `tests/scheduler.rs`, `tests/tick.rs`.
**Fichiers interdits** : `crates/ecs/**`, `crates/verbum/**`, `Cargo.toml` racine ; `data`/`contract`/`order`/`model` (sauf correctif d'un défaut démontré, consigné).
**Prérequis** : S3 mergée (7a et 7b verts).
**Livrable** : phase 8 (liste récursive sans plafond, `contract_of`, `Scheduler::new`, **test d'exécution des 13 systèmes** par le dispatch récursif, journal d'exécution) ; phase 9 (tests 1 à 10 du plan : lecture intra-phase, interchangeabilité, immutabilité, déterminisme, différé structurel 5a/5b, échec d'`apply` en trois phases, `BufferFull`, défaut de liaison, garde d'entrée `BufferNotEmpty`, buffer vide après succès).
**Commits attendus** : `scheduler: liste de systèmes typée, modèle construit des seuls contrats` (8) ; `scheduler: exécution d'un tick, lecture intra-phase` (9, commit A) ; `scheduler: validation en fin de phase, effets structurels différés` (9, commit B). Le variant `TickError::BufferNotEmpty` et sa garde entrent dans le commit B, avec leurs tests.
**Vérifications** : quatre contrôles ; **Miri** : `cargo +nightly miri test -p verbum-schedule` ; signaler le test de 13 systèmes (compilation longue, `recursion_limit`).
**Point d'arrêt interne** : après le commit de la phase 8 (rapport intermédiaire).
**Attention** : aucune promesse de rollback dans la documentation ni les messages d'erreur ; contrat d'erreur : plan §6.2.

### S5 — Phases 10a à 10c : acceptance, exemple, façade
**Fichiers à traiter** : `crates/schedule/tests/acceptance.rs`, `crates/schedule/examples/headless.rs` ; en 10c uniquement : `Cargo.toml` racine (`[workspace.dependencies]`), `crates/verbum/Cargo.toml`, `crates/verbum/src/lib.rs`.
**Fichiers interdits** : `crates/ecs/**` ; `src/` de `verbum-schedule` sauf défaut démontré par l'acceptance (consigner, un commit séparé).
**Prérequis** : S4 mergée.
**Livrable** : acceptance (pipeline `ClearAccumulator` → `ContributeA/B/C` + `ResolveMotion` ; tests 1 et 2 de la Roadmap ; **interchangeabilité des producteurs** par les six permutations ; deux ticks : 6 puis 12, témoin négatif 18) ; exemple headless (plusieurs ticks, aucune sortie, code retour 0) ; réexport `pub use verbum_schedule as schedule;` dans la façade.
**Commits attendus** : `scheduler: acceptance test, étape 1` ; `schedule: exemple headless, plusieurs ticks sans sortie` ; `verbum: réexporte verbum-schedule`.
**Vérifications** : quatre contrôles ; `cargo run -p verbum-schedule --example headless` (code retour 0) ; **Miri** : `cargo +nightly miri test -p verbum-schedule` (l'exemple n'est pas exécuté sous Miri).
**Arrêt** : fin de l'étape 1. Rapport final : décomptes de tests, écarts, limites ; reporter dans le journal de construction à la discrétion de l'utilisateur.

---

## 4. Handoff d'entrée autonome (gabarit)
À fournir au début de chaque session ; le texte est autosuffisant. Les champs « à renseigner » sont remplis à partir du rapport de sortie de la session précédente.

```
HANDOFF D'ENTRÉE — session <id>
Mission : <phases, d'après §3>. Rôle : implémenteur. Plan : 01-verbum-scheduler.md r5 (validé). Sessions : 01-verbum-scheduler-sessions.md.
Documents à lire : HANDOFF-Stage-1.md ; 01-verbum-scheduler.md : <sections citées pour ces phases> ; ADR-000 à ADR-006, dont **ADR-001** (stockage, isolation, capacité) et **ADR-002** (Query, modes d'accès), en plus des références du plan §1 ; sources de verbum-ecs : <fichiers utiles>.
Socle de décisions : <§1.6, recopié>
État attendu à l'entrée : <commits déjà fusionnés> ; tests : <décomptes par crate> ; Miri : <vert pour quels commits>.
  → À renseigner depuis le rapport de la session précédente : <hash, décomptes, écarts, choix locaux>.
Fichiers à traiter : <§3>. Fichiers interdits : <§3 + §1.3>.
Livrable et commits attendus : <§3>.
Vérifications : <§1.2> ; Miri à lancer par l'utilisateur : <§3>.
Limites connues et points ouverts : <§1.6 + rapport précédent>.
Règle d'arrêt : <§1.4>.
```

### Handoff d'entrée pré-rempli — S1
```
Mission : phases 0 à 5. État attendu : workspace {verbum-ecs, verbum} ; étape 0 close (64 unitaires, 11 intégration, 5 doctests ; Miri vert côté utilisateur). Aucun commit de l'étape 1.
Documents à lire : ADR-000 à ADR-006 (dont ADR-001 et ADR-002) ; plan §1 (I1 à I9, I11, I12, I15), §7 (D1 à D5, frontières), §8 phases 0 à 5, §6 (Q3, Q5), §9.
Points ouverts : aucun bloquant. Choix local à consigner : moment d'ajout de la dépendance `verbum-ecs` (phase 0 ou 7a).
```
Pour S2a : état attendu = S1 mergée, `verbum-ecs` encore à l'état de l'étape 0 (mêmes décomptes) ; points ouverts : B1 (signature). Pour S2b : état attendu = S2a mergée, Miri vert ; points ouverts : B2, B3. Pour S3 à S5 : état attendu = session précédente mergée et verte ; points ouverts = ceux du rapport.

---

## 5. Contrôle de cohérence de ce plan de sessions
- Chaque phase du plan (0 à 5, 6, 6bis, 7a, 7b, 8, 9, 10a, 10b, 10c) est attribuée à exactement une session.
- Ordre séquentiel recommandé S1 → S2a → S2b → S3 → S4 → S5. Les fichiers de S1 et de S2a/S2b sont disjoints ; aucune session ne modifie les mêmes fichiers qu'une session simultanée ; `Cargo.toml` racine et `Cargo.lock` ne sont touchés que par S1 (phase 0) et S5 (phase 10c), jamais en parallèle.
- S3 dépend de S2b (les implémenteurs de `ComponentSet` des tests du Scheduler ne sont pas écrits deux fois).
- Chaque session se termine sur un état vérifié ; les points de contrôle utilisateur (Miri après S2a, revue B2 avant S2b, Miri après S2b) sont explicites.
