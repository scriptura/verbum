# Document fondateur conceptuel — LPC (pipeline AOT) — v1.3.5

> **Statut** : **révision consolidée v1.3.5**. Cette version conserve le modèle des **réalisations physiques** et la dérivation contextuelle du `TargetBucket` introduite en v1.3.2. Elle consolide les clarifications issues des audits de cohérence et d’architecture, et simplifie la résolution sémantique autour de `AnimationAction` en écartant les mécanismes spécialisés de variantes d’action.
> **Socle normatif** : **v1.3**. La v1.3.5 consolide les corrections précédentes : dérivation contextuelle du `TargetBucket`, distinction entre équipement conducteur (`DriverEquipmentId`) et couches visuelles non conductrices, correction de la localisation physique du Longsword `thrust`, et simplification de la résolution sémantique autour de `AnimationAction` sans mécanisme spécialisé de variante d’action.
> **Évolution v1.3** : le schéma distingue désormais explicitement **Profile**, **PhysicalRealization**, **LocalizationContract** et **ActionResolution**. La sélection physique est déclarative et doit être unique ; l’optionalité est attachée à la réalisation physique. Aucun changement de responsabilité n’est introduit côté runtime.
> **Portée** : ce document couvre le **pipeline AOT**. Le runtime n'est mentionné que pour fixer les **invariants de frontière**.
> **Exhaustivité** : ce document **n'est pas exhaustif** sur les animations. Il pose le **principe**, des **exemples illustratifs**, des **invariants** et certains profils encore provisoires. Les YAML constituent le **corpus de référence effectivement déclaré** ; l'absence d'un profil provisoire ou non encore matérialisé dans ce corpus ne constitue pas une décision d'obsolescence.

---

## 1. Portée et principes

Ce document établit le **paradigme** qui gouverne la transformation des spritesheets LPC (PNG) en données consommables par un runtime ECS/DOD.

### Principes fondateurs

1. **LPC est une banque de pixels, pas un format d'animation.**
2. **Toute complexité de résolution, d’extraction, de composition structurelle et d’ordonnancement statique est absorbée AOT.**
3. **Le runtime ne manipule que des identifiants et des structures de données précompilées.**
4. **Le système d'animation est un service moteur global.**
5. **Le pipeline est un traducteur, pas un perroquet.**

### Convention d'indexation

**Toutes les rows et colonnes sont indexées en 0-based.**

### Unités de vocabulaire : row, grid_cell, grid, grid_y

| Terme | Signification | Nature |
|---|---|---|
| **row** | Une ligne d'animation LPC = une direction | **Logique** — déclaration YAML |
| **grid_cell** | Un carré de 64×64 dans la grid | **Physique** — brique élémentaire |
| **grid** | L'ensemble des grid_cells d'un PNG, organisé en lignes et colonnes | **Physique** — structure globale |
| **grid_y** | L'ordonnée d'itération dans la grid | **Physique** — coordonnée d'extraction |

- Une animation à 4 directions occupe **4 rows** logiques.
- Dans la **grid**, un row logique (une direction) consomme **1, 2 ou 3 `grid_y` successifs**, selon le `RealizationBucket` et le `frame_size` qui en découle : Small → 1, Medium → 2, Large → 3.

**Règle stricte d'usage** :
- **« row »** désigne toujours l'unité **logique** (une direction).
- **« grid_y »** désigne toujours la coordonnée **physique** d'extraction dans la grid.
- Ces deux termes ne sont **jamais interchangeables**.

**Important** : les YAML de localisation déclarent toujours les **rows logiques** (une par direction). Chaque élément de `rows` donne le `grid_y` de départ correspondant à cette direction ; l'expansion verticale requise par le `frame_size` est dérivée AOT.

### Rôles du `BucketId` : réalisation et cible

Le paradigme distingue deux rôles d'un même `BucketId` :

- **`RealizationBucket`** : bucket de la **réalisation physique source**. Il détermine `frame_size` lors de l'extraction (`Small` → 64, `Medium` → 128, `Large` → 192).
- **`TargetBucket`** : bucket du **canvas final** produit pour le contexte cinématique et consommé par le runtime.

Un `RealizationBucket` peut être inférieur au `TargetBucket`. C'est le cas, par exemple, du `slash` Small du `body` lorsqu'une attaque au Longsword impose un `TargetBucket = Large`.

Cette distinction est normative : **`RealizationBucket` et `TargetBucket` ne sont jamais synonymes**.

### Convention de vocabulaire — préfixes

- **`AnimationAction`** : **jamais de préfixe de catégorie**.
- **Profils (`ExtractionProfile`, `CompositionProfile`)** : **préfixe descriptif toléré** (ex. `tool_whip`).
- Aucun préfixe n'est **jamais** porté par une `AnimationAction`.

### Convention de vocabulaire — chemins et identifiants

**Les chemins, noms de fichiers et identifiants déclaratifs des données sont en lowercase. Les noms de concepts, types et structures du paradigme documentaire (`AnimationAction`, `ExtractionProfile`, `FrameSequence`, etc.) ne sont pas concernés par cette convention.**

---

## 2. Architecture à trois étages

```text
[ÉTAGE 1]   AnimationAction      ← vocabulaire d’animation fourni au runtime
                │
                │  résolution sémantique AOT
                ▼
[ÉTAGE 2]   Profile              ← identité sémantique
            ├── ExtractionProfile
            └── CompositionProfile
                │
                │  réalisation physique déclarative + extraction + composition AOT
                ▼
[ÉTAGE 3]   FrameSequence        ← ce que le runtime consomme
```

### Étage 1 — AnimationAction

C'est le langage du gameplay. C'est ce que les systèmes écrivent. Une `AnimationAction` désigne une action cinématique déclarée par le gameplay. Plusieurs actions sémantiquement distinctes peuvent employer des `Profile` différents ou converger vers le même `Profile`.

Exemples (liste **illustrative**, non exhaustive, **sans préfixes**) :
`attack`, `cast`, `walk`, `run`, `swim`, `idle`, `die`, `stagger`, `watering`, `sit`, `emote`…

- Indépendante de LPC, de l'équipement et de la taille des sprites.
- Ne dit **rien** sur la cinématique physique ni sur les pixels.
- Une `AnimationAction` n'implique pas qu'il n'existe qu'un seul `Profile` associé.

### Vocabulaire des actions et multiplicité des cinématiques

`AnimationAction` constitue le vocabulaire sémantique du gameplay. Une valeur donnée identifie une intention d'action ; elle ne constitue ni une localisation physique, ni un bucket, ni une identité de réalisation.

Le paradigme **n'introduit pas de mécanisme spécialisé de variante d’action**, ni d'équivalent tel que `ParryVariantId`, `CastVariantId`, etc. dans son modèle central. Lorsqu'une même famille d'action possède plusieurs cinématiques distinctes, le gameplay peut déclarer plusieurs `AnimationAction` sémantiquement distinctes et le YAML de résolution les relie aux `Profile` appropriés.

Ainsi, le corpus Longsword peut contenir :

```text
slash
slash_reverse
thrust
```

sans que le paradigme en déduise automatiquement trois variantes d'une même `AnimationAction`. La manière dont le gameplay nomme et distingue ces actions relève de son propre vocabulaire.

**Principe normatif** : **le nombre de cinématiques présentes dans un asset ne détermine pas le nombre d'`AnimationAction`.** Un PNG peut contenir plusieurs cinématiques exploitables par le jeu sans imposer l'existence d'un champ de variante dans le paradigme.

Si deux `AnimationAction` distinctes doivent employer le même mouvement physique, elles peuvent toutes deux converger vers le même `Profile`. Inversement, plusieurs `AnimationAction` peuvent pointer vers des `Profile` différents tout en utilisant le même `DriverEquipmentId`.

Le cas actuel Longsword peut donc être exprimé par une résolution plate :

```text
<action d'attaque du gameplay A> → slash
<action d'attaque du gameplay B> → slash_reverse
<action d'attaque du gameplay C> → thrust
```

Les identifiants concrets sont des décisions du vocabulaire gameplay/YAML ; ils ne doivent pas être confondus avec les `Profile` correspondants ni avec les réalisations physiques.

### `ResolutionContext` — définition normative

Le terme **`ResolutionContext`** désigne une notion conceptuelle AOT : l'ensemble minimal d'informations qui qualifie une demande de résolution sémantique. Il ne constitue ni un objet métier persistant, ni un agrégat runtime.

Dans le paradigme actuel :

```text
ResolutionContext
(
    AnimationAction,
    DriverEquipmentId?
)
```

- `AnimationAction` est toujours présent et constitue l'intention d'action demandée ;
- `DriverEquipmentId` est optionnel et désigne l'équipement actif — typiquement une arme ou un outil — capable d'influencer le contexte cinématique ;
- **`TargetBucket` n'appartient pas au contexte d'entrée par défaut** : il est normalement dérivé après sélection des réalisations physiques ;
- `TargetBucket` peut toutefois être fourni comme information déjà établie lorsqu'une résolution `by_bucket` l'exige.

La résolution nominale suit donc :

```text
ResolutionContext
        ↓
Profile
        ↓
PhysicalRealization(s)
        ↓
TargetBucket dérivé
```

Cette définition permet d'employer le terme `contexte` de manière uniforme dans les sections suivantes sans transformer cette notion en nouvelle couche architecturale.

### Étage 2 — Profile

**`Profile`** est le terme **générique** englobant les deux sous-types :

```text
Profile
├── ExtractionProfile
└── CompositionProfile
```

- **`ExtractionProfile`** — identité sémantique d’un **pool d'extraction**. Décrit `frame_count`, `directions`, et éventuellement une séquence canonique par défaut. Il ne porte aucune coordonnée physique.
- **`CompositionProfile`** — identité sémantique d’un **ordonnancement** appliqué au pool d’un `source_extraction`. Il hérite `frame_count` et `directions` de sa source et possède sa propre `sequence`. Il ne porte aucune coordonnée physique intrinsèque.

### Étage 2 bis — PhysicalRealization

Une **`PhysicalRealization`** est l'occurrence physique concrète d'un `Profile` dans un asset source.

Elle possède une identité propre :

```text
RealizationId
Profile
source asset
RealizationBucket
```

Sa **localisation physique** est ensuite portée par un contrat de localisation ; elle n'est pas contenue dans l'identité sémantique du `Profile`.

Une `PhysicalRealization` peut réaliser aussi bien un `ExtractionProfile` qu'un `CompositionProfile`.

Une même identité sémantique peut donc posséder plusieurs réalisations physiques :

```text
slash
├── RealizationId A → Small → base asset
└── RealizationId B → Large → longsword asset

slash_reverse
└── RealizationId C → Large → longsword asset
```

**Une `PhysicalRealization` n'est pas un nouveau type de `Profile`.** Les désignations documentaires telles que `slash_192` peuvent servir de labels humains, mais ne sont jamais des identités de `Profile`.

### Réalisation physique et localisation

La séparation est stricte :

```text
PhysicalRealization
    = identité physique / existence / bucket / asset

LocalizationContract
    = localisation physique de cette réalisation
```

La localisation contient notamment les `rows` / `grid_y` de départ. Elle ne redéfinit ni le `Profile`, ni le `RealizationBucket`, ni le `TargetBucket`.

### Cas des CompositionProfiles

`source_extraction` reste une relation **sémantique**. Elle fournit notamment `frame_count` et `directions`, mais aucune localisation physique implicite.

Ainsi :

```text
slash (ExtractionProfile)
    │
    ▼
slash_reverse (CompositionProfile, source_extraction: slash)
    │
    ▼
PhysicalRealization C
    Profile = slash_reverse
    RealizationBucket = Large
    localisation = propre
```

La localisation de C n'est jamais héritée de `slash`.

### Désignations `_128` / `_192`

- `walk_128`, `walk_192`, `slash_128`, `slash_192`, `thrust_192` sont des **désignations descriptives de réalisations physiques**.
- Elles ne constituent **pas** un troisième type de `Profile`.
- Elles ne sont jamais utilisées comme cibles du YAML de résolution.

### Précision sur `slash_reverse`

```text
slash (ExtractionProfile)
    │
    ▼
slash_reverse (CompositionProfile, source_extraction: slash,
               sequence: [5,4,3,2,1,0])
    │
    ▼
PhysicalRealization C
    RealizationBucket = Large
```

La réalisation Large possède son propre `RealizationId` et sa propre localisation.

### Distinction critique — pool source vs séquence jouée

