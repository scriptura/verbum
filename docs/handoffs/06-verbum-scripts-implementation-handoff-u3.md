# Handoff d'implémentation — étape 6 (scripts), unité U3 : crate `verbum-script`

Document de passation, non normatif, autonome pour U3. Il est destiné à une nouvelle session Claude dédiée **exclusivement à l'implémentation de l'unité U3** de l'étape 6 de Verbum. Il ne dépend d'aucune mémoire des sessions précédentes (U1 et U2). **Il ne vaut pas autorisation de démarrer U3.** Les unités U4 et U5 ne sont pas abordées dans cette session ; chacune aura sa propre session et, sur demande, son propre handoff.

Structure alignée sur `06-verbum-scripts-implementation-handoff.md` (handoff U1, § 1 à 13). Le périmètre, les tests, les mutations, les contrôles et les portes d'arrêt d'U3 sont **repris sans modification** de `06-verbum-scripts-execution.md` (§ 3.3 et U3 du § 4) ; ce handoff n'ajoute aucun invariant et ne modifie aucune exigence.

Légende : **[Validé]** décision arrêtée par l'utilisateur · **[Vérifié]** fait constaté dans un document ou un source lu, à revérifier au démarrage · **[À confirmer]** point que l'implémentation doit établir ou signaler · **[L]** choix local provisoire · **[Ambiguïté]** divergence ou lacune relevée à la rédaction, **non résolue** (§ 11). **Aucun résultat de compilation, de test, de Miri ou de mutation n'existe pour U3** : tout résultat mentionné ici est une attente, jamais un constat.

## 1. Rôle

Vous êtes implémenteur Rust. Vous exécutez de façon disciplinée l'unité U3 du plan et du document d'exécution de l'étape 6. Vous ne révisez pas l'architecture spontanément, vous ne refaites pas l'audit et vous ne rouvrez pas les arbitrages clos (§ 9). Le projet est Verbum (moteur 2D Rust, ECS/DOD/AOT). Vous travaillez avec l'utilisateur, qui raisonne en architecte système ECS/DOD/AOT sans avoir encore tout le vocabulaire du domaine. Registre : vouvoiement, réponses concises, structurées, opérationnelles, sans généralités ni métaphores ; un concept technique avancé est défini brièvement et précisément à sa première apparition.

## 2. Contexte d'implémentation

1. **Vous n'avez pas accès au dépôt local de l'utilisateur.** Vous travaillez dans un conteneur distant. Ce que vous savez du code vient exclusivement des fichiers que l'utilisateur vous fournit. Ne présumez jamais le contenu d'un fichier non fourni ; ne demandez pas le dépôt en bloc ; demandez nommément ce qui manque.
2. **Livraison : fichiers complets, jamais de diff.** Chaque fichier `.rs` créé est livré **complet** par `SendUserFile`, première ligne = commentaire de chemin suivi d'une ligne vide. Documentation des sources `.rs` **en français**. Les `Cargo.toml` sont exemptés du commentaire de chemin ; le champ `description` du `Cargo.toml` créé est en anglais. **Le `Cargo.toml` racine est livré complet à partir du fichier réel fourni par l'utilisateur, avec les deux seuls ajouts de U3 (§ 7) : jamais à partir d'un `Cargo.toml` d'espace de travail réduit reconstitué par la session** (le dépôt réel compte sept membres ; un espace réduit en compte moins). `Cargo.lock` : jamais livré, mis à jour par Cargo chez l'utilisateur.
3. **Contrôles réels exécutés par l'utilisateur** : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, plus les contrôles propres à U3 (§ 8) et Miri. Si une chaîne Rust est disponible dans votre conteneur, vous pouvez reconstituer un espace de travail réduit à partir des fichiers fournis (pour U3 : `verbum-assets` et le nouveau crate suffisent ; `verbum-render` et `verbum-ecs` ne sont que des modèles de lecture). Vous **distinguez toujours** « résultat dans l'espace de la session » et « résultat du dépôt réel », vous listez tout écart de l'espace réduit (membres absents, version résolue d'une dépendance, stub éventuel) et vous ne présentez jamais le premier comme le second. Aucun résultat n'est déduit, estimé ni rapporté sans avoir été produit. Lors des unités précédentes, la chaîne `nightly` n'était pas installée dans l'espace de session : Miri est alors exécuté par l'utilisateur seulement, sur la commande exacte que vous fournissez.
4. **Pas de commit par la session.** Vous proposez en fin d'unité un regroupement par invariant démontré. `tree.md` est régénéré par l'utilisateur : vous signalez les fichiers créés (`crates/extensions/script/Cargo.toml` et `crates/extensions/script/src/{lib,state,emissions,table,step}.rs`) sans y toucher.
5. **Seul l'utilisateur déclare l'unité validée et l'étape close.**
6. **Clause d'échappement** : en cas de problème, d'ambiguïté ou d'impossibilité, vous **vous arrêtez et exposez** (constat factuel : source, texte, observation ; options minimales) au lieu de contourner. Les cas d'arrêt sont au § 11.
7. **Historique à connaître.** U1 (lecteur `script.bin`, `verbum-assets`) et U2 (compilation `script.toml`, `verbum-forge`) sont **validées par l'utilisateur** (4 et 5 novembre 2026), contrôles du dépôt réel verts, dont Miri intégral sur `verbum-assets` et Miri partiel sur `verbum-forge`. U2 a exceptionnellement été traitée dans la session d'U1 (exception ponctuelle) ; **cette exception est close : U3 a sa session propre**. Aucune session ne traite deux unités.

## 3. Documents de référence et ordre de lecture

