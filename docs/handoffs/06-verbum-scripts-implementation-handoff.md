# Handoff d'implémentation — étape 6 (scripts), unité U1 : lecteur `script.bin`

Document de passation, non normatif, autonome pour U1. Il est destiné à une nouvelle session Claude dédiée **exclusivement à l'implémentation de l'unité U1** de l'étape 6 de Verbum. Il ne dépend d'aucune mémoire de la session de conception. **Il ne vaut pas autorisation de démarrer U1.** Les unités U2 à U5 ne sont pas abordées dans cette session ; chacune aura sa propre session et, sur demande, son propre handoff.

Structure alignée sur `05-verbum-assets-implementation-handoff.md` (§ 1 à 13).

Légende : **[Validé]** décision arrêtée par l'utilisateur · **[Vérifié]** fait constaté dans un document ou un source lu, à revérifier au démarrage · **[À confirmer]** point que l'implémentation doit établir ou signaler · **[L]** choix local provisoire. **Aucun résultat de compilation, de test, de Miri ou de mutation n'existe pour l'étape 6** : tout résultat mentionné ici est une attente, jamais un constat.

## 1. Rôle

Vous êtes implémenteur Rust. Vous exécutez de façon disciplinée l'unité U1 du plan et du document d'exécution de l'étape 6. Vous ne révisez pas l'architecture spontanément, vous ne refaites pas l'audit et vous ne rouvrez pas les arbitrages clos (§ 9). Le projet est Verbum (moteur 2D Rust, ECS/DOD/AOT). Vous travaillez avec l'utilisateur, qui raisonne en architecte système ECS/DOD/AOT sans avoir encore tout le vocabulaire du domaine. Registre : vouvoiement, réponses concises, structurées, opérationnelles, sans généralités ni métaphores ; un concept technique avancé est défini brièvement et précisément à sa première apparition.

## 2. Contexte d'implémentation

1. **Vous n'avez pas accès au dépôt local de l'utilisateur.** Vous travaillez dans un conteneur distant. Ce que vous savez du code vient exclusivement des fichiers que l'utilisateur vous fournit. Ne présumez jamais le contenu d'un fichier non fourni ; ne demandez pas le dépôt en bloc ; demandez nommément ce qui manque.
2. **Livraison : fichiers complets, jamais de diff.** Chaque fichier `.rs` créé ou modifié est livré **complet** par `SendUserFile`, première ligne = commentaire de chemin (`// crates/engine/assets/src/script.rs`) suivi d'une ligne vide. Documentation des sources `.rs` **en français**. Aucun `Cargo.toml` n'est modifié en U1. Un fichier existant modifié n'est livré qu'après avoir été fourni par l'utilisateur, et rendu en entier.
3. **Contrôles réels exécutés par l'utilisateur** : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, plus les contrôles propres à U1 (§ 8) et Miri. Si une chaîne Rust est disponible dans votre conteneur, vous pouvez reconstituer un espace de travail à partir des fichiers fournis ; vous **distinguez toujours** « résultat dans l'espace de la session » et « résultat du dépôt réel », et vous ne présentez jamais le premier comme le second. Aucun résultat n'est déduit, estimé ni rapporté sans avoir été produit.
4. **Pas de commit par la session.** Vous proposez en fin d'unité un regroupement par invariant démontré. `tree.md` est régénéré par l'utilisateur : vous signalez le fichier créé (`script.rs`) sans y toucher.
5. **Seul l'utilisateur déclare l'unité validée et l'étape close.**
6. **Clause d'échappement** : en cas de problème, d'ambiguïté ou d'impossibilité, vous **vous arrêtez et exposez** (constat factuel : source, texte, observation ; options minimales) au lieu de contourner. Les cas d'arrêt sont au § 11.

## 3. Documents de référence et ordre de lecture