- `frame_count` désigne la taille du **pool physique extrait**.
- `len(sequence)` désigne le nombre de frames **effectivement jouées**.
- Ces valeurs peuvent être différentes : `watering` a `frame_count = 8` et `len(sequence) = 7`.

### Étage 3 — FrameSequence

Séquence linéaire, **immuable dans son contenu** (ordre des frames figé), produite exclusivement AOT.

- Ne contient **aucun pixel**.
- Référence des frames stockées dans des structures globales.
- Le runtime n'itère que sur des **indices et offsets précompilés**.
- **Chaque séquence porte un `next_sequence_id`** (propriété adjacente) qui indique la séquence à jouer une fois le curseur épuisé.

**Distinction `sequence` vs `FrameSequence`** :
- `sequence` = **déclaration YAML**, tableau d'indices locaux dans le pool de la réalisation.
- `FrameSequence` = **artefact AOT final**, séquence de références globales consommable par le runtime.

**Identité d'une `FrameSequence` pour la déduplication** :
- l'identité comprend le **contenu ordonné de la séquence** ;
- elle comprend également `next_sequence_id`, qui fait partie de la topologie de succession.

Deux `FrameSequence` ne peuvent donc être dédupliquées que si leur contenu et leur `next_sequence_id` sont identiques. La **clé de génération AOT** et l'**identité de déduplication** restent deux notions distinctes.

---

## 3. Rôle du bucket (paramètre orthogonal)

| Bucket | Résolution |
|---|---|
| Small | 64×64 |
| Medium | 128×128 |
| Large | 192×192 |

- Le `BucketId` est **orthogonal** au `Profile`.
- Le même `Profile` peut être réalisé physiquement dans plusieurs buckets.
- Toutes les frames d'un même bucket ont même taille, même **centre géométrique d'alignement AOT**.
- Dans le **layout LPC Character**, la région Small historique occupe `grid_y 0–53` ; les réalisations oversized observées commencent à `grid_y 54`. Aucun ordre global des buckets oversized n'est imposé par le paradigme.
- Corps, cheveux, vêtements, armures et boucliers observés utilisent des réalisations **Small** ; ils peuvent ensuite être composés dans un `TargetBucket` Medium ou Large.
- Les armes et outils peuvent fournir des réalisations Small, Medium ou Large et sont les équipements conducteurs du `TargetBucket`.

### Réalisation source vs canvas cible

**`RealizationBucket`** désigne la taille intrinsèque de la réalisation physique extraite du PNG.

**`TargetBucket`** désigne la taille du canvas final dans lequel cette réalisation, ou une autre réalisation du même contexte, est consommée.

Exemple :

```
Contexte : `attack` + `longsword`
TargetBucket = Large (192×192)

body :
  Profile = slash
  RealizationBucket = Small
  source_size = 64×64
  → composition au centre du canvas 192×192

longsword :
  Profile = slash
  RealizationBucket = Large
  source_size = 192×192
  → composition directe dans le canvas 192×192
```

Il ne s'agit **jamais** d'un redimensionnement de la source : le body reste 64×64.

### Rôle cinématique et topologie de rendu : deux classifications orthogonales

La classification des éléments visuels selon leur capacité à piloter une résolution cinématique est **distincte** de leur position dans la stack graphique.

| Rôle cinématique | Exemples | Réalisations physiques observées | Effet sur le contexte |
|---|---|---|---|
| **Conducteur** | armes, outils | Small / Medium / Large | Peut modifier le `Profile` et contribuer au `TargetBucket` |
| **Non-conducteur** | body, head, hair, vêtements, armures, boots, shield, etc. | Small dans le corpus LPC visé | Participe à la composition mais ne modifie ni `Profile` ni `TargetBucket` |

`DriverEquipmentId` identifie l'équipement **conducteur** actif. Dans le paradigme actuel, il couvre les armes et les outils.

**Règle normative** : tous les layers constitutifs du personnage participent à la composition AOT. Le fait qu'un layer soit non conducteur ne signifie jamais qu'il est absent de l'animation. Il signifie uniquement qu'il ne pilote pas la résolution cinématique et n'impose pas de promotion de bucket.

**Le bouclier est un non-conducteur**. Sa réalisation physique observée reste Small ; ses règles particulières concernent sa position dans la stack selon la direction (§11), pas la résolution du `Profile` ou du `TargetBucket`.

La classification cinématique **conducteur / non-conducteur** et la topologie de rendu (`LayerId`, ordre de stack, positions `behind/front`) sont donc deux dimensions indépendantes. Une couche peut avoir un rôle de rendu particulier sans devenir pour autant un `DriverEquipmentId`.

### Distinction normative — `DriverEquipmentId` vs équipements non conducteurs

**Tous les objets visuels équipés participent à la composition de l’animation. Mais tous ne sont pas des `DriverEquipmentId`.**

`DriverEquipmentId` désigne uniquement l’équipement actif qui possède une capacité à **piloter le contexte cinématique**. Dans le paradigme actuel, cette catégorie comprend les **armes et les outils**. Un `DriverEquipmentId` peut donc :

- modifier le `Profile` résolu par `by_equipment` ;
- fournir une ou plusieurs `PhysicalRealization` de buckets `Small`, `Medium` ou `Large` ;
- contribuer ainsi à la dérivation du `TargetBucket`.

À l'inverse, les équipements et couches **non conducteurs** participent bien à l'animation courante, mais **ne modifient ni la résolution sémantique du `Profile`, ni le `TargetBucket`**. Dans le corpus LPC visé ici, ils restent physiquement dans le format historique **Small (64×64)** et sont simplement composés dans le canvas du `TargetBucket` courant.

Cela inclut notamment :

```text
body
head
hair
short sleeve
chainmail
long pants
boots
shield
...
```

Le bouclier est un cas particulier de layering, pas un équipement conducteur : sa position dans la stack varie selon la direction (§11), mais cette variation ne lui confère aucun rôle dans la résolution cinématique ou la dérivation du `TargetBucket`.

**Conséquence AOT** : une animation résolue pour `DriverEquipmentId = longsword` doit être composée avec **toutes les couches constitutives du personnage**. Le `DriverEquipmentId` n’est donc pas une whitelist des layers à afficher ; c’est uniquement le **pilote sémantique et physique** du contexte.

### Dérivation du `TargetBucket` à partir des réalisations sélectionnées

**Le `TargetBucket` est le plus grand `RealizationBucket` parmi les réalisations physiques effectivement sélectionnées pour le contexte d’animation courant.**

La dérivation suit donc cette chaîne :

```text
AnimationAction + contexte
        ↓
Profile(s) résolu(s)
        ↓
PhysicalRealization sélectionnée pour chaque layer
        ↓
TargetBucket = max(RealizationBucket effectivement sélectionnés)
```

Il ne faut **jamais** calculer le `TargetBucket` à partir de la capacité maximale de l’équipement sur l’ensemble de ses animations.

Exemple critique :

```text
DriverEquipmentId = longsword

longsword :
    walk  → Small
    slash → Large
    thrust → Large
```

Pour `walk`, seule la réalisation `longsword-walk / Small` est sélectionnée. Le contexte produit donc :

```text
TargetBucket = Small
```

Pour une action d’attaque résolue vers `slash`, la réalisation `longsword-slash / Large` est sélectionnée. Le contexte produit alors :

```text
TargetBucket = Large
```

Une autre action d’attaque résolue vers `thrust` sélectionnera, elle, `longsword-thrust / Large` ; le `TargetBucket` restera `Large`, mais il s’agit d’un **contexte sémantique et physique distinct**.

La présence d’une réalisation Large dans le même équipement ne provoque donc **aucune promotion globale permanente** des autres animations.

- Dans le corpus LPC visé, l’absence de `DriverEquipmentId` conduit actuellement à `TargetBucket = Small` lorsque les réalisations sélectionnées sont toutes Small.
- La règle s’applique identiquement aux armes et aux outils.
- La dérivation est une **déduction de contexte**, pas une transformation physique des assets.

**Invariant fondamental** : le `TargetBucket` dépend du **contexte d’animation courant**, jamais du catalogue complet des capacités physiques d’un équipement.

### Alignement des frames hétérogènes

**Toute frame source, quelle que soit son `RealizationBucket`, est centrée sur le canvas du `TargetBucket` avant composition.**

- Le **centre géométrique** de la frame source doit coïncider avec le **centre géométrique** de la frame cible.
- Convention **invariante**, non négociable, non configurable.
- **Indépendante** du pivot d'ancrage gameplay (hors-scope AOT).
- **Décision de design.** Validation visuelle effectuée.

### Structure en grid des PNG

**Un PNG source LPC est une grid physique de `grid_cell` de 64×64 dans le layout LPC Character.** Le canvas physique du PNG peut contenir des zones entièrement transparentes, des zones partiellement exploitées et plusieurs réalisations de tailles différentes.

- Une **`grid_cell`** = un carré de 64×64 pixels.
- Dans la grid, un **row logique** (une direction) consomme 1, 2 ou 3 **`grid_y` successifs**, selon le `RealizationBucket` : 64 → 1, 128 → 2, 192 → 3.
- La présence d’un `grid_y` dans le canvas ne signifie pas qu’un contenu graphique utile y est présent.

### Région Small historique — `grid_y 0–53`

Dans le **layout LPC Character**, la région physique `grid_y 0–53` constitue le **socle historique Small**. Les observations comparatives de nombreux PNG confirment la persistance de cette même région physique et de cette même convention de coordonnées d’un asset à l’autre.

**Important : la similarité porte sur la région physique et son repère, pas sur le contenu graphique.** Un PNG de base peut remplir largement cette région ; un asset spécialisé peut n’y exposer qu’un sous-ensemble de profils ; un autre peut laisser toute cette région transparente.

Exemples observés :

```
PNG base historique
  grid_y 0–53 → nombreuses réalisations Small

PNG trident
  grid_y 0–53 → présent physiquement, entièrement transparent
  grid_y 54–61 → réalisation Medium de walk
  grid_y 62–73 → réalisation Large de thrust

PNG longsword
  grid_y 0–53 → présent physiquement, partiellement exploité
  grid_y 54–65 → réalisation Large de slash
  grid_y 66–77 → réalisation Large de slash_reverse
  grid_y 78–89 → réalisation Large de thrust
```

La zone des réalisations oversized commence au **`grid_y 54`** dans ce layout. Cette valeur est une propriété empirique du **layout LPC Character**, pas une règle générale applicable à tous les corpus.

### Régions physiques et occupation du canvas

Un PNG peut contenir :

- plusieurs réalisations de buckets différents ;
- plusieurs réalisations d’un même `BucketId` ;
- plusieurs `Profile` distincts dans une même région de bucket ;
- des zones transparentes ou sans contenu exploité ;
- des profils Small partiellement présents au sein de la région historique `0–53`.

Les frontières physiques observées ne définissent pas, à elles seules, une identité de `Profile`. Le contrat d'équipement déclare quelles réalisations et localisations physiques le pipeline choisit d'exploiter.

**Aucun ordre global `Small → Medium → Large` n'est une invariante du paradigme.** Le trident présente `Medium → Large`; le Longsword présente plusieurs réalisations `Large`.

**Sur un même PNG, deux réalisations physiques distinctes peuvent correspondre à deux identités sémantiques partageant le même `source_extraction`.** Le Longsword montre notamment `slash` / `slash_reverse`.

### Origine physique et position horizontale des frames

**L'origine horizontale par défaut des frames est `x = 0`.** Aucun mécanisme de surcharge d'origine n'est prévu à ce stade.

Pour une frame source d'indice local `i` dans le pool extrait (`0 ≤ i < frame_count`), sa position horizontale est déterminée AOT par :

```
x(i) = i × frame_size
```

où `frame_size` est déduit du **`RealizationBucket`** de la réalisation physique.

Les frames sources sont donc contiguës horizontalement, sans padding entre elles. La position horizontale n'est pas déclarée dans les YAML : elle est dérivée de l'indice local `i` et du `RealizationBucket`.

Ainsi, `rows` porte la localisation verticale (`grid_y`) de chaque direction, tandis que l'indice de frame porte sa localisation horizontale.

### Extensibilité à d'autres gabarits

