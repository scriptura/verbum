# Document fondateur conceptuel — LPC (pipeline AOT) — v23

> **Statut** : brouillon / schéma directeur — après première passe de cohérence ; points d’arbitrage restants explicitement signalés.
> **Portée** : ce document couvre le **pipeline AOT**. Le runtime n'est mentionné que pour fixer les **invariants de frontière**.
> **Exhaustivité** : ce document **n'est pas exhaustif** sur les animations. Il pose le **principe**, des **exemples illustratifs** et des **invariants**. La liste complète relève des **YAML**.

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
- Dans la **grid**, un row logique (une direction) consomme **1, 2 ou 3 `grid_y` successifs**, selon la `frame_size` : 64 → 1, 128 → 2, 192 → 3.

**Règle stricte d'usage** :
- **« row »** désigne toujours l'unité **logique** (une direction).
- **« grid_y »** désigne toujours la coordonnée **physique** d'extraction dans la grid.
- Ces deux termes ne sont **jamais interchangeables**.

**Important** : les YAML déclarent toujours en **rows logiques** (une par direction). Dans un contrat d'équipement, chaque élément de `rows` matérialise le `grid_y` de départ correspondant à cette row ; l'expansion verticale requise par le `frame_size` est absorbée par le pipeline AOT.

### Convention de vocabulaire — préfixes

- **`AnimationAction`** : **jamais de préfixe de catégorie**.
- **Profils (`ExtractionProfile`, `CompositionProfile`)** : **préfixe descriptif toléré** (ex. `tool_whip`).
- Aucun préfixe n'est **jamais** porté par une `AnimationAction`.

### Convention de vocabulaire — chemins et identifiants

**Chemins, noms de fichiers et identifiants sont en lowercase.**

---

## 2. Architecture à trois étages

```
[ÉTAGE 1]   AnimationAction      ← vocabulaire d’animation fourni au runtime
                │
                │  résolution AOT (équipement, bucket, contexte)
                ▼
[ÉTAGE 2]   Profile              ← ce que les PNG fournissent / composent
            ├── ExtractionProfile        (extraction)
            └── CompositionProfile   (composition)
                │
                │  extraction + composition AOT
                ▼
[ÉTAGE 3]   FrameSequence        ← ce que le runtime consomme
```

### Étage 1 — AnimationAction

C'est le langage du gameplay. C'est ce que les systèmes écrivent, et la **seule notion sémantique d'animation qu'ils fournissent au runtime**.

Exemples (liste **illustrative**, non exhaustive, **sans préfixes**) :
`attack`, `cast`, `walk`, `run`, `swim`, `idle`, `die`, `stagger`, `watering`, `sit`, `emote`…

- Indépendant de LPC, de l'équipement, de la taille des sprites.
- Ne dit **rien** sur la cinématique ni sur les pixels.

### Étage 2 — Profile, ExtractionProfile, CompositionProfile

**`Profile`** est le terme **générique** englobant les deux sous-types :

```
Profile
├── ExtractionProfile
└── CompositionProfile
```

- **`ExtractionProfile`** — profil d'**extraction**. Décrit ce que les PNG fournissent : `frame_count`, `directions`, et une séquence canonique par défaut (`sequence`, optionnelle).
- **`CompositionProfile`** — profil de **composition ordinale**. Décrit comment les frames extraites sont ordonnées en séquence. **Hérite `frame_count` et `directions` de son `source_extraction`**.

**Précision sur les variantes physiques de bucket** :

- `walk_128`, `walk_192`, `slash_128`, `slash_192`, `thrust_192` désignent des **variantes physiques de bucket** d'un `ExtractionProfile` existant.
- Cette expression est **purement descriptive** : elle ne constitue **pas** un troisième type de `Profile`.
- Conceptuellement, `walk_128` **est** `walk` composé dans un canvas Medium. Ce n'est ni une nouvelle intention d'animation, ni un nouveau `ExtractionProfile`.
- Le suffixe `_128` / `_192` décrit la variante physique ; le `ExtractionProfile` sous-jacent reste `walk`.
- Le pipeline AOT résout `(ExtractionProfile, bucket applicable)` → variante physique. Aucun `ExtractionProfile::Walk128` n'est créé.

**Précision sur `slash_reverse_192`** :

- `slash_reverse_192` n'est **pas** un `ExtractionProfile` autonome.
- Si `slash_reverse` correspond bien à une inversion de `slash`, la chaîne conceptuelle est :

```
slash (ExtractionProfile)
    │
    ▼
slash_reverse (CompositionProfile, source_extraction: slash, sequence: [5,4,3,2,1,0])
    │
    ▼
variante physique Large : slash_reverse_192
```

- La variante physique `slash_reverse_192` hérite de la localisation physique de `slash` au bucket Large.
- **Cette chaîne reste à vérifier contre les YAML réels avant d'être tenue pour acquise.** Si les YAML existants traitent `slash_reverse_192` différemment, la modélisation devra être ajustée.

**Précision sur `rows`** :
- `rows` **n'est pas une propriété intrinsèque** d'un `ExtractionProfile`.
- `rows` est fourni par le **contrat d'équipement**, qui localise le `ExtractionProfile` dans un PNG donné.
- Un même `ExtractionProfile` peut être localisé à des `rows` différentes selon le contrat et le bucket.

**Distinction critique — `frame_count` source vs longueur de séquence jouée** :