À fournir par l'utilisateur au démarrage :
1. `docs/handoffs/06-verbum-scripts-implementation-handoff.md` (ce document).
2. `docs/implementation/06-verbum-scripts-execution.md` : règles de session (§ 1), interdits (§ 2), faits vérifiés (§ 3), **unité U1 du § 4**, gabarit de rapport (§ 5), arrêt et reprise (§ 6), écarts E-1 à E-11 (§ 9), points non vérifiés (§ 10).
3. `docs/implementation/06-verbum-scripts.md` : plan **r1.2** (Lire le plan r1.2 une fois en entier, puis ne consulter à nouveau que les sections utiles à U1.).
4. `docs/architecture/adr/ADR-Sc01-scripts.md`, `ADR-A01-assets.md` et `ADR-000-language-and-material-invariants.md` (points 1, 3 et 4).
5. `docs/implementation/05-verbum-assets-execution.md` : § 3.2 (portes G-CM) et écarts E-12 à E-14 seulement (E-12 ne concerne pas U1).
6. `tree.md` : fait foi sur les chemins du dépôt.

Les autres ADR, `00-principes.md`, `01-runtime-ecs.md` et `02-concepts.md` ne se relisent que si U1 les cite ; vous les demandez alors nommément.

## 4. Hiérarchie normative

1. Les ADR et le corpus gouvernent.
2. Le plan `06-verbum-scripts.md` r1.2 (validé, inchangé) définit les exigences de l'étape.
3. Le document d'exécution organise les sessions et ne contient aucune règle de conception.
4. Ce handoff ne contient aucune règle de conception.

Divergence plan / exécution : le plan prévaut ; plan / ADR : l'ADR prévaut ; exécution / handoff : le document d'exécution prévaut. Dans tous les cas, la divergence est signalée à l'utilisateur et jamais résolue par la session.

## 5. Statut des documents à la passation

- **Plan `06-verbum-scripts.md` r1.2** (1er novembre 2026) : **validé** ; arbitrages A-S1 à A-S5 arrêtés ; **inchangé**.
- **`06-verbum-scripts-execution.md`** (révision du 3 novembre 2026, nuit : U2 budget sur tous les nœuds et code inaccessible, U3 M-3.29 reclassée en revue de signature, U4 M-4.10 et ancrage de `Beat`, IN-5, M-4.16/M-4.17, découpage une session par unité) et **ce handoff** : **soumis à la validation finale de l'utilisateur, non marqués validés**. Tant que la validation finale n'est pas communiquée à la session, elle l'indique en une ligne et attend.
- Si l'utilisateur demande une correction ultérieure, appliquez **uniquement** cette correction, rendez le fichier **complet** (pas de diff), ne modifiez rien d'autre, attendez la validation.

## 6. Interdiction de coder avant instruction

**Aucune ligne de code, aucun fichier, aucune commande de build ou de test, avant que l'utilisateur ait écrit : « démarrez l'unité U1 ».** À la lecture initiale, accusez réception en cinq lignes maximum, en indiquant le statut des documents tel qu'au § 5.

## 7. Périmètre et découpage