- **Buckets** configurables.
- **Grid_cell de base** configurable par corpus (`64` pour LPC humanoïde).
- **Frame sizes** : tout multiple entier de la grid_cell de base.
- **ExtractionProfiles** extensibles.
- **Layouts physiques** : chaque famille d'assets peut définir son propre layout physique. Le layout LPC Character décrit ici n'est qu'un cas particulier. Un PNG peut n'exploiter qu'un sous-ensemble de ses régions physiques ; une zone transparente reste néanmoins présente dans le canvas.
- **Aucune convention implicite** : chaque YAML déclare explicitement ses valeurs.

**Cas particulier — créatures à animation unique** : 1 direction, 1 frame, sans séquence custom. Elles suivent la même normalisation AOT que toute autre animation mono-directionnelle destinée au runtime.

---

## 4. Extraction, Composition, propriétés AOT et schéma déclaratif

### Extraction

**Ce qu'on lit** dans le PNG :

- quelle **PhysicalRealization** a été sélectionnée pour le layer courant ;
- quel **`RealizationBucket`** lui correspond ;
- quelles **rows** logiques la localisent ;
- quel **nombre de frames source** (`frame_count`, fourni par le `Profile` canonique).

Le `TargetBucket` ne détermine donc **pas** la taille intrinsèque de la source extraite. Il appartient au contexte final de composition.

### Composition ordinale

**Ce qu'on joue** à partir du pool physique sélectionné :

- une **séquence ordonnée** d'indices,
- qui peut **répéter**, **sauter**, **réordonner**, ou toute combinaison équivalente.

### Les trois YAML

**1. YAML canonique d'animations** — un seul, liste les `Profile` sémantiques du corpus.

Chaque entrée porte, selon son type :

- **`ExtractionProfile`** : `name`, `type`, `frame_count`, `directions`, `sequence` (optionnel), `aliases` (optionnel).
- **`CompositionProfile`** : `name`, `type`, `source_extraction`, `sequence`, `aliases` (optionnel).

Un `CompositionProfile` **ne redéclare jamais** `frame_count` ni `directions` : ces propriétés sont héritées de son `source_extraction`.

`source_extraction` doit référencer un `ExtractionProfile` existant ; sinon → **erreur de build**.

La `sequence` d'un `CompositionProfile` est obligatoire.

**2. YAML d'équipement** — localise les **PhysicalRealization** dans les assets sources auxquels le contrat s'applique.

L'unité primaire du contrat est désormais `realizations`, chaque entrée référant une `RealizationId` :

```text
LocalizationContract
├── contract_id
├── applies_to
└── realizations[]
       ├── realization_id
       ├── rows
       └── optional
```

Le contrat **ne porte pas `realization_bucket`**. Le bucket appartient à la `PhysicalRealization` définie au niveau du build manifest / asset registry.

Le contrat ne définit pas non plus le `TargetBucket` final.

### Build manifest / asset registry

Le build manifest / asset registry constitue la déclaration globale de l'existence et de la topologie des PhysicalRealization.

Une entrée conceptuelle contient au minimum :

```text
PhysicalRealization
├── realization_id
├── profile
├── source_asset
├── layer_id / topologie d'asset
└── realization_bucket
```

Il porte également les relations déjà décidées :

```text
DriverEquipmentId
    → réalisations physiques d'équipement disponibles
    → RealizationBucket disponibles
```

**Invariant** : une `RealizationId` identifie une seule **occurrence déclarative** d'un `Profile` pour un `source asset`, un `LayerId` / une topologie et un `RealizationBucket` donnés. Les coordonnées `rows` n'entrent pas dans l'identité sémantique du `Profile` ; elles appartiennent au contrat de localisation.

ainsi que la classification `overlay` / `variant`.

Le manifest **ne porte pas les `rows`** : celles-ci restent dans le contrat de localisation.

### Séparation des responsabilités

**Principe de responsabilité** : le build manifest / asset registry ne connaît pas un `TargetBucket` final par équipement ; il fournit les réalisations physiques et leurs `RealizationBucket`, à partir desquels l'AOT dérive le `TargetBucket` pour chaque `ResolutionContext` effectivement matérialisé.

| Information | Profile canonique | Build manifest / asset registry | Contrat d'équipement | YAML de résolution |
|---|---:|---:|---:|---:|
| `Profile` | ✔ | référence | référence indirecte | cible |
| `frame_count`, `directions` | ✔ | — | — | — |
| `sequence` / `source_extraction` | ✔ | — | — | — |
| `RealizationId` | — | ✔ | référence | — |
| source asset | — | ✔ | sélectionné par matching | — |
| `RealizationBucket` | — | ✔ | — | — |
| `rows` | — | — | ✔ | — |
| optionalité de la réalisation | — | existence/topologie | ✔ | — |
| `AnimationAction` | — | — | — | ✔ |
| `TargetBucket` | — | fournit les `RealizationBucket` nécessaires à sa dérivation AOT | — | donnée de contexte dérivée |

### `RealizationId`

`RealizationId` est une identité déclarative de donnée, unique dans le scope du build.

Il ne porte aucune sémantique de bucket dans son nom. Un label documentaire comme `slash_192` peut être utilisé pour désigner humainement cette réalisation, mais il ne constitue ni une identité de `Profile` ni une référence YAML obligatoire.

Le build doit pouvoir référencer une même identité sémantique depuis plusieurs `RealizationId` distincts.

### `frame_size` déduit du `RealizationBucket`

**`frame_size` n'est jamais déclaré** dans les YAML. Il est déduit du `RealizationBucket` de la PhysicalRealization :

- `Small` → 64×64 ;
- `Medium` → 128×128 ;
- `Large` → 192×192.

Pour une réalisation donnée, chaque élément de `rows` représente le **`grid_y` de départ** d'une direction logique. Les `grid_y` supplémentaires sont déduits de `frame_size / grid_cell` : 1, 2 ou 3.

### Cohérence `rows` ↔ `directions`

Les deux listes sont **parallèles** :

```text
rows[i] ↔ directions[i]
```

avec :

```text
len(rows) == len(directions)
```

Sinon → **erreur de build**.

### Localisation et portée des contrats

`applies_to` est un **sélecteur d'asset source**. Sa forme sérialisée peut utiliser un identifiant exact ou un mécanisme de sélection tel qu'un préfixe de chemin, mais le build doit garantir que, pour toute PhysicalRealization requise, la sélection du contrat est déterministe :

```text
0 contrat applicable → erreur
1 contrat applicable → résolution normale
>1 contrat applicable → erreur d'ambiguïté
```

Aucune priorité ou fusion implicite entre contrats n'est autorisée.

### Sélection physique déclarative

La sélection physique est une opération de build, pas une résolution runtime.

Le modèle conceptuel est :

```text
(LayerId, DriverEquipmentId, Profile)
    ↓
PhysicalRealization candidates
    ↓
exactly one RealizationId
    |             |
    |             +→ 0 → absence de sélection
    |
    +→ >1 → build error
```

Le `TargetBucket` est dérivé des **réalisations effectivement sélectionnées dans le contexte d'animation courant** :

```text
`ResolutionContext`
    ↓
Profile
    ↓
PhysicalRealization sélectionnées
    ↓
TargetBucket = max(RealizationBucket effectivement sélectionnés)
```

Il intervient ensuite dans la **validation de compatibilité** et dans la composition :

```text
RealizationBucket ≤ TargetBucket
```

Sinon → **erreur de build**.

Le `TargetBucket` ne constitue pas une seconde identité de la PhysicalRealization et ne choisit pas heuristiquement entre plusieurs réalisations.

### Optionalité

L'optionalité est attachée à une `RealizationId` **dans le contrat de localisation**. Elle décrit une **exigence contractuelle**, pas la présence ou l'absence physique de la réalisation dans le PNG.

```text
realization_id: body-slash-small
optional: false

realization_id: accessory-slash-large
optional: true
```

Une réalisation requise absente → **erreur de build**.

Une réalisation optionnelle absente → **substitution transparente AOT** (§9).

**La présence physique et l’optionalité sont deux dimensions indépendantes** : une réalisation peut être physiquement présente dans un asset tout en étant contractuellement optionnelle. Inversement, une réalisation peut être optionnelle et absente, auquel cas seule la politique AOT d'absence s'applique.

Cette règle s'applique identiquement aux réalisations d'`ExtractionProfile` et de `CompositionProfile`.

### Contrat YAML d'équipement — forme conceptuelle

```yaml
contract_id: "lpc_base_character"

applies_to:
  path_prefix: "textures/characters/"

realizations:
  - realization_id: "base-slash-small"
    optional: false
    rows: [12, 13, 14, 15]

  - realization_id: "base-thrust-small"
    optional: false
    rows: [4, 5, 6, 7]
```

**Aucun `realization_bucket` n'est déclaré ici.** Il provient de la `PhysicalRealization` référencée.

### Indices locaux vs références globales

**Les indices de `sequence` sont locaux** au pool physique extrait. Le pipeline AOT les remappe en **références globales** consommables par le runtime.

### Frame transparente canonique

Pour chaque `TargetBucket`, l'AOT matérialise **exactement une** frame transparente canonique de dimensions `TargetBucketSize × TargetBucketSize`.

Le paradigme ne fixe **aucune numérotation réservée** : l'AOT maintient la correspondance `TargetBucket → TransparentFrameId`. Cette frame physique partagée ne doit pas être confondue avec l'optionalité d'une réalisation.

### Validation des références croisées

**Règle 0** — pour toute `sequence`, chaque indice local `i` doit satisfaire `0 ≤ i < frame_count`. Sinon → erreur de build.

**Règle 1** — chaque `RealizationId` référencé par un contrat doit exister dans le build manifest / asset registry. Sinon → erreur de build.

**Règle 2** — chaque `Profile` utilisé par une `PhysicalRealization` doit exister dans le YAML canonique. Sinon → erreur de build.

**Règle 3** — chaque `CompositionProfile.source_extraction` doit référencer un `ExtractionProfile`. Sinon → erreur de build.

**Règle 4** — pour chaque contexte généré, la sélection physique doit être unique. Plusieurs réalisations candidates applicables → erreur d'ambiguïté.

**Règle 5** — 0 contrat applicable / >1 contrat applicable → erreur selon la règle de portée ci-dessus.

**Règle 6** — présence simultanée de `by_equipment` et `by_bucket` dans une même entrée `AnimationAction` → erreur de build.

**Règle 7** — une réalisation sélectionnée doit satisfaire `RealizationBucket ≤ TargetBucket`. Sinon → erreur de build.

**Règle 8** — `optional: true` ne peut être déclaré que pour une `RealizationId` explicitement référencée par le contrat.

## 5. Vocabulaire LPC vs vocabulaire interne

### Décisions

- **Nomenclature des variantes physiques** : les suffixes `_128`, `_192` sont réservés aux désignations descriptives de réalisations physiques par bucket ; ils ne constituent jamais une partie de l’identité d’un `Profile` et ne sont jamais utilisés comme cibles dans les YAML.
- **Règle lexicale** : aucun `name` de `Profile` ne peut se terminer par `_128` ou `_192`. Dans les YAML de résolution, les valeurs sont toujours des identités pures de `Profile`.
- **Terminologie « Oversized Equipment »** : réalisations physiques dont le `RealizationBucket` dépasse Small, pour armes et outils.

### Profils écartés

**Un seul terme : « écarté »**, avec sous-catégorie « écarté par héritage » pour les dérivés. **Source unique : §7.**

### Pivot d'ancrage — hors-scope AOT

**Le pivot d'ancrage gameplay** est **hors-scope** du pipeline AOT.

- Il concerne le **runtime** (positionnement, hitbox).
- Il n'apparaît **pas** dans les YAML.
- Le pipeline applique l'alignement par **centre géométrique** sans qu'il soit besoin de le déclarer.

---

## 6. Invariant de timing

- **Toutes les frames ont la même durée.**
- **Toute animation mono-directionnelle destinée au runtime est normalisée AOT sur les quatre directions canoniques par référencement de la même `FrameSequence`, y compris le cas des créatures à animation unique.**
- Tenir une pose = **répéter la frame** dans la séquence.

### Politique de boucle — résolue AOT

**Graphe de succession.**

- `séquence A → séquence A` pour un **loop**.
- `séquence A → séquence B` pour une **transition**.

Chaque `FrameSequence` porte un `next_sequence_id`.

**Politique de bout** : une séquence terminale pointe vers un **sentinel** (`next_sequence_id = 0` ou valeur réservée). Le runtime maintient alors le curseur sur la dernière frame.

**Conséquence runtime** : la progression d'animation ne requiert aucune allocation dynamique ; la topologie est dictée par l'AOT, sans connaissance sémantique de la boucle.

