# Document fondateur conceptuel — LPC (pipeline AOT) — v21

> **Statut** : brouillon / schéma directeur — à auditer avant finalisation.
> **Portée** : ce document couvre le **pipeline AOT**. Le runtime n'est mentionné que pour fixer les **invariants de frontière**.
> **Exhaustivité** : ce document **n'est pas exhaustif** sur les animations. Il pose le **principe**, des **exemples illustratifs** et des **invariants**. La liste complète relève des **YAML**.

---

## 1. Portée et principes

Ce document établit le **paradigme** qui gouverne la transformation des spritesheets LPC (PNG) en données consommables par un runtime ECS/DOD.

### Principes fondateurs

1. **LPC est une banque de pixels, pas un format d'animation.**
2. **Toute complexité est absorbée AOT.**
3. **Le runtime ne manipule que des identifiants.**
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

**Important** : les YAML déclarent toujours en **rows logiques** (une par direction). La conversion rows logiques → `grid_y` est absorbée par le pipeline AOT.

### Convention de vocabulaire — préfixes

- **`AnimationAction`** : **jamais de préfixe de catégorie**.
- **Profils (`StrokeProfile`, `CompositionProfile`)** : **préfixe descriptif toléré** (ex. `tool_whip`).
- Aucun préfixe n'est **jamais** porté par une `AnimationAction`.

### Convention de vocabulaire — chemins et identifiants

**Chemins, noms de fichiers et identifiants sont en lowercase.** Aucune convention PascalCase n'est retenue, par cohérence avec les conventions LPC et pour éviter les ambiguïtés de casse dans un pipeline déterministe.

---

## 2. Architecture à trois étages

```
[ÉTAGE 1]   AnimationAction      ← ce que le runtime connaît
                │
                │  résolution AOT (équipement, bucket, contexte)
                ▼
[ÉTAGE 2]   Profile              ← ce que les PNG fournissent / composent
            ├── StrokeProfile        (extraction)
            └── CompositionProfile   (composition)
                │
                │  extraction + composition AOT
                ▼
[ÉTAGE 3]   FrameSequence        ← ce que le runtime consomme
```

### Étage 1 — AnimationAction

C'est le langage du gameplay. C'est ce que les systèmes écrivent, et la **seule** chose que le runtime connaît.

Exemples (liste **illustrative**, non exhaustive, **sans préfixes**) :
`attack`, `cast`, `walk`, `run`, `swim`, `idle`, `die`, `stagger`, `watering`, `sit`, `emote`…

- Indépendant de LPC, de l'équipement, de la taille des sprites.
- Ne dit **rien** sur la cinématique ni sur les pixels.

### Étage 2 — Profile, StrokeProfile, CompositionProfile

**`Profile`** est le terme **générique** englobant les deux sous-types :

```
Profile
├── StrokeProfile
└── CompositionProfile
```

- **`StrokeProfile`** — profil d'**extraction**. Décrit ce que les PNG fournissent : `frame_count`, `directions`, et une séquence canonique par défaut (`sequence`, optionnelle).
- **`CompositionProfile`** — profil de **composition**. Décrit comment les frames extraites sont ordonnées en séquence. **Hérite `frame_count` et `directions` de son `source_stroke`**.

**Précision sur `rows`** :
- `rows` **n'est pas une propriété intrinsèque** d'un `StrokeProfile`.
- `rows` est fourni par le **contrat d'équipement**, qui localise le `StrokeProfile` dans un PNG donné.
- Un même `StrokeProfile` peut être localisé à des `rows` différentes selon le contrat et le bucket.

**Distinction critique — `frame_count` source vs longueur de séquence jouée** :

- `frame_count` désigne la taille du **pool extrait**.
- `len(sequence)` désigne le nombre de frames **effectivement jouées**.
- Ces deux valeurs peuvent être différentes : `watering` a `frame_count = 8` (pool) et `len(sequence) = 7` (jouées).