**Une session Claude par unité** [Validé, utilisateur, 3 novembre 2026] : U1, U2, U3, U4, U5 dans cinq sessions distinctes. La recommandation A/B/C du plan reste un élément de planification et n'est pas suivie. U4 garde sa session propre (contexte ECS/Scheduler, porte E-12). **Cette session traite U1 seulement.** À la fin d'U1 : rapport selon le gabarit du § 12 du présent document (reprise du § 5 du document d'exécution), puis arrêt avec « En attente de la validation de l'unité U1. ». Elle ne prépare ni ne propose U2.

| Unité | Contenu | Cette session |
|---|---|---|
| **U1** | lecteur `script.bin` dans `verbum-assets` ; `Format::Script`, `Asset::Script`, `ReadError::Instruction` ; non-régression étape 5 | **oui, seule** |
| U2 | compilation `script.toml` → `script.bin` dans `verbum-forge`, `check_flow` | non |
| U3 | crate `verbum-script` : état, émissions, table, `step` | non |
| U4 | intégration Scheduler, interface fixe, reprise | non |
| U5 | démonstrateur, clôture | non |

**Contenu d'U1.** Ajouter le format de script au mécanisme de l'étape 5 sans le modifier : layout, lecteur à validation structurelle, `Format::Script`, `Asset::Script`, `ReadError::Instruction`, constantes partagées ; prouver la non-régression complète de l'étape 5. Invariants visés : X1, X10, X11, X12, X18 ; G-CM ; plan § 4.A, § 4.B, § 6.

**Fichiers à fournir (U1)**, versions courantes du dépôt réel, chemins de l'arborescence actuelle `crates/engine/**` :

| Fichier | Usage |
|---|---|
| `crates/engine/assets/Cargo.toml` | lecture ; **inchangé** |
| `crates/engine/assets/src/lib.rs` | **modifié** (module, exports) |
| `crates/engine/assets/src/format.rs` | **modifié** (`Format::Script`, `Asset::Script`, identité `VSCR`, bras de `decode`) |
| `crates/engine/assets/src/sprite.rs` | **modifié, ajout seul** de la variante `ReadError::Instruction` (`ReadError` y est défini) |
| `crates/engine/assets/src/header.rs` | lecture seule (`PREFIX_LEN = 10`, `read_prefix`, `Prefix`) |
| `crates/engine/assets/src/load.rs`, `manifest.rs` | lecture seule, **jamais modifiés** |
| `crates/engine/assets/src/font.rs` | lecture seule : **modèle** de lecteur à charge variable et de garde de constantes |
| `crates/engine/assets/src/script.rs` | **créé** par cette unité |

Recherche des `match` exhaustifs sur `Format`, `Asset`, `ReadError` : la session demande **par nom** les fichiers de `crates/engine/forge/src/` et `crates/engine/shell/` concernés si la recherche dans `assets` en laisse un hors périmètre (§ 11).

## 8. Méthode

- **Rouge avant vert** quand c'est possible : tests écrits contre des squelettes (`unimplemented!()` : `Script::read`, bras de `decode`), échec constaté par **panique** pour la raison attendue et rapporté. **Un échec de compilation n'est pas un rouge valide.** Sans rouge possible, justifié par écrit : SCR-K1 (constantes déjà définies, sensibilité par mutation), SCR-L4 (tests existants), SCR-R7 sur le seul contrôle de longueur s'il est déjà écrit au squelette.
- **Sensibilité par mutation** sur copie isolée **hors de l'arbre de travail**, une mutation à la fois, jamais présentée comme rouge initial ; le rapport nomme pour chaque mutation le test détecteur **et les autres assertions échouées**. **Une mutation non détectée est un arrêt, sauf si elle est établie équivalente** (comportement observable inchangé dans le domaine atteignable) : elle est alors rapportée comme telle avec sa justification, **sans test artificiel**. Aucune détection par échec de compilation n'est admise.
- **G-CM** : les validations propres au format script vivent **à l'intérieur** des niveaux 1 à 4 existants (niveau 2 : format du slot ; niveau 3 : `Script::read`) ; les ajouts additifs d'API sont admis. Créer ou supprimer un niveau, changer l'ordre, déposer avant la fin du niveau 3, modifier `VMAN`, introduire une allocation dans `load`, modifier le modèle d'identité, modifier une assertion d'un test de l'étape 5 = **arrêt**.
- **R-ADAPT** : seules les adaptations mécaniques de signature, exposées avant d'être faites et consignées une à une, sont admises hors des fichiers listés.
- **Contrôles propres à U1** : `cargo tree -p verbum-assets` (aucune dépendance, dev comprises) ; revue textuelle : aucun `alloc`, `unsafe`, `Vec`, `Box`, `String` dans `crates/engine/assets/src/**` ; diff de `load.rs`, `manifest.rs`, `header.rs` vide ; `sprite.rs` : seule la variante ajoutée ; `Cargo.toml` inchangé. La revue n'est jamais présentée comme une preuve statique.
- **Natif** : `cargo test -p verbum-assets`. **Miri (utilisateur)** : `cargo +nightly miri test -p verbum-assets` (suite complète, **sans `--skip`** ; SCR-R10 réduit à 512 instructions sous `cfg!(miri)`). Statut rapporté intégral ou partiel. Commande exacte à confirmer avec l'utilisateur.
- **Compatibilité MSRV 1.85 dès l'écriture** : pas de let-chains (`let … else` à la place) ; `is_empty` accompagne tout `len` public ; **aucune constante utilisée seulement dans une `const _: () = assert!(…)`** (avertissement `dead_code`, leçon E-14 de l'étape 5) : chaque constante est aussi utilisée dans du code compilé ; aucune assertion sur l'horloge. La vérification MSRV est faite en U5 (deux commandes séparées), pas ici.
- Chaque unité commence par **revérifier** les faits [Vérifié] qu'elle utilise sur les fichiers fournis (§ 3 du document d'exécution : F3 formats et décodage, F4 lecture et `ReadError`, F5 `load`, F6 indices, F14 tests de non-régression).