### Limite reconnue

Invariant calibré pour le gameplay (séquences ≤ 13 frames). Cinématiques longues : hors-scope.

---

## 7. Table canonique des profils

> **Note** : cette table est **illustrative**. La liste complète relève du YAML canonique.

### Directions

| Contrainte | Valeur |
|---|---|
| Direction canonique | `[top, left, down, right]` |
| Direction source de `hurt` | `[down]` |
| Direction source de `climb` | `[top]` |
| Variantes oversize | **toujours 4 directions** |

**Normalisation AOT des profils mono-directionnels** : lorsqu'un `ExtractionProfile` ne déclare qu'une seule direction (`hurt` ou `climb`), le pipeline AOT **réplique la séquence résolue sur les quatre directions canoniques**. Chaque direction normalisée référence ainsi la même `FrameSequence` ; cette réplication d'adressage n'implique pas quatre copies du stockage, la déduplication pouvant partager la séquence. Le runtime conserve ainsi un domaine de `Direction` uniforme ; aucune règle de remapping directionnel n'est nécessaire au runtime.


### ExtractionProfiles canoniques

| ExtractionProfile | directions | frame_count | sequence |
|---|---|---|---|
| `spellcast` | 4 | 7 | défaut |
| `thrust` | 4 | 8 | défaut |
| `walk` | 4 | 9 | `[1,2,3,4,5,6,7,8]` |
| `slash` | 4 | 6 | défaut |
| `shoot` | 4 | 13 | défaut |
| `hurt` | 1 (`down`) | 6 | défaut |
| `climb` | 1 (`top`) | 6 | défaut |
| `run` | 4 | 8 | défaut |
| `jump` | 4 | 5 | `[0,1,2,3,4,1]` |
| `sit` | 4 | 3 | `[0×5,1×5,2×5]` |
| `emote` | 4 | 3 | `[0×5,1×5,2×5]` |
| `idle` | 4 | 2 | `[0,0,1]` |

**Note sur `walk`** : `frame_count = 9`. La frame 0 existe mais est ignorée. Séquence à partir de `1`.

**Note sur la `sequence` d'un ExtractionProfile** : séquence canonique par défaut, utilisée quand aucun `CompositionProfile` ne dérive de lui.

### CompositionProfiles (exemples)

> Les entrées marquées **« à préciser »** ou **« provisoire »** sont illustratives et ne constituent pas encore des profils canoniques effectivement déclarés par le corpus YAML courant.

| CompositionProfile | source_extraction | sequence |
|---|---|---|
| `watering` | `thrust` | `[0,1,4,4,4,4,5]` |
| `tool_whip` | `slash` | `[0,1,2,3,4,5]` |
| `tool_axe` | `slash` | `[5,5,4,4,3,1,0,0,0,0]` |
| `slash_reverse` | `slash` | `[5,4,3,2,1,0]` |
| `tool_hoe` | `slash` | à préciser (YAML) |
| `tool_shovel` | `slash` | à préciser (YAML) |
| `swim` | `spellcast` | à préciser (provisoire, voir §13) |

**Note sur `slash_reverse`** : modélisé comme `CompositionProfile` dérivé de `slash`, avec une réalisation physique Large distincte dans le corpus Longsword. Le bloc observé est `grid_y 66–77` et il est immédiatement suivi par la réalisation `thrust` `grid_y 78–89`. Il n'y a aucun padding entre ces deux blocs.

### Variantes physiques de bucket (descriptives)

> **Rappel** : ces entrées ne sont **pas** des `ExtractionProfile` autonomes. Elles désignent le même `ExtractionProfile` composé dans un bucket plus grand. Le suffixe `_128` / `_192` est descriptif.

| ExtractionProfile de base | Small | Medium | Large |
|---|---|---|---|
| `walk` | `walk` | `walk_128` | `walk_192` |
| `thrust` | `thrust` | — | `thrust_192` |
| `slash` | `slash` | `slash_128` | `slash_192` |

### Rows LPC canoniques (0-based) — région Small historique

| ExtractionProfile | rows (logiques) | Nb rows |
|---|---|---|
| `spellcast` | 0–3 | 4 |
| `thrust` | 4–7 | 4 |
| `walk` | 8–11 | 4 |
| `slash` | 12–15 | 4 |
| `shoot` | 16–19 | 4 |
| `hurt` | 20 | 1 |
| `climb` | 21 | 1 |
| `idle` | 22–25 | 4 |
| `jump` | 26–29 | 4 |
| `sit` | 30–33 | 4 |
| `emote` | 34–37 | 4 |
| `run` | 38–41 | 4 |
| `combat_idle` | 42–45 | 4 (écarté) |
| `1h_slash` | 46–49 | 4 (écarté) |
| `1h_backslash` | 50–53 | 4 (écarté) |

### Blocs / réalisations physiques — exemples trident et Longsword

| Réalisation physique | PNG | grid_y observés | frame_size (déduit) |
|---|---|---|---|
| `walk_128` | trident | 54–61 | 128×128 |
| `thrust_192` | trident | 62–73 | 192×192 |
| `slash_192` | longsword | 54–65 | 192×192 |
| `slash_reverse_192` | longsword | 66–77 | 192×192 |
| `thrust_192` | longsword | 78–89 | 192×192 |

**Note critique** : **dans le layout LPC Character**, la **région Small (`grid_y` 0–53) est commune comme région physique et repère**. Son contenu peut toutefois varier selon l'asset, y compris être entièrement transparent. **La zone des réalisations oversized commence au `grid_y` 54.** Ces valeurs sont **spécifiques à ce layout**, pas des invariantes générales du pipeline AOT.

### Profils écartés

| Profil | Statut | Raison |
|---|---|---|
| `combat_idle` | écarté | Support très limité. |
| `1h_slash` | écarté | Terminologie obsolète. |
| `1h_backslash` | écarté | Terminologie obsolète. |
| `1h_halfslash` | écarté | Terminologie obsolète. |
| `backslash` | écarté | Nommage plugin. |
| `halfslash` | écarté | Nommage plugin. |
| `backslash_128` | écarté par héritage | Dérivé de `backslash`. |
| `halfslash_128` | écarté par héritage | Dérivé de `halfslash`. |

**Note** : `1h_backslash` et `backslash` désignent la même réalité sous deux nommages distincts.

**Invariant de sélection** : un ExtractionProfile n'est canonique que si tous les layers essentiels le supportent (whitelist empirique).

**Invariant d'héritage** : un CompositionProfile dérivé d'un ExtractionProfile **écarté** est **écarté par construction**.

---

## 8. Table de résolution `AnimationAction → Profile`

**Artefact central du paradigme.**

> **Une `AnimationAction` résout vers un `Profile` sémantique — `ExtractionProfile` ou `CompositionProfile` — à partir du `ResolutionContext`. Le choix de la PhysicalRealization intervient ensuite, layer par layer, selon les données déclaratives du build et les contrats de localisation.**

### Structure conceptuelle

```text
AnimationAction
├── (au plus un axe de sélection parmi :)
│   ├── by_equipment
│   └── by_bucket
└── default (repli)
```

**Invariant d'axe de résolution** : pour une `AnimationAction` donnée, la résolution déclare **au plus un axe de sélection** parmi `by_equipment` et `by_bucket`. La présence simultanée des deux axes constitue une **erreur de build**. `default` n'est pas un axe de variation : il constitue la résolution par défaut.

**Résolution sémantique** : une entrée `AnimationAction` résout le `Profile` demandé à partir du `ResolutionContext`. Le paradigme n'introduit pas de qualification spécialisée propre à `attack`. Une pluralité de cinématiques est représentée, lorsque le gameplay en a besoin, par plusieurs `AnimationAction` sémantiquement distinctes.

**Rôle de `by_equipment`** : `by_equipment` permet de faire dépendre le `Profile` de `DriverEquipmentId`. Il peut donc participer à l'établissement de la résolution sémantique du contexte.

**Rôle de `by_bucket`** : `by_bucket` reste un mécanisme sémantique dérivé. Il sélectionne un `Profile` à partir d'un `TargetBucket` **déjà établi indépendamment**. Il ne sélectionne jamais une `PhysicalRealization` et ne référence jamais `walk_128`, `walk_192`, `slash_192`, etc.

### Résolution en deux étapes

La séparation reste normative :

```text
ÉTAPE A — résolution sémantique
ResolutionContext
    ↓
Profile

ÉTAPE B — résolution physique
LayerId + DriverEquipmentId + Profile
    ↓
PhysicalRealization
```

Le `TargetBucket` n'est pas une identité de `PhysicalRealization`. Il est normalement calculé **après** la sélection physique à partir du maximum des `RealizationBucket` effectivement sélectionnés.

### Convergence

Plusieurs `AnimationAction` peuvent se résoudre vers le même `Profile` :

| AnimationAction | Profile |
|---|---|
| `die` | `hurt` (ExtractionProfile) |
| `stagger` | `hurt` (ExtractionProfile) |

Cette convergence ne doit pas être confondue avec une mécanique de variantes. De même, plusieurs `AnimationAction` peuvent pointer vers des `Profile` différents tout en partageant le même `DriverEquipmentId`.

Le corpus Longsword observé peut ainsi être exprimé, selon les besoins du gameplay, comme trois actions sémantiquement distinctes :

```text
<action d'attaque A> → slash
<action d'attaque B> → slash_reverse
<action d'attaque C> → thrust
```

Le paradigme ne fixe ni leurs noms ni leur cardinalité. **La présence de trois blocs physiques dans le PNG ne crée pas à elle seule une nouvelle dimension du modèle.**

### Matérialisation AOT

```text
[LayerId][TargetBucket][AnimationAction][Direction][DriverEquipmentId]
    → SequenceId
```

Cette table est générée exclusivement AOT. Les identifiants `Profile`, `RealizationId` et `RealizationBucket` n'ont pas besoin d'être connus du runtime.

Le runtime n'effectue aucune sélection physique.

### Résolution physique : aucune heuristique

La règle est :

```text
0 candidate  → absence de sélection
1 candidate  → sélection déterministe
>1 candidate → erreur de build
```

Une implémentation peut naturellement indexer cette sélection via le build manifest / asset registry ; le paradigme n'impose pas de structure logicielle particulière.

### Note terminologique

| Notion | Niveau | Nature |
|---|---|---|
| **Convergence** | Étage 1 → 2 | Plusieurs `AnimationAction` vers un même `Profile`. |
| **Sélection physique** | Build AOT | `Profile` → `PhysicalRealization` pour un layer/contexte donné. |
| **Composition ordinale** | Étage 2 → 3 | Réordonnancement, répétition. |
| **Déduplication** | Étage 3 (AOT) | Déduplication mécanique. |
| **Assemblage multi-couches** | Runtime | Superposition spatiale. |

## 9. Composition multi-couches, déduplication

### Composition dans le bucket cible

Pour chaque combinaison `(LayerId, AnimationAction, contexte de résolution)` :

1. Résoudre le **`ResolutionContext`** (`AnimationAction` + `DriverEquipmentId` éventuel) vers le `Profile` concerné.
2. Pour chaque `LayerId` concerné, déterminer la **PhysicalRealization** déclarativement sélectionnée pour ce `Profile` et cet asset source.
3. À partir de ces réalisations effectivement sélectionnées, déterminer le `TargetBucket` comme le maximum de leurs `RealizationBucket`.
4. Résoudre le contrat de localisation applicable à chaque `RealizationId`.
5. Extraire les frames du PNG source à partir des `grid_y` de départ déclarés pour chaque réalisation, en appliquant la largeur/hauteur induite par son `RealizationBucket`.
6. Si une réalisation correspond à un `CompositionProfile`, appliquer sa `sequence` au pool physique fourni par **cette même réalisation**. `source_extraction` fournit les métadonnées sémantiques du pool, mais n'impose aucune localisation physique.
7. Vérifier `RealizationBucket ≤ TargetBucket`.
8. **Composer la source dans le canvas du `TargetBucket`** — sans redimensionnement :
   - toile vide `TargetBucketSize × TargetBucketSize` ;
   - offset centré : `offset = (TargetBucketSize - source_size) / 2` ;
   - blitter la source à cet offset ;
   - padding transparent autour.
9. Si aucune réalisation physique applicable n'existe alors qu'une absence physique optionnelle est déclarée : **FrameSequence transparente** de même longueur/topologie que la séquence qu'elle remplace.
10. Produire la `FrameSequence` finale.
11. Empaqueter.