À fournir par l'utilisateur au démarrage :
1. `docs/handoffs/06-verbum-scripts-implementation-handoff-u3.md` (ce document).
2. `docs/implementation/06-verbum-scripts-execution.md` : règles de session (§ 1), interdits (§ 2), faits vérifiés (§ 3, dont **§ 3.3 en entier** : valeurs provisoires, API de `verbum-script`, codes de l'encodage, précision sur `pc`, scénario et trajectoire de référence), **unité U3 du § 4**, gabarit de rapport (§ 5), arrêt et reprise (§ 6), écarts E-2, E-4, E-5, E-6, E-10 (§ 9), points non vérifiés n° 2, 6 et 10 (§ 10).
3. `docs/implementation/06-verbum-scripts.md` : plan **r1.2** (à lire une fois ; puis seulement les sections utiles à U3 : § 4.C, § 4.D, § 4.E, § 4.F (crate d'exécution, borne des émissions), § 4.G, § 5, § 6, § 7.3, § 7.6, § 8 ligne U3, § 9).
4. `docs/architecture/adr/ADR-Sc01-scripts.md` et `ADR-000-language-and-material-invariants.md` (points 1, 3 et 4).
5. `tree.md` : fait foi sur les chemins du dépôt. **Le `tree.md` disponible lors de la rédaction (généré le 4 novembre 2026, 08:51) précède U1 et U2** : il ne liste ni `assets/src/script.rs` ni `forge/src/script.rs`. L'utilisateur fournit une version régénérée, ou le confirme.
6. Le **rapport de fin d'unité d'U2 (validé)**, avec son complément sur M-2.16b, et celui d'U1 avec ses deux compléments (mention SCR-R5 ; modes de détection des mutations), pour l'état réel et les écarts consignés.

Non requis pour U3, à ne demander que si un document requis les cite : `ADR-A01-assets.md`, `05-verbum-assets-execution.md` (G-CM est sans objet en U3 ; la leçon E-14 est reprise au § 8), les autres ADR, `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`. Le plan cite ADR-001 (point 5, capacité fixe) et ADR-003 (point 5) : demandez-les nommément seulement si une décision d'U3 en dépend.

## 4. Hiérarchie normative

1. Les ADR et le corpus gouvernent.
2. Le plan `06-verbum-scripts.md` r1.2 (validé, inchangé) définit les exigences de l'étape.
3. Le document d'exécution organise les sessions et ne contient aucune règle de conception.
4. Ce handoff ne contient aucune règle de conception.

Divergence plan / exécution : le plan prévaut ; plan / ADR : l'ADR prévaut ; exécution / handoff : le document d'exécution prévaut. Dans tous les cas, la divergence est signalée à l'utilisateur et jamais résolue par la session.

## 5. Statut des documents à la passation

- **Plan `06-verbum-scripts.md` r1.2** (1er novembre 2026) : **validé** ; arbitrages A-S1 à A-S5 arrêtés ; **inchangé**.
- **`06-verbum-scripts-execution.md`** et **handoff d'U1** : **validés** par l'utilisateur (déclaration du 4 novembre 2026 : les mentions « soumis à validation » de leurs en-têtes sont un oubli éditorial). Le point d'identification de la révision (« 3 novembre au soir » pour l'utilisateur ; « 3 novembre (nuit) » dans l'en-tête du fichier) est signalé en AMB-7.
- **Ce handoff** : soumis à la validation de l'utilisateur au moment de sa rédaction. Tant que la validation n'est pas communiquée à la session, elle l'indique en une ligne et attend.
- Si l'utilisateur demande une correction ultérieure, appliquez **uniquement** cette correction, rendez le fichier **complet** (pas de diff), ne modifiez rien d'autre, attendez la validation.

## 6. Interdiction de coder avant instruction

**Aucune ligne de code, aucun fichier, aucune commande de build ou de test, avant que l'utilisateur ait écrit : « démarrez l'unité U3 ».** À la lecture initiale, accusez réception en cinq lignes maximum, en indiquant le statut des documents tel qu'au § 5.

## 7. Périmètre et découpage

**Une session Claude par unité** [Validé, utilisateur, 3 novembre 2026] : U1, U2, U3, U4, U5 dans des sessions distinctes. **Cette session traite U3 seulement.** À la fin d'U3 : rapport selon le gabarit du § 12, puis arrêt avec « En attente de la validation de l'unité U3. ». Elle ne prépare ni ne propose U4.

| Unité | Contenu | Cette session |
|---|---|---|
| U1 | lecteur `script.bin` dans `verbum-assets` | validée |
| U2 | compilation `script.toml` → `script.bin` dans `verbum-forge`, `check_flow` | validée |
| **U3** | crate `verbum-script` : état, émissions, table, `step` ; racine : `members` et `[workspace.dependencies]` | **oui, seule** |
| U4 | intégration Scheduler, interface fixe, reprise | non |
| U5 | démonstrateur, clôture | non |

**U3 est indépendante** : aucune anticipation d'U4 ou d'U5 (aucun système, aucun `Binding`, aucun `ComponentSet`, aucun `Anchored`, aucun assemblage, aucun test dans `crates/engine/shell`) ; **aucune modification du noyau ECS, du Scheduler, de `verbum-render`, de `verbum-assets` (inchangé depuis U1), de `verbum-forge` (inchangé depuis U2) ni du Shell**.

**Contenu d'U3** (document d'exécution, U3, repris). Créer le crate d'exécution pur : encodage canonique, états valides par construction, accumulateur d'émissions, table indexée par slot, borne des émissions, fonction `step`. Aucune dépendance au `World` ni au Scheduler. Invariants visés : X3 à X6, X8, X9, X12 à X14 ; T4 à T6, T9, T10, T14 ; plan § 4.C, § 4.D, § 4.E.

**Fichiers à créer ou modifier** [Exécution]. Créés : `Cargo.toml` du crate (package `verbum-script`, champs hérités du workspace, `description` anglaise du type « Script runtime of the Verbum engine: execution state and pure interpreter step, no_std and allocation-free. », `[dependencies]` : `verbum-assets.workspace = true`, **aucune `[dev-dependencies]`**, `[lints] workspace = true`), `src/{lib,state,emissions,table,step}.rs`. Modifié : `Cargo.toml` racine (`"crates/extensions/script"` ajouté à `members` après `crates/engine/render` ; `verbum-script = { path = "crates/extensions/script", version = "0.1.0" }` ajouté à `[workspace.dependencies]` ; **rien d'autre**). **Emplacement du crate** [Validé, utilisateur, 5 novembre 2026] : répertoire `crates/extensions/script/`, **en dehors de `crates/engine/`** (séparation intentionnelle : `crates/engine/` contient le moteur, `crates/extensions/script/` accueille le crate de scripts distinct) ; package `verbum-script` (le nom ne mentionne pas « extensions ») ; entrée de `members` : `"crates/extensions/script"`, insérée après `crates/engine/render` ; entrée de `[workspace.dependencies]` : `path = "crates/extensions/script"`. Les chemins `crates/script` du plan et du document d'exécution se lisent donc `crates/extensions/script`. Le commentaire de première ligne de chaque source est de la forme `// crates/extensions/script/src/lib.rs`.

**Fichiers à fournir (U3)**, versions courantes du dépôt réel :

| Fichier | Usage |
|---|---|
| `Cargo.toml` (racine) | **modifié** (deux ajouts) ; revérification de F1 |
| `crates/engine/render/Cargo.toml` | lecture seule : modèle de manifeste (`no_std`, `[dependencies]` vide, `[lints] workspace = true`, hérités du workspace) |
| `crates/engine/render/src/lib.rs`, `src/accumulator.rs` | lecture seule : modèle de l'accumulateur à capacité fixe (`Full`, `new` const, `push`, `as_slice`, `len`, `is_empty`, `capacity`, `clear`, `PartialEq` manuel sur la tranche, `Debug` manuel) ; **déjà fournis lors de la rédaction, à fournir à nouveau** |
| `crates/engine/ecs/src/entity.rs` | lecture seule : motif de garde statique par constante associée (`CAPACITY_FITS_INDEX`) ; **déjà fourni, à fournir à nouveau** |
| `crates/engine/assets/src/script.rs`, `src/lib.rs` | lecture seule, **tels que rendus en U1** (validés) : `Script`, `Instruction`, constantes `SCRIPT_*` |
| `crates/engine/assets/src/manifest.rs` | lecture seule : définition de `SlotIndex` (`new(u32)`, `get() -> u32`, `Clone, Copy, Debug, PartialEq, Eq`), nécessaire à la revérification de F6. **Absent de la liste minimale du document d'exécution : ajout de ce handoff, signalé en AMB-8** |
| Sources d'U2 : `crates/engine/forge/src/{script,lib}.rs` | **facultatif**, lecture seule : sémantique de `check_flow` et de l'encodage côté Forge (U3 ne dépend pas de la Forge, aucun dev-dependency). `batch.rs`, `sprite.rs`, `tests/pipeline.rs` d'U2 et `format.rs`, `sprite.rs` d'U1 ne sont **pas** nécessaires à U3 ; à fournir seulement sur demande nommée |

Si l'utilisateur fournit un fichier non listé, la session le lit mais ne le modifie pas.

## 8. Méthode

- **Rouge avant vert** quand c'est possible : tests écrits contre des squelettes (`unimplemented!()` : `ScriptState::{start, from_bytes, to_bytes, …}`, `Emissions`, `ScriptTable`, `emissions_bound_holds`, `step`), échec constaté par **panique** pour la raison attendue et rapporté. **Un échec de compilation n'est pas un rouge valide.** Sans rouge possible : néant, selon le document d'exécution (U3).
- **Sensibilité par mutation** sur copie isolée **hors de l'arbre de travail**, une mutation à la fois, jamais présentée comme rouge initial ; le rapport nomme pour chaque mutation le test détecteur **et les autres assertions échouées**, avec le **mode de détection observé** (échec d'assertion, erreur de décodage, panique ; rien de supposé) [Validé, utilisateur, 4 novembre 2026]. **Une mutation non détectée est un arrêt, sauf si elle est établie équivalente** (comportement observable inchangé dans le domaine atteignable) : elle est alors rapportée comme telle avec sa justification, **sans test artificiel**. Aucune détection par échec de compilation n'est admise.
- **Précisions issues de la validation d'U1 et d'U2** [Validé, utilisateur] : (a) lorsqu'une assertion est ajoutée **après** le rouge pour détecter une mutation, le rapport le mentionne explicitement (précédent : SCR-R5, mutation M-1.15) ; (b) lorsque plusieurs sous-cas d'un même test peuvent détecter une mutation et que le **premier échec observé n'est pas le détecteur annoncé**, le sous-cas annoncé est exécuté **indépendamment**, sur une copie isolée, en le plaçant en tête dans un harnais temporaire de mutation, sans test permanent supplémentaire ni modification du code de production ; le rapport donne les deux résultats (précédent : M-2.16b, détecteur prévu k2, premier échec k1) ; (c) un écart entre détecteur annoncé et échecs observés est expliqué au rapport, par le code et non par supposition.
- **Débordement arithmétique** : M-3.20 (`Add` non enveloppant) est détectée par la panique de débordement du profil de test ; les tests s'exécutent en profil `dev` (défaut de `cargo test`), pas en `--release`.
- **Portes** : G-ST et G-ORD-S sont applicables à U3 ; G-CM et G-LT sont **sans objet** (U3 ne touche ni `load`, ni `verbum-assets`, ni un montage `World`/`Scheduler`). Le rapport l'indique, sans l'affirmer sans l'avoir vérifié (aucun fichier d'`assets` ni de `forge` modifié).
- **R-ADAPT** : aucun fichier existant n'est modifié en U3 hors le `Cargo.toml` racine ; une adaptation mécanique hors des fichiers listés est exposée avant d'être faite et consignée.
- **Contrôles propres à U3** (document d'exécution, U3, repris) : `cargo tree -p verbum-script -e normal` (seule dépendance : `verbum-assets`) ; `cargo tree -p verbum-script` (idem, aucune dev) ; `cargo tree -p verbum-assets` (vide) ; `cargo tree -i verbum-script --workspace` (aucun dépendant à ce stade) ; **revue G-ST** : recherche textuelle dans le crate de `static`, `thread_local`, `Cell`, `RefCell`, `Atomic`, `async`, `await`, `thread`, `Box`, `Vec`, `unsafe`, `dyn`, résultat rapporté (aucune occurrence admise hors tests pour les littéraux de tableaux) ; revue : `step` n'a aucun paramètre autre que `(script, état, sink)` ; revue : `ScriptState` sans référence. **Revue de conformité de la signature publique et de l'API** (anciennement M-3.29), point par point, rapportée : (a) `ScriptState::registers()` rend `[i32; SCRIPT_REGISTERS]` **par copie**, conformément au § 3.3 ; (b) les accesseurs `script()`, `pc()`, `status()`, `wait()` rendent des valeurs `Copy` par copie ; (c) aucun accesseur de `ScriptState` ne rend de référence ni de `&mut` sur l'état interne (champs privés, transition unique par `step`) ; (d) `ScriptTable::get` rend `Option<&Script<'a>>` conformément au § 3.3 ; (e) la liste des signatures publiques de `verbum-script` est comparée à celle du § 3.3, tout écart étant listé au rapport (règle 9 du § 1 du document d'exécution). La revue n'est jamais présentée comme une preuve statique.
- **Natif** : `cargo test -p verbum-script`. **Miri (utilisateur)** : `cargo +nightly miri test -p verbum-script` (suite complète, aucune réduction prévue : tous les tests emploient des tableaux de quelques centaines d'octets ; si la durée dépasse 10 minutes, rapporter et arbitrer, ne pas désactiver l'isolation, aucun `--skip` ni `ignore` sans décision).
- **Compatibilité MSRV 1.85 dès l'écriture** : pas de let-chains (`let … else` à la place) ; `is_empty` accompagne tout `len` public (`Emissions`) ; **aucune constante utilisée seulement dans un `const _: () = assert!(…)`** (avertissement `dead_code` sous 1.85, leçon E-14 de l'étape 5) : chaque constante est aussi utilisée dans du code compilé (la constante associée de garde de `ScriptTable` est référencée dans `new`, motif de `entity.rs`) ; aucune assertion sur l'horloge. La vérification MSRV est faite en U5 ; la compilation sous 1.85 de `ScriptTable<'a, CAP>` (tableau `[None; CAP]` d'`Option<Script<'a>>`, garde statique) et de `emissions_bound_holds` (`checked_mul` en `const fn`) est un point non vérifié (§ 10 point 6 du document d'exécution) : signalé au rapport s'il n'est pas établi.
- **Tests dans un crate `no_std`** : aucune dépendance de développement n'est admise ; les binaires de script sont des **tableaux d'octets de taille constante écrits à la main ou construits par code, sans `Vec`**, et lus par `Script::read` (seul constructeur de `Script`, champs privés). Un binaire doit donc être **valide pour le lecteur d'U1** : `n` sur `u16`, longueur déclarée `2 + 8·n`, opcodes connus, registre `< SCRIPT_REGISTERS`, cible `< n`, `Wait` de ticks `≥ 1`. Le lecteur ne vérifie ni cycle ni budget : un binaire rectiligne de `SCRIPT_MAX_STEPS_PER_TICK + 1` `Add` est lisible.
- Chaque unité commence par **revérifier** les faits [Vérifié] qu'elle utilise sur les fichiers fournis : F1 (`Cargo.toml` racine), F2 (manifeste de `verbum-render`), F4 (`Script`, `Instruction`, constantes, tels que rendus en U1), F6 (`SlotIndex`), plus la lecture de `render/src/accumulator.rs` et de `ecs/src/entity.rs` (§ 10, point 2 du document d'exécution : « non lus » à la rédaction du document d'exécution).

**État réel d'U1, utile à U3** [Vérifié lors de la rédaction, sur les fichiers livrés d'U1 ; à revérifier]. `verbum-assets` exporte publiquement : `Script<'a>` (`Clone, Copy, Debug, PartialEq, Eq`) avec `read(&'a [u8]) -> Result<Self, ReadError>`, `len(&self) -> u16`, `is_empty(&self) -> bool`, `instruction(&self, index: u16) -> Option<Instruction>` ; `Instruction` (`Clone, Copy, Debug, PartialEq, Eq`) ; les constantes `SCRIPT_FORMAT_ID`, `SCRIPT_FORMAT_VERSION`, `SCRIPT_FIXED_PAYLOAD_LEN`, `SCRIPT_INSTRUCTION_LEN`, `SCRIPT_REGISTERS: usize = 4`, `SCRIPT_MAX_STEPS_PER_TICK: usize = 16` ; `SlotIndex` (`new(u32)`, `get() -> u32` const) ; `ReadError` et `Asset`/`Format` (non employés par U3). `verbum-script` **réexporte** `SCRIPT_REGISTERS` et `SCRIPT_MAX_STEPS_PER_TICK` ; U3 ne définit aucune de ces constantes.