**Artefact, layout et lecteur** [Validé, plan § 4.A, § 4.B ; noms [L]]. Little-endian, champ par champ, sans cast : préfixe commun de 10 octets (identité 4, version `u16` 2, longueur de charge utile `u32` 4), puis charge utile : `n` (`u16`), puis `n` enregistrements de 8 octets (opcode `u8`, registre `u8`, `arg16` `u16`, `arg32` `u32`). Longueur déclarée attendue `2 + 8·n`. Constantes de `verbum-assets` : `SCRIPT_FORMAT_ID = *b"VSCR"`, `SCRIPT_FORMAT_VERSION = 1` (`u16`), `SCRIPT_FIXED_PAYLOAD_LEN = 2`, `SCRIPT_INSTRUCTION_LEN = 8`, `SCRIPT_REGISTERS = 4`, `SCRIPT_MAX_STEPS_PER_TICK = 16` (type `usize`, **provisoires**, non invariants du moteur).

| Opcode | Instruction | `registre` | `arg16` | `arg32` |
|---:|---|---|---|---|
| 0 | `Halt` | ignoré | ignoré | ignoré |
| 1 | `Emit` | ignoré | canal | valeur `i32` |
| 2 | `Wait` | ignoré | ignoré | ticks `u32` (≥ 1) |
| 3 | `Set` | registre (< `SCRIPT_REGISTERS`) | ignoré | valeur `i32` |
| 4 | `Add` | registre (< `SCRIPT_REGISTERS`) | ignoré | valeur `i32` |
| 5 | `Jump` | ignoré | cible (< `n`) | ignoré |
| 6 | `JumpIfNonZero` | registre (< `SCRIPT_REGISTERS`) | cible (< `n`) | ignoré |

Octets ignorés : **non contrôlés** par le lecteur (la Forge, en U2, les écrit à zéro). **Ordre des contrôles de `Script::read`** : (1) préfixe de 10 octets complet, sinon `Truncated` ; (2) identité ; (3) version ; (4) longueur déclarée `< SCRIPT_FIXED_PAYLOAD_LEN` : `PayloadLength` ; (5) restants inférieurs : `Truncated`, supérieurs : `TrailingBytes` ; (6) longueur déduite `2 + 8·n` calculée **en `u64`**, différente de la déclarée : `PayloadLength` ; (7) validation de chaque enregistrement dans l'ordre : opcode inconnu, registre hors bornes, cible `≥ n`, `Wait` de zéro tick : `ReadError::Instruction`. Zéro instruction (charge de 2 octets) : conforme ; charge de longueur nulle : invalide.

API [L], aucun trait public : `Script<'a>` (`Clone, Copy, Debug, PartialEq, Eq`) avec `read(&'a [u8]) -> Result<Self, ReadError>`, `len() -> u16`, `is_empty() -> bool`, `instruction(index: u16) -> Option<Instruction>` (décodage à la demande, `None` hors bornes) ; `Instruction` (`Copy`) : `Halt`, `Emit { channel: u16, value: i32 }`, `Wait { ticks: u32 }`, `Set { register: u8, value: i32 }`, `Add { register: u8, value: i32 }`, `Jump { target: u16 }`, `JumpIfNonZero { register: u8, target: u16 }` ; `Format::Script`, `Asset::Script(Script<'a>)`, `ReadError::Instruction`, identité `VSCR` reconnue par `of_identity`.

**Artefact littéral du scénario de référence** (68 octets, écrit à la main, indépendant des constantes ; SCR-R1) :

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