- `frame_count` désigne la taille du **pool extrait**.
- `len(sequence)` désigne le nombre de frames **effectivement jouées**.
- Ces deux valeurs peuvent être différentes : `watering` a `frame_count = 8` (pool) et `len(sequence) = 7` (jouées).

**Exemples** :

| Nom | Type | Source | Remarque |
|---|---|---|---|
| `slash`, `thrust`, `shoot`, `spellcast`, `walk`, `run`, `jump`, `climb`, `idle`, `sit`, `hurt`, `emote`… | ExtractionProfile | — | Extraction directe |
| `watering` | CompositionProfile | `thrust` | Séquence `[0,1,4,4,4,4,5]` |
| `tool_whip` | CompositionProfile | `slash` | Séquence `[0,1,2,3,4,5]` |
| `tool_axe` | CompositionProfile | `slash` | Séquence `[5,5,4,4,3,1,0,0,0,0]` |
| `slash_reverse` | CompositionProfile | `slash` | Séquence `[5,4,3,2,1,0]` |
| `swim` | CompositionProfile | `spellcast` | Provisoire (voir §13) |

### Étage 3 — FrameSequence

Séquence linéaire, **immuable dans son contenu** (ordre des frames figé), produite exclusivement AOT.

- Ne contient **aucun pixel**.
- Référence des frames stockées dans des structures globales.
- Le runtime n'itère que sur des **indices et offsets précompilés**.
- **Chaque séquence porte un `next_sequence_id`** (propriété adjacente) qui indique la séquence à jouer une fois le curseur épuisé.

**Distinction `sequence` vs `FrameSequence`** :
- `sequence` = **déclaration YAML**, tableau d'indices locaux (source).
- `FrameSequence` = **artefact AOT final**, séquence d'offsets globaux consommable par le runtime.

---

## 3. Rôle du bucket (paramètre orthogonal)

| Bucket | Résolution |
|---|---|
| Small | 64×64 |
| Medium | 128×128 |
| Large | 192×192 |

- Bucket **orthogonal** au ExtractionProfile.
- Toutes les frames d'un bucket ont même taille, même **centre géométrique d'alignement AOT**.
- Grid_cells hétérogènes **ajoutées en fin de PNG**, par ordre croissant.
- **Armes et outils uniquement** : les PNG > 64×64 sont réservés aux **variantes oversized d'équipement**.
- Corps, cheveux, vêtements, armures, bouclier : **toujours** 64×64.
- Les layers non-équipement sont **composés dans le canvas du bucket cible** imposé par l'équipement. **Il ne s'agit pas d'un redimensionnement** : la résolution intrinsèque du layer n'est pas modifiée. Un corps 64×64 reste 64×64, mais il est **positionné** dans un canvas 128×128 (ou 192×192) selon la règle de centrage.

**Exemple de composition (sans scaling)** :

```
body : 64×64
bucket cible : 128×128

offset de centrage :
(128 - 64) / 2 = 32

→ body placé à (+32, +32) dans le canvas 128×128
```

Le body n'est **pas redimensionné**. Il est composé dans un repère plus grand.

### Trois familles de layers

| Famille | Exemples | Buckets possibles | Rôle |
|---|---|---|---|
| **Corporel** | corps, cheveux, vêtements, armures, bouclier | Small uniquement | Composé dans le canvas cible |
| **Arme** | épée, arc, bâton… | Small / Medium / Large | Détermine le bucket cible |
| **Outil** | pioche, hache, fouet… | Small / Medium / Large | Détermine le bucket cible, comme une arme |

**Note sur le bouclier** : classé **corporel** (jamais pilote du bucket), mais sa **position d'empilement varie selon la direction** (§11).

### Promotion automatique de bucket

**Le bucket cible est déterminé par le maximum des buckets disponibles pour l'équipement actif.**

- Sans équipement : **Small par défaut**.
- Si le bucket de base est Small mais que l'équipement n'a qu'une variante Large : **promotion automatique vers Large**.
- **La promotion s'applique identiquement aux armes et aux outils.**

**Précision sur la nature de la promotion** : la promotion est une **déduction** (`bucket_cible = max(buckets_disponibles_de_l'équipement)`). Elle n'entraîne **pas** de transformation effective des assets : les layers non-équipement sont composés dans le bucket cible via le mécanisme standard de composition (§9).

### Alignement des frames hétérogènes

**Toute frame source, quelle que soit sa taille, est centrée sur un point d'ancrage commun avant composition.**

- Le **centre géométrique** de la frame source doit coïncider avec le **centre géométrique** de la frame cible.
- Convention **invariante**, non négociable, non configurable.
- **Indépendante** du pivot d'ancrage gameplay (hors-scope AOT).
- **Décision de design.** Validation visuelle effectuée.

### Structure en grid des PNG

**Un PNG LPC est une grid de grid_cells de 64×64.**

- Une **grid_cell** = un carré de 64×64 pixels.
- Dans la grid, un **row logique** (une direction) consomme 1, 2 ou 3 **`grid_y` successifs**, selon la `frame_size`.

**Un PNG étendu peut contenir plusieurs blocs de tailles différentes empilés verticalement dans la grid.**

Exemple concret (trident observé) :

```
Bloc Small      (grid_y 0–53)
Bloc Medium     (grid_y 54–61, walk_128)
Bloc Large      (grid_y 62–73, thrust_192)
```

**Structure monotone des PNG étendus** :