**Spécification d'U3 reprise du document d'exécution** (§ 3.3 et U3) :

*API de `verbum-script`* [L] : constantes `SCRIPT_REGISTERS`, `SCRIPT_MAX_STEPS_PER_TICK` réexportées ; `ENCODED_LEN = 10 + 4·SCRIPT_REGISTERS` ; `ScriptState` (`Clone, Copy, Debug, PartialEq, Eq`, **champs privés**) avec `start(SlotIndex) -> Result<Self, SlotTooLarge>`, accesseurs `script() -> u16`, `pc() -> u16`, `status() -> Status`, `wait() -> u32`, `registers() -> [i32; SCRIPT_REGISTERS]` (**retour par copie**), `to_bytes() -> [u8; ENCODED_LEN]`, `from_bytes(&[u8; ENCODED_LEN]) -> Result<Self, StateError>` ; `Status` ∈ {`Running`, `Waiting`, `Halted`, `Faulted(Fault)`}, `Fault` ∈ {`NoScript`, `Budget`} ; `Emission { channel: u16, value: i32 }` (`Copy`) ; `Emissions<const CAP: usize>` (`new`, `push -> Result<(), Full>`, `as_slice`, `len`, `is_empty`, `capacity`, `clear` ; `Clone`, `Debug`, `PartialEq` manuel sur la tranche observable) ; `Full` ; `ScriptTable<'a, const CAP: usize>` (`new`, `deposit(SlotIndex, Script<'a>) -> Result<(), TableError>`, `get(u16) -> Option<&Script<'a>>` ; `TableError` ∈ {`OutOfRange`, `Occupied`} ; garde statique `CAP ≤ 65 536` par constante associée, motif de `entity.rs`) ; `emissions_bound_holds(instances: usize, cap: usize) -> bool` (`const fn`, `checked_mul`) ; `step<'a, const CAP: usize>(script: Option<&Script<'a>>, state: &mut ScriptState, sink: &mut Emissions<CAP>) -> Result<(), Full>`. Aucun trait public de chargement ni d'exécution.