**Exemples** :

| Nom | Type | Source | Remarque |
|---|---|---|---|
| `slash`, `thrust`, `shoot`, `spellcast`, `walk`, `run`, `jump`, `climb`, `idle`, `sit`, `hurt`, `emote`… | StrokeProfile | — | Extraction directe |
| `watering` | CompositionProfile | `thrust` | Séquence `[0,1,4,4,4,4,5]` |
| `tool_whip` | CompositionProfile | `slash` | Séquence `[0,1,2,3,4,5]` |
| `tool_axe` | CompositionProfile | `slash` | Séquence `[5,5,4,4,3,1,0,0,0,0]` |
| `swim` | CompositionProfile | `spellcast` | Provisoire (voir §13) |

### Étage 3 — FrameSequence

Séquence linéaire, **immuable dans son contenu** (ordre des frames figé), produite exclusivement AOT.

- Ne contient **aucun pixel**.
- Référence des frames stockées dans des structures globales.
- Le runtime n'itère que sur des **indices et offsets**.
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

- Bucket **orthogonal** au StrokeProfile.
- Toutes les frames d'un bucket ont même taille, même **centre géométrique d'alignement AOT**.
- Grid_cells hétérogènes **ajoutées en fin de PNG**, par ordre croissant.
- **Armes et outils uniquement** : les PNG > 64×64 sont réservés aux **variantes oversized d'équipement**.
- Corps, cheveux, vêtements, armures, bouclier : **toujours** 64×64.
- Les layers non-équipement sont **reprojetés** vers le bucket cible imposé par l'équipement.

### Trois familles de layers

| Famille | Exemples | Buckets possibles | Rôle |
|---|---|---|---|
| **Corporel** | corps, cheveux, vêtements, armures, bouclier | Small uniquement | Reprojeté vers le bucket cible |
| **Arme** | épée, arc, bâton… | Small / Medium / Large | Détermine le bucket cible |
| **Outil** | pioche, hache, fouet… | Small / Medium / Large | Détermine le bucket cible, comme une arme |

**Note sur le bouclier** : classé **corporel** (jamais pilote du bucket), mais sa **position d'empilement varie selon la direction** (§11).

### Promotion automatique de bucket

**Le bucket cible est déterminé par le maximum des buckets disponibles pour l'équipement actif.**

- Sans équipement : **Small par défaut**.
- Si le bucket de base est Small mais que l'équipement n'a qu'une variante Large : **promotion automatique vers Large**.
- **La promotion s'applique identiquement aux armes et aux outils.**

**Précision sur la nature de la promotion** : la promotion est une **déduction** (`bucket_cible = max(buckets_disponibles_de_l'équipement)`). Elle n'entraîne **pas** de transformation effective des assets : les layers non-équipement sont reprojetés vers ce bucket cible via le mécanisme standard de reprojection (§9).

### Alignement des frames hétérogènes

**Toute frame source, quelle que soit sa taille, est centrée sur un point d'ancrage commun avant reprojection.**

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

**Structure monotone des PNG étendus :**

- **Le bloc Small (grid_y 0–53) est strictement identique d'un PNG à l'autre.** C'est l'**héritage LPC commun**.
- Les `grid_y` sont **monotones de 0 à 53** pour le bloc Small.
- **La zone des blocs oversize commence au `grid_y` 54.** Chaque bloc oversize occupe ensuite ses propres `grid_y`, dans l'ordre d'apparition.
- **Sur un même PNG, les variantes de taille d'un même profil partagent les mêmes `grid_y`.**

**Structure physique du PNG** :

```
grid_y 0 ──────────────── 53 │ 54 ────────────────>
         Bloc Small          │    Blocs oversize
         64×64 (héritage)    │    Medium / Large
```

**Les blocs sont contigus, sans padding.** Le YAML est la seule source qui identifie où commence et où finit chaque bloc dans la grid.