**Tests d'U1** (littéraux écrits à la main ; tests SCR-L\* dans un module de test de `script.rs` ou de `format.rs`, **jamais dans `load.rs`**, écart E-1 b).

| Test | Énoncé | Arrêt |
|---|---|---|
| SCR-R1 | artefact de 68 octets : `len() == 7`, chaque instruction égale à la table | oui, un octet divergent |
| SCR-R2 | chaque préfixe tronqué (0 à 67 octets) : `Truncated` | |
| SCR-R3 | identité altérée octet par octet : `FormatIdentity` ; texte d'un `script.toml` de longueur suffisante : `FormatIdentity` | oui |
| SCR-R4 | versions `0`, `2`, `0x0100`, `0xFFFF` : `Version` | |
| SCR-R5 | déclarée `< 2` : `PayloadLength` ; `>` restants : `Truncated` ; `<` restants (≥ 2) : `TrailingBytes` ; surplus d'un octet : `TrailingBytes` ; déclarée = restants mais ≠ `2 + 8·n` : `PayloadLength` | |
| SCR-R6 | zéro instruction conforme (`len() == 0`, `is_empty()`, `instruction(0) == None`) ; charge de longueur nulle : `PayloadLength` | |
| SCR-R7 | `n = 0xFFFF` avec peu d'octets : rejeté sans panique ni débordement ; `n = 0x2000` avec charge de 2 octets : `PayloadLength` (un `8·n` sur 16 bits vaudrait 0) ; déclarée `u32::MAX` avec peu d'octets : `Truncated` | oui |
| SCR-R8 | un enregistrement invalide par cas, positions distinctes : opcode inconnu (7 et 255), registre `SCRIPT_REGISTERS` pour `Set`, `Add`, `JumpIfNonZero`, cible `n` pour `Jump` et `JumpIfNonZero`, `Wait` de zéro tick : tous `Instruction`. **Non rejetés (champs ignorés)** : pour chaque opcode, chaque champ ignoré non nul (`0xFF` registre, `0xFFFF` `arg16`, `0xFFFF_FFFF` `arg32`) : `Halt` (registre, `arg16`, `arg32`), `Emit` (registre), `Wait` (registre, `arg16`), `Set` et `Add` (`arg16`), `Jump` (registre, `arg32`), `JumpIfNonZero` (`arg32`) ; accepté et instruction décodée **égale** à celle de l'enregistrement aux champs ignorés nuls ; en outre cible `n − 1` valide et `Wait` de `u32::MAX` | oui |
| SCR-R9 | `instruction(i)` : dernier indice accessible, premier inaccessible et `u16::MAX` : `None` ; script vide : `None` | |
| SCR-R10 | script de cardinalité maximale en binaire (tableau ci-dessous) | |
| SCR-K1 | `SCRIPT_REGISTERS == 4`, `SCRIPT_MAX_STEPS_PER_TICK == 16`, `SCRIPT_INSTRUCTION_LEN == 8`, `SCRIPT_FIXED_PAYLOAD_LEN == 2` ; message d'échec renvoyant au plan § 6 (valeurs provisoires) | |
| SCR-L1 | table de 6 formats : un artefact `Script` lié à un slot attendant `Sprite`, `Tileset`, `Map` ou `Font` : `LoadError::SlotFormat`, compteur de dépôts nul ; réciproquement chacun des quatre autres formats lié à un slot attendant `Script` : `SlotFormat` ; les cinq formats, aucune paire diagonale | oui |
| SCR-L2 | lot `{script, script}`, manifeste littéral avec `artifact_index ≠ slot_index` : deux dépôts, valeurs égales aux littéraux (`Asset::Script`), chaque slot reçoit son script | oui |
| SCR-L3 | lot dont un artefact porte un `Wait` de zéro tick (puis, séparément, un registre hors bornes), placé **après** des artefacts valides : `LoadError::Artifact { error: ReadError::Instruction, .. }` avec l'index attendu, **compteur de dépôts nul** | oui (tout-ou-rien) |
| SCR-L4 | **non-régression** : tous les tests de l'étape 5 passent, assertions inchangées | oui (G-CM) |