**Rappel important** : la source n'est **jamais** redimensionnée. Un body 64×64 reste 64×64 ; il est simplement positionné dans un canvas Medium ou Large.

**Distinction fondamentale** : `RealizationBucket` décrit la source extraite ; `TargetBucket` décrit le canvas final.

### Exemple normatif — plusieurs actions d'attaque avec `longsword`

Le corpus Longsword observé démontre trois cinématiques d'attaque distinctes :

```text
slash
slash_reverse
thrust
```

Le paradigme ne les transforme pas automatiquement en variantes d'une même `AnimationAction`. Selon le vocabulaire gameplay effectivement retenu, chacune peut être adressée par une `AnimationAction` distincte :

```text
<action d'attaque A> → slash
<action d'attaque B> → slash_reverse
<action d'attaque C> → thrust
```

Pour la première action, notre fixture actuelle connaît déjà :

```text
AnimationAction   = attack
DriverEquipmentId = longsword
Profile           = slash
TargetBucket      = Large (192×192)
```

Pour une autre action d'attaque aboutissant à `thrust` :

```text
AnimationAction   = <action d'attaque déclarée par le gameplay>
DriverEquipmentId = longsword
Profile           = thrust
TargetBucket      = Large (192×192)
```

La réalisation physique `longsword-thrust` est localisée sur `grid_y 78–89`, sans padding entre elle et la réalisation `slash_reverse` (`66–77`).

Les layers corporels ou autres non-conducteurs utilisent leurs propres réalisations physiques du `Profile` résolu, généralement Small dans le corpus LPC visé ; ils sont ensuite composés au centre du canvas Large.

**Règle structurante** : le nom de l'`AnimationAction` exprime l'intention gameplay ; le `Profile` décrit la cinématique sémantique ; `PhysicalRealization` et `LocalizationContract` portent la matérialité. Une nouvelle cinématique trouvée dans un PNG n'impose donc pas automatiquement une nouvelle abstraction intermédiaire.

**`slash_reverse`** réalise le `CompositionProfile` `slash_reverse`, dont `source_extraction = slash`. Il possède un `RealizationId` et une localisation propres ; il ne réutilise pas implicitement la réalisation `slash_192`.

### Invariant de sélection

Pour un contexte donné :

```text
(LayerId, DriverEquipmentId, Profile)
    → exactement une PhysicalRealization
```

ou :

```text
aucune
```

ou :

```text
plusieurs → erreur de build
```

Il n'existe aucune règle « largest compatible », « smallest compatible », ni « exact bucket otherwise ».

### Invariant de validation : `source_size ≤ TargetBucketSize`

Ici :

- `source_size` désigne la dimension intrinsèque de la PhysicalRealization extraite du layer ;
- `TargetBucketSize` désigne la dimension de la frame finale consommée par le runtime ;
- le `RealizationBucket` détermine `source_size` ;
- le `TargetBucket` détermine `TargetBucketSize`.

Sinon → **erreur de build**.

### Politique de bucket cible

`TargetBucket` = maximum des `RealizationBucket` **effectivement sélectionnés pour le contexte courant**.

- Il ne dépend pas des réalisations non sélectionnées.
- Il ne dépend pas du maximum global des capacités de `DriverEquipmentId`.
- Dans le corpus LPC visé, l’absence de `DriverEquipmentId` conduit actuellement à `TargetBucket = Small` lorsque les réalisations sélectionnées sont toutes Small.
- Avec un équipement conducteur, une réalisation Large sélectionnée pour l’action courante impose `TargetBucket = Large`.
- Une réalisation Large appartenant à une autre action n’impose rien au contexte courant.

### Dépendance et ordre de résolution du `TargetBucket`

La dérivation corrigée impose un ordre AOT explicite :

```text
1. `ResolutionContext` (`AnimationAction` + `DriverEquipmentId` éventuel)
2. `Profile` sémantique
3. PhysicalRealization par layer
4. TargetBucket
5. validation / composition
6. FrameSequence
```

Le `TargetBucket` ne doit donc pas être utilisé pour sélectionner la PhysicalRealization elle-même dans la même résolution, car cela créerait une dépendance circulaire :

```text
TargetBucket → PhysicalRealization → TargetBucket
```

**Règle normative pour `by_bucket`** : lorsqu’une `AnimationAction` utilise `by_bucket`, le bucket utilisé pour cette résolution doit être **déjà établi par un contexte indépendant**. Il ne peut pas être calculé à partir des réalisations que cette même résolution doit encore sélectionner. En l’absence d’un tel contexte indépendant, le couple est **invalide et absent de l’espace généré**.

Le cas nominal des actions utilisant `default` ou `by_equipment` ne présente pas cette circularité : le `Profile` est d’abord résolu à partir du `ResolutionContext`, puis les réalisations physiques sont sélectionnées, puis le `TargetBucket` est calculé.

`by_bucket` reste un mécanisme de résolution sémantique, mais il est **dérivé** et non fondateur du contexte : il n’est valide que lorsque le `TargetBucket` dont il dépend a déjà été établi indépendamment.

### Test de non-régression — `walk + longsword`

Le cas concret qui motive v1.3.2 doit désormais produire :

```text
AnimationAction = walk
DriverEquipmentId = longsword
Profile = walk

body / walk       → Small
head / walk       → Small
hair / walk       → Small
clothes / walk    → Small
...
longsword / walk  → Small

TargetBucket = Small
```

Pour le Longsword, la réalisation physique est localisée sur :

```text
grid_y 8
9
10
11
```

soit quatre directions Small, une `grid_y` par direction. Aucune composition `64×64 → 192×192` n’est produite pour ce contexte.

Le même équipement peut néanmoins produire un autre contexte :

```text
AnimationAction = attack
Profile = slash
longsword / slash → Large
TargetBucket = Large
```

La promotion est donc **locale au contexte d’animation**, et non une propriété permanente du personnage équipé.

### Test de non-régression — trois cinématiques d'attaque `longsword`

Le corpus Longsword contient trois cinématiques physiques distinctes : `slash`, `slash_reverse` et `thrust`. Le paradigme ne leur impose pas une hiérarchie de variantes ; elles peuvent être adressées par trois `AnimationAction` sémantiquement distinctes si le gameplay les distingue ainsi :

```text
<action d’attaque A> → slash
<action d’attaque B> → slash_reverse
<action d’attaque C> → thrust
```

Ces trois contextes peuvent partager le même `DriverEquipmentId` et le même `TargetBucket`, tout en produisant des `FrameSequence` différentes parce que le `Profile` résolu, et donc les réalisations physiques sélectionnées, diffèrent.

La règle reste symétrique avec `walk + longsword` : le fait que trois réalisations d’attaque Large existent ne transforme pas `walk` en Large. Les capacités physiques sont évaluées dans le contexte de l’action réellement demandée.

### Fallback transparent vs erreur de build

| Situation | Traitement |
|---|---|
| Réalisation physiquement requise absente | **Erreur de build** |
| Réalisation physiquement optionnelle absente | **FrameSequence transparente** |
| `CompositionProfile.source_extraction` manquant | **Erreur de build** |
| Contrat de localisation requis manquant | **Erreur de build** |
| Plusieurs contrats applicables à une même réalisation | **Erreur de build (ambiguïté)** |
| Plusieurs réalisations physiques candidates | **Erreur de build (ambiguïté)** |
| `RealizationBucket > TargetBucket` | **Erreur de build** |
| Couple `(DriverEquipmentId, TargetBucket)` invalide | **Absent de l'espace généré** |

### Matérialisation des absences optionnelles

Lorsqu'une `RealizationId` déclarée `optional: true` est absente physiquement pour le layer courant, l'AOT matérialise une **`FrameSequence` transparente ordinaire** dans le scope de déduplication de ce layer.

- Chaque frame de cette séquence référence la frame transparente canonique du **`TargetBucket`**.
- La `FrameSequence` transparente n'est pas un sentinel et n'introduit aucune branche runtime.
- Elle conserve la **même longueur logique** que la séquence qu'elle remplace.
- Elle conserve également la **même topologie de succession après substitution AOT**.
- Ainsi, une boucle `A → A` devient `A_transparent → A_transparent` ; une transition `A → B` devient `A_transparent → B` ou `A_transparent → B_transparent` si B est lui-même substitué.
- La frame transparente physique est partagée au niveau du `TargetBucket`, tandis que la `FrameSequence` transparente reste dans le scope de déduplication du layer.

### Traitement des overlays

Un effet visuel peut être traité de **deux manières** :

- **Layer séparé** : l'asset constitue un `LayerId` distinct et est généré indépendamment.
- **Variante d'un layer existant** : l'asset constitue une réalisation alternative d'un `LayerId` existant et reste dans le scope de génération et de déduplication de ce layer.

**Règle de classification** : la distinction `overlay` / `variant` est une **propriété déclarative du build manifest / asset registry**. Elle n'est **pas déduite du contenu des pixels du PNG**.

### Déduplication et génération — quatre notions distinctes

| Notion | Définition |
|---|---|
| **Scope de déduplication** | Au sein d'un même layer (ou variante de layer). |
| **Clé de génération AOT** | `(LayerId, TargetBucket, AnimationAction, Direction, DriverEquipmentId)` — limitée aux couples valides. |
| **Identité de déduplication** | `(contenu ordonné des frames, next_sequence_id)`. |
| **Clé d'adressage runtime** | La matrice finale `[LayerId][TargetBucket][AnimationAction][Direction][DriverEquipmentId] → SequenceId`. |

**Anti-explosion combinatoire** : l'AOT **n'itère pas** sur les combinaisons de layers. Chaque layer est traité indépendamment.

**Déduplication** : après génération, les `FrameSequence` identiques **dans le même scope de layer** partagent le même stockage. L'identité inclut `next_sequence_id`.

**La frame transparente canonique est une donnée distincte de cette règle** : une seule frame transparente physique existe par `TargetBucket`, et peut être référencée par plusieurs layers sans que leurs `FrameSequence` deviennent pour autant partageables entre layers.

## 10. Frontière AOT / runtime

### Connaît
- `LayerId`, `TargetBucket`, `AnimationAction`, `Direction`, `DriverEquipmentId`
- Identifiants et structures AOT déjà résolues
- Taille de frame résolue

**`DriverEquipmentId`** identifie l'équipement actif qui peut piloter le **contexte cinématique**. Ces identifiants servent uniquement à indexer une résolution déjà matérialisée ; le `TargetBucket` est issu des réalisations physiques effectivement sélectionnées. Le runtime reçoit des réalisations déjà résolues par l'AOT ; il ne choisit pas leurs localisations physiques.

Le runtime ne connaît aucun mécanisme spécialisé de variante d'action. Une distinction supplémentaire entre actions relève du vocabulaire `AnimationAction` déjà déclaré et de la table d'adressage AOT correspondante. Il ne déclenche aucune recherche de `Profile`, de `PhysicalRealization`, de `RealizationBucket` ou de localisation `grid_y`.

**Précision** : le runtime utilise `TargetBucket` et `DriverEquipmentId` comme **identifiants d'indexation**, mais ignore leur **sémantique** (pas de branche sur `if bucket == Large`), ainsi que les concepts de `RealizationBucket`, `row`, `grid_cell`, `grid`, `grid_y`.

### Ignore
- LPC, rows, grid_cells, grid, grid_y, PNG, `Profile`, `ExtractionProfile`, `CompositionProfile`
- Pivot d'ancrage gameplay
- Logique de résolution des contrats
- Interprétation des YAML

### Fait
- Écrit `AnimationAction` + `Direction`
- **Indexe par l'équipement** via `DriverEquipmentId`, **sans branche conditionnelle**
- Lit un identifiant de séquence et un index de frame
- **Applique la topologie de succession** (`next_sequence_id`)
- Superpose les layers dans l'ordre canonique

### Ne fait jamais
- Déduire, parser, inférer
- Brancher conditionnellement sur l'équipement
- Évaluer une politique de boucle
- Modifier l'ordre des frames d'une `FrameSequence`

### Frontière temporelle

L'AOT fige la topologie spatiale, ordinale et structurelle. Le runtime reste maître du domaine temporel.

---

## 11. Invariants LPC préservés

> **Note** : cette liste est **récapitulative**. Les formulations détaillées se trouvent dans les sections citées.

