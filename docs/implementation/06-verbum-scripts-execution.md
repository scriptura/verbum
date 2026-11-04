# Exécution — étape 6 : Scripts (`verbum-assets`, `verbum-forge`, `verbum-script`, `verbum-shell`)

Document d'exécution, non normatif. **Il ne contient aucune règle de conception.** Source de vérité unique : `docs/implementation/06-verbum-scripts.md`, **r1.2 (1er novembre 2026), validée par l'utilisateur**, arbitrages A-S1 à A-S5 arrêtés. En cas de divergence entre ce document et le plan, **le plan prévaut** ; la divergence est signalée à l'utilisateur, jamais résolue par la session. Les écarts d'exécution connus à la rédaction sont listés au § 9 et **soumis à validation avec ce document**. Ce document n'est modifié que sur décision explicite de l'utilisateur. Le normatif est `ADR-000`, `ADR-001`, `ADR-003`, `ADR-004`, `ADR-005`, `ADR-006`, `ADR-A01`, `ADR-Sc01`, `ADR-SV01` (pour ses exclusions), `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`.

**État : validé (révision après audit du 1er novembre 2026 : budget vérifié sur tous les nœuds (U2, E-11), champs ignorés dans SCR-R8 (U1), reclassement de M-3.29 (U3), reformulation de M-4.10 (U4) ; finalisation du 3 novembre 2026 : SCR-F4 (k) distinguant budget et cycle sur code inaccessible, réexamen de M-2.14, M-3.29 retirée des mutations équivalentes, M-4.10 reformulée en application prématurée des commandes ; révision du 3 novembre 2026 (soir) après audit : composant `Beat` ancré (U4), IN-5 étendu aux états construits par `from_bytes`, M-4.6 et M-4.13 corrigées, M-4.16 et M-4.17 ajoutées, constantes de capacité employées dans les `SparseSet`, interprétation du plan § 14.2 inscrite en E-10, précisions mineures ; révision du 3 novembre 2026 (nuit) : types explicites de l'assemblage U4 (`Components<'a>`, ancrage de `Beat`), IN-5 sur une instruction `Set` non terminale, IN-miss précisé, découpage **une session par unité**, handoff d'U1 produit séparément). Aucune unité ne démarre avant l'instruction explicite : « démarrez l'unité U1 ». Le handoff d'U1 est produit séparément (`06-verbum-scripts-implementation-handoff.md`) ; les handoffs d'U2 à U5 ne sont pas produits.**

Légende, appliquée dans chaque unité :
- **[Vérifié]** : fait constaté dans un document lu ou dans le code réel, avec sa source. Revérifié au démarrage de l'unité ; une divergence est un arrêt (§ 6).
- **[Plan]** : décision du plan r1.2. Ce n'est pas un fait.
- **[Exécution]** : précision opérationnelle de ce document, sans effet sur un invariant (liste des écarts : § 9).
- **[L]** : choix local provisoire (nom, layout, code), révisable sans ADR.
- **[À démontrer]** : point que l'unité doit établir, ou qu'elle ne peut pas établir et qui reste signalé au rapport.

## 0. Consigne d'amorce (à coller au début de la session)

*Si le handoff `06-verbum-scripts-implementation-handoff.md` est fourni, sa consigne d'amorce prévaut ; celle-ci sert quand le document d'exécution est fourni seul.*

> Vous implémentez l'étape 6 de Verbum (scripts : lecteur dans `verbum-assets`, compilation dans `verbum-forge`, crate `verbum-script`, intégration dans `verbum-shell`). Lisez, dans cet ordre : `docs/implementation/06-verbum-scripts-execution.md` (règles de session), puis `docs/implementation/06-verbum-scripts.md` (plan r1.2 ; à lire une seule fois en entier), puis `docs/architecture/adr/ADR-Sc01-scripts.md`, `ADR-A01-assets.md` et `ADR-000-language-and-material-invariants.md` (points 1, 3 et 4), puis `docs/implementation/05-verbum-assets-execution.md` (§ 3.2, E-12 à E-14 seulement), puis `tree.md` (fait foi sur les chemins). Les autres ADR ne se relisent que si une unité les cite. Ne demandez pas le dépôt en entier : demandez par nom de fichier ce qui manque (chaque unité liste ses fichiers, § 4). N'écrivez aucun code avant que l'utilisateur ait dit : « démarrez l'unité U1 ». Accusez réception en cinq lignes maximum.

## 1. Règles de session

1. **Une unité à la fois.** Le § 4 reprend U1 à U5 du plan, sans les fusionner ni les réordonner. À la fin d'une unité, la session s'arrête et rédige le rapport (§ 5). Elle ne commence la suivante que sur validation explicite (« unité Un validée, démarrez Un+1 »). **Découpage en sessions [Exécution, retenu par l'utilisateur le 3 novembre 2026]** : **une session Claude par unité** : U1, U2, U3, U4 et U5, chacune dans une session distincte. La recommandation A/B/C du plan (A = U1 et U2 ; B = U3 et U4 ; C = U5) reste un élément de planification et **n'est pas suivie pour l'exécution**. U4 garde sa session propre (contexte ECS/Scheduler, porte E-12). Une session ne traite que son unité et n'aborde jamais les suivantes. Le handoff d'U1 est `06-verbum-scripts-implementation-handoff.md` ; chaque handoff suivant est produit sur demande, après validation de l'unité précédente.
2. **Rouge avant vert quand un rouge est possible.** Tests écrits contre des squelettes d'API (signatures, corps `unimplemented!()`), constat d'un échec pour la raison attendue, puis implémentation. **Un échec de compilation n'est pas un rouge valide.** Le rouge est rapporté (test, motif d'échec observé), ou « sans rouge possible » avec motif.
3. **Sensibilité par mutation** : sur copie isolée **hors de l'arbre de travail**, une seule mutation à la fois, jamais présentée comme rouge initial. Pour chaque mutation, le rapport nomme **le test qui la détecte et les autres assertions qui échouent**. **Une mutation non détectée est un arrêt (§ 6), sauf si elle est établie équivalente** : une mutation équivalente est une modification qui ne change aucun comportement observable dans le domaine atteignable ; elle est alors rapportée comme telle, avec sa justification, et **aucun test artificiel n'est ajouté pour la détecter** (précédents : M-4.1 et M-4.3 de l'étape 5). Les équivalences connues à la rédaction sont marquées « (équivalente) » aux § 4. Une modification de signature publique (type de retour, emprunt, durée de vie) n'est **pas** une mutation de comportement : elle relève de la revue de conformité de l'API (§ 4, U3), non de cette règle. Une détection par échec de compilation n'est pas admise, **sauf la mutation « capacité des émissions sous la borne » (U4, M-4.12), dont la détection par la `const` assert est l'exception du plan § 7.6**.
4. **Contrôles réels.** À chaque unité : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, exécutés **par l'utilisateur** (la session n'a pas accès au dépôt) ; la session les exécute dans un espace de travail reconstitué et distingue les deux résultats. Le rapport donne les lignes de résultat réelles. Aucun résultat n'est déduit, estimé ni rapporté sans avoir été produit.
5. **Miri et MSRV 1.85 sont exécutés par l'utilisateur** : Miri en fin de chaque unité qui touche du code d'un crate (§ 8 donne les commandes et la répartition natif/Miri), MSRV en U5. La session fournit les commandes exactes. **Deux commandes séparées pour MSRV** : `cargo +1.85.0 build --workspace`, puis `cargo +1.85.0 test --workspace`.
6. **Compatibilité MSRV 1.85 dès l'écriture** : pas de chaînes `if let … && …` (let-chains stables à partir de 1.88) ; `let … else` à la place ; aucune assertion sur l'horloge murale ; cadence = fermeture vide dans les tests, seul l'exemple (U5) utilise `FixedStepPacing` ; **aucune constante utilisée seulement dans un `const _: () = assert!(…)`** (avertissement `dead_code` sous 1.85, E-14 de l'étape 5) : chaque constante est aussi utilisée dans du code compilé ; `is_empty` accompagne tout `len` public (`clippy::len_without_is_empty`).
7. **Pas de commit par la session.** Elle propose, en fin d'unité, un regroupement par invariant démontré ; messages et commits restent à la charge de l'utilisateur. `tree.md` est régénéré par le script de l'utilisateur : la session signale les fichiers créés, sans y toucher.
8. **Livraison.** Fichiers `.rs` livrés **complets** via `SendUserFile`, première ligne = commentaire de chemin (`// crates/engine/assets/src/script.rs`) puis ligne vide ; les `Cargo.toml` sont exemptés du commentaire. Documentation des sources `.rs` **en français** ; champ `description` des `Cargo.toml` en anglais. Un fichier existant modifié n'est livré qu'après avoir été fourni par l'utilisateur, et rendu en entier. Style de réponse : vouvoiement, concis, structuré, opérationnel.
9. **Noms provisoires [L].** Les noms, layouts et signatures du § 3.3 et du § 4 sont provisoires ; tout ajustement est listé au rapport, sans changement de sémantique. **Aucun trait public** de chargement ni d'exécution ; aucun test ni aucune API au-delà du plan.
10. **Économie de contexte.** Lire le plan une fois en entier, puis seulement la section de l'unité en cours ; ne pas relire les fichiers que la session vient d'écrire ; rapports concis.
11. **Résultats séparés.** Tout résultat de l'espace de travail de la session est étiqueté « espace de travail » ; seuls les résultats rapportés par l'utilisateur sont étiquetés « dépôt réel ».

## 2. Interdits sans arbitrage explicite de l'utilisateur

- Rouvrir un arbitrage clos du plan (A-S1 à A-S5, `Wait 0` rejeté, `Waiting ⇒ wait ≥ 1`, valeurs provisoires 4 et 16 dans `verbum-assets`, analyse de flot sur tout le graphe avec coupure aux `Wait`, `TRANSCRIPT` remplacé à chaque tick, `ScriptTable` indexée directement par `SlotIndex`, encodage canonique de l'état exclusivement pour la reconstruction) ou modifier le plan, les ADR, `ROADMAP.md`, les documents `00` à `05`, les handoffs.
- Modifier `crates/{ecs,schedule,render,verbum}/**` ; modifier **`crates/engine/shell/src/**`** (y compris la documentation `//!` : aucun doctest nouveau) ; modifier `crates/engine/assets/src/{load,manifest,header,map,font,tileset}.rs` (sauf adaptation de signature mécanique exigée et exposée avant d'être faite, règle R-ADAPT du plan 05 § 3.1) ; modifier un test d'une étape antérieure autrement que par adaptation de signature consignée une à une.
- Ajouter une dépendance normale ou de développement à `verbum-assets` (`[dependencies]` **et** `[dev-dependencies]` restent vides, vérifié par `cargo tree -p verbum-assets`) ; ajouter à `verbum-script` une dépendance autre que `verbum-assets`, ou une `[dev-dependencies]` ; ajouter à `verbum-forge` une dépendance autre que `toml` (et `verbum-assets`, `verbum-ecs` en développement déjà présentes) ; faire dépendre `verbum-forge` de `verbum-script`, ou `verbum-assets` de quoi que ce soit.
- Utiliser `unsafe` (lint du workspace), `extern crate alloc`, `Vec`, `Box`, `String` dans `crates/engine/assets/src/**` et `crates/script/src/**` ; un cast d'octets, `transmute`, `Box::leak`, `include_bytes!` ou un `static` pour contourner une question de durée de vie (G-LT).
- Introduire dans `crates/script/**` ou dans le corps d'un système : `static`, `thread_local`, `Cell`, `RefCell`, atomique, `async`, `await`, thread, `dyn`, `Rc`, `Arc` (G-ST). *Exception de test, une seule* : l'atomique de la mutation M-4.8, sur copie isolée seulement.
- Créer un **nouveau niveau** de contrôle, changer leur ordre, déposer avant la fin du niveau 3, modifier les constantes ou le layout de `VMAN`, introduire une allocation dans `load`, modifier le modèle d'identité : **G-CM** (§ 3.2).
- Affirmer dans un test, un exemple ou une API **un ordre des émissions entre instances** ou l'ordre dense d'un stockage (**G-ORD-S**) ; comparer par un ensemble qui efface les doublons (multiplicité conservée).
- Lire l'accumulateur `EMISSIONS` hors des systèmes ; conserver un historique d'émissions dans le `World` ; ajouter un `LOG_CAP` ou un journal persistant.
- Utiliser `BufferFull` pour signaler `Full`, ignorer un `Err(Full)`, ou introduire un `panic!` comme politique de saturation (§ 4 U4 : seul `unreachable!` de violation de contrat).
- Ajouter un opérande, une instruction ou une catégorie hors du jeu fermé du plan § 4.D ; ajouter une lecture de donnée métier du `World` par un script ; introduire un flottant.
- Ajouter un test ou une assertion non listés au plan ou à ce document ; implémenter un code de visualisation, de sauvegarde, de réseau.

## 3. Faits vérifiés communs à toutes les unités

### 3.1 Faits [Vérifié]
- **F1. Workspace.** `Cargo.toml` racine courant : `resolver = "3"` ; `members` = `crates/{ecs, schedule, verbum, assets, forge, shell, render}` ; `[workspace.package]` : édition 2024, `rust-version = "1.85"` ; `[workspace.dependencies]` = `verbum-assets`, `verbum-forge`, `verbum-ecs`, `verbum-schedule`, `verbum-render`, au motif `{ path = "crates/…", version = "0.1.0" }` (**aucune entrée pour `verbum-shell`**, qui n'est jamais une dépendance) ; `[workspace.lints.rust] unsafe_code = "deny"`. **Source : `Cargo.toml` fourni le 1er novembre 2026 ; à revérifier au démarrage d'U3.**
- **F2. Manifestes.** `verbum-assets` : `[dependencies]` vide, `[lints] workspace = true`. `verbum-forge` : dépendances `verbum-assets` (par `path = "../assets"`) et `toml` (`version = "1"`, `default-features = false`, features `std`, `parse`, `serde`) ; dev : `verbum-ecs`. `verbum-render` : `[dependencies]` vide ; dev : `verbum-ecs`, `verbum-schedule`. `verbum-shell` : `[dependencies]` vide ; dev : `verbum-assets`, `verbum-ecs`, `verbum-forge`, `verbum-render`, `verbum-schedule` (`.workspace = true`). Tous héritent `version`, `edition`, `rust-version`, `authors`, `license`, `repository`, `keywords`, `categories` du workspace.
- **F3. Formats et décodage (assets).** `Format` ∈ {`Sprite`, `Tileset`, `Map`, `Font`} (`Clone, Copy, Debug, PartialEq, Eq`) ; `Asset<'a>` : une variante par format (`Clone, Copy, Debug, PartialEq, Eq`) ; `Format::of_identity(&[u8]) -> Option<Format>` est `pub(crate)` et matche les identités `VSPR`, `VTLS`, `VMAP`, `VFNT` ; `decode(expected, bytes)` fait un **`match` exhaustif sans bras par défaut** sur `Format` : un nouveau variant exige un bras ; `Decoded::Ready(Asset)` pour les formats sans référence. Source : `format.rs`.
- **F4. Lecture (assets).** `ReadError` ∈ {`Truncated`, `FormatIdentity`, `Version`, `PayloadLength`, `TrailingBytes`} (`Clone, Copy, Debug, PartialEq, Eq`, **sans `#[non_exhaustive]`**), défini dans `sprite.rs` ; `header.rs` : `PREFIX_LEN = 10`, `read_prefix(bytes, identity, version) -> Result<Prefix, ReadError>` (complétude, identité, version), `Prefix { declared_len: u32, rest: &[u8] }`, `identity_of`. Modèle de lecteur à charge variable : `font.rs` (`split_at_checked`, `<&[u8; N]>::try_from`, longueur déduite en `u64`, vue `Copy` empruntée, accesseur borné par `get`). Sources : `header.rs`, `font.rs`, `map.rs`.
- **F5. `load` (inchangé).** `load<'a>(manifest: &[u8], artifacts: &[&'a [u8]], slot_formats: &[Format], deposit: impl FnMut(SlotIndex, Asset<'a>)) -> Result<(), LoadError>` ; niveau 2 : `check_bindings` puis rejet `SlotFormat` d'une identité **connue** différente du format attendu ; niveau 3 : `materialize` (`Decoded::Ready` → valeur) ; niveau 4 : redécodage et dépôt, seconde lecture en échec = `unreachable!`. Source : `load.rs`.
- **F6. Indices.** `SlotIndex` enveloppe un `u32` : `SlotIndex::new(u32)` public sans vérification de capacité, `get() -> u32`, `Clone, Copy, Debug, PartialEq, Eq` ; `ArtifactIndex` : `get()` seulement ; `AssetId(u64)` opaque, `from_raw` `pub(crate)`. Source : `manifest.rs`.
- **F7. Forge.** `AssetKind` ∈ {`Sprite`, `Tileset`, `Map`, `Font`} ; `AssetSource<'a> { kind, key, toml, slot }` ; `compile_assets(&[AssetSource<'_>], usize) -> Result<CompiledSet, ForgeError>` (signature épinglée par doctest) ; `compile_assets_with` (`pub(crate)`) : **`match` exhaustif sur `AssetKind`** ; `Pending::{Ready(Vec<u8>), Map(MapSource)}` ; `compile_sprite`, `compile_tileset`, `compile_font` sont `pub fn` réexportées par `lib.rs` ; `ForgeError` (défini dans `sprite.rs`, `Clone, Debug, PartialEq, Eq`, variantes à `String`, `&'static str`, `Box<ForgeError>`) ; helper `dimension` (`pub(crate)`) **refuse zéro** et ne convient donc pas aux opérandes qui admettent zéro (canal, registre). Le TOML se lit par `toml::Table`, `toml::Value::{String, Integer, Array, Table}` ; l'erreur de syntaxe est rendue par `error.to_string()`. Source : `lib.rs`, `batch.rs`, `sprite.rs`, `map.rs`.
- **F8. Scheduler.** `Scheduler::<B, L, SYSTEMS, PHASES>::new() -> Result<Self, ModelError>` ; `run_tick::<ENTITIES, COMMANDS, S, R>(&self, &mut B::World, &mut B::Buffer) -> Result<(), TickError>` avec `R: Default` ; `model()` ; `contract_of<'a, B: Binding + 'a, S: System<B>>() -> Contract` (const fn) ; `SystemList::{LEN, fill(&mut [(usize, Contract)], offset)}` ; `Binding::Parts<'w> where Self: 'w`, `System::Access<'w> where B: 'w` ; `Contract::{reads, writes, produces, effects}`, `DataSet::of`, `DataSet::contains`, `Contract: PartialEq`. Source : `scheduler.rs`, `system.rs`, `contract.rs`, `data.rs` (lu par ses usages).
- **F9. Accès.** `Write<ID, ValuesMut<'w, T, S, D>>` expose `len`, `is_empty`, `contains`, `get`, `get_mut`, `entities` ; `Read`/`Write`/`Produce` sur `&mut ResourceSlot<T>` exposent `get` (et, pour `Write`/`Produce`, `get_mut` et `insert`) ; une catégorie ne peut figurer deux fois dans un accès (erreur E0080) ; `Produce`/`Write` n'ajoutent pas de lecture au contrat. Source : `access.rs`.
- **F10. ECS.** `World<ENTITIES, S: ComponentSet, R>` : `new` (`R: Default`), `spawn`, `components()`, `resources()`, `resources_mut()`, `split_mut() -> (S::Values<'_>, &mut R)`, `apply(&mut CommandBuffer<COMMANDS, S::Add, S::Remove>)` ; **`add_component` et `remove_component` du `World` sont `pub(crate)`** : un composant ne s'ajoute depuis un autre crate que par `CommandBuffer::add_component` + `World::apply` ; `ComponentSet: Default` avec `Add: Copy`, `Remove: Copy`, `Counters: Copy + Default + AsMut<[usize]>`, `Values<'a>: ValuesView` et les méthodes `kind`, `remove_kind`, `dense_len`, `dense_capacity`, `covers`, `is_present`, `insert`, `remove`, `remove_all`, `values_mut` (modèle : doctest de `component_set.rs`) ; `SparseSet<T: Copy, SPARSE, DENSE>` : `get`, `len`, `entities`, `values_mut`. `ResourceSlot<T>` sans contrainte sur `T`, `Default`. Source : `world.rs`, `component_set.rs`, `command.rs`, `storage.rs`, `values.rs`, `resource.rs`.
- **F11. Durée de vie et E-12.** G-LT positive sur `Map<'a>` et `Font<'a>` (étape 5) : une Ressource portant une vue empruntée est déposée dans un vrai `World` et lue sous `run_tick`, octets construits à l'exécution déclarés avant le `World`. **E-12** : sous `Binding<'a>`, un système dont l'accès ne porte aucun type en `'a` ne compile pas (E0477) ; contournement local `Anchored<'a, T>` (enveloppe `PhantomData<&'a ()>`) **limité aux tests et à l'exemple**. L'enveloppe vaut pour tout type d'accès sans `'a`, **ressource ou composant stocké** : pour `Write<BEAT>`, le composant est stocké sous la forme `Anchored<'a, Beat>` ; aucun contrat n'est modifié. L'interpréteur, dont l'accès nomme `ScriptTable<'a, CAP>`, n'en a pas besoin. Source : `05-verbum-assets-execution.md` § 9 (E-12), `level_loaded.rs`, `borrowed_resource.rs`.
- **F12. Shell.** `run_loop(cadence, stop, inbox, infrastructure) -> Result<u64, E>` ; `Inbox::<u8>::new(4)` ; l'assemblage de l'étape 5 (`execute`, journal, présentation par fermeture) est dans `crates/engine/shell/tests/level_loaded.rs` et `examples/level_headless.rs`. **Source : fichiers lus le 1er novembre 2026 ; `crates/engine/shell/src/{shell,inbox,pacing}.rs` non lus : à revérifier au démarrage d'U5.**
- **F13. Étape 5, leçons.** E-13 : sous Miri, le parsing TOML de gros volumes est prohibitif (`fnt_f2` : 64 glyphes en 16 s, 65 535 non terminant) ; les tests de cardinalité maximale se découpent en assistant testé directement + bout en bout natif seulement ; exclusions Miri de `verbum-forge` : `p3_1`, `p3_2`, `t6_1`, `t6_3`, `t6_6` (système de fichiers réel) par `--skip`, jamais par `ignore`. E-14 : voir règle 6 du § 1. Source : `05-verbum-assets-execution.md` § 9.
- **F14. Non-régression.** Tests de l'étape 5 à ne pas modifier : `assets` (`l1` à `l5`, `m1` à `m8`, `p1_1` à `p1_6`, `sf1` à `sf6`, `rf1` à `rf6`, `tls_*`, `map_*`, `fnt_*`), `forge` (`f1` à `f8`, `p2_*`, `map_f*`, `fnt_f*`, `fs_*`, tests de `tests/pipeline.rs`), `shell` (`lifecycle`, `pacing`, `acquisition`, `acceptance`, `frame_loop`, `borrowed_resource`, `level_loaded`, doctests LT-3 à LT-5t).

### 3.2 Portes opérationnelles [Plan]

**G-CM — préservation des mécanismes de l'étape 5 (plan 05 § 1.3).** Sont admises sans arrêt : les validations propres au format script **à l'intérieur** des niveaux existants (niveau 2 : format du slot ; niveau 3 : validation des enregistrements par `Script::read`), et les ajouts additifs d'API (`Format::Script`, `Asset::Script`, `ReadError::Instruction`, `Script`, `Instruction`, constantes `SCRIPT_*`). Sont un **arrêt avec rapport** : création ou suppression d'un niveau, changement de leur ordre, dépôt avant la fin du niveau 3, modification de `VMAN`, introduction d'une allocation dans `load`, modification du modèle d'identité, modification d'une assertion d'un test de F14.

**G-LT — durée de vie.** Non rouverte. Un échec de compilation de durée de vie du montage d'U4 (`ScriptTable<'a>` dans un vrai `World` sous un vrai `Scheduler`) est un **arrêt sans contournement** : ni `'static`, ni fuite de mémoire, ni `unsafe`, ni changement de la propriété des octets, ni modification de `crates/engine/schedule`. Le message exact du compilateur est rapporté.

**G-ORD-S — indépendance de l'ordre inter-instances.** Aucun test, exemple ou API n'affirme l'ordre des émissions entre instances ni l'ordre dense d'un stockage ; comparaisons en multiensemble, multiplicité conservée. Une exigence qui ne peut s'énoncer sans ordre est un arrêt.

**G-ST — aucun état hors World.** Arrêt si un besoin de `static`, `thread_local`, `Cell`/`RefCell`/atomique, `async`, `unsafe`, ou d'un état dans un système apparaît.

### 3.3 Layouts, API et valeurs provisoires [L]

**Valeurs provisoires** (plan § 6, C16) : `SCRIPT_REGISTERS = 4`, `SCRIPT_MAX_STEPS_PER_TICK = 16`, définies dans `verbum-assets` (type `usize`) ; **provisoires, propres à cette première implémentation, non invariants du moteur**. Les tests de budget construisent leurs programmes à partir des constantes ; seuls SCR-K1 (constantes) et ST-1 (longueur d'encodage) épinglent les valeurs.

**Constantes de layout de `verbum-assets`** : `SCRIPT_FORMAT_ID = *b"VSCR"`, `SCRIPT_FORMAT_VERSION = 1` (`u16`), `SCRIPT_FIXED_PAYLOAD_LEN = 2`, `SCRIPT_INSTRUCTION_LEN = 8`.

**Artefact** (little-endian, champ par champ, sans cast) : préfixe commun de 10 octets (identité 4, version `u16` 2, longueur de charge utile `u32` 4), puis charge utile : nombre d'instructions `n` (`u16`), puis `n` enregistrements de 8 octets : opcode `u8`, registre `u8`, `arg16` `u16`, `arg32` `u32`. Longueur déclarée attendue `2 + 8·n`.

| Opcode | Instruction | `registre` | `arg16` | `arg32` |
|---:|---|---|---|---|
| 0 | `Halt` | ignoré | ignoré | ignoré |
| 1 | `Emit` | ignoré | canal | valeur `i32` |
| 2 | `Wait` | ignoré | ignoré | ticks `u32` (≥ 1) |
| 3 | `Set` | registre (< `SCRIPT_REGISTERS`) | ignoré | valeur `i32` |
| 4 | `Add` | registre (< `SCRIPT_REGISTERS`) | ignoré | valeur `i32` |
| 5 | `Jump` | ignoré | cible (< `n`) | ignoré |
| 6 | `JumpIfNonZero` | registre (< `SCRIPT_REGISTERS`) | cible (< `n`) | ignoré |

Les octets ignorés ne sont pas contrôlés par le lecteur ; la Forge les écrit à zéro.

**Ordre des contrôles de `Script::read`** [Plan, comme `font.bin`] : (1) préfixe de 10 octets complet, sinon `Truncated` ; (2) identité ; (3) version ; (4) longueur déclarée inférieure à `SCRIPT_FIXED_PAYLOAD_LEN` : `PayloadLength` ; (5) octets restants inférieurs : `Truncated`, supérieurs : `TrailingBytes` ; (6) longueur déduite `2 + 8·n` calculée en `u64`, différente de la déclarée : `PayloadLength` ; (7) validation de chaque enregistrement dans l'ordre des positions : opcode inconnu, registre hors bornes (`Set`, `Add`, `JumpIfNonZero`), cible `≥ n` (`Jump`, `JumpIfNonZero`), `Wait` de zéro tick : `ReadError::Instruction`. Un script de zéro instruction (charge utile de 2 octets) est conforme ; une charge utile de longueur nulle est invalide (étape (4)).

**API de `verbum-assets`** [L] : `Script<'a>` (`Clone, Copy, Debug, PartialEq, Eq`) avec `read(&'a [u8]) -> Result<Self, ReadError>`, `len() -> u16`, `is_empty() -> bool`, `instruction(index: u16) -> Option<Instruction>` (décodage à la demande par `from_le_bytes`, `None` hors bornes, jamais de lecture au-delà de la charge validée) ; `Instruction` énumération `Copy` à sept variantes : `Halt`, `Emit { channel: u16, value: i32 }`, `Wait { ticks: u32 }`, `Set { register: u8, value: i32 }`, `Add { register: u8, value: i32 }`, `Jump { target: u16 }`, `JumpIfNonZero { register: u8, target: u16 }` ; `Format::Script`, `Asset::Script(Script<'a>)`, `ReadError::Instruction`, identité `VSCR` reconnue par `of_identity`.

**API de `verbum-script`** [L] : constantes `SCRIPT_REGISTERS`, `SCRIPT_MAX_STEPS_PER_TICK` réexportées ; `ENCODED_LEN = 10 + 4·SCRIPT_REGISTERS` ; `ScriptState` (`Clone, Copy, Debug, PartialEq, Eq`, **champs privés**) avec `start(SlotIndex) -> Result<Self, SlotTooLarge>`, accesseurs `script() -> u16`, `pc() -> u16`, `status() -> Status`, `wait() -> u32`, `registers() -> [i32; SCRIPT_REGISTERS]` (**retour par copie**, voir revue de signature en U3), `to_bytes() -> [u8; ENCODED_LEN]`, `from_bytes(&[u8; ENCODED_LEN]) -> Result<Self, StateError>` ; `Status` ∈ {`Running`, `Waiting`, `Halted`, `Faulted(Fault)`}, `Fault` ∈ {`NoScript`, `Budget`} ; `Emission { channel: u16, value: i32 }` (`Copy`) ; `Emissions<const CAP: usize>` (`new`, `push -> Result<(), Full>`, `as_slice`, `len`, `is_empty`, `capacity`, `clear` ; `Clone`, `Debug`, `PartialEq` manuel sur la tranche observable) ; `Full` ; `ScriptTable<'a, const CAP: usize>` (`new`, `deposit(SlotIndex, Script<'a>) -> Result<(), TableError>`, `get(u16) -> Option<&Script<'a>>` ; `TableError` ∈ {`OutOfRange`, `Occupied`} ; garde statique `CAP ≤ 65 536` par constante associée, motif de `entity.rs`) ; `emissions_bound_holds(instances: usize, cap: usize) -> bool` (`const fn`, `checked_mul`) ; `step<'a, const CAP: usize>(script: Option<&Script<'a>>, state: &mut ScriptState, sink: &mut Emissions<CAP>) -> Result<(), Full>`.

**Codes de l'encodage d'état** [L, E-4] : `status` : 0 `Running`, 1 `Waiting`, 2 `Halted`, 3 `Faulted` ; `fault` : 0 aucune, 1 `NoScript`, 2 `Budget`. Layout : `script` `u16` (0 à 1), `pc` `u16` (2 à 3), `status` (4), `fault` (5), `wait` `u32` (6 à 9), registres `i32` (10 à 25), tout en little-endian.

**Précision opérationnelle sur `pc`** [Exécution, E-2] : `Halt` **n'avance pas** `pc` (il reste sur l'instruction `Halt`) ; une fin de programme implicite laisse `pc = n`. Sans effet sur un invariant du plan ; épinglé par IN-5 et SR-3.

**Source Forge** [Plan § 6] : voir U2.

**Scénario de référence, source TOML** (littéral, recalculé) :

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

**Scénario de référence, artefact `script.bin`** (68 octets, littéral écrit à la main, indépendant des constantes) :

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

**Trajectoire de référence** (instance lancée au tick 1, script au slot 4) : tick 1 : `Set`, `Emit (1,10)`, `Wait 3` ; ticks 2 et 3 : attente ; tick 4 : reprise, `Add` (r0 = 2), `JumpIfNonZero` pris, `Emit (1,10)`, `Wait 3` ; ticks 5 et 6 : attente ; tick 7 : même tour (r0 = 1) ; ticks 8 et 9 : attente ; tick 10 : reprise, `Add` (r0 = 0), `JumpIfNonZero` non pris, `Emit (2,99)`, `Halt` ; ticks 11 et 12 : aucun changement. Émissions : tick 1, 4, 7 : (1,10) ; tick 10 : (2,99) ; aucune aux autres ticks. Coûts Forge : entrée 0 : 3 ; entrée 3 : 4. Encodage canonique en fin de tick, 26 octets chacun (les 12 derniers octets sont nuls pour `r1` à `r3`, `r0` figure aux octets 10 à 13) :

| Fin de tick | Octets 0 à 13 |
|---|---|
| 0 (état initial) | `04 00 00 00 00 00 00 00 00 00 00 00 00 00` |
| 1 | `04 00 03 00 01 00 03 00 00 00 03 00 00 00` |
| 2 | `04 00 03 00 01 00 02 00 00 00 03 00 00 00` |
| 3 | `04 00 03 00 01 00 01 00 00 00 03 00 00 00` |
| 4 | `04 00 03 00 01 00 03 00 00 00 02 00 00 00` |
| 10 (et 11, 12) | `04 00 06 00 02 00 00 00 00 00 00 00 00 00` |

**Second script de scène** (SR-5, `script B`) : `emit 3,7 ; wait 1 ; emit 3,8 ; halt` (quatre pas ; coûts Forge : entrée 0 : 2 ; entrée 2 : 2). Lancé **avant le tick 1** (variante ii) : émissions tick 1 : (3,7) ; tick 2 : (3,8) ; `Halted` au tick 2 (`pc = 3`). Lancé **après le tick 2** (variante i) : tick 3 : (3,7) ; tick 4 : (3,8). Transcripts de la variante (ii) en multiensemble : tick 1 {(1,10), (3,7)} ; tick 2 {(3,8)} ; ticks 4 et 7 {(1,10)} ; tick 10 {(2,99)} ; autres ticks vides. Variante (i) : tick 1 {(1,10)} ; tick 3 {(3,7)} ; tick 4 {(1,10), (3,8)} ; ticks 7 {(1,10)} ; tick 10 {(2,99)} ; autres ticks vides.

**Constantes de scène (assemblage d'U4 et U5)** : `ENTITIES = 4`, `SPARSE = 4`, `STATE_DENSE = 4` (**employées dans les paramètres génériques des `SparseSet`, ordre `SPARSE, DENSE`, et non seulement dans la `const` assert**), `EMISSIONS_CAP = 64`, `SCRIPT_SLOTS = 6` (table de formats `[Format::Script; 6]`), slot de référence **4** (artefact 0 d'un lot d'un seul script), scripts de SR-5 : A au slot 4 (artefact 0), B au slot 1 (artefact 1) ; `SYSTEMS = 4`, `PHASES = 2` ; catégories `STATE`, `SCRIPTS`, `EMISSIONS`, `TRANSCRIPT`, `BEAT` (entiers attribués par l'assemblage, `< 64`) ; `Beat(u32)` composant `Copy` ; `const _: () = assert!(emissions_bound_holds(STATE_DENSE, EMISSIONS_CAP))`.

## 4. Unités

**Dépendances réelles entre unités** : U1 (lecteur) ← U2 (la Forge construit des artefacts relus par `Script::read` et partage les constantes) ; U1 ← U3 (l'interpréteur pur emploie `Script`, `Instruction`, les constantes) ; U2 et U3 sont indépendants entre eux **mais l'ordre du plan est conservé** (U2 avant U3) ; U4 exige U1, U2 et U3 (compilation par la Forge, `step`, chargement) ; U5 exige U4 (assemblage partagé). Aucune unité n'exige un fichier d'une unité ultérieure.

**Dépendances Cargo par unité** : U1 : aucune modification de manifeste. U2 : aucune. U3 : racine (`members`, `[workspace.dependencies]`), nouveau `crates/script/Cargo.toml` (`verbum-assets.workspace = true`, `[lints] workspace = true`, aucune `[dev-dependencies]`). U4 : `crates/engine/shell/Cargo.toml` (`[dev-dependencies]` : `verbum-script.workspace = true`). U5 : aucune. `Cargo.lock` : par Cargo seulement.

### U1 — Lecteur `script.bin` dans `verbum-assets`

**Objectif [Plan § 8].** Ajouter le format de script au mécanisme de l'étape 5 sans le modifier : layout, lecteur à validation structurelle, `Format::Script`, `Asset::Script`, `ReadError::Instruction`, constantes partagées. Prouver la non-régression complète de l'étape 5.

**À fournir.** `docs/implementation/06-verbum-scripts.md`, `06-verbum-scripts-execution.md`, `05-verbum-assets-execution.md`, `tree.md`, `crates/engine/assets/Cargo.toml`, `crates/engine/assets/src/{lib,format,sprite,header,load,manifest,font}.rs` (`font.rs` : modèle de lecteur à charge variable ; `load.rs`, `manifest.rs` : lecture seule, jamais modifiés).

**Fichiers** [Exécution]. Créé : `crates/engine/assets/src/script.rs`. Modifiés : `crates/engine/assets/src/lib.rs` (module, exports), `crates/engine/assets/src/format.rs` (`Format::Script`, `Asset::Script`, identité `VSCR` dans `of_identity`, bras de `decode` appelant `Script::read` et produisant `Decoded::Ready`), `crates/engine/assets/src/sprite.rs` (variante `ReadError::Instruction`, **ajout seul** : aucune autre modification du fichier). `load.rs`, `manifest.rs`, `header.rs`, `Cargo.toml` : inchangés. **Recherche au démarrage** de tout `match` exhaustif sur `Format`, `Asset`, `ReadError` dans `crates/engine/assets`, `crates/engine/forge`, `crates/engine/shell` : seuls `format.rs::decode` et `batch.rs` (côté `AssetKind`, hors U1) sont attendus ; tout autre est exposé avant correction.

**Invariants visés.** X1, X10, X11, X12, X18 ; G-CM ; plan § 4.A, § 4.B, § 6.

**Contenu.**
1. `script.rs` : constantes (§ 3.3), `Instruction`, `Script`, lecteur selon l'ordre du § 3.3, validation des enregistrements sans allocation, décodage à la demande, garde de cohérence des constantes à la compilation (motif de `font.rs`), tests unitaires du module.
2. `format.rs`, `lib.rs`, `sprite.rs` : ajouts additifs.
3. Aucun changement de `load`, ni de ses niveaux, ni de `VMAN`.

**Tests** (les littéraux sont écrits à la main, indépendants des constantes de layout ; hors ST/IN, ils sont dans `script.rs` et dans les tests de `load.rs`… **sans modifier `load.rs`** : les tests SCR-L* vont dans un fichier de test de module de `script.rs` ou `format.rs`, jamais dans `load.rs`, § 9 E-1).

| Test | Énoncé | Porte d'arrêt |
|---|---|---|
| SCR-R1 | Artefact littéral de 68 octets (§ 3.3) : `len() == 7`, chaque instruction décodée égale à la table | oui : un octet divergent du littéral = arrêt |
| SCR-R2 | Chaque préfixe tronqué (0 à 67 octets) : `Truncated` | |
| SCR-R3 | Identité altérée octet par octet : `FormatIdentity` ; texte d'un `script.toml` de longueur suffisante : `FormatIdentity` | oui (T2) |
| SCR-R4 | Versions `0`, `2`, `0x0100`, `0xFFFF` : `Version` | |
| SCR-R5 | Longueur déclarée `< 2` : `PayloadLength` ; `>` octets restants : `Truncated` ; `<` octets restants (≥ 2) : `TrailingBytes` ; surplus d'un octet : `TrailingBytes` ; déclarée = restants mais ≠ `2 + 8·n` : `PayloadLength` | |
| SCR-R6 | Script de zéro instruction (charge de 2 octets) conforme (`len() == 0`, `is_empty()`, `instruction(0) == None`) ; charge de longueur nulle : `PayloadLength` | |
| SCR-R7 | `n = 0xFFFF` avec peu d'octets : rejeté sans panique ni débordement ; `n = 0x2000` avec charge de 2 octets : `PayloadLength` (un calcul de `8·n` sur 16 bits vaudrait 0) ; longueur déclarée `u32::MAX` avec peu d'octets : `Truncated` | oui |
| SCR-R8 | Un enregistrement invalide par cas, sur des positions distinctes : opcode inconnu (7 et 255), registre `SCRIPT_REGISTERS` pour `Set`, pour `Add`, pour `JumpIfNonZero`, cible `n` pour `Jump` et pour `JumpIfNonZero`, `Wait` de zéro tick : tous `Instruction` ; **non rejetés (champs ignorés : le lecteur ignore les opérandes inutilisés, la Forge encode à zéro)** : pour chaque opcode, **chaque champ ignoré** de la table du § 3.3 mis à une valeur non nulle (`0xFF` pour le registre, `0xFFFF` pour `arg16`, `0xFFFF_FFFF` pour `arg32`) : `Halt` (registre, `arg16`, `arg32`), `Emit` (registre), `Wait` (registre, `arg16`), `Set` et `Add` (`arg16`), `Jump` (registre, `arg32`), `JumpIfNonZero` (`arg32`) ; chaque enregistrement est accepté et l'instruction décodée est **égale** à celle du même enregistrement dont les champs ignorés sont nuls ; en outre cible `n − 1` valide et `Wait` de `u32::MAX` | oui (A-S3, C12) |
| SCR-R9 | `instruction(i)` : dernier indice accessible, `Some` ; premier inaccessible et `u16::MAX` : `None` ; script vide : `None` | |
| SCR-R10 | Script de cardinalité maximale construit en binaire : voir tableau natif/Miri ci-dessous | |
| SCR-K1 | `SCRIPT_REGISTERS == 4`, `SCRIPT_MAX_STEPS_PER_TICK == 16`, `SCRIPT_INSTRUCTION_LEN == 8`, `SCRIPT_FIXED_PAYLOAD_LEN == 2` ; message d'échec renvoyant au plan § 6 (valeurs provisoires) | |
| SCR-L1 | Avec une table de 6 formats, un artefact `Script` lié à un slot attendant `Sprite`, `Tileset`, `Map` ou `Font` : `LoadError::SlotFormat` du slot concerné, compteur de dépôts nul ; réciproquement un artefact de chacun des quatre autres formats lié à un slot attendant `Script` : `SlotFormat` ; **les cinq formats**, aucune paire diagonale | oui (X18) |
| SCR-L2 | Lot `{script, script}`, manifeste littéral avec `artifact_index ≠ slot_index` : deux dépôts, valeurs égales aux littéraux (`Asset::Script`), chaque slot reçoit son script | oui (C13) |
| SCR-L3 | Lot dont un artefact porte un `Wait` de zéro tick (puis, séparément, un registre hors bornes), les autres valides, **placé après** des artefacts valides : `LoadError::Artifact { error: ReadError::Instruction, .. }` avec l'index d'artefact attendu, **compteur de dépôts nul** | oui (G-CM : tout-ou-rien) |
| SCR-L4 | **Non-régression** : tous les tests de F14 passent, assertions inchangées | oui (G-CM) |

**Cardinalité maximale, natif et Miri** (SCR-R10 et SCR-R7).

| Test | Natif | Miri |
|---|---|---|
| SCR-R7 (`n = 0xFFFF`, `0x2000`, déclarée `u32::MAX` avec peu d'octets) | complet | complet (aucune boucle sur `n` : rejet par longueur avant validation des enregistrements) |
| SCR-R10 (script maximal accepté, un octet de moins : `Truncated`) | **65 535 instructions** construites en binaire par boucle (tableau d'octets de `10 + 2 + 8·65 535`) ; contenu valide : registres 0, cibles 0, `Wait 1` ; premier, milieu, dernier enregistrement décodés et comparés à des littéraux | **512 instructions** (`if cfg!(miri)`), mêmes assertions de premier, milieu et dernier ; la couverture du maximum exact reste native et n'est jamais réduite |

**Rouge.** Squelettes `unimplemented!()` : `Script::read` ; `Format::Script` et le bras de `decode` (appel à `Script::read`) ; SCR-R1 à SCR-R10, SCR-L1 à SCR-L3 échouent par panique (SCR-L1 échoue après avoir atteint le niveau 2, selon l'implémentation du squelette : le motif de panique est rapporté). **Sans rouge possible** : SCR-K1 (constantes déjà définies : sensibilité par mutation), SCR-L4 (tests existants), SCR-R7 sur le seul contrôle de longueur si déjà écrit au squelette (rapporté par test).

**Mutations** (copies isolées).

| # | Mutation | Détecteur | Autres assertions attendues en échec |
|---|---|---|---|
| M-1.1 | opcode inconnu décodé comme `Halt` au lieu d'être rejeté | SCR-R8 (cas opcode) | SCR-L3 si son artefact invalide utilise un opcode inconnu (non requis) |
| M-1.2 | registre non contrôlé pour `Set` (puis pour `Add`, puis pour `JumpIfNonZero`, trois copies) | SCR-R8 (cas du registre correspondant) | — |
| M-1.3 | cible non contrôlée pour `Jump` (puis pour `JumpIfNonZero`) | SCR-R8 | — |
| M-1.4 | `Wait` de zéro accepté | SCR-R8 (cas `Wait 0`) | SCR-L3 (cas `Wait 0`) |
| M-1.5 | registre contrôlé aussi pour `Emit`, `Jump`, `Wait`, `Halt` (plus strict) | SCR-R8 (partie « champs ignorés », registre) | SCR-R1 si le littéral porte un registre non nul sur un opcode ignorant (non requis) |
| M-1.5b | lecteur plus strict : un `arg16` ou un `arg32` non nul sur un opcode qui l'ignore est rejeté (`Instruction`) | SCR-R8 (partie « champs ignorés », `arg16` et `arg32`) | — |
| M-1.5c | le décodage laisse un champ ignoré influencer le résultat (par exemple la cible de `Jump` composée de `arg16` et des deux octets bas de `arg32`) | SCR-R8 (égalité avec l'enregistrement aux champs ignorés nuls) | — |
| M-1.6 | `arg32` lu en big-endian | SCR-R1 | SCR-R9 |
| M-1.7 | `arg16` lu en big-endian | SCR-R1 | — |
| M-1.8 | longueur déduite calculée sur 16 bits (`8·n` réduit modulo 2¹⁶) | SCR-R7 (cas `0x2000`) | — |
| M-1.9 | longueur déduite en `u32` | **(équivalente)** : `n ≤ 0xFFFF` donne `2 + 8·n ≤ 524 282 < 2³²` ; aucun test artificiel | — |
| M-1.10 | `instruction(index)` accepte `index == n` par garde décalée, l'accès restant par `get` | **(équivalente)** : `get` borné rend déjà `None` pour `index == n` ; aucun comportement observable ne change ; aucun test artificiel | — |
| M-1.10′ | variante de M-1.10 : garde décalée **et** indexation directe `records[offset..offset+8]` | SCR-R9 (panique rapportée comme telle) | SCR-R6 |
| M-1.11 | `of_identity` ne reconnaît pas `VSCR` | SCR-L1 (le script lié à un slot non script remonte `Artifact { FormatIdentity }` au lieu de `SlotFormat`) | SCR-L2 inchangé (le format attendu le sauve) |
| M-1.12 | `decode` envoie `Script` au lecteur de `Font` | SCR-L2 | SCR-L3 |
| M-1.13 | `Script::read` ne valide aucun enregistrement | SCR-R8 | SCR-L3 |
| M-1.14 | `SCRIPT_MAX_STEPS_PER_TICK = 17` | SCR-K1 **seul** (les tests paramétrés suivent la constante) | — |
| M-1.15 | `ReadError::Instruction` renvoyé par la validation avant la vérification de longueur | SCR-R5 (ordre documenté) | SCR-R7 |

**Contrôles propres.** `cargo tree -p verbum-assets` (aucune dépendance, dev comprises) ; revue : aucun `alloc`, aucun `unsafe`, aucune chaîne ; `git diff` de `load.rs`, `manifest.rs`, `header.rs` vide ; `sprite.rs` : seul l'ajout de la variante ; `Cargo.toml` inchangé.

**Contrôles natifs et Miri (utilisateur).** Natif : quatre contrôles ; `cargo test -p verbum-assets`. Miri : `cargo +nightly miri test -p verbum-assets` (suite complète, sans `--skip` ; SCR-R10 réduit à 512 sous Miri).

**Résultats attendus.** Rouge constaté (§ ci-dessus) ; vert ; 18 mutations traitées (M-1.1 à M-1.15, plus M-1.5b, M-1.5c et M-1.10′ ; M-1.9 et M-1.10 équivalentes rapportées comme telles, M-1.10′ détectée) ; quatre contrôles verts ; Miri vert.

**Critère de sortie.** SCR-R*, SCR-K1, SCR-L1 à SCR-L4 verts ; contrôles propres conformes ; écarts de noms listés. « En attente de la validation de l'unité U1. »

### U2 — Compilation d'un script dans `verbum-forge`

**Objectif [Plan § 8].** Source TOML → artefact `script.bin` ; résolution des étiquettes ; validations de la Forge (opérandes, terminaison, budget) ; intégration au lot par `AssetKind::Script` ; chaîne `compile_assets → load` sur un lot mixte.

**À fournir.** Fichiers de la session, plus : les fichiers d'U1 tels que rendus (`crates/engine/assets/src/{script,format,sprite,lib}.rs`), `crates/engine/forge/Cargo.toml`, `crates/engine/forge/src/{lib,batch,sprite,map,id,font,tileset}.rs` (`font.rs` et `tileset.rs` : modèles non lus), `crates/engine/forge/tests/pipeline.rs`.

**Fichiers** [Exécution]. Créé : `crates/engine/forge/src/script.rs`. Modifiés : `crates/engine/forge/src/lib.rs` (`mod script`, `pub use script::compile_script`), `crates/engine/forge/src/batch.rs` (`AssetKind::Script`, bras `AssetKind::Script => compile_script(source.toml).map(Pending::Ready)`), `crates/engine/forge/src/sprite.rs` (variantes de `ForgeError`, **ajouts seuls**), `crates/engine/forge/tests/pipeline.rs` (extension : SCR-C1 ; adaptation R-ADAPT si un `match` exhaustif est découvert). `Cargo.toml` inchangé.

**Invariants visés.** X1, X7, X10, X11, X12 ; T1, T10, T11 ; plan § 4.D (convention d'analyse), § 6 (source).

**Source Forge** [Plan § 6, précisions d'exécution] : un fichier TOML par script ; clé de premier niveau unique `step` (tableau de tables) ; tout autre champ de premier niveau : `UnknownField`. Par pas : champs permis = `op`, `label` (optionnel) et les opérandes de l'opération ; tout autre : `UnknownField`. Opérandes : `emit` : `channel` (entier `0..=65535`, zéro admis), `value` (`i32`) ; `wait` : `ticks` (`1..=u32::MAX`) ; `set`, `add` : `register` (`0..SCRIPT_REGISTERS`, zéro admis), `value` (`i32`) ; `jump` : `target` (chaîne) ; `jnz` : `register`, `target` ; `halt` : aucun. `label` : chaîne non vide, unique.

**Ordre des contrôles de `compile_script`** [Exécution] : (1) syntaxe (`Syntax`) ; (2) champ inconnu de premier niveau ; (3) `step` présent (`MissingField("step")`) et tableau (`WrongType("step")`) ; (4) nombre de pas par `checked_step_count` (0 : `EmptyScript` ; > 65 535 : `TooManySteps`) ; (5) pour chaque pas dans l'ordre : table (`WrongType("step")`), `op` présent (`MissingField("op")`) et chaîne (`WrongType("op")`), opération connue (`UnknownOperation(nom)`), champs inconnus du pas (`UnknownField(nom)`), `label` vide (`EmptyLabel { step }`) ou dupliqué (`DuplicateLabel(nom)`), opérandes dans l'ordre listé (présence, type, intervalle ; `ticks` zéro : `ZeroValue("ticks")`, négatif ou `> u32::MAX` : `OutOfRange("ticks")`) ; (6) résolution des cibles (`UnknownTarget(nom)`) ; (7) `check_flow` (cycle sans yield : `CycleWithoutYield`, puis budget : `BudgetExceeded { entry }`) ; (8) encodage. Aucun artefact n'est produit en cas d'erreur.

**Variantes de `ForgeError` ajoutées** [L] : `UnknownOperation(String)`, `EmptyScript`, `TooManySteps`, `DuplicateLabel(String)`, `EmptyLabel { step: usize }`, `UnknownTarget(String)`, `CycleWithoutYield`, `BudgetExceeded { entry: usize }`.

**Structure interne** [Exécution] : `compile_script` produit d'abord un `Vec<verbum_assets::Instruction>` (cibles résolues en positions), puis appelle `check_flow(&[Instruction]) -> Result<(), ForgeError>` (`pub(crate)`), puis encode ; `checked_step_count(len: usize) -> Result<u16, ForgeError>` (`pub(crate)`, **réellement employé par `compile_script`**). `check_flow` ne reçoit **aucun texte** : il est testable sur des instructions construites en mémoire, sans parsing TOML.

**Algorithme de `check_flow`** [Plan § 4.D ; interprétation plus stricte, E-11, soumise et acceptée] : graphe sur les `n` instructions et un nœud virtuel `FIN` ; successeurs : `Halt`, `Wait` : aucun ; `Jump` : sa cible ; `JumpIfNonZero` : sa cible **et** `pc + 1` ; autres : `pc + 1` ; `pc + 1 == n` désigne `FIN`. (1) **Détection des cycles sans yield** par parcours en profondeur **itératif** (pile explicite, trois couleurs) lancé depuis **chaque nœud non encore visité de `0..n`**, accessible ou non : tout cycle ⇒ `CycleWithoutYield`. (2) **Coût maximal de chaque nœud**, calculé une fois les cycles écartés : `coût(Halt) = coût(Wait) = 1`, `coût(FIN) = 0`, sinon `1 + max` des coûts des successeurs. (3) **Budget sur tous les nœuds** : pour chaque nœud `0..n` par position croissante, si `coût(nœud) > SCRIPT_MAX_STEPS_PER_TICK` ⇒ `BudgetExceeded { entry: nœud }`, le **premier nœud en dépassement** étant celui rapporté ; la vérification porte sur les nœuds accessibles **et** inaccessibles. Les « entrées d'un tour » du plan (instruction 0 et instructions de reprise *i + 1* après un `Wait`) ne forment pas une liste distincte : ce sont des nœuds, couverts par cette vérification ; une reprise qui désigne `FIN` n'ajoute aucun nœud (coût 0). Un nœud accessible a un coût inférieur ou égal à celui de l'entrée de son tour : la vérification de tous les nœuds subsume celle des entrées et couvre en plus le code inaccessible (E-11). **Ordre de priorité** : tous les cycles sont écartés avant tout calcul de coût, donc `CycleWithoutYield` l'emporte sur `BudgetExceeded`, y compris pour un code inaccessible. **Aucune récursion** (script jusqu'à 65 535 pas : une récursion peut dépasser la pile).

**Tests.**

| Test | Énoncé | Porte d'arrêt |
|---|---|---|
| SCR-F1 | La source du scénario de référence (§ 3.3) produit **exactement** les 68 octets littéraux ; variante : même script sans étiquette, cible exprimée par la seule étiquette du pas 1 | oui : octets divergents = arrêt |
| SCR-F2 | La sortie de SCR-F1 est relue par `Script::read` ; instructions égales à la table ; relecture des scripts de refus de SR-5 (script B) | |
| SCR-F3 | Refus, un cas chacun, erreur exacte : `step` absent ; `step` non tableau ; tableau vide (`EmptyScript`) ; `op` absent, non chaîne, inconnu ; champ inconnu (par opération, et de premier niveau) ; opérande manquant (par opération) ; mauvais type (par opération) ; `register` `SCRIPT_REGISTERS` et négatif (`OutOfRange`), `0` accepté ; `value` hors `i32` ; `channel` négatif et `65536` (`OutOfRange`), `0` et `65535` acceptés ; **`ticks` `0` (`ZeroValue`), `-1` (`OutOfRange`), `4294967296` (`OutOfRange`)**, `1` et `4294967295` acceptés ; `label` vide ; `label` dupliqué ; `target` inconnu ; TOML invalide | oui (T11) |
| SCR-F4 | **Terminaison et budget**, sur `check_flow` (instructions construites en mémoire, **tous modes**) et sur `compile_script` (sources TOML **générées par code à partir des constantes**, natif seulement pour les gros programmes, voir tableau). (a) cycle sans yield refusé : `Jump` vers soi, saut arrière direct, `JumpIfNonZero` seul vers un pas antérieur ; cycle contenant un `Wait` accepté ; (b) **chemin linéaire de exactement `SCRIPT_MAX_STEPS_PER_TICK` instructions, `Wait` ou `Halt` final compris, accepté**, sous deux formes : depuis l'entrée 0, et depuis une **entrée de reprise** (un `Wait` initial court, puis un segment de la longueur limite) ; (c) **chemin de `SCRIPT_MAX_STEPS_PER_TICK + 1` refusé**, sous les deux mêmes formes, avec `entry` exact (0, puis la position de reprise, premier nœud en dépassement) ; **cas de deux nœuds en dépassement** : deux segments de exactement `SCRIPT_MAX_STEPS_PER_TICK + 1` instructions chacun (`Wait` ou `Halt` final compris), séparés par un `Wait` ; seuls le nœud 0 et le nœud de reprise dépassent ; `entry` = **le nœud 0 (le premier)**, et le test vérifie que la valeur rapportée n'est pas celle du second ; (d) chemin de `SCRIPT_MAX_STEPS_PER_TICK` instructions suivi de la fin du programme accepté, de `+ 1` refusé ; (e) `JumpIfNonZero` : **bras cible seul trop long** (bras `pc + 1` court) refusé ; (f) `JumpIfNonZero` : **bras `pc + 1` seul trop long** (bras cible court) refusé ; (g) branchement dont le plus long chemin dépasse la limite alors que le plus court la respecte : refusé ; (h) les deux bras à la limite exacte : accepté ; (i) plus de `SCRIPT_MAX_STEPS_PER_TICK` instructions au total avec des `Wait` placés de sorte que chaque segment tienne : accepté ; (j) entrée de reprise placée à la fin du programme (coût 0) acceptée ; (k) **code inaccessible**, deux sous-cas **distincts** qui doivent produire **deux erreurs distinctes** : **(k1) portion inaccessible acyclique dépassant le budget** (par exemple un `Halt` en position 0, puis `SCRIPT_MAX_STEPS_PER_TICK + 1` `Add` jamais atteints) : `BudgetExceeded { entry }` avec `entry` = position du premier nœud inaccessible en dépassement (ici 1), **et non** `CycleWithoutYield` ; **(k2) portion inaccessible cyclique sans yield** (par exemple un `Halt` en position 0, puis un `Jump` vers soi en position 1) : `CycleWithoutYield`, **et non** `BudgetExceeded` ; en outre, (k1) accepté si la portion inaccessible tient exactement dans la limite (`SCRIPT_MAX_STEPS_PER_TICK` instructions), et (k2) accepté si le cycle inaccessible contient un `Wait` ; (l) priorité : un script cyclique **et** trop long donne `CycleWithoutYield`, y compris lorsque le cycle est dans une portion inaccessible et le dépassement dans une portion accessible (et réciproquement) | oui (T10, T11) |
| SCR-F5 | Déterminisme : deux compilations de la même source donnent les mêmes octets ; invariance de la sortie d'un lot `{script, script}` par permutation de l'entrée (seuls les `artifact_index` changent) | |
| SCR-F6 | Aucune chaîne dans la sortie : ni l'étiquette `loop`, ni les noms d'opération (`emit`, `wait`, …) ne figurent dans les 68 octets (recherche de sous-chaîne) | oui (X10) |
| SCR-F7 | Limite des 65 535 pas : voir tableau natif/Miri | |
| SCR-C1 | **Lot mixte** `compile_assets` → `load` : sprite, tileset, map, font et **deux scripts** en slots non contigus (par exemple 1 et 4) avec `artifact_index ≠ slot_index` pour chaque entrée ; capacité 6 ; table de formats `[Sprite, Script, Tileset, Font, Script, Map]` ou équivalent respectant les genres ; **cinq dépôts** ; un `ScriptTable`-témoin de test (tableau `[Option<Script>; 6]` local, **pas** le type de `verbum-script` : `verbum-forge` ne dépend pas de `verbum-script` et n'en a aucune dev-dépendance, § 2 ; le témoin ne prouve que la correspondance slot ↔ indice du chargement, la table réelle étant exercée par TB-1 en U3 et SR-5 en U4) reçoit les scripts aux indices de **slot** ; les autres indices restent vides ; aucun changement de mécanisme (SCR-L4 revérifié) | oui (G-CM, C13) |

**Cardinalité maximale, natif et Miri** (SCR-F4 grands programmes, SCR-F7).

| Test | Natif | Miri |
|---|---|---|
| `checked_step_count` : 0 (`EmptyScript`), 1, 65 535 (acceptés), 65 536 et `usize::MAX` (`TooManySteps`) | complet | complet (aucun parsing) |
| `check_flow` sur 65 535 instructions construites en mémoire : motif « 15 `Add` puis un `Wait` » répété, terminé par 15 `Add` : **accepté** ; 65 535 `Add` consécutifs : `BudgetExceeded { entry: 0 }` ; chaîne de 65 535 `Jump` vers le suivant puis vers soi : `CycleWithoutYield` (démontre l'absence de récursion) | complet | **512 instructions** (`if cfg!(miri)`), mêmes motifs |
| `compile_script` de bout en bout : 65 535 pas TOML (motif « 15 `add` puis un `wait` », terminé par 15 `add`) accepté ; 65 536 pas : `TooManySteps` | **natif seulement** (`if !cfg!(miri)`) | non exécuté (coût du parsing, E-13) |
| `compile_script` : refus et acceptation à la limite du budget (SCR-F4, programmes de `SCRIPT_MAX_STEPS_PER_TICK` et `+ 1` pas TOML) | complet | **réduit** : seuls un programme de limite acceptée et un programme `+ 1` refusé (une forme chacun, entrée 0) ; les autres formes, dont (k1) et (k2), passent par `check_flow` direct |
| Autres tests de `compile_script` (SCR-F1 à SCR-F3, SCR-F5, SCR-F6, SCR-C1) | complet | complet (sources de ≤ 7 pas) |

Le raccordement de `check_flow` à `compile_script` (appel effectif) est établi **nativement** par les programmes TOML de limite et par les mutations M-2.14b et M-2.15 ; sous Miri il n'est établi que par les deux programmes réduits ci-dessus. Statut rapporté comme partiel, précédent E-13.

**Rouge.** Squelettes `unimplemented!()` : `compile_script`, `check_flow`, `checked_step_count`, et le bras `AssetKind::Script`. SCR-F1 à SCR-F7 et SCR-C1 échouent par panique. **Sans rouge possible** : néant, sauf les vérifications de non-régression.

**Mutations** (copies isolées ; équivalentes marquées).

| # | Mutation | Détecteur | Autres assertions attendues en échec |
|---|---|---|---|
| M-2.1 | `ticks` zéro non refusé | SCR-F3 | — |
| M-2.2 | `ticks` négatif converti par `as u32` | SCR-F3 (`-1`) | — |
| M-2.3 | étiquette résolue en position + 1 | SCR-F1 | SCR-F2, SCR-C1 si le lot exécute |
| M-2.4 | `value` négative écrite en valeur absolue | SCR-F1 (`Add −1`) | SCR-F2 |
| M-2.5 | intervalle de registre `<= SCRIPT_REGISTERS` | SCR-F3 | — |
| M-2.6 | `channel` écrit en big-endian | SCR-F1 | SCR-F2 |
| M-2.7 | octets d'opérande ignorés écrits non nuls | SCR-F1 | SCR-F6 non |
| M-2.8 | détection de cycle supprimée | SCR-F4 (a) | SCR-F4 (k2), SCR-F4 (l) |
| M-2.9 | détection ne suivant pas l'arête de `JumpIfNonZero` vers sa cible | SCR-F4 (a) (cas `jnz` seul) | — |
| M-2.10 | cycle compté même à travers un `Wait` (arêtes des `Wait` non coupées) | SCR-F1 (la boucle de référence est refusée) | SCR-F4 (a) (cycle avec `Wait` accepté), SCR-F4 (k2) (cycle inaccessible avec `Wait` accepté), SCR-C1 |
| M-2.11 | coût de `Wait` mis à zéro | SCR-F4 (c) (chemin de `+ 1` terminé par `Wait` accepté à tort) | SCR-F4 (b)/(d) selon la forme |
| M-2.12 | coût de `JumpIfNonZero` ignorant le bras `pc + 1` | SCR-F4 (f) | SCR-F4 (g) |
| M-2.13 | coût de `JumpIfNonZero` ignorant le bras cible | SCR-F4 (e) | SCR-F4 (g) |
| M-2.14 | **liste des entrées de reprise supprimée** : la vérification du budget ne passe plus par une liste d'entrées (instruction 0 et nœuds *i + 1* après un `Wait`) | **(équivalente)** : `check_flow` vérifie le coût de **chaque** nœud de `0..n` ; l'ensemble des entrées d'un tour est contenu dans cet ensemble (instruction 0, nœuds de reprise, ou `FIN` de coût 0) ; la liste est donc redondante par construction, et sa suppression ne change aucun comportement observable. Aucun test artificiel. La couverture du risque réel (une reprise non vérifiée) est portée par M-2.14b | — |
| M-2.14b | **nœuds de reprise exemptés** : la vérification du budget saute les nœuds situés immédiatement après un `Wait` (la boucle sur `0..n` reste, mais ces nœuds sont ignorés) | SCR-F4 (c), forme de reprise (`entry` = position de reprise) | — |
| M-2.14c | **budget limité au nœud 0** : seul le coût du nœud 0 est comparé à la limite | SCR-F4 (c), forme de reprise ; SCR-F4 (k1) | SCR-F4 (e), (f) si le bras long n'est pas atteint depuis 0 par le chemin testé |
| M-2.15 | `check_flow` non appelé par `compile_script` | SCR-F4 (c) en bout en bout natif (+ cycle) | **non détectée sous Miri seul** : rapporté |
| M-2.16 | **budget limité aux nœuds accessibles** (depuis le nœud 0 et les nœuds de reprise) : les nœuds inaccessibles ne sont pas comparés à la limite | SCR-F4 (k1) (`BudgetExceeded` attendu, script accepté à tort) | — |
| M-2.16b | **cycles recherchés seulement depuis les nœuds accessibles** : le parcours en profondeur n'est lancé que depuis le nœud 0 et les nœuds de reprise | SCR-F4 (k2) (`CycleWithoutYield` attendu, script accepté à tort) | SCR-F4 (l) (cas du cycle inaccessible) |
| M-2.16c | **priorité inversée** : le budget est évalué avant la détection des cycles et rapporté en premier | SCR-F4 (l) | SCR-F4 (k2) si la portion cyclique dépasse aussi la limite |
| M-2.17 | seuil du budget `>= SCRIPT_MAX_STEPS_PER_TICK` au lieu de `>` | SCR-F4 (b) | SCR-F1 si le scénario atteint la limite (non : coûts 3 et 4) |
| M-2.18 | seuil du budget `> SCRIPT_MAX_STEPS_PER_TICK + 1` | SCR-F4 (c) | — |
| M-2.19 | `AssetKind::Script` aiguillé vers `compile_font` | SCR-C1 | SCR-F5 (lot) |
| M-2.20 | `checked_step_count` borne `65 536` | SCR-F7 (assistant direct) | — |
| M-2.21 | l'étiquette ou le nom d'opération écrit dans l'artefact | SCR-F6 | SCR-F1 |
| M-2.22 | `UnknownTarget` non détecté (cible résolue en 0) | SCR-F3 | — |
| M-2.23 | `EmptyLabel` non détecté | SCR-F3 | — |
| M-2.24 | `BudgetExceeded` renvoyant le dernier nœud en dépassement au lieu du premier | SCR-F4 (c), cas de deux nœuds en dépassement (`entry` attendu = 0, la mutation rapporte la position de reprise) | — |

**Contrôles propres.** `cargo tree -p verbum-forge` (dépendances : `verbum-assets`, `toml` ; dev : `verbum-ecs`) ; `cargo tree -i toml --workspace` (seul `verbum-forge`) ; `cargo tree -i verbum-forge --workspace -e normal` (aucun dépendant) ; `cargo tree -p verbum-assets` (vide) ; revue : aucune récursion dans `check_flow`, aucun accès disque ; revue : `check_flow` ne parcourt pas une liste d'entrées distincte, la boucle de budget porte sur `0..n`.

**Contrôles natifs et Miri (utilisateur).** Natif : quatre contrôles ; `cargo test -p verbum-forge`. Miri : `cargo +nightly miri test -p verbum-forge -- --skip p3_1 --skip p3_2 --skip t6_1 --skip t6_3 --skip t6_6` (exclusions existantes de l'étape 5, F13) ; **Miri partiel de la Forge, non intégral**, rapporté comme tel. Miri sur `verbum-assets` revérifié (suite complète).

**Critère de sortie.** SCR-F1 à SCR-F7, SCR-C1 verts ; mutations traitées (M-2.14 équivalente rapportée comme telle) ; quatre contrôles verts ; Miri conforme (partiel Forge). « En attente de la validation de l'unité U2. »

### U3 — Crate `verbum-script` : état, émissions, table, `step`

**Objectif [Plan § 8].** Créer le crate d'exécution pur : encodage canonique, états valides par construction, accumulateur d'émissions, table indexée par slot, borne des émissions, fonction `step`. Aucune dépendance au `World` ni au Scheduler.

**À fournir.** Fichiers de la session, plus : les fichiers d'U1 et d'U2 tels que rendus (au minimum `crates/engine/assets/src/{script,lib}.rs`), `Cargo.toml` racine, `crates/engine/render/Cargo.toml` (modèle de manifeste), `crates/engine/render/src/{lib,accumulator}.rs` (modèle de l'accumulateur à capacité fixe ; fichiers non lus), `crates/engine/ecs/src/entity.rs` (motif de garde statique `CAPACITY_FITS_INDEX`, lu).

**Fichiers** [Exécution]. Créés : `crates/script/Cargo.toml` (package `verbum-script`, champs hérités du workspace, `description` anglaise du type « Script runtime of the Verbum engine: execution state and pure interpreter step, no_std and allocation-free. », `[dependencies]` : `verbum-assets.workspace = true`, **aucune `[dev-dependencies]`**, `[lints] workspace = true`), `crates/script/src/{lib,state,emissions,table,step}.rs`. Modifié : `Cargo.toml` racine (`"crates/script"` ajouté à `members` après `crates/engine/render` ; `verbum-script = { path = "crates/script", version = "0.1.0" }` ajouté à `[workspace.dependencies]` ; rien d'autre). `Cargo.lock` : par Cargo.

**Invariants visés.** X3 à X6, X8, X9, X12 à X14 ; T4 à T6, T9, T10, T14 ; plan § 4.C, § 4.D, § 4.E.

**Contenu.**
1. `state.rs` : `ScriptState` (champs privés), `Status`, `Fault`, `SlotTooLarge`, `StateError`, `ENCODED_LEN`, `to_bytes`, `from_bytes` selon la table de validité du plan § 4.C : `Running` (fault 0, wait 0), `Waiting` (fault 0, wait ≥ 1), `Halted` (fault 0, wait 0), `Faulted(NoScript)` (fault 1, wait 0), `Faulted(Budget)` (fault 2, wait 0) ; tout le reste est `StateError`.
2. `emissions.rs` : `Emission`, `Emissions<CAP>`, `Full`, `emissions_bound_holds`.
3. `table.rs` : `ScriptTable<'a, CAP>`, `TableError`, garde statique `CAP ≤ 65 536`.
4. `step.rs` : `step` selon le plan § 4.D, avec la définition de budget **sans décalage** (recherche de l'instruction, puis test du budget, puis exécution) et la précision de `pc` du § 3.3 ; `Wait` décrément saturant ; arithmétique `wrapping_add` ; registre invalide inatteignable (`unreachable!` avec message, jamais de panique sur artefact chargé).
5. `lib.rs` : `#![no_std]`, modules, exports, réexport des constantes de `verbum-assets`.

**Tests** (littéraux écrits à la main ; binaires de script construits par code dans des tableaux d'octets de taille constante, **sans `Vec`** : le crate est `no_std` sans `alloc`).

| Test | Énoncé | Porte d'arrêt |
|---|---|---|
| ST-1 | `ENCODED_LEN == 10 + 4·SCRIPT_REGISTERS == 26` (épinglé) ; `to_bytes` des états de la trajectoire de référence (§ 3.3, fins de tick 0, 1, 2, 3, 4, 10) égal aux littéraux, 26 octets chacun | oui (T4) |
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
| IN-miss | Script absent (`None`), puis slot vacant, **slot occupé dans le manifeste par un asset d'un autre format** (représenté par `None` dans la `ScriptTable` : cette couche ne connaît pas les autres variantes d'`Asset` et **ne reconnaît aucun autre format** ; en U3 le cas est simulé par un slot sans dépôt, par exemple 2, alors que des scripts sont déposés aux slots 1 et 4 ; la preuve du lot mixte relève de SCR-C1 (U2), qui emploie un **tableau témoin local** et non la `ScriptTable` réelle, voir § 10 point 10) et slot hors `CAP` d'une `ScriptTable` : `Faulted(NoScript)`, sans panique ; une instance en `Waiting` dont le script manque se comporte comme `step` le prévoit (la faute survient à la reprise) | oui (T14) |
| IN-full | `step` sur un sink de capacité 1 : première `Emit` acceptée, seconde `Err(Full)`, `pc` non avancé sur la seconde, état et émission déjà acceptée conservés ; **propriété de la fonction pure** : la branche est inatteignable en configuration conforme (U4) | |
| IN-det | Même entrée, même sortie, deux appels (états, émissions) | |
| EM-1 | `Emissions` : acceptation jusqu'à `CAP`, `Err(Full)` au-delà sans modification, ordre conservé, `clear`, `CAP = 0` refuse tout, `PartialEq` sur la tranche observable | |
| EM-2 | `emissions_bound_holds(4, 64)` vrai (à la borne exacte), `(4, 63)` faux, `(0, 0)` vrai (**cas dégénéré** : aucune instance, capacité nulle), **`(usize::MAX, usize::MAX)` faux (débordement du produit)** | oui (T10) |
| EM-3 | `same_capacity(&Emissions<N>, &Emissions<N>)` accepte (accumulateur, transcript) ; `Clone` d'un accumulateur plein donne une valeur égale, sans insertion élément par élément | |
| TB-1 | `ScriptTable<'_, 6>` : dépôt aux slots 4 et 1 de scripts distincts ; `get(4)`, `get(1)` rendent leurs scripts ; `get(0)`, `get(2)`, `get(3)`, `get(5)` : `None` ; dépôt au slot 6 (`OutOfRange`) et au slot 4 une seconde fois (`Occupied`) refusés ; `get(6)` et `get(u16::MAX)` : `None` ; garde statique `CAP ≤ 65 536` vérifiée **en contexte de compilation** par l'évaluation de la constante associée de garde pour `CAP = 65 536` dans un `const _: () = …` (accepté) ; **aucune `ScriptTable<'_, 65 536>` n'est instanciée sur la pile** ; le refus de `CAP = 65 537` (échec de compilation) n'est pas testé | oui (T14, C13) |

**Rouge.** Squelettes `unimplemented!()` des fonctions et méthodes ; ST-1 à ST-4, IN-*, EM-*, TB-1 échouent par panique. **Sans rouge possible** : néant.

**Mutations** (copies isolées).

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

**Ancienne M-3.29 retirée des mutations.** Le changement de `ScriptState::registers()` d'un retour par copie (`[i32; SCRIPT_REGISTERS]`) à un retour par référence n'est pas une mutation de comportement équivalente : les deux signatures imposent des contraintes d'emprunt et de durée de vie différentes à l'appelant (une référence maintient un emprunt de l'état, interdit toute modification de l'état tant qu'elle vit, et change le type des expressions qui l'utilisent). Ce point est classé comme **conformité de la signature publique**, vérifié par la revue de l'API ci-dessous. **Aucun test artificiel n'est ajouté** et aucune mutation n'est rapportée pour lui.

**Contrôles propres.** `cargo tree -p verbum-script -e normal` (seule dépendance : `verbum-assets`) ; `cargo tree -p verbum-script` (idem, aucune dev) ; `cargo tree -p verbum-assets` (vide) ; `cargo tree -i verbum-script --workspace` (aucun dépendant à ce stade) ; **revue G-ST** : recherche textuelle dans `crates/script/**` de `static`, `thread_local`, `Cell`, `RefCell`, `Atomic`, `async`, `await`, `thread`, `Box`, `Vec`, `unsafe`, `dyn`, résultat rapporté (aucune occurrence admise hors tests pour les littéraux de tableaux) ; revue : `step` n'a aucun paramètre autre que `(script, état, sink)` ; revue : `ScriptState` sans référence. **Revue de conformité de la signature publique et de l'API** (anciennement M-3.29), point par point, rapportée : (a) `ScriptState::registers()` rend `[i32; SCRIPT_REGISTERS]` **par copie**, conformément au § 3.3 ; (b) les accesseurs `script()`, `pc()`, `status()`, `wait()` rendent des valeurs `Copy` par copie ; (c) aucun accesseur de `ScriptState` ne rend de référence ni de `&mut` sur l'état interne (champs privés, transition unique par `step`) ; (d) `ScriptTable::get` rend `Option<&Script<'a>>` conformément au § 3.3 ; (e) la liste des signatures publiques de `verbum-script` est comparée à celle du § 3.3, tout écart étant listé au rapport (règle 9 du § 1).

**Contrôles natifs et Miri (utilisateur).** Natif : quatre contrôles ; `cargo test -p verbum-script`. Miri : `cargo +nightly miri test -p verbum-script` (suite complète : tous les tests de cette unité emploient des tableaux de quelques centaines d'octets ; aucune réduction prévue ; si la durée dépasse 10 minutes, rapporter et arbitrer, ne pas désactiver l'isolation).

**Critère de sortie.** ST-*, IN-*, EM-*, TB-1 verts ; mutations traitées (M-3.28 équivalente) ; revue G-ST et revue de la signature publique rapportées ; quatre contrôles verts ; Miri vert. « En attente de la validation de l'unité U3. »

### U4 — Intégration Scheduler, interface fixe et test d'invariant de reprise

**Objectif [Plan § 8].** Assembler `World`, `Scheduler`, systèmes d'assemblage et interpréteur ; prouver l'interface fixe (CT), la compatibilité avec `ScriptTable<'a>` empruntant des octets (SY-0), la suspension et la reprise (SR-1 à SR-7).

**À fournir.** Fichiers de la session, plus : fichiers des unités précédentes tels que rendus (`crates/engine/assets/src/{script,format,lib}.rs`, `crates/engine/forge/src/{lib,batch,script}.rs`, `crates/script/**`), `Cargo.toml` racine, `crates/engine/shell/Cargo.toml`, `crates/engine/shell/tests/{level_loaded,borrowed_resource}.rs` (modèle), `crates/engine/schedule/src/{system,access,data,contract,model,scheduler}.rs`, `crates/engine/schedule/examples/headless.rs`, `crates/engine/ecs/src/{lib,world,component_set,command,resource,values,storage}.rs`.

**Fichiers** [Exécution]. Créés : `crates/engine/shell/tests/script_support/mod.rs` (assemblage partagé : voir ci-dessous, E-1), `crates/engine/shell/tests/script_suspend.rs`. Modifié : `crates/engine/shell/Cargo.toml` (`[dev-dependencies]` : ajout de `verbum-script.workspace = true`, rien d'autre). **Aucune modification de `crates/engine/shell/src/**`, de `crates/engine/schedule`, de `crates/engine/ecs`.**

**Assemblage partagé (`script_support/mod.rs`)** [Exécution, E-1] : module inclus par `#[path = "script_support/mod.rs"] mod support;` dans `script_suspend.rs` (U4) et `script_loaded.rs` (U5), comme `tests/support/mod.rs` de `crates/engine/schedule`. Contenu : `Anchored<'a, T>` (E-12, A-S4) ; `Beat(u32)` ; `Components<'a>` à deux champs publics (`states: SparseSet<ScriptState, SPARSE, STATE_DENSE>`, `beats: SparseSet<Anchored<'a, Beat>, SPARSE, STATE_DENSE>`, **ordre réel des paramètres `SPARSE, DENSE`** ; le composant `Beat` stocké est **ancré**, car `Drift` accède à `Write<BEAT>` et un accès sans type en `'a` ne compile pas, E-12 ; `Beat` lui-même ne porte aucune durée de vie ; le contrat `Write<BEAT>` est inchangé, aucun accès supplémentaire) avec `Add`/`Remove` à deux variantes (celle de `Beat` porte `Anchored<'a, Beat>`) et l'implémentation complète de `ComponentSet` ; `Rs<'a>` (`scripts: ScriptTable<'a, 6>`, **non enveloppé** car il porte déjà `'a` ; `emissions: Anchored<'a, Emissions<EMISSIONS_CAP>>` et `transcript: Anchored<'a, Emissions<EMISSIONS_CAP>>`) ; **les comparaisons de transcripts (SR-1, SR-2, SR-5, DM-1) et celles d'EM-3 portent sur les `Emissions<EMISSIONS_CAP>` internes, jamais sur les enveloppes** ; `Parts<'w, 'a>`, les `impl Locate` (une par catégorie), `B<'a>` (`Binding`) ; les systèmes `Clear`, `Collect`, `Interpreter`, `Drift` et, pour les tests de contrôle, `WideInterpreter` et `RivalWriter` ; les listes de systèmes ; des fonctions d'assemblage (`build_world`, `load_scripts`, `launch`, `capture`, `reconstruct`). Un attribut `#[allow(dead_code, reason = "…")]` sur la déclaration du module est admis **si et seulement si** nécessaire pour `clippy -D warnings` (chaque fichier de test n'utilise qu'une partie), rapporté au rapport (E-1).

**Types retenus de l'assemblage (E-12, E-1)** [L, à compiler en U4] :
- `Beat(u32)` : donnée `Copy`, **sans durée de vie**. `Anchored<'a, T>` : enveloppe `Clone, Copy` quand `T: Copy`, champ interne accessible, `PhantomData<&'a ()>`.
- `Components<'a>` : `states: SparseSet<ScriptState, SPARSE, STATE_DENSE>` et `beats: SparseSet<Anchored<'a, Beat>, SPARSE, STATE_DENSE>` ; l'ordre des paramètres est `SPARSE, DENSE` ; **aucun littéral `4, 4`**. `Components<'a>` porte la durée de vie, car le stockage de `Beat` est ancré.
- `Add<'a>` : `State(ScriptState)`, `Beat(Anchored<'a, Beat>)` ; `Remove` : deux variantes sans donnée ; l'implémentation de `ComponentSet` pour `Components<'a>` reflète ce stockage (`insert`, `remove`, `values_mut`, compteurs).
- `Rs<'a>` : `scripts: ScriptTable<'a, 6>` (**non enveloppée**), `emissions: Anchored<'a, Emissions<EMISSIONS_CAP>>`, `transcript: Anchored<'a, Emissions<EMISSIONS_CAP>>`.
- `Drift` conserve **strictement** son contrat `Write<BEAT>` ; aucun accès ni aucune catégorie supplémentaire ; les cinq catégories restent `STATE`, `SCRIPTS`, `EMISSIONS`, `TRANSCRIPT`, `BEAT`.
- Le dépôt de `load_scripts` ne retient que `Asset::Script` ; toute autre variante laisse l'emplacement à `None` dans `ScriptTable`, qui ne reconnaît aucun format (§ 10 point 10).

**Systèmes** [Plan § 4.F] : `Clear` (phase 0, `Produce<EMISSIONS>` : `insert` d'un `Emissions` vide) ; phase 1, **dans cet ordre de déclaration** : `Collect` (`Read<EMISSIONS>`, `Produce<TRANSCRIPT>` : remplace **entièrement** `TRANSCRIPT` par un clone de `EMISSIONS`, sans condition, y compris vide), `Interpreter` (`Write<STATE>`, `Read<SCRIPTS>`, `Produce<EMISSIONS>` : pour chaque instance, copie l'`EntityId` à l'indice *i* de `entities()`, `get_mut`, appelle `step` avec `table.get(état.script())` ; `Err(Full)` ⇒ `unreachable!` avec message nommant la borne `EMISSIONS_CAP ≥ STATE_DENSE × SCRIPT_MAX_STEPS_PER_TICK`), `Drift` (`Write<BEAT>` : ajoute 1 à chaque `Beat`, champ interne de `Anchored<'a, Beat>`). `Scheduler::<B<'a>, L, 4, 2>`. Initialisation à l'assemblage : `EMISSIONS` et `TRANSCRIPT` vides, table vide avant `load`.

**Invariants visés.** X2, X3, X5, X6, X8, X14, X15 ; T3 à T9, T14, T15 ; G-LT, G-ORD-S, G-ST ; plan § 4.E, § 4.F, § 4.G.

**Tests.**

| Test | Énoncé | Porte d'arrêt |
|---|---|---|
| CT-1 | `contract_of::<B<'static>, Interpreter>()` égal au littéral : `reads = {SCRIPTS}`, `writes = {STATE}`, `produces = {EMISSIONS}`, effets `NONE`. Les contrats de `Clear`, `Collect`, `Drift` sont contrôlés **séparément** (littéraux propres) et n'entrent pas dans cette égalité. `WideInterpreter` (même corps, un accès `Read<TRANSCRIPT>` de plus) : le contrôle **échoue**. `B<'static>` est l'instanciation retenue **du test de contrat seulement** ; elle ne fixe aucune durée de vie runtime : le contrat est indépendant de `'a`, et les montages exécutés (SY-0, SR-*) emploient `B<'a>` avec une durée de vie locale | oui (T3) |
| CT-2 | Seul `Interpreter` a `STATE` dans `writes ∪ produces` (liste de production) ; une liste contenant `RivalWriter` (second `Write<STATE>` dans la phase 1) : `Scheduler::new()` rend `Err` de variante `MultipleWriters` (ou, si le type d'erreur n'est pas publiquement nommable, `is_err()` et message `Debug` rapportés, § 10) | oui (T8) |
| CT-3 | Par `SystemList::fill` : `Collect` est l'unique lecteur de `EMISSIONS` ; `EMISSIONS` n'est produite que par `Clear` et `Interpreter` ; `Collect` est l'unique producteur de `TRANSCRIPT` ; `Clear` en phase 0 strictement avant `Collect` en phase 1 ; aucun système n'est dans une phase postérieure à la phase 1 | oui (X15, T15) |
| SY-0 | **Montage emprunté** : deux tampons d'octets construits à l'exécution (scripts de même forme, valeurs d'émission différentes, déclarés **avant** le `World`), table chargée par le vrai `load`, `run_tick` sous le vrai `Scheduler` : les transcripts diffèrent selon le tampon ; le montage compile (G-LT) | oui (G-LT) |
| SR-1 | Scénario de référence sur 12 ticks (instance au slot 4, `Beat` initial 0). **Avant le premier tick**, `TRANSCRIPT` est présent et vide. **Après chaque tick**, `TRANSCRIPT` est égal **exactement** aux émissions de ce tick : (1,10) aux ticks 1, 4, 7 ; (2,99) au tick 10 ; **vide aux ticks 2, 3, 5, 6, 8, 9, 11, 12 bien qu'une émission ait eu lieu à un tick antérieur** ; l'observateur extérieur (un `Vec` de l'appelant) reconstitue la suite de 12 transcripts. `Beat` vaut *k* après le tick *k*, **y compris pendant les attentes** ; `Beat` vaut 12 à la fin | oui (T7, T15) |
| SR-2 | **Reconstruction** (plan § 4.E point 4), pour **chaque** *t* de 0 à 12 : capture des 26 octets du composant après *t* ticks de la référence ; monde, ordonnanceur, table (rechargée depuis les mêmes artefacts), `Beat` initial à 100 ; instance recréée par `from_bytes` + `CommandBuffer::add_component` + `apply` ; ticks restants exécutés ; on exige (a) `to_bytes(from_bytes(capture)) == capture`, (b) l'encodage du composant égal à celui de la référence à chaque tick restant, (c) transcripts du suffixe égaux aux littéraux, (d) état final égal octet à octet à la ligne « 10 » du § 3.3. `Beat` du monde reconstruit n'influence aucune de ces assertions | oui (T4, T7) |
| SR-3 | Table du § 3.3 : `wait` persistant 3, 2, 1 en fin de ticks 1, 2, 3 ; aux ticks 2 et 3 **tous les octets autres que `wait` sont constants** ; aucune valeur 0 de `wait` en fin de tick, ticks 1 à 9 ; reprise au tick 4 : `r0` 3 → 2, `pc` revenu à 3, `wait` réarmé à 3, émission (1,10) ; octets de fin de ticks 1, 2, 3, 4, 10 égaux aux littéraux | oui (T6, T7) |
| SR-4 | Remplacement de l'ordonnanceur par un `Scheduler::new()` neuf **entre les ticks 4 et 5**, sur le même monde : suite d'encodages et de transcripts des ticks 5 à 12 identique à la référence | oui (T5) |
| SR-5 | **Deux instances.** Variante (i) : script A (slot 4, artefact 0) lancé avant le tick 1, script B (slot 1, artefact 1) lancé par `apply` **après le tick 2** : transcripts en multiensemble égaux aux littéraux du § 3.3 ; chaque instance exécute le script de **son slot**. Variante (ii) : A et B lancés avant le tick 1, dans l'ordre (A, B) puis dans l'ordre (B, A) : transcripts égaux en multiensemble entre les deux ordres et aux littéraux ; états finaux égaux par instance. Aucune comparaison d'ordre d'émission entre instances | oui (G-ORD-S, C6, C13) |
| SR-6 | **Lancement par commande différée.** Scénario en quatre étapes, dans cet ordre : (1) `world.spawn()`, puis `CommandBuffer::add_component(entité, Add::State(ScriptState::start(SlotIndex::new(4))?))` : la commande est **ajoutée au tampon** (appel réussi) ; (2) **le composant reste absent du `World`** : aucune instance n'est présente pour l'entité (`world.components().states.get(entité)` rend `None`, ou `contains` faux) ; (3) `World::apply(&mut buffer)` est exécuté ; (4) **le composant devient présent** : l'instance est `Running`, `pc = 0`, registres nuls, `script() == 4`. **Aucune écriture directe du composant** : `World::add_component` est `pub(crate)` dans `verbum-ecs` (F10) et n'est pas appelable depuis l'assemblage ; le seul chemin est le tampon puis `apply`. L'étape (2) précède strictement l'étape (3) | oui (C9, ADR-006) |
| SR-7 | **Script accepté par la Forge, jamais `Faulted(Budget)`.** Famille générée (TOML construit par code à partir de `SCRIPT_MAX_STEPS_PER_TICK`) : segments rectilignes de longueurs `MAX − 1`, `MAX`, `MAX + 1`, `MAX + 2` (terminés par `Wait`, par `Halt`, ou par la fin du programme), et branchements dont l'un ou l'autre bras est le plus long, **avec les deux issues du `JumpIfNonZero`** (registre mis à zéro, puis non nul avant le saut). Pour chaque script : `compile_script` ; **s'il est accepté**, compilation puis exécution par `step` sur plusieurs ticks : aucun état `Faulted(Budget)` n'est jamais observé ; `step` ne retourne pas de compteur d'instructions : **aucune mesure directe** du nombre d'instructions par tour n'est affirmée, la limite étant établie par IN-budget, IN-bound et IN-end (U3) et par l'absence de `Faulted(Budget)` (E-8) ; la borne découle de l'analyse de la Forge (plan § 4.D) et du budget appliqué par `step`, sans qu'une mesure inexistante soit présentée comme assertion de test ; **s'il est refusé**, l'erreur est `BudgetExceeded` et la longueur du plus long chemin dépasse la limite | oui (T10, C14) |

**Cardinalité et coût, natif et Miri** (SR-7, SR-2).

| Test | Natif | Miri |
|---|---|---|
| SR-7 | famille complète (longueurs `MAX − 1` à `MAX + 2`, trois terminaisons, branchements des deux bras, deux issues) | **paire limite seulement** : un segment de `MAX` pas accepté et exécuté, un segment de `MAX + 1` refusé, un branchement à bras long accepté et exécuté avec ses deux issues ; **raccordement partiel**, rapporté (E-7) ; si la durée sous Miri dépasse 10 minutes, rapporter et arbitrer (pas de `--skip` sans décision) |
| SR-2 | 13 reconstructions complètes | 13 reconstructions complètes (monde de 4 entités, ≤ 12 ticks chacune) |
| SR-1, SR-3, SR-4, SR-5, SR-6, SY-0, CT-* | complets | complets |

**Rouge.** Les systèmes `Collect`, `Interpreter`, `Clear`, `Drift` sont d'abord écrits avec des corps `unimplemented!()` (types et accès réels) : SR-1 à SR-7 et SY-0 échouent par panique. **Sans rouge possible** : CT-1 à CT-3 (contrats déjà typés, indépendants des corps) : vert au premier passage, sensibilité par mutation.

**Mutations** (copies isolées ; pour M-4.8 : `cargo test -- --test-threads=1`).

| # | Mutation | Détecteur | Autres assertions attendues en échec |
|---|---|---|---|
| M-4.1 | `Collect` ne remplace pas le transcript quand l'accumulateur est vide | SR-1 (transcript périmé aux ticks 2 et 3) | DM-1 (U5) |
| M-4.2 | `Collect` ajoute au transcript au lieu de le remplacer | SR-1 | SR-5 |
| M-4.3 | `Clear` supprimé de la liste (`SYSTEMS = 3`) | SR-1 (émissions cumulées entre ticks) | CT-3 |
| M-4.4 | `TRANSCRIPT` non initialisé à l'assemblage (slot vide) | SR-1 (assertion avant le premier tick) | — |
| M-4.5 | `RivalWriter` ajouté à la liste d'assemblage | CT-2 (explicite) | toutes les exécutions (`Scheduler::new()` échoue) : rapporté comme telles |
| M-4.6 | `Interpreter` déclare un accès supplémentaire (`Read<TRANSCRIPT>`) | CT-1 **seul** (CT-3 porte sur les lecteurs de `EMISSIONS`, les producteurs de `TRANSCRIPT` et les phases, non sur les lectures de `TRANSCRIPT` : il reste vert) | — |
| M-4.7 | `Drift` absent de la liste | SR-1 (`Beat`) | SR-3 |
| M-4.8 | **état hors World** : `pc` conservé dans un `static` atomique et restauré dans l'état avant chaque `step` (copie isolée **seulement**, exception de G-ST) | SR-2 (une reconstruction à partir d'une capture reprend avec la valeur du `static` d'une exécution antérieure) | SR-4 possible ; si SR-2 et SR-4 sont tous deux verts, **arrêt** (mutation non détectée) |
| M-4.9 | l'adaptateur appelle `step` deux fois par instance et par tick | SR-1 (émissions décalées) | SR-3, SR-2 |
| M-4.10 | **application prématurée des commandes** : dans l'assemblage de lancement (`launch`), `World::apply` est appelé immédiatement après `CommandBuffer::add_component`, **avant** la vérification de l'absence du composant | SR-6, étape (2) (le composant est déjà présent au moment où son absence est exigée) | SR-5 (variante (i)) si le lancement différé est observé avant le tick prévu |
| M-4.11 | l'index de dépôt dans la table est le rang d'appel du dépôt au lieu du `SlotIndex` | SR-5 (A au slot 4, B au slot 1) | SR-1 (le script du slot 4 n'est pas trouvé : `Faulted(NoScript)`), SCR-C1 non (autre crate) |
| M-4.12 | `EMISSIONS_CAP` abaissé sous `STATE_DENSE × SCRIPT_MAX_STEPS_PER_TICK` | **`const` assert de l'assemblage (échec de compilation)** : **exception admise du plan § 7.6** ; complété par EM-2 (U3, échec d'assertion) | — |
| M-4.13 | `Collect` ignore `EMISSIONS` et produit systématiquement un transcript vide | SR-1, aux ticks qui comportent une émission attendue (1, 4, 7 et 10) | SR-2 (c), SR-5, DM-1 (U5) |
| M-4.14 | l'adaptateur itère les instances en ordre inverse | **contrôle d'invariance (CI-S)** : SR-1 à SR-7 et SY-0 doivent **rester verts** ; un échec prouve une dépendance à l'ordre : arrêt (G-ORD-S) | — |
| M-4.15 | `ScriptState::start` remplacé par une construction directe d'un état `Waiting` avec `wait = 0` (via `from_bytes` accepté, mutation de U3 rejouée dans l'assemblage) | **(équivalente à M-3.2 de U3)** : détectée en U3 par ST-2 ; non rejouée en U4 | — |
| M-4.16 | **harnais de reconstruction seulement (pas une mutation de production)** : dans la fonction `reconstruct` de `script_support`, l'état de l'instance n'est plus restitué par `from_bytes(capture)` mais lu dans la trajectoire de référence à l'indice `Beat` du monde neuf (initial 100, ramené modulo 13, soit 9) ; le reste du code (systèmes, `step`, table) est intact | SR-2 (assertion (a) ou (b) pour tout *t* ≠ 9 ; la reconstruction doit reposer sur les seuls octets capturés) | — |
| M-4.17 | **table indexée par l'`artifact_index` du manifeste au lieu du `SlotIndex`** : dans `load_scripts` (assemblage), le dépôt ignore le `SlotIndex` reçu et utilise l'indice d'artefact du script dans le lot (A : 0, B : 1) ; `ScriptTable` et `step` sont intacts. Distincte de M-4.11 (rang d'appel) : l'indice vient ici du manifeste | SR-5, SR-1 et SY-0 (chacun : le script du slot 4, artefact 0, est déposé à l'indice 0 et l'instance du slot 4 passe en `Faulted(NoScript)`) | — |

**Remarque sur M-4.10.** Cette mutation n'est **pas** un accès direct à `World::add_component` : cette méthode est `pub(crate)` dans `verbum-ecs` et inaccessible depuis `crates/engine/shell`, donc non mutable depuis l'assemblage. La mutation ne modifie que le **moment** d'application des commandes : l'ordre `add_component` → absence vérifiable → `apply` → présence est celui que SR-6 impose.

**Contrôles propres.** `cargo tree -p verbum-shell` (dev : `verbum-assets`, `verbum-ecs`, `verbum-forge`, `verbum-render`, `verbum-schedule`, `verbum-script`) ; `cargo tree -i verbum-script --workspace` (seul `verbum-shell`, en dev-dépendance) ; `cargo tree -i verbum-forge --workspace -e normal` (aucun dépendant) ; `cargo tree -p verbum-script -e normal` (seule `verbum-assets`) ; **revue G-ST** rapportée (recherche textuelle dans `crates/script/**` et dans les corps des systèmes d'assemblage : `static`, `thread_local`, `Cell`, `RefCell`, `Atomic`, `async`, `await`, `thread`, `Box`, `Vec`, `unsafe`, `dyn`) ; revue : `EMISSIONS` n'est lue par aucun code hors de `Collect` ; revue : aucune modification de `crates/engine/shell/src/**`, `crates/engine/schedule/**`, `crates/engine/ecs/**` (diff).

**Contrôles natifs et Miri (utilisateur).** Natif : quatre contrôles ; `cargo test -p verbum-shell` ; `cargo test -p verbum-script`. Miri : `cargo +nightly miri test -p verbum-shell` (suite complète ; SR-7 réduit sous Miri, E-7 ; exclusions précédentes de `verbum-forge` inchangées).

**Portes.** G-LT : l'échec de compilation du montage (SY-0 ou `ScriptTable<'a>` dans le `World`) arrête l'unité, message du compilateur rapporté. G-ORD-S : voir M-4.14. E-12 : `Anchored` pour `Clear`, `Collect`, `Drift` seulement (**et pour le composant `Beat` stocké**, I-1) ; **si `Interpreter` exige aussi un contournement** (E0477 sur son accès), c'est une divergence du plan : arrêt.

**Critère de sortie.** CT-1 à CT-3, SY-0, SR-1 à SR-7 verts ; mutations traitées (dont M-4.14 en invariance) ; revue G-ST rapportée ; quatre contrôles verts ; Miri conforme. « En attente de la validation de l'unité U4. »

### U5 — Démonstrateur de bout en bout et clôture

**Objectif [Plan § 8].** Premier comportement scripté de bout en bout, headless, sous la boucle du Shell ; contrôles de clôture ; rapport « non démontré ».

**À fournir.** Fichiers de la session, plus : `crates/engine/shell/tests/{script_support/mod.rs,script_suspend.rs}` tels que rendus, `crates/engine/shell/src/{lib,shell,inbox,pacing}.rs` (lecture ; non lus à la rédaction), `crates/engine/shell/examples/level_headless.rs`, `crates/engine/shell/tests/level_loaded.rs` (modèle d'`execute` sous `run_loop`).

**Fichiers** [Exécution]. Créés : `crates/engine/shell/tests/script_loaded.rs`, `crates/engine/shell/examples/script_headless.rs`. Aucune autre modification (`Cargo.toml` de `crates/engine/shell` déjà complet depuis U4, racine inchangée).

**Invariants visés.** T12, T13, T15 ; plan § 7.5.

**Assemblage** [Exécution] : le test inclut `script_support` ; l'exemple **ne peut pas** inclure un module de `tests/` sans risque de duplication d'avertissements : il **recopie** l'assemblage minimal nécessaire (précédent `level_headless.rs`, E-1). Chaîne : source TOML (scénario de référence) → `compile_assets` (un script, slot 4, capacité 6) → `CompiledSet` → `load` (table `[Format::Script; 6]`) → dépôt dans `ScriptTable` → lancement par `apply` → `run_loop` (cadence vide, arrêt après `N = 12` itérations, `Inbox::<u8>::new(4)` vide) → fermeture d'infrastructure : journal `TickStart(k)`, `run_tick`, `TickEnd(k)`, lecture **en lecture seule** de `TRANSCRIPT`, copie dans l'observateur de la Plateforme (un `Vec` de l'appelant), `Present(k)`. Le `CompiledSet` est déclaré **avant** le `World` (les vues empruntent ses octets).

**Tests.**

| Test | Énoncé | Porte d'arrêt |
|---|---|---|
| DM-1 | Chaîne complète : 12 itérations ; l'observateur reçoit 12 transcripts égaux au littéral (tick 1, 4, 7 : {(1,10)} ; tick 10 : {(2,99)} ; vides ailleurs) ; état final égal à la ligne « 10 » du § 3.3 ; `Beat` = 12 | oui (T12) |
| DM-2 | Quatre lots corrompus, chacun : (a) artefact tronqué d'un octet ; (b) `Wait` de zéro tick (octet de ticks de l'enregistrement 2 mis à zéro) ; (c) registre hors bornes (octet de registre de l'enregistrement 0 mis à `SCRIPT_REGISTERS`) ; (d) table de formats attendant `Format::Sprite` au slot 4 : `load` échoue (`Artifact { Truncated }`, `Artifact { Instruction }` deux fois, `SlotFormat` du slot 4), **aucun tick, aucune présentation** (journal vide, observateur vide) | oui |
| DM-3 | Journal exactement `[TickStart(k), TickEnd(k), Present(k)]` pour *k* de 1 à 12 ; la présentation reçoit un `&Emissions` en lecture seule ; aucune lecture de `EMISSIONS` hors des systèmes (revue) ; l'historique est tenu par l'observateur de la Plateforme, jamais par le `World` (après le dernier tick, `TRANSCRIPT` du `World` ne contient que le transcript du tick 12, vide) | oui (T15) |

**Exemple.** `examples/script_headless.rs` : mêmes sources et chaîne, imprime pour chaque tick la liste de ses émissions (format libre, provisoire), vérifie en interne que les 12 transcripts sont ceux du littéral (panique sinon), code de sortie 0, aucun backend graphique, `FixedStepPacing` de 1 ms et 12 itérations.

**Rouge.** **Aucun rouge n'est possible** : les tests n'exercent que des API déjà implémentées (U1 à U4 et `run_loop`) ; vert au premier passage, sensibilité par mutation.

**Mutations** (copies isolées).

| # | Mutation | Détecteur | Autres assertions attendues en échec |
|---|---|---|---|
| M-5.1 | le résultat de `load` est ignoré (l'exécution continue après une erreur) | DM-2 (journal non vide) | — |
| M-5.2 | présentation avant `run_tick` | DM-3 (ordre du journal) | DM-1 (transcripts décalés) |
| M-5.3 | l'observateur lit `TRANSCRIPT` après le tick suivant | DM-1 | DM-3 |
| M-5.4 | valeur d'émission du scénario modifiée dans la source (par exemple 10 → 11) | DM-1 | SR-* non (autre fichier) |
| M-5.5 | `Collect` désactivé dans l'assemblage de l'exemple | exemple (panique de vérification interne, code non nul) | — |
| M-5.6 | l'exemple ne vérifie pas ses 12 transcripts | **(équivalente côté tests)** : la propriété est portée par DM-1 ; pas de test artificiel sur l'exemple ; revue de l'exemple | — |

**Contrôles de clôture (utilisateur, dépôt réel).** `cargo fmt --all -- --check` ; `cargo build --workspace` ; `cargo test --workspace` (doctests compris) ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo run -p verbum-shell --example script_headless` (code 0, timeline attendue) ; `cargo tree -p verbum-assets` (aucune dépendance) ; `cargo tree -p verbum-script` et `-e normal` (seule `verbum-assets`) ; `cargo tree -p verbum-forge` ; `cargo tree -i toml --workspace` (seul `verbum-forge`) ; `cargo tree -i verbum-forge --workspace -e normal` (aucun dépendant) ; `cargo tree -i verbum-script --workspace` (seul `verbum-shell`, dev) ; diff : `crates/{ecs,schedule,render,verbum}/**` et `crates/engine/shell/src/**` vides ; liste des adaptations R-ADAPT consignées ; assertions des tests de F14 inchangées.

**Miri (utilisateur).** `cargo +nightly miri test -p verbum-assets` ; `cargo +nightly miri test -p verbum-script` ; `cargo +nightly miri test -p verbum-forge -- --skip p3_1 --skip p3_2 --skip t6_1 --skip t6_3 --skip t6_6` ; `cargo +nightly miri test -p verbum-shell`. Statut rapporté : `verbum-assets` et `verbum-script` intégraux ; `verbum-forge` **partiel** (exclusions de l'étape 5 ; programmes TOML de limite réduits) ; `verbum-shell` intégral sauf réduction de SR-7 (E-7). L'exemple n'est pas exécuté sous Miri.

**MSRV 1.85 (utilisateur).** Deux commandes séparées : `cargo +1.85.0 build --workspace`, puis `cargo +1.85.0 test --workspace`. Points de vigilance rapportés : absence d'avertissement `dead_code` sur des constantes utilisées seulement dans une `const _` assert (règle 6) ; absence de let-chains ; les avertissements MSRV préexistants des étapes 3 et 4 (E-14) ne sont pas modifiés et sont distingués des nouveaux.

**Critère de sortie.** DM-1 à DM-3 verts ; exemple de code 0 ; mutations traitées ; contrôles de clôture conformes ; rapport de clôture contenant le tableau T1 à T15 (§ 8) et **reproduisant littéralement le § 10 du plan r1.2** (plan § 14.9). « En attente de la validation de l'unité U5. »

## 5. Rapport de fin d'unité (gabarit, rien de narratif)
1. Unité, état (livrée, en attente de validation). 2. Fichiers créés, modifiés (chemins complets) et fichiers livrés par `SendUserFile`. 3. Faits [Vérifié] revérifiés : confirmés ou divergence (arrêt). 4. Rouge : test, motif d'échec observé (ou « sans rouge possible », motif). 5. Résultats des quatre contrôles : lignes réelles, **espace de travail de la session** et **dépôt réel** séparés ; contrôles non exécutés nommés. 6. Mutations : mutation, test détecteur, autres assertions échouées, mode de détection (échec, panique) ; mutations équivalentes rapportées comme telles avec leur justification. 7. Portes : G-CM (aucun arrêt, ou arrêt rapporté), G-LT, G-ORD-S, G-ST ; liste R-ADAPT. 8. Garanties : statiques / revue (sans mélange), revue G-ST point par point, revue de la signature publique (U3). 9. Contrôles natifs, Miri (intégral ou partiel, avec la liste des réductions), MSRV. 10. Ajustements de noms et écarts du plan ou de ce document (§ 6). 11. Regroupement de commit proposé, `tree.md` à régénérer. 12. Fin : « En attente de la validation de l'unité Un. »

## 6. Arrêt, reprise
**Arrêt immédiat, rapport à l'utilisateur, aucun contournement** : un fait [Vérifié] qui diverge ; une mutation non détectée **et non établie équivalente** ; une contradiction entre ce document et le plan ; un besoin de modifier un fichier interdit (§ 2), un crate du noyau, le Scheduler ou le Shell source ; un besoin d'`unsafe`, d'allocation, d'un état hors World, d'un trait public, d'un ordre ou d'une clé (**G-ST, G-ORD-S**) ; **un échec de compilation de durée de vie du montage (G-LT)** ; **une situation de G-CM** (nouveau niveau, ordre modifié, dépôt anticipé, `VMAN` modifié, allocation dans `load`, assertion d'un test de F14 à modifier) ; un `match` exhaustif cassé par une variante ajoutée ; un octet de SCR-F1 ou de SCR-R1 qui diverge du littéral du § 3.3 ; un `Interpreter` qui exige `Anchored` ; un test hors liste ; une mutation qui fait échouer d'autres assertions que celles annoncées sans explication ; un code d'erreur de compilation épinglé ; une durée Miri jugée excessive (rapporter et arbitrer, aucun `--skip` ni `ignore` sans décision). À l'arrêt : état exact (fichiers, derniers résultats réels), cause, options minimales, aucune modification d'office. **Reprise** : seulement sur instruction explicite ; une session reprise (nouveau contexte) relit ce document, le plan, le handoff et le rapport de la dernière unité validée ; elle ne rouvre aucun arbitrage clos ni aucune unité validée ; une unité interrompue est refaite depuis ses tests.

## 7. Non démontré à la clôture (compléments d'exécution)
Le rapport de U5 **reproduit littéralement le § 10 du plan r1.2** (copie sans reformulation, plan § 14.9), avec la mention que 4 et 16 sont provisoires. Les puces ci-dessous sont une version de travail, **non littérale** : elles n'y figurent qu'en complément distinct, intitulé « Compléments d'exécution », et seulement pour ce que le § 10 du plan ne dit pas (réductions Miri, Miri partiel de `verbum-forge`, précision sur le code inaccessible).
- **Lecture de données métier par les scripts** : non exprimable à cette étape ; le comportement « conditionnel » se limite aux registres de l'instance ; ce n'est pas un système de comportement complet.
- **Lancement depuis un système** (`Commands<ADD>`) et arrêt ou retrait d'une instance par commande : seul le lancement par `apply` à l'assemblage est exercé.
- **Appels, sous-programmes, pile** : non introduits ; une pile à capacité fixe portée par le composant resterait conforme à ADR-Sc01 pt 4.
- **Conversion des durées humaines en ticks** ; politique temporelle (future ADR-S04, non rédigée).
- **Persistance de l'état** (ADR-SV01) : l'encodage canonique n'est qu'un outil de reconstruction ; compatibilité de version non traitée.
- **Fixité de l'interface par les types** : contrôle de composition (CT-1) et revue seulement ; un système supplémentaire hors liste ou un handle externe n'est pas empêché par les types.
- **Absence d'état caché dans tout assemblage** : revue textuelle (G-ST), SR-2 et SR-4 sur l'assemblage de référence.
- **Lot mixte dans la `ScriptTable` réelle** : aucun test ne charge un lot mixte (sprite, tileset, map, font, script) dans la `ScriptTable` réelle ; la correspondance slot ↔ indice est établie par SCR-C1 (tableau témoin local), TB-1 et SR-5 (lots de scripts seulement).
- **Ordre des émissions entre instances** ; clés de tri, priorité.
- **Journal persistant des émissions** : non fourni ; l'historique est tenu hors du World par l'observateur de la Plateforme.
- **Analyse de flot de contrôle** conservatrice (code inaccessible compris, plus long des deux bras de `JumpIfNonZero`) : elle peut refuser un script dont le chemin réel serait plus court, ou dont seul un code inaccessible dépasse la limite.
- **Valeurs 4 et 16** : provisoires ; adéquation à un contenu réel non démontrée ; `ScriptState.script` limité aux slots `0..=65 535`.
- **Miri** : `verbum-forge` partiel (exclusions de l'étape 5 ; programmes TOML de limite réduits) ; SR-7 réduit sous Miri ; SCR-R10 et `check_flow` réduits sous Miri avec couverture native complète.
- **Performance** (double décodage, dispatch) ; assets de taille réaliste.
- **Visualisation du jeu** : étude distincte après la clôture de l'étape.

## 8. Clôture de l'étape (U5)
L'utilisateur déclare l'étape close ; la session ne la déclare pas. Le rapport d'U5 contient en plus un tableau T1 à T15 : pour chaque critère du plan § 1.3, l'identifiant des tests et revues qui l'établissent et leur résultat réel (lignes de sortie). Critères : plan § 14 (1 à 9). Le rapport d'U5 reproduit en outre **littéralement le § 10 du plan r1.2** (plan § 14.9). Aucun travail de visualisation n'est entrepris.

## 9. Écarts entre le plan et ce document (à valider avec lui)

**E-1 — Fichiers de tests partagés et emplacement des tests de chargement.** (a) Le plan § 13 liste `tests/{script_suspend,script_loaded}.rs` et l'exemple ; ce document ajoute `crates/engine/shell/tests/script_support/mod.rs` (assemblage partagé par les deux fichiers de test, précédent `crates/engine/schedule/tests/support/mod.rs`), avec `#[allow(dead_code, reason = …)]` admis seulement si `clippy -D warnings` l'exige ; l'exemple recopie son assemblage (précédent de l'étape 5). **Types ancrés** : dans `Components<'a>`, le stockage des `Beat` est `SparseSet<Anchored<'a, Beat>, SPARSE, STATE_DENSE>` (et **non** `SparseSet<Beat, 4, 4>`), celui de `ScriptState` est `SparseSet<ScriptState, SPARSE, STATE_DENSE>`, avec les constantes `SPARSE` et `STATE_DENSE` dans cet ordre ; `Beat` reste `Copy` sans durée de vie ; `Add`/`Remove` et `ComponentSet` reflètent ce stockage (types complets : U4, « Types retenus de l'assemblage »). `Anchored<'a, T>` enveloppe les ressources `EMISSIONS` et `TRANSCRIPT` (`Anchored<'a, Emissions<EMISSIONS_CAP>>`) et le composant stocké `Beat` (`Anchored<'a, Beat>`), pas `ScriptTable<'a, 6>` ni `ScriptState` ; les comparaisons de transcripts et celles d'EM-3 portent sur les `Emissions` internes, jamais sur les enveloppes (EM-3, en U3, n'emploie pas `Anchored`, absent du crate `verbum-script`). (b) Les tests SCR-L* (lot chargé avec `Format::Script`) sont placés dans des tests de module de `script.rs` ou de `format.rs`, jamais dans `load.rs` (qui reste inchangé, plan § 13).

**E-2 — Position de `pc` après `Halt` et fin de programme.** Le plan n'en fixe pas la valeur. Retenu : `Halt` laisse `pc` sur l'instruction `Halt` ; une fin de programme implicite donne `pc = n`. Sans effet sur un invariant ; épinglé par IN-5 et SR-3. Changement possible sur décision de l'utilisateur, impactant ST-1 (littéral de la ligne « 10 ») et IN-5.

**E-3 — Sous-cas de SCR-F7.** Le plan décrit SCR-F7 comme « limite des 65 535 pas par un assistant testé directement ; bout en bout natif seulement ». Ce document y inclut, dans le même test, l'analyse `check_flow` itérative sur 65 535 instructions construites en mémoire (motif acceptable, motif rejeté, chaîne de sauts cyclique) pour établir l'absence de récursion. Aucun test nouveau : un sous-cas de SCR-F7.

**E-4 — Codes numériques de l'encodage d'état.** `status` : 0 à 3 ; `fault` : 0 à 2 ; layout champ par champ du § 3.3. Choix locaux provisoires ; le plan fixe seulement la longueur (26 pour 4 registres) et l'ordre des champs.

**E-5 — Types des constantes partagées.** `SCRIPT_REGISTERS` et `SCRIPT_MAX_STEPS_PER_TICK` sont de type `usize` (usage en longueur de tableau et en comparaison de compteurs) ; non précisé au plan.

**E-6 — `is_empty`.** `is_empty` est ajouté à `Script`, `Emissions` pour `clippy::len_without_is_empty` (précédent E-6 de l'étape 4) ; aucune autre API.

**E-7 — Réductions sous Miri.** SCR-R10 (512 instructions), `check_flow` (512 nœuds), programmes TOML de limite (paire limite) et SR-7 (paire limite) sont réduits sous `cfg!(miri)` ; la couverture native n'est jamais réduite. Statut rapporté partiel pour le raccordement de `check_flow` à `compile_script` sous Miri (précédent E-13). Le plan demande des tests « natifs et Miri distingués » ; la répartition exacte est une précision d'exécution.

**E-8 — Deux formes de SCR-F4 (b) et (c) ; périmètre exact de SR-7.** Les cas du plan (chemin de `MAX` accepté, `MAX + 1` refusé) sont exécutés sous deux formes (entrée 0 et entrée de reprise) pour détecter l'omission des entrées de reprise ; ce sont des **cas du même test**, non des tests nouveaux. **SR-7** : le plan § 7.4 y énonce un « compte d'instructions exécutées par tour ≤ `SCRIPT_MAX_STEPS_PER_TICK` » ; `step` ne retourne pas ce compteur, ce document n'affirme donc aucune mesure directe : la propriété est établie par l'absence de `Faulted(Budget)` sur des scripts acceptés par la Forge, complétée par IN-budget, IN-bound et IN-end.

**E-9 — Mutation M-4.8 et `static`.** Le plan interdit tout état hors World ; la mutation M-4.8 emploie un atomique statique **sur copie isolée hors de l'arbre de travail**, pour établir la sensibilité de SR-2 ; ce n'est pas du code livré.

**E-10 — Mutations équivalentes et reclassements.** M-1.9, M-1.10, M-2.14, M-3.28 et M-5.6 sont déclarées équivalentes (règle 3 du § 1) ; la session confirme l'équivalence par raisonnement écrit au rapport, sans test artificiel. **M-3.29 n'est pas une mutation équivalente** : elle est retirée du tableau des mutations d'U3 et reclassée en revue de conformité de la signature publique (U3, « Contrôles propres »), le retour par copie et le retour par référence n'étant pas interchangeables (emprunt et durée de vie). **M-4.10** est reformulée en application prématurée des commandes (U4), non en accès direct à `World::add_component` (`pub(crate)`, F10). **M-4.16** est limitée au harnais de reconstruction et n'est pas une mutation de production. **Lecture du critère du plan § 14.2** (« chaque mutation du § 7.6 détectée par un test nommé ») : toute mutation non équivalente doit être détectée par un test nommé ; une mutation démontrée équivalente est rapportée avec sa justification, sans test artificiel. Cette interprétation est proposée par ce document et acceptée par l'utilisateur le 3 novembre 2026 ; le plan r1.2 n'est pas modifié.

**E-11 — Budget sur tous les nœuds et code inaccessible.** Le plan § 4.D décrit un coût par « entrée d'un tour » (instruction 0 et instructions de reprise) et énonce que l'analyse porte sur l'ensemble du graphe, code inaccessible compris. `check_flow` réalise cette double exigence par une seule vérification : après détection des cycles, **chaque nœud `0..n`** est comparé à `SCRIPT_MAX_STEPS_PER_TICK`. **`BudgetExceeded { entry }` désigne le premier nœud en dépassement par position, et non nécessairement l'entrée d'un segment** (instruction 0 ou reprise) ; le nom du champ est celui du plan. **Statut** : c'est une **interprétation plus stricte** du plan § 4.D, et non sa citation littérale ; elle a été soumise avec ce document et **acceptée par l'utilisateur** (3 novembre 2026). Conséquences : (a) la liste des entrées de reprise devient redondante (M-2.14 équivalente) ; (b) le risque réel qu'elle couvrait (une reprise non vérifiée) est porté par M-2.14b et SCR-F4 (c), forme de reprise ; (c) SCR-F4 (k) distingue deux erreurs sur code inaccessible : acyclique dépassant la limite ⇒ `BudgetExceeded` (k1), cyclique sans yield ⇒ `CycleWithoutYield` (k2) ; (d) la priorité `CycleWithoutYield` > `BudgetExceeded` est conservée (SCR-F4 (l), M-2.16c). Cette interprétation n'est pas contradictoire avec le plan : la vérification de tous les nœuds est au moins aussi stricte que celle des entrées, et identique sur le code accessible ; le plan n'est pas modifié.

## 10. Relevé des points non vérifiés à la rédaction
1. **Visibilité du type d'erreur de `Scheduler::new()` (`ModelError`)** et de son module : lu seulement par `scheduler.rs` (import interne). CT-2 se rabat sur `is_err()` et `Debug` si la variante n'est pas nommable.
2. **Contenu de `crates/engine/forge/tests/pipeline.rs`**, `crates/engine/forge/src/{font,tileset}.rs`, `crates/engine/render/src/{lib,accumulator}.rs` : non lus ; modèles à relire au démarrage d'U2 et d'U3.
3. **`crates/engine/shell/src/{shell,inbox,pacing}.rs`** : non lus ; `run_loop` et `Inbox` connus par leurs usages (`level_loaded.rs`, `level_headless.rs`) ; à revérifier au démarrage d'U5.
4. **`crates/engine/forge/tests/pipeline.rs` et autres tests** : présence éventuelle de `match` exhaustifs sur `AssetKind`, `Format`, `Asset`, `ReadError` ou `ForgeError` : recherche au démarrage d'U1 et U2.
5. **Durées sous Miri** des tests de `verbum-forge` (programmes TOML), de SR-7 et des 13 reconstructions de SR-2 : non mesurées.
6. **Compilation sous MSRV 1.85** de `ScriptTable<'a, CAP>` (tableau `[None; CAP]` d'`Option<Script<'a>>`, garde statique par constante associée), de `emissions_bound_holds` (`checked_mul` en `const fn`), de l'`Interpreter` : non vérifiée avant U3 et U4.
7. **Interpréteur et E0477** : l'hypothèse « son accès, qui nomme `ScriptTable<'a>`, compile sans `Anchored` » est un [Déduit] du motif de `ProduceTiles` de l'étape 5 ; confirmée ou infirmée par U4 (arrêt si infirmée).
8. **API de `CommandBuffer` et de lecture des composants depuis l'assemblage** (SR-6, étape (2)) : la forme exacte de l'observation de l'absence du composant avant `apply` (`components().states.get(entité)` ou `contains`) est déduite de `storage.rs` et `values.rs` ; à revérifier au démarrage d'U4.
9. **Forme de `Components<'a>` et de son `ComponentSet`** (U4) : un composant stocké ancré (`Anchored<'a, Beat>`) et des `Add`/`Remove` portant une durée de vie sont déduits des contraintes de `component_set.rs` et de F11 ; non compilés. Si `ComponentSet` ou `World` l'interdit, c'est une divergence : arrêt, message du compilateur rapporté, aucun contournement.
10. **Lot mixte et `ScriptTable` réelle** (point documentaire non résolu, signalé sans supposition) : SCR-C1 (U2) prouve la correspondance slot ↔ indice sur un lot mixte avec un tableau témoin local ; TB-1 (U3) et SR-5 (U4) exercent la `ScriptTable` réelle, mais sur des scripts seuls. Aucun test ne combine lot mixte et `ScriptTable` réelle. Le plan (§ 7.2, SCR-C1) ne l'exige pas ; une décision de l'utilisateur serait nécessaire pour l'ajouter.
11. **Résultats sur le dépôt réel** : aucun, par construction (aucun code n'est écrit à ce stade).
