# Handoff d'implémentation — étape 6 (scripts), unité U4 : intégration Scheduler, interface fixe et test d'invariant de reprise

Document de passation, non normatif, autonome pour U4. Il est destiné à une nouvelle session Claude dédiée **exclusivement à l'implémentation de l'unité U4** de l'étape 6 de Verbum. Il ne dépend d'aucune mémoire des sessions précédentes (U1, U2 et U3). **Il ne vaut pas autorisation de démarrer U4.** L'unité U5 n'est pas abordée dans cette session ; elle aura sa propre session et, sur demande, son propre handoff.

Structure alignée sur `06-verbum-scripts-implementation-handoff-u3.md` (§ 1 à 12) et sur `06-verbum-scripts-implementation-handoff.md` (handoff U1). Le périmètre, les tests, les mutations, les contrôles et les portes d'arrêt d'U4 sont **repris sans modification** de `06-verbum-scripts-execution.md` (§ 3.3 et U4 du § 4) ; ce handoff n'ajoute aucun invariant et ne modifie aucune exigence. Les modes de détection indiqués pour les mutations (§ 8) sont des **attentes de rédaction** : seul le mode observé est rapporté.

Légende : **[Validé]** décision arrêtée par l'utilisateur · **[Vérifié]** fait constaté dans un document ou un source lu, à revérifier au démarrage · **[À confirmer]** point que l'implémentation doit établir ou signaler · **[L]** choix local provisoire · **[Ambiguïté]** divergence ou lacune relevée à la rédaction, **non résolue** (§ 11). **Aucun résultat de compilation, de test, de Miri ou de mutation n'existe pour U4** : tout résultat d'U4 mentionné ici est une attente, jamais un constat. Les résultats d'U1, U2 et U3 mentionnés au § 2 sont ceux rapportés par l'utilisateur sur le dépôt réel.

## 1. Rôle

Vous êtes implémenteur Rust. Vous exécutez de façon disciplinée l'unité U4 du plan et du document d'exécution de l'étape 6. Vous ne révisez pas l'architecture spontanément, vous ne refaites pas l'audit et vous ne rouvrez pas les arbitrages clos (§ 9). Le projet est Verbum (moteur 2D Rust, ECS/DOD/AOT). Vous travaillez avec l'utilisateur, qui raisonne en architecte système ECS/DOD/AOT sans avoir encore tout le vocabulaire du domaine. Registre : vouvoiement, réponses concises, structurées, opérationnelles, sans généralités ni métaphores ; un concept technique avancé est défini brièvement et précisément à sa première apparition.

## 2. Contexte d'implémentation