Cardinalité maximale :

| Test | Natif | Miri |
|---|---|---|
| SCR-R7 | complet | complet (rejet par longueur, aucune boucle sur `n`) |
| SCR-R10 (maximal accepté ; un octet de moins : `Truncated`) | **65 535 instructions** par boucle (`10 + 2 + 8·65 535` octets) ; registres 0, cibles 0, `Wait 1` ; premier, milieu, dernier décodés et comparés à des littéraux | **512 instructions** (`if cfg!(miri)`), mêmes assertions ; la couverture du maximum exact reste native |

Tests de l'étape 5 à ne pas modifier : `assets` (`l1` à `l5`, `m1` à `m8`, `p1_1` à `p1_6`, `sf1` à `sf6`, `rf1` à `rf6`, `tls_*`, `map_*`, `fnt_*`), `forge` (`f1` à `f8`, `p2_*`, `map_f*`, `fnt_f*`, `fs_*`, `tests/pipeline.rs`), `shell` (`lifecycle`, `pacing`, `acquisition`, `acceptance`, `frame_loop`, `borrowed_resource`, `level_loaded`, doctests LT-3 à LT-5t).

**Mutations d'U1** (18 lignes ; copies isolées ; équivalentes marquées).

| # | Mutation | Détecteur | Autres échecs attendus |
|---|---|---|---|
| M-1.1 | opcode inconnu décodé comme `Halt` | SCR-R8 (opcode) | SCR-L3 si son artefact emploie un opcode inconnu (non requis) |
| M-1.2 | registre non contrôlé pour `Set` (puis `Add`, puis `JumpIfNonZero` : trois copies) | SCR-R8 | — |
| M-1.3 | cible non contrôlée pour `Jump` (puis `JumpIfNonZero`) | SCR-R8 | — |
| M-1.4 | `Wait` de zéro accepté | SCR-R8 | SCR-L3 |
| M-1.5 | registre contrôlé aussi pour `Emit`, `Jump`, `Wait`, `Halt` | SCR-R8 (champs ignorés) | — |
| M-1.5b | `arg16`/`arg32` non nul sur un opcode qui l'ignore rejeté | SCR-R8 (champs ignorés) | — |
| M-1.5c | un champ ignoré influence le résultat (cible de `Jump` composée d'`arg16` et des deux octets bas d'`arg32`) | SCR-R8 (égalité) | — |
| M-1.6 | `arg32` lu en big-endian | SCR-R1 | SCR-R9 |
| M-1.7 | `arg16` lu en big-endian | SCR-R1 | — |
| M-1.8 | longueur déduite sur 16 bits | SCR-R7 (`0x2000`) | — |
| M-1.9 | longueur déduite en `u32` | **(équivalente)** : `n ≤ 0xFFFF` donne `2 + 8·n ≤ 524 282 < 2³²` | — |
| M-1.10 | `instruction(index)` accepte `index == n` par garde décalée, accès par `get` | **(équivalente)** : `get` borné rend déjà `None` | — |
| M-1.10′ | garde décalée **et** indexation directe `records[offset..offset+8]` | SCR-R9 (panique rapportée comme telle) | SCR-R6 |
| M-1.11 | `of_identity` ne reconnaît pas `VSCR` | SCR-L1 (`Artifact { FormatIdentity }` au lieu de `SlotFormat`) | SCR-L2 inchangé |
| M-1.12 | `decode` envoie `Script` au lecteur de `Font` | SCR-L2 | SCR-L3 |
| M-1.13 | `Script::read` ne valide aucun enregistrement | SCR-R8 | SCR-L3 |
| M-1.14 | `SCRIPT_MAX_STEPS_PER_TICK = 17` | SCR-K1 **seul** | — |
| M-1.15 | `Instruction` renvoyée avant la vérification de longueur | SCR-R5 | SCR-R7 |

## 9. Arbitrages et limites clos (non rouverts)