*Codes de l'encodage d'état* [L, E-4] : `status` : 0 `Running`, 1 `Waiting`, 2 `Halted`, 3 `Faulted` ; `fault` : 0 aucune, 1 `NoScript`, 2 `Budget`. Layout : `script` `u16` (octets 0 à 1), `pc` `u16` (2 à 3), `status` (4), `fault` (5), `wait` `u32` (6 à 9), registres `i32` (10 à 25), tout en little-endian, champ par champ, sans remplissage ni cast.

*Table de validité* (plan § 4.C, appliquée par `from_bytes`) : `Running` (fault 0, wait 0), `Waiting` (fault 0, wait ≥ 1), `Halted` (fault 0, wait 0), `Faulted(NoScript)` (fault 1, wait 0), `Faulted(Budget)` (fault 2, wait 0) ; toute autre combinaison est un `StateError`. `pc` et `script` ne sont pas contraints par l'encodage.

*Contenu* : (1) `state.rs` : `ScriptState`, `Status`, `Fault`, `SlotTooLarge`, `StateError`, `ENCODED_LEN`, `to_bytes`, `from_bytes`. (2) `emissions.rs` : `Emission`, `Emissions<CAP>`, `Full`, `emissions_bound_holds`. (3) `table.rs` : `ScriptTable<'a, CAP>`, `TableError`, garde statique `CAP ≤ 65 536`. (4) `step.rs` : `step` selon le plan § 4.D, avec la définition de budget **sans décalage** (recherche de l'instruction, puis test du budget, puis exécution) et la précision de `pc` ci-dessous ; `Wait` décrément saturant ; arithmétique `wrapping_add` ; registre invalide inatteignable (`unreachable!` avec message, jamais de panique sur artefact chargé). (5) `lib.rs` : `#![no_std]`, modules, exports, réexport des constantes de `verbum-assets`.

*`step`* (plan § 4.D) : (1) `Halted` ou `Faulted` : aucun changement. (2) `Waiting` : `wait -= 1` saturant ; si `wait == 0`, statut `Running` et l'exécution continue **dans ce même tick** ; sinon fin du tick. (3) `Running` : script absent → `Faulted(NoScript)` ; sinon boucle : **chercher** l'instruction à `pc` ; absente → `Halted` implicite ; **si `executed == SCRIPT_MAX_STEPS_PER_TICK` → `Faulted(Budget)`** ; sinon exécuter, `executed += 1` ; `Wait` et `Halt` terminent le tour. (4) `Emit` sur sink plein : `Err(Full)` sans avancer `pc`. Effets : `Halt` → `Halted` ; `Emit` pousse et `pc += 1` ; `Wait` : `wait = ticks`, `Waiting`, `pc += 1`, fin de tour ; `Set` ; `Add` : `wrapping_add` ; `Jump` : `pc = cible` ; `JumpIfNonZero` : `pc = cible` si registre ≠ 0, sinon `pc += 1`. **Précision sur `pc`** [Exécution, E-2] : `Halt` **n'avance pas** `pc` ; une fin de programme implicite laisse `pc = n`.

*Scénario de référence* (utilisé par ST-1 et ST-4 ; artefact de 68 octets, littéral écrit à la main, indépendant des constantes) :

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

Trajectoire de référence (instance lancée au tick 1, script au slot 4) : tick 1 : `Set`, `Emit (1,10)`, `Wait 3` ; ticks 2 et 3 : attente ; tick 4 : reprise, `Add` (r0 = 2), `JumpIfNonZero` pris, `Emit (1,10)`, `Wait 3` ; ticks 5 et 6 : attente ; tick 7 : même tour (r0 = 1) ; ticks 8 et 9 : attente ; tick 10 : reprise, `Add` (r0 = 0), `JumpIfNonZero` non pris, `Emit (2,99)`, `Halt` ; ticks 11 et 12 : aucun changement. Encodage canonique en fin de tick, 26 octets chacun (les 12 derniers octets sont nuls pour `r1` à `r3`, `r0` figure aux octets 10 à 13) :

| Fin de tick | Octets 0 à 13 |
|---|---|
| 0 (état initial) | `04 00 00 00 00 00 00 00 00 00 00 00 00 00` |
| 1 | `04 00 03 00 01 00 03 00 00 00 03 00 00 00` |
| 2 | `04 00 03 00 01 00 02 00 00 00 03 00 00 00` |
| 3 | `04 00 03 00 01 00 01 00 00 00 03 00 00 00` |
| 4 | `04 00 03 00 01 00 03 00 00 00 02 00 00 00` |
| 10 (et 11, 12) | `04 00 06 00 02 00 00 00 00 00 00 00 00 00` |

**Tests d'U3** (littéraux écrits à la main ; document d'exécution, U3, repris sans modification).