- **Dans le layout LPC Character étudié ici**, le bloc Small (`grid_y` 0–53) est strictement identique d'un PNG à l'autre. C'est l'**héritage LPC commun**.
- Cette uniformité est une propriété de **ce layout particulier**, pas une invariante générale du pipeline AOT. D'autres familles d'assets (foliage, water, fire, etc.) pourraient avoir des layouts physiques entièrement différents.
- Les `grid_y` sont **monotones de 0 à 53** pour le bloc Small.
- **La zone des blocs oversize commence au `grid_y` 54** (dans ce layout).
- **Sur un même PNG, les variantes de taille d'un même profil partagent les mêmes `grid_y`.**

**Structure physique du layout LPC Character** :

```
grid_y 0 ──────────────── 53 │ 54 ────────────────>
         Bloc Small          │    Blocs oversize
         64×64 (héritage)    │    Medium / Large
```

**Les blocs sont contigus, sans padding.** Le YAML est la seule source qui identifie où commence et où finit chaque bloc dans la grid.

**Précision importante** : les rows logiques (0–53) couvrent **l'ensemble du bloc Small**, mais chaque animation n'occupe qu'une plage restreinte de rows dans ce bloc. La frontière `0–53` est **structurelle** ; les frontières entre animations sont **sémantiques**.

### Origine physique par défaut

**L'origine physique par défaut des frames est `x = 0`.** Le pipeline AOT extrait les frames à partir de cette origine. Aucun mécanisme de surcharge d'origine n'est prévu à ce stade.

### Extensibilité à d'autres gabarits

- **Buckets** configurables.
- **Grid_cell de base** configurable par corpus (`64` pour LPC humanoïde).
- **Frame sizes** : tout multiple entier de la grid_cell de base.
- **ExtractionProfiles** extensibles.
- **Layouts physiques** : chaque famille d'assets peut définir son propre layout physique. Le layout LPC Character décrit ici n'est qu'un cas particulier.
- **Aucune convention implicite** : chaque YAML déclare explicitement ses valeurs.

**Cas particulier — créatures à animation unique** : 1 direction, 1 frame, sans séquence custom.

---

## 4. Extraction, Composition, et propriétés AOT

### Extraction

**Ce qu'on lit** dans le PNG :

