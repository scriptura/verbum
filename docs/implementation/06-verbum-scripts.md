# Plan de construction — étape 6 : Scripts — r1.2, 1er novembre 2026

## Statut
Document de planification interne, non normatif, révisable sans ADR (même statut que `ROADMAP.md` et les plans des étapes 2 à 5). Il détaille l'étape 6 en unités. Il n'introduit aucun invariant : chaque exigence cite sa source, et tout ce qui ne découle pas du corpus est signalé comme tel. **Aucun code, aucune modification d'ADR, de document validé ou de dépôt.**

**État : r1.2, soumis à validation de l'utilisateur. Les arbitrages A-S1 à A-S5 et les choix locaux du §15 de r1.1 ont été rendus le 1er novembre 2026 (§5). Le document d'exécution et le handoff ne sont pas produits avant la validation de r1.2.**

**Révision r1.2 — passages modifiés, et seulement ceux-ci :**
1. **§5** : arbitrages A-S1 à A-S5 consignés comme **retenus** ; options non retenues conservées comme historique.
2. **§4.F (A-S5)** : trois propriétés explicitées (remplacement intégral du transcript à chaque tick, y compris sans émission ; aucune émission d'un tick précédent ne subsiste dans le World ; l'historique relève d'un observateur extérieur) ; capacité du transcript ; énoncé de la borne des émissions et statut de la branche `Err(Full)` (violation de contrat, non politique de saturation).
3. **§4.D** : convention d'analyse de flot et de budget reprise telle que validée ; propriété « un script accepté par la Forge n'atteint jamais `Faulted(Budget)` ».
4. **§4.C** : bornes exactes de `SlotIndex` (`u32`) et conversion vers `u16` ; aucune table de correspondance ni compactage.
5. **§4.G (nouveau)** : table de cohérence lecteur / Forge / `step` / tests / états canoniques / capacités.
6. **§6** : valeurs 4 et 16 déclarées **provisoires, propres à cette première implémentation**, non invariants du moteur ; tests de budget paramétrés par les constantes ; test d'épinglage SCR-K1.
7. **§7** : tests ajoutés ou précisés : SCR-F4 (g, h : les deux successeurs de `JumpIfNonZero`), SCR-K1, EM-3, SR-1 (assertions de transcript), SR-7 (script accepté par la Forge jamais en `Faulted(Budget)`) ; mutations associées (§7.6). Faute corrigée : « Toute état » devient « Tout état » (ST-4).
8. **§1.3** : critère T15 ajouté ; T10 complété. **§10, §11, §15** mis à jour.

Layouts, jeu d'instructions, unités, portes : inchangés par rapport à r1.1.

Légende : **[Lu]** texte vérifié dans un document ou un source lu dans cette session · **[Imposé]** disposition normative citée · **[Déduit]** conséquence que je tire de textes lus · **[L]** choix local, non invariant, révisable · **[C]** ambiguïté, contradiction ou lacune · **[A]** arbitrage demandé à l'utilisateur · **[Non vérifié]** non lu ou non exécuté.

## 0. Sources et limites

**[Lu] normatif** : `00-principes.md`, `01-runtime-ecs.md`, `02-concepts.md`, `README.md` d'architecture, ADR-000, 001, 002, 003, 004, 005, 006, A01, S01, S03, SV01, **Sc01**.
**[Lu] non normatif** : `ROADMAP.md`, `tree.md` (1er novembre 2026, 02:41), plans, exécutions et handoffs des étapes 0 à 5 (étape 0 : plan et handoff ; étape 1 : plan r5, sessions, handoff ; étape 2 : plan r8, exécution, handoff ; étape 3 : plan, exécution, handoff ; étape 4 : plan r3.2, exécution, handoff ; étape 5 : plan r1.3, exécution à jour après U5 (E-12 à E-14), handoff).
**[Lu] code réel** : `crates/engine/schedule/src/{scheduler,contract,system,access}.rs` ; `crates/engine/ecs/src/{lib,resource,component_set,command,world,entity,values,storage}.rs` ; `crates/engine/assets/src/{lib,header,format,load,manifest,sprite,tileset,map,font}.rs` ; `crates/engine/forge/src/{lib,batch,id,sprite,map}.rs` ; `crates/engine/shell/src/lib.rs`, `tests/{borrowed_resource,level_loaded}.rs`, `examples/level_headless.rs` ; les `Cargo.toml` (racine, `ecs`, `schedule`, `assets`, `forge`, `shell`, `render`, `verbum`) et `Cargo.lock`.

**Non lus, donc non vérifiés** :
- ADR-R01 à R03, ADR-AU01, ADR-N01 : connus par citations seulement. Aucune décision de ce plan n'en dépend.
- `crates/engine/forge/src/{font,tileset}.rs`, `crates/engine/shell/src/{shell,inbox,pacing}.rs`, `crates/engine/ecs/src/{query,fixtures}.rs`, `crates/engine/render/**` : signatures connues par leurs usages dans les documents et tests lus. Les faits F10 du document d'exécution de l'étape 3 sont à revérifier au démarrage de l'unité qui les emploie.
- **La page d'axes du domaine Scripts** (Modèles 1, 3, 4), citée par ADR-Sc01 : non fournie. La comparaison du §4.A est faite sur les seuls textes lus ; **elle n'est pas présentée comme exhaustive**.
- Aucune commande n'a été exécutée, aucune compilation tentée : toute affirmation de compilabilité est un **[Déduit]** à confirmer par les portes d'unité.

ADR-S02, S04 et S05 sont annoncés par ADR-S01 mais **non rédigés** : toute mention de « politique temporelle » vise la future ADR-S04, qui n'existe pas. Toute durée de script s'exprime en ticks.

## 1. Objectif et critères de sortie

### 1.1 Contrat de la Roadmap [Lu]
« **Contenu** : premier système interpréteur, interface déclarative fixe, validation par la Forge de l'ensemble de données accessible au script. **Dépendance** : étapes 2 et 5. **Test d'invariant** : un script suspendu à mi-exécution ne laisse aucune pile hors du World — l'état repris après un « yield » doit être bit-à-bit reconstructible depuis le seul composant/Ressource qui le porte (ADR-Sc01, point 4). Le test doit couvrir plusieurs ticks entre suspension et reprise […]. **Artefact** : premier dialogue ou comportement scripté exécuté de bout en bout. » Règle de jalon : validé seulement si aucun invariant architectural n'a dû être modifié.

### 1.2 Périmètre retenu [Déduit]
Un type d'asset de plus (**script**), au même mécanisme que tileset, map et font ; un petit crate d'exécution pur (état, jeu d'instructions, pas d'interprétation) ; un système interpréteur d'assemblage à interface fixe ; une preuve de suspension/reprise ; un démonstrateur headless `Forge → Asset → load → World → Scheduler → interpréteur → Shell`.

**Portée explicite de cette première version.** Un script ne lit **aucune donnée métier du `World`** : il ne dispose que de l'état propre de son instance (registres, position, délai) et d'émissions génériques (canal, valeur). `JumpIfNonZero` ne teste que les registres de l'instance. Ce périmètre est le minimum de l'étape ; il **n'est pas un système de comportement conditionnel complet** (pas de condition sur un composant, une Ressource ou une autre entité).

### 1.3 Critères de clôture

| # | Exigence démontrée | Source | Preuve |
|---|---|---|---|
| T1 | Un script est un Asset : source lue par la Forge seule, artefact binaire, chargement tout-ou-rien, Ressource ordinaire, sans changement de mécanisme | ADR-A01 ; Sc01 pt 1 ; Roadmap | SCR-R*, SCR-F*, SCR-L*, SCR-C1 ; porte G-CM |
| T2 | Le runtime ne parse jamais la source | `00-principes.md` inv. 1 | SCR-R3 ; arbre de dépendances |
| T3 | Interface du système interpréteur fixe : ensemble borné de catégories, jamais variable selon le script | Sc01 pts 2 et 4 ; ADR-004 | CT-1, CT-2, CT-3 |
| T4 | Toute l'information de reprise est portée par un composant (position, variables, délai) | Sc01 pts 1 et 4 | ST-*, SR-2, SR-4 |
| T5 | Aucune primitive de suspension du langage hôte, aucun état privé survivant à une itération | Sc01 pts 4 et 5 ; ADR-003 pt 2 | Revue (§7.5), SR-4 |
| T6 | Un « yield » est un programme-compteur explicite stocké comme champ ordinaire | Sc01 pt 4 | IN-*, SR-1, SR-3 |
| T7 | Reprise bit-à-bit après plusieurs ticks, autres systèmes exécutés dans l'intervalle | Roadmap ; Sc01 pt 4 | SR-1, SR-2, SR-3 |
| T8 | Unique propriétaire d'écriture de l'état d'exécution | Sc01 pt 2 ; `01-runtime-ecs.md` §4 | CT-2 ; `MultipleWriters` |
| T9 | Déterminisme (flottants exclus, ticks seuls) | `00-principes.md` inv. 3 | IN-det, SR-2 |
| T10 | Bornes statiques : budget par tick vérifié par la Forge et appliqué au runtime ; un script accepté par la Forge n'atteint jamais `Faulted(Budget)` ; capacités des émissions et du transcript prouvées suffisantes à la compilation | ADR-001 pt 5 ; ADR-000 pt 4 | SCR-F4, IN-budget, IN-bound, IN-end, SR-7, EM-2, EM-3 |
| T11 | La Forge rejette tout opérande hors de l'ensemble fixe, tout cycle sans yield et tout chemin dépassant le budget | Sc01 pt 4 | SCR-F3, SCR-F4 |
| T12 | Premier comportement scripté de bout en bout, headless, observable dans le Shell | Roadmap | DM-1, exemple |
| T13 | Aucun invariant, ADR ni crate interdit modifié | principe directeur | Contrôles manuels (§9) |
| T14 | États canoniques : toute combinaison `status`/`fault`/`wait` impossible est inconstructible ou rejetée ; correspondance exacte entre slot de chargement et `ScriptTable` | §4.C | ST-2, ST-4, TB-1, SCR-C1 |
| T15 | Les sorties d'un tick ne survivent pas à ce tick dans le World ; l'historique relève d'un observateur extérieur | §4.F (A-S5) ; `01-runtime-ecs.md` §4 | SR-1 (assertions de transcript), CT-3 ; mutations §7.6 |

## 2. Exigences documentées et source

| Id | Exigence | Source | Statut |
|---|---|---|---|
| X1 | Un script est un Asset produit par la Forge, jamais du code connu à la compilation du moteur | Sc01 pt 1 ; ADR-A01 pts 1 à 3 | Imposé |
| X2 | Il est interprété par un ou plusieurs systèmes **ordinaires** satisfaisant ADR-003, jamais un second ordonnanceur | Sc01 pts 1 et 5 | Imposé |
| X3 | Aucun état d'exécution privé ne survit à une itération ; position, variables, délais sont portés par un composant ou une Ressource | Sc01 pt 1 | Imposé |
| X4 | Aucune pile d'exécution suspendue **hors du World** ; aucune primitive de suspension du langage hôte (`async`, thread, fibre, coroutine) | Sc01 pt 4 | Imposé |
| X5 | Un yield est un programme-compteur explicite, champ ordinaire d'un composant ou d'une Ressource | Sc01 pt 4 | Imposé |
| X6 | Interface déclarative de l'interpréteur fixe, couvrant un ensemble borné de composants et Ressources désigné une fois pour toutes | Sc01 pt 4 ; ADR-004 | Imposé |
| X7 | La Forge valide, script par script, qu'aucune donnée hors de cet ensemble n'est référencée ; violation = erreur de compilation du script | Sc01 pts 4 et 5 | Imposé |
| X8 | L'état d'exécution a un unique propriétaire en écriture : le système interpréteur | Sc01 pt 2 | Imposé |
| X9 | L'interpréteur ignore le sens métier de ce qu'il joue | Sc01 pt 3 | Imposé |
| X10 | Seule la Forge lit un format humain ; aucune chaîne ni chemin ne franchit la frontière de compilation | `00-principes.md` inv. 1 ; ADR-A01 pt 3 | Imposé |
| X11 | Aucune validation de cohérence métier au runtime ; conformité binaire et valeurs invalides vérifiées au chargement | `00-principes.md` inv. 1 et 5 ; ADR-000 pt 3 et pt 4 (table) | Imposé |
| X12 | Capacités fixées avant simulation ; dépassement = erreur de configuration, jamais croissance ni cas silencieux | ADR-001 pt 5 ; ADR-000 pt 4 | Imposé |
| X13 | Le noyau est `no_std` sans `alloc` ; aucun `unsafe` dans le code des systèmes | ADR-000 pts 1 et 4 | Imposé |
| X14 | Un système ne décide pas quand il s'exécute, n'a aucun état caché, ne raisonne que sur son interface déclarative | ADR-003 pts 2, 3, 5 | Imposé |
| X15 | Produire est libre ; résoudre est unique | `01-runtime-ecs.md` §4 | Imposé |
| X16 | Un concept n'est codé que s'il est porté par un invariant ou rendu nécessaire par un test | I11 ; règle gouvernante | Imposé |
| X17 | Aucun vocabulaire de jeu dans le noyau ni dans les tests génériques | handoff étape 2 §4.5 | Imposé |
| X18 | Mécanisme de l'étape 5 inchangé : niveaux 1 à 4, `VMAN`, tout-ou-rien, aucune allocation, `AssetId` opaque | Roadmap étape 5 ; plan 05 §1.3 (G-CM) | Imposé |

## 3. Ambiguïtés, contradictions et lacunes

**Aucune contradiction démontrée avec un ADR ou un contrat adopté.** Les points suivants sont des ambiguïtés ou des lacunes, traitées dans la section indiquée.

| # | Constat | Traitement |
|---|---|---|
| C1 | **Cardinalité du porteur.** Sc01 pt 1 dit « un composant ou une Ressource » ; une Ressource est de cardinal 1 par `World` (ADR-005 pt 1), or plusieurs entités ont chacune leur instance. | §4.C : composant par entité, après comparaison. |
| C2 | **Portée exacte de X4.** Le texte interdit une pile suspendue **hors du `World`**. Il n'interdit pas une structure bornée de type pile **portée par un composant**. | §4.C : le jeu minimal n'a pas d'appel, donc pas de pile (X16). Une pile à capacité fixe portée par le composant resterait conforme. Aucune portée plus restrictive que le texte n'est posée. |
| C3 | **« Interface fixe décidée à la conception du moteur »** face à `Locate`/`Binding` : les identifiants de catégorie sont attribués par l'application, une fois par monde (plan 01 §2.2). | §4.B : l'ensemble fixe est une propriété de l'interface déclarative de l'adaptateur et du type de `step` ; les entiers d'identité restent un choix d'assemblage. Fixité vérifiée par un contrôle de composition (CT-1), non imposée par les types : même limite que C14 de l'étape 3. |
| C4 | **« Validation par la Forge de l'ensemble de données accessible. »** Aucune instruction ne peut nommer une donnée hors de l'ensemble : le jeu d'instructions est fermé. | §4.B : la validation porte sur les opérandes, la terminaison des boucles et le budget. |
| C5 | **Unités de temps.** `00-principes.md` inv. 1 attend de la Forge une conversion des unités humaines en ticks ; la politique temporelle relève d'une future ADR-S04, non rédigée. | La source s'exprime en ticks (précédent R4 de l'étape 2). Conversion secondes → ticks non démontrée (§10). |
| C6 | **Ordre inter-instances.** L'ordre dense d'un stockage n'est pas stable (ADR-001 pt 4, swap-remove) ; l'ordre d'émission entre instances ne l'est donc pas non plus. | Aucun test n'affirme cet ordre (porte G-ORD-S, §5). Comparaisons en multiensemble. |
| C7 | **E-12** (étape 5) : sous `Binding<'a>`, un système dont l'accès ne porte aucun type en `'a` ne compile pas (E0477). | §5, A-S4. L'interpréteur n'est pas concerné (son accès nomme `ScriptTable<'a>`). |
| C8 | **Capacité des émissions.** `System::run` n'a pas de canal d'erreur propre (C7 de l'étape 4, ouvert). | §4.F : la borne est **prouvée suffisante à la compilation** pour tout artefact, grâce au budget appliqué au runtime ; la branche `Err(Full)` de l'adaptateur est inatteignable : sa détection est une **violation de contrat**, non une politique de saturation. |
| C9 | **Lancement d'un script.** Le texte parle de « lancement » sans en fixer le vecteur. | Lancement = ajout du composant d'état par `CommandBuffer::add_component` + `World::apply` (ADR-006), ici à l'assemblage. Lancement depuis un système (`Commands<ADD>`) non démontré (§10). |
| C10 | **« Bit-à-bit »** non défini par la Roadmap. | §4.E : définition par encodage canonique de l'état. |
| C11 | Ajouts additifs à l'API publique de `verbum-assets` (`Format::Script`, `Asset::Script`, `ReadError::Instruction`, constantes de layout et de budget). | §5, A-S3 ; non-régression de l'étape 5. Les seuls `match` exhaustifs vus sur ces énumérations sont `format.rs::decode` (sur `Format`) et `batch.rs` (sur `AssetKind`) ; les autres `match` lus ont un bras par défaut ou des motifs ciblés. À rechercher à nouveau au démarrage de U1 et U2. |
| C12 | **`Wait 0`** : sémantique ambiguë (instant transitoire, représentation `Waiting + wait = 0`). | **Rejeté** : lecteur (`ReadError::Instruction`) et Forge (`ZeroValue("ticks")`). Conséquence : `Waiting ⇒ wait ≥ 1` dans tout état persistant (§4.C). |
| C13 | **Correspondance slot ↔ table.** `ScriptState.script` désigne un slot de chargement ; le lot mêle d'autres formats. | §4.C : `ScriptTable` indexée directement par `SlotIndex`, slots non scripts vacants, aucun compactage ni table parallèle. |
| C14 | **Budget et chemins acycliques.** Rejeter les cycles sans yield ne borne pas un chemin linéaire plus long que le budget. | §4.D : calcul du plus long chemin par la Forge ; `Faulted(Budget)` reste la défense runtime. |
| C15 | Un journal persistant de capacité fixe ne peut pas absorber un flux sans fin. | §4.F : journal supprimé, `TRANSCRIPT` transitoire remplacé à chaque tick (A-S5). |
| C16 | **Valeurs 4 et 16.** `SCRIPT_REGISTERS` et `SCRIPT_MAX_STEPS_PER_TICK` ne découlent d'aucun texte du corpus. | §6 : **provisoires, propres à cette première implémentation**, non invariants du moteur ; épinglées par SCR-K1 ; tests de budget paramétrés par les constantes. |

## 4. Analyses

### 4.A Représentation du script (source, artefact, rôles)

**Options comparées** (le plan ne présuppose ni bytecode ni langage tiers). **Limite de source : la page d'axes d'ADR-Sc01 n'a pas été lue ; cette comparaison porte sur les textes lus et n'est pas exhaustive.**

| Option | Description | Verdict |
|---|---|---|
| a. Programme linéaire d'instructions de taille fixe, interprété | Tableau d'enregistrements de 8 octets décodés à la demande | **Retenue** |
| b. Arbre syntaxique interprété | Nœuds à pointeurs ou indices, évaluation récursive | Écartée : récursion non bornée statiquement, indirections, état d'évaluation intermédiaire à externaliser |
| c. Table de transitions d'une machine à états | Sous-cas de (a) sans registre | Écartée : ne porte aucune variable, donc ne démontre pas « variables » (Sc01 pt 1) |
| d. Génération de code Rust par script (Modèle 4) | Chaque script devient un système | Écartée : recompilation du moteur par contenu, génération de code à la compilation (Forge = bibliothèque, ni `build.rs` ni CLI, étape 2) ; ADR-Sc01 pt 6 en fait un arbitrage de production, non architectural ; la Roadmap demande un « système interpréteur » |
| e. Langage tiers embarqué | Interpréteur externe | Écartée : allocation, dispatch dynamique, dépendances, `no_std` sans `alloc` (ADR-000) |

**Représentation source [L]** : un fichier TOML par script ; tableau de tables `[[step]]` avec un champ `op` (chaîne, jeu fermé) et les opérandes de l'opération ; champ optionnel `label` par pas. Les noms d'opérations et d'étiquettes n'existent qu'à la Forge (X10).
**Représentation compilée [L]** : artefact `script.bin`, identité `VSCR`, préfixe commun de 10 octets, charge utile : nombre d'instructions `u16`, puis `n` enregistrements de 8 octets (§6). Charge utile variable : même traitement que `font.bin` et `map.bin` (A-2 option a de l'étape 5 : vue empruntée, octets possédés par l'infrastructure de chargement).
**Rôles** : la Forge parse, résout les étiquettes en positions, valide (X7, X11), encode ; le runtime (`verbum-assets`) vérifie la conformité binaire de l'artefact et décode à la demande ; l'exécution relève de `verbum-script` (§4.F). Le chargement intègre le script **par les mécanismes existants** : `Format::Script` attendu par slot, dispatch par identité, double décodage, dépôt de `Asset::Script(Script<'a>)`. `load.rs` et `manifest.rs` restent inchangés (comme pour la police) ; le script ne référence aucun autre asset, donc aucune fermeture de référence n'est ajoutée.

### 4.B Interface déclarative fixe

**Ensemble fixe [L]** : trois catégories, désignées par la conception du moteur :

| Catégorie | Nature | Mode de l'interpréteur | Contenu |
|---|---|---|---|
| `STATE` | composant, un par entité porteuse | `Write` | `ScriptState` (§4.C) |
| `SCRIPTS` | Ressource persistante | `Read` | `ScriptTable<'a, CAP>` : vues sur les scripts chargés, indexées par slot |
| `EMISSIONS` | Ressource transitoire | `Produce` | `Emissions<CAP>` : intentions émises (canal, valeur) |

Contrat dérivé du type d'accès (plan 01 §2.1) : `reads = {SCRIPTS}`, `writes = {STATE}`, `produces = {EMISSIONS}`, aucun effet structurel. Aucun script ne peut élargir cet ensemble : le jeu d'instructions est fermé, et ses opérandes ne désignent que les registres de l'instance, des positions du même script, un canal et une valeur immédiate. **Ce qu'un script « lit » d'autre — un composant d'une autre entité, une Ressource de jeu — n'est pas exprimable à cette étape** (§1.2) ; l'étendre exigerait un nouvel opérande et une nouvelle catégorie fixe, décision de conception du moteur, jamais une variation par script.

**Validation par la Forge (X7)** : registre `< SCRIPT_REGISTERS`, cible de saut = étiquette existante, canal et valeurs dans leur intervalle, `ticks ≥ 1`, cycle sans yield rejeté, chemin entre deux yield borné par le budget (§4.D). La violation est une `ForgeError`, détectée avant que le runtime ne consomme l'artefact. Côté runtime, le lecteur applique la **vérification binaire correspondante** (opcode connu, registre et cible dans les bornes, `ticks ≥ 1` ; ADR-000 pt 4, « valeurs invalides vérifiées au chargement »), de sorte que `step` est total sur tout artefact chargé (A-S3). Le lecteur ne vérifie ni les cycles ni le budget (analyse de flot de contrôle : validation de la Forge, I2) ; le budget runtime les couvre.

**Cohérence avec `Binding`/`Locate`/`Bind` [Lu]** : le système interpréteur est un système ordinaire dont le type d'accès est `(Write<STATE, ValuesMut<…>>, Read<SCRIPTS, &mut ResourceSlot<ScriptTable<'a>>>, Produce<EMISSIONS, &mut ResourceSlot<Emissions<N>>>)`. Les trois `impl Locate` vivent dans le `Parts` de l'assemblage. Aucune modification de `Binding`, `Access`, `Locate`, `Bind`, `Scheduler`, ni du modèle de dépendances.

### 4.C État d'exécution

**Ce que l'état doit porter** (Sc01 pt 1) : position dans le programme, variables, délai d'attente, statut. Rien d'autre n'est nécessaire à la reprise : **pas de pile**, car le jeu minimal n'a ni appel ni sous-programme (X16, C2). Les données temporaires d'un `step` sont des locaux de l'appel, morts en fin d'appel (« un système n'existe jamais entre deux itérations », ADR-003 pt 5) ; ce ne sont pas une pile de script suspendue.

**`ScriptState` [L]** : donnée pure `Copy`, sans référence ni durée de vie, **champs privés** : `script: u16` (slot de chargement du script), `pc: u16`, `status` ∈ {`Running`, `Waiting`, `Halted`, `Faulted(NoScript | Budget)`}, `wait: u32` (ticks restants), `registers: [i32; SCRIPT_REGISTERS]`. Accesseurs en lecture ; **deux constructeurs seulement** : `ScriptState::start(slot: SlotIndex) -> Result<Self, SlotTooLarge>` (état `Running`, `pc = 0`, registres nuls) et `from_bytes` (ci-dessous). `step`, dans le même crate, est la seule transition. Aucun état invalide n'est donc constructible.

**Table de validité des états canoniques** (appliquée par `from_bytes` ; vérifiée sur les trajectoires de `step` par ST-4) :

| `status` | `fault` | `wait` | Remarque |
|---|---|---|---|
| `Running` | 0 | 0 | état initial ; un état `Running` n'est jamais observé en fin de tick d'une instance exécutée |
| `Waiting` | 0 | ≥ 1 | jamais 0 (C12) |
| `Halted` | 0 | 0 | |
| `Faulted(NoScript)` | 1 | 0 | |
| `Faulted(Budget)` | 2 | 0 | |

Toute autre combinaison (`fault ≠ 0` hors `Faulted`, `Faulted` avec `fault = 0` ou > 2, `wait ≠ 0` hors `Waiting`, `Waiting` avec `wait = 0`, octet de statut inconnu) est rejetée par `from_bytes` (`StateError`). `pc` et `script` ne sont pas contraints par l'encodage : leur cohérence avec un script relève du chargement, non de l'état.

**Comparaison objective des représentations de plusieurs instances** :

| Critère | Composant par entité | Table dans une Ressource unique |
|---|---|---|
| Cardinalité 0..N, adressage par `EntityId` | natif (`SparseSet<ScriptState>`) | à reconstruire (table + correspondance indice ↔ entité) |
| Purge à la destruction d'une entité | par `World::despawn` (`remove_all`) | à réimplémenter, hors World |
| Création et retrait | `AddComponent`/`RemoveComponent` différés (ADR-006) | mutation directe de la table, contourne la validation de phase |
| Propriétaire unique en écriture | une catégorie `STATE`, un écrivain par phase | idem, mais la structure interne devient un second stockage |
| Itération déterministe | ordre dense (non stable inter-frame, ADR-001 pt 4) | ordre de la table (stable, mais découplé du World) |
| Compatibilité `ComponentSet` / `Copy` / capacité fixe | `T: Copy`, `DENSE` fixé avant simulation (ADR-001 pt 5) | `[T; N]` : même capacité, sans mécanisme existant |
| Sauvegarde déclarative (ADR-SV01) | composant déclaré persistant | Ressource déclarée persistante, avec correspondance à remapper |

**Décision** : **composant par entité** (pas un arbitrage : aucun invariant ni contrat touché). La table en Ressource duplique l'identité et la purge que le `World` assure déjà et déplace une mutation structurelle hors du point de validation. Ressources retenues pour ce qui est réellement de cardinal 1 : la table des scripts chargés et les émissions.

**`ScriptTable<'a, const CAP: usize>` — correspondance avec les slots [L, retenu]** : tableau `[Option<Script<'a>>; CAP]`, **indexé directement par le `SlotIndex` de chargement** (le seul indice que reçoit le dépôt de `load` : `FnMut(SlotIndex, Asset<'a>)`). `CAP` est la longueur de la table des formats de slot de l'application (`slot_formats.len()`). Les slots occupés par d'autres formats (sprite, tileset, map, font) restent **vacants** (`None`) : l'indice d'un script dans la table est exactement son slot dans le manifeste. **Aucun compactage, aucune table de correspondance parallèle.**

**Bornes exactes de `SlotIndex`** : `SlotIndex` enveloppe un `u32` (`SlotIndex::new(u32)`, `get() -> u32`, **sans vérification de capacité**, plan 02 §4.1). Donc :
- `ScriptTable::deposit(slot: SlotIndex, script)` convertit `slot.get()` par `usize::try_from` et refuse (`TableError`) un slot `≥ CAP` ou déjà occupé (précondition de `load` : slots vides à l'entrée) ;
- `ScriptState::start(slot: SlotIndex)` convertit `slot.get()` en `u16` par `u16::try_from` et refuse (`SlotTooLarge`) toute valeur `> u16::MAX` ; `ScriptState.script` ne peut donc désigner que les slots `0..=65 535` ;
- garde statique : `CAP ≤ 65 536` (`const` assert dans `ScriptTable`), de sorte que tout slot valide du chargement (`< CAP`) tient dans un `u16` ;
- `get(index: u16)` rend `None` si `usize::from(index) ≥ CAP` ;
- l'état ne connaît pas `CAP` : un slot `u16` valide mais vacant ou hors table donne `Faulted(NoScript)` à l'exécution (IN-miss), jamais une panique.

L'`AssetId` n'est pas constructible hors du crate `verbum-assets` et l'`ArtifactIndex` est un type distinct : `ScriptState::start` n'accepte qu'un `SlotIndex`, donc aucune confusion n'est constructible ; un test à `artifact_index ≠ slot_index` la pince néanmoins (SCR-C1, TB-1, SR-5).

**Itération** : le système copie l'`EntityId` (`Copy`) à l'indice `i` de `entities()`, puis appelle `get_mut` ; aucun emprunt ne traverse la modification, aucune allocation. Aucune mutation structurelle n'a lieu pendant la phase.

### 4.D Interpréteur et exécution déterministe

**Jeu d'instructions minimal [L]** (justifié par la scène : variable qui survit à un yield, boucle, sortie observable) :

| Opcode | Nom | Opérandes | Effet |
|---|---|---|---|
| 0 | `Halt` | — | statut `Halted` |
| 1 | `Emit` | canal `u16`, valeur `i32` | pousse une émission ; `pc += 1` |
| 2 | `Wait` | ticks `u32 ≥ 1` | `wait = ticks`, statut `Waiting`, `pc += 1`, **fin du tick pour l'instance** |
| 3 | `Set` | registre, valeur `i32` | `registre = valeur` ; `pc += 1` |
| 4 | `Add` | registre, valeur `i32` | `registre = registre.wrapping_add(valeur)` ; `pc += 1` |
| 5 | `Jump` | cible `u16` | `pc = cible` |
| 6 | `JumpIfNonZero` | registre, cible `u16` | si registre ≠ 0, `pc = cible`, sinon `pc += 1` |

**`step` (fonction pure de `(script, état, sink)`, sans effet de bord)** :
1. `Halted` ou `Faulted` : aucun changement.
2. `Waiting` : `wait -= 1` (saturant) ; si `wait == 0`, statut `Running` et l'exécution continue **dans ce même tick** ; sinon fin du tick pour l'instance. Un `Wait n` posé au tick *t* reprend donc l'exécution au tick *t + n*, et les valeurs persistantes de `wait` en fin de tick sont *n*, *n − 1*, …, 1 (jamais 0).
3. `Running` : script absent de la table → `Faulted(NoScript)`. Sinon, boucle : **chercher** l'instruction à `pc` ; absente (`pc` au-delà de la dernière) → `Halted` implicite ; **si `executed == SCRIPT_MAX_STEPS_PER_TICK` → `Faulted(Budget)`** ; sinon exécuter, `executed += 1`. `Wait` et `Halt` terminent le tour.
4. `Emit` sur sink plein : `Err(Full)` **sans avancer `pc`** (atomicité par instruction). Voir §4.F : cette branche est inatteignable dans une configuration conforme.

**Comptage du budget** : le budget borne le **nombre d'instructions exécutées** par instance et par tick, `Wait` et `Halt` compris. Un chemin de `SCRIPT_MAX_STEPS_PER_TICK` instructions terminé par `Wait` ou `Halt` tient ; un chemin de `SCRIPT_MAX_STEPS_PER_TICK` instructions suivi de la fin du programme tient (la recherche d'instruction précède le test du budget) ; une instruction de plus à exécuter déclenche `Faulted(Budget)`.

**Pas de pile cachée, pas de continuation, pas d'état global** : `step` n'a ni `static`, ni `thread_local`, ni `Cell`, ni retenue d'emprunt ; le système est un type unité (X14). Arithmétique entière enveloppante (aucun flottant, aucune panique de débordement).

**Constantes partagées Forge ↔ runtime** : `SCRIPT_REGISTERS` et `SCRIPT_MAX_STEPS_PER_TICK` sont définies dans `verbum-assets` (la Forge dépend de `verbum-assets` mais **pas** de `verbum-script`) ; `verbum-script` les réexporte. **Ce placement est celui d'un contrat de layout et de budget partagé entre la Forge et le runtime ; il ne confère aucune responsabilité d'interprétation à `verbum-assets`**, qui ne contient que des lecteurs et des constantes. Leur valeur (4 et 16) est **provisoire** (§6) : la modifier change le contrat entre artefacts et runtime et impose de recompiler les artefacts.

**Analyse de flot de contrôle et budget par la Forge [L, retenu]** — convention :
- chaque instruction est un nœud ; un nœud terminal virtuel `FIN` représente `pc == n` ;
- successeurs : `Halt` et `Wait` : **aucun** (la progression est interrompue après un `Wait`, frontière de yield) ; `Jump` : sa cible ; **`JumpIfNonZero` : ses deux successeurs possibles**, sa cible et `pc + 1` ; autres instructions : `pc + 1` ; `pc + 1 == n` désigne `FIN` ;
- **cycle** : tout cycle subsistant dans ce graphe (donc sans `Wait`) est un cycle sans yield et est refusé (`ForgeError::CycleWithoutYield`) ;
- **coût** : `coût(i) = 1 + max(coût(s))` sur les successeurs `s` de `i` ; `coût(Halt) = coût(Wait) = 1` (**le chemin maximal compte l'instruction `Wait` elle-même**) ; `coût(FIN) = 0` ; calcul par programmation dynamique itérative en ordre topologique inverse ;
- **entrées d'un tour** : l'instruction 0 (premier tour) et, pour chaque `Wait` en position *i*, l'instruction *i + 1* (reprise ; `FIN` si *i + 1 = n*) ;
- **budget** : si `coût(entrée) > SCRIPT_MAX_STEPS_PER_TICK` pour une entrée, la compilation échoue (`ForgeError::BudgetExceeded { entry }`) ;
- l'analyse porte sur **l'ensemble du graphe, code inaccessible compris** ; elle est volontairement conservatrice : un code inaccessible ne dispense pas du respect des limites, et le plus long des deux bras de `JumpIfNonZero` est compté.

**Propriété** : un script accepté par la Forge n'exécute jamais plus de `SCRIPT_MAX_STEPS_PER_TICK` instructions par tour ; **il n'atteint donc jamais `Faulted(Budget)`**. Ce statut reste une protection défensive contre un artefact non conforme (écrit à la main, corrompu ou produit par un autre outil). SR-7 vérifie la propriété sur des scripts compilés par la Forge à la limite, avec les deux issues de `JumpIfNonZero`.

### 4.E Test de suspension et reprise : définition de « bit-à-bit »

1. **Encodage canonique** : `ScriptState::to_bytes() -> [u8; ENCODED_LEN]` avec `ENCODED_LEN = 10 + 4·SCRIPT_REGISTERS` (26 pour 4 registres), little-endian explicite, champ par champ, sans remplissage : `script` 2, `pc` 2, `status` 1, `fault` 1, `wait` 4, registres 4 chacun ; et `from_bytes` selon la table du §4.C. Bijection entre octets valides et états. **L'encodage sert exclusivement à la reconstruction de l'état dans la preuve de l'étape : il n'est ni un format de sauvegarde, ni un engagement de compatibilité** (ADR-SV01 reste hors périmètre).
2. **Critère d'égalité** : égalité des **tableaux d'octets** de l'encodage canonique, jamais le seul `PartialEq` dérivé.
3. **Suspension** : instant où `status == Waiting` après au moins deux ticks d'attente écoulés pendant lesquels d'autres systèmes ont écrit leurs propres données.
4. **Reconstruction (SR-2)** : pour **chaque** tick *t* de 0 à 12 d'une exécution de référence, on capture les octets du composant ; on construit un monde, un ordonnanceur, une table de scripts (rechargée depuis les mêmes artefacts), des composants autres que `ScriptState` **entièrement neufs et d'état initial différent** ; la seule donnée transférée est la capture ; on exécute les ticks restants. On exige : (a) `to_bytes(from_bytes(capture)) == capture` ; (b) à chaque tick restant, l'encodage du composant est égal à celui de la référence ; (c) les émissions du suffixe sont égales (littéral écrit à la main) ; (d) l'état final est égal octet à octet.
5. **Aucune pile hors du World (SR-4)** : l'ordonnanceur est remplacé en cours d'exécution par un `Scheduler::new()` neuf, sur le même monde : résultat identique. Le système étant sans état, aucune information d'exécution ne vit hors du `World`.
6. **Continuité (SR-3)** : pour le scénario de référence (§6), l'instance est lancée au tick 1 :

| Fin de tick | `wait` persistant | `pc` | `r0` | Émission du tick |
|---|---|---|---|---|
| 1 | 3 | 3 | 3 | (1, 10) |
| 2 | 2 | 3 | 3 | — |
| 3 | 1 | 3 | 3 | — |
| 4 | 3 | 3 | 2 | (1, 10) |

Aux ticks 2 et 3, **tous les octets autres que `wait` sont constants** et `wait` vaut exactement 2 puis 1. La valeur 0 n'est jamais observée en fin de tick. La **reprise effective au tick 4** est démontrée par le changement de `r0` (3 → 2), le parcours de `pc` (3 → 4 → 1 → 2 → 3, soit un tour complet de boucle, observable par la seule trajectoire de `step` en IN-*), la réarmement de `wait` à 3 et l'émission attendue. Pendant ces ticks, `Beat` (autre composant, autre système) avance de 1 à chaque tick.

**Ce que le test ne prouve pas** : l'absence d'état externe caché *dans tout assemblage possible* (revue §7.5, comme pour l'étape 4) ; la persistance sur disque.

### 4.F Intégration ECS et Scheduler

**Crate d'exécution [A-S1, retenu]** : `crates/script`, package `verbum-script`, `no_std` sans `alloc`, **seule dépendance : `verbum-assets`**. Il contient `ScriptState` et son encodage, `Emissions<CAP>`, `ScriptTable<'a, CAP>`, la fonction `emissions_bound_holds`, `step`, et réexporte les constantes de `verbum-assets`. Aucune dépendance à `verbum-ecs` ni `verbum-schedule` : le crate ne nomme ni `World`, ni `Scheduler`, ni système. Il se classe comme `verbum-render` : types du noyau atteignables depuis `run_tick`.

**Système interpréteur [A-S2, retenu]** : adaptateur d'une vingtaine de lignes, écrit **dans l'assemblage** (tests et exemple de `crates/engine/shell`), qui itère les instances et appelle la fonction pure `step` de `verbum-script`. Raisons : `Binding`/`Locate` appartiennent à l'application (plan 01 §2.2 ; ADR-S01 pt 7) ; une version générique exigerait de contraindre `B::Parts<'w>: Locate<…>` sur plusieurs `ID` const-génériques et sur le type des `Store` ; aucun système n'est livré par un crate à ce jour. La fixité de l'interface est vérifiée par CT-1 et par la revue.

**Systèmes de la scène (assemblage)** : `Clear` (phase 0, `Produce<EMISSIONS>` : remise à l'état initial) ; phase 1 : `Collect` (`Read<EMISSIONS>`, `Produce<TRANSCRIPT>` : unique consommateur des émissions), `Interpreter`, `Drift` (`Write<BEAT>` : autre système qui écrit un second composant générique à chaque tick). Ordre de déclaration : `Clear`, `Collect`, `Interpreter`, `Drift` (le consommateur est déclaré avant son producteur : l'ordre exécuté ne vient que de la déduction). `SYSTEMS = 4`, `PHASES = 2`, cinq catégories (`STATE`, `SCRIPTS`, `EMISSIONS`, `TRANSCRIPT`, `BEAT`).

**Observation des sorties [A-S5, retenu]** : `TRANSCRIPT` est une Ressource transitoire de même type `Emissions<EMISSIONS_CAP>` que l'accumulateur. Propriétés exigées :
- **(a) Remplacement intégral à chaque tick.** `Collect` remplace **entièrement** `TRANSCRIPT` à chaque tick par une copie de `EMISSIONS` (`insert` d'un clone de la valeur entière, jamais d'ajout élément par élément ni d'omission), **même si aucune émission n'a eu lieu** : le transcript d'un tick sans émission est vide. Aucune condition (« si non vide ») ne gouverne ce remplacement.
- **(b) Aucune émission d'un tick précédent ne subsiste dans le World.** `Clear` (phase 0) remet `EMISSIONS` à l'état initial avant toute lecture déclarée ; `Collect` remplace `TRANSCRIPT` ; avant le premier tick, `TRANSCRIPT` et `EMISSIONS` sont initialisés vides à l'assemblage. Le World ne conserve donc, des sorties de scripts, que celles du tick courant.
- **(c) L'historique est extérieur au World.** La conservation de la suite complète des transcripts dans les tests et l'exemple est le fait d'un **observateur côté Plateforme** (un `Vec` de l'appelant, alimenté après chaque retour de `run_tick` par une lecture en lecture seule de `TRANSCRIPT`, forme 2 de l'étape 4) ; ce n'est pas un état du moteur. L'accumulateur `EMISSIONS` n'est **jamais** lu hors des systèmes.
- **Capacité du transcript** : `TRANSCRIPT` a **le même paramètre de capacité** que `EMISSIONS` (`EMISSIONS_CAP`) et est obtenu par copie de la valeur entière : il ne peut pas déborder par construction (types), et aucune politique de saturation ne lui est associée (EM-3).
- Limites héritées de l'étape 4 : l'unicité de `Collect` comme consommateur et producteur n'est pas garantie par les types (CT-3, revue) ; l'absence d'état « absent » repose sur l'initialisation à l'assemblage.

**Borne des émissions (compilation) [retenu]** : chaque instruction produit au plus une émission ; chaque instance exécute au plus `SCRIPT_MAX_STEPS_PER_TICK` instructions par tick **quel que soit l'artefact** (le budget est appliqué à l'exécution) ; le nombre d'instances ne dépasse pas la capacité dense `STATE_DENSE` du stockage de `ScriptState` (ADR-001 pt 5). Donc : `EMISSIONS_CAP ≥ STATE_DENSE × SCRIPT_MAX_STEPS_PER_TICK`. Cette relation est portée par `const fn emissions_bound_holds(instances: usize, cap: usize) -> bool`, qui calcule le produit par **`checked_mul`** (débordement ⇒ faux, jamais de dépassement silencieux), appelée dans un `const _: () = assert!(…)` de l'assemblage ; la fonction est testée directement (EM-2). Dans le scénario : `STATE_DENSE = 4`, `EMISSIONS_CAP = 64`. Conséquence : **la saturation de l'accumulateur est impossible pour une configuration conforme**. La branche `Err(Full)` de l'adaptateur n'est qu'une détection de violation du contrat ci-dessus (`unreachable!` avec message explicite nommant la borne) ; ce n'est ni une politique de saturation ni un substitut au dimensionnement. Il n'y a **pas de `LOG_CAP`** (C15). Un journal persistant, s'il était un jour voulu, exigerait une décision séparée (anneau, versement hors World) : hors périmètre.

**Ordre de composition** : `ComponentSet` à deux stockages (`ScriptState`, `Beat`), `Add`/`Remove` écrits par l'appelant (modèle : `component_set.rs`). `Beat` est un composant générique `Copy` (compteur `u32`).

**Lecture des octets empruntés** : G-LT a été établie positive sur `Map<'a>` et `Font<'a>` avec les bornes de `system.rs` (r1.3). Le même motif est repris sans nouveau doctest ; un échec de compilation de durée de vie à l'unité U4 est un **arrêt** (§5).

### 4.G Table de cohérence des règles

| Règle | Forge (U2) | Lecteur `Script::read` (U1) | `step` (U3) | Tests |
|---|---|---|---|---|
| Opération/opcode connu | `op` fermé : `UnknownOperation` | opcode inconnu : `Instruction` | total (pas de bras par défaut) | SCR-F3, SCR-R8 |
| Registre `< SCRIPT_REGISTERS` | `OutOfRange("register")` pour `set`, `add`, `jnz` | `Instruction` pour `Set`, `Add`, `JumpIfNonZero` seulement | accès direct ; hors bornes inatteignable (`unreachable!`) | SCR-F3, SCR-R8, IN-2 |
| Cible de saut `< n` | étiquette inconnue : `UnknownTarget` ; étiquette dupliquée ou vide : erreurs dédiées | `Instruction` pour `Jump`, `JumpIfNonZero` | `pc` valide ou fin de programme | SCR-F1, SCR-F3, SCR-R8, IN-3 |
| `Wait` : `ticks ≥ 1` | zéro : `ZeroValue("ticks")` ; négatif : `OutOfRange` | zéro : `Instruction` | `Waiting ⇒ wait ≥ 1` | SCR-F3, SCR-R8, IN-4, ST-2, ST-4 |
| Valeur `i32`, canal `u16` | `OutOfRange` hors intervalle | champs de largeur fixe | arithmétique enveloppante | SCR-F3, IN-2 |
| `n ≤ 65 535` | `TooManySteps` ; `EmptyScript` si `n = 0` | `n` sur `u16` ; `n = 0` conforme (C12 de l'étape 5 : valeur structurelle) | `pc` sur `u16` | SCR-F3, SCR-F7, SCR-R6, SCR-R10 |
| Cycle sans yield | `CycleWithoutYield` | **non vérifié** (analyse de flot) | budget défensif | SCR-F4 (a), IN-budget |
| Budget `≤ SCRIPT_MAX_STEPS_PER_TICK` | `BudgetExceeded { entry }` | **non vérifié** | `Faulted(Budget)` | SCR-F4 (b à h), IN-budget, IN-bound, IN-end, SR-7 |
| Slot du script | `DuplicateSlot`, `SlotOutOfRange` (existants) ; slot = indice de `ScriptTable` | `check_bindings`, `SlotFormat` (existants) | script absent : `NoScript` | SCR-C1, SCR-L1, TB-1, IN-miss, SR-5 |
| États canoniques | — | — | `step` ne produit que des états de la table | ST-2, ST-3, ST-4 |
| Capacité des émissions | — | — | `Err(Full)` atomique, inatteignable en configuration conforme | EM-1, EM-2, IN-full |
| Capacité du transcript | — | — | copie d'un `Emissions<EMISSIONS_CAP>` par `Collect` | EM-3, SR-1 |

## 5. Arbitrages retenus et portes

**Arbitrages rendus par l'utilisateur le 1er novembre 2026 (consignés, non rouverts).**

| # | Décision retenue | Précisions de l'utilisateur intégrées |
|---|---|---|
| **A-S1** | Nouveau crate `verbum-script`, `no_std` sans `alloc`, dépendant uniquement de `verbum-assets` ; membre du workspace, entrée dans `[workspace.dependencies]`, dev-dépendance de `verbum-shell` | — |
| **A-S2** | Fonction d'exécution pure (`step`) dans `verbum-script` ; système interpréteur adapté dans l'assemblage | — |
| **A-S3** | Validation structurelle des instructions dans `Script::read`, avec `ReadError::Instruction` ; ajouts additifs d'API publique de `verbum-assets` (§C11) | `SCRIPT_MAX_STEPS_PER_TICK` placé dans `verbum-assets` : contrat partagé Forge ↔ runtime, sans responsabilité d'interprétation (§4.D) |
| **A-S4** | Maintien du contournement local `Anchored<'a, T>` pour `Clear`, `Collect`, `Drift`, limité aux tests et à l'exemple, sans modification du Scheduler | — |
| **A-S5** | Suppression de `LOG_CAP` ; `TRANSCRIPT` transitoire contenant exclusivement les émissions du tick courant | Trois propriétés explicitées (§4.F) |

Choix locaux validés : `Wait 0` rejeté par la Forge et le lecteur ; `Waiting ⇒ wait ≥ 1` ; `SCRIPT_REGISTERS = 4` et `SCRIPT_MAX_STEPS_PER_TICK = 16` comme **valeurs provisoires** ; analyse de flot sur l'ensemble du graphe ; `ScriptTable` indexée par `SlotIndex` avec emplacements non scripts conservés ; scénario de référence du §6 ; encodage canonique de `ScriptState` exclusivement pour la reconstruction de l'état.

Options non retenues (historique) : A-S1 : tout dans `verbum-assets` ; tout dans l'assemblage de test. A-S2 : système générique livré par `verbum-script` (bornes de traits, premier système livré, E-12). A-S3 : aucune validation au chargement (faute runtime). A-S4 : réouverture du Scheduler. A-S5 : journal persistant à capacité fixe (croissance non bornée, C15) ; lecture directe de l'accumulateur.

**Portes d'arrêt** :
- **G-CM** (plan 05 §1.3) : toute modification d'un niveau de contrôle, de leur ordre, de `VMAN`, de l'atomicité, du modèle d'identité, ou introduction d'une allocation dans `load` est un arrêt. Aucun test de l'étape 5 ne change de sémantique ; adaptations de signature seulement (R-ADAPT).
- **G-LT** : non rouverte (§4.F). Échec de durée de vie en U4 = arrêt sans contournement.
- **G-ORD-S** : aucun test ni exemple n'affirme l'ordre des émissions entre instances ni l'ordre dense d'un stockage ; comparaisons en multiensemble, multiplicité conservée.
- **G-ST** (aucun état hors World) : arrêt si un besoin de `static`, `thread_local`, `Cell`/`RefCell`/atomique, `async`, `unsafe` ou d'un état dans le système apparaît.

Aucune contradiction n'est apparue, donc aucun arrêt de conception n'est déclenché par ce plan.

## 6. Choix locaux [L]

**Valeurs provisoires.** `SCRIPT_REGISTERS = 4` et `SCRIPT_MAX_STEPS_PER_TICK = 16` sont des **choix provisoires de cette première implémentation**. Ils ne sont **pas** des invariants généraux du moteur : aucune ADR ne les fixe. Les tests qui en dépendent sont paramétrés par les constantes (programmes de `SCRIPT_MAX_STEPS_PER_TICK` et `SCRIPT_MAX_STEPS_PER_TICK + 1` instructions construits par code) ; deux tests seulement épinglent leurs valeurs (SCR-K1 pour les constantes, ST-1 pour la longueur de l'encodage), de sorte que modifier ces valeurs soit un acte délibéré, listé, qui impose de réviser les artefacts littéraux.

- Constantes de `verbum-assets` : `SCRIPT_FORMAT_ID = *b"VSCR"`, `SCRIPT_FORMAT_VERSION = 1`, `SCRIPT_FIXED_PAYLOAD_LEN = 2`, `SCRIPT_INSTRUCTION_LEN = 8`, `SCRIPT_REGISTERS = 4`, `SCRIPT_MAX_STEPS_PER_TICK = 16`.
- Charge utile de `script.bin`, little-endian, lue champ à champ sans cast :

| Offset | Taille | Champ |
|---:|---:|---|
| 0 | 4 | identité `VSCR` |
| 4 | 2 | version `u16` |
| 6 | 4 | longueur de charge utile `u32` : `2 + 8·n` |
| 10 | 2 | nombre d'instructions `n`, `u16` |
| 12 | `8·n` | enregistrements |

Enregistrement : opcode `u8`, registre `u8`, `arg16` `u16`, `arg32` `u32` (octets d'opérande inutilisés ignorés par le lecteur ; le registre n'est validé que pour `Set`, `Add`, `JumpIfNonZero`). Association : `Emit` : `arg16` = canal, `arg32` = valeur ; `Wait` : `arg32` = ticks (≥ 1) ; `Set`/`Add` : registre, `arg32` = valeur ; `Jump` : `arg16` = cible ; `JumpIfNonZero` : registre, `arg16` = cible.
- Ordre des contrôles du lecteur (comme `font.bin`) : préfixe complet, identité, version, longueur déclarée inférieure à la partie fixe (`PayloadLength`), octets restants inférieurs ou supérieurs (`Truncated`, `TrailingBytes`), longueur déduite en `u64` (`PayloadLength`), puis validation de chaque enregistrement (`Instruction`).
- API publique du lecteur : `Script<'a>` (`Clone, Copy, Debug, PartialEq, Eq`), `Script::read(&'a [u8])`, `len() -> u16`, `is_empty()`, `instruction(index: u16) -> Option<Instruction>`, `Instruction` énumération `Copy` à sept variantes.
- Source Forge : `[[step]]` avec `op` ∈ {`halt`, `emit`, `wait`, `set`, `add`, `jump`, `jnz`} ; champs : `emit` : `channel` (`0..=65535`), `value` (`i32`) ; `wait` : `ticks` (`1..=u32::MAX`) ; `set`/`add` : `register` (`0..SCRIPT_REGISTERS`), `value` (`i32`) ; `jump` : `target` (chaîne, étiquette) ; `jnz` : `register`, `target` ; tout pas : `label` optionnel (chaîne non vide, unique) ; au moins un pas, au plus 65 535 ; champ inconnu et opération inconnue refusés. Erreurs nouvelles de la Forge : `UnknownOperation`, `EmptyScript`, `TooManySteps`, `DuplicateLabel`, `EmptyLabel`, `UnknownTarget`, `CycleWithoutYield`, `BudgetExceeded { entry }` ; `ZeroValue("ticks")` réutilisé.
- Genre de lot : `AssetKind::Script`. Le slot d'un script est l'indice de `ScriptTable`.
- Constantes de scène (assemblage) : `STATE_DENSE = 4`, `EMISSIONS_CAP = 64`, `SCRIPT_SLOTS` (= longueur de la table de formats de slot), avec `const _: () = assert!(emissions_bound_holds(STATE_DENSE, EMISSIONS_CAP))` ; les constantes utilisées **seulement** dans un `const _` sont évitées (E-14 : avertissement `dead_code` sous 1.85).
- **Scénario de référence** (littéral, à recalculer au document d'exécution) : `set r0 3 ; loop: emit 1,10 ; wait 3 ; add r0 −1 ; jnz r0 loop ; emit 2,99 ; halt` (sept pas, indices 0 à 6 ; `loop` = pas 1). Coûts Forge : entrée 0 : 3 (`set`, `emit`, `wait`) ; entrée 3 (après le `wait`) : `add`, `jnz`, puis au plus `emit`, `wait` ou `emit`, `halt` : 4 ≤ 16. Émissions aux ticks 1, 4, 7 (canal 1, valeur 10) et 10 (canal 2, valeur 99) ; l'instance est `Halted` au tick 10 avec `wait = 0` ; `r0` vaut 3, 2, 1, puis 0 ; `Beat` vaut 12 après 12 ticks. Le scénario n'exécute jamais plus de 4 instructions par tour : il ne dépend pas de la valeur 16.

## 7. Invariants, tests, API minimale

Méthode : invariant → test → API minimale → implémentation ; tests écrits d'abord contre des squelettes ; octets littéraux **écrits à la main, indépendants des constantes de layout** ; vocabulaire générique (canal, valeur, registre), aucun nom de jeu.

### 7.1 Lecteur et chargement (`verbum-assets`, U1)

| Test | Énoncé | Invariants |
|---|---|---|
| SCR-R1 | Lecture d'un artefact littéral : champs et chaque instruction décodée égaux au littéral | X1, X11 |
| SCR-R2 | Chaque préfixe tronqué rejeté (`Truncated`) | X11 |
| SCR-R3 | Identité altérée octet par octet ; texte d'un `script.toml` rejeté (`FormatIdentity`) | X10, T2 |
| SCR-R4 | Versions inconnues rejetées | X11 |
| SCR-R5 | Longueurs incohérentes rejetées dans l'ordre documenté (`PayloadLength`, `Truncated`, `TrailingBytes`) | X11 |
| SCR-R6 | Script de zéro instruction (charge utile de 2 octets) conforme ; charge utile de longueur nulle invalide | X11 |
| SCR-R7 | `n = 0xFFFF` avec peu d'octets rejeté sans panique ni débordement | X12 |
| SCR-R8 | Opcode inconnu, registre ≥ `SCRIPT_REGISTERS` (pour `Set`, `Add`, `JumpIfNonZero`), cible ≥ `n` (`Jump`, `JumpIfNonZero`), **`Wait` de zéro tick** : chacun rejeté (`Instruction`), sur des enregistrements et des positions distincts ; un registre invalide sur `Emit` ou `Jump` (octet ignoré) n'est **pas** rejeté | X11, A-S3, C12 |
| SCR-R9 | `instruction(i)` borné : dernier indice accessible, premier inaccessible, `None` hors bornes | X12 |
| SCR-R10 | Script maximal (65 535 instructions) construit en binaire accepté ; un octet de moins : `Truncated` | X12 |
| SCR-K1 | **Épinglage des valeurs provisoires** : `SCRIPT_REGISTERS == 4`, `SCRIPT_MAX_STEPS_PER_TICK == 16`, `SCRIPT_INSTRUCTION_LEN == 8` ; message d'échec renvoyant au §6 | C16 |
| SCR-L1 | Un script lié à un slot d'un autre format, et réciproquement, rejeté `SlotFormat` avant tout dépôt (les cinq formats) | X18 |
| SCR-L2 | Lot `{script, script}` : deux dépôts, valeurs égales aux littéraux, `artifact_index ≠ slot_index` | X18, C13 |
| SCR-L3 | Instruction invalide dans un artefact d'un lot valide : rejet de niveau 3, compteur de dépôts nul | X18, T10 |
| SCR-L4 | Non-régression : tous les tests de l'étape 5 passent, assertions inchangées | G-CM |

### 7.2 Forge (`verbum-forge`, U2)

Les programmes de test de budget sont **construits par code à partir de `SCRIPT_MAX_STEPS_PER_TICK`** (texte TOML généré), non écrits avec le littéral 16.

| Test | Énoncé | Invariants |
|---|---|---|
| SCR-F1 | Une source valide produit exactement les octets littéraux (étiquettes résolues en positions) | X1, X10 |
| SCR-F2 | La sortie est relue par `Script::read` | X1 |
| SCR-F3 | Refus : champ manquant, champ inconnu par opération, opération inconnue, registre hors bornes, valeur hors `i32`, canal hors `u16`, **`ticks` zéro** (`ZeroValue`) ou **négatif** (`OutOfRange`), étiquette dupliquée ou vide, cible inconnue, script vide, TOML invalide | X7, T11 |
| SCR-F4 | **Terminaison et budget** (convention du §4.D). (a) **cycle sans yield refusé** : saut arrière direct, `jnz` seul, retour sur soi ; **cycle contenant un `wait` accepté** ; (b) **chemin linéaire de exactement `SCRIPT_MAX_STEPS_PER_TICK` instructions, terminé par `wait` (compté) ou `halt`, accepté** ; (c) **chemin de `SCRIPT_MAX_STEPS_PER_TICK + 1` instructions refusé** (`BudgetExceeded`, `entry` exact) ; (d) chemin de `SCRIPT_MAX_STEPS_PER_TICK` instructions suivi de la fin du programme accepté, de `+ 1` refusé ; (e) **`JumpIfNonZero` : bras *cible* seul trop long** (le bras `pc + 1` court) refusé ; (f) **`JumpIfNonZero` : bras `pc + 1` seul trop long** (le bras cible court) refusé ; (g) **branchement dont le chemin le plus long dépasse la limite**, alors que le chemin le plus court la respecte : refusé (analyse conservatrice) ; (h) les deux bras de `JumpIfNonZero` à la limite exacte : accepté ; (i) plus de `SCRIPT_MAX_STEPS_PER_TICK` instructions avec un `wait` placé de sorte que chaque segment tienne : accepté ; (j) entrée de reprise placée à la fin du programme (coût 0) acceptée ; (k) **code inaccessible** dépassant la limite, ou formant un cycle sans yield : refusé (analyse sur l'ensemble du graphe) | X7, T10, T11, C14 |
| SCR-F5 | Déterminisme ; invariance de la sortie du lot par permutation de l'entrée (seuls les `artifact_index` changent) | X1 |
| SCR-F6 | Aucune chaîne dans la sortie : ni étiquette, ni nom d'opération | X10 |
| SCR-F7 | Limite des 65 535 pas par un assistant testé directement ; bout en bout en natif seulement (leçon E-13 : TOML interprété sous Miri) | X12 |
| SCR-C1 | Chaîne `compile_assets` → `load` → `Script<'a>` sur un lot **mixte** (sprite, tileset, map, font, script) dont deux scripts sont en slots non contigus (ex. 1 et 4) avec `artifact_index ≠ slot_index` : cinq dépôts ; les scripts atterrissent dans `ScriptTable` aux indices de **slot** ; les autres indices restent `None` ; aucun changement de mécanisme | T1, G-CM, C13 |

### 7.3 Exécution pure (`verbum-script`, U3)

| Test | Énoncé | Invariants |
|---|---|---|
| ST-1 | Encodage canonique : `ENCODED_LEN == 10 + 4·SCRIPT_REGISTERS == 26` (épinglé), littéral écrit à la main, champ par champ | T4, C10 |
| ST-2 | `from_bytes` conforme à la **table du §4.C** : les cinq combinaisons valides acceptées (dont `Waiting` avec `wait = 1` et avec `wait = u32::MAX`) ; chaque combinaison impossible rejetée : `Waiting` avec `wait = 0`, `wait ≠ 0` pour `Running`, `Halted`, `Faulted`, `fault ≠ 0` hors `Faulted`, `Faulted` avec `fault = 0` ou 3, octet de statut inconnu | T14 |
| ST-3 | Bijection : `from_bytes ∘ to_bytes = id` sur un échantillon couvrant tous les statuts ; `to_bytes ∘ from_bytes = id` sur les octets valides ; `ScriptState` est `Copy` et `'static` (contrôle de type) | T4 |
| ST-4 | **Tout état atteint par `step`** sur les trajectoires de référence (scénario de référence, budget, script absent, `Halt`, fin de programme) est canonique : `to_bytes` puis `from_bytes` réussit et redonne l'état ; en particulier `Waiting` n'est jamais observé avec `wait = 0` ; `ScriptState::start` rejette un slot `> u16::MAX` et accepte `u16::MAX` | T14, C12 |
| IN-1 | `Emit` : pousse (canal, valeur) dans l'ordre, avance `pc` | T6 |
| IN-2 | `Set`, `Add` : effet exact ; `Add` enveloppant aux bornes de `i32`, sans panique | T9 |
| IN-3 | `Jump`, `JumpIfNonZero` : branche prise et non prise | T6 |
| IN-4 | `Wait n` pour *n* = 1, 2, 3 : reprise au tick *t + n* exactement ; valeurs persistantes de `wait` en fin de tick *n*, *n − 1*, …, 1 | T6 |
| IN-5 | `Halt` terminal ; états terminaux inchangés bit-à-bit par `step` | T10 |
| IN-budget | Binaire écrit par code de `SCRIPT_MAX_STEPS_PER_TICK + 1` `Add` consécutifs sans yield (`Add r0, +1`) : `Faulted(Budget)` après l'exécution d'exactement `SCRIPT_MAX_STEPS_PER_TICK` instructions (`r0` = `pc` = `SCRIPT_MAX_STEPS_PER_TICK`) ; état déterministe | T10 |
| IN-bound | Binaire de `SCRIPT_MAX_STEPS_PER_TICK` instructions terminé par `Wait` : aucune faute ; reprise normale | T10 |
| IN-end | Binaire de `SCRIPT_MAX_STEPS_PER_TICK` instructions suivies de la fin du programme : `Halted` implicite, aucune faute | T10 |
| IN-miss | Script absent de la table (slot vacant, slot occupé par un autre format donc vacant dans la table, slot hors `CAP`) : `Faulted(NoScript)` | T10, T14 |
| IN-full | `step` sur un sink de capacité 1 : `Err(Full)` à la seconde émission, `pc` non avancé, état et émissions déjà acceptées conservés (propriété de la fonction pure ; branche inatteignable en configuration conforme, §4.F) | X12 |
| IN-det | Même entrée, même sortie, deux appels | T9 |
| EM-1 | `Emissions` : acceptation jusqu'à `CAP`, `Err(Full)` ensuite sans modification, `clear` | X12 |
| EM-2 | `emissions_bound_holds` : vrai à la borne exacte, faux à une unité sous la borne, **faux sur débordement du produit** (`checked_mul`) | T10, X12 |
| EM-3 | Le transcript a le type de capacité de l'accumulateur : une fonction `fn same_capacity<const N: usize>(&Emissions<N>, &Emissions<N>)` accepte (accumulateur, transcript) ; la copie par `Clone` d'un accumulateur plein donne un transcript égal ; aucune insertion élément par élément n'est nécessaire | T10 |
| TB-1 | `ScriptTable` indexée par slot : dépôt, lecture ; slot non script `None` ; dépôt hors `CAP` et sur un slot occupé refusés ; `get(index)` hors `CAP` : `None` ; indice lu par `get` = slot déposé, `artifact_index` jamais utilisé | T14, C13 |

### 7.4 Intégration Scheduler et test d'invariant (`crates/engine/shell/tests/script_suspend.rs`, U4)

Montage : `World<ENTITIES, Components, Rs<'a>>`, `Components` à deux stockages (`ScriptState`, `Beat`) ; systèmes `Clear`, `Collect`, `Interpreter`, `Drift` (§4.F) ; table chargée par le véritable `load` depuis des artefacts compilés par la Forge ; `Anchored` pour `Clear`, `Collect`, `Drift` (A-S4) ; `EMISSIONS` et `TRANSCRIPT` initialisés vides à l'assemblage.

| Test | Énoncé | Invariants |
|---|---|---|
| CT-1 | **Contrat propre à `Interpreter`**, obtenu par `contract_of::<B<'_>, Interpreter>()` (et non par fusion de la liste entière), égal au littéral : `reads = {SCRIPTS}`, `writes = {STATE}`, `produces = {EMISSIONS}`, aucun effet. Les contrats des autres systèmes sont contrôlés séparément et n'entrent pas dans cette égalité. Un `WideInterpreter` de test (même corps, un accès de plus) fait échouer le contrôle | T3 |
| CT-2 | Seul `Interpreter` a `STATE` dans `writes ∪ produces` ; un second écrivain dans la même phase est refusé par `MultipleWriters` à la construction | T8 |
| CT-3 | Composition : `Collect` est l'unique lecteur de `EMISSIONS` ; seuls `Clear` et `Interpreter` la produisent ; `Collect` est l'unique producteur de `TRANSCRIPT` ; `Clear` en phase 0, `Collect` en phase 1 | X15, T15 |
| SY-0 | Le montage compile et s'exécute sous le véritable `Scheduler::run_tick` avec une table de vues empruntant des octets construits à l'exécution ; la sortie dépend des octets | G-LT |
| SR-1 | Scénario de référence sur 12 ticks. **Assertions de transcript** : avant le premier tick, `TRANSCRIPT` est vide ; après chaque tick, `TRANSCRIPT` est égal **exactement** aux émissions de ce tick (littéral : (1,10) aux ticks 1, 4, 7 ; (2,99) au tick 10) ; **aux ticks 2, 3, 5, 6, 8, 9, 11, 12, le transcript est vide, bien qu'une émission ait eu lieu à un tick antérieur** ; l'observateur extérieur reconstitue l'historique complet. `Beat` avance de 1 à chaque tick, y compris pendant les attentes | T7, T15, X14 |
| SR-2 | Reconstruction de l'état à partir des octets seuls, pour chaque tick *t* de 0 à 12, selon §4.E point 4 | T4, T7 |
| SR-3 | Table du §4.E point 6 : `wait` persistant 3, 2, 1 en fin de ticks 1, 2, 3 ; aux ticks 2 et 3 tous les autres octets constants ; aucune valeur 0 en fin de tick ; reprise au tick 4 démontrée par `r0` (3 → 2), `pc` revenu à 3, `wait` réarmé à 3 et émission (1, 10) | T6, T7 |
| SR-4 | Remplacement de l'ordonnanceur par un `Scheduler::new()` neuf au tick 5 sur le même monde : suite d'états et d'émissions identique | T5 |
| SR-5 | Deux instances (scripts distincts en slots non contigus, `artifact_index ≠ slot_index`, la seconde lancée après le tick 2 par `apply`) : états indépendants, transcript égal en multiensemble au littéral ; inversion de l'ordre de création des entités : même multiensemble ; chaque instance exécute le script de **son slot** | C6, G-ORD-S, C13 |
| SR-6 | Lancement par `CommandBuffer::add_component` + `World::apply` : aucune écriture directe du composant ; l'instance démarre à `pc = 0`, `Running` | C9, ADR-006 |
| SR-7 | **Script accepté par la Forge, jamais `Faulted(Budget)`** : scripts compilés par la Forge à la limite (b, d, h de SCR-F4) et à bras de `JumpIfNonZero` aux coûts maximaux, exécutés sur plusieurs ticks, **pour chacune des deux issues du `jnz`** (registre nul, registre non nul) : aucun état `Faulted(Budget)` n'est jamais observé ; compte d'instructions exécutées par tour ≤ `SCRIPT_MAX_STEPS_PER_TICK` | T10, C14 |

### 7.5 Démonstrateur (`crates/engine/shell/tests/script_loaded.rs`, `examples/script_headless.rs`, U5)

| Test | Énoncé |
|---|---|
| DM-1 | Chaîne complète sous `run_loop` : sources TOML → `compile_assets` → `load` → `World` → `Scheduler` → interpréteur ; N ticks ; transcript par tick égal au littéral ; les ticks sans émission sont observés vides |
| DM-2 | Artefact de script corrompu (troncature, instruction invalide, `Wait` de zéro tick, format de slot erroné) : `load` échoue, aucun tick, aucune présentation |
| DM-3 | Séquence tick → lecture de `TRANSCRIPT` → présentation, sur journal d'événements de la Plateforme ; aucune lecture de `EMISSIONS` hors systèmes ; l'historique est tenu par l'observateur de la Plateforme, non par le World |

L'exemple imprime la timeline des émissions sans backend graphique et se termine par le code 0.

**Revue de l'absence d'état caché (T5)** : recherche textuelle dans `crates/script/**` et dans les systèmes d'assemblage de `static`, `thread_local`, `Cell`, `RefCell`, atomiques, `async`, `await`, `thread`, `Box`, `Vec`, `unsafe` ; rapportée au rapport d'unité ; aucune de ces occurrences n'est admise dans `crates/script/**` ni dans le corps des systèmes (les `Vec` du journal de l'observateur de la Plateforme, côté assemblage, sont hors systèmes). Limite : revue, non preuve statique.

### 7.6 Sensibilité par mutation (copies isolées, une à la fois)
Familles retenues (mutations exactes et détecteurs au document d'exécution) :
- **Lecteur** : sans contrôle de registre, de cible, d'opcode ou de `Wait` nul.
- **Encodage et état** : endianness inversée d'un champ d'instruction ou de l'encodage d'état ; `from_bytes` acceptant `Waiting` avec `wait = 0` ; `from_bytes` acceptant `wait ≠ 0` hors `Waiting`.
- **`step`** : décrément de `wait` omis ou doublé ; reprise au tick *t + n ± 1* ; `Wait` sans fin de tick ; `pc` non avancé après `Emit` ; budget décalé d'une unité (17ᵉ instruction exécutée, ou 16ᵉ refusée) ; budget ignoré ; `Add` non enveloppant.
- **Forge** : cycle sans yield accepté ; coût de la Forge omettant l'un des deux successeurs de `jnz` (chacun des deux) ; coût d'une reprise omis ; `Wait` non compté dans le chemin ; analyse limitée au code accessible ; étiquette résolue en index décalé.
- **Slots et capacités** : table indexée par `artifact_index` au lieu du slot ; compactage des slots non scripts ; produit de `emissions_bound_holds` sans `checked_mul`.
- **Transcript (T15)** : `Collect` qui ne remplace pas le transcript quand l'accumulateur est vide (transcript périmé aux ticks 2, 3) ; `Collect` qui ajoute au lieu de remplacer ; `Clear` supprimé (émissions cumulées entre ticks) ; transcript non initialisé vide.
- **Assemblage** : état partiel conservé hors du composant (par exemple `pc` dans un `static` atomique) ; reconstruction qui lit `Beat` ; second écrivain de `STATE` ; contrat élargi d'une catégorie ; ordre d'émission affirmé entre instances (doit rester sans effet sous permutation).

**Exception admise, une seule** : la mutation « `EMISSIONS_CAP` sous la borne » est détectée par l'échec de compilation de la `const` assert, comme M-R4 à l'étape 4 ; elle est complétée par EM-2 (détection par échec d'assertion). Toute autre détection par échec de compilation n'est pas admise.

## 8. Unités (une session d'implémentation chacune ; validation de l'utilisateur entre unités)

| Unité | Contenu | Porte d'arrêt |
|---|---|---|
| **U1** | `verbum-assets` : `script.rs` (layout, lecteur, `Script`, `Instruction`, constantes), `Format::Script`, `Asset::Script`, `ReadError::Instruction`, identité dans `format.rs` ; tests SCR-R*, SCR-K1, SCR-L* ; non-régression complète | Une assertion de l'étape 5 à modifier → arrêt (G-CM) ; `match` exhaustif cassé par une variante → exposé avant correction |
| **U2** | `verbum-forge` : `script.rs`, `AssetKind::Script`, aiguillage dans `batch.rs`, erreurs, validations (étiquettes, opérandes, cycles, budget) ; tests SCR-F*, SCR-C1 | Besoin d'une dépendance autre que `toml` → arrêt |
| **U3** | Crate `verbum-script` : `ScriptState` et encodage, `Emissions`, `ScriptTable`, `emissions_bound_holds`, `step` ; racine : `members` et `[workspace.dependencies]` ; tests ST-*, IN-*, EM-*, TB-1 | `step` exigeant `alloc`, `unsafe` ou un état externe → arrêt (G-ST) |
| **U4** | `crates/engine/shell/tests/script_suspend.rs` : `ComponentSet`, `Binding`, systèmes d'assemblage, CT-*, SY-0, SR-1 à SR-7 ; `crates/engine/shell/Cargo.toml` (dev-dépendance) | Échec de durée de vie du montage → arrêt sans contournement (G-LT) ; exigence d'ordre inter-instances → arrêt (G-ORD-S) |
| **U5** | `script_loaded.rs` (DM-1 à DM-3), `examples/script_headless.rs`, contrôles de clôture, Miri, MSRV 1.85, rapport « non démontré » | Résultat non conforme à la liste du §10 → exposé |

Découpage en sessions recommandé : A = U1 et U2 ; B = U3 et U4 ; C = U5.

## 9. Contrôles communs
Quatre contrôles à chaque unité : `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, par l'utilisateur ; les résultats de l'espace de travail de la session sont toujours séparés de ceux du dépôt réel. Par unité : `cargo tree -p verbum-assets` (aucune dépendance, **`[dev-dependencies]` compris**), `cargo tree -p verbum-script -e normal` (seule `verbum-assets`), `cargo tree -i toml --workspace` (seul `verbum-forge`), `cargo tree -i verbum-forge --workspace -e normal` (aucun dépendant), `cargo tree -i verbum-script --workspace` (seul `verbum-shell`, en dev-dépendance). Miri : `verbum-assets`, `verbum-script`, `verbum-forge` (avec les exclusions déjà connues de l'étape 5 si nécessaire) et `verbum-shell`. MSRV 1.85 en **deux commandes** (`build`, puis `test`). Compatibilité dès l'écriture : pas de chaînes `if let … &&` ; aucune assertion sur l'horloge ; tests lourds réduits sous `cfg!(miri)` avec couverture native conservée. Revue de diff : aucun fichier interdit.

## 10. Ce qui reste non démontré à la clôture (à consigner)
- **Lecture de données métier par les scripts** : non exprimable à cette étape. Les scripts ne lisent ni composant, ni Ressource de jeu, ni autre entité ; le comportement « conditionnel » se limite aux registres de l'instance (§1.2). Ce n'est pas un système de comportement complet.
- **Lancement depuis un système** (`Commands<ADD>`) et arrêt/retrait d'une instance par commande : seul le lancement par `apply` à l'assemblage est exercé (C9).
- **Appels, sous-programmes, pile** : non introduits (X16). Une pile à capacité fixe portée par le composant resterait conforme à Sc01 pt 4.
- **Conversion des durées humaines en ticks** (C5) ; politique temporelle (future ADR-S04, non rédigée).
- **Persistance de l'état** (ADR-SV01) : l'encodage canonique n'est qu'un outil de reconstruction ; compatibilité de version d'un composant non traitée.
- **Fixité de l'interface par les types** : CT-1 et la revue seulement ; un système supplémentaire hors liste, ou un handle externe, n'est pas empêché par les types.
- **Absence d'état caché dans tout assemblage** : revue de grep, SR-2 et SR-4 sur l'assemblage de référence.
- **Ordre des émissions entre instances** (C6) ; clés de tri, priorité.
- **Journal persistant des émissions** : non fourni (C15, A-S5) ; l'historique est tenu hors du World par l'observateur de la Plateforme.
- **Analyse de flot de contrôle** : conservatrice (code inaccessible compris, plus long des deux bras de `jnz`) ; elle peut refuser un script dont le chemin réel serait plus court.
- **Valeurs 4 et 16** : provisoires ; leur caractère adéquat pour un contenu réel n'est pas démontré ; **taille de slot** limitée à `u16` pour `ScriptState.script`.
- **Performance** (coût du double décodage et du dispatch) et assets de taille réaliste.
- **Visualisation du jeu** : étude distincte après la clôture de l'étape.

## 11. Risques de fermeture prématurée
1. **Jeu d'instructions présenté comme le langage de script du moteur** : il est un minimum de preuve ; format, opcodes, étiquettes sont des choix locaux.
2. **`ScriptState` figé sans pile** : l'absence de pile est une économie de l'étape, non une règle (C2).
3. **Adaptateur d'assemblage pris pour l'interface fixe** : la fixité est un contrat vérifié (CT-1), pas un type.
4. **Validation de terminaison et de budget confondue avec une garantie runtime** : le budget runtime (`Faulted(Budget)`) reste la défense, et la borne des émissions en dépend.
5. **Valeurs 4 et 16 prises pour des invariants** : provisoires (C16) ; épinglées par SCR-K1 et ST-1, tests de budget paramétrés.
6. **Vocabulaire de jeu** : « dialogue » n'apparaît que comme intitulé de l'artefact de la Roadmap ; types, champs, fixtures restent génériques (X17).
7. **Extension additive de `ReadError`/`Format`/`Asset`** prise pour un changement de mécanisme : SCR-L4 et la porte G-CM l'encadrent.
8. **Contournement `Anchored` généralisé** : limité aux tests et à l'exemple (A-S4).
9. **Preuve de reprise rejouée dans un même processus** : un état caché dans un `static` pourrait y survivre ; SR-2 reconstruit des mondes neufs et la revue grep couvre ce cas.
10. **Rejet de `Wait 0`** présenté comme une règle métier : c'est une contrainte de forme canonique de l'état (C12), révisable avec le layout.
11. **Historique des émissions pris pour un état du moteur** : il est tenu par l'observateur de la Plateforme ; le World ne conserve que les sorties du tick courant (T15).
12. **`unreachable!` de la branche `Err(Full)` pris pour une politique de saturation** : c'est la détection d'une violation d'un contrat prouvé à la compilation (§4.F).

## 12. Hors périmètre
Backend graphique, caméra, rendu ; compilateur de DSL de dialogue ; appels et sous-programmes ; accès aux données de jeu ; conversion secondes → ticks ; journal persistant ; parallélisme ; streaming, rechargement à chaud ; sauvegarde ; réseau ; modification d'un ADR, de la Roadmap, de `verbum-ecs`, `verbum-schedule`, `verbum-render`, de la façade `verbum` ou des documents des étapes précédentes ; tout `Plugin`, `Registry`, `Service`, `Factory`, `Builder`, `TypeId`, `PhaseKind`, `SystemSet`, `dyn Trait`, `unsafe`.

## 13. Fichiers (prévisions, à confirmer au document d'exécution)
- **`crates/engine/assets/src/`** : `script.rs` (créé), `format.rs`, `lib.rs`, `sprite.rs` (variante de `ReadError`) ; `load.rs` et `manifest.rs` inchangés ; `Cargo.toml` inchangé.
- **`crates/engine/forge/src/`** : `script.rs` (créé), `batch.rs`, `lib.rs`, `sprite.rs` (variantes de `ForgeError`) ; `tests/pipeline.rs` étendu.
- **`crates/script/`** (créé) : `Cargo.toml`, `src/{lib,state,emissions,table,step}.rs`.
- **`crates/engine/shell/`** : `Cargo.toml` (dev-dépendance `verbum-script`), `tests/{script_suspend,script_loaded}.rs`, `examples/script_headless.rs`.
- **Racine** : `Cargo.toml` (`members`, `[workspace.dependencies]`), `Cargo.lock` (par Cargo).
- **Jamais modifiés** : `crates/{ecs,schedule,render,verbum}/**`, `crates/engine/shell/src/**` (y compris la documentation `//!`, aucun doctest nouveau), `crates/engine/assets/src/{load,manifest,header,map,font,tileset}.rs` sauf mention ci-dessus, ADR, `ROADMAP.md`, documents des étapes 0 à 5.
- **Documentation** : ce plan, le document d'exécution, le handoff ; `tree.md` régénéré par l'utilisateur.

## 14. Critères d'acceptation (clôture)
1. Chaque unité validée une à une, quatre contrôles verts sur le dépôt réel à chaque unité.
2. SCR-R*, SCR-K1, SCR-L*, SCR-F*, SCR-C1, ST-*, IN-*, EM-*, TB-1, CT-*, SY-0, SR-1 à SR-7, DM-1 à DM-3 verts ; chaque mutation du §7.6 détectée par un test nommé, avec la seule exception admise de la `const` assert.
3. Tests de l'étape 5 inchangés (adaptations de signature consignées une à une).
4. `verbum-assets` et `verbum-script` : `no_std`, sans `alloc` ; `verbum-assets` sans aucune dépendance ; `verbum-script` dépend de `verbum-assets` seule ; aucun parseur de format humain hors de la Forge.
5. `cargo run -p verbum-shell --example script_headless` : code 0, timeline attendue, aucun backend graphique.
6. Revue §7.5 rapportée ; aucun `static`, `thread_local`, état de système, `async`, `unsafe`.
7. Miri et MSRV 1.85 exécutés par l'utilisateur et verts, ou signalés non exécutés.
8. Aucun invariant ni ADR modifié ; seules modifications hors périmètre des crates `assets`, `forge`, `script`, `shell` : racine du workspace.
9. §10 reproduit tel quel au rapport de clôture, avec la mention que 4 et 16 sont provisoires.

## 15. Décisions restantes pour valider ce plan
Tous les arbitrages et choix locaux de r1.1 sont rendus (§5). Il reste :
1. **Validation de r1.2** (cohérence du §4.G, des tests ajoutés SCR-F4 (e à k), SCR-K1, EM-3, SR-1, SR-7, et des mutations du §7.6).
2. **Découpage en sessions A/B/C** : à choisir au document d'exécution.

Aucun point d'arbitrage d'architecture ne reste ouvert.

## 16. Fin de l'étape
L'étape 6 se clôt quand le §14 est satisfait et que l'utilisateur l'a déclarée après exécution des contrôles sur le dépôt réel. Le document d'exécution suit la validation de ce plan ; le handoff suit la validation du document d'exécution. Aucun travail de visualisation n'est entrepris.