### Vocabulaire et indexation
- **Quatre unités** : `row`, `grid_cell`, `grid`, `grid_y`. §1
- « Row » = ligne d'animation LPC (une direction). §1
- « grid_y » = coordonnée d'extraction dans la grid. §1
- Row et grid_y ne sont jamais interchangeables. §1
- Indexation 0-based. §1
- Chemins, noms de fichiers et identifiants déclaratifs des données en lowercase ; les noms de concepts, types et structures documentaires ne sont pas concernés. §1

### Préfixes
- `AnimationAction` : jamais de préfixe. §1
- Profils : préfixe descriptif toléré. §1

### `ResolutionContext`
- `ResolutionContext` est une notion conceptuelle AOT, utilisée pour qualifier une demande de résolution ; il ne constitue ni un objet métier persistant ni un agrégat runtime. §1, §8
- `ResolutionContext` comprend `AnimationAction` et `DriverEquipmentId` éventuel ; le `TargetBucket` est normalement une donnée dérivée. §1, §3, §8

### Profils
- `Profile` = terme générique englobant `ExtractionProfile` et `CompositionProfile`. §2
- Une **réalisation physique** est une occurrence concrète d'un `Profile` dans un PNG source et un `RealizationBucket` donnés ; elle possède sa propre localisation physique. §2
- Plusieurs réalisations physiques peuvent correspondre à une même identité sémantique. §2, §3
- `RealizationBucket` et `TargetBucket` sont deux rôles distincts d'un même `BucketId`. §1, §3
- Un `ExtractionProfile` possède une séquence canonique par défaut. §2
- `walk_128`, `walk_192`, `slash_128`, `slash_192`, `thrust_192` sont des **désignations descriptives de réalisations physiques de bucket**, jamais des identités de `Profile` ni des cibles YAML. §2, §8
- `slash_reverse` est un `CompositionProfile` dérivé de `slash`. Le Longsword fournit une réalisation physique Large distincte correspondant à ce profil. §2, §13

### Contextualisation des contrats
- L'AOT **ne fusionne jamais** les contrats. §4
- Un même `Profile` peut apparaître dans plusieurs réalisations physiques et plusieurs contrats, avec des `rows` propres à chaque `RealizationBucket`. §4
- Un `CompositionProfile` ne porte **pas** de coordonnées physiques intrinsèques ; une réalisation physique de ce profil peut être localisée indépendamment du `source_extraction`. §2, §4
- 0 contrat applicable → erreur ; 1 contrat → résolution normale ; >1 → erreur d'ambiguïté. §4

### Couples (DriverEquipmentId, TargetBucket)
- Seuls les couples valides sont matérialisés. §4
- Pas de produit cartésien. §4
- Le runtime utilise les identifiants comme clés d'indexation, sans interpréter leur sémantique. §4

### Directions
- Directions canoniques `[top, left, down, right]`. §7
- Exceptions de source : `hurt` = `[down]`, `climb` = `[top]`. §7
- Les profils mono-directionnels sont normalisés AOT sur les quatre directions canoniques par réplication de la même `FrameSequence`. §7
- Variantes oversize : toujours 4 directions. §7

### Structure PNG
- Un PNG est une **grid** de grid_cells de 64×64. §3
- Blocs contigus, sans padding. §3
- **Dans le layout LPC Character**, la région Small `grid_y 0–53` est commune comme région physique et repère ; son contenu peut être complet, partiel ou transparent selon l'asset. §3
- **Dans le layout LPC Character**, la zone des blocs oversize commence au grid_y 54. §3
- Sur un même PNG, les variantes de taille d'un même `ExtractionProfile` occupent des blocs physiques distincts et possèdent des `grid_y` propres à leur `BucketId`. §3
- Les valeurs du layout LPC Character ne sont **pas** des invariantes générales du pipeline AOT. §3

### Alignement et dimensions
- Toute frame centrée sur le centre géométrique. §3
- `source_size ≤ BucketSize` (erreur sinon). §9
- `source_size` désigne la dimension intrinsèque de la réalisation physique ; `TargetBucketSize` la dimension de la frame finale ; `frame_size` dérive du `RealizationBucket`. §9
- **Origine physique par défaut : `x = 0`.** §3
- **Position horizontale** : pour l'indice local `i`, `x(i) = i × frame_size`, sans padding horizontal. §3, §4
- **Correspondance positionnelle** : `rows[i] ↔ directions[i]`. §4

### Composition dans le bucket
- Un layer n'est **jamais redimensionné** pour le bucket. §3, §9
- Un layer est **composé** dans le canvas du bucket cible (positionnement centré, padding transparent). §3, §9
- Pour un même contexte cinématique, les layers peuvent utiliser des réalisations physiques différentes du même `Profile`. §8, §9

### Bucket
- Deux rôles cinématiques : conducteur / non-conducteur ; les exemples de conducteurs actuels sont les armes et les outils. §3
- Promotion contextuelle du `TargetBucket` lorsqu’une réalisation Medium ou Large est effectivement sélectionnée. §3
- **Dans le corpus LPC visé, l’absence de `DriverEquipmentId` conduit actuellement à `TargetBucket = Small` lorsque les réalisations sélectionnées sont toutes Small.** §3

### Équipement
- `DriverEquipmentId` identifie l’équipement actif — arme ou outil — qui peut piloter le contexte cinématique ; il ne détermine pas directement le `TargetBucket`, lequel est dérivé des réalisations effectivement sélectionnées. §4
- Le build manifest / asset registry associe `DriverEquipmentId` aux réalisations physiques d'équipement et à leurs `RealizationBucket` disponibles ; le YAML d'équipement reste scopé à la localisation physique. §4
- Le `DriverEquipmentId` peut établir le contexte cinématique commun ; chaque layer peut utiliser une réalisation physique distincte du `Profile` résolu. Le `TargetBucket` est ensuite dérivé des réalisations effectivement sélectionnées. §4, §8, §9
- Le bouclier est un équipement visuel auxiliaire, sans impact sur la cinématique ni sur le `TargetBucket`. §3
- Les assets de bouclier observés restent au format Small du layout historique. §3
- Jamais de dual wield. §3
- Bouclier : position variable selon direction. §11 (voir ci-dessous)

### YAML et validation
- **Trois YAML distincts** : canonique + équipement + résolution. §4
- `frame_size` déduit du `RealizationBucket`. §4
- CompositionProfiles héritent `frame_count` et `directions` de leur `source_extraction`. §4
- `RealizationId` manquant ou inconnu → erreur. Cible de résolution inexistante → erreur. Ambiguïté de contrat ou de sélection physique → erreur. `by_equipment` et `by_bucket` simultanément dans une même action → erreur. Le YAML de résolution constitue la déclaration du vocabulaire `AnimationAction`. §4, §8
- `tool_whip` et `tool_axe` dérivent de `slash`. §4
- Le YAML canonique est une **déclaration d'intentions**, pas une description physique. §12

### Politique de boucle
- Résolue AOT par graphe de succession (`next_sequence_id`). §6
- Runtime ignore la nature du bouclage (loop vs transition). §6

### Déduplication et génération
- Déduplication par layer. §9
- Identité d'une `FrameSequence` = contenu ordonné des frames + `next_sequence_id`. §2, §9
- Génération par combinaison `(LayerId, TargetBucket, AnimationAction, Direction, DriverEquipmentId)` — couples valides uniquement. §9
- Pas de combinaison multi-layers matérialisée. §9
- La frame transparente est partagée par `TargetBucket`, mais sa `FrameSequence` reste dans le scope du layer. §9

### Fallback
- Toute `PhysicalRealization` référencée par un contrat est requise par défaut. §4
- `optional: true` porte l'exception au niveau de la `RealizationId`. §4
- Réalisation requise manquante → erreur ; réalisation optionnelle absente → séquence transparente AOT. §9
- Une frame transparente canonique existe par `TargetBucket`. §4, §9
- CompositionProfile source manquante → erreur. §9
- Une séquence transparente conserve la topologie de succession après substitution AOT. §9

### Overlays
- Layer séparé ou variante, selon asset. §9

### Écartés
- `1h_slash`, `1h_backslash`, `1h_halfslash`, `backslash`, `halfslash`, `combat_idle` : écartés. §7
- Écartés par héritage : `backslash_128`, `halfslash_128`. §7
- `_128` / `_192` : suffixes réservés aux désignations descriptives de réalisations physiques ; jamais identités de `Profile`. §5

### Exhaustivité
- Le document n'est pas exhaustif sur le vocabulaire d'animation ; les YAML décrivent le **corpus effectivement déclaré et testé**, pas nécessairement l'ensemble des profils conceptuellement mentionnés ou encore provisoires. §1, §7

### Ordre d'empilement canonique

```
 1. background
 2. body
 3. body_overlays
 4. hair_behind
 5. cape_behind
 6. torso
 7. legs
 8. feet
 9. arms
10. hair_front
11. head
12. facial
13. neck
14. arms_armor
15. equipment_behind
16. equipment
17. shield (position variable selon direction)
18. cape_front
19. equipment_front
20. overlays
```

**Note sur les trois positions d'équipement** : composants visuels distincts d'un même équipement, résolus depuis le **même `DriverEquipmentId`**.

**Exception bouclier** (validé) :

| Direction | Position |
|---|---|
| `down` | Devant |
| `top` | **Derrière** |
| `left` | Devant |
| `right` | Devant |

**Note** : l'exception bouclier est **codée en dur** dans le pipeline AOT.

---

## 12. Sources de vérité

**La seule source de vérité physique ultime est les PNG eux-mêmes.**

| Source | Rôle | Fiabilité |
|---|---|---|
| **PNG** | Vérité des pixels, canvas, `grid_y`, frames | **Autoritaire absolue** |
| **YAML canonique** | Déclaration d'intentions d'animation (`Profile`) | Autoritaire pour nos intentions |
| **Build manifest / asset registry** | Existence, identité et topologie des `PhysicalRealization` | Autoritaire |
| **YAML d'équipement** | Localisation physique des `RealizationId` dans les assets | Autoritaire pour nos intentions |
| **YAML de résolution** | Mapping `AnimationAction` ↔ `Profile` | Autoritaire pour nos intentions |

Règle : `Asset utilisé = déclaré au build ∩ présent sur disque ∩ correctement localisé par le YAML applicable`. Toute divergence = **erreur de build**.

Le **build manifest / asset registry** porte les relations globales qui ne relèvent pas de la localisation physique des PNG :

```text
DriverEquipmentId
    → PhysicalRealization disponibles
    → RealizationBucket disponibles

PhysicalRealization
    → Profile
    → source asset
    → LayerId / topologie
    → RealizationBucket
```

Il porte également la classification `overlay` / `variant`.

**Précision sur la portée PNG vs YAML** :
- Le PNG fixe le **canvas physique et les pixels qu'il contient** (`grid_x`, `grid_y`, `grid_cells`). Une zone transparente reste physiquement présente sans constituer automatiquement une réalisation exploitable.
- Le YAML canonique dit **ce que nous voulons** que les animations soient.
- Le build manifest / asset registry dit **quelles réalisations physiques existent dans le modèle de build**.
- Le YAML d'équipement dit **où chercher physiquement une réalisation donnée** (en `rows` / `grid_y`) ; il ne définit ni le `RealizationBucket` ni le `TargetBucket`.
- Le YAML de résolution ne choisit jamais une réalisation physique.
- Si le YAML de localisation contredit le PNG, **erreur de build**.

### Présence physique vs réalisation exploitable

Une zone de canvas peut être :

```text
physiquement présente
mais entièrement transparente
```

sans constituer une `PhysicalRealization`.

Inversement, une `PhysicalRealization` peut contenir des frames individuellement transparentes ; la transparence locale d'un frame n'est pas en elle-même une absence de réalisation.

La différence entre :

```text
réalisation absente
```

et :

```text
zone transparente du PNG
```

est donc normative. L'absence est déterminée par la déclaration/topologie de build et la disponibilité physique attendue, non par une simple lecture sémantique des pixels.

### Validation croisée

- `RealizationId` manquant → **erreur de build**.
- `Profile` manquant → **erreur de build**.
- localisation manquante pour une réalisation requise → **erreur de build**.
- contrat ambigu → **erreur de build**.
- sélection physique ambiguë → **erreur de build**.
- cible de résolution inexistante → **erreur de build**.
- animation canonique non utilisée → **warning informatif**.

## 13. État des décisions

### Vérifié (empirique)