- quelles **rows** logiques LPC (déclarées par le contrat d'équipement),
- quel **nombre de frames source** (`frame_count`, déclaré par le YAML canonique),
- quel **bucket cible** (déclaré par le contrat d'équipement).

### Composition ordinale

**Ce qu'on joue** à partir de ce pool :

- une **séquence ordonnée** d'indices,
- qui peut **répéter**, **sauter**, **réordonner**, ou toute **combinaison**.

### Trois types de YAML

**1. YAML canonique d'animations** — un seul, liste **toutes** les animations du corpus. Il représente la **déclaration d'intentions d'animation**.

Chaque entrée porte :
- `name`, `type` (`ExtractionProfile` par défaut, ou `CompositionProfile`),
- `frame_count`, `directions`,
- `sequence` (optionnel),
- `source_extraction` (CompositionProfile uniquement),
- `aliases` (optionnel — liste de synonymes historiques).

**2. YAML d'équipement** — localise les `ExtractionProfile` dans un PNG donné.

Chaque entrée porte :
- `contract_id`, `applies_to` (`path_prefix` + `asset_whitelist` optionnelle),
- `target_bucket`,
- `extraction_profiles` (dictionnaire `reference` + `rows`).

> **Note sur `extraction_profiles`** : la clé reste `extraction_profiles` car le contrat d'équipement ne référence **que des sources physiques**. Un `CompositionProfile` n'a pas de localisation physique propre — il dérive d'un `ExtractionProfile` localisé ailleurs.

**3. YAML de résolution `AnimationAction`** (ex. `action_resolution.yaml`) — **contrat de liaison** entre le domaine Gameplay et le domaine Moteur.

Chaque entrée porte :
- `animation_action` — l'action déclenchée par le gameplay,
- `resolution` — table de résolution vers les profils (voir §8).

**Répartition des responsabilités :**

| Champ | Canonique | Équipement | Résolution |
|---|---|---|---|
| `name`, `type` | ✔ | — | — |
| `frame_count`, `directions` | ✔ (ExtractionProfile) | — | — |
| `sequence` | ✔ | — | — |
| `source_extraction` | ✔ (CompositionProfiles) | — | — |
| `aliases` | ✔ | — | — |
| `reference`, `rows` | — | ✔ | — |
| `target_bucket` | — | ✔ | — |
| `animation_action` | — | — | ✔ |
| `resolution` | — | — | ✔ |

### `frame_size` déduit du bucket

**`frame_size` n'est jamais déclaré** dans les YAML. Il est déduit de `target_bucket` au build : `Small` → 64, `Medium` → 128, `Large` → 192.

Pour un contrat d'équipement, chaque élément de `rows` représente le **`grid_y` de départ** d'une direction logique. Les `grid_y` physiques supplémentaires occupés par cette direction sont déduits de `frame_size / grid_cell` : 1 pour Small, 2 pour Medium, 3 pour Large. Ainsi, `len(rows) == len(directions)` reste vrai quel que soit le bucket.

### Héritage des CompositionProfiles

Un **CompositionProfile** hérite `frame_count` et `directions` de son `source_extraction`.

Exemple simplifié :

```yaml
- name: watering
  type: CompositionProfile
  source_extraction: thrust
  sequence: [0, 1, 4, 4, 4, 4, 5]
```

### Invariant de contextualisation des contrats

**Le pipeline AOT ne fusionne jamais les contrats d'équipement.** Il **sélectionne** le contrat applicable au contexte, puis résout la référence dans ce contrat.

Formellement :

```
(Profile, applicable Contract) → extraction coordinates
```

Le contrat est lui-même sélectionné par :

```
(applies_to, target_bucket) → applicable Contract
```

**Conséquences** :

- Un même `ExtractionProfile` (ex. `slash`) peut être déclaré dans **plusieurs contrats**, avec des `rows` propres à chaque bucket.
- L'AOT ne produit **jamais** de contrat fusionné.
- La chaîne de résolution est :

```
AnimationAction + EquipmentId + BucketId
        ↓
   Profile (ExtractionProfile ou CompositionProfile)
        ↓
   source_extraction éventuel
        ↓
   ExtractionProfile
        ↓
   contrat applicable (applies_to, target_bucket)
        ↓
   rows physiques
        ↓
   frames
```

**Distinction clé — `CompositionProfile` n'est jamais localisé physiquement.** Un `CompositionProfile` est une **identité de composition/résolution**. La localisation physique est toujours portée par son `ExtractionProfile` source.

**Invariant d'unicité de contrat** :

| Nombre de contrats applicables | Traitement |
|---|---|
| 0 | **Erreur de build** |
| 1 | Résolution normale |
| >1 | **Erreur de build par ambiguïté** |

Cette règle évite toute fusion implicite ou priorité cachée entre contrats.

### Invariant des couples `(EquipmentId, BucketId)` valides

**Le pipeline AOT ne matérialise que les couples `(EquipmentId, BucketId)` effectivement résolus par la logique d'équipement.**

- `EquipmentId = None` → `BucketId = Small`.
- `EquipmentId = X` → `BucketId = max(buckets_disponibles(X))`.
- Le produit cartésien `tous les EquipmentId × tous les BucketId` **n'est jamais généré**.
- Les couples invalides sont **absents** de l'espace généré.

Le runtime utilise `EquipmentId` et `BucketId` comme **identifiants d'indexation**, sans en interpréter la sémantique. Aucun branchement conditionnel du type `if equipment == X` n'est jamais effectué.

### Contrat YAML d'équipement — exemple idiomatique

```yaml
contract_id: "lpc_base_64"

applies_to:
  path_prefix: "textures/characters/"

target_bucket: "Small"

extraction_profiles:
  spellcast:
    reference: spellcast
    rows: [0, 1, 2, 3]
  thrust:
    reference: thrust
    rows: [4, 5, 6, 7]
  walk:
    reference: walk
    rows: [8, 9, 10, 11]
  slash:
    reference: slash
    rows: [12, 13, 14, 15]
  hurt:
    reference: hurt
    rows: [20]
```

### YAML de résolution — exemple idiomatique

```yaml
# action_resolution.yaml

actions:

  - animation_action: attack
    resolution:
      default: slash
      by_equipment:
        spear: thrust
        crossbow: thrust
        bow: shoot
        longsword: slash

  - animation_action: walk
    resolution:
      by_bucket:
        Small: walk
        Medium: walk_128
        Large: walk_192

  - animation_action: watering
    resolution:
      default: watering
```

**Note** : les entrées `walk_128` / `walk_192` de la résolution sont **descriptives** — elles désignent la variante physique de `walk` au bucket correspondant. Conceptuellement, `walk_128` **est** `walk` composé dans un canvas Medium.

**Note sur `fallback`** : la clé `fallback` du YAML de résolution est **réservée** pour une politique de repli explicite. Sa sémantique exacte relève de la Phase 5. En son absence, le comportement par défaut est : `default` si présent, sinon **erreur de build**.

### Indices locaux vs offsets globaux

**Les indices de `sequence` sont locaux** au pool extrait. Le pipeline AOT les remappe en **offsets globaux**. L'**index global `0`** est réservé à la **frame transparente canonique**.

### Cohérence `rows` ↔ `directions`

```
len(rows) == len(directions)
```

Sinon → **erreur de build**.

### Validation des références croisées

**Règle 1 — référence manquante dans un contrat d'équipement → erreur de build.**

**Règle 2 — cible de résolution référencée mais inexistante dans le YAML canonique → erreur de build.**

**Règle 3 — animation canonique non référencée → warning informatif.**

**Règle 4 — `AnimationAction` déclarée mais non référencée par une entrée de résolution → warning informatif.**

**Règle 5 — ambiguïté de contrat (plusieurs contrats applicables au même asset et bucket) → erreur de build.**

### Cas notable : partage de rows entre profils

`tool_whip` et `tool_axe` dérivent de `slash`. **Sur un PNG donné, les variantes de taille d'un même profil partagent leurs `grid_y`.**

---

## 5. Vocabulaire LPC vs vocabulaire interne

### Décisions

- **Nomenclature interne uniformisée** : suffixes `_128`, `_192`. Lowercase partout (y compris chemins).
- **Terminologie « Oversized Equipment »** : animations dont la frame source dépasse 64×64, pour armes et outils.

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
- Tenir une pose = **répéter la frame** dans la séquence.

### Politique de boucle — résolue AOT

**Graphe de succession.**

- `séquence A → séquence A` pour un **loop**.
- `séquence A → séquence B` pour une **transition**.

Chaque `FrameSequence` porte un `next_sequence_id`.

**Politique de bout** : une séquence terminale pointe vers un **sentinel** (`next_sequence_id = 0` ou valeur réservée). Le runtime maintient alors le curseur sur la dernière frame.

**Conséquence runtime** : zéro allocation, topologie dictée par AOT, aucune connaissance sémantique de la boucle.

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

**Normalisation AOT des profils mono-directionnels** : lorsqu'un `ExtractionProfile` ne déclare qu'une seule direction (`hurt` ou `climb`), le pipeline AOT **duplique la séquence résolue sur les quatre directions canoniques**. Le runtime conserve ainsi un domaine de `Direction` uniforme ; aucune règle de remapping directionnel n'est nécessaire au runtime.


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

| CompositionProfile | source_extraction | sequence |
|---|---|---|
| `watering` | `thrust` | `[0,1,4,4,4,4,5]` |
| `tool_whip` | `slash` | `[0,1,2,3,4,5]` |
| `tool_axe` | `slash` | `[5,5,4,4,3,1,0,0,0,0]` |
| `slash_reverse` | `slash` | `[5,4,3,2,1,0]` |
| `tool_hoe` | `slash` | à préciser (YAML) |
| `tool_shovel` | `slash` | à préciser (YAML) |
| `swim` | `spellcast` | à préciser (provisoire, voir §13) |

**Note sur `slash_reverse`** : modélisé comme `CompositionProfile` dérivé de `slash`, sa variante physique au bucket Large est notée `slash_reverse_192`. Cette chaîne reste à vérifier contre les YAML réels.

### Variantes physiques de bucket (descriptives)

> **Rappel** : ces entrées ne sont **pas** des `ExtractionProfile` autonomes. Elles désignent le même `ExtractionProfile` composé dans un bucket plus grand. Le suffixe `_128` / `_192` est descriptif.

| ExtractionProfile de base | Small | Medium | Large |
|---|---|---|---|
| `walk` | `walk` | `walk_128` | `walk_192` |
| `thrust` | `thrust` | — | `thrust_192` |
| `slash` | `slash` | `slash_128` | `slash_192` |

### Rows LPC canoniques (0-based) — bloc Small

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

### Blocs oversize — exemple trident (layout LPC Character)

| ExtractionProfile | grid_y (allocation) | frame_size (déduit) |
|---|---|---|
| `walk_128` | 54–61 | 128×128 |
| `thrust_192` | 62–73 | 192×192 |

**Note critique** : **dans le layout LPC Character**, le **bloc Small (`grid_y` 0–53) est strictement identique d'un PNG à l'autre**. **La zone des blocs oversize commence au `grid_y` 54.** Ces valeurs sont **spécifiques à ce layout**, pas une invariante générale du pipeline AOT.

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

> **Une `AnimationAction` peut correspondre à plusieurs cinématiques, selon le contexte. La résolution produit un `Profile` — `ExtractionProfile` ou `CompositionProfile`.**

### Structure conceptuelle

```
AnimationAction
├── default
├── by_equipment
├── by_bucket
└── fallback
```

**Note** : `by_bucket` est un axe de résolution à part entière. Une même action peut résoudre vers `walk` (Small), `walk_128` (Medium), `walk_192` (Large), indépendamment de l'équipement. Les entrées `walk_128` / `walk_192` désignent la variante physique de `walk` au bucket correspondant.

### Localisation dans le YAML de résolution

La table de résolution est **intégralement portée par le troisième YAML** (§4).

### Exemples illustratifs

```
attack → slash (défaut), thrust (spear/crossbow), shoot (bow)
walk → walk (Small), walk_128 (Medium), walk_192 (Large)
run → run
swim → swim (CompositionProfile, source spellcast — provisoire)
cast → spellcast
idle → idle
die → hurt
stagger → hurt
watering → watering (CompositionProfile, source thrust)
hoe → tool_hoe (CompositionProfile, source slash — à préciser)
shovel → tool_shovel (CompositionProfile, source slash — à préciser)
sit → sit
emote → emote
```

### Convergence

Plusieurs `AnimationAction` peuvent se résoudre vers le même `Profile` :

| AnimationAction | Profile |
|---|---|
| `die` | `hurt` (ExtractionProfile) |
| `stagger` | `hurt` (ExtractionProfile) |

### Matérialisation runtime

```
[LayerId][BucketId][AnimationAction][Direction][EquipmentId] → SequenceId
```

- Aucun parcours d'arbre au runtime.
- `ExtractionProfile` et `CompositionProfile` disparaissent du binaire final.
- `EquipmentId` est un **index**, pas un prédicat.
- **Seuls les couples `(EquipmentId, BucketId)` valides sont matérialisés** (§4).
- **La table est générée par l'AOT** à partir du YAML de résolution, du YAML canonique et des contrats d'équipement.

### Note terminologique

| Notion | Niveau | Nature |
|---|---|---|
| **Convergence** | Étage 1 → 2 | Plusieurs `AnimationAction` vers un même `Profile`. |
| **Composition ordinale** | Étage 2 → 3 | Réordonnancement, répétition. |
| **Déduplication** | Étage 3 (AOT) | Déduplication mécanique. |
| **Assemblage multi-couches** | Runtime | Superposition spatiale. |

---

## 9. Composition multi-couches, déduplication

### Composition dans le bucket cible

Pour chaque combinaison `(LayerId, AnimationAction, Direction, contexte de résolution)` :

1. Résoudre le `Profile` cible.
2. Identifier le `ExtractionProfile` sous-jacent (si `CompositionProfile`, remonter à `source_extraction`).
3. Sélectionner le **contrat applicable** `(applies_to, target_bucket)`.
4. Extraire les frames du PNG source à partir des `grid_y` de départ déclarés dans ce contrat, en appliquant la largeur verticale induite par `frame_size`.
5. **Composer la source dans le canvas du bucket cible** — sans redimensionnement :
   - toile vide `BucketSize × BucketSize` ;
   - offset centré : `offset = (BucketSize - source_size) / 2` ;
   - blitter la source à cet offset ;
   - padding transparent autour.
6. Si le layer n'a **pas** de variante : **frame transparente canonique**.
7. Composer la séquence finale.
8. Empaqueter.

**Rappel important** : la source n'est **jamais** redimensionnée. Un body 64×64 reste 64×64 ; il est simplement positionné dans un canvas plus grand.

**Note sur « contexte de résolution »** : ce terme désigne la dimension **conceptuelle** de résolution (par équipement, par bucket). Il **ne correspond pas** à une dimension runtime supplémentaire.

### Invariant de validation : `source_size ≤ BucketSize`

Ici :
- `source_size` désigne la dimension intrinsèque de la frame source extraite du layer ;
- `BucketSize` désigne la dimension de la frame finale consommée par le runtime ;
- `frame_size` est la dimension de build dérivée de `target_bucket` et correspond à `BucketSize`.

Sinon → **erreur de build**.

### Politique de bucket cible

Cible = maximum des buckets disponibles pour l'équipement actif. Sans équipement : **Small**.

### Fallback transparent vs erreur de build

| Situation | Traitement |
|---|---|
| Layer ne peut pas fournir une animation **requise** | **Erreur de build** |
| Layer n'a pas une animation **non requise** | **Frame transparente** injectée, longueur alignée |
| CompositionProfile dont la source manque | **Erreur de build** |
| `reference` manquante dans un contrat d'équipement | **Erreur de build** |
| Cible de résolution inexistante dans le YAML canonique | **Erreur de build** |
| Plusieurs contrats applicables au même asset/bucket | **Erreur de build (ambiguïté)** |
| Couple `(EquipmentId, BucketId)` invalide | **Absent de l'espace généré** |
| Animation canonique non référencée | **Warning informatif** |
| `AnimationAction` déclarée mais non référencée | **Warning informatif** |

### Traitement des overlays

Un effet visuel peut être traité de **deux manières** :

- **Layer séparé** si l'asset LPC est superposable.
- **Variante d'un layer existant** si l'asset remplace intégralement un layer.

**Règle de classification** : déterminée au build par inspection de l'asset.

### Déduplication et génération — trois notions distinctes

| Notion | Définition |
|---|---|
| **Scope de déduplication** | Au sein d'un même layer (ou variante de layer). |
| **Clé de génération AOT** | `(LayerId, BucketId, AnimationAction, Direction, EquipmentId)` — limitée aux couples valides. |
| **Clé d'adressage runtime** | La matrice finale `[LayerId][BucketId][AnimationAction][Direction][EquipmentId] → SequenceId`. |

**Anti-explosion combinatoire** : l'AOT **n'itère pas** sur les combinaisons de layers. Chaque layer est traité indépendamment.

**Déduplication** : après génération, les séquences identiques au sein d'un même layer partagent le même stockage.

**Chiffrage indicatif** : ordre de 60 000–80 000 séquences.

---

## 10. Frontière AOT / runtime

### Connaît
- `LayerId`, `BucketId`, `AnimationAction`, `Direction`, `EquipmentId`
- Identifiants et structures AOT déjà résolues
- Taille de frame résolue

**Précision** : le runtime utilise `BucketId` et `EquipmentId` comme **identifiants d'indexation**, mais ignore leur **sémantique** (pas de branche sur `if bucket == Large`), ainsi que les concepts de `row`, `grid_cell`, `grid`, `grid_y`.

### Ignore
- LPC, rows, grid_cells, grid, grid_y, PNG, `Profile`, `ExtractionProfile`, `CompositionProfile`
- Pivot d'ancrage gameplay
- Logique de résolution des contrats
- Interprétation des YAML

### Fait
- Écrit `AnimationAction` + `Direction`
- **Indexe par l'équipement** via `EquipmentId`, **sans branche conditionnelle**
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
- Chemins et identifiants en lowercase. §1

### Préfixes
- `AnimationAction` : jamais de préfixe. §1
- Profils : préfixe descriptif toléré. §1

### Profils
- `Profile` = terme générique englobant `ExtractionProfile` et `CompositionProfile`. §2
- Un `ExtractionProfile` possède une séquence canonique par défaut. §2
- `walk_128`, `walk_192`, `slash_128`, `slash_192`, `thrust_192` sont des **variantes physiques de bucket**, descriptives, pas des Profiles autonomes. §2
- `slash_reverse` est un `CompositionProfile` dérivé de `slash`. Sa variante physique Large est `slash_reverse_192` (à vérifier contre les YAML réels). §2

### Contextualisation des contrats
- L'AOT **ne fusionne jamais** les contrats. §4
- Un même `ExtractionProfile` peut apparaître dans plusieurs contrats, avec des `rows` propres à chaque bucket. §4
- Un `CompositionProfile` n'est **jamais** localisé physiquement : sa localisation est portée par son `source_extraction`. §4
- 0 contrat applicable → erreur ; 1 contrat → résolution normale ; >1 → erreur d'ambiguïté. §4

### Couples (EquipmentId, BucketId)
- Seuls les couples valides sont matérialisés. §4
- Pas de produit cartésien. §4
- Le runtime utilise les identifiants comme clés d'indexation, sans interpréter leur sémantique. §4

### Directions
- Directions canoniques `[top, left, down, right]`. §7
- Exceptions de source : `hurt` = `[down]`, `climb` = `[top]`. §7
- Les profils mono-directionnels sont dupliqués AOT sur les quatre directions canoniques. §7
- Variantes oversize : toujours 4 directions. §7

### Structure PNG
- Un PNG est une **grid** de grid_cells de 64×64. §3
- Blocs contigus, sans padding. §3
- **Dans le layout LPC Character**, le bloc Small (grid_y 0–53) est identique d'un PNG à l'autre. §3
- **Dans le layout LPC Character**, la zone des blocs oversize commence au grid_y 54. §3
- Sur un même PNG, variantes de taille d'un même profil partagent leurs grid_y. §3
- Les valeurs du layout LPC Character ne sont **pas** des invariantes générales du pipeline AOT. §3

### Alignement et dimensions
- Toute frame centrée sur le centre géométrique. §3
- `source_size ≤ BucketSize` (erreur sinon). §9
- `source_size` désigne la dimension intrinsèque de la frame source ; `BucketSize` la dimension de la frame finale ; `frame_size` dérive de `target_bucket`. §9
- **Origine physique par défaut : `x = 0`.** §3

### Composition dans le bucket
- Un layer n'est **jamais redimensionné** pour le bucket. §3, §9
- Un layer est **composé** dans le canvas du bucket cible (positionnement centré, padding transparent). §3, §9

### Bucket
- Trois familles : corporel / arme / outil. §3
- Promotion automatique (équipement = arme ou outil). §3
- Sans équipement : Small. §3

### Équipement
- `EquipmentId` identifie l'arme ou l'outil actif. §4
- L'équipement pilote la cinématique du corps. §3
- Bouclier surajouté, sans impact sur la cinématique. §3
- Jamais de dual wield. §3
- Bouclier : position variable selon direction. §11 (voir ci-dessous)

### YAML et validation
- **Trois YAML distincts** : canonique + équipement + résolution. §4
- `frame_size` déduit de `target_bucket`. §4
- CompositionProfiles héritent `frame_count` et `directions` de leur source. §4
- Référence manquante → erreur. Cible de résolution inexistante → erreur. Ambiguïté → erreur. Éléments déclarés mais non référencés → warnings. §4
- `tool_whip` et `tool_axe` dérivent de `slash`. §4
- Le YAML canonique est une **déclaration d'intentions**, pas une description physique. §12

### Politique de boucle
- Résolue AOT par graphe de succession (`next_sequence_id`). §6
- Runtime ignore la nature du bouclage (loop vs transition). §6

### Déduplication et génération
- Déduplication par layer. §9
- Génération par combinaison `(LayerId, BucketId, AnimationAction, Direction, EquipmentId)` — couples valides uniquement. §9
- Pas de combinaison multi-layers matérialisée. §9

### Fallback
- Requis manquant → erreur. Non-requis absent → transparent. §9
- CompositionProfile source manquante → erreur. §9

### Overlays
- Layer séparé ou variante, selon asset. §9

### Écartés
- `1h_slash`, `1h_backslash`, `1h_halfslash`, `backslash`, `halfslash`, `combat_idle` : écartés. §7
- Écartés par héritage : `backslash_128`, `halfslash_128`. §7
- Nomenclature : `_128`, `_192`, lowercase. §5

### Exhaustivité
- Le document n'est pas exhaustif : les YAML le sont. §1, §7

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

**Note sur les trois positions d'équipement** : composants visuels distincts d'un même équipement, résolus depuis le **même `EquipmentId`**.

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
| **PNG** | Vérité des pixels, `grid_y`, frames | **Autoritaire absolue** |
| **YAML canonique** | Déclaration d'intentions d'animation | Autoritaire pour nos intentions |
| **YAML d'équipement** | Déclaration du mapping PNG ↔ équipement | Autoritaire pour nos intentions |
| **YAML de résolution** | Déclaration du mapping `AnimationAction` ↔ `Profile` | Autoritaire pour nos intentions |
| **Liste des assets au build** | Vérité d'existence | Autoritaire |

Règle : `Asset utilisé = déclaré au build ∩ présent sur disque ∩ déclaré en YAML`. Toute divergence = **erreur de build**.

**Précision sur la portée PNG vs YAML** :
- Le PNG dit **où sont physiquement les pixels** (`grid_y`, grid_cells).
- Le YAML canonique dit **ce que nous voulons** que les animations soient.
- Le YAML d'équipement dit **où chercher les pixels** (en rows logiques) pour un équipement donné.
- Si le YAML contredit le PNG, **erreur de build**.

**Validation croisée :**
- `reference` manquante → **erreur de build**.
- Cible de résolution inexistante → **erreur de build**.
- Ambiguïté de contrat → **erreur de build**.
- Animation canonique non référencée → **warning informatif**.
- `AnimationAction` déclarée mais non référencée par une entrée de résolution → **warning informatif**.

---

## 13. Points ouverts

### Vérifié (empirique)

- ✔ Rows LPC canoniques (0-based).
- ✔ Bloc Small (grid_y 0–53) identique d'un PNG à l'autre, dans le layout LPC Character.
- ✔ Zone des blocs oversize à partir du grid_y 54, dans le layout LPC Character.
- ✔ `1h_backslash` = rows 50–53.
- ✔ `walk_128` = 9 frames.
- ✔ `tool_whip` et `tool_axe` dérivent de `slash`.
- ✔ Alignement par centre géométrique, validé visuellement.
- ✔ Ordre d'empilement canonique, validé visuellement.
- ✔ Exception bouclier selon direction, validée visuellement.

### Décidé (design)

- ✔ Quatre unités : `row`, `grid_cell`, `grid`, `grid_y`.
- ✔ `Profile` comme terme générique.
- ✔ `ExtractionProfile` comme terme pour les profils d'extraction directe.
- ✔ Indexation 0-based.
- ✔ Chemins et identifiants en lowercase.
- ✔ Préfixes interdits pour `AnimationAction`, tolérés pour profils.
- ✔ `EquipmentId` — arme ou outil.
- ✔ `by_equipment` partout.
- ✔ Pivot d'ancrage gameplay hors-scope AOT.
- ✔ Nomenclature uniformisée `_128` / `_192`, lowercase.
- ✔ Écartés : `1h_*`, `backslash`, `halfslash`, + héritage.
- ✔ Trois familles de layers.
- ✔ Promotion automatique de bucket.
- ✔ Politique de boucle par graphe de succession.
- ✔ Déduplication par layer.
- ✔ Trois YAML distincts.
- ✔ `frame_size` déduit de `target_bucket`.
- ✔ PNG source de vérité physique unique.
- ✔ Terme générique « Oversized Equipment ».
- ✔ Contextualisation des contrats : sélection, jamais fusion.
- ✔ `aliases` non utilisé pour `hurt`.
- ✔ **Variantes physiques de bucket** : `walk_128` et consorts sont descriptives, pas des Profiles.
- ✔ **`slash_reverse`** : modélisé comme `CompositionProfile` dérivé de `slash`.
- ✔ **Composition dans le bucket cible** : pas de scaling.
- ✔ **Layout LPC Character** : qualifié comme spécifique.
- ✔ **Origine `x = 0`** : par défaut, sans mécanisme de surcharge.
- ✔ **Couples `(EquipmentId, BucketId)`** : valides uniquement.

### Réserves d'arbitrage avant v1

Les points suivants restent volontairement **non tranchés** dans cette version :

- **Résolution multi-axes** : priorité ou exclusivité entre `by_equipment`, `by_bucket` et `default` (§8).
- **Variantes physiques de bucket** : statut nominal et représentation exacte de `walk_128`, `walk_192`, etc. (§2, §8).
- **Animations requises** : définition de la notion de `required` et éventuel `required_animations` (§9).
- **`EquipmentId`** : pertinence du nom actuel vis-à-vis du périmètre réel du concept, notamment en présence du bouclier (§3, §4, §11).
- **Déduplication** : identité exacte d'une séquence lorsqu'elle porte également une topologie de succession via `next_sequence_id` (§6, §9).
- **Frame transparente canonique** : nature exacte de la représentation associée à l'index global `0` pour les différents buckets (§4, §9).

### Notes de veille (non des tickets ouverts)

- **B11 — CompositionProfiles multi-source** : non retenu. À reconsidérer si besoin concret.
- **`swim`** : CompositionProfile dérivé de `spellcast`, marqué **provisoire**.
- **`fallback`** : clé réservée dans le YAML de résolution ; sémantique à figer Phase 5.
- **`slash_reverse_192`** : chaîne conceptuelle `slash → slash_reverse → variante physique Large` à **vérifier contre les YAML réels**.

### Tickets conceptuels ouverts

- (aucun)

> Les **réserves d'arbitrage** ci-dessus sont intentionnellement distinctes des tickets conceptuels : elles représentent des décisions de vocabulaire ou de contrat encore en discussion, sans constituer à ce stade de nouvelles fonctionnalités ou extensions du paradigme.

---

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
- **Exhaustivité sur les animations.**
- **Champ `frame_size` dans les YAML** — déduit de `target_bucket`.
- **Champ `align` dans les YAML** — centrage par défaut.
- **CompositionProfiles multi-source** — en veille.
- **Sources de vérité externes** — écartées.
- **Terme « rangée de cellules »** — remplacé par `grid_y`.
- **Terme `cell`** — remplacé par `grid_cell`.
- **Runtime-only pour les `AnimationAction`** — écarté.
- **`WeaponId`** — remplacé par `EquipmentId` ; le nom `EquipmentId` reste soumis à arbitrage lexical (§13).
- **`StrokeProfile`** — remplacé par `ExtractionProfile`.
- **Zone « héritée 21–53 »** — remplacée par « bloc Small 0–53 ».
- **Terme `MotorAction`** — supprimé du document.
- **`by_weapon`** — remplacé par `by_equipment`.
- **`aliases: [death]` sur `hurt`** — retiré.
- **Convention PascalCase pour les chemins** — écartée.
- **Fusion de contrats d'équipement** — écartée (sélection, pas fusion).
- **Redimensionnement (scaling) d'un layer** — écarté (composition dans le canvas uniquement).
- **Troisième type de `Profile`** — écarté (les variantes physiques de bucket sont descriptives).
- **Surcharge de l'origine physique `x = 0`** — écartée (par défaut, sans mécanisme).

---

## 15. Synthèse en une phrase

> **Le pipeline fournit au moteur un contrat strict : les PNG LPC et leurs conventions sont une source absorbée AOT ; le runtime ne manipule que des actions, des directions, des identifiants de layer, de bucket, d'équipement et de séquence, et applique une topologie de succession figée (`next_sequence_id`), sans jamais connaître ni LPC, ni les `Profile`, ni les pixels, ni la nature du bouclage.**

---

_Document révisé le 19 septembre 2026_