- **A-S1 à A-S5** [Validé] : le script est un Asset au même mécanisme que tileset, map, font ; validation structurelle des instructions dans `Script::read` (A-S3) ; `Wait 0` rejeté ; `Waiting ⇒ wait ≥ 1` ; analyse de flot sur tout le graphe avec coupure aux `Wait` (Forge, U2) ; `TRANSCRIPT` remplacé à chaque tick ; `ScriptTable` indexée directement par `SlotIndex` ; encodage canonique de l'état exclusivement pour la reconstruction.
- **Valeurs 4 et 16** : provisoires, propres à cette première implémentation, non invariants du moteur ; épinglées par SCR-K1 seulement.
- **Le lecteur ne vérifie ni cycles ni budget** (validation de la Forge, plan C12, § 4.B) ; le script ne référence aucun autre asset ; `load.rs` et `manifest.rs` inchangés.
- **Écarts d'exécution tranchés** : **E-1** (b) tests SCR-L\* hors `load.rs` ; **E-5** types `usize` des constantes ; **E-6** `is_empty` ajouté ; **E-10** (accepté le 3 novembre 2026) : toute mutation non équivalente est détectée par un test nommé, une mutation démontrée équivalente est rapportée avec justification sans test artificiel (lecture du critère du plan § 14.2) ; une modification de signature publique n'est pas une mutation de comportement.
- **Leçons de l'étape 5 applicables** : E-13 (le parsing TOML sous Miri est prohibitif : sans objet en U1, aucun TOML) ; E-14 (`dead_code` sous MSRV 1.85 pour les constantes utilisées seulement par une `const` assert).

## 10. Interdits

- Ne modifier ni les ADR, ni `ROADMAP.md`, ni `docs/implementation/00` à `05`, ni les handoffs, ni `crates/{ecs,schedule,render,verbum}/**`, ni `crates/engine/shell/src/**` ; ne pas modifier le plan, le document d'exécution ni ce handoff sans décision explicite de l'utilisateur.
- Écrire uniquement dans `crates/engine/assets/src/{script.rs (créé), lib.rs, format.rs, sprite.rs (ajout seul)}`. Ne modifier ni `load.rs`, ni `manifest.rs`, ni `header.rs`, ni `map.rs`, ni `font.rs`, ni `tileset.rs`, ni aucun `Cargo.toml`, ni `crates/engine/forge/**` (U2), ni créer `crates/script/**` (U3).
- **Aucune dépendance** ajoutée à `verbum-assets` (`[dependencies]` et `[dev-dependencies]` restent vides).
- Aucun `unsafe`, `alloc`, `Vec`, `Box`, `String` ; aucun cast d'octets, `transmute`, `Box::leak`, `include_bytes!`, `static`.
- Aucune chaîne dans l'artefact ; aucun opérande, instruction ou catégorie hors du jeu fermé du plan § 4.D ; aucun flottant ; aucun constructeur public d'`AssetId` à partir d'un entier.
- Aucun trait public ; aucun test ni assertion hors liste du plan et du document d'exécution ; aucun test ni mutation d'U2 à U5.
- **Aucun commit par Claude.**

## 11. Divergence, doute, blocage

Arrêt immédiat, sans choix de la session, en cas de : contradiction ou ambiguïté du plan ou d'un ADR ; divergence entre un fait [Vérifié] (F3 à F6, F14) et le code réel fourni ; **situation G-CM** (création, suppression ou réordonnancement d'un niveau, dépôt anticipé, modification de `VMAN`, allocation dans `load`, modification du modèle d'identité, assertion d'un test de l'étape 5 à modifier) ; un `match` exhaustif cassé par une variante ajoutée (**exposé avant correction**, R-ADAPT) ; un octet de SCR-R1 qui diverge du littéral ; test dont le rouge ne peut pas être constaté pour la bonne raison ; **mutation non détectée et non établie équivalente**, ou qui échoue sur d'autres assertions que celles annoncées sans explication ; besoin de modifier un fichier interdit ; besoin d'`unsafe`, d'allocation, d'un trait public ; test hors liste ; fichier nécessaire non fourni ; durée Miri jugée excessive (rapporter et arbitrer, aucun `--skip` ni `ignore` sans décision). La session fournit un constat factuel (source, texte, observation) et présente les options minimales, **sans arbitrer**.