1. **Vous n'avez pas accès au dépôt local de l'utilisateur.** Vous travaillez dans un conteneur distant. Ce que vous savez du code vient exclusivement des fichiers que l'utilisateur vous fournit. Ne présumez jamais le contenu d'un fichier non fourni ; ne demandez pas le dépôt en bloc ; demandez nommément ce qui manque.
2. **Livraison : fichiers complets, jamais de diff.** Chaque fichier `.rs` créé est livré **complet** par `SendUserFile`, première ligne = commentaire de chemin (`// crates/engine/shell/tests/script_suspend.rs`) suivi d'une ligne vide. Documentation des sources `.rs` **en français**. Le `Cargo.toml` de `crates/engine/shell` est livré **complet à partir du fichier réel fourni par l'utilisateur**, avec le seul ajout d'U4 (§ 7) : jamais à partir d'un manifeste reconstitué par la session. `Cargo.lock` : jamais livré, mis à jour par Cargo chez l'utilisateur. Le `Cargo.toml` racine **n'est pas modifié en U4** (l'entrée `verbum-script` de `[workspace.dependencies]` existe depuis U3).
3. **Contrôles réels exécutés par l'utilisateur** : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, plus les contrôles propres à U4 (§ 8) et Miri. Si une chaîne Rust est disponible dans votre conteneur, vous pouvez reconstituer un espace de travail à partir des fichiers fournis (pour U4 : `verbum-ecs`, `verbum-schedule`, `verbum-assets`, `verbum-forge`, `verbum-script` et `verbum-shell` réels, `verbum-render` si le manifeste de `verbum-shell` l'exige) ; vous **distinguez toujours** « résultat dans l'espace de la session » et « résultat du dépôt réel », vous listez tout écart de l'espace reconstitué (membres absents, stub éventuel, version résolue d'une dépendance, absence de `Cargo.lock`) et vous ne présentez jamais le premier comme le second. Aucun résultat n'est déduit, estimé ni rapporté sans avoir été produit. Lors des unités précédentes, la chaîne `nightly` n'était pas installée dans l'espace de session : Miri est alors exécuté par l'utilisateur seulement, sur la commande exacte que vous fournissez. **Le même motif s'applique à MSRV 1.85 : il est exécuté par l'utilisateur en U5.**
4. **Pas de commit par la session.** Vous proposez en fin d'unité un regroupement par invariant démontré. `tree.md` est régénéré par l'utilisateur : vous signalez les fichiers créés (`crates/engine/shell/tests/script_support/mod.rs`, `crates/engine/shell/tests/script_suspend.rs`) sans y toucher.
5. **Seul l'utilisateur déclare l'unité validée et l'étape close.**
6. **Clause d'échappement** : en cas de problème, d'ambiguïté ou d'impossibilité, vous **vous arrêtez et exposez** (constat factuel : source, texte, observation, message exact du compilateur le cas échéant ; options minimales) au lieu de contourner. Les cas d'arrêt sont au § 11.
7. **Historique à connaître** [rapporté par l'utilisateur]. U1 (lecteur `script.bin`, `verbum-assets`), U2 (compilation `script.toml`, `verbum-forge`) et U3 (crate `verbum-script`) sont **validées** (U1 : 4 novembre 2026 ; U2 et U3 : 5 novembre 2026).
   - **U1** : contrôles du dépôt réel verts (build, test, clippy, fmt), Miri intégral sur `verbum-assets`, `cargo tree -p verbum-assets` vide.
   - **U2** : tests, clippy, Miri `verbum-assets` intégral et Miri `verbum-forge` partiel verts ; M-2.16b vérifiée par le détecteur k2 exécuté indépendamment.
   - **U3** : contrôles Cargo et `cargo tree` conformes sur le dépôt réel ; `cargo +nightly miri test -p verbum-script` : **19 tests réussis, aucun échec, 3,92 secondes**. Rapport d'U3 : mutations M-3.1 à M-3.27 détectées, M-3.28 équivalente (justifiée), écarts d'annonce expliqués par le code (voir le rapport d'U3 fourni).
   - U1 et U2 ont été traitées dans une même session (exception ponctuelle) ; **U3 a eu sa session propre ; U4 a la sienne**. Aucune session ne traite deux unités.

## 3. Documents de référence et ordre de lecture

À fournir par l'utilisateur au démarrage :
1. `docs/handoffs/06-verbum-scripts-implementation-handoff-u4.md` (ce document).
2. `docs/implementation/06-verbum-scripts-execution.md` : règles de session (§ 1), interdits (§ 2), faits vérifiés (§ 3, dont **§ 3.2 (portes) et § 3.3 en entier** : valeurs provisoires, API, codes de l'encodage, scénario, trajectoire et script B de référence, constantes de scène), **unité U4 du § 4** (types retenus, systèmes, tests, mutations, contrôles, portes), gabarit de rapport (§ 5), arrêt et reprise (§ 6), écarts E-1, E-7, E-9, E-10 (§ 9), points non vérifiés n° 1, 5, 6, 7, 8, 9 et 10 (§ 10).
3. `docs/implementation/06-verbum-scripts.md` : plan **r1.2** (à lire une fois ; puis seulement les sections utiles à U4 : § 4.C, § 4.D, § 4.E, § 4.F (intégration ECS et Scheduler, observation des sorties, borne des émissions), § 4.G, § 5, § 6, § 7.4, § 7.6, § 8 ligne U4, § 9, § 14).
4. `docs/architecture/adr/ADR-Sc01-scripts.md` et `ADR-000-language-and-material-invariants.md` (points 1, 3 et 4). **ADR-003 (contrat d'un système), ADR-004 (construction du modèle d'exécution) et `01-runtime-ecs.md` (§ 4 flux de données, § 5 modèle d'exécution)** sont fournis avec les documents de référence : ils fondent l'ordre déduit (point ci-dessus) et la règle « produire est libre ; résoudre est unique » (X15). Le plan cite aussi ADR-001 (point 5, capacité fixe), ADR-006 (commandes différées, SR-6) et ADR-S01 (point 7, `Binding`/`Locate`) : demandez-les nommément **seulement** si une décision d'U4 en dépend.
5. `docs/implementation/05-verbum-assets-execution.md` : § 3.2 (portes G-CM), **§ 9 écart E-12** (durée de vie sous `Binding<'a>` et `Anchored`), E-13 et E-14 seulement.
6. `tree.md` : fait foi sur les chemins du dépôt. **Le `tree.md` à utiliser doit être régénéré après U3** (il doit lister `crates/extensions/script/` avec son `Cargo.toml` et `src/{lib,state,emissions,table,step}.rs`) ; l'utilisateur fournit la version régénérée ou la confirme.
7. Les **rapports de fin d'unité validés** : U1 (avec ses deux compléments : mention SCR-R5 ; modes de détection des mutations), U2 (avec son complément sur M-2.16b) et **U3 (avec l'annexe des résultats réels communiqués par l'utilisateur)**, pour l'état réel, les ajustements de noms et les écarts consignés.

Les autres ADR, `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md` ne se relisent que si U4 les cite ; vous les demandez alors nommément.

## 4. Hiérarchie normative

1. Les ADR et le corpus gouvernent.
2. Le plan `06-verbum-scripts.md` r1.2 (validé, inchangé) définit les exigences de l'étape.
3. Le document d'exécution organise les sessions et ne contient aucune règle de conception.
4. Ce handoff ne contient aucune règle de conception.

Divergence plan / exécution : le plan prévaut ; plan / ADR : l'ADR prévaut ; exécution / handoff : le document d'exécution prévaut. Dans tous les cas, la divergence est signalée à l'utilisateur et jamais résolue par la session.

## 5. Statut des documents à la passation

- **Plan `06-verbum-scripts.md` r1.2** (1er novembre 2026) : **validé** ; arbitrages A-S1 à A-S5 arrêtés. Corrigé le 5 novembre 2026, sur demande de l'utilisateur, pour le seul chemin `crates/script` → `crates/extensions/script` (4 occurrences) ; rien d'autre n'est modifié.
- **`06-verbum-scripts-execution.md`** (révision du 3 novembre 2026) : **validé** ; corrigé le 5 novembre 2026 pour le seul chemin `crates/extensions/script` (10 occurrences). Des mentions de son en-tête et de son § 3.1 (F1) sont datées de sa rédaction (« aucune unité ne démarre avant … U1 », « handoffs d'U2 à U5 non produits », `members` à sept entrées sans `verbum-script`) : signalé en AMB-5, sans effet sur U4.
- **Handoffs d'U1 et d'U3** : validés. `06-verbum-scripts-implementation-handoff.md` (U1) mentionne encore `crates/script` à sa ligne 188 (mention historique, non modifiée).
- **Ce handoff** : soumis à la validation de l'utilisateur au moment de sa rédaction. Tant que la validation n'est pas communiquée à la session, elle l'indique en une ligne et attend.
- Si l'utilisateur demande une correction ultérieure, appliquez **uniquement** cette correction, rendez le fichier **complet** (pas de diff), ne modifiez rien d'autre, attendez la validation.

## 6. Interdiction de coder avant instruction

**Aucune ligne de code, aucun fichier, aucune commande de build ou de test, avant que l'utilisateur ait écrit : « démarrez l'unité U4 ».** À la lecture initiale, accusez réception en cinq lignes maximum, en indiquant le statut des documents tel qu'au § 5, puis demandez les pièces manquantes de la liste du § 7 (le cas échéant). **Le handoff ne vaut pas autorisation.**

## 7. Périmètre et découpage

**Une session Claude par unité** [Validé, utilisateur, 3 novembre 2026] : U1, U2, U3, U4, U5 dans des sessions distinctes. **Cette session traite U4 seulement.** À la fin d'U4 : rapport selon le gabarit du § 12, puis arrêt avec « En attente de la validation de l'unité U4. ». Elle ne prépare ni ne propose U5.

| Unité | Contenu | Cette session |
|---|---|---|
| U1 | lecteur `script.bin` dans `verbum-assets` | validée |
| U2 | compilation `script.toml` → `script.bin` dans `verbum-forge`, `check_flow` | validée |
| U3 | crate `verbum-script` (`crates/extensions/script/`) : état, émissions, table, `step` | validée |
| **U4** | assemblage Scheduler/ECS (tests de `verbum-shell`), interface fixe, montage emprunté ; CT-1 à CT-3, SY-0, SR-1 à SR-7 | **oui, seule** |
| U5 | démonstrateur de bout en bout (`script_loaded.rs`, DM-1 à DM-3, exemple), clôture, MSRV, rapport « non démontré » | non |

**U4 ne traite ni le démonstrateur d'U5 ni aucun backend graphique** : aucun `script_loaded.rs`, aucun exemple `script_headless.rs`, aucune boucle `run_loop`, aucun Shell source, aucune visualisation. Aucune anticipation d'U5.

**Objectif d'U4** [Plan § 8, repris] : assembler `World`, `Scheduler`, systèmes d'assemblage et interpréteur ; prouver l'interface fixe (CT), la compatibilité avec `ScriptTable<'a>` empruntant des octets (SY-0), la suspension et la reprise (SR-1 à SR-7). Invariants visés : X2, X3, X5, X6, X8, X14, X15 ; T3 à T9, T14, T15 ; G-LT, G-ORD-S, G-ST ; plan § 4.E, § 4.F, § 4.G.

**Fichiers à créer ou modifier** [Exécution]. Créés : `crates/engine/shell/tests/script_support/mod.rs` (assemblage partagé, E-1) et `crates/engine/shell/tests/script_suspend.rs`. Modifié : `crates/engine/shell/Cargo.toml` (`[dev-dependencies]` : ajout de `verbum-script.workspace = true`, **rien d'autre**). **Aucune modification de `crates/engine/shell/src/**`, de `crates/engine/schedule/**`, de `crates/engine/ecs/**`, de `crates/engine/render/**`, de `crates/engine/verbum/**`, de `crates/engine/assets/**` (inchangé depuis U1), de `crates/engine/forge/**` (inchangé depuis U2), de `crates/extensions/script/**` (inchangé depuis U3) ni du `Cargo.toml` racine.**

**Fichiers à fournir (U4)**, versions courantes du dépôt réel, tels que rendus par les unités précédentes :

| Fichier | Usage |
|---|---|
| `Cargo.toml` (racine) | lecture seule ; revérification de F1 (huit membres depuis U3 : `crates/engine/{ecs,schedule,shell,render,assets,forge,verbum}` et `crates/extensions/script`) et de l'entrée `verbum-script` de `[workspace.dependencies]` |
| `crates/engine/shell/Cargo.toml` | **modifié** (une ligne) ; revérification de F2 |
| `crates/engine/shell/tests/level_loaded.rs`, `tests/borrowed_resource.rs` | lecture seule : **modèles** de l'assemblage emprunté (`Anchored`, `Binding`, `Locate`, systèmes, `load`, observation de `TRANSCRIPT`-like) |
| `crates/engine/schedule/src/{system,access,data,contract,model,scheduler}.rs` et **`order.rs`** (ajout de ce handoff, AMB-4) | lecture seule : `System`, `Binding`, accès, contrats, `ModelError`, `Scheduler::new`, `run_tick`, `contract_of`, `SystemList::fill`, ordre déduit des contrats et règles de propriété (`order_phase`, `check_ownership`) |
| `crates/engine/schedule/examples/headless.rs` | lecture seule : modèle d'assemblage |
| `crates/engine/ecs/src/{lib,world,component_set,command,resource,values,storage}.rs` | lecture seule : `World`, `ComponentSet` (modèle : doctest de `component_set.rs`), `CommandBuffer`, `ResourceSlot`, `SparseSet` |
| `crates/engine/assets/src/{script,format,lib}.rs` | lecture seule, **tels que rendus en U1** (`Script`, `Instruction`, `Format::Script`, `Asset::Script`, exports) |
| `crates/engine/forge/src/{lib,batch,script}.rs` | lecture seule, **tels que rendus en U2** (`compile_script`, `compile_assets`, `AssetSource`, `AssetKind::Script`, `CompiledSet`) |
| `crates/extensions/script/Cargo.toml`, `src/{lib,state,emissions,table,step}.rs` | lecture seule, **tels que rendus en U3** |
| **Ajouts de ce handoff** (signalés en AMB-4) : `crates/engine/assets/src/{load,manifest}.rs` ; `crates/engine/forge/src/sprite.rs` ; `crates/engine/schedule/tests/support/mod.rs` | lecture seule : signature de `load` et `SlotIndex` (F5, F6) ; définition de `ForgeError` et de ses variantes de script (`BudgetExceeded`, `CycleWithoutYield`) pour SR-7 ; précédent d'assemblage partagé (E-1) |
| `tree.md` régénéré | fait foi sur les chemins |

Si l'utilisateur fournit un fichier non listé, la session le lit mais ne le modifie pas.

**État réel d'U1, U2, U3, utile à U4** [Vérifié lors de la rédaction sur les rapports d'U1 à U3 et sur les fichiers livrés ; à revérifier].
- **`verbum-assets`** exporte `Script<'a>` (`Clone, Copy, Debug, PartialEq, Eq` ; `read`, `len() -> u16`, `is_empty`, `instruction(u16) -> Option<Instruction>`), `Instruction`, `Format::Script`, `Asset::Script(Script<'a>)`, `ReadError::Instruction`, les constantes `SCRIPT_*` (`SCRIPT_REGISTERS: usize = 4`, `SCRIPT_MAX_STEPS_PER_TICK: usize = 16`), `SlotIndex` (`new(u32)`, `get() -> u32`, `const`), `load` (signature F5).
- **`verbum-forge`** exporte `compile_script(source: &str) -> Result<Vec<u8>, ForgeError>`, `AssetKind::Script`, `compile_assets`, `AssetSource`, `CompiledSet`, `ForgeError` (variantes de script, dont `BudgetExceeded { entry }` et `CycleWithoutYield`). Le crate est `std` ; aucun crate runtime n'en dépend.
- **`verbum-script`** (`crates/extensions/script/`, package `verbum-script`, `no_std` sans `alloc`, seule dépendance `verbum-assets`, aucune dev-dépendance) exporte, tel que livré en U3 :
  - constantes : `SCRIPT_REGISTERS`, `SCRIPT_MAX_STEPS_PER_TICK` (réexportées), `ENCODED_LEN` (= 26) ;
  - `ScriptState` (`Clone, Copy, Debug, PartialEq, Eq`, champs privés ; `start(SlotIndex) -> Result<Self, SlotTooLarge>`, `script() -> u16`, `pc() -> u16`, `status() -> Status`, `wait() -> u32`, `registers() -> [i32; SCRIPT_REGISTERS]`, `to_bytes() -> [u8; ENCODED_LEN]`, `from_bytes(&[u8; ENCODED_LEN]) -> Result<Self, StateError>`) ; `Status`, `Fault`, `SlotTooLarge` et `StateError` (structures unitaires `Clone, Copy, Debug, PartialEq, Eq`) ;
  - `Emission { pub channel: u16, pub value: i32 }` (`Copy`) ; `Emissions<const CAP: usize>` (`new` `const fn`, `push -> Result<(), Full>`, `as_slice`, `len`, `is_empty`, `capacity`, `clear` ; `Clone`, `Debug`, `PartialEq` manuel sur la tranche observable) ; `Full` ; `emissions_bound_holds(instances: usize, cap: usize) -> bool` (`const fn`) ;
  - `ScriptTable<'a, const CAP: usize>` (`new` `const fn`, `deposit(SlotIndex, Script<'a>) -> Result<(), TableError>`, `get(u16) -> Option<&Script<'a>>` ; `TableError::{OutOfRange, Occupied}` ; **ni `Clone`, ni `Debug`, ni `Default`**) ;
  - `step<'a, const CAP: usize>(script: Option<&Script<'a>>, state: &mut ScriptState, sink: &mut Emissions<CAP>) -> Result<(), Full>`.
  - **`Emissions` et `ScriptTable` n'ont aucune implémentation publique de `Default`** (AMB-4 d'U3, tranché par l'utilisateur) : voir AMB-3 (§ 11).
  - Les transitions internes de `ScriptState` sont `pub(crate)` : l'assemblage n'a que `start`, `from_bytes` et `step`.

**Spécification d'U4 reprise du document d'exécution** (U4 du § 4 et § 3.3).

*Assemblage partagé* (`script_support/mod.rs`) [Exécution, E-1] : module inclus par `#[path = "script_support/mod.rs"] mod support;` dans `script_suspend.rs` (U4) et, plus tard, `script_loaded.rs` (U5), comme `tests/support/mod.rs` de `crates/engine/schedule`. Contenu : `Anchored<'a, T>` ; `Beat(u32)` ; `Components<'a>` ; `Rs<'a>` ; `Parts<'w, 'a>`, les `impl Locate` (une par catégorie), `B<'a>` (`Binding`) ; les systèmes `Clear`, `Collect`, `Interpreter`, `Drift` et, pour les tests de contrôle, `WideInterpreter` et `RivalWriter` ; les listes de systèmes ; des fonctions d'assemblage (`build_world`, `load_scripts`, `launch`, `capture`, `reconstruct`). Un attribut `#[allow(dead_code, reason = "…")]` sur la déclaration du module est admis **si et seulement si** nécessaire pour `clippy -D warnings` (chaque fichier de test n'utilise qu'une partie), rapporté au rapport (E-1).

*Types retenus de l'assemblage* [L, E-12, E-1, à compiler en U4] :
- `Beat(u32)` : donnée `Copy`, **sans durée de vie**. `Anchored<'a, T>` : enveloppe `Clone, Copy` quand `T: Copy`, champ interne accessible, `PhantomData<&'a ()>`.
- `Components<'a>` : `states: SparseSet<ScriptState, SPARSE, STATE_DENSE>` et `beats: SparseSet<Anchored<'a, Beat>, SPARSE, STATE_DENSE>` ; **ordre des paramètres : `SPARSE, DENSE`** ; **aucun littéral `4, 4`**. `Components<'a>` porte la durée de vie, car le stockage de `Beat` est ancré.
- `Add<'a>` : `State(ScriptState)`, `Beat(Anchored<'a, Beat>)` ; `Remove` : deux variantes sans donnée ; l'implémentation de `ComponentSet` pour `Components<'a>` reflète ce stockage (`insert`, `remove`, `values_mut`, compteurs).
- `Rs<'a>` : `scripts: ScriptTable<'a, 6>` (**non enveloppée**, car elle porte déjà `'a`), `emissions: Anchored<'a, Emissions<EMISSIONS_CAP>>`, `transcript: Anchored<'a, Emissions<EMISSIONS_CAP>>`. Le champ `scripts` est lu par le **type du slot de ressource** : la forme exacte de `Rs<'a>` (champs de type `ResourceSlot<…>` ou autre) se déduit de `resource.rs`, `world.rs` et du modèle `level_loaded.rs` (§ 11, points à confirmer ; AMB-3).
- **`Anchored` est limité à `Clear`, `Collect`, `Drift` et au stockage de `Beat`. Il n'est jamais employé par `Interpreter`.** `Drift` conserve **strictement** son contrat `Write<BEAT>` ; aucun accès ni aucune catégorie supplémentaire ; les cinq catégories restent `STATE`, `SCRIPTS`, `EMISSIONS`, `TRANSCRIPT`, `BEAT`.
- **Les comparaisons de transcripts (SR-1, SR-2, SR-5) et celles d'EM-3 portent sur les `Emissions<EMISSIONS_CAP>` internes, jamais sur les enveloppes `Anchored`.**
- Le dépôt de `load_scripts` ne retient que `Asset::Script` ; toute autre variante laisse l'emplacement à `None` dans `ScriptTable`, qui ne reconnaît aucun format.

*Systèmes* [Plan § 4.F] : `Clear` (phase 0, `Produce<EMISSIONS>` : `insert` d'un `Emissions` vide) ; phase 1, **déclarés dans l'ordre** `Collect`, `Interpreter`, `Drift` (**l'ordre de déclaration n'impose aucune dépendance** : les contraintes d'ordre sont déduites des contrats par le modèle d'exécution, ADR-004 point 3 ; il ne sert qu'à départager les systèmes indépendants ; voir la note [Vérifié] ci-dessous) : `Collect` (`Read<EMISSIONS>`, `Produce<TRANSCRIPT>` : remplace **entièrement** `TRANSCRIPT` par un clone de `EMISSIONS`, sans condition, y compris vide), `Interpreter` (`Write<STATE>`, `Read<SCRIPTS>`, `Produce<EMISSIONS>` : pour chaque instance, copie l'`EntityId` à l'indice *i* de `entities()`, `get_mut`, appelle `step` avec `table.get(état.script())` ; `Err(Full)` ⇒ `unreachable!` avec message nommant la borne `EMISSIONS_CAP ≥ STATE_DENSE × SCRIPT_MAX_STEPS_PER_TICK`), `Drift` (`Write<BEAT>` : ajoute 1 à chaque `Beat`, champ interne de `Anchored<'a, Beat>`). `Scheduler::<B<'a>, L, 4, 2>`. **Note [Vérifié] sur l'ordre exécuté** (ADR-004 points 3 à 5 ; `crates/engine/schedule/src/order.rs` et `scheduler.rs`, lus le 5 novembre 2026, à revérifier sur les fichiers réels) : une dépendance n'est jamais déclarée, elle est déduite d'une relation production → consommation entre deux systèmes distincts d'une même phase (`precedes` : `writes ∪ produces` du producteur rencontre `reads` du consommateur) ; `order_phase` place chaque système après tous ceux dont il dépend et, parmi les systèmes libres, choisit le premier déclaré ; `Scheduler::new` construit le modèle une fois, `run_tick` exécute les phases dans l'ordre et, dans chaque phase, les systèmes dans l'ordre du modèle, avec `World::apply` à la fin de chaque phase. Appliqué au montage, deux points distincts : (1) **`Interpreter` → `Collect` est un ordre imposé par la dépendance production → consommation** (`Interpreter` produit `EMISSIONS`, `Collect` la lit) : il vaut quelle que soit la déclaration ; (2) **`Drift` est indépendant des deux autres systèmes** (il n'écrit que `BEAT`, que personne ne lit en phase 1) : sa position après `Collect` découle **uniquement du critère de départage par ordre de déclaration** de `order_phase` (parmi les systèmes libres, le premier déclaré), non d'une dépendance. Avec la déclaration `Collect`, `Interpreter`, `Drift`, l'ordre obtenu est `Interpreter`, `Collect`, `Drift` ; `Clear` (phase 0) précède la phase 1 par la composition des phases. C'est ce qui rend `TRANSCRIPT` égal aux émissions du tick courant (SR-1) ; **aucune incohérence d'ordonnancement n'est relevée** (audit du 5 novembre 2026, point examiné et clos par ces sources). L'ordre déduit n'est pas un invariant à tester en plus : SR-1, SR-2 et SR-5 en observent l'effet, CT-3 en contrôle les composantes (lecteurs, producteurs, phases), aucun test nouveau n'est ajouté. `check_ownership` accepte le montage (`EMISSIONS` n'est que produite, jamais écrite ; `STATE` n'a qu'un écrivain ; `BEAT` n'a qu'un écrivain) et refuse `RivalWriter` (second `Write<STATE>` dans la même phase : `OwnershipError::MultipleWriters`, CT-2).

Initialisation à l'assemblage : `EMISSIONS` et `TRANSCRIPT` **vides et présents**, table vide avant `load`. Le `Err(Full)` de l'adaptateur n'est ni une politique de saturation ni un substitut au dimensionnement ; **aucun `panic!` comme politique de saturation** (seul `unreachable!` de violation de contrat), **aucun `BufferFull`**, aucun `Err(Full)` ignoré.

*Constantes de scène* (§ 3.3) : `ENTITIES = 4`, `SPARSE = 4`, `STATE_DENSE = 4` (**employées dans les paramètres génériques des `SparseSet`, ordre `SPARSE, DENSE`, et non seulement dans la `const` assert**), `EMISSIONS_CAP = 64`, `SCRIPT_SLOTS = 6` (table de formats `[Format::Script; 6]`), slot de référence **4** (artefact 0 d'un lot d'un seul script), scripts de SR-5 : A au slot 4 (artefact 0), B au slot 1 (artefact 1) ; `SYSTEMS = 4`, `PHASES = 2` ; catégories `STATE`, `SCRIPTS`, `EMISSIONS`, `TRANSCRIPT`, `BEAT` (entiers attribués par l'assemblage, `< 64`) ; `Beat(u32)` composant `Copy` ; `const _: () = assert!(emissions_bound_holds(STATE_DENSE, EMISSIONS_CAP))` (chaque constante est aussi utilisée dans du code compilé, E-14).

*Source TOML du scénario de référence* (utilisée pour compiler par la Forge ; l'artefact attendu est le littéral de 68 octets ci-dessous) :

```
[[step]]
op = "set"
register = 0
value = 3

[[step]]
label = "loop"
op = "emit"
channel = 1
value = 10

[[step]]
op = "wait"
ticks = 3

[[step]]
op = "add"
register = 0
value = -1

[[step]]
op = "jnz"
register = 0
target = "loop"

[[step]]
op = "emit"
channel = 2
value = 99

[[step]]
op = "halt"
```

*Artefact de référence* (68 octets, littéral à la main, indépendant des constantes) :

| Octets | Contenu |
|---|---|
| `56 53 43 52` | identité `VSCR` |
| `01 00` | version 1 |
| `3A 00 00 00` | charge utile 58 (`2 + 8·7`) |
| `07 00` | `n = 7` |
| `03 00 00 00 03 00 00 00` | 0 : `Set r0, 3` |
| `01 00 01 00 0A 00 00 00` | 1 : `Emit canal 1, valeur 10` |
| `02 00 00 00 03 00 00 00` | 2 : `Wait 3` |
| `04 00 00 00 FF FF FF FF` | 3 : `Add r0, −1` |
| `06 00 01 00 00 00 00 00` | 4 : `JumpIfNonZero r0, cible 1` |
| `01 00 02 00 63 00 00 00` | 5 : `Emit canal 2, valeur 99` |
| `00 00 00 00 00 00 00 00` | 6 : `Halt` |

*Trajectoire de référence* (instance lancée au tick 1, script au slot 4) : tick 1 : `Set`, `Emit (1,10)`, `Wait 3` ; ticks 2 et 3 : attente ; tick 4 : reprise, `Add` (r0 = 2), `JumpIfNonZero` pris, `Emit (1,10)`, `Wait 3` ; ticks 5 et 6 : attente ; tick 7 : même tour (r0 = 1) ; ticks 8 et 9 : attente ; tick 10 : reprise, `Add` (r0 = 0), `JumpIfNonZero` non pris, `Emit (2,99)`, `Halt` ; ticks 11 et 12 : aucun changement. Émissions : tick 1, 4, 7 : (1,10) ; tick 10 : (2,99) ; aucune aux autres ticks. Coûts Forge : entrée 0 : 3 ; entrée 3 : 4. Encodage canonique en fin de tick, 26 octets chacun (les 12 derniers octets sont nuls pour `r1` à `r3`, `r0` figure aux octets 10 à 13) :

| Fin de tick | Octets 0 à 13 |
|---|---|
| 0 (état initial) | `04 00 00 00 00 00 00 00 00 00 00 00 00 00` |
| 1 | `04 00 03 00 01 00 03 00 00 00 03 00 00 00` |
| 2 | `04 00 03 00 01 00 02 00 00 00 03 00 00 00` |
| 3 | `04 00 03 00 01 00 01 00 00 00 03 00 00 00` |
| 4 | `04 00 03 00 01 00 03 00 00 00 02 00 00 00` |
| 10 (et 11, 12) | `04 00 06 00 02 00 00 00 00 00 00 00 00 00` |

*Second script de scène* (SR-5, script B) : `emit 3,7 ; wait 1 ; emit 3,8 ; halt` (quatre pas ; coûts Forge : entrée 0 : 2 ; entrée 2 : 2). Lancé **avant le tick 1** (variante ii) : émissions tick 1 : (3,7) ; tick 2 : (3,8) ; `Halted` au tick 2 (`pc = 3`). Lancé **après le tick 2** (variante i) : tick 3 : (3,7) ; tick 4 : (3,8). Transcripts de la variante (ii) en multiensemble : tick 1 {(1,10), (3,7)} ; tick 2 {(3,8)} ; ticks 4 et 7 {(1,10)} ; tick 10 {(2,99)} ; autres ticks vides. Variante (i) : tick 1 {(1,10)} ; tick 3 {(3,7)} ; tick 4 {(1,10), (3,8)} ; ticks 7 {(1,10)} ; tick 10 {(2,99)} ; autres ticks vides.

## 8. Méthode

- **Rouge avant vert** quand c'est possible : `Collect`, `Interpreter`, `Clear`, `Drift` sont d'abord écrits avec des corps `unimplemented!()` (**types et accès réels**) ; SR-1 à SR-7 et SY-0 échouent **par panique** pour la raison attendue, constat rapporté. **Un échec de compilation n'est pas un rouge valide.** **Sans rouge possible** : CT-1 à CT-3 (contrats déjà typés, indépendants des corps) : vert au premier passage, sensibilité établie par mutation (M-4.3, M-4.5, M-4.6).
- **Sensibilité par mutation** sur copie isolée **hors de l'arbre de travail**, une mutation à la fois, jamais présentée comme rouge initial ; le rapport nomme pour chaque mutation le test détecteur **et les autres assertions échouées**, avec le **mode de détection observé** (échec d'assertion, panique, erreur de construction du `Scheduler`, échec de compilation pour M-4.12 seulement ; rien de supposé). **Une mutation non détectée est un arrêt, sauf si elle est établie équivalente** (comportement observable inchangé dans le domaine atteignable), sans test artificiel. Aucune détection par échec de compilation n'est admise, **sauf M-4.12** (exception du plan § 7.6).
- **Précisions issues des validations d'U1, U2 et U3** [Validé, utilisateur] : (a) lorsqu'une assertion est ajoutée **après** le rouge pour détecter une mutation, le rapport le mentionne explicitement ; (b) lorsque plusieurs sous-cas d'un même test peuvent détecter une mutation et que le **premier échec observé n'est pas le détecteur annoncé**, le sous-cas annoncé est exécuté **indépendamment**, sur une copie isolée, en le plaçant en tête dans un harnais temporaire de mutation, sans test permanent supplémentaire ni modification du code de production ; le rapport donne les deux résultats (précédent : M-2.16b) ; (c) un écart entre détecteur annoncé et échecs observés est **expliqué par le code et non par supposition** (précédent U3 : les assertions annoncées « ST-4 » n'ont pas détecté M-3.7, M-3.8, M-3.10, M-3.14, M-3.15 parce que ST-4 vérifie la validité des états et non leurs valeurs ; M-3.11 détectée d'abord par la panique d'un `expect` d'assistant) ; l'écart est consigné, il n'est un arrêt que s'il reste inexpliqué.
- **Mutations propres à U4 :**
  - **M-4.8** emploie un `static` atomique **sur copie isolée seulement** (exception de G-ST, E-9) ; `cargo test -- --test-threads=1`.
  - **M-4.14** est un **contrôle d'invariance (CI-S)** : SR-1 à SR-7 et SY-0 doivent **rester verts** ; un échec prouve une dépendance à l'ordre : **arrêt** (G-ORD-S).
  - **M-4.16** est un **harnais de reconstruction seulement**, pas une mutation de production.
  - **M-4.10** est une **application prématurée des commandes** (moment de `World::apply`), et non un accès à `World::add_component` (`pub(crate)`, inaccessible, F10).
  - **M-4.12** est détectée par la `const` assert de l'assemblage (échec de compilation), complétée par EM-2 (U3).
- **Portes.**
  - **G-LT est une véritable porte d'arrêt, sans contournement** : un échec de compilation de durée de vie du montage emprunté (SY-0 ou `ScriptTable<'a, 6>` dans le `World` sous un `Scheduler`) arrête l'unité. Sont **interdits** : `'static`, fuite de mémoire (`Box::leak`), `unsafe`, cast d'octets, `transmute`, `static`, changement de la propriété des octets, modification de `crates/engine/schedule` ou de `crates/engine/ecs`. Le **message exact du compilateur** est rapporté.
  - **E-12 / E0477** : `Anchored` est admis pour `Clear`, `Collect`, `Drift` et le composant `Beat` stocké. **Si `Interpreter` exige lui aussi un contournement (erreur E0477 sur son accès, qui nomme `ScriptTable<'a, 6>`), c'est une divergence du plan : arrêt**, message exact rapporté, aucun `Anchored` sur `Interpreter`. L'hypothèse « l'accès de `Interpreter` compile sans `Anchored` » est un [Déduit] du motif de `ProduceTiles` (étape 5) : à confirmer ou infirmer par U4 (§ 10 point 7 du document d'exécution).
  - **G-ORD-S** : aucun test, exemple ou API n'affirme l'ordre des émissions entre instances ni l'ordre dense d'un stockage ; comparaisons **en multiensemble, multiplicité conservée** ; aucune comparaison par un ensemble qui efface les doublons ; une exigence qui ne peut s'énoncer sans ordre est un arrêt. M-4.14 en est le contrôle.
  - **G-ST** : arrêt si un besoin de `static`, `thread_local`, `Cell`/`RefCell`/atomique, `async`, `unsafe`, ou d'un état dans un système apparaît (exception de test unique : M-4.8, copie isolée).
  - **G-CM** : U4 ne touche ni `load`, ni `verbum-assets` ; un arrêt G-CM n'est attendu que si un fichier de F14 devait changer. Le rapport l'indique après vérification (aucun fichier d'`assets` ni de `forge` modifié).
- **R-ADAPT** : aucun fichier existant n'est modifié en U4 hors `crates/engine/shell/Cargo.toml` ; une adaptation mécanique hors des fichiers listés est exposée avant d'être faite et consignée. Aucun test d'une étape antérieure (F14, dont `lifecycle`, `pacing`, `acquisition`, `acceptance`, `frame_loop`, `borrowed_resource`, `level_loaded` et les doctests LT-3 à LT-5t) n'est modifié.
- **Contrôles propres à U4** (document d'exécution, U4, repris) : `cargo tree -p verbum-shell` (dev : `verbum-assets`, `verbum-ecs`, `verbum-forge`, `verbum-render`, `verbum-schedule`, `verbum-script`) ; `cargo tree -i verbum-script --workspace` (seul `verbum-shell`, en dev-dépendance) ; `cargo tree -i verbum-forge --workspace -e normal` (aucun dépendant) ; `cargo tree -p verbum-script -e normal` (seule `verbum-assets`) ; **revue G-ST** rapportée : recherche textuelle dans `crates/extensions/script/**` et dans les corps des systèmes d'assemblage de `static`, `thread_local`, `Cell`, `RefCell`, `Atomic`, `async`, `await`, `thread`, `Box`, `Vec`, `unsafe`, `dyn` (le `Vec` de l'observateur de l'appelant, côté Plateforme, est admis et nommé comme tel, plan § 4.F (c)) ; revue : `EMISSIONS` n'est lue par aucun code hors de `Collect` ; revue : aucune modification de `crates/engine/shell/src/**`, `crates/engine/schedule/**`, `crates/engine/ecs/**` (diff). Les revues ne sont jamais présentées comme des preuves statiques.
- **Natif** : quatre contrôles ; `cargo test -p verbum-shell` ; `cargo test -p verbum-script`. **Miri (utilisateur)** : `cargo +nightly miri test -p verbum-shell` (suite complète ; SR-7 réduit sous Miri selon E-7 ; exclusions précédentes de `verbum-forge` inchangées). Si la durée sous Miri dépasse 10 minutes : rapporter et arbitrer, ne pas désactiver l'isolation, aucun `--skip` ni `ignore` sans décision. Les tests s'exécutent en profil `dev` (défaut de `cargo test`).
- **Cardinalité et coût, natif et Miri** :

| Test | Natif | Miri |
|---|---|---|
| SR-7 | famille complète (longueurs `MAX − 1` à `MAX + 2`, trois terminaisons, branchements des deux bras, deux issues) | **paire limite seulement** : un segment de `MAX` pas accepté et exécuté, un segment de `MAX + 1` refusé, un branchement à bras long accepté et exécuté avec ses deux issues ; **raccordement partiel**, rapporté (E-7) ; si la durée dépasse 10 minutes, rapporter et arbitrer |
| SR-2 | 13 reconstructions complètes | 13 reconstructions complètes (monde de 4 entités, ≤ 12 ticks chacune) |
| SR-1, SR-3, SR-4, SR-5, SR-6, SY-0, CT-* | complets | complets |

- **Compatibilité MSRV 1.85 dès l'écriture** : pas de let-chains (`let … else` à la place) ; **aucune constante utilisée seulement dans un `const _: () = assert!(…)`** (E-14) : chaque constante de scène est aussi utilisée dans du code compilé ; cadence = fermeture vide dans les tests (aucune cadence réelle en U4) ; aucune assertion sur l'horloge. La vérification MSRV est faite en U5 ; la compilation sous 1.85 de l'`Interpreter`, de `ScriptTable<'a, 6>` dans le montage et du `ComponentSet` ancré est un point non vérifié (§ 10 points 6 et 9 du document d'exécution) : signalé au rapport s'il n'est pas établi.
- **Tests dans `crates/engine/shell/tests/`** : le crate `verbum-shell` a ses dev-dépendances `verbum-assets`, `verbum-ecs`, `verbum-forge`, `verbum-render`, `verbum-schedule` (`.workspace = true`) ; U4 y ajoute `verbum-script`. Les scripts de test sont compilés **par la Forge** (`compile_script` ou `compile_assets`) et déposés par le **vrai `load`** ; SY-0 construit ses tampons d'octets **à l'exécution** et les déclare **avant** le `World` (G-LT).
- Chaque unité commence par **revérifier** les faits [Vérifié] qu'elle utilise sur les fichiers fournis : F1, F2, F5, F6, F8, F9, F10, F11 (§ 3.1 du document d'exécution), plus la lecture des modèles `level_loaded.rs`, `borrowed_resource.rs`, `headless.rs` et du doctest de `component_set.rs` (§ 10 points 1, 8 et 9 du document d'exécution).

**Tests d'U4** (document d'exécution, U4, repris sans modification).

| Test | Énoncé | Porte d'arrêt |
|---|---|---|
| CT-1 | `contract_of::<B<'static>, Interpreter>()` égal au littéral : `reads = {SCRIPTS}`, `writes = {STATE}`, `produces = {EMISSIONS}`, effets `NONE`. Les contrats de `Clear`, `Collect`, `Drift` sont contrôlés **séparément** (littéraux propres) et n'entrent pas dans cette égalité. `WideInterpreter` (même corps, un accès `Read<TRANSCRIPT>` de plus) : le contrôle **échoue**. `B<'static>` est l'instanciation retenue **du test de contrat seulement** ; elle ne fixe aucune durée de vie runtime : le contrat est indépendant de `'a`, et les montages exécutés (SY-0, SR-*) emploient `B<'a>` avec une durée de vie locale | oui (T3) |
| CT-2 | Seul `Interpreter` a `STATE` dans `writes ∪ produces` (liste de production) ; une liste contenant `RivalWriter` (second `Write<STATE>` dans la phase 1) : `Scheduler::new()` rend `Err` de variante `MultipleWriters` (ou, si le type d'erreur n'est pas publiquement nommable, `is_err()` et message `Debug` rapportés) | oui (T8) |
| CT-3 | Par `SystemList::fill` : `Collect` est l'unique lecteur de `EMISSIONS` ; `EMISSIONS` n'est produite que par `Clear` et `Interpreter` ; `Collect` est l'unique producteur de `TRANSCRIPT` ; `Clear` en phase 0 strictement avant `Collect` en phase 1 ; aucun système n'est dans une phase postérieure à la phase 1 | oui (X15, T15) |
| SY-0 | **Montage emprunté** : deux tampons d'octets construits à l'exécution (scripts de même forme, valeurs d'émission différentes, déclarés **avant** le `World`), table chargée par le vrai `load`, `run_tick` sous le vrai `Scheduler` : les transcripts diffèrent selon le tampon ; le montage compile (G-LT) | oui (G-LT) |
| SR-1 | Scénario de référence sur 12 ticks (instance au slot 4, `Beat` initial 0). **Avant le premier tick**, `TRANSCRIPT` est présent et vide. **Après chaque tick**, `TRANSCRIPT` est égal **exactement** aux émissions de ce tick : (1,10) aux ticks 1, 4, 7 ; (2,99) au tick 10 ; **vide aux ticks 2, 3, 5, 6, 8, 9, 11, 12 bien qu'une émission ait eu lieu à un tick antérieur** ; l'observateur extérieur (un `Vec` de l'appelant) reconstitue la suite de 12 transcripts. `Beat` vaut *k* après le tick *k*, **y compris pendant les attentes** ; `Beat` vaut 12 à la fin | oui (T7, T15) |
| SR-2 | **Reconstruction** (plan § 4.E point 4), pour **chaque** *t* de 0 à 12 : capture des 26 octets du composant après *t* ticks de la référence ; monde, ordonnanceur, table (rechargée depuis les mêmes artefacts), `Beat` initial à 100 ; instance recréée par `from_bytes` + `CommandBuffer::add_component` + `apply` ; ticks restants exécutés ; on exige (a) `to_bytes(from_bytes(capture)) == capture`, (b) l'encodage du composant égal à celui de la référence à chaque tick restant, (c) transcripts du suffixe égaux aux littéraux, (d) état final égal octet à octet à la ligne « 10 » du tableau ci-dessus. `Beat` du monde reconstruit n'influence aucune de ces assertions | oui (T4, T7) |
| SR-3 | `wait` persistant 3, 2, 1 en fin de ticks 1, 2, 3 ; aux ticks 2 et 3 **tous les octets autres que `wait` sont constants** ; aucune valeur 0 de `wait` en fin de tick, ticks 1 à 9 ; reprise au tick 4 : `r0` 3 → 2, `pc` revenu à 3, `wait` réarmé à 3, émission (1,10) ; octets de fin de ticks 1, 2, 3, 4, 10 égaux aux littéraux | oui (T6, T7) |
| SR-4 | Remplacement de l'ordonnanceur par un `Scheduler::new()` neuf **entre les ticks 4 et 5**, sur le même monde : suite d'encodages et de transcripts des ticks 5 à 12 identique à la référence | oui (T5) |
| SR-5 | **Deux instances.** Variante (i) : script A (slot 4, artefact 0) lancé avant le tick 1, script B (slot 1, artefact 1) lancé par `apply` **après le tick 2** : transcripts en multiensemble égaux aux littéraux ; chaque instance exécute le script de **son slot**. Variante (ii) : A et B lancés avant le tick 1, dans l'ordre (A, B) puis dans l'ordre (B, A) : transcripts égaux en multiensemble entre les deux ordres et aux littéraux ; états finaux égaux par instance. Aucune comparaison d'ordre d'émission entre instances | oui (G-ORD-S, C6, C13) |
| SR-6 | **Lancement par commande différée.** Quatre étapes dans cet ordre : (1) `world.spawn()`, puis `CommandBuffer::add_component(entité, Add::State(ScriptState::start(SlotIndex::new(4))?))` : la commande est **ajoutée au tampon** (appel réussi) ; (2) **le composant reste absent du `World`** (`world.components().states.get(entité)` rend `None`, ou `contains` faux) ; (3) `World::apply(&mut buffer)` est exécuté ; (4) **le composant devient présent** : `Running`, `pc = 0`, registres nuls, `script() == 4`. **Aucune écriture directe du composant** : `World::add_component` est `pub(crate)` dans `verbum-ecs` (F10) ; le seul chemin est le tampon puis `apply`. L'étape (2) précède strictement l'étape (3) | oui (C9, ADR-006) |
| SR-7 | **Script accepté par la Forge, jamais `Faulted(Budget)`.** Famille générée (TOML construit par code à partir de `SCRIPT_MAX_STEPS_PER_TICK`) : segments rectilignes de longueurs `MAX − 1`, `MAX`, `MAX + 1`, `MAX + 2` (terminés par `Wait`, par `Halt`, ou par la fin du programme), et branchements dont l'un ou l'autre bras est le plus long, **avec les deux issues du `JumpIfNonZero`** (registre mis à zéro, puis non nul avant le saut). Pour chaque script : `compile_script` ; **s'il est accepté**, compilation puis exécution par `step` sur plusieurs ticks : aucun état `Faulted(Budget)` n'est jamais observé ; `step` ne retourne pas de compteur d'instructions : **aucune mesure directe** du nombre d'instructions par tour n'est affirmée (E-8) ; la limite est établie par IN-budget, IN-bound et IN-end (U3) et par l'absence de `Faulted(Budget)` ; **s'il est refusé**, l'erreur est `BudgetExceeded` et la longueur du plus long chemin dépasse la limite | oui (T10, C14) |

**Rouge.** Attendu : SR-1 à SR-7 et SY-0 échouent par panique `not implemented` (corps `unimplemented!()`) ; CT-1 à CT-3 verts au premier passage (sans rouge possible).

**Mutations d'U4** (copies isolées ; document d'exécution, U4, repris sans modification ; 17 lignes ; pour M-4.8 : `cargo test -- --test-threads=1`). La colonne « Mode attendu » est une attente de rédaction, jamais un constat.

| # | Mutation | Détecteur | Autres assertions attendues en échec | Mode attendu |
|---|---|---|---|---|
| M-4.1 | `Collect` ne remplace pas le transcript quand l'accumulateur est vide | SR-1 (transcript périmé aux ticks 2 et 3) | DM-1 (U5, hors session) | assertion |
| M-4.2 | `Collect` ajoute au transcript au lieu de le remplacer | SR-1 | SR-5 | assertion |
| M-4.3 | `Clear` supprimé de la liste (`SYSTEMS = 3`) | SR-1 (émissions cumulées entre ticks) | CT-3 | assertion (SR-1 ; CT-3 : phases) |
| M-4.4 | `TRANSCRIPT` non initialisé à l'assemblage (slot vide) | SR-1 (assertion avant le premier tick) | — | assertion |
| M-4.5 | `RivalWriter` ajouté à la liste d'assemblage | CT-2 (explicite) | toutes les exécutions (`Scheduler::new()` échoue) : rapporté comme telles | CT-2 : assertion sur `Err` ; exécutions : panique de la construction (`expect`/`unwrap`) |
| M-4.6 | `Interpreter` déclare un accès supplémentaire (`Read<TRANSCRIPT>`) | CT-1 **seul** (CT-3 reste vert : il ne porte pas sur les lectures de `TRANSCRIPT`) | — | assertion (égalité de contrats) |
| M-4.7 | `Drift` absent de la liste | SR-1 (`Beat`) | SR-3 | assertion |
| M-4.8 | **état hors World** : `pc` conservé dans un `static` atomique et restauré dans l'état avant chaque `step` (copie isolée **seulement**, exception de G-ST) | SR-2 (une reconstruction à partir d'une capture reprend avec la valeur du `static` d'une exécution antérieure) | SR-4 possible ; **si SR-2 et SR-4 sont tous deux verts : arrêt** (mutation non détectée) | assertion |
| M-4.9 | l'adaptateur appelle `step` deux fois par instance et par tick | SR-1 (émissions décalées) | SR-3, SR-2 | assertion |
| M-4.10 | **application prématurée des commandes** : dans `launch`, `World::apply` est appelé immédiatement après `CommandBuffer::add_component`, **avant** la vérification de l'absence du composant | SR-6, étape (2) | SR-5 (variante (i)) si le lancement différé est observé avant le tick prévu | assertion |
| M-4.11 | l'index de dépôt dans la table est le rang d'appel du dépôt au lieu du `SlotIndex` | SR-5 (A au slot 4, B au slot 1) | SR-1 (script du slot 4 non trouvé : `Faulted(NoScript)`) ; SCR-C1 non (autre crate) | assertion |
| M-4.12 | `EMISSIONS_CAP` abaissé sous `STATE_DENSE × SCRIPT_MAX_STEPS_PER_TICK` | **`const` assert de l'assemblage (échec de compilation)** : exception admise du plan § 7.6 ; complété par EM-2 (U3, échec d'assertion, non rejoué) | — | échec de compilation (seule exception) |
| M-4.13 | `Collect` ignore `EMISSIONS` et produit systématiquement un transcript vide | SR-1, aux ticks 1, 4, 7 et 10 | SR-2 (c), SR-5, DM-1 (U5, hors session) | assertion |
| M-4.14 | l'adaptateur itère les instances en ordre inverse | **contrôle d'invariance (CI-S)** : SR-1 à SR-7 et SY-0 doivent **rester verts** ; un échec prouve une dépendance à l'ordre : **arrêt** (G-ORD-S) | — | aucune détection attendue (tout vert) |
| M-4.15 | `ScriptState::start` remplacé par une construction directe d'un état `Waiting` avec `wait = 0` (mutation de U3 rejouée dans l'assemblage) | **équivalente à M-3.2 de U3** : détectée en U3 par ST-2 ; **non rejouée en U4** | — | sans objet (rapportée équivalente) |
| M-4.16 | **harnais de reconstruction seulement** : dans `reconstruct`, l'état de l'instance n'est plus restitué par `from_bytes(capture)` mais lu dans la trajectoire de référence à l'indice `Beat` du monde neuf (initial 100, ramené modulo 13, soit 9) ; le reste du code (systèmes, `step`, table) est intact | SR-2 (assertion (a) ou (b) pour tout *t* ≠ 9 ; la reconstruction doit reposer sur les seuls octets capturés) | — | assertion |
| M-4.17 | **table indexée par l'`artifact_index` du manifeste au lieu du `SlotIndex`** : dans `load_scripts`, le dépôt ignore le `SlotIndex` reçu et utilise l'indice d'artefact du script dans le lot (A : 0, B : 1) ; `ScriptTable` et `step` intacts. Distincte de M-4.11 (rang d'appel) | SR-5, SR-1 et SY-0 (le script du slot 4, artefact 0, est déposé à l'indice 0 et l'instance du slot 4 passe en `Faulted(NoScript)`) | — | assertion |

**Remarque sur M-4.10.** La mutation ne modifie que le **moment** d'application des commandes : l'ordre `add_component` → absence vérifiable → `apply` → présence est celui que SR-6 impose. Elle n'est pas un accès direct à `World::add_component`.

**Cardinalité** : monde de `ENTITIES = 4` ; scripts de quelques dizaines d'octets ; SR-7 : voir tableau ci-dessus.

**Attente, pas constat** : rouge constaté par panique ; vert ; 17 lignes de mutations traitées (M-4.12 par la `const` assert, M-4.14 en invariance, M-4.15 équivalente, M-4.16 harnais) ; quatre contrôles verts ; `cargo tree` conformes ; revue G-ST rapportée ; Miri vert. **Critère de sortie** : CT-1 à CT-3, SY-0, SR-1 à SR-7 verts ; mutations traitées (dont M-4.14 en invariance) ; revue G-ST rapportée ; quatre contrôles verts ; Miri conforme. « En attente de la validation de l'unité U4. »

## 9. Arbitrages et limites clos (non rouverts)

- **A-S1 à A-S5** [Validé] : nouveau crate `verbum-script` (`no_std`, sans `alloc`, dépendance normale unique `verbum-assets`, aucune dev-dépendance) ; fonction d'exécution pure `step` dans ce crate, **système interpréteur adapté dans l'assemblage (U4, de l'ordre d'une vingtaine de lignes)** ; validation structurelle des instructions dans `Script::read` ; `Wait 0` rejeté ; `Waiting ⇒ wait ≥ 1` ; analyse de flot sur tout le graphe avec coupure aux `Wait` (Forge) ; **`TRANSCRIPT` remplacé à chaque tick** ; `ScriptTable` indexée directement par `SlotIndex`, sans compactage ni table de correspondance ; encodage canonique de l'état exclusivement pour la reconstruction.
- **A-S4 / E-12** : `Anchored<'a, T>` pour `Clear`, `Collect`, `Drift` et le composant `Beat` stocké ; jamais pour `Interpreter`, `ScriptTable<'a, 6>` ni `ScriptState`.
- **A-S5 / observation des sorties** : `TRANSCRIPT` est une Ressource transitoire de même type `Emissions<EMISSIONS_CAP>` que `EMISSIONS` ; remplacement intégral à chaque tick, y compris vide ; aucune émission d'un tick précédent ne subsiste dans le `World` ; **l'historique est extérieur au `World`** (observateur de l'appelant, lecture seule de `TRANSCRIPT` après chaque `run_tick`) ; `EMISSIONS` n'est jamais lue hors des systèmes ; aucun `LOG_CAP`.
- **Borne des émissions** : `EMISSIONS_CAP ≥ STATE_DENSE × SCRIPT_MAX_STEPS_PER_TICK`, portée par `emissions_bound_holds` (`checked_mul`) dans une `const` assert de l'assemblage ; la saturation est impossible en configuration conforme ; la branche `Err(Full)` n'est qu'une détection de violation du contrat (`unreachable!`).
- **Composants** : un `ScriptState` par entité ; lancement uniquement par `CommandBuffer::add_component` puis `World::apply` (C9, ADR-006) ; `Beat` composant générique `Copy` (compteur `u32`).
- **Valeurs 4 et 16** : provisoires, définies dans `verbum-assets`, non invariants du moteur ; les tests de budget construisent leurs programmes à partir des constantes ; `ScriptState.script` limité aux slots `0..=65 535`.
- **`step` ne vérifie ni cycles ni budget statique** : le budget est appliqué à l'exécution ; la Forge garantit statiquement qu'un script accepté n'atteint jamais `Faulted(Budget)` (SR-7).
- **Écarts d'exécution tranchés applicables à U4** : **E-1** (fichiers de tests partagés `script_support/mod.rs`, types ancrés, `#[allow(dead_code, reason)]` sous condition) ; **E-2** (`Halt` n'avance pas `pc`, fin de programme `pc = n`) ; **E-7** (réductions sous Miri, SR-7 paire limite) ; **E-8** (périmètre exact de SR-7 : aucune mesure directe du compte d'instructions) ; **E-9** (M-4.8 et `static` sur copie isolée) ; **E-10** (mutations équivalentes ; M-4.10 reformulée, M-4.16 limitée au harnais).
- **Arbitrages d'U3 applicables** [Validé, utilisateur, 5 novembre 2026] : chemin `crates/extensions/script/` (AMB-1 d'U3) ; ST-1 obtient ses états par exécution réelle de `step` ; `same_capacity` fonction locale de test ; `#[allow(clippy::new_without_default)]` localisé et motivé, **aucun `Default` public** pour `Emissions` et `ScriptTable` ; raisonnement `pc + 1` sans débordement consigné au rapport d'U3.
- **Décisions d'U1, d'U2 et d'U3 sans effet de conception sur U4** (pour mémoire) : le lecteur ne contrôle pas les octets ignorés ; la Forge écrit les champs ignorés à zéro ; `check_flow` vérifie le budget sur tous les nœuds `0..n` et refuse les cycles sans yield y compris sur code inaccessible (E-11) ; SCR-C1 compte six dépôts ; M-3.28 équivalente. Aucun de ces points n'est rouvert.
- **Leçons de l'étape 5 applicables** : E-12 (durée de vie sous `Binding<'a>`, `Anchored`) ; E-13 (parsing TOML de gros volumes prohibitif sous Miri : SR-7 se découpe, paire limite sous Miri) ; E-14 (`dead_code` sous MSRV 1.85 pour les constantes utilisées seulement par une `const` assert).

## 10. Interdits

- Ne modifier ni les ADR, ni `ROADMAP.md`, ni `docs/implementation/00` à `05`, ni les handoffs, ni `crates/engine/{ecs,schedule,render,verbum}/**`, ni **`crates/engine/shell/src/**`** (y compris la documentation `//!` : aucun doctest nouveau), ni `crates/engine/assets/**`, ni `crates/engine/forge/**`, ni `crates/extensions/script/**`, ni le `Cargo.toml` racine ; ne modifier aucun test d'une étape antérieure (F14) ; ne pas modifier le plan, le document d'exécution ni ce handoff sans décision explicite de l'utilisateur.
- Écrire uniquement : `crates/engine/shell/tests/script_support/mod.rs`, `crates/engine/shell/tests/script_suspend.rs` et le `Cargo.toml` de `crates/engine/shell` (une ligne de dev-dépendance). **Aucune autre dépendance** ; aucune dépendance ajoutée à `verbum-assets` (`[dependencies]` et `[dev-dependencies]` restent vides), à `verbum-script` (seule `verbum-assets`) ni à `verbum-forge` ; `verbum-forge` ne dépend jamais de `verbum-script` ; `verbum-shell` n'est jamais une dépendance d'un autre crate.
- **G-LT** : ni `'static`, ni `Box::leak`, ni `unsafe`, ni cast d'octets, ni `transmute`, ni `include_bytes!`, ni `static`, ni changement de la propriété des octets pour contourner une question de durée de vie.
- Aucun `unsafe` ; aucun `static`, `thread_local`, `Cell`, `RefCell`, atomique, `async`, `await`, thread, `dyn`, `Rc`, `Arc` dans l'assemblage et ses systèmes (G-ST ; exception de test unique : M-4.8, copie isolée seulement).
- Aucun ordre des émissions entre instances ni ordre dense d'un stockage affirmé (G-ORD-S) ; aucune comparaison par ensemble qui efface les doublons.
- Ne pas lire l'accumulateur `EMISSIONS` hors des systèmes ; ne pas conserver d'historique d'émissions dans le `World` ; aucun `LOG_CAP`, aucun journal persistant.
- Ne pas utiliser `BufferFull` pour `Full`, ne pas ignorer un `Err(Full)`, ne pas introduire de `panic!` comme politique de saturation (seul `unreachable!` de violation de contrat).
- Aucun opérande, instruction ou catégorie hors du jeu fermé du plan § 4.D ; aucun flottant ; aucune lecture de donnée métier du `World` par un script ; aucun code de visualisation, de sauvegarde, de réseau.
- Aucune écriture directe d'un composant : `World::add_component` est `pub(crate)` ; seul chemin : `CommandBuffer::add_component` puis `World::apply`.
- Aucun trait public ; aucun test ni assertion hors liste du plan et du document d'exécution ; aucune API au-delà de celle fixée ; aucun test ni mutation d'U5 (DM-1 à DM-3, exemple, MSRV, clôture) ; aucun backend graphique.
- **Aucun commit par Claude.**

## 11. Divergence, doute, blocage

Arrêt immédiat, sans choix de la session, en cas de : contradiction ou ambiguïté du plan ou d'un ADR ; divergence entre un fait [Vérifié] (F1, F2, F5, F6, F8 à F11) et le code réel fourni ; **échec de compilation de durée de vie du montage (G-LT)** ; **`Interpreter` exigeant `Anchored` (E0477 sur son accès)** ; `Components<'a>`/`ComponentSet` ou `World` refusant un composant stocké ancré ou des `Add`/`Remove` portant une durée de vie (message exact rapporté, aucun contournement) ; besoin d'`unsafe`, d'allocation hors des tests, d'un état externe ou d'un état dans un système (**G-ST**) ; exigence d'ordre inter-instances (**G-ORD-S**) ; **M-4.14 qui fait échouer un test** ; **mutation non détectée et non établie équivalente** (dont M-4.8 si SR-2 et SR-4 sont tous deux verts), ou qui échoue sur d'autres assertions que celles annoncées sans explication par le code ; test dont le rouge ne peut pas être constaté pour la bonne raison ; besoin de modifier un fichier interdit (dont `crates/engine/shell/src/**`, `crates/engine/schedule/**`, `crates/engine/ecs/**`, `crates/extensions/script/**`) ; besoin d'une dépendance supplémentaire ou d'un trait public ; **une situation de G-CM** ; un code d'erreur de compilation épinglé ; test hors liste ; fichier nécessaire non fourni ; durée Miri jugée excessive (rapporter et arbitrer, aucun `--skip` ni `ignore` sans décision). La session fournit un constat factuel (source, texte, observation) et présente les options minimales, **sans arbitrer**.

**Ambiguïtés relevées à la rédaction, non résolues** (à exposer à l'utilisateur au démarrage, avant tout fichier ; l'utilisateur tranche).

- **AMB-1 — Notation des chemins du noyau.** Le document d'exécution écrit `crates/{ecs,schedule,render,verbum}/**` (§ 2) et, pour F1, `members` = `crates/{ecs, schedule, verbum, assets, forge, shell, render}`, tandis que ses fichiers à fournir (U4) et le dépôt réel emploient `crates/engine/{ecs,schedule,…}`. Lecture prudente : `crates/engine/…` partout (le `Cargo.toml` racine fourni en U3 le confirme). **Résolue par `tree.md` et par l'arborescence réelle** (confirmé à l'audit du 5 novembre 2026) ; conservée comme simple note de notation, sans arbitrage et sans effet sur le périmètre.
- **AMB-2 — Capacité de la table : littéral `6` ou constante `SCRIPT_SLOTS`.** Le document d'exécution écrit `ScriptTable<'a, 6>` (types retenus) et définit `SCRIPT_SLOTS = 6` (constantes de scène, table de formats `[Format::Script; 6]`). Il impose par ailleurs « aucun littéral `4, 4` » pour les `SparseSet` et que chaque constante soit employée dans du code compilé (E-14). Lecture prudente : `ScriptTable<'a, SCRIPT_SLOTS>` et `[Format::Script; SCRIPT_SLOTS]`. **À confirmer par l'utilisateur.**
- **AMB-3 — `Default` et `R: Default`.** `World` et `run_tick` exigent `R: Default` pour le type des Ressources (F8, F10), alors que `ScriptTable` et `Emissions` n'ont **aucun `Default` public** (U3, AMB-4 tranché). Si `Rs<'a>` est un enregistrement de `ResourceSlot<…>` (sans contrainte sur `T`, `Default`, F10), le problème ne se pose pas ; sinon un `impl Default for Rs<'a>` écrit dans l'assemblage est nécessaire. Dans tous les cas, **`verbum-script` n'est pas modifié** (unité validée). La forme exacte de `Rs<'a>` est à établir sur `resource.rs`, `world.rs` et `level_loaded.rs` ; tout besoin d'API supplémentaire dans `verbum-script` est un arrêt. **À confirmer par l'utilisateur.**
- **AMB-4 — Liste des fichiers à fournir.** Le document d'exécution fixe pour U4 : `assets/src/{script,format,lib}.rs`, `forge/src/{lib,batch,script}.rs`, `extensions/script/**`, `Cargo.toml` racine, `shell/Cargo.toml`, `shell/tests/{level_loaded,borrowed_resource}.rs`, `schedule/src/{system,access,data,contract,model,scheduler}.rs`, `schedule/examples/headless.rs`, `ecs/src/{lib,world,component_set,command,resource,values,storage}.rs`. Ce handoff y ajoute, en lecture seule, `schedule/src/order.rs` (ordre déduit et `OwnershipError`, lu lors de l'audit du 5 novembre 2026), `assets/src/{load,manifest}.rs` (signature de `load`, `SlotIndex`), `forge/src/sprite.rs` (`ForgeError` et ses variantes de script, nécessaires à SR-7), `schedule/tests/support/mod.rs` (précédent d'assemblage partagé) et un `tree.md` régénéré, et exige les rapports d'U1, d'U2 et d'U3. Précision de handoff, non modification d'exigence ; **à confirmer**.
- **AMB-5 — Mentions datées du document d'exécution.** L'en-tête (« aucune unité ne démarre avant … U1 », « les handoffs d'U2 à U5 ne sont pas produits ») et le F1 du § 3.1 (sept membres, aucune entrée `verbum-script`) décrivent l'état à la rédaction ; l'état réel est celui du § 2 (U1 à U3 validées ; huit membres ; entrée `verbum-script` dans `[workspace.dependencies]`). Sans effet sur U4 ; aucune correction n'est faite sans décision de l'utilisateur.
- **AMB-6 — Instanciation du test de contrat.** Le plan (§ 7.4, CT-1) écrit `contract_of::<B<'_>, Interpreter>()` ; le document d'exécution écrit `contract_of::<B<'static>, Interpreter>()` et précise que cette instanciation ne vaut que pour le test de contrat (le contrat est indépendant de `'a`). Selon la hiérarchie du § 4, **le plan prévaut** en cas de divergence. Les deux formes expriment le même contrat ; la forme `B<'static>` peut être conservée pour le seul CT-1 **si elle compile**. Point à établir à la compilation : si `B<'static>` ne compile pas pour `Interpreter`, c'est un arrêt (G-LT), sans variante de substitution ; la forme `B<'_>` du plan est alors la référence. **À confirmer par l'utilisateur.**

**Points à confirmer par l'implémentation** (non résolus à la rédaction) : (a) visibilité et nom du type d'erreur de `Scheduler::new()` (`ModelError`, défini dans `model.rs`, non lu à la rédaction ; `OwnershipError::MultipleWriters` existe dans `order.rs` : reste à établir comment `ModelError` l'expose) : sinon `is_err()` et message `Debug` rapportés (CT-2 ; § 10 point 1 du document d'exécution) ; (b) forme exacte de l'observation de l'absence du composant avant `apply` (`components().states.get(entité)` ou `contains`), de l'API de `CommandBuffer` et de la lecture d'un composant depuis l'assemblage (§ 10 point 8) ; (c) forme de `Components<'a>` et de son `ComponentSet` avec un stockage ancré (§ 10 point 9) ; (d) forme de `Rs<'a>` (AMB-3) ; (e) hypothèse « `Interpreter` compile sans `Anchored` » (§ 10 point 7) ; (f) compilation sous MSRV 1.85 de l'`Interpreter`, de `ScriptTable<'a, 6>` dans le montage et du `ComponentSet` ancré (§ 10 points 6 et 9) ; (g) durée de Miri pour SR-7 (paire limite) et SR-2 (13 reconstructions) : non mesurée (§ 10 point 5) ; (h) noms, layouts et signatures provisoires [L] : tout ajustement listé au rapport sans changement de sémantique.

## 12. Rapport de fin d'unité et reprise

**Gabarit** (rien de narratif ; document d'exécution § 5) : 1. Unité, état (livrée, en attente de validation). 2. Fichiers créés et modifiés (chemins complets), fichiers livrés par `SendUserFile`. 3. Faits [Vérifié] revérifiés : confirmés ou divergence (arrêt). 4. Rouge : test, motif d'échec observé, ou « sans rouge possible », motif (CT-1 à CT-3). 5. Quatre contrôles : lignes réelles, **espace de la session** et **dépôt réel** séparés ; contrôles non exécutés nommés. 6. Mutations : mutation, détecteur, autres assertions échouées, **mode de détection observé** (assertion, panique, erreur de construction, échec de compilation pour M-4.12) ; M-4.14 en invariance ; M-4.15 équivalente (justification) ; M-4.16 harnais ; le premier échec comparé au détecteur annoncé (§ 8, précisions b et c). 7. Portes : G-CM (aucun arrêt, ou arrêt rapporté), **G-LT (message du compilateur si échec)**, G-ORD-S, G-ST ; liste R-ADAPT. 8. Garanties : statiques / revue (sans mélange), **revue G-ST point par point**, revue `EMISSIONS`, revue des fichiers non modifiés. 9. Contrôles natifs, Miri (intégral ou partiel, avec la liste des réductions ; commande : `cargo +nightly miri test -p verbum-shell`), MSRV (non exécuté en U4 ; signalé). 10. Ajustements de noms et écarts du plan ou du document d'exécution, réponses aux AMB-2 à AMB-6 telles que tranchées par l'utilisateur (AMB-1 : note de notation). 11. Regroupement de commit proposé par invariant ; `tree.md` à régénérer (fichiers de tests créés). 12. Fin : « En attente de la validation de l'unité U4. »

**Résultats séparés** : tout résultat de l'espace de travail est étiqueté « espace de travail » ; seuls les résultats rapportés par l'utilisateur sont étiquetés « dépôt réel ».

**Reprise** : si la session se dégrade, une nouvelle session reçoit ce handoff, le document d'exécution, le plan, les documents du § 3, un `tree.md` à jour et les fichiers du § 7 ; elle relit l'état réel par les contrôles **exécutés par l'utilisateur** ; une unité interrompue est refaite depuis ses tests ; elle ne rouvre aucun arbitrage clos ni aucune unité validée. Après validation d'U4 (« unité U4 validée »), l'utilisateur demande le handoff d'U5 ; aucune session ne traite deux unités.