| Test | Énoncé | Porte d'arrêt |
|---|---|---|
| ST-1 | `ENCODED_LEN == 10 + 4·SCRIPT_REGISTERS == 26` (épinglé) ; `to_bytes` des états de la trajectoire de référence (fins de tick 0, 1, 2, 3, 4, 10) égal aux littéraux, 26 octets chacun | oui (T4) |
| ST-2 | `from_bytes` : les cinq combinaisons valides acceptées (dont `Waiting` avec `wait = 1` et `u32::MAX`) ; **chaque combinaison impossible rejetée** : `Waiting` avec `wait = 0` ; `wait ≠ 0` pour `Running`, `Halted`, `Faulted` ; `fault ≠ 0` pour `Running`, `Waiting`, `Halted` ; `Faulted` avec `fault = 0` et `fault = 3` ; octet de statut `4` et `255` ; faute inconnue `255` | oui (T14) |
| ST-3 | Bijection : `from_bytes ∘ to_bytes = id` sur un échantillon couvrant les cinq états valides et des valeurs de `pc`, `script`, registres aux bornes ; `to_bytes ∘ from_bytes = id` sur les octets valides ; `ScriptState` est `Copy` et `'static` (fonction générique à bornes) | oui (T4) |
| ST-4 | **Tout état atteint par `step`** sur les trajectoires (scénario de référence, budget, script absent, `Halt`, fin de programme) satisfait `from_bytes(to_bytes(s)) == s` ; `Waiting` n'est jamais observé avec `wait = 0` ; `ScriptState::start(SlotIndex::new(65 535))` accepté (`script() == 65 535`), `65 536` et `u32::MAX` : `SlotTooLarge` ; `start(SlotIndex::new(4))` : `Running`, `pc = 0`, registres nuls | oui (T14, C12) |
| IN-1 | `Emit` : pousse (canal, valeur) dans l'ordre, avance `pc` ; trois `Emit` consécutifs produisent trois émissions dans l'ordre | |
| IN-2 | `Set`, `Add` : effet exact ; `Add` enveloppant (`i32::MAX + 1`, `i32::MIN − 1`) sans panique | |
| IN-3 | `Jump` ; `JumpIfNonZero` pris (registre non nul) et non pris (registre nul), sur le registre 3 (dernier) également | |
| IN-4 | `Wait n`, *n* = 1, 2, 3 : reprise au tick *t + n* exactement (le tour de reprise exécute l'instruction suivante) ; valeurs de `wait` en fin de tick *n*, *n − 1*, …, 1 ; `Wait` termine le tour (l'instruction suivante n'est pas exécutée au même tick) | oui (T6) |
| IN-5 | `Halt` : `Halted`, `pc` inchangé (reste sur le `Halt`) ; `Halted` et `Faulted` inchangés **bit-à-bit** par `step` répété ; fin de programme implicite : `Halted`, `pc = n` ; **états terminaux construits par `ScriptState::from_bytes`** (trois cas, un script valide fourni, par exemple `Set r0, 5` à la position 0 suivie d'un `Halt`) : un état `Halted`, puis un état `Faulted(NoScript)`, puis un état `Faulted(Budget)`, **chacun avec `pc` désignant l'instruction `Set` non terminale** (`pc = 0`), `wait = 0`, `r0 = 1` et d'autres registres non nuls ; `step` est appelé avec ce script (`Some`) et un **sink initialement vide** : l'encodage canonique (`to_bytes`) de l'état est **strictement égal, octet par octet**, à celui de l'état d'entrée, `r0` vaut toujours 1 et le sink reste vide et inchangé. Cela établit que **les trois états terminaux sont inertes indépendamment de la position de `pc`** (un état terminal atteint par `step` a toujours `pc` sur `Halt`, cas où une ré-exécution serait indiscernable : seuls ces cas distinguent un `step` qui ré-exécute) | oui |
| IN-budget | Binaire de `SCRIPT_MAX_STEPS_PER_TICK + 1` `Add r0, +1` consécutifs sans yield (tableau de taille constante calculée à partir de la constante) : `Faulted(Budget)` après l'exécution d'**exactement** `SCRIPT_MAX_STEPS_PER_TICK` instructions (`r0` = `pc` = `SCRIPT_MAX_STEPS_PER_TICK`) ; l'état est déterministe (deux appels) ; aucune émission supplémentaire | oui (T10) |
| IN-bound | Binaire de `SCRIPT_MAX_STEPS_PER_TICK` instructions dont la dernière est `Wait 1` : aucune faute, `Waiting` ; reprise normale au tick suivant | oui |
| IN-end | Binaire de `SCRIPT_MAX_STEPS_PER_TICK` `Add` suivis de la fin du programme : `Halted` implicite, aucune faute, `r0 = SCRIPT_MAX_STEPS_PER_TICK` | oui |
| IN-miss | Script absent (`None`), puis slot vacant, **slot occupé dans le manifeste par un asset d'un autre format** (représenté par `None` dans la `ScriptTable` : cette couche ne connaît pas les autres variantes d'`Asset` et **ne reconnaît aucun autre format** ; en U3 le cas est simulé par un slot sans dépôt, par exemple 2, alors que des scripts sont déposés aux slots 1 et 4 ; la preuve du lot mixte relève de SCR-C1 (U2), qui emploie un **tableau témoin local** et non la `ScriptTable` réelle, voir § 10 point 10 du document d'exécution) et slot hors `CAP` d'une `ScriptTable` : `Faulted(NoScript)`, sans panique ; une instance en `Waiting` dont le script manque se comporte comme `step` le prévoit (la faute survient à la reprise) | oui (T14) |
| IN-full | `step` sur un sink de capacité 1 : première `Emit` acceptée, seconde `Err(Full)`, `pc` non avancé sur la seconde, état et émission déjà acceptée conservés ; **propriété de la fonction pure** : la branche est inatteignable en configuration conforme (U4) | |
| IN-det | Même entrée, même sortie, deux appels (états, émissions) | |
| EM-1 | `Emissions` : acceptation jusqu'à `CAP`, `Err(Full)` au-delà sans modification, ordre conservé, `clear`, `CAP = 0` refuse tout, `PartialEq` sur la tranche observable | |
| EM-2 | `emissions_bound_holds(4, 64)` vrai (à la borne exacte), `(4, 63)` faux, `(0, 0)` vrai (**cas dégénéré** : aucune instance, capacité nulle), **`(usize::MAX, usize::MAX)` faux (débordement du produit)** | oui (T10) |
| EM-3 | `same_capacity(&Emissions<N>, &Emissions<N>)` accepte (accumulateur, transcript) ; `Clone` d'un accumulateur plein donne une valeur égale, sans insertion élément par élément | |
| TB-1 | `ScriptTable<'_, 6>` : dépôt aux slots 4 et 1 de scripts distincts ; `get(4)`, `get(1)` rendent leurs scripts ; `get(0)`, `get(2)`, `get(3)`, `get(5)` : `None` ; dépôt au slot 6 (`OutOfRange`) et au slot 4 une seconde fois (`Occupied`) refusés ; `get(6)` et `get(u16::MAX)` : `None` ; garde statique `CAP ≤ 65 536` vérifiée **en contexte de compilation** par l'évaluation de la constante associée de garde pour `CAP = 65 536` dans un `const _: () = …` (accepté) ; **aucune `ScriptTable<'_, 65 536>` n'est instanciée sur la pile** ; le refus de `CAP = 65 537` (échec de compilation) n'est pas testé | oui (T14, C13) |

**Rouge.** Squelettes `unimplemented!()` des fonctions et méthodes ; ST-1 à ST-4, IN-*, EM-*, TB-1 échouent par panique. **Sans rouge possible** : néant.

**Mutations d'U3** (copies isolées ; document d'exécution, U3, repris sans modification ; 29 lignes dont M-3.13b).

| # | Mutation | Détecteur | Autres assertions attendues en échec |
|---|---|---|---|
| M-3.1 | `wait` encodé en big-endian | ST-1 | ST-3 |
| M-3.2 | `from_bytes` accepte `Waiting` avec `wait = 0` | ST-2 | — |
| M-3.3 | `from_bytes` accepte `wait ≠ 0` pour `Halted` | ST-2 | — |
| M-3.4 | `from_bytes` accepte `fault ≠ 0` hors `Faulted` | ST-2 | — |
| M-3.5 | `from_bytes` accepte `fault = 3` | ST-2 | — |
| M-3.6 | `ScriptState::start` convertit le slot par `as u16` (troncature) | ST-4 (slot 65 536) | — |
| M-3.7 | `step` ne décrémente pas `wait` | IN-4 | ST-4 (trajectoire de référence) |
| M-3.8 | `step` décrémente `wait` deux fois | IN-4 (reprise trop tôt pour *n* = 3) | ST-4 |
| M-3.9 | reprise un tick plus tard (test de `wait == 0` avant le décrément) | IN-4 | ST-4 |
| M-3.10 | `Wait` ne termine pas le tour | IN-4, IN-bound | ST-4 |
| M-3.11 | `Emit` n'avance pas `pc` | IN-1 | ST-4 |
| M-3.12 | `Halt` avance `pc` | IN-5 | ST-1 non |
| M-3.13 | `Halted` ré-exécuté par `step` | IN-5 (cas de l'état `Halted` construit par `from_bytes` avec `pc` sur une instruction `Set` non terminale ; **aucun test nouveau**) | — |
| M-3.13b | `Faulted` (`NoScript` puis `Budget`, deux copies) ré-exécuté par `step` | IN-5 (cas des états `Faulted` construits par `from_bytes`, même position de `pc`) | — |
| M-3.14 | `JumpIfNonZero` inversé | IN-3 | ST-4 |
| M-3.15 | cible de `Jump` décalée de 1 | IN-3 | ST-4 |
| M-3.16 | budget : une instruction de plus autorisée (`executed > MAX`) | IN-budget (`r0 = MAX + 1`) | — |
| M-3.17 | budget : une instruction de moins (`executed >= MAX − 1`) | IN-bound | IN-end |
| M-3.18 | budget testé avant la recherche d'instruction (consomme le budget à la fin du programme) | IN-end | — |
| M-3.19 | budget ignoré (aucun test) | IN-budget (assertion sur `r0` et `Faulted`) ; **le binaire d'IN-budget est rectiligne : aucune boucle, aucun blocage** | — |
| M-3.20 | `Add` non enveloppant (`+` simple) | IN-2 (panique de débordement en débogage, rapportée comme telle) | — |
| M-3.21 | script absent provoque une panique | IN-miss | — |
| M-3.22 | `Emit` plein : `pc` avancé malgré l'erreur | IN-full | — |
| M-3.23 | `emissions_bound_holds` en `wrapping_mul` | EM-2 (cas de débordement) | — |
| M-3.24 | `Emissions::push` ignore la capacité (écrase) | EM-1 | EM-2 non |
| M-3.25 | `ScriptTable::get(index)` accepte `index == CAP` par indexation directe | TB-1 (`get(6)`) | — |
| M-3.26 | `ScriptTable::deposit` écrase un slot occupé | TB-1 | — |
| M-3.27 | `ScriptTable` indexée par le rang de dépôt au lieu du slot | TB-1 | IN-miss |
| M-3.28 | `Waiting` : `wait -= 1` non saturant (`-` simple) | **(équivalente)** : `Waiting ⇒ wait ≥ 1` est un invariant de l'état (champs privés, `from_bytes`) ; aucun état `Waiting` à `wait = 0` n'est constructible ; aucun test artificiel | — |

**Ancienne M-3.29 retirée des mutations.** Le changement de `ScriptState::registers()` d'un retour par copie à un retour par référence n'est pas une mutation de comportement équivalente (emprunt et durée de vie différents pour l'appelant) : il est classé **conformité de la signature publique** et vérifié par la revue du § 8 ; aucun test artificiel, aucune mutation rapportée. Une modification de signature publique n'est pas une mutation de comportement (règle 3 du § 1 du document d'exécution).

**Cardinalité** : pas de cardinalité maximale en U3 ; tableaux de quelques centaines d'octets au plus (IN-budget : `12 + 8·(SCRIPT_MAX_STEPS_PER_TICK + 1)` octets pour 17 instructions) ; Miri complet, sans réduction.

## 9. Arbitrages et limites clos (non rouverts)

- **A-S1 à A-S5** [Validé] : nouveau crate `verbum-script`, `no_std` sans `alloc`, **dépendance normale unique `verbum-assets`**, aucune dev-dépendance ; fonction d'exécution pure `step` dans ce crate, système interpréteur adapté dans l'assemblage (U4) ; validation structurelle des instructions dans `Script::read` ; `Wait 0` rejeté ; `Waiting ⇒ wait ≥ 1` ; analyse de flot sur tout le graphe avec coupure aux `Wait` (Forge) ; `TRANSCRIPT` remplacé à chaque tick (U4) ; `ScriptTable` indexée directement par `SlotIndex`, sans compactage ni table de correspondance ; encodage canonique de l'état exclusivement pour la reconstruction.
- **État par composant** (plan § 4.C, décision, non arbitrage) : un `ScriptState` par entité ; `ScriptState` `Copy`, sans référence ni durée de vie, champs privés, deux constructeurs (`start`, `from_bytes`), `step` seule transition. `ScriptTable` et émissions : Ressources de cardinal 1 (assemblage, U4).
- **Contraintes de `verbum-script`** : `no_std`, sans `alloc`, aucune allocation, aucun état externe ni `static`, aucun accès au `World` ni au Scheduler (le crate ne nomme ni `World`, ni `Scheduler`, ni système), capacités fixes (`Emissions<CAP>`, `ScriptTable<CAP>`), état reconstructible à partir de l'encodage canonique seul ; il se classe comme `verbum-render` (types du noyau atteignables depuis `run_tick`).
- **Valeurs 4 et 16** : provisoires, propres à cette première implémentation, non invariants du moteur ; définies dans `verbum-assets`, réexportées ici ; épinglées seulement par SCR-K1 (U1) et ST-1 (longueur d'encodage). Les tests de budget construisent leurs programmes à partir des constantes.
- **`step` ne vérifie ni cycles ni budget statique** : le budget est appliqué à l'exécution (défense contre un artefact non conforme) ; la Forge (U2) garantit statiquement qu'un script accepté n'atteint jamais `Faulted(Budget)`.
- **Écarts d'exécution tranchés applicables à U3** : **E-2** (`Halt` n'avance pas `pc` ; fin de programme : `pc = n`) ; **E-4** (codes numériques de l'encodage) ; **E-5** (constantes de type `usize`) ; **E-6** (`is_empty` sur `Emissions`) ; **E-10** (toute mutation non équivalente est détectée par un test nommé ; une mutation démontrée équivalente est rapportée avec justification sans test artificiel ; M-3.29 retirée, reclassée en revue de signature).
- **Décisions d'U1 et d'U2 sans effet de conception sur U3** (consignées pour mémoire) : le lecteur ne contrôle pas les octets ignorés ; la Forge écrit les champs ignorés à zéro ; `check_flow` vérifie le budget sur tous les nœuds `0..n` et refuse les cycles sans yield y compris sur code inaccessible (interprétation E-11, acceptée) ; SCR-C1 compte six dépôts (le « cinq » des documents est une coquille arithmétique, confirmée par l'utilisateur) ; la normalisation défensive d'une cible invalide vers `FIN` dans `check_flow` est hors du domaine valide. Aucun de ces points n'est rouvert.
- **Leçons de l'étape 5 applicables** : E-14 (`dead_code` sous MSRV 1.85 pour les constantes utilisées seulement par une `const` assert) ; E-13 (parsing TOML sous Miri prohibitif) est **sans objet** en U3 (aucun TOML).

## 10. Interdits

- Ne modifier ni les ADR, ni `ROADMAP.md`, ni `docs/implementation/00` à `05`, ni les handoffs, ni `crates/engine/{ecs,schedule,render,verbum}/**`, ni `crates/engine/shell/**`, ni `crates/engine/assets/**` (inchangé depuis U1), ni `crates/engine/forge/**` (inchangé depuis U2) ; ne pas modifier le plan, le document d'exécution ni ce handoff sans décision explicite de l'utilisateur.
- Écrire uniquement : les fichiers créés du crate `verbum-script` (§ 7) et le `Cargo.toml` racine (deux ajouts). **Aucune dépendance** autre que `verbum-assets` pour `verbum-script` ; **aucune `[dev-dependencies]`** ; aucune dépendance ajoutée à `verbum-assets`, `verbum-forge` ni `verbum-shell`. `verbum-forge` ne dépend jamais de `verbum-script`.
- Aucun `unsafe`, `extern crate alloc`, `Vec`, `Box`, `String` ; aucun cast d'octets, `transmute`, `Box::leak`, `include_bytes!`.
- Dans le crate : aucun `static`, `thread_local`, `Cell`, `RefCell`, atomique, `async`, `await`, thread, `dyn`, `Rc`, `Arc` (G-ST). *Exception de test unique du plan, M-4.8, appartient à U4, non à U3.*
- Aucun ordre des émissions entre instances ni ordre dense d'un stockage affirmé (G-ORD-S : sans objet en U3, aucune instance multiple) ; aucun historique d'émissions ; aucun `LOG_CAP`.
- Aucun opérande, instruction ou catégorie hors du jeu fermé du plan § 4.D ; aucun flottant ; aucune lecture de donnée métier du `World` par un script.
- Aucun système, `Binding`, `ComponentSet`, `Anchored`, `Beat`, assemblage ni test d'intégration : U4 et U5.
- Aucun trait public ; aucun test ni assertion hors liste du plan et du document d'exécution ; aucune API au-delà du § 3.3 (règle 9) ; aucun test ni mutation d'U4 ou d'U5.
- **Aucun commit par Claude.**

## 11. Divergence, doute, blocage

Arrêt immédiat, sans choix de la session, en cas de : contradiction ou ambiguïté du plan ou d'un ADR ; divergence entre un fait [Vérifié] (F1, F2, F4, F6) et le code réel fourni ; besoin d'`unsafe`, d'allocation (`alloc`, `Vec`, `Box`), d'un état externe ou d'un état dans `step` (**G-ST**) ; exigence d'ordre inter-instances (G-ORD-S) ; test dont le rouge ne peut pas être constaté pour la bonne raison ; **mutation non détectée et non établie équivalente**, ou qui échoue sur d'autres assertions que celles annoncées sans explication ; besoin de modifier un fichier interdit ; besoin d'une dépendance autre que `verbum-assets`, d'une `[dev-dependencies]` ou d'un trait public ; test hors liste ; fichier nécessaire non fourni ; durée Miri jugée excessive (rapporter et arbitrer, aucun `--skip` ni `ignore` sans décision). La session fournit un constat factuel (source, texte, observation) et présente les options minimales, **sans arbitrer**.

**Ambiguïtés relevées à la rédaction, non résolues** (à exposer à l'utilisateur au démarrage, avant tout fichier ; l'utilisateur tranche).

- **AMB-2 — ST-1 / ST-4 et M-3.12.** La ligne M-3.12 (« `Halt` avance `pc` ») annonce IN-5 pour détecteur et « ST-1 non » en autres échecs. Le document ne dit pas si ST-1 obtient les états de fin de tick par **exécution de `step`** sur le scénario de référence (auquel cas le littéral du tick 10, `pc = 6`, échouerait sous M-3.12) ou par **construction** à partir de littéraux. La session établit la provenance exacte au rouge, rapporte les échecs observés pour M-3.12, et expose tout écart avec « ST-1 non » avant de conclure. Le plan (§ 7.3, ST-1) ne tranche pas.
- **AMB-3 — `same_capacity` (EM-3).** La fonction est nommée dans le plan et le document d'exécution mais ne figure pas dans l'API publique du § 3.3. Non précisé : fonction de test locale ou export public. Le document d'exécution interdisant toute API au-delà du § 3.3, la lecture prudente est une fonction **locale au test** ; **à confirmer par l'utilisateur**.
- **AMB-4 — `clippy::new_without_default`.** `Emissions::new` et `ScriptTable::new` sont des `new()` sans argument ; le § 3.3 ne prévoit aucun `Default` public. Le modèle fourni (`render/src/accumulator.rs`) résout le cas par `#[allow(clippy::new_without_default)]` localisé et motivé, sans `Default` public. **À confirmer par l'utilisateur** : reprise de ce traitement pour `Emissions` et `ScriptTable`, ou autre solution.
- **AMB-5 — Débordement de `pc` (point à établir, non un test).** `pc` est un `u16` ; `n ≤ 65 535` ; `pc` n'est pas contraint par l'encodage (un état construit par `from_bytes` peut porter `pc = 65 535`). La session établit **par raisonnement écrit** que `pc + 1` ne déborde jamais à l'exécution d'une instruction (l'index exécuté est `≤ n − 1`) ou expose le cas. Aucune assertion ni aucun test n'est ajouté.
- **AMB-6 — Faits non lus à la rédaction du document d'exécution.** `render/src/{lib,accumulator}.rs` et `ecs/src/entity.rs` étaient « non lus » (§ 10 point 2) ; ils sont désormais fournis. Un écart entre leur contenu et ce que le document d'exécution en dit (accumulateur, garde statique) est un arrêt.
- **AMB-7 — Identification de la révision du document d'exécution** : « 3 novembre au soir » (déclaration de l'utilisateur) contre « révision du 3 novembre 2026 (nuit) » (en-tête du fichier). Sans effet sur U3 ; à confirmer pour éviter toute confusion de version.
- **AMB-8 — Liste des fichiers à fournir.** Le document d'exécution fixe comme minimum `assets/src/{script,lib}.rs`, `Cargo.toml` racine, `render/Cargo.toml`, `render/src/{lib,accumulator}.rs`, `ecs/src/entity.rs`. Ce handoff y ajoute `assets/src/manifest.rs` (lecture seule, définition de `SlotIndex` pour F6), le rapport d'U2, un `tree.md` régénéré, et classe les sources d'U2 comme facultatives. Précision de handoff, non modification d'exigence ; à confirmer.

**Points à confirmer par l'implémentation** (non résolus à la rédaction) : (a) forme exacte de `Script::instruction` et de `Instruction` sur les fichiers fournis ; (b) forme de la constante associée de garde de `ScriptTable` (visibilité nécessaire à son évaluation dans TB-1, sans API publique nouvelle ; motif de `entity.rs`) ; (c) compilation sous MSRV 1.85 de `[None; CAP]`, de `checked_mul` en `const fn` et de la garde (§ 10 point 6 du document d'exécution) ; (d) durée de Miri (non mesurée) ; (e) noms, layouts et signatures provisoires [L] : tout ajustement listé au rapport sans changement de sémantique.

## 12. Rapport de fin d'unité et reprise

**Gabarit** (rien de narratif ; document d'exécution § 5) : 1. Unité, état (livrée, en attente de validation). 2. Fichiers créés et modifiés (chemins complets), fichiers livrés par `SendUserFile`. 3. Faits [Vérifié] revérifiés : confirmés ou divergence (arrêt). 4. Rouge : test, motif d'échec observé, ou « sans rouge possible », motif. 5. Quatre contrôles : lignes réelles, **espace de la session** et **dépôt réel** séparés ; contrôles non exécutés nommés. 6. Mutations : mutation, détecteur, autres assertions échouées, **mode de détection observé** (assertion, erreur de décodage ou panique) ; équivalentes avec justification (M-3.28) ; le premier échec comparé au détecteur annoncé (§ 8, précisions b et c). 7. Portes : G-CM, G-LT (sans objet, motivé), G-ORD-S (sans objet, motivé), G-ST ; liste R-ADAPT. 8. Garanties : statiques / revue (sans mélange), **revue G-ST point par point**, **revue de la signature publique points (a) à (e)**. 9. Contrôles natifs, Miri (intégral ou partiel, avec la liste des réductions ; commande : `cargo +nightly miri test -p verbum-script`), MSRV (non exécuté en U3 ; signalé). 10. Ajustements de noms et écarts du plan ou du document d'exécution, réponses aux AMB-2 à AMB-8 telles que tranchées par l'utilisateur. 11. Regroupement de commit proposé par invariant ; `tree.md` à régénérer (fichiers du crate créés). 12. Fin : « En attente de la validation de l'unité U3. »

**Attente, pas constat** : rouge constaté par panique ; vert ; 29 mutations traitées (M-3.28 équivalente) ; quatre contrôles verts ; `cargo tree` conformes ; revue G-ST et revue de signature rapportées ; Miri vert. **Critère de sortie** : ST-*, IN-*, EM-*, TB-1 verts ; mutations traitées ; revues rapportées ; quatre contrôles verts ; Miri vert.

**Résultats séparés** : tout résultat de l'espace de travail est étiqueté « espace de travail » ; seuls les résultats rapportés par l'utilisateur sont étiquetés « dépôt réel ».

**Reprise** : si la session se dégrade, une nouvelle session reçoit ce handoff, le document d'exécution, le plan, les documents du § 3, un `tree.md` à jour et les fichiers du § 7 ; elle relit l'état réel par les contrôles **exécutés par l'utilisateur** ; une unité interrompue est refaite depuis ses tests ; elle ne rouvre aucun arbitrage clos ni aucune unité validée. Après validation d'U3 (« unité U3 validée »), l'utilisateur demande le handoff d'U4 ; aucune session ne traite deux unités.

## 13. Consigne d'amorce

*Cette consigne prévaut sur celle du § 0 du document d'exécution lorsque le handoff est fourni.*

> Vous implémentez **uniquement l'unité U3** de l'étape 6 de Verbum : le crate `verbum-script` (état d'exécution et encodage canonique, accumulateur d'émissions, table de scripts indexée par slot, borne des émissions, fonction pure `step`), `no_std` sans `alloc`, dépendance normale unique `verbum-assets`. Lisez, dans cet ordre : `06-verbum-scripts-implementation-handoff-u3.md`, `06-verbum-scripts-execution.md` (règles § 1, interdits § 2, faits § 3 dont § 3.3, unité U3 du § 4, § 5, § 6, § 9, § 10 points 2, 6 et 10), `06-verbum-scripts.md` (plan r1.2, une seule fois ; puis § 4.C, § 4.D, § 4.E, § 4.F, § 4.G, § 5, § 6, § 7.3, § 7.6, § 8 ligne U3, § 9), `ADR-Sc01` et `ADR-000` (points 1, 3, 4), puis `tree.md`. Vous n'avez pas accès au dépôt : les contrôles réels et Miri sont exécutés par l'utilisateur, les fichiers sont livrés complets (pas de diff), les fichiers d'U3 vous sont fournis nommément. N'abordez en aucun cas U4 ni U5 ; ne modifiez ni `verbum-assets`, ni `verbum-forge`, ni le noyau ECS, ni le Scheduler, ni le Shell. N'écrivez aucun code et ne lancez aucune commande avant que l'utilisateur ait dit exactement : « démarrez l'unité U3 ». Accusez réception en cinq lignes maximum, en indiquant le statut des documents tel qu'au § 5. Après l'instruction de démarrage, exposez les ambiguïtés AMB-2 à AMB-8 du § 11.