- ✔ Rows LPC canoniques (0-based).
- ✔ Région Small `grid_y 0–53` commune comme repère/région physique du layout LPC Character observé ; son contenu peut être complet, partiel ou entièrement transparent selon l'asset.
- ✔ Zone des blocs oversize à partir du grid_y 54, dans le layout LPC Character.
- ✔ `1h_backslash` = rows 50–53.
- ✔ La réalisation Medium observée de `walk` sur le Trident contient 9 frames source (héritées du `ExtractionProfile` `walk`).
- ✔ PNG trident : `grid_y 0–53` présent physiquement mais transparent ; réalisation Medium `54–61` ; réalisation Large `62–73`.
- ✔ PNG longsword : réalisation Small `walk` `8–11`, réalisation Small `hurt` `20`, réalisation Large `slash` `54–65`, réalisation Large `slash_reverse` `66–77`, réalisation Large `thrust` `78–89`.
- ✔ Les blocs Longsword `slash`, `slash_reverse` et `thrust` sont contigus ; il n'y a **pas** de padding `grid_y 78–80`.
- ✔ Les assets de bouclier observés restent au format Small du layout historique.
- ✔ `tool_whip` et `tool_axe` dérivent de `slash`.
- ✔ Alignement par centre géométrique, validé visuellement.
- ✔ Ordre d'empilement canonique, validé visuellement.
- ✔ Exception bouclier selon direction, validée visuellement.

### Décidé (design)

- ✔ Quatre unités : `row`, `grid_cell`, `grid`, `grid_y`.
- ✔ `Profile` comme terme générique.
- ✔ `ExtractionProfile` comme terme pour les profils d'extraction directe.
- ✔ `CompositionProfile` comme terme pour les ordonnancements dérivés.
- ✔ **`PhysicalRealization` comme unité physique déclarative distincte du `Profile`.**
- ✔ **`RealizationId` comme identité propre d'une réalisation physique.**
- ✔ **`RealizationBucket` appartient à la PhysicalRealization, jamais au contrat.**
- ✔ Indexation 0-based.
- ✔ Chemins, noms de fichiers et identifiants déclaratifs des données en lowercase ; les noms de concepts, types et structures documentaires ne sont pas concernés.
- ✔ Préfixes interdits pour `AnimationAction`, tolérés pour profils.
- ✔ **`DriverEquipmentId` identifie l'arme ou l'outil actif qui peut piloter le contexte cinématique ; le `TargetBucket` est ensuite dérivé des réalisations effectivement sélectionnées.** Le bouclier est hors de cette dimension.
- ✔ **Les couches non conductrices participent toutes à la composition de l'animation, mais n'influencent ni le `Profile` ni le `TargetBucket` ; dans le corpus visé, leurs réalisations physiques restent Small.**
- ✔ `by_equipment` partout.
- ✔ Pivot d'ancrage gameplay hors-scope AOT.
- ✔ Nomenclature `_128` / `_192` réservée aux désignations descriptives de réalisations physiques ; jamais identités de `Profile`.
- ✔ Écartés : `1h_*`, `backslash`, `halfslash`, + héritage.
- ✔ Trois familles de layers.
- ✔ **`TargetBucket` dérivé du maximum des `RealizationBucket` effectivement sélectionnés dans le contexte courant ; il ne provient pas du catalogue global de l’équipement.**
- ✔ Politique de boucle par graphe de succession.
- ✔ Déduplication par layer.
- ✔ Identité de déduplication d'une `FrameSequence` = contenu ordonné des frames + `next_sequence_id`.
- ✔ Trois YAML distincts.
- ✔ `frame_size` déduit du `RealizationBucket`.
- ✔ PNG source de vérité physique unique.
- ✔ Terme générique « Oversized Equipment ».
- ✔ Contextualisation des contrats : sélection, jamais fusion.
- ✔ `aliases` non utilisé pour `hurt`.
- ✔ Composition dans le bucket cible : pas de scaling.
- ✔ Layout LPC Character qualifié comme spécifique.
- ✔ Origine `x = 0` : par défaut, sans mécanisme de surcharge.
- ✔ Position horizontale des frames : `x(i) = i × frame_size`, frames contiguës sans padding horizontal.
- ✔ Couples `(DriverEquipmentId, TargetBucket)` : valides uniquement.
- ✔ Relation globale équipement : le build manifest / asset registry associe `DriverEquipmentId` aux `PhysicalRealization` et buckets disponibles ; le YAML d'équipement reste scopé à la localisation physique.
- ✔ Résolution `AnimationAction` : au plus un axe de variation parmi `by_equipment` et `by_bucket` ; `default` est un repli ; `by_bucket` dépend d’un `TargetBucket` déjà établi indépendamment ; `fallback` reste réservé.
- ✔ **Déclaration des `AnimationAction` : le YAML de résolution constitue le vocabulaire déclaré ; le nombre de cinématiques d’un asset ne détermine pas le nombre d’actions.**
- ✔ Correspondance `rows` ↔ `directions` : les listes sont parallèles, avec cardinalité identique.
- ✔ Profils mono-directionnels : normalisation AOT sur les quatre directions canoniques par référencement de la même `FrameSequence`.
- ✔ **Optionalité portée par `RealizationId` dans le contrat de localisation.**
- ✔ **Sélection physique déclarative : 0 = absence de sélection, 1 = sélection, >1 = ambiguïté de build ; l’absence physique d’une réalisation est traitée séparément selon l’optionalité du contrat.**
- ✔ **`TargetBucket` n'intervient pas dans l'identité de la réalisation ; il sert de contexte de composition et de validation de compatibilité.**
- ✔ **Les réalisations de `CompositionProfile` possèdent leur propre localisation physique.**
- ✔ **Les contrats de localisation utilisent `realizations[]` et référencent des `RealizationId`.**
- ✔ Classification `overlay` / `variant` au build manifest / asset registry, jamais inférée des pixels.
- ✔ Absence optionnelle : matérialisation AOT d'une `FrameSequence` transparente, avec substitution de topologie AOT.
- ✔ Frame transparente canonique : une par `TargetBucket`.
- ✔ Séquences transparentes : partagent la frame physique du `TargetBucket` mais restent dans le scope de déduplication du layer.

### Notes de veille (non des tickets ouverts)

- **B11 — CompositionProfiles multi-source** : non retenu. À reconsidérer si besoin concret.
- **`swim`** : CompositionProfile dérivé de `spellcast`, marqué **provisoire**.
- **`fallback`** : clé réservée dans le YAML de résolution ; sémantique à définir si elle est activée ultérieurement.
- **B4 — Extensibilité de l'alignement** : la règle `offset = (BucketSize - source_size) / 2` suppose une différence paire. Les valeurs LPC actuelles (`64`, `128`, `192`) satisfont cette propriété.

### Tickets conceptuels

- **Aucun.**

> Les arbitrages conceptuels sont clos dans cette version. Les notes de veille ci-dessus représentent uniquement des vérifications ou extensions éventuelles ; elles ne constituent pas des décisions ouvertes du paradigme.

## 14. Ce que ce document écarte

- Dépendance à un langage ou framework.
- Signatures, types concrets, formats binaires figés.
- Décisions d'outillage.
- Implémentations antérieures obsolètes.
- **Pivot d'ancrage gameplay** — hors-scope.
- **Spécification runtime concrète.**
- **Sémantique runtime du bouclage** — résolue AOT.
- **Terminologie obsolète** `1h_*`, `backslash`, `halfslash`.
- **Préfixes de catégorie sur les `AnimationAction`.**
- **Mécanismes spécialisés de variantes d’action (`AttackVariantId`, `ParryVariantId`, etc.) dans le paradigme central** — écartés : lorsque le gameplay distingue des intentions distinctes, elles sont exprimées par le vocabulaire `AnimationAction`.
- **Exhaustivité sur les animations.**
- **Champ `frame_size` dans les YAML** — déduit du `RealizationBucket`.
- **Champ `align` dans les YAML** — centrage par défaut.
- **CompositionProfiles multi-source** — en veille.
- **Modèle « un Profile = une seule localisation physique »** — écarté.
- **Modèle « un contrat = un seul RealizationBucket »** — écarté : un même contrat peut localiser plusieurs réalisations de buckets différents.
- **Utilisation de `Profile` comme unité de localisation physique** — écartée au profit de `RealizationId`.
- **Utilisation de `reference` comme référence physique générique** — écartée au profit de `realization_id`.
- **Sélection heuristique d'une réalisation physique** — écartée.
- **Fusion de contrats d'équipement** — écartée (sélection, pas fusion).
- **Redimensionnement (scaling) d'un layer** — écarté (composition dans le canvas uniquement).
- **Troisième type de `Profile`** — écarté (les variantes physiques de bucket sont descriptives et ne sont pas des identités de `Profile`).
- **Surcharge de l'origine physique `x = 0`** — écartée.
- **Index global unique `0` pour la frame transparente** — écarté.
- **Sentinel runtime pour représenter une absence de layer** — écarté : l'absence est matérialisée AOT par une `FrameSequence` transparente.
- **Optionalité portée au niveau du seul `ExtractionProfile`** — écartée : elle appartient à la réalisation physique dans le contrat.
- **Partage inter-layer des `FrameSequence` transparentes** — écarté : seule la frame transparente physique est partagée entre layers.

## 15. Synthèse en une phrase

> **Le pipeline fournit au moteur un contrat strict : les PNG LPC fixent le canvas et la vérité physique des pixels ; le build manifest / asset registry définit les `PhysicalRealization` et relie les équipements conducteurs à leurs `RealizationBucket` disponibles ; les `Profile` fixent la sémantique ; le YAML de résolution relie les `AnimationAction` au `Profile` approprié ; les contrats localisent les `RealizationId` ; l’AOT résout le `ResolutionContext → Profile`, puis `Profile → PhysicalRealization`, dérive le `TargetBucket` depuis les réalisations effectivement sélectionnées, valide `RealizationBucket ≤ TargetBucket`, compose au centre du canvas cible et produit les `FrameSequence`.**


## Annexe A — Représentations ASCII de ressources PNG LPC

Cette annexe documente trois **PNG sources physiques** représentatifs du corpus LPC Character. Ils ne représentent **pas** des sorties AOT reconstruites. Ils servent d'étalons physiques pour distinguer le canvas présent, les zones transparentes et les réalisations physiques exploitées. Les symboles décrivent l’occupation graphique utile de la grille ; une zone transparente peut donc être physiquement présente dans le PNG sans contenir de contenu exploitable.

### Légende

| Symbole | Signification |
|---|---|
| `○` | `grid_cell` appartenant à une frame **Small** (64×64) |
| `◎` | `grid_cell` appartenant à une frame **Medium** (128×128) |
| `●` | `grid_cell` appartenant à une frame **Large** (192×192) |
| `.` | `grid_cell` transparente / sans contenu graphique exploitable |

**Conventions de lecture** : chaque ligne représente une `grid_y` ; chaque colonne une `grid_x`. Une frame Small consomme 1 `grid_y` par direction, une frame Medium 2, une frame Large 3. Les frames sont contiguës horizontalement, sans padding, avec origine `x = 0`.

### A.1 — PNG de base historique

```text
       0         1         2
       012345678901234567890123
  0    ○○○○○○○.................  spellcast (top)
  1    ○○○○○○○.................  spellcast (left)
  2    ○○○○○○○.................  spellcast (down)
  3    ○○○○○○○.................  spellcast (right)
  4    ○○○○○○○○................  thrust (top)
  5    ○○○○○○○○................  thrust (left)
  6    ○○○○○○○○................  thrust (down)
  7    ○○○○○○○○................  thrust (right)
  8    ○○○○○○○○○...............  walk (top)
  9    ○○○○○○○○○...............  walk (left)
 10    ○○○○○○○○○...............  walk (down)
 11    ○○○○○○○○○...............  walk (right)
 12    ○○○○○○..................  slash (top)
 13    ○○○○○○..................  slash (left)
 14    ○○○○○○..................  slash (down)
 15    ○○○○○○..................  slash (right)
 16    ○○○○○○○○○○○○○...........  shoot (top)
 17    ○○○○○○○○○○○○○...........  shoot (left)
 18    ○○○○○○○○○○○○○...........  shoot (down)
 19    ○○○○○○○○○○○○○...........  shoot (right)
 20    ○○○○○○..................  hurt (down)
 21    ○○○○○○..................  climb (top)
 22    ○○......................  idle (top)
 23    ○○......................  idle (left)
 24    ○○......................  idle (down)
 25    ○○......................  idle (right)
 26    ○○○○○...................  jump (top)
 27    ○○○○○...................  jump (left)
 28    ○○○○○...................  jump (down)
 29    ○○○○○...................  jump (right)
 30    ○○○.....................  sit (top)
 31    ○○○.....................  sit (left)
 32    ○○○.....................  sit (down)
 33    ○○○.....................  sit (right)
 34    ○○○.....................  emote (top)
 35    ○○○.....................  emote (left)
 36    ○○○.....................  emote (down)
 37    ○○○.....................  emote (right)
 38    ○○○○○○○○................  run (top)
 39    ○○○○○○○○................  run (left)
 40    ○○○○○○○○................  run (down)
 41    ○○○○○○○○................  run (right)
 42    ○○○○....................  combat_idle (top)     [écarté]
 43    ○○○○....................  combat_idle (left)    [écarté]
 44    ○○○○....................  combat_idle (down)    [écarté]
 45    ○○○○....................  combat_idle (right)   [écarté]
 46    ○○○○....................  1h_slash (top)        [hors-scope]
 47    ○○○○....................  1h_slash (left)       [hors-scope]
 48    ○○○○....................  1h_slash (down)       [hors-scope]
 49    ○○○○....................  1h_slash (right)      [hors-scope]
 50    ○○○○....................  1h_backslash (top)    [hors-scope]
 51    ○○○○....................  1h_backslash (left)   [hors-scope]
 52    ○○○○....................  1h_backslash (down)   [hors-scope]
 53    ○○○○....................  1h_backslash (right)  [hors-scope]
```