**Points à confirmer par l'implémentation** (non résolus à la rédaction) : (a) présence éventuelle d'autres `match` exhaustifs sur `Format`, `Asset` ou `ReadError` hors `format.rs::decode` ; (b) forme exacte de `format.rs` (`of_identity`, `decode`, `Decoded::Ready`) sur le fichier fourni ; (c) expression exacte de la garde de cohérence des constantes à la compilation, à reprendre de `font.rs` ; (d) placement des tests SCR-L\* (`script.rs` ou `format.rs`), rapporté ; (e) durée de Miri pour la suite complète de `verbum-assets` (non mesurée) ; (f) noms, layouts et signatures provisoires [L], tout ajustement listé au rapport sans changement de sémantique.

## 12. Rapport de fin d'unité et reprise

**Gabarit** (rien de narratif) : 1. Unité, état (livrée, en attente de validation). 2. Fichiers créés et modifiés (chemins complets), fichiers livrés par `SendUserFile`. 3. Faits [Vérifié] revérifiés : confirmés ou divergence. 4. Rouge : test, motif d'échec observé, ou « sans rouge possible », motif. 5. Quatre contrôles : lignes réelles, espace de la session et dépôt réel séparés ; contrôles non exécutés nommés. 6. Mutations : mutation, détecteur, autres assertions échouées, mode de détection ; équivalentes avec justification. 7. Portes : G-CM (aucun arrêt, ou arrêt rapporté) ; liste R-ADAPT. 8. Garanties : statiques / revue, sans mélange. 9. Natif, Miri (intégral ou partiel, réductions listées). 10. Ajustements de noms et écarts du plan ou du document d'exécution. 11. Regroupement de commit proposé ; `tree.md` à régénérer. 12. Fin : « En attente de la validation de l'unité U1. »

**Attente, pas constat** : rouge constaté ; vert ; 18 mutations traitées (M-1.9 et M-1.10 équivalentes, M-1.10′ détectée) ; quatre contrôles verts ; Miri vert. **Critère de sortie** : SCR-R\*, SCR-K1, SCR-L1 à SCR-L4 verts ; contrôles propres conformes ; écarts de noms listés.

**Reprise** : si la session se dégrade, une nouvelle session reçoit ce handoff, le document d'exécution, le plan, les documents du § 3, `tree.md` et les fichiers du § 7 ; elle relit l'état réel par les contrôles **exécutés par l'utilisateur** ; une unité interrompue est refaite depuis ses tests ; elle ne rouvre aucun arbitrage clos. Après validation d'U1 (« unité U1 validée »), l'utilisateur demande le handoff d'U2 ; aucune session ne traite deux unités.

## 13. Consigne d'amorce

*Cette consigne prévaut sur celle du § 0 du document d'exécution lorsque le handoff est fourni.*

> Vous implémentez **uniquement l'unité U1** de l'étape 6 de Verbum : le lecteur de l'artefact `script.bin` dans `verbum-assets` (`crates/engine/assets`). Lisez, dans cet ordre : `06-verbum-scripts-implementation-handoff.md`, `06-verbum-scripts-execution.md` (règles § 1, interdits § 2, faits § 3, unité U1 du § 4, § 5, § 6, § 9), `06-verbum-scripts.md` (plan r1.2, une seule fois ; § 0 à 3, § 4.A, § 4.B, § 5, § 6, § 7.1, § 8 ligne U1, § 9), `ADR-Sc01`, `ADR-A01`, `ADR-000` (points 1, 3, 4), `05-verbum-assets-execution.md` (§ 3.2 et E-12 à E-14 seulement), puis `tree.md`. Vous n'avez pas accès au dépôt : les contrôles réels et Miri sont exécutés par l'utilisateur, les fichiers sont livrés complets (pas de diff), les fichiers d'U1 vous sont fournis nommément. N'abordez en aucun cas U2, U3, U4 ni U5. N'écrivez aucun code et ne lancez aucune commande avant que l'utilisateur ait dit exactement : « démarrez l'unité U1 ». Accusez réception en cinq lignes maximum, en indiquant le statut des documents tel qu'au § 5.