**Précision importante** : les rows logiques (0–53) couvrent **l'ensemble du bloc Small**, mais chaque animation n'occupe qu'une plage restreinte de rows dans ce bloc. La frontière `0–53` est **structurelle** ; les frontières entre animations sont **sémantiques**.

### Extensibilité à d'autres gabarits

- **Buckets** configurables.
- **Grid_cell de base** configurable par corpus (`64` pour LPC humanoïde).
- **Frame sizes** : tout multiple entier de la grid_cell de base.
- **StrokeProfiles** extensibles.
- **Aucune convention implicite** : chaque YAML déclare explicitement ses valeurs.

**Cas particulier — créatures à animation unique** : 1 direction, 1 frame, sans séquence custom.

---

## 4. Extraction, Composition, et propriétés AOT

### Extraction

**Ce qu'on lit** dans le PNG :

- quelles **rows** logiques LPC (déclarées par le contrat d'équipement),
- quel **nombre de frames source** (`frame_count`, déclaré par le YAML canonique),
- quel **bucket cible** (déclaré par le contrat d'équipement).

### Composition

**Ce qu'on joue** à partir de ce pool :

- une **séquence ordonnée** d'indices,
- qui peut **répéter**, **sauter**, **réordonner**, ou toute **combinaison**.

### Trois types de YAML

**1. YAML canonique d'animations** — un seul, liste **toutes** les animations du corpus. Il représente la **déclaration d'intentions d'animation**.

Chaque entrée porte :
- `name`, `type` (`StrokeProfile` par défaut, ou `CompositionProfile`),
- `frame_count`, `directions`,
- `sequence` (optionnel),
- `source_stroke` (CompositionProfile uniquement),
- `aliases` (optionnel — liste de synonymes historiques).

**2. YAML d'équipement** — localise les `StrokeProfile` dans un PNG donné.

Chaque entrée porte :
- `contract_id`, `applies_to` (`path_prefix` + `asset_whitelist` optionnelle),
- `target_bucket`,
- `stroke_profiles` (dictionnaire `reference` + `rows`).

> **Note sur `stroke_profiles`** : la clé reste `stroke_profiles` car le contrat d'équipement ne référence **que des sources physiques**. Un `CompositionProfile` n'a pas de localisation physique propre — il dérive d'un `StrokeProfile` localisé ailleurs.

**3. YAML de résolution `AnimationAction`** (ex. `action_resolution.yaml`) — **contrat de liaison** entre le domaine Gameplay et le domaine Moteur.

Chaque entrée porte :
- `animation_action` — l'action déclenchée par le gameplay,
- `resolution` — table de résolution vers les profils (voir §8).

**Répartition des responsabilités :**

| Champ | Canonique | Équipement | Résolution |
|---|---|---|---|
| `name`, `type` | ✔ | — | — |
| `frame_count`, `directions` | ✔ (StrokeProfile) | — | — |
| `sequence` | ✔ | — | — |
| `source_stroke` | ✔ (CompositionProfiles) | — | — |
| `aliases` | ✔ | — | — |
| `reference`, `rows` | — | ✔ | — |
| `target_bucket` | — | ✔ | — |
| `animation_action` | — | — | ✔ |
| `resolution` | — | — | ✔ |

### `frame_size` déduit du bucket

**`frame_size` n'est jamais déclaré** dans les YAML. Il est déduit de `target_bucket` au build : `Small` → 64, `Medium` → 128, `Large` → 192.

### Héritage des CompositionProfiles

Un **CompositionProfile** hérite `frame_count` et `directions` de son `source_stroke`.

Exemple simplifié :

```yaml
- name: watering
  type: CompositionProfile
  source_stroke: thrust
  sequence: [0, 1, 4, 4, 4, 4, 5]
```

### Invariant de contextualisation des contrats

**Le pipeline AOT ne fusionne jamais les contrats d'équipement.** Il **sélectionne** le contrat applicable au contexte, puis résout la référence dans ce contrat.

Formellement :

```
(Profile, applicable Contract) → physical rows
```

Le contrat est lui-même sélectionné par :

```
(applies_to, target_bucket) → applicable Contract
```

**Conséquences** :

- Un même `StrokeProfile` (ex. `slash`) peut être déclaré dans **plusieurs contrats**, avec des `rows` propres à chaque bucket.
- L'AOT ne produit **jamais** de contrat fusionné.
- La chaîne de résolution est :

```
AnimationAction + EquipmentId + BucketId
        ↓
   Profile (StrokeProfile ou CompositionProfile)
        ↓
   source_stroke éventuel
        ↓
   StrokeProfile
        ↓
   contrat applicable (applies_to, target_bucket)
        ↓
   rows physiques
        ↓
   frames
```

**Distinction clé — `CompositionProfile` n'est jamais localisé physiquement.** Un `CompositionProfile` est une **identité de composition/résolution**. La localisation physique est toujours portée par son `StrokeProfile` source.

**Invariant d'unicité de contrat** :

| Nombre de contrats applicables | Traitement |
|---|---|
| 0 | **Erreur de build** |
| 1 | Résolution normale |
| >1 | **Erreur de build par ambiguïté** |

Cette règle évite toute fusion implicite ou priorité cachée entre contrats.

### Contrat YAML d'équipement — exemple idiomatique

```yaml
contract_id: "lpc_base_64"

applies_to:
  path_prefix: "textures/characters/"

target_bucket: "Small"

stroke_profiles:
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

**Règle 2 — résolution manquante dans le YAML de résolution → erreur de build.**

**Règle 3 — animation canonique non référencée → warning informatif.**

**Règle 4 — `AnimationAction` sans résolution → warning informatif.**

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
| Exception `hurt` | `[down]` |
| Exception `climb` | `[top]` |
| Variantes oversize | **toujours 4 directions** |

### StrokeProfiles canoniques

| StrokeProfile | directions | frame_count | sequence |
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

**Note sur la `sequence` d'un StrokeProfile** : séquence canonique par défaut, utilisée quand aucun `CompositionProfile` ne dérive de lui.

### CompositionProfiles (exemples)

| CompositionProfile | source_stroke | sequence |
|---|---|---|
| `watering` | `thrust` | `[0,1,4,4,4,4,5]` |
| `tool_whip` | `slash` | `[0,1,2,3,4,5]` |
| `tool_axe` | `slash` | `[5,5,4,4,3,1,0,0,0,0]` |
| `tool_hoe` | `slash` | à préciser (YAML) |
| `tool_shovel` | `slash` | à préciser (YAML) |
| `swim` | `spellcast` | à préciser (provisoire, voir §13) |

### Variantes Oversized Equipment

| Profile de base | Small | Medium | Large |
|---|---|---|---|
| `walk` | `walk` | `walk_128` | `walk_192` |
| `thrust` | `thrust` | — | `thrust_192` |
| `slash` | `slash` | `slash_128` | `slash_192` |
| `slash_reverse` | — | — | `slash_reverse_192` |

### Rows LPC canoniques (0-based) — bloc Small

| StrokeProfile | rows (logiques) | Nb rows |
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

### Blocs oversize — exemple trident

| StrokeProfile | grid_y (allocation) | frame_size (déduit) |
|---|---|---|
| `walk_128` | 54–61 | 128×128 |
| `thrust_192` | 62–73 | 192×192 |

**Note critique** : le **bloc Small (`grid_y` 0–53) est strictement identique d'un PNG à l'autre**. **La zone des blocs oversize commence au `grid_y` 54.**

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

**Invariant de sélection** : un StrokeProfile n'est canonique que si tous les layers essentiels le supportent (whitelist empirique).

**Invariant d'héritage** : un CompositionProfile dérivé d'un StrokeProfile **écarté** est **écarté par construction**.

---

## 8. Table de résolution `AnimationAction → Profile`

**Artefact central du paradigme.**

> **Une `AnimationAction` peut correspondre à plusieurs cinématiques, selon le contexte. La résolution produit un `Profile` — `StrokeProfile` ou `CompositionProfile`.**

### Structure conceptuelle

```
AnimationAction
├── default
├── by_equipment
├── by_bucket
└── fallback
```

**Note** : `by_bucket` est un axe de résolution à part entière. Une même action peut résoudre vers `walk` (Small), `walk_128` (Medium), `walk_192` (Large), indépendamment de l'équipement.

### Localisation dans le YAML de résolution

La table de résolution est **intégralement portée par le troisième YAML** (§4). Chaque entrée associe une `AnimationAction` à sa résolution.

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
| `die` | `hurt` (StrokeProfile) |
| `stagger` | `hurt` (StrokeProfile) |

### Matérialisation runtime

```
[LayerId][BucketId][AnimationAction][Direction][EquipmentId] → SequenceId
```

- Aucun parcours d'arbre au runtime.
- `StrokeProfile` et `CompositionProfile` disparaissent du binaire final.
- `EquipmentId` est un **index**, pas un prédicat.
- **La table est générée par l'AOT** à partir du YAML de résolution, du YAML canonique et des contrats d'équipement.

### Note terminologique

| Notion | Niveau | Nature |
|---|---|---|
| **Convergence** | Étage 1 → 2 | Plusieurs `AnimationAction` vers un même `Profile`. |
| **Composition de séquence** | Étage 2 → 3 | Réordonnancement, répétition. |
| **Interning** | Étage 3 (AOT) | Déduplication mécanique. |
| **Assemblage multi-couches** | Runtime | Superposition spatiale. |

---

## 9. Reprojection, composition multi-couches, interning

### Reprojection

Pour chaque combinaison `(LayerId, AnimationAction, Direction, contexte de résolution)` :

1. Résoudre le `Profile` cible.
2. Identifier le `StrokeProfile` sous-jacent (si `CompositionProfile`, remonter à `source_stroke`).
3. Sélectionner le **contrat applicable** `(applies_to, target_bucket)`.
4. Extraire les rows du PNG source selon les `rows` déclarées dans ce contrat.
5. Reprojeter dans le bucket cible (toile vide, offset centré, padding transparent).
6. Si le layer n'a **pas** de variante : **frame transparente canonique**.
7. Composer la séquence finale.
8. Empaqueter.

**Note sur « contexte de résolution »** : ce terme désigne la dimension **conceptuelle** de résolution (par équipement, par bucket). Il **ne correspond pas** à une dimension runtime supplémentaire.

### Invariant de validation : `source_size ≤ BucketSize`

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
| Résolution cible manquante dans le YAML de résolution | **Erreur de build** |
| Plusieurs contrats applicables au même asset/bucket | **Erreur de build (ambiguïté)** |
| Animation canonique non référencée | **Warning informatif** |
| `AnimationAction` sans résolution | **Warning informatif** |

### Traitement des overlays

Un effet visuel peut être traité de **deux manières** :

- **Layer séparé** si l'asset LPC est superposable.
- **Variante d'un layer existant** si l'asset remplace intégralement un layer.

**Règle de classification** : déterminée au build par inspection de l'asset.

### Interning et génération — trois notions distinctes

| Notion | Définition |
|---|---|
| **Scope de déduplication** | Au sein d'un même layer (ou variante de layer). |
| **Clé de génération AOT** | `(LayerId, BucketId, AnimationAction, Direction, EquipmentId)`. |
| **Clé d'adressage runtime** | La matrice finale `[LayerId][BucketId][AnimationAction][Direction][EquipmentId] → SequenceId`. |

**Anti-explosion combinatoire** : l'AOT **n'itère pas** sur les combinaisons de layers. Chaque layer est traité indépendamment.

**Interning** : après génération, les séquences identiques au sein d'un même layer partagent le même stockage.

**Chiffrage indicatif** : ordre de 60 000–80 000 séquences.

---

## 10. Frontière AOT / runtime

### Connaît
- `LayerId`, `BucketId`, `AnimationAction`, `Direction`, `EquipmentId`
- Taille de frame résolue

**Précision** : le runtime utilise `BucketId` et `EquipmentId` comme **identifiants d'indexation**, mais ignore leur **sémantique** (pas de branche sur `if bucket == Large`), ainsi que les concepts de `row`, `grid_cell`, `grid`, `grid_y`.

### Ignore
- LPC, rows, grid_cells, grid, grid_y, PNG, `Profile`, `StrokeProfile`, `CompositionProfile`
- Pivot d'ancrage gameplay

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

L'AOT fige la topologie spatiale et ordinale. Le runtime est maître du domaine temporel.

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
- `Profile` = terme générique englobant `StrokeProfile` et `CompositionProfile`. §2
- Un `StrokeProfile` possède une séquence canonique par défaut. §2

### Contextualisation des contrats
- L'AOT **ne fusionne jamais** les contrats. §4
- Un même `StrokeProfile` peut apparaître dans plusieurs contrats, avec des `rows` propres à chaque bucket. §4
- Un `CompositionProfile` n'est **jamais** localisé physiquement : sa localisation est portée par son `source_stroke`. §4
- 0 contrat applicable → erreur ; 1 contrat → résolution normale ; >1 → erreur d'ambiguïté. §4

### Directions
- Directions canoniques `[top, left, down, right]`. §7
- Exceptions : `hurt` = `[down]`, `climb` = `[top]`. §7
- Variantes oversize : toujours 4 directions. §7

### Structure PNG
- Un PNG est une **grid** de grid_cells de 64×64. §3
- Blocs contigus, sans padding. §3
- Bloc Small (grid_y 0–53) identique d'un PNG à l'autre. §3
- Zone des blocs oversize à partir du grid_y 54. §3
- Sur un même PNG, variantes de taille d'un même profil partagent leurs grid_y. §3

### Alignement et dimensions
- Toute frame centrée sur le centre géométrique. §3
- `source_size ≤ BucketSize` (erreur sinon). §9
- Frames commencent à x = 0. §3

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
- Référence manquante → erreur. Résolution manquante → erreur. Ambiguïté → erreur. Fantômes → warnings. §4
- `tool_whip` et `tool_axe` dérivent de `slash`. §4
- Le YAML canonique est une **déclaration d'intentions**, pas une description physique. §12

### Politique de boucle
- Résolue AOT par graphe de succession (`next_sequence_id`). §6
- Runtime ignore la nature du bouclage (loop vs transition). §6

### Interning et génération
- Interning par layer. §9
- Génération par combinaison `(LayerId, BucketId, AnimationAction, Direction, EquipmentId)`. §9
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
15. weapon_behind
16. weapon
17. shield (position variable selon direction)
18. cape_front
19. weapon_front
20. overlays
```

**Note sur les trois positions d'arme** : composants visuels distincts d'une même arme, résolus depuis le **même `EquipmentId`**.

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
| **YAML d'équipement** | Liaison PNG ↔ équipement | Autoritaire pour nos intentions |
| **YAML de résolution** | Liaison `AnimationAction` ↔ `Profile` | Autoritaire pour nos intentions |
| **Liste des assets au build** | Vérité d'existence | Autoritaire |

Règle : `Asset utilisé = déclaré au build ∩ présent sur disque ∩ déclaré en YAML`. Toute divergence = **erreur de build**.

**Précision sur la portée PNG vs YAML** :
- Le PNG dit **où sont physiquement les pixels** (`grid_y`, grid_cells).
- Le YAML canonique dit **ce que nous voulons** que les animations soient.
- Le YAML d'équipement dit **où chercher les pixels** (en rows logiques) pour un équipement donné.
- Si le YAML contredit le PNG, **erreur de build**.

**Validation croisée :**
- `reference` manquante → **erreur de build**.
- Résolution cible manquante → **erreur de build**.
- Ambiguïté de contrat → **erreur de build**.
- Animation canonique non référencée → **warning informatif**.
- `AnimationAction` sans résolution → **warning informatif**.

---

## 13. Points ouverts

### Vérifié (empirique)

- ✔ Rows LPC canoniques (0-based).
- ✔ Bloc Small (grid_y 0–53) identique d'un PNG à l'autre.
- ✔ Zone des blocs oversize à partir du grid_y 54.
- ✔ `1h_backslash` = rows 50–53.
- ✔ `walk_128` = 9 frames.
- ✔ `tool_whip` et `tool_axe` dérivent de `slash`.
- ✔ Alignement par centre géométrique, validé visuellement.
- ✔ Ordre d'empilement canonique, validé visuellement.
- ✔ Exception bouclier selon direction, validée visuellement.

### Décidé (design)

- ✔ Quatre unités : `row`, `grid_cell`, `grid`, `grid_y`.
- ✔ `Profile` comme terme générique.
- ✔ Indexation 0-based.
- ✔ Chemins et identifiants en lowercase.
- ✔ Préfixes interdits pour `AnimationAction`, tolérés pour profils.
- ✔ `EquipmentId` — arme ou outil.
- ✔ `by_equipment` partout (remplace `by_weapon`).
- ✔ Pivot d'ancrage gameplay hors-scope AOT.
- ✔ Nomenclature uniformisée `_128` / `_192`, lowercase.
- ✔ Écartés : `1h_*`, `backslash`, `halfslash`, + héritage. Terme unique : « écarté ».
- ✔ Trois familles de layers.
- ✔ Promotion automatique de bucket.
- ✔ Politique de boucle par graphe de succession.
- ✔ Interning par layer.
- ✔ Trois YAML distincts.
- ✔ `frame_size` déduit de `target_bucket`.
- ✔ PNG source de vérité physique unique.
- ✔ Terme générique « Oversized Equipment ».
- ✔ **Contextualisation des contrats** : sélection, jamais fusion. Ambiguïté = erreur.
- ✔ **`aliases` non utilisé pour `hurt`** : seul `hurt` est retenu.

### Notes de veille (non des tickets ouverts)

- **B11 — CompositionProfiles multi-source** : non retenu. À reconsidérer si besoin concret.
- **`swim`** : CompositionProfile dérivé de `spellcast`, marqué **provisoire**.
- **`fallback`** : clé réservée dans le YAML de résolution ; sémantique à figer Phase 5.

### Tickets conceptuels ouverts

- (aucun)

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
- **`WeaponId`** — remplacé par `EquipmentId`.
- **Zone « héritée 21–53 »** — remplacée par « bloc Small 0–53 ».
- **Terme `MotorAction`** — supprimé du document.
- **`by_weapon`** — remplacé par `by_equipment`.
- **`aliases: [death]` sur `hurt`** — retiré (legacy non conservé).
- **Convention PascalCase pour les chemins** — écartée (lowercase partout).
- **Fusion de contrats d'équipement** — écartée (sélection, pas fusion).

---

## 15. Synthèse en une phrase

> **Le pipeline fournit au moteur un contrat strict : les PNG LPC et leurs conventions sont une source absorbée AOT ; le runtime ne manipule que des actions, des directions, des identifiants de layer, de bucket, d'équipement et de séquence, et applique une topologie de succession figée (`next_sequence_id`), sans jamais connaître ni LPC, ni les `Profile`, ni les pixels, ni la nature du bouclage.**

---

_Document rédigé le 15 septembre 2026_