### A.2 — PNG trident

**`grid_y 0–53` est physiquement présent dans le PNG mais entièrement transparent dans cet asset.**

```text
       0         1         2
       012345678901234567890123
  0    ........................  [transparent]
  1    ........................  [transparent]
  ...  ........................  [transparent]
 53    ........................  [transparent]
       ─────── fin de la région Small historique (contenu transparent) ───────
 54    ◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎......  walk_128 (top, 1/2)
 55    ◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎......  walk_128 (top, 2/2)
 56    ◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎......  walk_128 (left, 1/2)
 57    ◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎......  walk_128 (left, 2/2)
 58    ◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎......  walk_128 (down, 1/2)
 59    ◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎......  walk_128 (down, 2/2)
 60    ◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎......  walk_128 (right, 1/2)
 61    ◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎◎......  walk_128 (right, 2/2)
       ─────── fin du bloc Medium ───────
 62    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (top, 1/3)
 63    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (top, 2/3)
 64    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (top, 3/3)
 65    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (left, 1/3)
 66    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (left, 2/3)
 67    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (left, 3/3)
 68    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (down, 1/3)
 69    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (down, 2/3)
 70    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (down, 3/3)
 71    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (right, 1/3)
 72    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (right, 2/3)
 73    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (right, 3/3)
```

### A.3 — PNG longsword

**Les annotations `[transparent]` signifient « aucun contenu graphique exploitable dans cette zone » ; les `grid_y` appartiennent néanmoins au canvas physique du PNG.**

```text
       0         1         2
       012345678901234567890123
  0    ........................  [transparent]
  1    ........................  [transparent]
  2    ........................  [transparent]
  3    ........................  [transparent]
  4    ........................  [transparent]
  5    ........................  [transparent]
  6    ........................  [transparent]
  7    ........................  [transparent]
  8    ○○○○○○○○○...............  walk (top)
  9    ○○○○○○○○○...............  walk (left)
 10    ○○○○○○○○○...............  walk (down)
 11    ○○○○○○○○○...............  walk (right)
 12    ........................  [transparent]
 13    ........................  [transparent]
 14    ........................  [transparent]
 15    ........................  [transparent]
 16    ........................  [transparent]
 17    ........................  [transparent]
 18    ........................  [transparent]
 19    ........................  [transparent]
 20    ○○○○○○..................  hurt (down)
 21    ........................  [transparent]
 ...    ........................  [zone transparente — grid_y 21 à 53]
 53    ........................  [transparent]
       ─────── fin de la région Small historique (grid_y 0–53) ───────
 54    ●●●●●●●●●●●●●●●●●●......  slash_192 (top, 1/3)
 55    ●●●●●●●●●●●●●●●●●●......  slash_192 (top, 2/3)
 56    ●●●●●●●●●●●●●●●●●●......  slash_192 (top, 3/3)
 57    ●●●●●●●●●●●●●●●●●●......  slash_192 (left, 1/3)
 58    ●●●●●●●●●●●●●●●●●●......  slash_192 (left, 2/3)
 59    ●●●●●●●●●●●●●●●●●●......  slash_192 (left, 3/3)
 60    ●●●●●●●●●●●●●●●●●●......  slash_192 (down, 1/3)
 61    ●●●●●●●●●●●●●●●●●●......  slash_192 (down, 2/3)
 62    ●●●●●●●●●●●●●●●●●●......  slash_192 (down, 3/3)
 63    ●●●●●●●●●●●●●●●●●●......  slash_192 (right, 1/3)
 64    ●●●●●●●●●●●●●●●●●●......  slash_192 (right, 2/3)
 65    ●●●●●●●●●●●●●●●●●●......  slash_192 (right, 3/3)
 66    ●●●●●●●●●●●●●●●●●●......  slash_reverse_192 (top, 1/3)
 67    ●●●●●●●●●●●●●●●●●●......  slash_reverse_192 (top, 2/3)
 68    ●●●●●●●●●●●●●●●●●●......  slash_reverse_192 (top, 3/3)
 69    ●●●●●●●●●●●●●●●●●●......  slash_reverse_192 (left, 1/3)
 70    ●●●●●●●●●●●●●●●●●●......  slash_reverse_192 (left, 2/3)
 71    ●●●●●●●●●●●●●●●●●●......  slash_reverse_192 (left, 3/3)
 72    ●●●●●●●●●●●●●●●●●●......  slash_reverse_192 (down, 1/3)
 73    ●●●●●●●●●●●●●●●●●●......  slash_reverse_192 (down, 2/3)
 74    ●●●●●●●●●●●●●●●●●●......  slash_reverse_192 (down, 3/3)
 75    ●●●●●●●●●●●●●●●●●●......  slash_reverse_192 (right, 1/3)
 76    ●●●●●●●●●●●●●●●●●●......  slash_reverse_192 (right, 2/3)
 77    ●●●●●●●●●●●●●●●●●●......  slash_reverse_192 (right, 3/3)
       ─────── fin de la réalisation slash_reverse_192 confirmée (66–77) ───────
 78    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (top, 1/3)
 79    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (top, 2/3)
 80    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (top, 3/3)
 81    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (left, 1/3)
 82    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (left, 2/3)
 83    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (left, 3/3)
 84    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (down, 1/3)
 85    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (down, 2/3)
 86    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (down, 3/3)
 87    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (right, 1/3)
 88    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (right, 2/3)
 89    ●●●●●●●●●●●●●●●●●●●●●●●●  thrust_192 (right, 3/3)
```

### A.4 — Lecture croisée et invariants physiques illustrés

Les trois PNG ne constituent pas une définition exhaustive du corpus. Ils illustrent néanmoins les invariants et distinctions désormais retenus :

1. **Le canvas physique d’un PNG peut contenir des zones entièrement transparentes ; présence physique et contenu exploitable sont distincts.**
2. **La région Small historique `0–53` est une région physique du layout LPC Character ; son contenu dépend de l’asset.**
3. **Les réalisations oversized observées commencent à `54` dans ce layout.**
4. **Un même PNG peut contenir plusieurs réalisations physiques et plusieurs buckets.**
5. **L’absence de contenu dans une zone du PNG n’est pas une absence du canvas ; l’exploitation de cette zone relève du contrat AOT.**
6. **Le contexte cinématique peut être commun à plusieurs layers alors que leurs réalisations physiques diffèrent.**
7. **Pour `attack + longsword`, le contexte sémantique est `slash` et le `TargetBucket` est Large ; `slash_192` réalise le layer d’équipement, tandis que la réalisation `slash` Small fournit les layers de personnage, composée dans le canvas Large.**
8. **Le même `source_extraction` peut alimenter des réalisations physiques distinctes et des `FrameSequence` distinctes.**

## Annexe B — Scénarios de validation S1 à S4

Cette annexe transforme les scénarios concrets utilisés pendant les arbitrages en **tests de cohérence du pipeline AOT**. Ils ne constituent pas une spécification YAML complète ; ils vérifient que le paradigme sait absorber les contextes représentatifs sans introduire de résolution runtime.

### B.1 — S1 : `walk` avec `longsword`

```text
AnimationAction   = walk
DriverEquipmentId = longsword
Profile           = walk

Longsword / walk  → RealizationBucket = Small
Autres layers     → réalisations Small

TargetBucket      = Small
Target canvas      = 64×64
```

Tous les layers constitutifs participent à la composition. Les réalisations Large du même équipement, appartenant à d'autres actions, ne provoquent aucune promotion de `walk`.

Le Longsword est physiquement localisé sur `grid_y 8, 9, 10, 11`.

**Verdict paradigme : absorbé.**

### B.2 — S2 : première attaque `longsword` → `slash`

```text
AnimationAction   = attack
DriverEquipmentId = longsword
Profile           = slash
```

La résolution `attack → slash` existe déjà dans le corpus représentatif. La réalisation d'équipement est `longsword-slash`, de bucket Large, localisée sur `grid_y 54–65`. Les layers non conducteurs restent Small et sont composés au centre du canvas Large.

```text
TargetBucket      = Large
Target canvas      = 192×192

Small source      → offset (64, 64)
Large longsword   → offset (0, 0)
```

**Verdict paradigme : absorbé.**

### B.3 — S3 : autre action d'attaque `longsword` → `thrust`

```text
AnimationAction   = <action d'attaque déclarée par le gameplay>
DriverEquipmentId = longsword
Profile           = thrust
```

Aucun identifiant spécialisé de variante n'est requis par le paradigme. Le gameplay choisit le nom sémantique de l'action ; le YAML de résolution la relie à `thrust`.

La réalisation `longsword-thrust` est Large et occupe sans padding :

```text
grid_y 78–89

rows de départ : [78, 81, 84, 87]
```

Le bloc est immédiatement contigu au bloc `slash_reverse` (`66–77`).

```text
TargetBucket      = Large
Target canvas      = 192×192
```

**Verdict paradigme : absorbé.**

### B.4 — S4 : `hurt` avec `longsword` toujours équipé

```text
AnimationAction   = hurt
DriverEquipmentId = longsword
Profile           = hurt
```

Toutes les couches constitutives du personnage participent à la composition. Les couches non conductrices restent Small dans le corpus LPC visé ; le Longsword possède lui aussi une réalisation `hurt` Small.

```text
body / hurt       → Small
head / hurt       → Small
hair / hurt       → Small
clothes / hurt    → Small
boots / hurt      → Small
shield / hurt     → Small
longsword / hurt  → Small
```

La réalisation du Longsword est localisée sur `grid_y 20`. Aucune des réalisations Large d'attaque du Longsword n'est pertinente pour ce contexte.

**Verdict paradigme : absorbé.**

**Remarque corpus** : la fixture représentative actuelle ne matérialise pas encore toutes les couches du personnage complet ; cela relève de la couverture YAML, pas d'une restriction du paradigme.

### B.5 — Synthèse

| Scénario | Profil résolu | RealizationBucket conducteur | TargetBucket | Canvas | Paradigme |
|---|---|---:|---:|---:|---|
| S1 — `walk + longsword` | `walk` | Small | Small | 64×64 | **Absorbé** |
| S2 — première action d'attaque `longsword` | `slash` | Large | Large | 192×192 | **Absorbé** |
| S3 — autre action d'attaque `longsword` | `thrust` | Large | Large | 192×192 | **Absorbé** |
| S4 — `hurt + longsword` | `hurt` | Small | Small | 64×64 | **Absorbé** |

Ces quatre scénarios couvrent les propriétés structurantes recherchées :

```text
1. équipement conducteur sans promotion globale ;
2. composition Small → Large ;
3. plusieurs cinématiques accessibles par le vocabulaire `AnimationAction` sans mécanisme spécialisé de variante ;
4. retour à une animation Small malgré la conservation de l’équipement conducteur ;
5. participation de toutes les couches sans confusion entre présence visuelle et rôle conducteur.
```

Ils constituent une base appropriée pour une annexe de non-régression et pour la prochaine phase de matérialisation YAML.

_Document révisé le 24 septembre 2026 — v1.3.5 (socle normatif v1.3)_
